use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use bevy::{math::DVec3, prelude::*};
use flate2::read::GzDecoder;

pub use tesseract_base::persistence::*;
use tesseract_base::{actor, level};
use tesseract_java_protocol::types::{Biome, BitStorage, PalettedContainer};

use crate::{registry, replication};

/// Support for Minecraft: Java Edition persistence
pub struct PersistencePlugin(pub HashMap<String, PathBuf>);

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let levels = self.0.clone();
        let spawn_levels = move |mut commands: Commands| {
            for (level_name, level_path) in levels.iter() {
                let savegame_level = {
                    let mut data = vec![];
                    GzDecoder::new(File::open(level_path.join("level.dat")).unwrap())
                        .read_to_end(&mut data)
                        .unwrap();
                    tesseract_nbt::de::from_slice::<tesseract_java_savegame::level::Level>(
                        &mut data.as_slice(),
                    )
                    .unwrap()
                }
                .data;

                commands.spawn((
                    level::LevelBundle {
                        base: level::Base::new(level_name.clone(), level_name.clone()),
                        age_and_time: level::AgeAndTime {
                            age: savegame_level.time as u64,
                            time: savegame_level.day_time as u64,
                        },
                        chunk_lut: Default::default(),
                    },
                    Persistence {
                        region_storage: tesseract_java_savegame::region::RegionStorage::new(
                            level_path.join("region"),
                        ),
                    },
                ));
            }
        };

        app.add_systems(PreStartup, spawn_levels)
            .add_systems(First, (load_players, load_chunks).before(UpdateFlush))
            .add_systems(
                First,
                apply_system_buffers
                    .in_set(UpdateFlush)
                    .after(replication::UpdateFlush),
            );
    }
}

#[derive(Component)]
struct Persistence {
    region_storage: tesseract_java_savegame::region::RegionStorage,
}

/// Loads savegame data for newly connected players
fn load_players(
    mut commands: Commands,

    levels: Query<(Entity, &level::Base)>,
    players: Query<(Entity, &replication::Connection), Added<replication::Connection>>,
) {
    for (player, connection) in players.iter() {
        let savegame_player_path =
            format!("levels/overworld/playerdata/{}.dat", connection.user().id);
        let savegame_player_path = Path::new(&savegame_player_path);
        if savegame_player_path.exists() {
            let savegame_player = {
                let mut data = vec![];
                GzDecoder::new(File::open(savegame_player_path).unwrap())
                    .read_to_end(&mut data)
                    .unwrap();
                tesseract_nbt::de::from_slice::<tesseract_java_savegame::entity::Player>(
                    &mut data.as_slice(),
                )
                .unwrap()
            };

            if let Some((level, _)) = levels
                .iter()
                .find(|(_, level_base)| level_base.name() == savegame_player.level)
            {
                commands
                    .entity(player)
                    .insert(actor::player::PlayerBundle {
                        base: actor::Base {
                            id: connection.user().id,
                            type_: "minecraft:player".into(),
                        },
                        position: actor::Position(DVec3::from_array(
                            savegame_player.entity.position,
                        )),
                        rotation: actor::Rotation {
                            pitch: savegame_player.entity.rotation[1],
                            yaw: savegame_player.entity.rotation[0],
                        },
                        interaction: Default::default(),
                    })
                    .set_parent(level);
            } else {
                warn!(
                    "Level ({:?}) for {:?} does not exist",
                    savegame_player.level, player
                );
            }
        }
    }
}

struct RenderCache {
    blocks: HashMap<String, u32>,
}

impl FromWorld for RenderCache {
    fn from_world(_world: &mut World) -> Self {
        todo!()
    }
}

/// Loads savegame chunks for newly spawned chunks
fn load_chunks(
    render_cache: Local<RenderCache>,
    biome_registry: Res<registry::DataRegistry<Biome>>,

    mut commands: Commands,

    mut levels: Query<&mut Persistence>,
    chunks: Query<(Entity, &level::chunk::Base, &Parent), Added<level::chunk::Base>>,
) {
    for (chunk, chunk_base, level) in chunks.iter() {
        let region_storage = &mut levels.get_mut(level.get()).unwrap().region_storage;
        if let Some(region_chunk_data) = region_storage.read(chunk_base.position()) {
            let savegame_chunk = tesseract_nbt::de::from_slice::<
                tesseract_java_savegame::chunk::Chunk,
            >(&mut region_chunk_data.as_slice())
            .unwrap();
            let sections = savegame_chunk
                .sections
                .into_iter()
                .map(|region_chunk_section| level::chunk::DataSection {
                    block_states: if let Some(data) = region_chunk_section.block_states.data {
                        PalettedContainer::Indirect {
                            palette: region_chunk_section
                                .block_states
                                .palette
                                .iter()
                                .map(|entry| render_cache.blocks[&entry.name])
                                .collect(),
                            storage: BitStorage::from_data(16 * 16 * 16, data),
                        }
                    } else {
                        PalettedContainer::Single(
                            render_cache.blocks[&region_chunk_section
                                .block_states
                                .palette
                                .first()
                                .unwrap()
                                .name],
                        )
                    },
                    biomes: if let Some(data) = region_chunk_section.biomes.data {
                        PalettedContainer::Indirect {
                            palette: region_chunk_section
                                .biomes
                                .palette
                                .iter()
                                .map(|entry| biome_registry.id(entry))
                                .collect(),
                            storage: BitStorage::from_data(4 * 4 * 4, data),
                        }
                        .fix()
                    } else {
                        PalettedContainer::Single(
                            biome_registry.id(region_chunk_section.biomes.palette.first().unwrap()),
                        )
                    },
                    block_state_changes: Default::default(),
                })
                .collect::<Vec<_>>();

            commands.entity(chunk).insert(level::chunk::Data {
                sections,
                y_offset: 4,
            });
        }
    }
}
