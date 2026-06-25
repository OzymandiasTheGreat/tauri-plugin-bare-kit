import { spawnSync } from "node:child_process"
import path from "node:path"
import { fileURLToPath } from "node:url"

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

let app = null

export const config = {
  runner: "local",

  specs: [path.resolve(__dirname, "specs", "*.spec.js")],

  exclude: [],
  maxInstances: 1,
  capabilities: [
    {
      "platformName": "Android",
      "appium:automationName": "UiAutomator2",
      "appium:udid": "emulator-5554",
      "appium:app": path.resolve(
        __dirname,
        "src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk",
      ),
      "appium:autoWebview": true,
      "appium:autoWebviewTimeout": 30_000,
      "appium:allowChromeDriverAutoDownload": true,
    },
  ],

  hostname: "127.0.0.1",
  port: 4723,
  path: "/",

  logLevel: "info",
  bail: 0,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 240_000,
  connectionRetryCount: 3,
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 240_000,
  },
}
