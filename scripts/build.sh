#!/bin/sh
set -eu
. "$(dirname "$0")/env.sh"
cd "$HK_ROOT"
cargo test --workspace --locked --release
cargo build -p hk-ffi --release
case "$(uname -s)" in
  Darwin) HK_LIBRARY=target/release/libhk_ffi.dylib ;;
  Linux) HK_LIBRARY=target/release/libhk_ffi.so ;;
  *) echo "Use macOS or Linux to build this Android package." >&2; exit 1 ;;
esac
cargo run -p hk-bindgen --release -- generate --library "$HK_LIBRARY" --language kotlin --out-dir android/app/src/main/java --config crates/hk-ffi/uniffi.toml
mkdir -p artifacts
HK_JNI_STAGE=$(mktemp -d "$HK_ROOT/artifacts/jni-XXXXXX")
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 --platform 26 -o "$HK_JNI_STAGE" build -p hk-ffi --release
python3 - "$HK_JNI_STAGE" <<'PYJNI'
from pathlib import Path
import shutil, sys
output = Path("android/app/src/main/jniLibs")
if output.exists(): shutil.rmtree(output)
shutil.move(sys.argv[1], output)
PYJNI
# Keep generated bindings stable when ktlint is not installed.
python3 -c 'from pathlib import Path; p=Path("android/app/src/main/java/game/hexkeep/core/hk_ffi.kt"); p.write_text("\n".join(line.rstrip() for line in p.read_text().splitlines())+"\n")'
cd android
./gradlew --no-daemon assembleDevDebug assembleDevRelease assembleProdRelease assemblePlayDebug assemblePlayDebugAndroidTest bundlePlayRelease lintPlayRelease
mkdir -p ../artifacts
cp app/build/outputs/apk/dev/debug/app-dev-debug.apk ../artifacts/
cp app/build/outputs/apk/prod/release/app-prod-release-unsigned.apk ../artifacts/

cp app/build/outputs/apk/dev/release/app-dev-release.apk ../artifacts/HEXKEEP-v0.8-DEV.apk

cp app/build/outputs/apk/play/debug/app-play-debug.apk ../artifacts/HEXKEEP-v0.8-PLAY-preview.apk
cp app/build/outputs/bundle/playRelease/app-play-release.aab ../artifacts/HEXKEEP-v0.8-PLAY.aab
cp app/build/outputs/apk/androidTest/play/debug/app-play-debug-androidTest.apk ../artifacts/
