npx appium --allow-insecure "*:chromedriver_autodownload" &
APPIUM_PID=$!
npm run test:android
EXIT_CODE=$?
kill $APPIUM_PID
exit $EXIT_CODE
