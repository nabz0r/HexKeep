//! Reproducible playthrough of actual movement, collision, assisted aim and objectives.
use hk_core::Game;
use hk_proto::*;
use std::collections::VecDeque;
fn next_step(g: &Game, goal: Vec2) -> Vec2 {
    let b = g.battle.as_ref().unwrap();
    let p = b.fighters[0].pos;
    let cell = |v: Vec2| {
        (
            (v.x / UNIT).clamp(0, 29) as usize,
            (v.y / UNIT).clamp(0, 15) as usize,
        )
    };
    let (sx, sy) = cell(p);
    let (tx, ty) = cell(goal);
    let mut prev = [usize::MAX; 480];
    let start = sy * 30 + sx;
    let end = ty * 30 + tx;
    prev[start] = start;
    let mut q = VecDeque::from([start]);
    while let Some(v) = q.pop_front() {
        if v == end {
            break;
        }
        let x = v % 30;
        let y = v / 30;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if !(1..29).contains(&nx) || !(1..15).contains(&ny) {
                continue;
            }
            let n = ny as usize * 30 + nx as usize;
            let xx = nx * UNIT + UNIT / 2;
            let yy = ny * UNIT + UNIT / 2;
            if prev[n] != usize::MAX
                || b.obstacles.iter().any(|o| {
                    xx + 80 > o.x && xx - 80 < o.x + o.w && yy + 80 > o.y && yy - 80 < o.y + o.h
                })
            {
                continue;
            }
            prev[n] = v;
            q.push_back(n);
        }
    }
    if prev[end] == usize::MAX {
        return Vec2::new(goal.x - p.x, goal.y - p.y).scaled(1024);
    }
    let mut at = end;
    while prev[at] != start && at != start {
        at = prev[at];
    }
    let dest = if at == start {
        goal
    } else {
        Vec2::new(
            (at % 30) as i32 * UNIT + UNIT / 2,
            (at / 30) as i32 * UNIT + UNIT / 2,
        )
    };
    Vec2::new(dest.x - p.x, dest.y - p.y).scaled(1024)
}
fn main() {
    let mut wins = 0;
    for realm in 0..3 {
        for role in 0..3 {
            let mut g = Game::new("", true);
            g.save.created = true;
            g.save.realm = Realm::from_index(realm);
            g.save.role = Role::from_index(role);
            g.save.kills = role as u32;
            g.ui_action("expedition");
            let map = g.battle.as_ref().unwrap().seed % 3;
            while g.screen == 6 {
                let b = g.battle.as_ref().unwrap();
                let p = &b.fighters[0];
                let run = g.expedition.as_ref().unwrap();
                let tick = b.tick;
                let mut mv = Vec2::default();
                if let Some(i) = run.charges.iter().position(|v| *v < 90) {
                    let (x, y) = [(6, 4), (15, 12), (25, 5)][i];
                    let target = Vec2::new(x * UNIT, y * UNIT);
                    if p.pos.dist2(target) > (UNIT as i64).pow(2) {
                        mv = next_step(&g, target)
                    }
                } else {
                    let e = &b.fighters[1];
                    let d = p.pos.dist2(e.pos);
                    if d > (6 * UNIT as i64).pow(2) {
                        mv = next_step(&g, e.pos)
                    } else if d < (3 * UNIT as i64).pow(2) {
                        mv = Vec2::new(p.pos.x - e.pos.x, p.pos.y - e.pos.y).scaled(1024)
                    }
                }
                g.controls(
                    mv.x as i16,
                    mv.y as i16,
                    0,
                    0,
                    true,
                    tick % 100 == 1,
                    tick % 181 == 1,
                );
                g.tick(2000000000 + tick as u64 / 30);
            }
            let b = g.battle.as_ref().unwrap();
            let run = g.expedition.as_ref().unwrap();
            if run.victory {
                wins += 1;
            }
            println!("realm={realm} role={role} map={map} victory={} seconds={} lights={:?} kills={} deaths={} hp={}",run.victory,b.tick/30,run.charges,b.fighters[0].kills,b.fighters[0].deaths,b.fighters[0].hp);
        }
    }
    println!("RESULT {wins}/9 expeditions won using real controls");
    assert!(wins >= 6, "Basic assisted runs should be approachable");
}
