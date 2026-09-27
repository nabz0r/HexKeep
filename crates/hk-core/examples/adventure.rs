//! Plays all 45 realm/role/contract combinations without warps or altered combat stats.
use hk_core::Game;
use hk_proto::*;
fn main() {
    let mut wins = 0;
    let offline = std::env::args().any(|a| a == "--offline");
    let tiers = if std::env::args().any(|a| a == "--all-tiers") {
        3
    } else {
        1
    };
    for tier in 0..tiers {
        for realm in 0..3 {
            for role in 0..3 {
                for kind in 0..5 {
                    let mut g = if offline {
                        Game::new_offline("")
                    } else {
                        Game::new("", true)
                    };
                    g.save.created = true;
                    g.save.journey.victories = tier * 6;
                    g.save.journey.difficulty = tier as u8;
                    g.save.realm = Realm::from_index(realm);
                    g.save.role = Role::from_index(role);
                    g.save.kills = role as u32;
                    g.ui_action(&format!("contract:{kind}"));
                    let mut inspected = 0;

                    while g.screen == 6 {
                        if inspected != g.save.journey.items.len() {
                            inspected = g.save.journey.items.len();
                            let best: Vec<_> = (0..3)
                                .filter_map(|slot| {
                                    g.save
                                        .journey
                                        .items
                                        .iter()
                                        .filter(|i| i.slot == slot)
                                        .max_by_key(|i| {
                                            i.vitality + i.power * 2 + i.guard + i.haste as i32 * 4
                                        })
                                        .map(|i| i.id)
                                })
                                .collect();
                            g.ui_action("inventory");
                            for id in best {
                                g.ui_action(&format!("equip:{id}"));
                            }
                            g.ui_action("inventory_back");
                        }
                        let b = g.battle.as_ref().unwrap();
                        let p = &b.fighters[0];
                        let run = g.expedition.as_ref().unwrap();
                        let tick = b.tick;
                        let goal = if let Some(i) = run.caches.iter().position(|v| !*v) {
                            let (x, y) = [(3, 13), (16, 2), (27, 12)][i];
                            Some(Vec2::new(x * UNIT, y * UNIT))
                        } else if kind == 4 && run.wave == 0 {
                            run.sites
                                .iter()
                                .filter(|s| !s.opened)
                                .min_by_key(|s| p.pos.dist2(s.pos))
                                .map(|s| s.pos)
                        } else if (kind == 0 || kind == 2) && run.charges.iter().any(|v| *v < 90) {
                            let i = run.charges.iter().position(|v| *v < 90).unwrap();
                            let (x, y) = [(6, 4), (15, 12), (25, 5)][i];
                            Some(Vec2::new(x * UNIT, y * UNIT))
                        } else {
                            None
                        };
                        let mut mv = Vec2::default();
                        if let Some(goal) = goal {
                            if p.pos.dist2(goal) > (UNIT as i64).pow(2) {
                                mv = hk_sim::navigation::direction(p.pos, goal, &b.obstacles);
                            }
                        } else if let Some(e) = b
                            .fighters
                            .iter()
                            .skip(1)
                            .filter(|f| f.hp > 0)
                            .min_by_key(|f| p.pos.dist2(f.pos))
                        {
                            let d = p.pos.dist2(e.pos);
                            let range = if role == 1 { 3 } else { 5 };
                            if d > (range * UNIT as i64).pow(2)
                                || !hk_sim::navigation::clear_line(p.pos, e.pos, &b.obstacles, 24)
                            {
                                mv = hk_sim::navigation::direction(p.pos, e.pos, &b.obstacles)
                            } else if d < (2 * UNIT as i64).pow(2) {
                                mv = Vec2::new(p.pos.x - e.pos.x, p.pos.y - e.pos.y).scaled(1024);
                            }
                        }
                        if let Some(h) = run.hazards.iter().find(|h| {
                            h.strike > tick
                                && p.pos.dist2(h.pos) < ((h.radius + UNIT / 2) as i64).pow(2)
                        }) {
                            let offset = if p.pos.x < 15 * UNIT {
                                4 * UNIT
                            } else {
                                -4 * UNIT
                            };
                            let escape = hk_sim::navigation::safe_position(
                                Vec2::new(h.pos.x + offset, h.pos.y),
                                &b.obstacles,
                            );
                            mv = hk_sim::navigation::direction(p.pos, escape, &b.obstacles);
                        }
                        let heal = p.hp < p.max_hp / 2;
                        g.controls(
                            mv.x as i16,
                            mv.y as i16,
                            0,
                            0,
                            true,
                            tick % 100 == 1,
                            tick % 181 == 1,
                        );
                        if heal {
                            g.ui_action("heal");
                        }
                        if kind == 4 {
                            g.ui_action("interact");
                        }
                        g.tick(2000000000 + tick as u64 / 30);
                    }
                    let run = g.expedition.as_ref().unwrap();
                    let victory = run.victory;
                    let caches = run.caches.iter().filter(|v| **v).count();
                    let b = g.battle.as_ref().unwrap();
                    println!("tier={tier} realm={realm} role={role} event={kind} victory={victory} caches={caches} seconds={} deaths={}",b.tick/30,b.fighters[0].deaths);
                    if victory {
                        wins += 1;
                    }
                    assert_eq!(caches, 3);
                    g.ui_action("finish");
                    let n = g.save.journey.items.len();
                    assert!(n >= 6);
                    let saved = g.snapshot();
                    let restored = if offline {
                        Game::new_offline(&saved)
                    } else {
                        Game::new(&saved, true)
                    };
                    assert_eq!(restored.save.journey.items.len(), n);
                }
            }
        }
    }
    println!(
        "RESULT {wins}/{} adventures won with real movement, aim, caches and save reload",
        tiers * 45
    );
    assert_eq!(wins, tiers * 45);
}
