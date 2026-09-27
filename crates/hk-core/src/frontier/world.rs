//! 48×32 authored regions, integer collision and bounded navigation.
use super::{data::Zone, entities::Point};
pub const UNIT: i32 = 256;
pub const CELLS: usize = 48 * 32;
pub fn at(p: [i32; 2]) -> Point {
    Point {
        x: p[0] * UNIT + UNIT / 2,
        y: p[1] * UNIT + UNIT / 2,
    }
}
pub fn blocked(z: &Zone, x: i32, y: i32) -> bool {
    x < 1
        || y < 1
        || x >= z.width - 1
        || y >= z.height - 1
        || z.obstacles
            .iter()
            .any(|r| x >= r[0] && y >= r[1] && x < r[0] + r[2] && y < r[1] + r[3])
}
pub fn free(z: &Zone, p: Point) -> bool {
    if p.x < UNIT || p.y < UNIT || p.x >= (z.width - 1) * UNIT || p.y >= (z.height - 1) * UNIT {
        return false;
    }
    [-65, 65].iter().all(|dx| {
        [-65, 65]
            .iter()
            .all(|dy| !blocked(z, (p.x + dx) / UNIT, (p.y + dy) / UNIT))
    })
}
pub fn walk(z: &Zone, p: &mut Point, dx: i32, dy: i32) {
    let nx = Point {
        x: p.x + dx,
        y: p.y,
    };
    if free(z, nx) {
        p.x = nx.x;
    }
    let ny = Point {
        x: p.x,
        y: p.y + dy,
    };
    if free(z, ny) {
        p.y = ny.y;
    }
}
pub fn direction(a: Point, b: Point, speed: i32) -> (i32, i32) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let d = ((dx as f64 * dx as f64 + dy as f64 * dy as f64).sqrt() as i32).max(1);
    (dx * speed / d, dy * speed / d)
}
pub fn reveal(z: &Zone, p: Point, cells: &mut [bool]) {
    let x = p.x / UNIT;
    let y = p.y / UNIT;
    for yy in (y - 4).max(0)..=(y + 4).min(z.height - 1) {
        for xx in (x - 4).max(0)..=(x + 4).min(z.width - 1) {
            if (xx - x) * (xx - x) + (yy - y) * (yy - y) <= 20 {
                cells[(yy * z.width + xx) as usize] = true;
            }
        }
    }
}
pub fn visible(z: &Zone, a: Point, b: Point) -> bool {
    let steps = ((a.x - b.x).abs().max((a.y - b.y).abs()) / 100).max(1);
    (0..=steps).all(|i| {
        !blocked(
            z,
            (a.x + (b.x - a.x) * i / steps) / UNIT,
            (a.y + (b.y - a.y) * i / steps) / UNIT,
        )
    })
}
/// A single flood field serves every melee actor; no per-actor heap search.
pub fn field(z: &Zone, target: Point) -> Vec<u16> {
    let mut distances = vec![u16::MAX; CELLS];
    let mut queue = std::collections::VecDeque::with_capacity(CELLS);
    let start = (target.y / UNIT * z.width + target.x / UNIT) as usize;
    if start >= CELLS {
        return distances;
    }
    distances[start] = 0;
    queue.push_back(start);
    while let Some(i) = queue.pop_front() {
        let x = i as i32 % z.width;
        let y = i as i32 / z.width;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let nx = x + dx;
            let ny = y + dy;
            if blocked(z, nx, ny) {
                continue;
            }
            let n = (ny * z.width + nx) as usize;
            if distances[n] == u16::MAX {
                distances[n] = distances[i] + 1;
                queue.push_back(n);
            }
        }
    }
    distances
}
pub fn seek(z: &Zone, p: Point, target: Point, flow: &[u16]) -> Point {
    let steps = ((p.x - target.x).abs().max((p.y - target.y).abs()) / 32).max(1);
    let clear = (0..=steps).all(|i| {
        free(
            z,
            Point {
                x: p.x + (target.x - p.x) * i / steps,
                y: p.y + (target.y - p.y) * i / steps,
            },
        )
    });
    if clear {
        return target;
    }
    let x = p.x / UNIT;
    let y = p.y / UNIT;
    [(x, y), (x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
        .into_iter()
        .filter(|(x, y)| !blocked(z, *x, *y))
        .min_by_key(|(x, y)| flow[(y * z.width + x) as usize])
        .map(|(x, y)| at([x, y]))
        .unwrap_or(p)
}
