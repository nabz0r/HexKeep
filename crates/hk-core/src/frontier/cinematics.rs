//! Timelines advance only on simulation ticks, so pause/resume cannot skip a line.
use super::data;
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Timeline {
    pub id: String,
    pub beat: usize,
    pub elapsed: u32,
}
impl Timeline {
    pub fn new(id: String) -> Option<Self> {
        data::catalog()
            .scenes
            .iter()
            .find(|s| s.id == id)
            .map(|_| Self {
                id,
                beat: 0,
                elapsed: 0,
            })
    }
    pub fn valid(&self) -> bool {
        data::catalog()
            .scenes
            .iter()
            .find(|s| s.id == self.id)
            .is_some_and(|s| {
                self.beat < s.beats.len() && self.elapsed <= s.beats[self.beat].duration
            })
    }
    pub fn advance(&mut self, next: bool) -> bool {
        let Some(scene) = data::catalog().scenes.iter().find(|s| s.id == self.id) else {
            return true;
        };
        self.elapsed += 1;
        if next || self.elapsed >= scene.beats[self.beat].duration {
            self.beat += 1;
            self.elapsed = 0;
        }
        self.beat >= scene.beats.len()
    }
    pub fn view(&self) -> serde_json::Value {
        let scene = data::catalog()
            .scenes
            .iter()
            .find(|s| s.id == self.id)
            .unwrap();
        let beat = &scene.beats[self.beat];
        let previous = &scene.beats[self.beat.saturating_sub(1)];
        serde_json::json!({"id":self.id,"beat":self.beat,"total":scene.beats.len(),"elapsed":self.elapsed,"duration":beat.duration,"speaker":beat.speaker,"text":beat.text,"camera":beat.camera,"previous_camera":previous.camera,"zoom":beat.zoom,"previous_zoom":previous.zoom})
    }
}
