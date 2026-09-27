//! Original ambient score, 44.1 kHz. Audio floats never enter the deterministic simulation.
const RATE: f32 = 44100.;
const CHORDS: [[f32; 4]; 4] = [
    [146.832, 174.614, 220., 293.665],
    [116.541, 146.832, 174.614, 233.082],
    [130.813, 174.614, 220., 261.626],
    [130.813, 164.814, 195.998, 261.626],
];
const NOTES: [f32; 16] = [
    587.33, 0., 440., 0., 523.25, 0., 349.23, 0., 440., 0., 659.26, 587.33, 0., 523.25, 440., 0.,
];
pub struct Apu {
    sample: u64,
    phase: [f32; 10],
    noise: u32,
    wind: f32,
    effect: u8,
    effect_age: u32,
    echo: Vec<f32>,
    cursor: usize,
    music_gain: f32,
    tension: f32,
    pub enabled: bool,
    pub effects_enabled: bool,
    pub theme: u8,
}
impl Default for Apu {
    fn default() -> Self {
        Self {
            sample: 0,
            phase: [0.; 10],
            noise: 0x1234abcd,
            wind: 0.,
            effect: 0,
            effect_age: 0,
            echo: vec![0.; 15437],
            cursor: 0,
            music_gain: 0.,
            tension: 0.,
            enabled: true,
            effects_enabled: true,
            theme: 0,
        }
    }
}
fn sine(phase: f32) -> f32 {
    (phase * std::f32::consts::TAU).sin()
}
impl Apu {
    pub fn trigger(&mut self, id: u8) {
        if self.effects_enabled {
            self.effect = id;
            self.effect_age = 0;
        }
    }
    pub fn samples(&mut self, n: usize) -> Vec<i16> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let t = self.sample as f32 / RATE;
            let beat = RATE * 0.72;
            let step = (self.sample as f32 / beat) as usize;
            let chord = (step / 8) % 4;
            let blend = ((self.sample as f32 % (beat * 8.)) / (RATE * 1.8)).clamp(0., 1.);
            let mut pad = 0.;
            for i in 0..4 {
                let frequency =
                    CHORDS[(chord + 3) % 4][i] * (1. - blend) + CHORDS[chord][i] * blend;
                self.phase[i] = (self.phase[i] + frequency / RATE) % 1.;
                pad += sine(self.phase[i]) * (0.7 + 0.3 * sine(t * 0.071 + i as f32 * 0.2));
            }
            let note = NOTES[step % 16];
            self.phase[4] = (self.phase[4] + note / RATE) % 1.;
            let age = (self.sample as f32 % beat) / RATE;
            let env = (age * 40.).min(1.) * (-age * 5.).exp();
            let bell = if note > 0. {
                (sine(self.phase[4]) + 0.24 * sine(self.phase[4] * 2.003)) * env
            } else {
                0.
            };
            self.noise ^= self.noise << 13;
            self.noise ^= self.noise >> 17;
            self.noise ^= self.noise << 5;
            let noise = self.noise as i32 as f32 / i32::MAX as f32;
            self.wind += 0.002 * (noise - self.wind);
            let combat = if self.theme == 4 || self.theme == 5 {
                1.
            } else {
                0.
            };
            self.tension += (combat - self.tension) * 0.00004;
            let drum_age = (self.sample as f32 % (beat * 2.)) / RATE;
            self.phase[5] = (self.phase[5] + (45. + 65. * (-drum_age * 30.).exp()) / RATE) % 1.;
            let drum = sine(self.phase[5]) * (-drum_age * 12.).exp() * self.tension * 0.07;
            let music = pad * 0.032 + bell * 0.057 + self.wind * 0.07 + drum;
            self.music_gain += ((if self.enabled { 1. } else { 0. }) - self.music_gain) * 0.0002;
            let delayed = self.echo[self.cursor];
            self.echo[self.cursor] = music + delayed * 0.38;
            self.cursor = (self.cursor + 1) % self.echo.len();
            let mut v = (music + delayed * 0.25) * self.music_gain;
            if self.effect > 0 && self.effects_enabled {
                let e = self.effect_age as f32 / RATE;
                let (freq, decay, gain) = match self.effect {
                    1 => (100., 22., 0.15),
                    2 => (520., 35., 0.08),
                    3 => (330., 7., 0.13),
                    4 => (880., 3., 0.16),
                    _ => (65., 4., 0.19),
                };
                self.phase[6] = (self.phase[6] + freq * (1. + 0.12 * (-e * 20.).exp()) / RATE) % 1.;
                let attack = (e * 600.).min(1.);
                let tone = if self.effect <= 2 {
                    sine(self.phase[6]) * 0.65 + noise * 0.35
                } else {
                    sine(self.phase[6]) + sine(self.phase[6] * 1.5) * 0.35
                };
                v += tone * gain * attack * (-e * decay).exp();
                self.effect_age += 1;
                if e > 2. {
                    self.effect = 0;
                }
            }
            if !self.effects_enabled {
                self.effect = 0;
            }
            out.push((v.clamp(-0.9, 0.9) * 32767.) as i16);
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
        for theme in 0..7 {
            a.theme = theme;
            a.trigger(theme);
            let v = a.samples(44100);
            assert!(v.iter().any(|x| *x != 0));
            assert!(v.iter().all(|x| x.unsigned_abs() < 29491));
        }
    }
    #[test]
    fn music_and_effects_are_independent() {
        let mut a = Apu::default();
        a.enabled = false;
        assert!(a.samples(1000).iter().all(|v| *v == 0));
        a.trigger(4);
        assert!(a.samples(1000).iter().any(|v| *v != 0));
        a.effects_enabled = false;
        assert!(a.samples(1000).iter().all(|v| *v == 0));
    }
    #[test]
    fn continuous_blocks() {
        let mut a = Apu::default();
        let mut b = Apu::default();
        let whole = a.samples(3000);
        let split = [b.samples(1470), b.samples(1530)].concat();
        assert_eq!(whole, split);
    }
}
