#!/usr/bin/env node
import fs from "fs/promises"
import link from "bare-link"
import { spawn } from "child_process"
import os from "os"
import path from "path"
import { parse, stringify } from "yaml"

interface Dependency {
  framework?: string
  sdk?: string
}

const BareKit = "BareKit.framework"
const Template = "src-tauri/gen/apple/project.yml"
const SearchPath = "$(PROJECT_DIR)/Frameworks"
const ARM64_PATH_KEY = "LIBRARY_SEARCH_PATHS[arch=arm64]"
const X64_PATH_KEY = "LIBRARY_SEARCH_PATHS[arch=x86_64]"

let addon_tmp = null
try {
  const project_root = await find_root()
  const template_path = path.join(project_root, Template)

  if (!(await exists(template_path))) {
    throw new Error("iOS project not initialized.\nRun `npx tauri ios init` first")
  }

  const template = parse(await fs.readFile(template_path, "utf8"))
  const target = `${template.name}_iOS`
  const settings = template.targets[target].settings.base
  const dependencies: Dependency[] = template.targets[target].dependencies

  if (!settings[ARM64_PATH_KEY].includes(SearchPath)) {
    settings[ARM64_PATH_KEY] = `${settings[ARM64_PATH_KEY]} ${SearchPath}`
  }

  if (!settings[X64_PATH_KEY].includes(SearchPath)) {
    settings[X64_PATH_KEY] = `${settings[X64_PATH_KEY]} ${SearchPath}`
  }

  addon_tmp = await fs.mkdtemp(path.join(os.tmpdir(), "bare-kit-"))
  await link(project_root, { preset: "ios", needs: [BareKit], out: addon_tmp })

  const frameworks = await fs
    .readdir(addon_tmp)
    .then((fr) => [
      BareKit,
      ...fr.map((fn) => `${path.basename(fn, ".xcframework")}.framework`),
    ])
  const framework_names = frameworks.map((f) => {
    const index = f.indexOf(".")
    return f.slice(0, index)
  })
  const filtered_dependencies = dependencies.filter(
    (d) => !framework_names.some((f) => d.framework?.includes(f)),
  )

  for (const framework of frameworks) {
    filtered_dependencies.push({ framework: path.join(SearchPath, framework) })
  }

  template.targets[target].dependencies = filtered_dependencies

  await fs.writeFile(template_path, stringify(template))

  const xcodegen = spawn("xcodegen", ["generate", "--spec", template_path])
  await new Promise((resolve, reject) => {
    xcodegen.on("error", reject)
    xcodegen.on("close", resolve)
  })

  console.log("🚀 iOS project successfully linked")
} finally {
  if (addon_tmp) {
    await fs.rm(addon_tmp, { recursive: true, force: true })
  }
}

async function find_root(): Promise<string> {
  let cwd = process.cwd()

  if (await exists(path.join(cwd, "package.json"))) {
    return cwd
  }

  while ((cwd = path.dirname(cwd))) {
    if (await exists(path.join(cwd, "package.json"))) {
      return cwd
    }
  }

  throw new Error("Could not determine root project")
}

async function exists(path: string): Promise<boolean> {
  return fs
    .access(path, fs.constants.F_OK)
    .then(() => true)
    .catch(() => false)
}
