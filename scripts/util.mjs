import fs from "fs/promises"
import path from "path"

const PKG = "package.json"

export async function safe_fetch(uri) {
  return fetch(uri)
    .then((response) => {
      if (response.ok) {
        return response.json()
      }
      return null
    })
    .catch(() => null)
}

export async function get_dependencies(version) {
  const lock = await safe_fetch(
    `https://raw.githubusercontent.com/holepunchto/bare-kit/${version}/package-lock.json`,
  )

  if (!lock) {
    throw new Error("Could not retrieve bare-kit dependencies")
  }

  const dependencies = {}

  for (const [key, dep] of Object.entries(lock.packages)) {
    if (!dep.version || dep.dev || dep.optional || dep.devOptional) {
      continue
    }

    const name = path.basename(key)

    if (name.startsWith("bare")) {
      dependencies[name] = dep.version
    }
  }

  return dependencies
}

export async function find_root() {
  let cwd = process.env.INIT_CWD
  let parent

  while (!(await exists(path.join(cwd, PKG)))) {
    parent = path.dirname(cwd)

    if (parent === cwd) {
      throw new Error("Could not determine package root!")
    }

    cwd = parent
  }

  return cwd
}

export async function exists(filepath) {
  try {
    await fs.access(filepath, fs.constants.F_OK)
    return true
  } catch (err) {
    if (err.code === "ENOENT") {
      return false
    }
    throw err
  }
}
