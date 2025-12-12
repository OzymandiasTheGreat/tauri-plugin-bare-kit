const fs = require("fs/promises")
const os = require("os")
const path = require("path")
const { exists } = require("./util")

module.exports = class BareKitCleaner {
  static async clean() {
    const temp = path.join(os.tmpdir(), "tauri-plugin-bare-kit")

    if (!(await exists(temp))) {
      console.log(`🟢 Nothing to do!`)
      process.exit(0)
    } else {
      try {
        await fs.rm(temp, { recursive: true, force: true })
        console.log(`🚀 Build artifacts and caches cleared!`)
        process.exit(0)
      } catch (err) {
        console.error(`🛑 Failed to remove build artifacts and caches`)
        console.error(err.message)
        process.exit(1)
      }
    }
  }
}
