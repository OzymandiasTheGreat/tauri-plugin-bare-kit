import fs from "fs/promises"
import link from "bare-link"
import make from "bare-make"
import { spawn } from "child_process"
import npx from "libnpmexec"
import os from "os"
import path from "path"
import { fileURLToPath } from "url"
import { parse, stringify } from "yaml"
import { exists, find_root } from "./util.mjs"

export default class BareKitLinker {
  static async android(arch, profile = "debug") {}

  static async ios(arch, profile = "debug") {
    const BareKit = "BareKit.xcframework"
    const XCode = "src-tauri/gen/apple"
    const Frameworks = path.join(XCode, "Frameworks")
    const Template = path.join(XCode, "project.yml")
    const SearchPath = "$(PROJECT_DIR)/Frameworks"
    const ARM64_PATH_KEY = "LIBRARY_SEARCH_PATHS[arch=arm64]"
    const X64_PATH_KEY = "LIBRARY_SEARCH_PATHS[arch=x86_64]"

    const source = await find_root(path.dirname(fileURLToPath(import.meta.url))).catch(
      (err) => {
        console.error(`🛑`, err)
        process.exit(1)
      },
    )
    const node = await find_root().catch((err) => {
      console.error(`🛑`, err)
      process.exit(1)
    })
    const temp = path.join(os.tmpdir(), "tauri-plugin-bare-kit", profile)
    const scratch = path.join(temp, "scratch")
    const dest = path.join(temp, "Frameworks", "ios")
    const archs = [["arm64", false], os.arch() === "arm64" ? ["arm64", true] : ["x64", true]]
    const bare_kit = path.join(dest, BareKit)

    if (await exists(bare_kit)) {
      await fs.rm(bare_kit, { force: true, recursive: true })
    }

    const frameworks = ["-create-xcframework"]

    for (const [arch, simulator] of archs) {
      const target = `ios-${arch}${simulator ? "-simulator" : ""}`
      const build = path.join(temp, "build", target)
      const prefix = path.join(scratch, target)

      await make
        .generate({
          source,
          build,
          platform: "ios",
          arch,
          simulator,
          debug: profile === "debug",
          stdio: "inherit",
        })
        .catch((err) => {
          console.error(`🛑`, err)
          process.exit(1)
        })
      await make
        .build({
          build,
          stdio: "inherit",
        })
        .catch((err) => {
          console.error(`🛑`, err)
          process.exit(1)
        })
      await make
        .install({
          build,
          prefix,
          stdio: "inherit",
        })
        .catch((err) => {
          console.error(`🛑`, err)
          process.exit(1)
        })

      const framework = path.join(prefix, "Frameworks/BareKit.framework")
      frameworks.push("-framework", framework)
    }

    frameworks.push("-output", bare_kit)

    const xcodebuild = spawn("xcodebuild", frameworks)
    await new Promise((resolve, reject) => {
      xcodebuild.on("error", reject)
      xcodebuild.on("close", resolve)
    }).catch((err) => {
      console.error(`🛑`, err)
      process.exit(1)
    })
    console.log(`🟢 ${BareKit} combined binary built`)

    try {
      for await (const _ of link(node, { preset: "ios", out: dest })) {
      }
    } catch (err) {
      console.error(`🛑`, err)
      process.exit(1)
    }
    console.log(`🟢 Addons linked`)

    const template = path.join(node, Template)

    if (!(await exists(template))) {
      const tauri = await npx({ args: ["tauri", "ios", "init"], path: node, yes: true })

      if (tauri.code !== 0) {
        console.error(`🛑 Failed to initialize iOS project`)
        console.error(`Run "npx tauri ios init" and try again`)
        process.exit(1)
      }

      console.log(`🟢 Initialized iOS project at ${path.dirname(template)}`)
    } else {
      console.log(`🟢 Found iOS project at ${path.dirname(template)}`)
    }

    const project = parse(await fs.readFile(template, "utf8"))
    const target = `${project.name}_iOS`
    const settings = project.targets[target].settings.base
    const dependencies = project.targets[target].dependencies

    if (!settings[ARM64_PATH_KEY].includes(SearchPath)) {
      settings[ARM64_PATH_KEY] = `${settings[ARM64_PATH_KEY]} ${SearchPath}`
    }

    if (!settings[X64_PATH_KEY].includes(SearchPath)) {
      settings[X64_PATH_KEY] = `${settings[X64_PATH_KEY]} ${SearchPath}`
    }

    const _frameworks = await fs
      .readdir(dest)
      .then((frameworks) => frameworks.filter((f) => path.extname(f) === ".xcframework"))
    const filtered = dependencies.filter(
      (d) => !_frameworks.some((f) => d.framework?.includes(f.slice(0, f.indexOf(".")))),
    )

    for (const framework of _frameworks) {
      filtered.push({ framework: path.join(SearchPath, framework) })
    }

    project.targets[target].dependencies = filtered

    await fs.writeFile(template, stringify(project)).catch((err) => {
      console.error(`🛑`, err)
      process.exit(1)
    })
    console.log(`🟢 XCode project template updated`)

    const xcodegen = spawn("xcodegen", ["generate", "--spec", template], { stdio: "inherit" })
    await new Promise((resolve, reject) => {
      xcodegen.on("error", reject)
      xcodegen.on("close", resolve)
    }).catch((err) => {
      console.error(`🛑`, err)
      process.exit(1)
    })
    console.log(`🟢 XCode project generated`)

    await fs.symlink(dest, path.join(node, Frameworks)).catch((err) => {
      if (err.code === "EEXIST") return
      console.error(`🛑`, err)
      process.exit(1)
    })
    console.log(`🟢 XCFrameworks installed`)

    console.log(`🚀 Ready to build for iOS`)
    process.exit(0)
  }

  static async darwin(arch, profile = "debug") {}

  static async linux(arch, profile = "debug") {}

  static async win32(arch, profile = "debug") {}
}
