#!/usr/bin/env bash
set -euo pipefail

npx appium --allow-insecure "*:chromedriver_autodownload" &
APPIUM_PID=$!

cleanup() {
    echo "Stopping Appium (PID $APPIUM_PID)..."
    kill "$APPIUM_PID" 2>/dev/null || true
    wait "$APPIUM_PID" 2>/dev/null || true
}

trap cleanup EXIT

npm run test:android

echo "TESTS FINISHED"
echo "Appium PID: $APPIUM_PID"
ps -ef | grep -E '[a]ppium|[c]hromedriver|[c]rashpad_handler'
