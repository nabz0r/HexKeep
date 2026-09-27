use super::*;
fn game() -> Game {
    let mut g = Game::new_offline("");
    g.ui_action("f:atlas");
    g.ui_action("f:travel:0");
    g.ui_action("f:skip");
    g
}
fn run(g: &Game) -> &Run {
    g.save.frontier.run.as_ref().unwrap()
}
fn ticks(g: &mut Game, n: u32) {
    for _ in 0..n {
        g.tick(g.now + 1);
    }
}
#[test]
fn authored_world_has_reachable_objectives_and_complete_content() {
    let c = data::catalog();
    assert_eq!(c.zones.len(), 3);
    assert_eq!(c.quests.len(), 12);
    assert_eq!(c.monsters.len(), 5);
    for z in &c.zones {
        assert_eq!((z.width * z.height) as usize, world::CELLS);
        let distances = world::field(z, world::at(z.spawn));
        for p in z
            .beacons
            .iter()
            .chain(&z.relics)
            .chain([&z.rescue, &z.boss])
        {
            assert!(
                world::free(z, world::at(*p)),
                "{} objective {p:?} blocked",
                z.name
            );
            assert_ne!(
                distances[(p[1] * z.width + p[0]) as usize],
                u16::MAX,
                "{} objective {p:?} unreachable",
                z.name
            );
        }
        assert!(z.lore.len() > 100);
        assert_eq!(c.quests.iter().filter(|q| q.zone == z.id).count(), 4);
        for kind in ["intro", "boss", "outro"] {
            assert!(c.scenes.iter().any(|s| s.id == format!("{kind}-{}", z.id)));
        }
    }
    for s in &c.scenes {
        assert!(!s.beats.is_empty());
        for b in &s.beats {
            assert!(b.duration >= 120);
            assert!((0.5..=2.0).contains(&b.zoom));
        }
    }
}
#[test]
fn six_slot_migration_preserves_v06_identity_items_and_loadout() {
    let mut old = Game::new_offline("");
    old.save.journey.items.truncate(3);
    old.save.journey.equipped.truncate(3);
    let original = serde_json::to_value(&old.save.journey.items).unwrap();
    let identity = old.save.secret;
    let mut value = serde_json::to_value(&old.save).unwrap();
    value.as_object_mut().unwrap().remove("frontier");
    let restored = Game::new_offline(&value.to_string());
    assert!(!restored.storage_error);
    assert_eq!(restored.save.secret, identity);
    assert_eq!(
        serde_json::to_value(&restored.save.journey.items[..3]).unwrap(),
        original
    );
    assert_eq!(restored.save.journey.equipped.len(), 6);
    assert_eq!(restored.save.journey.items.len(), 6);
    let again = Game::new_offline(&restored.snapshot());
    assert_eq!(again.save.journey.items.len(), 6);
    assert_eq!(again.save.journey.equipped, restored.save.journey.equipped);
}
#[test]
fn timeline_resume_pause_and_skip_are_exact_and_do_not_advance_combat() {
    let mut g = game();
    g.ui_action("f:leave");
    g.save.frontier.seen.clear();
    g.ui_action("f:travel:0");
    assert_eq!(g.screen, 53);
    ticks(&mut g, 45);
    assert_eq!(run(&g).tick, 0);
    let elapsed = run(&g).scene.as_ref().unwrap().elapsed;
    g.ui_action("pause");
    ticks(&mut g, 300);
    assert_eq!(run(&g).scene.as_ref().unwrap().elapsed, elapsed);
    let mut restored = Game::new_offline(&g.snapshot());
    assert!(!restored.storage_error);
    restored.ui_action("continue");
    assert_eq!(restored.screen, 54);
    restored.ui_action("f:resume");
    assert_eq!(restored.screen, 53);
    assert_eq!(run(&restored).scene.as_ref().unwrap().elapsed, elapsed);
    restored.ui_action("f:skip");
    assert_eq!(restored.screen, 52);
    assert!(run(&restored).scene.is_none());
}
#[test]
fn combo_has_three_quick_strikes_then_a_heavy_and_input_survives_hitstop() {
    let mut g = game();
    let r = g.save.frontier.run.as_mut().unwrap();
    r.monsters.clear();
    r.hitstop = 3;
    g.controls(0, 0, 0, 0, true, false, false);
    ticks(&mut g, 1);
    g.controls(0, 0, 0, 0, false, false, false);
    ticks(&mut g, 3);
    assert_eq!(run(&g).player.state, State::Windup);
    for _ in 0..150 {
        g.controls(0, 0, 0, 0, true, false, false);
        ticks(&mut g, 1);
    }
    assert!(run(&g).player.heavy_count >= 1);
    assert!(run(&g).player.heavy_count <= 3);
    g.ui_action("f:pause");
    let snapshot = serde_json::to_string(run(&g)).unwrap();
    ticks(&mut g, 300);
    assert_eq!(serde_json::to_string(run(&g)).unwrap(), snapshot);
}
#[test]
fn stance_changes_have_real_tradeoffs_and_guard_break_is_heavy_only() {
    assert!(Stance::Assault.damage() > Stance::Balanced.damage());
    assert!(Stance::Assault.defense() > 100);
    assert!(Stance::Bulwark.defense() < 100);
    assert!(Stance::Bulwark.speed() < 100);
    fn damage(combo: u8) -> i32 {
        let mut g = game();
        let r = g.save.frontier.run.as_mut().unwrap();
        let pos = Point {
            x: r.player.pos.x + 150,
            y: r.player.pos.y,
        };
        r.monsters = vec![Monster::new(0, 2, pos, 0)];
        r.player.combo = combo;
        r.player.combo_window = 60;
        let hp = r.monsters[0].hp;
        g.controls(0, 0, 0, 0, true, false, false);
        for _ in 0..20 {
            ticks(&mut g, 1);
            if run(&g).monsters[0].hp < hp {
                break;
            }
        }
        hp - run(&g).monsters[0].hp
    }
    assert!(damage(3) > damage(0) * 3);
}
#[test]
fn missions_acceptance_rewards_and_save_reload_are_idempotent() {
    let mut g = game();
    g.save.frontier.progress(0, "kills", 4);
    assert_eq!(g.save.frontier.quests["hunt-0"].value, 0);
    g.ui_action("f:accept:hunt-0");
    g.save.frontier.progress(0, "kills", 100);
    let before = g.save.journey.dust;
    g.ui_action("f:claim:hunt-0");
    let paid = g.save.journey.dust;
    assert!(paid > before);
    let items = g.save.journey.items.len();
    g.ui_action("f:claim:hunt-0");
    assert_eq!(g.save.journey.dust, paid);
    assert_eq!(g.save.journey.items.len(), items);
    let mut g = Game::new_offline(&g.snapshot());
    g.ui_action("f:claim:hunt-0");
    assert_eq!(g.save.journey.dust, paid);
    assert!(g.save.frontier.quests["hunt-0"].claimed);
    g.ui_action("f:leave");
    g.ui_action("f:travel:2");
    assert!(g.save.frontier.run.is_none());
}
#[test]
fn inventory_sort_reorder_equipment_and_snapshot_do_not_heal() {
    let mut g = game();
    g.save.frontier.run.as_mut().unwrap().player.hp = 77;
    g.ui_action("inventory");
    assert_eq!(g.screen, 40);
    let item = g.save.journey.make_item(9, 3);
    let id = item.id;
    g.save.journey.store(item);
    g.ui_action(&format!("equip:{id}"));
    assert_eq!(run(&g).player.hp, 77);
    assert_eq!(g.save.journey.equipped[3], id);
    g.ui_action("bag_sort");
    let first = g.save.journey.items[0].id;
    let last = g.save.journey.items.last().unwrap().id;
    g.ui_action(&format!("bag_move:{first}:{last}"));
    assert_eq!(g.save.journey.items[0].id, last);
    g.ui_action("inventory_back");
    assert_eq!(g.screen, 54);
    let restored = Game::new_offline(&g.snapshot());
    assert_eq!(run(&restored).player.hp, 77);
    assert_eq!(restored.save.journey.equipped[3], id);
}
#[test]
fn corrupted_frontier_checkpoints_fail_closed() {
    let g = game();
    let mut value: serde_json::Value = serde_json::from_str(&g.snapshot()).unwrap();
    value["frontier"]["run"]["zone"] = serde_json::json!(99);
    let mut restored = Game::new_offline(&value.to_string());
    assert!(restored.storage_error);
    assert!(serde_json::from_str::<serde_json::Value>(&restored.presentation(584)).is_ok());
    let mut value: serde_json::Value = serde_json::from_str(&g.snapshot()).unwrap();
    value["frontier"]["run"]["explored"] = serde_json::json!([]);
    assert!(Game::new_offline(&value.to_string()).storage_error);
}
#[test]
fn all_enemy_archetypes_act_and_boss_changes_phase() {
    for kind in 0..5 {
        let mut g = game();
        let r = g.save.frontier.run.as_mut().unwrap();
        let pos = Point {
            x: r.player.pos.x,
            y: r.player.pos.y + if kind == 1 { 700 } else { 250 },
        };
        let mut m = Monster::new(7, kind, pos, 0);
        m.active = true;
        m.cooldown = 0;
        if kind == 4 {
            m.hp = m.max_hp / 4;
        }
        r.monsters = vec![m];
        let hp = r.player.hp;
        ticks(&mut g, 100);
        let r = run(&g);
        assert!(
            r.player.hp < hp || !r.projectiles.is_empty(),
            "kind {kind} did not attack"
        );
        if kind == 4 {
            assert_eq!(r.monsters[0].phase, 2);
            assert!(r.next_monster > 17);
        }
    }
}
/// Complete all authored maps through movement, attacks and interaction commands.
/// No teleports, stat changes, forced kills or objective mutation.
#[test]
fn campaign_is_winnable_and_all_twelve_missions_can_be_claimed() {
    let mut g = game();
    let mut total_ticks = 0;
    for zone in 0..3u8 {
        if zone > 0 {
            g.ui_action(&format!("f:travel:{zone}"));
            g.ui_action("f:skip");
        }
        assert_eq!(run(&g).zone, zone);
        for kind in ["hunt", "relic", "rescue"] {
            g.ui_action(&format!("f:accept:{kind}-{zone}"));
        }
        let z = data::zone(zone);
        let objectives: Vec<_> = z
            .relics
            .iter()
            .map(|p| ("relic", *p))
            .chain([("rescue", z.rescue)])
            .chain(z.beacons.iter().map(|p| ("beacon", *p)))
            .chain([("boss", z.boss)])
            .collect();
        for (kind, goal) in objectives {
            for step in 0..16000 {
                total_ticks += 1;
                if g.screen == 53 {
                    g.ui_action("f:skip");
                }
                assert!(
                    run(&g).player.hp > 0,
                    "zone {zone}, {kind} {goal:?}, step {step}, hp=0, kills={}",
                    run(&g).kills
                );
                if run(&g).victory {
                    break;
                }
                let r = run(&g);
                let player = &r.player;
                let nearest = r
                    .monsters
                    .iter()
                    .filter(|m| {
                        m.active && m.hp > 0 && player.pos.dist2(m.pos) < (9 * UNIT as i64).pow(2)
                    })
                    .min_by_key(|m| player.pos.dist2(m.pos));
                let fighting = nearest.is_some();
                let target = nearest.map(|m| m.pos).unwrap_or(world::at(goal));
                let flow = world::field(z, target);
                let dest = world::seek(z, player.pos, target, &flow);
                let (dx, dy) = world::direction(player.pos, dest, 1024);
                let dist = player.pos.dist2(target);
                let attack = fighting && dist < (UNIT as i64 * 2).pow(2);
                let move_now = dist
                    > if fighting {
                        (UNIT as i64).pow(2)
                    } else {
                        (300i64).pow(2)
                    };
                let heal = player.hp < player.max_hp / 2 && player.flasks > 0;
                let skill = attack && player.skill_cd == 0;
                if heal {
                    g.ui_action("f:heal");
                }
                g.controls(
                    if move_now { dx as i16 } else { 0 },
                    if move_now { dy as i16 } else { 0 },
                    0,
                    0,
                    attack,
                    false,
                    skill,
                );
                ticks(&mut g, 1);
                if !fighting && run(&g).player.pos.dist2(world::at(goal)) < (UNIT as i64 * 2).pow(2)
                {
                    g.ui_action("f:interact");
                    if kind != "boss" {
                        break;
                    }
                }
                assert!(
                    step < 15999,
                    "stuck zone={zone} kind={kind} goal={goal:?} pos={:?}",
                    run(&g).player.pos
                );
            }
        }
        if g.screen == 53 {
            g.ui_action("f:skip");
        }
        assert!(run(&g).victory, "zone {zone} did not complete");
        assert!(run(&g).phase_events >= 2);
        for kind in ["main", "hunt", "relic", "rescue"] {
            let id = format!("{kind}-{zone}");
            g.ui_action(&format!("f:claim:{id}"));
            assert!(
                g.save.frontier.quests[&id].claimed,
                "unclaimed {id}: {}",
                g.save.frontier.quests[&id].value
            );
        }
        assert!(g.save.frontier.valid());
        assert!(run(&g).effects.len() <= 64);
        assert!(run(&g).projectiles.len() <= 128);
        assert!(run(&g).monsters.len() <= 48);
        let snapshot = g.snapshot();
        assert!(snapshot.len() < 250_000);
        g = Game::new_offline(&snapshot);
        assert!(!g.storage_error);
        g.ui_action("f:leave");
    }
    assert!(g.save.frontier.bosses.iter().all(|b| *b));
    eprintln!(
        "Campaign complete: {total_ticks} simulation ticks, 12 missions, {} items",
        g.save.journey.items.len()
    );
}

#[test]
fn journal_inventory_and_codex_return_to_their_source_without_resuming_combat() {
    let mut g = Game::new_offline("");
    g.ui_action("f:atlas");
    g.ui_action("inventory");
    g.ui_action("inventory_back");
    assert_eq!(g.screen, 50);
    g.ui_action("f:travel:0");
    g.ui_action("f:skip");
    g.ui_action("inventory");
    g.ui_action("codex");
    g.ui_action("codex_back");
    assert_eq!(g.screen, 54);
    let r = serde_json::to_string(run(&g)).unwrap();
    ticks(&mut g, 300);
    assert_eq!(serde_json::to_string(run(&g)).unwrap(), r);
    g.ui_action("home");
    g.ui_action("contract:0");
    assert!(g.battle.is_none());
    assert!(g.save.frontier.run.is_some());
}

#[test]
fn checkpoint_replay_produces_the_same_combat_state() {
    let mut a = game();
    a.controls(1024, -1024, 0, 0, false, false, false);
    ticks(&mut a, 120);
    a.ui_action("f:pause");
    let mut b = Game::new_offline(&a.snapshot());
    a.ui_action("f:resume");
    b.ui_action("f:resume");
    for i in 0..600 {
        let (mx, my) = if i % 160 < 80 { (1024, 0) } else { (0, 1024) };
        for g in [&mut a, &mut b] {
            g.controls(mx, my, 0, 0, i % 5 < 3, i % 55 == 0, i % 181 == 0);
            g.tick(12345);
        }
    }
    assert_eq!(
        serde_json::to_value(run(&a)).unwrap(),
        serde_json::to_value(run(&b)).unwrap()
    );
}

#[test]
fn ten_minutes_of_boss_ai_keeps_transient_collections_bounded() {
    // Synthetic endurance fixture: immunity prevents the harness from ending the stress run.
    // Enemy attacks, projectiles, phases and expiration use the real simulation.
    let mut g = game();
    let r = g.save.frontier.run.as_mut().unwrap();
    r.monsters.retain(|m| m.kind == 4);
    let boss = &mut r.monsters[0];
    boss.active = true;
    boss.pos = Point {
        x: r.player.pos.x,
        y: r.player.pos.y + 500,
    };
    boss.hp = boss.max_hp / 4;
    for _ in 0..18_000 {
        let r = g.save.frontier.run.as_mut().unwrap();
        r.player.invulnerable = 100;
        r.step(Controls::default());
        assert!(r.monsters.len() <= 48);
        assert!(r.projectiles.len() <= 128);
        assert!(r.effects.len() <= 64);
    }
    assert!(run(&g).tick > 17_000);
    assert!(g.snapshot().len() < 100_000);
}
