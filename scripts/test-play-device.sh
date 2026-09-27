#!/bin/sh
# Each journey gets a fresh instrumentation process so a failed multi-touch
# gesture cannot contaminate another journey. Encrypted storage remains intact.
set -eu
HK_REPORT_DIR=${1:-artifacts}
mkdir -p "$HK_REPORT_DIR"
HK_FAILED=0
: > "$HK_REPORT_DIR/play-device.txt"
for HK_CASE in PlayReleaseTest AdventureTest EveilTest FrontierTest 'GameplayTest#soloOfflineLifecycle'; do
  HK_EXPECTED=1
  if [ "$HK_CASE" = PlayReleaseTest ] || [ "$HK_CASE" = FrontierTest ]; then HK_EXPECTED=3; fi
  HK_CASE_REPORT="$HK_REPORT_DIR/$HK_CASE.txt"
  # FPS is recorded in device-files, but a shared SwiftShader runner cannot
  # certify physical-device performance. Direct local runs retain a 20 FPS floor.
  adb shell am instrument -w -e minimumFps 0 -e class "game.hexkeep.$HK_CASE" game.hexkeep.test/androidx.test.runner.AndroidJUnitRunner > "$HK_CASE_REPORT"
  tee -a "$HK_REPORT_DIR/play-device.txt" < "$HK_CASE_REPORT"
  if ! grep -q "OK ($HK_EXPECTED tests\{0,1\})" "$HK_CASE_REPORT"; then HK_FAILED=1; fi
  adb shell am force-stop game.hexkeep
done
adb pull /sdcard/Android/data/game.hexkeep/files "$HK_REPORT_DIR/device-files"
exit "$HK_FAILED"
