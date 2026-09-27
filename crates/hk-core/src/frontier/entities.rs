use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
impl Point {
    pub fn dist2(self, b: Self) -> i64 {
        let x = (self.x - b.x) as i64;
        let y = (self.y - b.y) as i64;
        x * x + y * y
    }
}
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    #[default]
    Idle,
    Move,
    Windup,
    Strike,
    Recovery,
    Dodge,
    Hurt,
    Dead,
}
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Stance {
    #[default]
    Balanced,
    Assault,
    Bulwark,
}
impl Stance {
    pub fn next(self) -> Self {
        match self {
            Self::Balanced => Self::Assault,
            Self::Assault => Self::Bulwark,
            Self::Bulwark => Self::Balanced,
        }
    }
    pub fn damage(self) -> i32 {
        match self {
            Self::Balanced => 100,
            Self::Assault => 130,
            Self::Bulwark => 80,
        }
    }
    pub fn defense(self) -> i32 {
        match self {
            Self::Balanced => 100,
            Self::Assault => 120,
            Self::Bulwark => 65,
        }
    }
    pub fn speed(self) -> i32 {
        match self {
            Self::Balanced => 100,
            Self::Assault => 112,
            Self::Bulwark => 82,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Player {
    pub pos: Point,
    pub hp: i32,
    pub max_hp: i32,
    pub power: i32,
    pub guard: i32,
    pub haste: u16,
    pub state: State,
    pub frames: u16,
    pub stance: Stance,
    pub combo: u8,
    pub combo_window: u16,
    pub facing: Point,
    pub dash_cd: u16,
    pub skill_cd: u16,
    pub invulnerable: u16,
    pub flasks: u8,
    pub heavy_count: u32,
}
impl Player {
    pub fn new(pos: Point, stats: (i32, i32, i32, u16)) -> Self {
        Self {
            pos,
            hp: 260 + stats.0,
            max_hp: 260 + stats.0,
            power: 28 + stats.1 / 2,
            guard: stats.2,
            haste: stats.3,
            state: State::Idle,
            frames: 0,
            stance: Stance::Balanced,
            combo: 0,
            combo_window: 0,
            facing: Point { x: 256, y: 0 },
            dash_cd: 0,
            skill_cd: 0,
            invulnerable: 0,
            flasks: 3,
            heavy_count: 0,
        }
    }
    pub fn equipment(&mut self, stats: (i32, i32, i32, u16)) {
        self.max_hp = 260 + stats.0;
        self.hp = self.hp.min(self.max_hp);
        self.power = 28 + stats.1 / 2;
        self.guard = stats.2;
        self.haste = stats.3;
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Monster {
    pub id: u32,
    pub kind: u8,
    pub pos: Point,
    pub home: Point,
    pub hp: i32,
    pub max_hp: i32,
    pub state: State,
    pub frames: u16,
    pub cooldown: u16,
    pub stance: Stance,
    pub phase: u8,
    pub target: Point,
    pub active: bool,
    pub corpse: u16,
}
impl Monster {
    pub fn new(id: u32, kind: u8, pos: Point, zone: u8) -> Self {
        let hp =
            [75, 55, 140, 32, 650][kind as usize] + zone as i32 * if kind == 4 { 170 } else { 12 };
        Self {
            id,
            kind,
            pos,
            home: pos,
            hp,
            max_hp: hp,
            state: State::Idle,
            frames: 0,
            cooldown: 30 + id as u16 % 45,
            stance: if kind == 2 {
                Stance::Bulwark
            } else {
                Stance::Balanced
            },
            phase: 0,
            target: pos,
            active: kind != 4,
            corpse: 0,
        }
    }
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Projectile {
    pub pos: Point,
    pub dx: i32,
    pub dy: i32,
    pub life: u16,
    pub damage: i32,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Effect {
    pub pos: Point,
    pub life: u16,
    pub kind: u8,
    pub amount: i32,
}
#[derive(Clone, Copy, Default)]
pub struct Controls {
    pub mx: i16,
    pub my: i16,
    pub attack: bool,
    pub dash: bool,
    pub skill: bool,
}
