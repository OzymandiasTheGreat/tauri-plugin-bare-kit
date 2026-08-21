#! /usr/bin/env node
import { spawnSync } from "child_process"
import fs from "fs/promises"
import path from "path"
import TOML from "@ltd/j-toml"

const root = path.dirname(import.meta.dirname)
const pkg = JSON.parse(await fs.readFile(path.join(root, "package.json"), "utf-8"))
const cargo_toml = TOML.parse(await fs.readFile(path.join(root, "Cargo.toml"), "utf-8"))

cargo_toml.package.version = pkg.version

await fs.writeFile(
  path.join(root, "Cargo.toml"),
  TOML.stringify(cargo_toml, { newline: "\n", newlineAround: "section" }),
)

spawnSync("cargo", [
  "generate-lockfile",
  "--manifest-path",
  path.join(root, "Cargo.toml"),
  "--offline",
])
spawnSync("cargo", [
  "generate-lockfile",
  "--manifest-path",
  path.join(root, "example/src-tauri", "Cargo.toml"),
  "--offline",
])
spawnSync("cargo", [
  "generate-lockfile",
  "--manifest-path",
  path.join(root, "tests/src-tauri", "Cargo.toml"),
  "--offline",
])
spawnSync("git", [
  "add",
  path.join(root, "Cargo.toml"),
  path.join(root, "Cargo.lock"),
  path.join(root, "example/src-tauri", "Cargo.lock"),
  path.join(root, "tests/src-tauri", "Cargo.lock"),
])
