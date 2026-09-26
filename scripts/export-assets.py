from pathlib import Path
import re,struct,zlib,json
root=Path(__file__).resolve().parents[1]
source=(root/'crates/hk-ppu/src/lib.rs').read_text()
rows=re.findall(r'"([.123]{16})"',source)
assert len(rows)==64
colors=[(0,0,0),(232,186,107),(98,121,134),(245,235,212)]
def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
for i,name in enumerate(['veilleur','sans-temoin','lanterne','bastion']):
 data=b''.join(b'\0'+bytes(0 if c=='.' else int(c)for c in row)for row in rows[i*16:(i+1)*16])
 palette=colors if i!=1 else [(0,0,0),(113,75,139),(53,44,80),(240,148,187)]
 png=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',16,16,8,3,0,0,0))+chunk(b'PLTE',bytes(c for rgb in palette for c in rgb))+chunk(b'tRNS',bytes([0,255,255,255]))+chunk(b'IDAT',zlib.compress(data))+chunk(b'IEND',b'')
 (root/f'assets/sprites/{name}.png').write_bytes(png)
all_source=source+(root/'crates/hk-core/src/lib.rs').read_text()
palette=sorted(set(re.findall(r'0x[0-9a-fA-F]{6}(?![0-9a-fA-F])',all_source)))
(root/'assets/palette.json').write_text(json.dumps(palette,indent=2))
(root/'assets/music/premiere-nuit.hkseq').write_text('''# HEXKEEP original score — La Première Nuit
# 22050 Hz, step 6615 samples. Two pulse voices, triangle bass, LFSR noise.
PULSE1 duty=25 gain=800
440 - 523 587 659 587 523 - 392 - 440 523 587 523 440 -
349 - 440 523 587 659 784 659 523 587 440 - 392 349 330 -
PULSE2 duty=50 gain=800
Every fourth step: PULSE1 / 2; otherwise silent.
TRIANGLE gain=9
Eight steps per note: 110 98 87 98
NOISE
LFSR 15-bit, taps 0 and 1; accent every fourth step, first 500 samples.
# The executable sequencer is crates/hk-apu/src/lib.rs.
''')
print('Original indexed sprites exported; renderer color constants:',len(palette))
