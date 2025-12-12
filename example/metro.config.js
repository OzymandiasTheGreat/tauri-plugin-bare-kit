const { getDefaultConfig } = require("expo/metro-config")
const path = require("path")

const root = path.resolve(__dirname, "..")

// /** @type {import('expo/metro-config').MetroConfig} */
const config = getDefaultConfig(__dirname)

config.watchFolders = [__dirname, root]
config.resolver.nodeModulesPaths = [
  path.resolve(__dirname, "node_modules"),
  path.resolve(root, "node_modules"),
]
config.resolver.blockList = [/\/target\//].concat(config.resolver.blockList)
config.resolver.unstable_enablePackageExports = true
config.resolver.unstable_conditionNames = ["react-native", "require"]

module.exports = config
