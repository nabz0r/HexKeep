use hk_core::Game;
use hk_crown::{EdictKind, Scope};
use hk_proto::*;
#[test]
fn v01_save_migrates_and_m7_persists() {
    let mut g = Game::new("", true);
    g.save.created = true;
    let mut old: serde_json::Value = serde_json::from_str(&g.snapshot()).unwrap();
    old.as_object_mut().unwrap().remove("expansion");
    let mut migrated = Game::new(&old.to_string(), true);
    assert!(!migrated.storage_error);
    migrated.now = migrated.save.expansion.authority.genesis.body.created + 1;
    migrated.save.expansion.campaign.xp = 4000;
    migrated.save.expansion.campaign.claim();
    migrated
        .save
        .expansion
        .campaign
        .buy_dev(&migrated.save.secret, 0, migrated.now)
        .unwrap();
    migrated.save.expansion.campaign.claim();
    assert_eq!(migrated.save.expansion.campaign.owned.len(), 82);
    let restored = Game::new(&migrated.snapshot(), true);
    assert_eq!(
        restored.save.expansion.campaign.owned,
        migrated.save.expansion.campaign.owned
    );
}
#[test]
fn throne_ban_propagation_rollback_and_season() {
    let mut a = Game::new("", true);
    a.save.created = true;
    a.now = a.save.expansion.authority.genesis.body.created + 1;
    a.throne_unlock();
    let throne = a.save.expansion.throne.unwrap();
    let cell = a.save.world.current;
    a.save.ledger.append(
        &a.save.secret,
        cell,
        hk_ledger::Kind::Foundation,
        Realm::Aurelon,
        0,
        a.now,
    );
    let first = a
        .save
        .expansion
        .authority
        .seal(&throne, &a.save.ledger, a.now)
        .unwrap();
    a.now += 1;
    let nom = hk_crypto::public(&a.save.secret);
    a.crown_action(EdictKind::Ban {
        nom,
        scope: Scope::All,
        until: 0,
    });
    assert!(!a.save.expansion.authority.banned(&nom, Scope::All, a.now));
    a.save
        .expansion
        .authority
        .seal(&throne, &a.save.ledger, a.now)
        .unwrap();
    assert!(a.save.expansion.authority.banned(&nom, Scope::All, a.now));
    a.now += 1;
    a.crown_action(EdictKind::Rollback {
        cells: vec![cell],
        seal: first,
    });
    a.crown_action(EdictKind::SeasonKey {
        season: 2,
        key: [6; 32],
    });
    a.save
        .expansion
        .authority
        .seal(&throne, &a.save.ledger, a.now)
        .unwrap();
    a.apply_canonical();
    assert!(!a.save.expansion.campaign.chronicle.is_empty());
    a.export_authority();
    let mut b = Game::new("", true);
    b.import_exchange(&a.last_proof).unwrap();
    assert_eq!(
        a.save.expansion.authority.head(),
        b.save.expansion.authority.head()
    );
    assert!(b.save.expansion.authority.banned(&nom, Scope::All, a.now));
    assert_eq!(b.save.world.cells[&cell].bastion, Some(Realm::Aurelon));
}
#[test]
fn siege_gate_capture_and_contestation() {
    let mut b = hk_sim::Battle::new(81, Realm::Aurelon, Role::Foudre, 9, false);
    b.begin_siege(Realm::Aurelon, 20);
    b.obstacles.clear();
    for f in &mut b.fighters {
        f.bot = false;
        f.pos = Vec2::new(2 * UNIT, 2 * UNIT);
    }
    b.fighters[0].pos = Vec2::new(16 * UNIT, 8 * UNIT);
    let shoot = Input {
        aim_x: 1024,
        shoot: true,
        ..Default::default()
    };
    for _ in 0..350 {
        b.step(&[(0, shoot)]);
    }
    assert_eq!(b.siege.as_ref().unwrap().gate_hp, 0);
    b.fighters[0].pos = Vec2::new(24 * UNIT, 8 * UNIT);
    for _ in 0..1799 {
        b.step(&[]);
    }
    assert!(!b.finished);
    b.fighters[1].pos = b.fighters[0].pos;
    b.fighters[1].realm = Realm::Skarn;
    b.step(&[]);
    assert_eq!(b.siege.as_ref().unwrap().capture, 0);
    b.fighters[1].pos = Vec2::new(2 * UNIT, 2 * UNIT);
    for _ in 0..1800 {
        b.step(&[]);
    }
    assert!(b.siege.as_ref().unwrap().captured);
    assert!(b.finished);
}
#[test]
fn all_expansion_screens_at_every_supported_width() {
    let mut g = Game::new("", true);
    g.save.created = true;
    for screen in 20..=33 {
        g.screen = screen;
        for w in [400, 520, 584, 640] {
            let pixels = g.frame(w);
            assert_eq!(pixels.len(), w as usize * 240 * 4);
        }
    }
}
#[test]
fn purchased_cosmetics_cannot_change_combat() {
    let mut a = Game::new("", true);
    let mut b = Game::new(&a.snapshot(), true);
    b.save
        .expansion
        .campaign
        .buy_dev(&b.save.secret, 2, 1)
        .unwrap();
    b.save.expansion.campaign.equipped = "skin_givre".into();
    a.start_battle(1);
    b.start_battle(1);
    for tick in 0..180 {
        a.tick(tick / 30);
        b.tick(tick / 30);
    }
    assert_eq!(a.battle.unwrap().hash(), b.battle.unwrap().hash());
}
#[test]
fn cosmetic_edicts_wait_for_seal_and_revoke() {
    let mut g = Game::new("", true);
    g.now = g.save.expansion.authority.genesis.body.created + 1;
    g.throne_unlock();
    let nom = hk_crypto::public(&g.save.secret);
    g.crown_action(EdictKind::GrantCosmetic {
        nom,
        item: "skin_pierre".into(),
    });
    g.apply_canonical();
    assert!(!g.save.expansion.campaign.owned.contains("skin_pierre"));
    let t = g.save.expansion.throne.unwrap();
    g.save
        .expansion
        .authority
        .seal(&t, &g.save.ledger, g.now)
        .unwrap();
    g.apply_canonical();
    assert!(g.save.expansion.campaign.owned.contains("skin_pierre"));
    g.crown_action(EdictKind::RevokeCosmetic {
        nom,
        item: "skin_pierre".into(),
    });
    g.save
        .expansion
        .authority
        .seal(&t, &g.save.ledger, g.now)
        .unwrap();
    g.apply_canonical();
    assert!(!g.save.expansion.campaign.owned.contains("skin_pierre"));
}
#[test]
fn thirty_minute_partition_reconciles() {
    let mut a = Game::new("", true);
    a.now = a.save.expansion.authority.genesis.body.created + 1801;
    a.throne_unlock();
    let mut b = Game::new("", true);
    let cell = a.save.world.current;
    for minute in 0..30 {
        a.save.ledger.append(
            &a.save.secret,
            cell,
            hk_ledger::Kind::Watch,
            Realm::Aurelon,
            60,
            a.now - 1800 + minute * 60,
        );
        b.save.ledger.append(
            &b.save.secret,
            cell,
            hk_ledger::Kind::Watch,
            Realm::Skarn,
            60,
            a.now - 1800 + minute * 60,
        );
    }
    let mut first = a.save.ledger.clone();
    let mut second = b.save.ledger.clone();
    first.merge(&second);
    second.merge(&a.save.ledger);
    assert_eq!(
        first.fold("dev", &Default::default()),
        second.fold("dev", &Default::default())
    );
    a.save.ledger = first;
    let t = a.save.expansion.throne.unwrap();
    a.save
        .expansion
        .authority
        .seal(&t, &a.save.ledger, a.now)
        .unwrap();
    b.save.ledger = second;
    b.save
        .expansion
        .authority
        .merge(&a.save.expansion.authority, &b.save.ledger)
        .unwrap();
    assert_eq!(
        a.save.expansion.authority.head(),
        b.save.expansion.authority.head()
    );
}
#[test]
fn balance_editor_changes_only_after_seal() {
    let mut g = Game::new("", true);
    g.now = g.save.expansion.authority.genesis.body.created + 1;
    g.throne_unlock();
    g.screen = 32;
    g.tap(100, 62);
    assert_eq!(g.screen, 36);
    g.tap(450, 107);
    g.tap(400, 10);
    g.start_battle(1);
    assert_eq!(g.battle.as_ref().unwrap().fighters[0].hp, 100);
    let t = g.save.expansion.throne.unwrap();
    g.save
        .expansion
        .authority
        .seal(&t, &g.save.ledger, g.now)
        .unwrap();
    g.apply_canonical();
    g.start_battle(1);
    assert_eq!(g.battle.as_ref().unwrap().fighters[0].hp, 105);
}
