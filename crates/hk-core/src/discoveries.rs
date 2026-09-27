//! PvE encounter direction and discoveries, deliberately outside the shared combat protocol.
use super::*;
#[derive(Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: u8,
    pub pos: Vec2,
    pub kind: u8,
    pub name: String,
    pub opened: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u8,
    pub kind: u8,
    pub name: String,
    pub elite: bool,
    pub defeated: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Hazard {
    pub pos: Vec2,
    pub radius: i32,
    pub born: u32,
    pub strike: u32,
}
pub const NAMES: [&str; 7] = [
    "Rôdeur de cendre",
    "Tisseur de brume",
    "Sentinelle brisée",
    "Oracle des ronces",
    "Traqueur de givre",
    "Pilleur de braises",
    "Gardien des chemins",
];
pub const TACTICS: [&str; 7] = [
    "S'arrête, puis bondit. Esquive de côté quand ses yeux s'allument.",
    "Tire à distance. Rapproche-toi en utilisant les ruines comme abri.",
    "Prépare une salve en éventail. Quitte son cône avant la frappe.",
    "Marque le sol et invoque des rôdeurs. Sors des cercles violets.",
    "Contourne et tire de côté. Garde un obstacle entre vous.",
    "S'enfuit avec un objet rare. Coupe sa route avant de l'attaquer.",
    "À mi-vie, ses secousses accélèrent. Quitte les cercles avant leur fermeture.",
];
pub fn secret_entries(j: &adventure::Journey) -> Vec<serde_json::Value> {
    let titles = [
        "Le nom sous la cendre",
        "La première lanterne",
        "Neuf vies, un thé",
        "La quatrième note",
        "Le veilleur sans couronne",
        "Le raccourci de Minuit",
        "Le ciel sous nos pas",
        "Les trois mains",
        "La clé sans serrure",
    ];
    let stories = [
        "Éline n’a pas perdu la carte : elle l’a laissée vide pour que chacun puisse tracer sa route. La cloche gardait son nom pendant son absence.",
        "Le premier veilleur n’avait ni épée ni royaume. Il a seulement refusé de laisser son voisin rentrer seul dans la nuit.",
        "Minuit a neuf vies, mais une seule tasse. Il l’offre aux voyageurs qui prennent le temps de s’arrêter. Le thé refroidit. La veille, jamais.",
        "Trois royaumes entendaient trois notes différentes. La quatrième n’apparaît que lorsque quelqu’un écoute les autres.",
        "Ici repose une couronne renversée. Son porteur préféra ouvrir les portes du refuge plutôt que gouverner ses ruines.",
        "Minuit connaît tous les raccourcis. Il emprunte toujours le plus long : on y croise davantage de gens à raccompagner.",
        "Le verre n’enferme pas les étoiles. Il reflète les lanternes des veilleurs disparus, qui nous montrent encore le chemin du retour.",
        "Aube, givre et sève ont forgé ensemble cette clef. Aucune main ne pouvait la tenir seule. Aucun royaume ne peut veiller seul.",
        "Minuit laisse tomber une clé sans serrure. Éline sourit : certaines portes attendent d’être imaginées. Une autre histoire commence ici.",
    ];
    (0..9).map(|i| { let key=format!("{}:{}",i/3,i%3+2); let found=j.secrets.contains(&key); serde_json::json!({"key":key,"found":found,"title":if found {titles[i]} else {"Une mémoire attend"},"text":if found {stories[i]} else {["Écoute la cloche de cette région.","Survis au défi de l’autel de cette région.","Trouve Minuit, le chat à la lanterne."][i%3]},"region":(["Jardins de cendre","Val des cloches","Hauts de verre"][i/3])}) }).collect()
}
pub fn configure(f: &mut Fighter, kind: u8, tier: u8, elite: bool) -> Enemy {
    let kind = kind.min(6);
    f.role = match kind {
        0 | 4 | 5 => Role::Foudre,
        1 | 3 => Role::Lien,
        _ => Role::Rempart,
    };
    f.hp = [65, 80, 120, 100, 70, 90, 420][kind as usize]
        + tier as i32 * if kind == 6 { 140 } else { 25 };
    if elite {
        f.hp = f.hp * 3 / 2;
    }
    f.max_hp = f.hp;
    f.power = if kind == 5 {
        0
    } else {
        35 + tier as i32 * 12 + if elite { 8 } else { 0 }
    };
    f.armor = if kind == 2 || kind == 6 {
        35 + tier as i32 * 15
    } else {
        10
    };
    f.invulnerable = 20;
    Enemy {
        id: f.id,
        kind,
        name: if elite {
            format!("{} · Élite", NAMES[kind as usize])
        } else {
            NAMES[kind as usize].into()
        },
        elite,
        defeated: false,
    }
}
pub fn prepare(b: &mut Battle, run: &mut Expedition, tier: u8) {
    run.tier = tier;
    for f in b.fighters.iter_mut().skip(1) {
        let kind = (f.id - 1) % 5;
        run.enemies.push(configure(f, kind, tier, false));
        f.pos = hk_sim::navigation::safe_position(f.pos, &b.obstacles);
    }
    let variants = [
        [(5, 2), (12, 11), (24, 3), (26, 14), (2, 6), (19, 14)],
        [(3, 3), (13, 6), (26, 3), (18, 14), (5, 14), (27, 8)],
        [(5, 14), (12, 2), (26, 13), (18, 6), (3, 3), (27, 3)],
    ];
    for (i, (x, y)) in variants[(b.seed % 3) as usize].iter().enumerate() {
        run.sites.push(Site {
            id: i as u8,
            pos: hk_sim::navigation::safe_position(Vec2::new(x * UNIT, y * UNIT), &b.obstacles),
            kind: [0, 0, 1, 2, 3, 4][i],
            name: [
                "Coffre du voyageur",
                "Cache du passeur",
                "Source de lune",
                "Cloche sans voix",
                "Autel des téméraires",
                "Minuit, le chat veilleur",
            ][i]
                .into(),
            opened: false,
        });
    }
}
pub fn inputs(b: &Battle, run: &Expedition, player: Input) -> Vec<(u8, Input)> {
    let mut inputs = vec![(0, player)];
    let p = &b.fighters[0];
    for (i, f) in b.fighters.iter().enumerate().skip(1) {
        if f.hp <= 0 {
            continue;
        }
        let kind = run
            .enemies
            .iter()
            .find(|e| e.id == f.id)
            .map(|e| e.kind)
            .unwrap_or(0);
        let mut input = b.bot_input(i);
        let phase = (b.tick + f.id as u32 * 17) % 150;
        let distance = f.pos.dist2(p.pos);
        let visible = hk_sim::navigation::clear_line(f.pos, p.pos, &b.obstacles, 24);
        let toward = Vec2::new(p.pos.x - f.pos.x, p.pos.y - f.pos.y).scaled(1024);
        if distance > (11 * UNIT).pow(2) as i64 && run.wave == 0 {
            // Distant packs patrol their territory instead of all piling onto the player.
            input = Input::default();
        } else {
            match kind {
                0 => {
                    if phase >= 100 && phase < 123 {
                        input.move_x = 0;
                        input.move_y = 0;
                    }
                    input.shoot = visible && distance < (4 * UNIT).pow(2) as i64 && phase >= 123;
                    input.dash = phase == 123 && distance < (6 * UNIT).pow(2) as i64;
                    input.skill = false;
                }
                1 | 3 => {
                    input.shoot = visible && phase >= 110;
                    input.skill = false;
                    if phase >= 80 {
                        input.move_x = 0;
                        input.move_y = 0;
                    }
                }
                2 | 6 => {
                    input.move_x /= 2;
                    input.move_y /= 2;
                    if phase >= 90 {
                        input.move_x = 0;
                        input.move_y = 0;
                    }
                    input.shoot = visible && phase >= 125;
                    input.skill = false;
                }
                4 => {
                    input.shoot = visible && phase >= 100;
                    input.skill = false;
                    input.dash = phase == 80 && distance < (4 * UNIT).pow(2) as i64;
                }
                5 => {
                    let corner = Vec2::new(
                        if p.pos.x < 15 * UNIT {
                            27 * UNIT
                        } else {
                            3 * UNIT
                        },
                        if p.pos.y < 8 * UNIT {
                            13 * UNIT
                        } else {
                            3 * UNIT
                        },
                    );
                    let v = hk_sim::navigation::direction(f.pos, corner, &b.obstacles);
                    input.move_x = v.x as i16;
                    input.move_y = v.y as i16;
                    input.shoot = false;
                    input.skill = false;
                    input.dash = phase == 120;
                }
                _ => {}
            }
            if kind != 5 {
                input.aim_x = toward.x as i16;
                input.aim_y = toward.y as i16;
            }
        }
        inputs.push((f.id, input));
    }
    inputs
}
fn award(j: &mut adventure::Journey, run: &mut Expedition, seed: u64, rarity: u8) -> String {
    let item = j.make_item(seed, rarity.min(3));
    let name = if j.items.len() >= 60 {
        format!(
            "{} → {} braises (sac plein)",
            item.name,
            4 * (item.rarity as u32 + 1)
        )
    } else {
        item.name.clone()
    };
    run.loot_count += 1;
    j.store(item);
    if j.recent.len() > 12 {
        j.recent.remove(0);
    }
    name
}
pub fn interact(
    b: &mut Battle,
    run: &mut Expedition,
    j: &mut adventure::Journey,
) -> Option<String> {
    let p = &b.fighters[0];
    if p.hp <= 0 || b.finished {
        return None;
    }
    let index = run
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| !s.opened && s.pos.dist2(p.pos) < (2 * UNIT).pow(2) as i64)
        .min_by_key(|(_, s)| s.pos.dist2(p.pos))
        .map(|(i, _)| i)?;
    let site = run.sites[index].clone();
    run.sites[index].opened = true;
    let seed = b.seed.wrapping_add(107 * site.id as u64 + 17 * j.next_id);
    match site.kind {
        0 => {
            let name = award(j, run, seed, if site.id == 1 { 2 } else { 1 });
            Some(format!("Butin trouvé : {name}."))
        }
        1 => {
            let p = &mut b.fighters[0];
            p.hp = (p.hp + p.max_hp / 3).min(p.max_hp);
            run.flasks = (run.flasks + 1).min(2);
            Some("La source restaure ta vie et une fiole. Reprends ton souffle.".into())
        }
        2 | 4 => {
            let key = format!("{}:{}", adventure::region(run.cell), site.kind);
            let new = j.secrets.insert(key);
            let name = award(j, run, seed, if new { 3 } else { 1 });
            Some(if site.kind == 4 {
                format!(
                    "Minuit ronronne : « Le thé refroidit. La veille, jamais. » Tu trouves {name}."
                )
            } else {
                format!("La cloche livre une mémoire. Lis-la dans le carnet. Butin : {name}.")
            })
        }
        _ => {
            run.challenge_target = b.fighters[0].kills + 2;
            for offset in 0..2 {
                let id = b.fighters.len() as u8;
                let mut f = Fighter::new(
                    id,
                    Realm::from_index((b.fighters[0].realm.index() + 1) % 3),
                    Role::Foudre,
                    true,
                );
                f.pos = hk_sim::navigation::safe_position(
                    Vec2::new(
                        site.pos.x + (offset * 2 - 1) * UNIT * 2,
                        site.pos.y + 2 * UNIT,
                    ),
                    &b.obstacles,
                );
                run.enemies.push(configure(
                    &mut f,
                    if offset == 0 { 5 } else { 2 },
                    run.tier,
                    true,
                ));
                b.fighters.push(f);
            }
            Some("Défi : dissipe deux ombres supplémentaires. Un objet épique t’attend.".into())
        }
    }
}
pub fn tick(b: &mut Battle, run: &mut Expedition, j: &mut adventure::Journey) -> Option<String> {
    let mut message = None;
    for i in 0..3 {
        if run.caches[i] && !run.secured[i] {
            run.secured[i] = true;
            let name = award(
                j,
                run,
                b.seed.wrapping_add(i as u64 * 337 + j.next_id * 11),
                if i == 2 { 2 } else { 1 },
            );
            message = Some(format!("Cache ouverte · {name}."));
        }
    }
    let mut defeated = vec![];
    for e in &mut run.enemies {
        if !e.defeated && b.fighters.iter().any(|f| f.id == e.id && f.hp <= 0) {
            e.defeated = true;
            *j.bestiary.entry(e.kind).or_default() += 1;
            defeated.push((e.kind, e.elite));
        }
    }
    for (kind, elite) in defeated {
        if kind == 5 || elite || b.fighters[0].kills % 4 == 0 {
            let name = award(
                j,
                run,
                b.seed.wrapping_add(j.next_id * 73),
                if elite { 2 } else { 1 },
            );
            message = Some(format!("Butin de rencontre · {name}."));
        }
    }
    if run.challenge_target > 0 && b.fighters[0].kills >= run.challenge_target {
        run.challenge_target = 0;
        j.secrets
            .insert(format!("{}:3", adventure::region(run.cell)));
        let name = award(j, run, b.seed.wrapping_add(j.next_id * 137), 3);
        message = Some(format!("Défi accompli ! Mémoire dans le carnet · {name}."));
    }
    // Telegraphs are fixed in world space: leaving their circle is a real counterplay.
    if b.fighters[0].hp > 0 {
        let mut marks = vec![];
        for e in &run.enemies {
            let Some(f) = b.fighters.iter().find(|f| f.id == e.id && f.hp > 0) else {
                continue;
            };
            let interval = if e.kind == 6 && f.hp < f.max_hp / 2 {
                120
            } else {
                210
            };
            if (e.kind == 3 || e.kind == 6)
                && (b.tick + e.id as u32 * 31) % interval == 1
                && f.pos.dist2(b.fighters[0].pos) < (12 * UNIT).pow(2) as i64
            {
                marks.push(Hazard {
                    pos: b.fighters[0].pos,
                    radius: if e.kind == 6 {
                        2 * UNIT
                    } else {
                        UNIT + UNIT / 2
                    },
                    born: b.tick,
                    strike: b.tick + 36,
                });
            }
        }
        run.hazards.extend(marks);
    }
    for h in &run.hazards {
        let p = &mut b.fighters[0];
        if h.strike == b.tick
            && p.hp > 0
            && p.invulnerable == 0
            && p.pos.dist2(h.pos) < (h.radius as i64).pow(2)
        {
            p.hp = (p.hp - 20 - run.tier as i32 * 8).max(0);
            if p.hp == 0 {
                p.deaths += 1;
                p.respawn = 120;
                p.stride = Vec2::default();
                p.velocity = Vec2::default();
            }
        }
    }
    run.hazards.retain(|h| b.tick < h.strike + 15);
    // The oracle summons only once; this flag is part of the expedition, not wall-clock time.
    if !run.summoned
        && b.fighters.iter().any(|f| {
            f.hp > 0
                && f.hp < f.max_hp / 2
                && run.enemies.iter().any(|e| e.id == f.id && e.kind == 3)
        })
    {
        run.summoned = true;
        for n in 0..2 {
            let id = b.fighters.len() as u8;
            let mut f = Fighter::new(
                id,
                Realm::from_index((b.fighters[0].realm.index() + 1) % 3),
                Role::Foudre,
                true,
            );
            f.pos = hk_sim::navigation::safe_position(
                Vec2::new(15 * UNIT, (3 + n * 10) * UNIT),
                &b.obstacles,
            );
            run.enemies.push(configure(&mut f, 0, run.tier, false));
            b.fighters.push(f);
        }
        message = Some("L’oracle appelle des rôdeurs. Garde tes distances.".into());
    }
    if run.kind == 3
        && run.wave == 0
        && b.tick % 450 == 0
        && b.fighters.iter().filter(|f| f.hp > 0).count() < 10
    {
        for n in 0..2 {
            let id = b.fighters.len() as u8;
            let mut f = Fighter::new(
                id,
                Realm::from_index((b.fighters[0].realm.index() + 1) % 3),
                Role::Foudre,
                true,
            );
            f.pos = hk_sim::navigation::safe_position(
                Vec2::new(if n == 0 { 3 * UNIT } else { 26 * UNIT }, 8 * UNIT),
                &b.obstacles,
            );
            run.enemies
                .push(configure(&mut f, (b.tick / 450 % 5) as u8, run.tier, false));
            b.fighters.push(f);
        }
    }
    message
}
pub fn view(b: &Battle, run: &Expedition) -> serde_json::Value {
    let p = &b.fighters[0];
    let nearest = run
        .sites
        .iter()
        .filter(|s| !s.opened)
        .min_by_key(|s| s.pos.dist2(p.pos));
    let context = nearest.filter(|s| s.pos.dist2(p.pos) < (2 * UNIT).pow(2) as i64);
    let points = [
        Vec2::new(6 * UNIT, 4 * UNIT),
        Vec2::new(15 * UNIT, 12 * UNIT),
        Vec2::new(25 * UNIT, 5 * UNIT),
    ];
    let goal = if run.wave == 1 {
        b.fighters.get(1).map(|f| f.pos)
    } else if run.kind == 4 {
        nearest.map(|s| s.pos)
    } else if run.kind == 1 {
        b.fighters
            .iter()
            .skip(1)
            .filter(|f| f.hp > 0)
            .min_by_key(|f| f.pos.dist2(p.pos))
            .map(|f| f.pos)
    } else if run.kind == 3 {
        None
    } else {
        (0..3)
            .filter(|i| run.charges[*i] < 90)
            .min_by_key(|i| points[*i].dist2(p.pos))
            .map(|i| points[i])
    };
    let instruction = if run.wave == 1 {
        "Vaincs le gardien. Sors des cercles avant leur fermeture."
    } else {
        [
            "Rejoins les feux bleus. Reste dans leur cercle pour les rallumer.",
            "Dissipe six ombres. Les cercles rouges annoncent leurs attaques.",
            "Rejoins les pierres bleues pour retrouver leurs mémoires.",
            "Tiens 90 secondes. Utilise les abris, les fioles et la source.",
            "Explore cinq curiosités. Approche et utilise le bouton Interagir.",
        ][run.kind.min(4) as usize]
    };
    serde_json::json!({"instruction":instruction,"goal":goal,"distance":goal.map(|g|isqrt(g.dist2(p.pos) as u64)/256),"context":context,"opened":run.sites.iter().filter(|s|s.opened).count(),"duration":b.tick/30,"boss_since":run.boss_since})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn expedition() -> Game {
        let mut g = Game::new("", true);
        g.save.created = true;
        g.ui_action("contract:0");
        g
    }
    #[test]
    fn sites_reward_once_immediately_and_memories_survive_reload() {
        let mut g = expedition();
        let before = g.save.journey.items.len();
        let site = g
            .expedition
            .as_ref()
            .unwrap()
            .sites
            .iter()
            .find(|s| s.kind == 2)
            .unwrap()
            .clone();
        g.battle.as_mut().unwrap().fighters[0].pos = site.pos;
        g.ui_action("interact");
        assert_eq!(g.save.journey.items.len(), before + 1);
        assert_eq!(g.save.journey.secrets.len(), 1);
        g.ui_action("interact");
        assert_eq!(g.save.journey.items.len(), before + 1);
        let restored = Game::new(&g.snapshot(), true);
        assert_eq!(restored.save.journey.secrets, g.save.journey.secrets);
        assert_eq!(
            secret_entries(&restored.save.journey)
                .iter()
                .filter(|e| e["found"] == true)
                .count(),
            1
        );
    }
    #[test]
    fn found_caches_are_kept_on_retreat_and_not_awarded_twice() {
        let mut g = expedition();
        let n = g.save.journey.items.len();
        g.expedition.as_mut().unwrap().caches[0] = true;
        tick(
            g.battle.as_mut().unwrap(),
            g.expedition.as_mut().unwrap(),
            &mut g.save.journey,
        );
        assert_eq!(g.save.journey.items.len(), n + 1);
        tick(
            g.battle.as_mut().unwrap(),
            g.expedition.as_mut().unwrap(),
            &mut g.save.journey,
        );
        assert_eq!(g.save.journey.items.len(), n + 1);
        g.ui_action("home");
        assert_eq!(g.save.journey.items.len(), n + 1);
    }
    #[test]
    fn marked_ground_can_be_escaped_or_dodged_and_hits_only_once() {
        for counter in 0..3 {
            let mut g = expedition();
            let b = g.battle.as_mut().unwrap();
            let run = g.expedition.as_mut().unwrap();
            let pos = b.fighters[0].pos;
            b.tick = 80;
            run.hazards.push(Hazard {
                pos,
                radius: 256,
                born: 44,
                strike: 80,
            });
            b.fighters[0].invulnerable = if counter == 1 { 4 } else { 0 };
            if counter == 2 {
                b.fighters[0].pos.x += 512;
            }
            let hp = b.fighters[0].hp;
            tick(b, run, &mut g.save.journey);
            assert_eq!(b.fighters[0].hp, if counter == 0 { hp - 20 } else { hp });
            b.tick += 1;
            tick(b, run, &mut g.save.journey);
            assert_eq!(b.fighters[0].hp, if counter == 0 { hp - 20 } else { hp });
        }
    }
    #[test]
    fn full_bag_converts_loot_and_reports_that_conversion() {
        let mut g = expedition();
        while g.save.journey.items.len() < 60 {
            let i = g.save.journey.make_item(g.save.journey.next_id, 0);
            g.save.journey.store(i);
        }
        let dust = g.save.journey.dust;
        let message = award(&mut g.save.journey, g.expedition.as_mut().unwrap(), 5, 2);
        assert!(message.contains("sac plein"));
        assert_eq!(g.save.journey.items.len(), 60);
        assert_eq!(g.save.journey.dust, dust + 12);
    }
}
