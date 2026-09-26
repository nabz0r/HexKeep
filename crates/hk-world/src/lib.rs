use h3o::{CellIndex, LatLng, Resolution};
use hk_proto::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub fn cell_at(lat_e7: i32, lng_e7: i32) -> Option<u64> {
    LatLng::new(lat_e7 as f64 / 1e7, lng_e7 as f64 / 1e7)
        .ok()
        .map(|v| v.to_cell(Resolution::Nine).into())
}
pub fn pilot_cell() -> u64 {
    cell_at(496116000, 61319000).unwrap()
}
pub fn neighbors(cell: u64, rings: u32) -> Vec<u64> {
    let Ok(c) = CellIndex::try_from(cell) else {
        return vec![];
    };
    let mut cells: Vec<u64> = c
        .grid_disk::<Vec<_>>(rings)
        .into_iter()
        .map(u64::from)
        .collect();
    cells.sort();
    cells
}
pub fn site(cell: u64) -> Option<u64> {
    CellIndex::try_from(cell)
        .ok()?
        .parent(Resolution::Eight)
        .map(u64::from)
}
pub fn marche(cell: u64) -> Option<u64> {
    CellIndex::try_from(cell)
        .ok()?
        .parent(Resolution::Seven)
        .map(u64::from)
}
pub fn offsets(cell: u64, rings: u32) -> Vec<(u64, i32, i32)> {
    let Ok(origin) = CellIndex::try_from(cell) else {
        return vec![];
    };
    let Ok(base) = origin.to_local_ij(origin) else {
        return vec![];
    };
    neighbors(cell, rings)
        .into_iter()
        .filter_map(|id| {
            let c = CellIndex::try_from(id).ok()?;
            let ij = c.to_local_ij(origin).ok()?;
            Some((id, ij.coord.i - base.coord.i, ij.coord.j - base.coord.j))
        })
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub id: u64,
    pub last_watch: u64,
    pub continuous: u32,
    pub clear: bool,
    pub bastion: Option<Realm>,
    pub battles: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub current: u64,
    pub anchor: u64,
    pub cells: BTreeMap<u64, Cell>,
    pub seconds: u64,
    pub banner: bool,
    pub lower_at: Option<u64>,
    pub gps: bool,
}
impl Default for World {
    fn default() -> Self {
        let current = pilot_cell();
        let mut v = Self {
            current,
            anchor: current,
            cells: BTreeMap::new(),
            seconds: 0,
            banner: false,
            lower_at: None,
            gps: false,
        };
        v.enter(current, 0);
        if let Some(c) = v.cells.get_mut(&current) {
            c.clear = true;
            c.continuous = 600;
        }
        v
    }
}
impl World {
    pub fn enter(&mut self, cell: u64, now: u64) {
        if CellIndex::try_from(cell).is_err() {
            return;
        }
        self.current = cell;
        for id in neighbors(cell, 3) {
            self.cells.entry(id).or_insert(Cell {
                id,
                last_watch: now,
                continuous: 0,
                clear: false,
                bastion: None,
                battles: 0,
            });
        }
    }
    pub fn elapse(&mut self, now: u64, seconds: u32) {
        self.seconds += seconds as u64;
        let watched = neighbors(self.current, 2);
        for c in self.cells.values_mut() {
            if now.saturating_sub(c.last_watch) > 1800 {
                c.clear = false;
                c.continuous = 0;
            }
            if watched.binary_search(&c.id).is_ok() {
                if now.saturating_sub(c.last_watch) > 3 {
                    c.continuous = 0;
                }
                c.last_watch = now;
                c.continuous = c.continuous.saturating_add(seconds);
                if c.continuous >= 600 {
                    c.clear = true;
                }
            }
        }
        if self.lower_at.is_some_and(|t| now >= t) {
            self.banner = false;
            self.lower_at = None;
        }
    }
    pub fn found(&mut self, realm: Realm) -> Result<(), &'static str> {
        let c = self.cells.get(&self.current).ok_or("Cellule inconnue")?;
        if !c.clear {
            return Err("Éclaire ce lieu : 10 minutes de veille.");
        }
        let s = site(self.current);
        if self
            .cells
            .values()
            .any(|c| c.bastion.is_some() && site(c.id) == s)
        {
            return Err("Un bastion existe déjà sur ce site.");
        }
        self.cells.get_mut(&self.current).unwrap().bastion = Some(realm);
        Ok(())
    }
    pub fn toggle_banner(&mut self, now: u64) {
        if self.banner {
            if self.lower_at.is_none() {
                self.lower_at = Some(now + 60)
            }
        } else {
            self.banner = true;
            self.lower_at = None;
        }
    }
    pub fn memory_must_drop(speed_millimeters_sec: u32, missing_secs: u32) -> bool {
        speed_millimeters_sec > 3888 || missing_secs > 60
    }
}
/// Q16 exponent approximation: (1 + S/(3*1024))^-1024, all fixed-point.
pub fn guard(weights_q16: &[u32]) -> u32 {
    let s: u64 = weights_q16.iter().map(|x| *x as u64).sum();
    let den = (1u128 << 32) + (s as u128 * (1u128 << 16) / (3 * 1024));
    let mut base = ((1u128 << 64) / den) as u64;
    for _ in 0..10 {
        base = ((base as u128 * base as u128) >> 32) as u64;
    }
    (((1u64 << 32) - base) * 100 + (1u64 << 31))
        .checked_shr(32)
        .unwrap() as u32
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry() {
        assert_eq!(neighbors(pilot_cell(), 2).len(), 19);
        assert_eq!(offsets(pilot_cell(), 2).len(), 19);
        assert!(site(pilot_cell()).is_some());
    }
    #[test]
    fn darkness_and_watch() {
        let mut w = World::default();
        assert!(w.cells[&w.current].clear);
        w.elapse(4000, 1);
        assert!(!w.cells[&w.current].clear);
        for t in 4001..4601 {
            w.elapse(t, 1);
        }
        assert!(w.cells[&w.current].clear);
        assert!(w.found(Realm::Aurelon).is_ok());
        assert!(w.found(Realm::Skarn).is_err());
    }
    #[test]
    fn guard_saturates() {
        assert_eq!(guard(&[]), 0);
        assert!((27..=29).contains(&guard(&[65536])));
        assert!(guard(&[65536; 50]) >= 99);
    }
    #[test]
    fn memory_speed() {
        assert!(!World::memory_must_drop(3888, 60));
        assert!(World::memory_must_drop(3889, 0));
        assert!(World::memory_must_drop(0, 61));
    }
    #[test]
    fn banner_delay() {
        let mut w = World::default();
        w.toggle_banner(0);
        w.toggle_banner(10);
        w.elapse(69, 1);
        assert!(w.banner);
        w.elapse(70, 1);
        assert!(!w.banner);
    }
}
