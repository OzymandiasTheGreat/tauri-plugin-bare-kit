import fs from "fs/promises"
import path from "path"

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

export async function find_root() {
  let cwd = process.env.INIT_CWD
  let parent

  while (!(await exists(path.join(cwd, "package.json")))) {
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
