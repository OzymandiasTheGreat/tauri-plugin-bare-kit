npm run tauri android build -- --debug --ci --apk --target x86_64
npx appium --allow-insecure "*:chromedriver_autodownload" &
APPIUM_PID=$!
npm run test:android
EXIT_CODE=$?
kill $APPIUM_PID
exit $EXIT_CODE
