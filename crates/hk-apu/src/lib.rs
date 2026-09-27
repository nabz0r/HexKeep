//! Original four-channel tracker: two pulse voices, triangle bass, LFSR noise.
const MELODY: [u32; 32] = [
    440, 0, 523, 587, 659, 587, 523, 0, 392, 0, 440, 523, 587, 523, 440, 0, 349, 0, 440, 523, 587,
    659, 784, 659, 523, 587, 440, 0, 392, 349, 330, 0,
];
const THEMES: [[u32; 16]; 6] = [
    [
        262, 330, 392, 0, 523, 392, 330, 294, 262, 0, 349, 440, 392, 330, 294, 0,
    ],
    [
        147, 0, 220, 196, 147, 165, 0, 220, 294, 0, 262, 220, 196, 165, 147, 0,
    ],
    [
        330, 392, 440, 494, 587, 0, 494, 440, 392, 330, 294, 0, 392, 440, 330, 0,
    ],
    [
        131, 0, 139, 0, 196, 0, 185, 0, 131, 123, 0, 139, 0, 165, 0, 0,
    ],
    [
        196, 196, 294, 0, 262, 220, 196, 0, 392, 294, 262, 0, 220, 196, 147, 0,
    ],
    [
        262, 330, 392, 523, 0, 523, 587, 659, 784, 659, 587, 523, 392, 330, 262, 0,
    ],
];
pub struct Apu {
    sample: u64,
    phase: [u32; 3],
    noise: u16,
    effect: u32,
    pub enabled: bool,
    pub theme: u8,
}
impl Default for Apu {
    fn default() -> Self {
        Self {
            sample: 0,
            phase: [0; 3],
            noise: 0x7fff,
            effect: 0,
            enabled: true,
            theme: 0,
        }
    }
}
impl Apu {
    pub fn trigger(&mut self, id: u8) {
        self.effect = if id == 1 { 3500 } else { 1800 };
    }
    pub fn samples(&mut self, n: usize) -> Vec<i16> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let step = (self.sample / 6615) as usize;
            let melody = if self.theme == 0 {
                MELODY[step % 32]
            } else {
                THEMES[(self.theme as usize - 1) % 6][step % 16]
            };
            let bass = [110, 98, 87, 98][step / 8 % 4];
            let freqs = [melody, if step % 4 == 0 { melody / 2 } else { 0 }, bass];
            let mut v = 0i32;
            for (ch, f) in freqs.iter().enumerate() {
                self.phase[ch] =
                    self.phase[ch].wrapping_add(((*f as u64 * (1u64 << 32)) / 22050) as u32);
                let ph = self.phase[ch] >> 24;
                v += if *f == 0 {
                    0
                } else if ch < 2 {
                    if ph < if ch == 0 { 64 } else { 128 } {
                        800
                    } else {
                        -800
                    }
                } else {
                    ((if ph < 128 { ph } else { 255 - ph }) as i32 - 64) * 9
                };
            }
            let fb = (self.noise ^ (self.noise >> 1)) & 1;
            self.noise = (self.noise >> 1) | (fb << 14);
            if self.effect > 0 {
                v += (if self.noise & 1 == 0 { 1 } else { -1 }) * self.effect as i32 / 2;
                self.effect -= 1;
            }
            if step % 4 == 0 && self.sample % 6615 < 500 {
                v += if self.noise & 1 == 0 { 200 } else { -200 };
            }
            out.push(if self.enabled {
                v.clamp(-12000, 12000) as i16
            } else {
                0
            });
            self.sample += 1;
        }
        out
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_audio() {
        let mut a = Apu::default();
        let v = a.samples(22050);
        assert!(v.iter().any(|x| *x != 0));
        assert!(v.iter().all(|x| x.abs() < 12001));
    }
}
