#!/usr/bin/env bash
set -euo pipefail

setsid npx appium --allow-insecure "*:chromedriver_autodownload" &
APPIUM_PID=$!

cleanup() {
    kill -- "-$APPIUM_PID" 2>/dev/null || true
    wait "$APPIUM_PID" 2>/dev/null || true
}

trap cleanup EXIT

npm run test:android
