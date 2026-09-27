#!/bin/sh
set -eu
. "$(dirname "$0")/env.sh"
cd "$HK_ROOT"
cargo test --workspace --locked --release
cargo build -p hk-ffi
case "$(uname -s)" in
  Darwin) HK_LIBRARY=target/debug/libhk_ffi.dylib ;;
  Linux) HK_LIBRARY=target/debug/libhk_ffi.so ;;
  *) echo "Use macOS or Linux to build this Android package." >&2; exit 1 ;;
esac
cargo run -p hk-bindgen -- generate --library "$HK_LIBRARY" --language kotlin --out-dir android/app/src/main/java --config crates/hk-ffi/uniffi.toml
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 --platform 26 -o android/app/src/main/jniLibs build -p hk-ffi --release
cd android
./gradlew --no-daemon assembleDevDebug assembleDevRelease assembleProdRelease
mkdir -p ../artifacts
cp app/build/outputs/apk/dev/debug/app-dev-debug.apk ../artifacts/
cp app/build/outputs/apk/prod/release/app-prod-release-unsigned.apk ../artifacts/

cp app/build/outputs/apk/dev/release/app-dev-release.apk ../artifacts/HEXKEEP-v0.5.apk
