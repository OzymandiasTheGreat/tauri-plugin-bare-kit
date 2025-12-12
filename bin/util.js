const fs = require("fs/promises")
const path = require("path")

module.exports.find_root = async function (cwd = process.cwd()) {
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

module.exports.exists = async function (path) {
  return fs
    .access(path, fs.constants.F_OK)
    .then(() => true)
    .catch(() => false)
}
