//! Original pixel renderer. All coordinates are internal pixels. No Android UI assets.
pub const INK: u32 = 0x0b111c;
pub const PANEL: u32 = 0x141f2f;
pub const EDGE: u32 = 0x314251;
pub const GOLD: u32 = 0xe8ba6b;
pub const WHITE: u32 = 0xf5ebd4;
pub const MUTED: u32 = 0x8b9eaa;
pub const BLUE: u32 = 0x73bddd;
pub const RED: u32 = 0xe47d72;
pub const GREEN: u32 = 0x9ac8a2;
pub const REALMS: [u32; 3] = [GOLD, BLUE, GREEN];
#[derive(serde::Serialize)]
#[serde(tag = "kind")]
pub enum UiCommand {
    Text {
        x: i32,
        y: i32,
        text: String,
        color: u32,
        scale: i32,
    },
    Button {
        x: i32,
        y: i32,
        w: i32,
        text: String,
        active: bool,
    },
    Panel {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    Sprite {
        x: i32,
        y: i32,
        sprite: u8,
        realm: usize,
        scale: i32,
    },
}
pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
    pub ui: Vec<UiCommand>,
    pub recording: bool,
}
impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            px: vec![0; (w * h * 4) as usize],
            ui: vec![],
            recording: false,
        }
    }
    pub fn recording(w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            px: vec![],
            ui: vec![],
            recording: true,
        }
    }
    pub fn clear(&mut self, c: u32) {
        for p in self.px.chunks_exact_mut(4) {
            p.copy_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]);
        }
    }
    pub fn pixel(&mut self, x: i32, y: i32, c: u32) {
        if !self.px.is_empty() && x >= 0 && y >= 0 && x < self.w && y < self.h {
            let i = ((y * self.w + x) * 4) as usize;
            self.px[i..i + 4].copy_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]);
        }
    }
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        if self.px.is_empty() {
            return;
        }
        for yy in y.max(0)..(y + h).min(self.h) {
            for xx in x.max(0)..(x + w).min(self.w) {
                self.pixel(xx, yy, c);
            }
        }
    }
    pub fn line(&mut self, mut x: i32, mut y: i32, x1: i32, y1: i32, c: u32) {
        let dx = (x1 - x).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let dy = -(y1 - y).abs();
        let sy = if y < y1 { 1 } else { -1 };
        let mut e = dx + dy;
        loop {
            self.pixel(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * e;
            if e2 >= dy {
                e += dy;
                x += sx;
            }
            if e2 <= dx {
                e += dx;
                y += sy;
            }
        }
    }
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: u32) {
        self.rect(x, y, w, 1, c);
        self.rect(x, y + h - 1, w, 1, c);
        self.rect(x, y, 1, h, c);
        self.rect(x + w - 1, y, 1, h, c);
    }
    pub fn panel(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if self.recording {
            self.ui.push(UiCommand::Panel { x, y, w, h });
        }
        self.rect(x, y, w, h, PANEL);
        self.frame(x, y, w, h, EDGE);
    }
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: u32, scale: i32) {
        if self.recording {
            self.ui.push(UiCommand::Text {
                x,
                y,
                text: s.into(),
                color: c,
                scale,
            });
        }
        if self.px.is_empty() {
            return;
        }
        let mut xx = x;
        for ch in s.chars() {
            let g = glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(xx + col * scale, y + row as i32 * scale, scale, scale, c);
                    }
                }
            }
            xx += 6 * scale;
        }
    }
    pub fn center(&mut self, y: i32, s: &str, c: u32, scale: i32) {
        self.text(
            (self.w - s.chars().count() as i32 * 6 * scale) / 2,
            y,
            s,
            c,
            scale,
        );
    }
    pub fn button(&mut self, x: i32, y: i32, w: i32, s: &str, active: bool) {
        if self.recording {
            self.ui.push(UiCommand::Button {
                x,
                y,
                w,
                text: s.into(),
                active,
            });
            return;
        }
        let col = if active { GOLD } else { EDGE };
        self.rect(x, y, w, 23, if active { 0x352e27 } else { PANEL });
        self.frame(x, y, w, 23, col);
        self.text(
            x + (w - s.chars().count() as i32 * 6) / 2,
            y + 8,
            s,
            if active { GOLD } else { WHITE },
            1,
        );
    }
    pub fn hex(&mut self, x: i32, y: i32, r: i32, fill: u32, edge: u32) {
        let hh = r * 7 / 8;
        for yy in -hh..=hh {
            let half = if yy.abs() < hh / 2 {
                r
            } else {
                r - (yy.abs() - hh / 2) * r / (hh + 1)
            };
            self.rect(x - half, y + yy, half * 2 + 1, 1, fill);
        }
        let p = [
            (x - r, y),
            (x - r / 2, y - hh),
            (x + r / 2, y - hh),
            (x + r, y),
            (x + r / 2, y + hh),
            (x - r / 2, y + hh),
        ];
        for i in 0..6 {
            self.line(p[i].0, p[i].1, p[(i + 1) % 6].0, p[(i + 1) % 6].1, edge);
        }
    }
    pub fn circle(&mut self, x: i32, y: i32, r: i32, c: u32) {
        for yy in -r..=r {
            for xx in -r..=r {
                if xx * xx + yy * yy <= r * r {
                    self.pixel(x + xx, y + yy, c);
                }
            }
        }
    }
    pub fn ring(&mut self, x: i32, y: i32, r: i32, c: u32) {
        for yy in -r..=r {
            for xx in -r..=r {
                let d = xx * xx + yy * yy;
                if d <= r * r && d >= (r - 1) * (r - 1) {
                    self.pixel(x + xx, y + yy, c);
                }
            }
        }
    }
    pub fn sprite(&mut self, x: i32, y: i32, kind: u8, realm: usize, scale: i32, anim: u32) {
        if self.recording {
            self.ui.push(UiCommand::Sprite {
                x,
                y,
                sprite: kind,
                realm,
                scale,
            });
            return;
        }
        let color = REALMS[realm % 3];
        let rows: &[&str] = match kind {
            0 => &[
                "................",
                ".....2222.......",
                "....233332......",
                "....233332......",
                ".....2222.......",
                "....111111......",
                "...11222111.....",
                "..1112222111....",
                "..1.122221.1....",
                "....122221......",
                "....111111......",
                "....11..11......",
                "....11..11......",
                "...222..222.....",
                "................",
                "................",
            ],
            1 => &[
                "................",
                "...1.......1....",
                "...11.....11....",
                "....1222221.....",
                "...122222221....",
                "...123223221....",
                "...122222221....",
                "....1222221.....",
                "..11122222111...",
                ".11.1222221.11..",
                "....1122211.....",
                ".....11111......",
                ".....1...1......",
                "....11...11.....",
                "................",
                "................",
            ],
            2 => &[
                "......2222......",
                ".....2....2.....",
                ".....2....2.....",
                "...2222222222...",
                "...2........2...",
                "...2...3....2...",
                "...2..333...2...",
                "...2..3333..2...",
                "...2.33333..2...",
                "...2..333...2...",
                "...2...3....2...",
                "...2........2...",
                "...2222222222...",
                "..222222222222..",
                "................",
                "................",
            ],
            _ => &[
                "..22..22..22....",
                "..22..22..22....",
                "..2222222222....",
                "...22222222.....",
                "...21111112.....",
                "...21211212.....",
                "...21111112.....",
                "...22222222.....",
                "...22222222.....",
                "...22211222.....",
                "...22111122.....",
                "...22111122.....",
                "..2221111222....",
                "..2221111222....",
                "22222222222222..",
                "................",
            ],
        };
        for (yy, row) in rows.iter().enumerate() {
            for (xx, b) in row.bytes().enumerate() {
                let col = match b {
                    b'1' => {
                        if kind == 1 {
                            0x714b8b
                        } else {
                            color
                        }
                    }
                    b'2' => {
                        if kind == 1 {
                            0x352c50
                        } else {
                            0x627986
                        }
                    }
                    b'3' => {
                        if kind == 1 {
                            0xf094bb
                        } else {
                            WHITE
                        }
                    }
                    _ => continue,
                };
                let shift = if kind == 0 && yy > 10 && anim % 20 < 10 {
                    1
                } else {
                    0
                };
                self.rect(
                    x + (xx as i32 + shift) * scale,
                    y + yy as i32 * scale,
                    scale,
                    scale,
                    col,
                );
            }
        }
    }
    pub fn landscape(&mut self, t: u32) {
        self.clear(INK);
        for i in 0..75 {
            let x = (i * 73 + 19) % self.w;
            let y = (i * i * 17 + 3) % 105;
            self.pixel(
                x,
                y,
                if (i as u32 + t / 30) % 7 == 0 {
                    GOLD
                } else {
                    0x526778
                },
            );
        }
        self.circle(self.w - 65, 35, 17, 0x40505c);
        self.circle(self.w - 60, 30, 17, INK);
        for x in 0..self.w {
            let y = 106 + (x * 13 % 47) / 3;
            self.rect(x, y, 1, 134 - y, 0x182638);
        }
        for x in (0..self.w).step_by(9) {
            let y = 124 + (x * 7 % 37) / 3;
            self.rect(x, y, 10, 166 - y, 0x21313d);
        }
        for y in 150..240 {
            self.rect(
                0,
                y,
                self.w,
                1,
                if y % 11 == 0 { 0x2d3d43 } else { 0x192930 },
            );
        }
        for i in 0..32 {
            let x = (i * 67 + t as i32 / 10) % self.w;
            let y = 162 + i * 19 % 70;
            self.rect(x, y, 5 + i % 13, 1, 0x3d5053);
        }
        self.rect(0, 212, self.w, 28, 0x101b24);
        for x in (0..self.w).step_by(12) {
            self.rect(x, 207 + (x * 7 % 6), 9, 10, 0x101b24);
        }
        self.sprite(self.w - 100, 135, 3, 0, 4, 0);
        self.sprite(25, 180, 2, 0, 2, t);
    }
}
fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        'é' | 'É' => [2, 4, 31, 16, 30, 16, 31],
        'è' | 'È' => [8, 4, 31, 16, 30, 16, 31],
        'ê' | 'Ê' => [4, 10, 31, 16, 30, 16, 31],
        'à' | 'À' => [8, 4, 14, 17, 31, 17, 17],
        'â' | 'Â' => [4, 10, 14, 17, 31, 17, 17],
        'ù' | 'Ù' => [8, 4, 17, 17, 17, 17, 14],
        'î' | 'Î' => [4, 10, 14, 4, 4, 4, 14],
        'ô' | 'Ô' => [4, 10, 14, 17, 17, 17, 14],
        'ç' | 'Ç' => [14, 16, 16, 16, 14, 4, 8],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        ',' => [0, 0, 0, 0, 6, 6, 4],
        ':' => [0, 6, 6, 0, 6, 6, 0],
        ';' => [0, 6, 6, 0, 6, 6, 4],
        '-' | '—' => [0, 0, 0, 31, 0, 0, 0],
        '!' => [4, 4, 4, 4, 4, 0, 4],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '\'' | '’' => [4, 4, 0, 0, 0, 0, 0],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        '%' => [17, 2, 4, 8, 16, 17, 0],
        '<' => [1, 2, 4, 8, 4, 2, 1],
        '>' => [16, 8, 4, 2, 4, 8, 16],
        '=' => [0, 31, 0, 31, 0, 0, 0],
        '#' => [10, 31, 10, 10, 31, 10, 0],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        '·' => [0, 0, 0, 4, 0, 0, 0],
        _ => [0; 7],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries() {
        let mut c = Canvas::new(480, 240);
        c.clear(INK);
        c.rect(-20, -20, 40, 40, WHITE);
        c.landscape(0);
        assert_eq!(c.px.len(), 480 * 240 * 4);
        assert!(c.px.chunks_exact(4).all(|p| p[3] == 255));
    }
}
