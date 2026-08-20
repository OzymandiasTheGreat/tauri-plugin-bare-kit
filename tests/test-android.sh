#!/usr/bin/env bash
set -euo pipefail

setsid npx appium --allow-insecure "*:chromedriver_autodownload" &
APPIUM_PID=$!

cleanup() {
    echo "Stopping Appium (PID $APPIUM_PID)..."
    kill -- "-$APPIUM_PID" 2>/dev/null || true
    wait "$APPIUM_PID" 2>/dev/null || true
    # pkill -TERM -f '/android/sdk/emulator/crashpad_handler' || true
    ps -ef | grep -E '[a]ppium|[c]hromedriver|[c]rashpad|[e]mulator' || true
}

trap cleanup EXIT

npm run test:android
