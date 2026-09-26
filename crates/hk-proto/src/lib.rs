use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
pub type Hash = [u8; 32];
pub const UNIT: i32 = 256;
pub const TICK_HZ: u32 = 30;
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}
impl Vec2 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn dist2(self, o: Self) -> i64 {
        let dx = self.x as i64 - o.x as i64;
        let dy = self.y as i64 - o.y as i64;
        dx * dx + dy * dy
    }
    pub fn scaled(self, n: i32) -> Self {
        let d = isqrt(self.dist2(Self::default()) as u64) as i32;
        if d == 0 {
            Self::default()
        } else {
            Self::new(
                (self.x as i64 * n as i64 / d as i64) as i32,
                (self.y as i64 * n as i64 / d as i64) as i32,
            )
        }
    }
}
pub fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x / 2 + (x & 1);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum Realm {
    Aurelon,
    Skarn,
    Vylde,
}
impl Realm {
    pub fn index(self) -> usize {
        match self {
            Self::Aurelon => 0,
            Self::Skarn => 1,
            Self::Vylde => 2,
        }
    }
    pub fn from_index(n: usize) -> Self {
        [Self::Aurelon, Self::Skarn, Self::Vylde][n % 3]
    }
    pub fn hp(self) -> i32 {
        [100, 110, 90][self.index()]
    }
    pub fn dash(self) -> u16 {
        [90, 105, 72][self.index()]
    }
    pub fn name(self) -> &'static str {
        ["AURELON", "SKARN", "VYLDE"][self.index()]
    }
}
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub enum Role {
    Foudre,
    Rempart,
    Lien,
}
impl Role {
    pub fn index(self) -> usize {
        match self {
            Self::Foudre => 0,
            Self::Rempart => 1,
            Self::Lien => 2,
        }
    }
    pub fn from_index(n: usize) -> Self {
        [Self::Foudre, Self::Rempart, Self::Lien][n % 3]
    }
    pub fn name(self) -> &'static str {
        ["FOUDRE", "REMPART", "LIEN"][self.index()]
    }
}
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub struct Input {
    pub move_x: i16,
    pub move_y: i16,
    pub aim_x: i16,
    pub aim_y: i16,
    pub shoot: bool,
    pub dash: bool,
    pub skill: bool,
}
impl Input {
    pub fn movement(self) -> Vec2 {
        Vec2::new(
            self.move_x.clamp(-1024, 1024) as i32,
            self.move_y.clamp(-1024, 1024) as i32,
        )
    }
    pub fn aim(self) -> Vec2 {
        Vec2::new(
            self.aim_x.clamp(-1024, 1024) as i32,
            self.aim_y.clamp(-1024, 1024) as i32,
        )
    }
}
pub fn digest<T: BorshSerialize>(v: &T) -> Hash {
    *blake3::hash(&borsh::to_vec(v).expect("canonical encoding")).as_bytes()
}
pub fn hex(h: &[u8]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn domain(network: &str, kind: &str, payload: &[u8]) -> Vec<u8> {
    let mut b = format!("HEXKEEP/v1/{network}/{kind}\0").into_bytes();
    b.extend_from_slice(payload);
    b
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn square_roots() {
        for n in 0..10000 {
            let r = isqrt(n);
            assert!(r * r <= n && (r + 1) * (r + 1) > n);
        }
    }
    #[test]
    fn domains() {
        assert_ne!(domain("dev", "event", b"x"), domain("prod", "event", b"x"));
    }
}
