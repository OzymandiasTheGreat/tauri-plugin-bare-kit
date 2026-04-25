#! /usr/bin/env node
import fs from "fs/promises"
import path from "path"
import { coerce } from "semver"
import { get_dependencies, safe_fetch } from "./util.mjs"

const root = path.dirname(import.meta.dirname)
const cmake_regex =
  /(?<=^fetch_package\("github:holepunchto\/bare-kit@)\d+?\.\d+?\.\d+?(?="\)$)/gm

const bare_kit_version = await get_bare_kit_version()
const cmake_lists = await fs.readFile(path.join(root, "CMakeLists.txt"), "utf-8")
await fs.writeFile("CMakeLists.txt", cmake_lists.replace(cmake_regex, coerce(bare_kit_version)))

const dependencies = await get_dependencies(bare_kit_version)
const pkg = JSON.parse(await fs.readFile(path.join(root, "package.json"), "utf-8"))
pkg.overrides = dependencies

for (const dep of Object.keys(pkg.dependencies)) {
  if (dep in dependencies) {
    pkg.dependencies[dep] = dependencies[dep]
  }
}

await fs.writeFile("package.json", JSON.stringify(pkg, null, 2))

const meta = { version: coerce(bare_kit_version) }
await fs.writeFile(
  path.join(import.meta.dirname, "bare-kit.json"),
  JSON.stringify(meta, null, 2),
)

async function get_bare_kit_version() {
  const tags = await safe_fetch("https://api.github.com/repos/holepunchto/bare-kit/tags")

  if (!tags) {
    throw new Error("Could not fetch bare-kit version")
  }

  return tags[0].name
}
