//! Authored content stays separate from simulation and presentation.
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
#[derive(Clone, Deserialize, Serialize)]
pub struct Zone {
    pub id: u8,
    pub name: String,
    pub biome: String,
    pub subtitle: String,
    pub colors: Vec<String>,
    pub map_x: f32,
    pub map_y: f32,
    pub spawn: [i32; 2],
    pub beacons: Vec<[i32; 2]>,
    pub relics: Vec<[i32; 2]>,
    pub rescue: [i32; 2],
    pub boss: [i32; 2],
    pub boss_name: String,
    pub speaker: String,
    pub overview: String,
    pub lore: String,
    pub obstacles: Vec<[i32; 4]>,
    pub width: i32,
    pub height: i32,
    pub npcs: String,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Quest {
    pub id: String,
    pub zone: u8,
    pub title: String,
    pub description: String,
    pub main: bool,
    pub objective: String,
    pub target: u32,
    #[serde(rename = "reward_dust")]
    pub dust: u32,
    #[serde(rename = "reward_xp")]
    pub xp: u32,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Beat {
    pub speaker: String,
    pub text: String,
    pub camera: [i32; 2],
    pub zoom: f32,
    pub duration: u32,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Scene {
    pub id: String,
    pub beats: Vec<Beat>,
}
#[derive(Deserialize, Serialize)]
pub struct Catalog {
    pub version: u8,
    pub title: String,
    pub zones: Vec<Zone>,
    pub quests: Vec<Quest>,
    pub scenes: Vec<Scene>,
    pub monsters: Vec<serde_json::Value>,
}
pub fn catalog() -> &'static Catalog {
    static DATA: OnceLock<Catalog> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../assets/frontier/campaign.json"))
            .expect("validated campaign data")
    })
}
pub fn zone(id: u8) -> &'static Zone {
    &catalog().zones[id.min(2) as usize]
}
pub const CHRONICLES: &str = include_str!("../../../../assets/frontier/chronicles.txt");
