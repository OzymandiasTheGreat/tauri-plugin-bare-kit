import fs from "fs/promises"
import path from "path"

export async function find_root(cwd = process.cwd()): Promise<string> {
  if (await exists(path.join(cwd, "package.json"))) {
    return cwd
  }

  while ((cwd = path.dirname(cwd))) {
    if (await exists(path.join(cwd, "package.json"))) {
      return cwd
    }
  }

  throw new Error("Could not determine node project path")
}

export async function exists(path: string): Promise<boolean> {
  return fs
    .access(path, fs.constants.F_OK)
    .then(() => true)
    .catch(() => false)
}
