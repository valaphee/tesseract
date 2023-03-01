use std::collections::HashMap;
use std::time::Duration;

use bevy::app::ScheduleRunnerSettings;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use tesseract_protocol::packet::s2c;
use tesseract_protocol::types::{Biome, BiomeEffects, BiomePrecipitation, DimensionType, GameType, Nbt, Registries, Registry, RegistryEntry, VarInt};

use crate::connection::ConnectionPlugin;

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

        .add_system(actor::populate_packet_queue)

        .add_system(level::chunk::process_packet_queue)

        // Testing
        .add_startup_system(load_dimension)
        .add_system(load_connection)
        .add_system(populate_chunk)

        .run();
}

fn load_dimension(
    mut commands: Commands,
) {
    commands.spawn((
        level::Dimension {
            name: "overworld".to_string(),
        },
        level::chunk::LookupTable::default()
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

        connection.sender.send(s2c::GamePacket::Login {
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
                            depth: None,
                            temperature: 0.0,
                            scale: None,
                            downfall: 0.0,
                            category: None,
                            temperature_modifier: None,
                            effects: BiomeEffects {
                                sky_color: 0,
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
                        name: "overworld".to_string(),
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
                            min_y: -64,
                            height: 384,
                            logical_height: 384,
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
        }).unwrap();
    }
}

#[derive(Component)]
struct Terrain;

fn populate_chunk(
    mut commands: Commands,

    unpopulated_chunks: Query<(Entity, &level::chunk::Position), Without<Terrain>>
) {
    for (chunk, position) in unpopulated_chunks.iter() {


        commands.entity(chunk).insert((
            Terrain,
            level::chunk::InitialPacket(s2c::GamePacket::LevelChunkWithLight {
                x: position.0[0],
                y: position.0[1],
                chunk_data: s2c::game::LevelChunkPacketData {
                    heightmaps: Nbt(s2c::game::LevelChunkPacketDataHeightmap {
                        /*motion_blocking: vec![],
                        world_surface: vec![],*/
                    }),
                    buffer: vec![],
                    block_entities_data: vec![],
                },
                light_data: s2c::game::LightUpdatePacketData {
                    trust_edges: false,
                    sky_y_mask: vec![],
                    block_y_mask: vec![],
                    empty_sky_y_mask: vec![],
                    empty_block_y_mask: vec![],
                    sky_updates: vec![],
                    block_updates: vec![],
                },
            }),
        ));
    }
}
