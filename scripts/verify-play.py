#!/usr/bin/env python3
"""Inspect built artifacts, not just Gradle declarations. No signing secrets required."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import zipfile


def run(*args):
    return subprocess.check_output([str(a) for a in args], text=True, stderr=subprocess.STDOUT)


def inspect_native(path):
    libraries = []
    with zipfile.ZipFile(path) as archive:
        for name in archive.namelist():
            if not name.endswith('.so'):
                continue
            data = archive.read(name)
            assert data[:4] == b'\x7fELF', name
            bits = data[4]
            assert data[5] == 1, 'Only little-endian Android ELFs expected'
            if bits == 2:
                offset = struct.unpack_from('<Q', data, 32)[0]
                size, count = struct.unpack_from('<HH', data, 54)
                align_offset, align_format = 48, '<Q'
            else:
                offset = struct.unpack_from('<I', data, 28)[0]
                size, count = struct.unpack_from('<HH', data, 42)
                align_offset, align_format = 28, '<I'
            loads = [struct.unpack_from(align_format, data, offset+i*size+align_offset)[0]
                     for i in range(count) if struct.unpack_from('<I', data, offset+i*size)[0] == 1]
            assert loads, f'No LOAD segments: {name}'
            if bits == 2:
                assert min(loads) >= 16384, f'Not 16 KB aligned: {name}: {loads}'
            if name.endswith('/libhk_ffi.so'):
                assert b'uniffi_hk_ffi_checksum_method_engine_can_save' in data, f'Stale engine: {name}'
                assert b'uniffi_hk_ffi_checksum_constructor_engine_offline' in data, f'Stale bindings: {name}'
                assert 'Les Échos des Confins'.encode() in data, f'Missing v0.7 campaign: {name}'
                assert b'beacons_boss' in data, f'Missing v0.7 mission engine: {name}'
            libraries.append({'path': name, 'bits': 64 if bits == 2 else 32, 'load_alignment': loads})
        engines = {n.split('/')[-2] for n in archive.namelist() if n.endswith('/libhk_ffi.so')}
        assert engines == {'arm64-v8a', 'armeabi-v7a', 'x86_64'}, engines
    return libraries


def inspect_art(path):
    with zipfile.ZipFile(path) as archive:
        names = [n for n in archive.namelist() if '/art/v08/' in n and n.endswith('.png')]
        assert len(names) == 26, f'Expected 26 painted v0.8 assets in {path}'
        result = []
        for name in sorted(names):
            data = archive.read(name)
            assert data[:8] == b'\x89PNG\r\n\x1a\n', name
            width, height = struct.unpack_from('>II', data, 16)
            stem = Path(name).stem
            opaque = stem.startswith('floor-') or stem in ('refuge', 'world-atlas')
            assert width >= 1000 and height >= 800, name
            if not opaque:
                assert data[25] == 6, f'Sprite without RGBA: {name}'
            result.append({'path': name, 'width': width, 'height': height, 'sha256': hashlib.sha256(data).hexdigest()})
        assert not any(n.endswith('/art/v05/aurelon.png') for n in archive.namelist()), 'Superseded sprites should not ship'
        return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--apk', type=Path, required=True)
    parser.add_argument('--aab', type=Path, required=True)
    parser.add_argument('--bundletool', type=Path, required=True)
    parser.add_argument('--require-signed', action='store_true')
    args = parser.parse_args()
    sdk = Path(os.environ['ANDROID_HOME']) / 'build-tools' / '36.0.0'
    badging = run(sdk/'aapt', 'dump', 'badging', args.apk)
    assert "package: name='game.hexkeep'" in badging
    assert "versionCode='8'" in badging and "targetSdkVersion:'36'" in badging
    permissions = re.findall(r"uses-permission: name='([^']+)'", badging)
    assert permissions == ['android.permission.VIBRATE'], permissions
    run(sdk/'apksigner', 'verify', '--verbose', args.apk)
    run(sdk/'zipalign', '-c', '-P', '16', '4', args.apk)
    run('java', '-jar', args.bundletool, 'validate', '--bundle='+str(args.aab))
    manifest = run('java', '-jar', args.bundletool, 'dump', 'manifest', '--bundle='+str(args.aab), '--module=base')
    assert 'android:debuggable="true"' not in manifest
    assert 'PhareService' not in manifest and 'com.android.vending.BILLING' not in manifest
    assert re.findall(r'<uses-permission android:name="([^"]+)"', manifest) == ['android.permission.VIBRATE']
    with zipfile.ZipFile(args.aab) as archive:
        signed = any(n.startswith('META-INF/') and n.endswith(('.RSA', '.EC', '.DSA')) for n in archive.namelist())
    if args.require_signed:
        assert signed, 'Upload candidate must be signed with the publisher upload key'
        certificate = run('keytool', '-printcert', '-jarfile', args.aab)
        assert 'Android Debug' not in certificate, 'Debug key is not an upload key'
        assert 'jar verified.' in run('jarsigner', '-verify', args.aab)
    result = {'application_id': 'game.hexkeep', 'target_sdk': 36, 'version_code': 8,
              'permissions': permissions, 'aab_signed': signed, 'play_console_approval': 'not performed',
              'artifacts': {str(p): {'sha256': hashlib.sha256(p.read_bytes()).hexdigest(),
                            'bytes': p.stat().st_size, 'native': inspect_native(p), 'art': inspect_art(p)} for p in [args.apk, args.aab]}}
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
