use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Entity {
    #[serde(rename = "Pos")]
    pub position: [f64; 3],
    #[serde(rename = "Rotation")]
    pub rotation: [f32; 2],
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    #[serde(flatten)]
    pub entity: Entity,

    #[serde(rename = "Dimension")]
    pub level: String,
}
