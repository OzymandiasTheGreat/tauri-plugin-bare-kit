set -euo pipefail

npx appium --allow-insecure "*:chromedriver_autodownload" &
APPIUM_PID=$!
trap 'kill "$APPIUM_PID" 2>/dev/null || true' EXIT
npm run test:android
