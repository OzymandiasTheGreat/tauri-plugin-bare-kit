#! /usr/bin/env node
import fs from "fs/promises"
import path from "path"
import TOML from "@ltd/j-toml"
import meta from "./bare-kit.json" with { type: "json" }
import { exists, find_root, get_dependencies } from "./util.mjs"

const AUTOLINK = "tauri_plugin_bare_kit::autolink();"

const root = await find_root()
const src_tauri = path.join(root, "src-tauri")
const pkg = JSON.parse(await fs.readFile(path.join(root, "package.json"), "utf-8"))
const pkg_lock = await exists(path.join(root, "package-lock.json"))
const yarn_lock = await exists(path.join(root, "yarn.lock"))
const cargo_toml = TOML.parse(await fs.readFile(path.join(src_tauri, "Cargo.toml"), "utf-8"))
const build_rs = await fs.readFile(path.join(src_tauri, "build.rs"), "utf-8")
const dependencies = await get_dependencies(`v${meta.version}`)

for (const dep of Object.keys(pkg.dependencies)) {
  if (dep in dependencies) {
    pkg.dependencies[dep] = dependencies.dep
  }
}

if (pkg_lock) {
  pkg.overrides = Object.assign(pkg.overrides, dependencies)
}

if (yarn_lock) {
  pkg.resolutions = Object.assign(pkg.resolutions, dependencies)
}

cargo_toml.dependencies["tauri-plugin-bare-kit"] = meta.version
cargo_toml["build-dependencies"]["tauri-plugin-bare-kit"] = TOML.inline({
  "version": meta.version,
  "default-features": false,
  "features": ["build"],
})

const indent_regex = /(?<=fn main\(\) {\r?\n)\s+/g
const insert_regex = /(?<=fn main\(\) {\r?\n\s+)\b/
const indentation = "".padStart(build_rs.match(indent_regex)[0].length)
const new_line = build_rs.includes("\r") ? "\r\n" : "\n"
const build_rs_linked = build_rs.replace(insert_regex, `${AUTOLINK}${new_line}${indentation}`)

await fs.writeFile(path.join(root, "package.json"), JSON.stringify(pkg, null, 2))
await fs.writeFile(
  path.join(src_tauri, "Cargo.toml"),
  TOML.stringify(cargo_toml, { newline: "\n", newlineAround: "section" }),
)
if (!build_rs.includes(AUTOLINK)) {
  await fs.writeFile(path.join(src_tauri, "build.rs"), build_rs_linked)
}
