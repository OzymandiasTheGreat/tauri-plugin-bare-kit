import { spawnSync } from "node:child_process"
import path from "node:path"
import { fileURLToPath } from "node:url"

const port = 4445
const BUNDLE = "me.tomasrav.tauri-plugin-bare-kit-tests"

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

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
  connectionRetryTimeout: 180_000,
  connectionRetryCount: 3,
  framework: "mocha",
  reporters: ["spec"],
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },

  onPrepare: async function (config, capabilities) {
    const device = getDevice()

    simctl("boot", device)
    simctl("bootstatus", device, "-b")
    simctl(
      "install",
      "booted",
      path.resolve(
        __dirname,
        "src-tauri/gen/apple/build/arm64-sim/tauri-plugin-bare-kit-tests.app",
      ),
    )
    simctl("launch", "booted", BUNDLE)

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
    simctl("terminate", "booted", BUNDLE)
    simctl("shutdown", "booted")
  },
}

function getDevice() {
  const result = spawnSync("xcrun", ["simctl", "list", "devices", "available", "--json"], {
    encoding: "utf-8",
    stdio: "pipe",
  })

  if (result.error) {
    throw result.error
  }

  if (result.status !== 0) {
    throw new Error("Could not list available devices")
  }

  const json = JSON.parse(result.output.join(""))

  for (const version of Object.values(json.devices)) {
    for (const device of version) {
      if (device.name === "iPhone 17") {
        return device.udid
      }
    }
  }

  throw new Error("Could not find specified device")
}

function simctl(...args) {
  const result = spawnSync("xcrun", ["simctl", ...args], {
    encoding: "utf-8",
    stdio: "inherit",
  })

  if (result.error) {
    throw result.error
  }

  if (result.status !== 0) {
    throw new Error(`'xcrun simctl ${args.join(" ")}' exited with status ${result.status}`)
  }
}
