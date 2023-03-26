use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Level {
    #[serde(rename = "Data")]
    pub data: LevelData,
}

#[derive(Serialize, Deserialize)]
pub struct LevelData {
    #[serde(rename = "Time")]
    pub time: i64,
    #[serde(rename = "DayTime")]
    pub day_time: i64,
}
