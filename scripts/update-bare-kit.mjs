#! /usr/bin/env node
import fs from "fs/promises"
import path from "path"
import { coerce } from "semver"
import { safe_fetch } from "./util.mjs"

const root = path.dirname(import.meta.dirname)
const cmake_regex =
  /(?<=^fetch_package\("github:holepunchto\/bare-kit@)\d+?\.\d+?\.\d+?(?="\)$)/gm

const bare_kit_version = await get_bare_kit_version()
const cmake_lists = await fs.readFile(path.join(root, "CMakeLists.txt"), "utf-8")
await fs.writeFile("CMakeLists.txt", cmake_lists.replace(cmake_regex, bare_kit_version))

async function get_bare_kit_version() {
  const tags = await safe_fetch("https://api.github.com/repos/holepunchto/bare-kit/tags")

  if (!tags) {
    throw new Error("Could not fetch bare-kit version")
  }

  return coerce(tags[0].name)
}
