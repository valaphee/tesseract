use crate::{connection, level};
use bevy::{math::DVec3, prelude::*};
use tesseract_protocol::{
    packet::s2c,
    types::{Biome, BiomeEffects, GameType, Nbt, Registries, Registry, RegistryEntry, VarI32},
};

#[derive(Component)]
pub struct Position(pub DVec3);

#[derive(Component)]
pub struct Rotation {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Component)]
pub struct HeadRotation {
    pub head_yaw: f32,
}

pub fn load_connection(
    mut commands: Commands,
    levels: Query<(Entity, &level::Level)>,
    new_connections: Query<(Entity, &connection::Connection), Added<connection::Connection>>,
) {
    for (entity, connection) in new_connections.iter() {
        let (level_entity, level) = levels.single();
        commands
            .entity(entity)
            .insert((
                Position(DVec3::new(0.0, 0.0, 0.0)),
                Rotation {
                    pitch: 0.0,
                    yaw: 0.0,
                },
                HeadRotation { head_yaw: 0.0 },
            ))
            .set_parent(level_entity);

        connection.send(s2c::GamePacket::Login {
            player_id: entity.index() as i32,
            hardcore: false,
            game_type: GameType::Survival,
            previous_game_type: 0,
            levels: vec![level.name.clone()],
            registry_holder: Nbt(Registries {
                dimension_type_registry: Registry {
                    type_: "minecraft:dimension_type".to_string(),
                    value: vec![RegistryEntry {
                        name: level.name.clone(),
                        id: 0,
                        element: level.dimension.clone(),
                    }],
                },
                biome_registry: Registry {
                    type_: "minecraft:worldgen/biome".to_string(),
                    value: vec![RegistryEntry {
                        name: "minecraft:plains".to_string(),
                        id: 0,
                        element: Biome {
                            has_precipitation: true,
                            precipitation: "rain".to_string(),
                            temperature: 0.0,
                            temperature_modifier: None,
                            downfall: 0.0,
                            effects: BiomeEffects {
                                fog_color: 0,
                                water_color: 0,
                                water_fog_color: 0,
                                sky_color: 0,
                                foliage_color: None,
                                grass_color: None,
                                grass_color_modifier: None,
                                ambient_sound: None,
                                mood_sound: None,
                                additions_sound: None,
                                music: None,
                            },
                        },
                    }],
                },
            }),
            dimension_type: level.name.clone(),
            dimension: level.name.clone(),
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
            yaw: 0.0,
        })
    }
}
