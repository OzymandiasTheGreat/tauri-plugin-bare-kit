import { spawn, spawnSync } from "node:child_process"
import path from "node:path"
import { fileURLToPath } from "node:url"

const port = 4445
const BIN = "tauri-plugin-bare-kit-tests"

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
      browserName: "chrome",
    },
  ],

  hostname: "127.0.0.1",
  port,
  path: "/",

  logLevel: "info",
  bail: 0,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 120_000,
  connectionRetryCount: 3,
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },

  onPrepare: async function (config, capabilities) {
    const bin = path.resolve(__dirname, "src-tauri/target/release", BIN)

    console.log(`Starting tauri app at ${bin}`)

    app = spawn(bin, [], {
      env: {
        ...process.env,
        TAURI_WEBDRIVER_PORT: port.toString(10),
      },
      stdio: ["ignore", "pipe", "pipe"],
    })
    app.stdout?.on("data", (data) => console.log(`[app stdout]: ${data.toString().trim()}`))
    app.stderr?.on("data", (data) => console.error(`[app stderr]: ${data.toString().trim()}`))
    app.on("error", (err) => console.error(`Failed to start tauri app: ${err}`))
    app.on("exit", (code, signal) => {
      console.log(`App exited with code ${code}, signal ${signal}`)
      app = null
    })

    const start = Date.now()

    while (Date.now() - start < 30_000) {
      try {
        const response = await fetch(`http://127.0.0.1:${port}/status`)

        if (response.ok) {
          return
        }
      } catch {}

      await new Promise((resolve) => setTimeout(resolve, 100))
    }

    throw new Error(`WebDriver server failed to start`)
  },

  onComplete: function (exitCode, config, capabilities, results) {
    if (app) {
      app.kill("SIGTERM")
      app = null
    }
  },
}
