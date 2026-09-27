//! Original integer movement, swept sliding and deterministic grid navigation.
use super::*;
use std::collections::VecDeque;
pub const RADIUS: i32 = 64;
pub fn solid(p: Vec2, o: &Obstacle, r: i32) -> bool {
    let x = p.x.clamp(o.x, o.x + o.w);
    let y = p.y.clamp(o.y, o.y + o.h);
    (p.x - x) as i64 * (p.x - x) as i64 + (p.y - y) as i64 * ((p.y - y) as i64) < (r * r) as i64
}
pub fn blocked(p: Vec2, obs: &[Obstacle], _air: bool) -> bool {
    p.x < RADIUS
        || p.y < RADIUS
        || p.x > WIDTH - RADIUS
        || p.y > HEIGHT - RADIUS
        || obs.iter().any(|o| solid(p, o, RADIUS))
}
pub fn slide(mut pos: Vec2, v: Vec2, obs: &[Obstacle]) -> Vec2 {
    let steps = (v.x.abs().max(v.y.abs()) / 24 + 1).min(128);
    for i in 0..steps {
        let dx = v.x * (i + 1) / steps - v.x * i / steps;
        let dy = v.y * (i + 1) / steps - v.y * i / steps;
        let whole = Vec2::new(pos.x + dx, pos.y + dy);
        if !blocked(whole, obs, false) {
            pos = whole;
            continue;
        }
        let next = Vec2::new(pos.x + dx, pos.y);
        if !blocked(next, obs, false) {
            pos = next;
        }
        let next = Vec2::new(pos.x, pos.y + dy);
        if !blocked(next, obs, false) {
            pos = next;
        }
    }
    pos
}
pub fn clear_line(a: Vec2, b: Vec2, obs: &[Obstacle], radius: i32) -> bool {
    // Integer slab intersection avoids hundreds of ray samples per AI query.
    // An expanded AABB is conservative around round corners, never a wall shortcut.
    const SCALE: i64 = 1 << 20;
    !obs.iter().any(|o| {
        let mut enter = 0i64;
        let mut exit = SCALE;
        for (start, end, low, high) in [
            (a.x, b.x, o.x - radius, o.x + o.w + radius),
            (a.y, b.y, o.y - radius, o.y + o.h + radius),
        ] {
            let delta = (end - start) as i64;
            if delta == 0 {
                if start < low || start > high {
                    return false;
                }
            } else {
                let t1 = (low - start) as i64 * SCALE / delta;
                let t2 = (high - start) as i64 * SCALE / delta;
                enter = enter.max(t1.min(t2));
                exit = exit.min(t1.max(t2));
                if enter > exit {
                    return false;
                }
            }
        }
        true
    })
}
pub fn safe_position(p: Vec2, obs: &[Obstacle]) -> Vec2 {
    if !blocked(p, obs, false) {
        return p;
    }
    (0..480)
        .map(|i| Vec2::new((i % 30) * UNIT + UNIT / 2, (i / 30) * UNIT + UNIT / 2))
        .filter(|v| !blocked(*v, obs, false))
        .min_by_key(|v| v.dist2(p))
        .unwrap_or(Vec2::new(UNIT, UNIT))
}
pub fn direction(from: Vec2, to: Vec2, obs: &[Obstacle]) -> Vec2 {
    // A swept circle can legally stop inside the corners of an expanded AABB.
    // The conservative ray must not trap that actor by rejecting every exit.
    let walk_line = |a: Vec2, b: Vec2, padding: i32| {
        if clear_line(a, a, obs, padding) && clear_line(b, b, obs, padding) {
            clear_line(a, b, obs, padding)
        } else {
            let steps = ((b.x - a.x).abs().max((b.y - a.y).abs()) / 16 + 1).max(1);
            (0..=steps).all(|i| {
                !blocked(
                    Vec2::new(a.x + (b.x - a.x) * i / steps, a.y + (b.y - a.y) * i / steps),
                    obs,
                    false,
                )
            })
        }
    };
    if walk_line(from, to, RADIUS + 8) {
        return Vec2::new(to.x - from.x, to.y - from.y).scaled(1024);
    }
    let cell = |v: Vec2| ((v.y / UNIT).clamp(0, 15) * 30 + (v.x / UNIT).clamp(0, 29)) as usize;
    let center = |i: usize| {
        Vec2::new(
            (i % 30) as i32 * UNIT + UNIT / 2,
            (i / 30) as i32 * UNIT + UNIT / 2,
        )
    };
    let start = cell(from);
    let end = cell(safe_position(to, obs));
    let mut prev = [usize::MAX; 480];
    let mut q = VecDeque::from([start]);
    prev[start] = start;
    while let Some(at) = q.pop_front() {
        if at == end {
            break;
        }
        let x = (at % 30) as i32;
        let y = (at / 30) as i32;
        for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if !(0..30).contains(&nx) || !(0..16).contains(&ny) {
                continue;
            }
            let next = (ny * 30 + nx) as usize;
            if prev[next] != usize::MAX || blocked(center(next), obs, false) {
                continue;
            }
            if !walk_line(
                if at == start { from } else { center(at) },
                center(next),
                RADIUS + 2,
            ) {
                continue;
            }
            prev[next] = at;
            q.push_back(next);
        }
    }
    if prev[end] == usize::MAX {
        return Vec2::default();
    }
    let mut path = vec![end];
    let mut at = end;
    while at != start {
        at = prev[at];
        path.push(at);
    }
    let dest = path
        .into_iter()
        .find(|i| walk_line(from, center(*i), RADIUS + 8))
        .map(center)
        .unwrap_or(to);
    Vec2::new(dest.x - from.x, dest.y - from.y).scaled(1024)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actor_can_leave_a_wall_margin_and_rounded_corner() {
        let obs = vec![Obstacle {
            x: 9 * UNIT,
            y: 12 * UNIT,
            w: 2 * UNIT,
            h: UNIT,
            kind: 0,
            life: 0,
        }];
        for start in [
            Vec2::new(9 * UNIT - 64, 12 * UNIT + 100),
            Vec2::new(9 * UNIT - 47, 12 * UNIT - 47),
        ] {
            assert!(!blocked(start, &obs, false));
            let mut p = start;
            let goal = Vec2::new(14 * UNIT, 14 * UNIT);
            for _ in 0..200 {
                p = slide(p, direction(p, goal, &obs).scaled(43), &obs);
                if p.dist2(goal) < 10000 {
                    break;
                }
            }
            assert!(p.dist2(goal) < 10000, "Actor remained trapped at {p:?}");
        }
    }
    #[test]
    fn swept_dash_cannot_tunnel() {
        let o = vec![Obstacle {
            x: 3 * UNIT,
            y: UNIT,
            w: 64,
            h: 5 * UNIT,
            kind: 0,
            life: 0,
        }];
        let p = slide(Vec2::new(2 * UNIT, 2 * UNIT), Vec2::new(4 * UNIT, UNIT), &o);
        assert!(p.x <= 3 * UNIT - RADIUS);
        assert!(p.y > 2 * UNIT);
    }
    #[test]
    fn route_goes_around_wall() {
        let o = vec![Obstacle {
            x: 10 * UNIT,
            y: 3 * UNIT,
            w: 2 * UNIT,
            h: 9 * UNIT,
            kind: 0,
            life: 0,
        }];
        let mut p = Vec2::new(8 * UNIT, 7 * UNIT);
        let goal = Vec2::new(14 * UNIT, 7 * UNIT);
        for _ in 0..400 {
            let v = direction(p, goal, &o).scaled(45);
            p = slide(p, v, &o);
            assert!(!blocked(p, &o, false));
            if p.dist2(goal) < 10000 {
                return;
            }
        }
        panic!("route stalled at {p:?}");
    }
}

#[cfg(test)]
mod gameplay_tests {
    use super::*;
    #[test]
    fn analogue_is_proportional_and_release_brakes() {
        let mut slow = Battle::new(0, Realm::Aurelon, Role::Foudre, 0, false);
        slow.obstacles.clear();
        slow.fighters[0].pos = Vec2::new(UNIT * 4, UNIT * 8);
        let mut fast = slow.clone();
        for _ in 0..30 {
            slow.step(&[(
                0,
                Input {
                    move_x: 512,
                    ..Default::default()
                },
            )]);
            fast.step(&[(
                0,
                Input {
                    move_x: 1024,
                    ..Default::default()
                },
            )]);
        }
        let a = slow.fighters[0].pos.x - UNIT * 4;
        let b = fast.fighters[0].pos.x - UNIT * 4;
        assert!(b > a * 18 / 10 && b < a * 22 / 10);
        fast.step(&[(0, Input::default())]);
        fast.step(&[(0, Input::default())]);
        let pos = fast.fighters[0].pos;
        fast.step(&[(0, Input::default())]);
        assert_eq!(pos, fast.fighters[0].pos);
    }
    #[test]
    fn every_spawn_is_walkable() {
        for seed in 0..30 {
            let b = Battle::new(seed, Realm::Aurelon, Role::Lien, 9, false);
            for f in b.fighters {
                assert!(!blocked(f.pos, &b.obstacles, false));
            }
        }
    }
    #[test]
    fn all_enemy_roles_can_navigate_a_barrier() {
        for role in 0..3 {
            let mut b = Battle::new(0, Realm::Aurelon, Role::Foudre, 1, false);
            b.obstacles = vec![Obstacle {
                x: 10 * UNIT,
                y: 3 * UNIT,
                w: 2 * UNIT,
                h: 9 * UNIT,
                kind: 0,
                life: 0,
            }];
            b.fighters[0].pos = Vec2::new(14 * UNIT, 7 * UNIT);
            b.fighters[1].pos = Vec2::new(8 * UNIT, 7 * UNIT);
            b.fighters[1].role = Role::from_index(role);
            for _ in 0..700 {
                let mut input = b.bot_input(1);
                input.shoot = false;
                input.skill = false;
                input.dash = false;
                b.step(&[(0, Input::default()), (1, input)]);
                assert!(!blocked(b.fighters[1].pos, &b.obstacles, false));
            }
            assert!(
                clear_line(b.fighters[0].pos, b.fighters[1].pos, &b.obstacles, 24),
                "role {role} never went around the wall"
            );
        }
    }
}
