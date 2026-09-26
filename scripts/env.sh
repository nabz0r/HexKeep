#!/bin/sh
HK_ROOT=${HK_ROOT:-$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)}
export JAVA_HOME="${JAVA_HOME:-$(find "$HK_ROOT/.tools" -maxdepth 4 -type d -path '*/Contents/Home' | head -1)}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$HOME/.cargo/bin:$PATH"
