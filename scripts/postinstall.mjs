#! /usr/bin/env node
import link from "bare-link"
import { spawn } from "child_process"
import fs from "fs/promises"
import os from "os"
import path from "path"
import TOML from "@ltd/j-toml"
import YAML from "yaml"
import pkg from "../package.json" with { type: "json" }
import { exists, find_root } from "./util.mjs"

if (process.env.INIT_CWD !== process.cwd()) {
  const AUTOLINK = "tauri_plugin_bare_kit::autolink();"
  const FRAMEWORK = "$(PROJECT_DIR)/Externals/$(NATIVE_ARCH)/$(CONFIGURATION)/"

  const root = await find_root()
  const src_tauri = path.join(root, "src-tauri")
  const project_yaml = path.join(src_tauri, "gen/apple/project.yml")
  const link_ios = await exists(project_yaml)
  const cargo_toml = TOML.parse(await fs.readFile(path.join(src_tauri, "Cargo.toml"), "utf-8"))
  const build_rs = await fs.readFile(path.join(src_tauri, "build.rs"), "utf-8")

  cargo_toml.dependencies["tauri-plugin-bare-kit"] = pkg.version
  cargo_toml["build-dependencies"]["tauri-plugin-bare-kit"] = TOML.inline({
    "version": pkg.version,
    "default-features": false,
    "features": ["build"],
  })

  const indent_regex = /(?<=fn main\(\) {\r?\n)\s+/g
  const insert_regex = /(?<=fn main\(\) {\r?\n\s+)\b/
  const indentation = "".padStart(build_rs.match(indent_regex)[0].length)
  const new_line = build_rs.includes("\r") ? "\r\n" : "\n"
  const build_rs_linked = build_rs.replace(insert_regex, `${AUTOLINK}${new_line}${indentation}`)

  await fs.writeFile(
    path.join(src_tauri, "Cargo.toml"),
    TOML.stringify(cargo_toml, { newline: "\n", newlineAround: "section" }),
  )

  if (!build_rs.includes(AUTOLINK)) {
    await fs.writeFile(path.join(src_tauri, "build.rs"), build_rs_linked)
  }

  if (process.platform === "darwin" && link_ios) {
    const yaml = YAML.parse(await fs.readFile(project_yaml, "utf-8"))
    const target = `${yaml.name}_iOS`
    const ios_dependencies = yaml.targets[target].dependencies
    const frameworks = ["BareKit.framework"]
    const out = path.join(os.tmpdir(), "tauri-plugin-bare-kit", "ios")
    const cwd = process.cwd()

    if (await exists(out)) await fs.rm(out, { recursive: true, force: true })

    process.chdir(root)

    for await (const resource of link(root, { hosts: ["ios-arm64"], out })) {
      if (path.extname(resource) == ".framework") {
        const framework = path.basename(resource)

        frameworks.push(framework)
      }
    }

    process.chdir(cwd)

    await fs.rm(out, { recursive: true, force: true })

    for (const framework of frameworks) {
      const dependency = `${FRAMEWORK}${framework}`

      if (!ios_dependencies.find((dep) => dep.framework === dependency)) {
        ios_dependencies.push({ framework: dependency })
      }
    }

    yaml.targets[target].dependencies = ios_dependencies
    await fs.writeFile(project_yaml, YAML.stringify(yaml))

    const xcodegen = spawn("xcodegen", ["generate", "--spec", project_yaml], {
      stdio: "inherit",
    })

    await new Promise((resolve, reject) => {
      xcodegen.on("error", reject)
      xcodegen.on("close", resolve)
    })
  }
}
