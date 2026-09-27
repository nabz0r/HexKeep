use super::{data, Campaign, Run};
use serde::{Deserialize, Serialize};
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Progress {
    pub accepted: bool,
    pub value: u32,
    pub claimed: bool,
}
impl Campaign {
    pub fn accept(&mut self, id: &str) -> bool {
        let Some(q) = data::catalog()
            .quests
            .iter()
            .find(|q| q.id == id && q.zone <= self.unlocked)
        else {
            return false;
        };
        let p = self.quests.entry(id.into()).or_default();
        if p.accepted {
            return false;
        }
        p.accepted = true;
        if q.objective == "beacons_boss" {
            p.value = self
                .run
                .as_ref()
                .filter(|r| r.zone == q.zone)
                .map(|r| r.beacons.iter().filter(|v| **v).count() as u32 + u32::from(r.victory))
                .unwrap_or(0);
        }
        self.tracked = id.into();
        true
    }
    pub fn progress(&mut self, zone: u8, event: &str, amount: u32) {
        for q in &data::catalog().quests {
            if q.zone == zone && q.objective == event {
                let p = self.quests.entry(q.id.clone()).or_default();
                if p.accepted && !p.claimed {
                    p.value = (p.value + amount).min(q.target);
                }
            }
        }
    }
    pub fn sync_main(&mut self, run: &Run) {
        let id = format!("main-{}", run.zone);
        let p = self.quests.entry(id).or_default();
        if p.accepted && !p.claimed {
            p.value = p
                .value
                .max(run.beacons.iter().filter(|v| **v).count() as u32 + u32::from(run.victory));
        }
    }
    pub fn quest_view(&self) -> Vec<serde_json::Value> {
        data::catalog().quests.iter().map(|q|{
        let p=self.quests.get(&q.id).cloned().unwrap_or_default();
        serde_json::json!({"id":q.id,"zone":q.zone,"title":q.title,"description":q.description,"main":q.main,"objective":q.objective,"target":q.target,"dust":q.dust,"xp":q.xp,"accepted":p.accepted,"progress":p.value,"claimed":p.claimed,"ready":p.accepted&&!p.claimed&&p.value>=q.target,"locked":q.zone>self.unlocked,"tracked":self.tracked==q.id})
    }).collect()
    }
}
