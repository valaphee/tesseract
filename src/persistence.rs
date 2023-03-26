use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use bevy::{math::DVec3, prelude::*};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};

use tesseract_protocol::types::{Biome, BitStorage, PalettedContainer};

use crate::{actor, level, registry, replication};

#[derive(Serialize, Deserialize)]
pub struct PersistencePlugin {
    path: PathBuf,
}

impl Default for PersistencePlugin {
    fn default() -> Self {
        Self {
            path: "levels".into(),
        }
    }
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        let path = self.path.clone();

        let spawn_levels = move |mut commands: Commands| {
            commands.spawn((
                level::LevelBundle {
                    level: level::Level {
                        name: "minecraft:overworld".into(),
                        dimension_type: "minecraft:overworld".into(),
                    },
                    chunks: default(),
                },
                Persistence {
                    region_storage: tesseract_savegame::region::RegionStorage::new(
                        path.join("overworld/region"),
                    ),
                },
            ));
        };

        app.add_systems(PreStartup, spawn_levels)
            .add_systems(PreUpdate, load_players)
            .add_systems(PreUpdate, load_chunks);
    }
}

#[derive(Component)]
struct Persistence {
    region_storage: tesseract_savegame::region::RegionStorage,
}

/// Loads savegame data for newly connected players
fn load_players(
    mut commands: Commands,
    levels: Query<(Entity, &level::Level)>,
    players: Query<(Entity, &replication::Connection), Added<replication::Connection>>,
) {
    for (player, connection) in players.iter() {
        let savegame_player_path =
            format!("levels/overworld/playerdata/{}.dat", connection.user.id);
        let savegame_player_path = Path::new(&savegame_player_path);
        if savegame_player_path.exists() {
            let savegame_player = {
                let mut data = vec![];
                GzDecoder::new(File::open(savegame_player_path).unwrap())
                    .read_to_end(&mut data)
                    .unwrap();
                tesseract_nbt::de::from_slice::<tesseract_savegame::entity::Player>(
                    &mut data.as_slice(),
                )
                .unwrap()
            };

            let (level, _) = levels
                .iter()
                .find(|(_, level_base)| level_base.name == savegame_player.level)
                .unwrap();

            commands
                .entity(player)
                .insert((actor::ActorBundle {
                    actor: actor::Actor {
                        id: connection.user.id,
                        type_: "minecraft:player".into(),
                    },
                    position: actor::Position(DVec3::from_array(savegame_player.entity.position)),
                    rotation: actor::Rotation {
                        pitch: savegame_player.entity.rotation[1],
                        yaw: savegame_player.entity.rotation[0],
                    },
                    head_rotation: actor::HeadRotation {
                        head_yaw: savegame_player.entity.rotation[0],
                    },
                },))
                .set_parent(level);
        }
    }
}

/// Loads savegame chunks for newly spawned chunks
fn load_chunks(
    block_state_registry: Res<registry::BlockStateRegistry>,
    biome_registry: Res<registry::DataRegistry<Biome>>,
    mut commands: Commands,
    mut levels: Query<&mut Persistence>,
    chunks: Query<(Entity, &level::chunk::Chunk, &Parent), Without<level::chunk::Terrain>>,
) {
    for (chunk, chunk_base, level) in chunks.iter() {
        let region_storage = &mut levels.get_mut(level.get()).unwrap().region_storage;
        if let Some(region_chunk_data) = region_storage.read(chunk_base.0) {
            let savegame_chunk = tesseract_nbt::de::from_slice::<tesseract_savegame::chunk::Chunk>(
                &mut region_chunk_data.as_slice(),
            )
            .unwrap();
            let sections = savegame_chunk
                .sections
                .into_iter()
                .map(|region_chunk_section| {
                    (
                        if let Some(data) = region_chunk_section.block_states.data {
                            if region_chunk_section.block_states.palette.is_empty() {
                                PalettedContainer::Global(BitStorage::from_data(16 * 16 * 16, data))
                            } else {
                                PalettedContainer::Linear {
                                    palette: region_chunk_section
                                        .block_states
                                        .palette
                                        .iter()
                                        .map(|entry| block_state_registry.id(&entry.name()))
                                        .collect(),
                                    storage: BitStorage::from_data(16 * 16 * 16, data),
                                }
                                .fix()
                            }
                        } else {
                            PalettedContainer::SingleValue(
                                block_state_registry.id(&region_chunk_section
                                    .block_states
                                    .palette
                                    .first()
                                    .unwrap()
                                    .name()),
                            )
                        },
                        if let Some(data) = region_chunk_section.biomes.data {
                            if region_chunk_section.biomes.palette.is_empty() {
                                PalettedContainer::Global(BitStorage::from_data(4 * 4 * 4, data))
                            } else {
                                PalettedContainer::Linear {
                                    palette: region_chunk_section
                                        .biomes
                                        .palette
                                        .iter()
                                        .map(|entry| biome_registry.id(entry))
                                        .collect(),
                                    storage: BitStorage::from_data(4 * 4 * 4, data),
                                }
                                .fix()
                            }
                        } else {
                            PalettedContainer::SingleValue(
                                biome_registry.id(region_chunk_section
                                    .biomes
                                    .palette
                                    .first()
                                    .unwrap()),
                            )
                        },
                    )
                })
                .collect::<Vec<_>>();

            commands
                .entity(chunk)
                .insert(level::chunk::Terrain { sections });
        }
    }
}
