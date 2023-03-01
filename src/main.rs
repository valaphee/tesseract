use std::time::Duration;

use bevy::app::ScheduleRunnerSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use byteorder::{BigEndian, WriteBytesExt};

use tesseract_protocol::bit_storage::BitStorage;
use tesseract_protocol::packet::{c2s, s2c};
use tesseract_protocol::paletted_container::PalettedContainer;
use tesseract_protocol::types::{
    Biome, BiomeEffects, BiomePrecipitation, DimensionType, GameType, Nbt, Registries, Registry,
    RegistryEntry, VarInt,
};
use tesseract_protocol::Encode;

use crate::connection::{Connection, ConnectionPlugin};
use crate::level::chunk::{Section, Terrain};

pub mod actor;
pub mod connection;
pub mod level;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 20.0,
        )))
        .add_plugin(LogPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(ConnectionPlugin::default())
        .add_system(level::chunk::initialize_chunk)
        .add_system(level::chunk::update_chunk_hierarchy)
        .add_system(level::chunk::send_initial_packet)
        //.add_system(actor::populate_packet_queue)
        .add_system(level::chunk::process_packet_queue)
        // Testing
        .add_startup_system(load_dimension)
        .add_system(load_connection)
        .add_system(populate_chunk)
        .add_system(update_pos)
        .run();
}

fn load_dimension(mut commands: Commands) {
    commands.spawn((
        level::Dimension {
            name: "overworld".to_string(),
        },
        level::chunk::LookupTable::default(),
    ));
}

fn load_connection(
    mut commands: Commands,
    new_connections: Query<(Entity, &connection::Connection), Added<connection::Connection>>,
) {
    for (entity, connection) in new_connections.iter() {
        commands.entity(entity).insert((
            actor::Position([0.0, 0.0, 0.0]),
            actor::Rotation([0.0, 0.0]),
            actor::HeadRotation(0.0),
            level::chunk::Load(4),
            level::chunk::Loaded::default(),
        ));

        connection
            .sender
            .send(s2c::GamePacket::Login {
                player_id: entity.index() as i32,
                hardcore: false,
                game_type: GameType::Survival,
                previous_game_type: 0,
                levels: vec!["minecraft:overworld".to_string()],
                registry_holder: Nbt(Registries {
                    biome_registry: Registry {
                        _type: "minecraft:worldgen/biome".to_string(),
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
                        _type: "minecraft:dimension_type".to_string(),
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
                max_players: VarInt(0),
                chunk_radius: VarInt(0),
                simulation_distance: VarInt(0),
                reduced_debug_info: false,
                show_death_screen: false,
                is_debug: false,
                is_flat: false,
                last_death_location: None,
            })
            .unwrap();

        connection
            .sender
            .send(s2c::game::GamePacket::SetDefaultSpawnPosition {
                pos: glam::IVec3::new(0, 100, 0),
                angle: 0.0,
            })
            .unwrap();
    }
}

fn update_pos(mut packet_que: Query<(&Connection, &mut actor::Position)>) {
    for (con, mut pos) in packet_que.iter_mut() {
        for packet in &con.received {
            match packet {
                c2s::GamePacket::MovePlayerPos { x, y, z, on_ground } => {
                    println!("update");
                    pos.0[0] = *x;
                    pos.0[1] = *y;
                    pos.0[2] = *z;
                }
                c2s::GamePacket::MovePlayerPosRot {
                    x,
                    y,
                    z,
                    y_rot,
                    x_rot,
                    on_ground,
                } => {
                    println!("update2");
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
    unpopulated_chunks: Query<(Entity, &level::chunk::Position), Without<Terrain>>,
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
            .insert((Terrain { column: columns },));
    }
}
