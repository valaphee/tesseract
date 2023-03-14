use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, math::DVec3, prelude::*};

use tesseract_protocol::{
    packet::{c2s, s2c},
    types::{
        Biome, BiomeEffects, BiomePrecipitation, DimensionType, GameType, Nbt, PalettedContainer,
        Registries, Registry, RegistryEntry, VarI32,
    },
};

use crate::level::terrain::Section;

mod actor;
mod block;
mod connection;
mod level;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0,
        )))
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(connection::ConnectionPlugin::default())
        .add_system(level::replication::replicate)
        .add_system(level::replication::replicate_2)
        .add_system(level::chunk::update_hierarchy)
        .add_startup_system(load_dimension)
        .add_system(load_connection)
        .add_system(update_pos)
        .add_system(populate_chunk)
        .run();
}

fn load_dimension(mut commands: Commands) {
    commands.spawn(level::chunk::LookupTable::default());
}

fn load_connection(
    mut commands: Commands,
    dimensions: Query<Entity, With<level::chunk::LookupTable>>,
    new_connections: Query<(Entity, &connection::Connection), Added<connection::Connection>>,
) {
    for (entity, connection) in new_connections.iter() {
        commands
            .entity(entity)
            .insert((
                actor::Position(DVec3::new(0.0, 0.0, 0.0)),
                actor::Rotation(Vec2::new(0.0, 0.0)),
                actor::HeadRotation(0.0),
                level::replication::Replicated::default(),
                level::replication::ReplicationDistance(4),
            ))
            .set_parent(dimensions.single());

        connection.send(s2c::GamePacket::Login {
            player_id: entity.index() as i32,
            hardcore: false,
            game_type: GameType::Survival,
            previous_game_type: 0,
            levels: vec!["minecraft:overworld".to_string()],
            registry_holder: Nbt(Registries {
                biome_registry: Registry {
                    type_: "minecraft:worldgen/biome".to_string(),
                    value: vec![RegistryEntry {
                        name: "plains".to_string(),
                        id: 0,
                        element: Biome {
                            precipitation: BiomePrecipitation::None,
                            temperature: 0.0,
                            downfall: 0.0,
                            temperature_modifier: None,
                            effects: BiomeEffects {
                                sky_color: 0xFFFF0000,
                                water_fog_color: 0,
                                fog_color: 0,
                                water_color: 0,
                                foliage_color: None,
                                grass_color: None,
                                grass_color_modifier: None,
                                music: None,
                                ambient_sound: None,
                                additions_sound: None,
                                mood_sound: None,
                            },
                        },
                    }],
                },
                dimension_type_registry: Registry {
                    type_: "minecraft:dimension_type".to_string(),
                    value: vec![RegistryEntry {
                        name: "minecraft:overworld".to_string(),
                        id: 0,
                        element: DimensionType {
                            piglin_safe: true,
                            has_raids: true,
                            monster_spawn_light_level: 0,
                            monster_spawn_block_light_limit: 0,
                            natural: true,
                            ambient_light: 1.0,
                            fixed_time: None,
                            infiniburn: "#minecraft:infiniburn_overworld".to_string(),
                            respawn_anchor_works: true,
                            has_skylight: true,
                            bed_works: true,
                            effects: "minecraft:overworld".to_string(),
                            min_y: 0,
                            height: 16 * 16,
                            logical_height: 16 * 16,
                            coordinate_scale: 1.0,
                            ultrawarm: false,
                            has_ceiling: false,
                        },
                    }],
                },
            }),
            dimension_type: "minecraft:overworld".to_string(),
            dimension: "minecraft:overworld".to_string(),
            seed: 0,
            max_players: VarI32(0),
            chunk_radius: VarI32(0),
            simulation_distance: VarI32(0),
            reduced_debug_info: false,
            show_death_screen: false,
            is_debug: false,
            is_flat: false,
            last_death_location: None,
        });

        connection.send(s2c::game::GamePacket::SetDefaultSpawnPosition {
            pos: IVec3::new(0, 100, 0),
            angle: 0.0,
        });
    }
}

fn update_pos(mut packet_que: Query<(&connection::Connection, &mut actor::Position)>) {
    for (con, mut pos) in packet_que.iter_mut() {
        for packet in &con.incoming {
            match packet {
                c2s::GamePacket::MovePlayerPos { x, y, z, .. } => {
                    pos.0[0] = *x;
                    pos.0[1] = *y;
                    pos.0[2] = *z;
                }
                c2s::GamePacket::MovePlayerPosRot { x, y, z, .. } => {
                    pos.0[0] = *x;
                    pos.0[1] = *y;
                    pos.0[2] = *z;
                }
                _ => {}
            }
        }
    }
}

fn populate_chunk(
    mut commands: Commands,
    unpopulated_chunks: Query<(Entity, &level::chunk::Position), Without<level::terrain::Terrain>>,
) {
    for (chunk, position) in unpopulated_chunks.iter() {
        let mut columns = Vec::new();
        for _ in 0..16 {
            let mut block_paletted_container = PalettedContainer::SingleValue {
                value: 0,
                storage_size: 16 * 16 * 16,
                linear_min_bits: 4,
                linear_max_bits: 8,
                global_bits: 15,
            };
            for x in 0..16 {
                for z in 0..16 {
                    block_paletted_container.get_and_set(0 << 16 | z << 4 | x, 1);
                }
            }

            let biome_paletted_container = PalettedContainer::SingleValue {
                value: 0,
                storage_size: 4 * 4 * 4,
                linear_min_bits: 3,
                linear_max_bits: 3,
                global_bits: 6,
            };

            columns.push(Section {
                blocks: block_paletted_container,
                biomes: biome_paletted_container,
            })
        }

        commands
            .entity(chunk)
            .insert((level::terrain::Terrain { sections: columns },));
    }
}
