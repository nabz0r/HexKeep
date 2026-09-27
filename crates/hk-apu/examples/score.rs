use std::io::Write;
fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or("hexkeep-les-braises.wav".into());
    let mut a = hk_apu::Apu::default();
    let mut pcm = Vec::new();
    for section in 0..6 {
        a.theme = if section >= 3 { 4 } else { 0 };
        if section == 4 {
            a.trigger(4);
        }
        pcm.extend(a.samples(44100 * 4));
    }
    let mut f = std::fs::File::create(path).unwrap();
    let size = (pcm.len() * 2) as u32;
    f.write_all(b"RIFF").unwrap();
    f.write_all(&(size + 36).to_le_bytes()).unwrap();
    f.write_all(b"WAVEfmt ").unwrap();
    for v in [16u32, 0x00010001, 44100, 88200, 0x00100002] {
        f.write_all(&v.to_le_bytes()).unwrap();
    }
    f.write_all(b"data").unwrap();
    f.write_all(&size.to_le_bytes()).unwrap();
    for v in pcm {
        f.write_all(&v.to_le_bytes()).unwrap();
    }
}
