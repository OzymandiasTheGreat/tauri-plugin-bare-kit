#! /usr/bin/env node
import fs from "fs/promises"
import path from "path"
import { inc, parse } from "semver"
import TOML from "smol-toml"

const cmake_regex =
  /(?<=project\(tauri_plugin_bare_kit LANGUAGES C CXX VERSION )\d+?\.\d+?\.\d+?(?=\)$)/gm

const version_type = process.argv[2]
const root = path.dirname(import.meta.dirname)
const pkg = JSON.parse(await fs.readFile(path.join(root, "package.json"), "utf-8"))
const version = inc(pkg.version, version_type)
const cargo_toml = TOML.parse(await fs.readFile(path.join(root, "Cargo.toml"), "utf-8"))
const cmake_lists = await fs.readFile(path.join(root, "CMakeLists.txt"), "utf-8")

pkg.version = version
cargo_toml.package.version = version

await fs.writeFile(path.join(root, "package.json1"), JSON.stringify(pkg, null, 2))
await fs.writeFile(path.join(root, "Cargo.toml1"), TOML.stringify(cargo_toml))
await fs.writeFile(
  path.join(root, "CMakeLists.txt1"),
  cmake_lists.replace(cmake_regex, version),
)
