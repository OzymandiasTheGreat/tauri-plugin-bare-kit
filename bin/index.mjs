#!/usr/bin/env node
import { command, flag, header } from "paparam"
import BareKitCleaner from "./clean.mjs"
import BareKitLinker from "./link.mjs"

const FAT = ["android", "ios", "darwin"]
const THIN = ["linux", "win32"]
const PLATFORMS = [...FAT, ...THIN]
const ARCHS = {
  android: ["arm", "arm64", "ia32", "x64"],
  ios: ["arm64", "x64"],
  darwin: ["arm64", "x64"],
  linux: ["arm64", "x64"],
  win32: ["arm64", "x64"],
}

const clean = command("clean", header("Build artifact cleaner for tauri-plugin-bare-kit"))
const link = command(
  "link",
  header("Manual binary linker for tauri-plugin-bare-kit"),
  flag(
    "--platform|-p <platform>",
    "The platform to link. Only ios requires manual linking, as other platforms are linked automatically during build. Valid values are android, ios, darwin, linux, win32.",
  ),
  flag(
    "--arch|-a <arch>",
    "The architecture to link. Only linux and windows require arch to be specified as other platforms link all supported archs.",
  ),
  flag("--profile|-d <profile>", "The profile to use for linking. Defaults to debug"),
)
const cmd = command(
  "bare-kit",
  header("Utility commands for tauri-plugin-bare-kit"),
  clean,
  link,
)

const { flags, name } = cmd.parse()

if (name === "clean") {
  await BareKitCleaner.clean()
} else if (name === "link") {
  if (!flags.platform) {
    if (!flags.arch && !flags.profile) {
      console.log(`🟢 No platform specified`)
      console.log(cmd.help())
      process.exit(0)
    }

    console.error(`🛑 "--platform" is required`)
    process.exit(1)
  }

  if (!PLATFORMS.includes(flags.platform)) {
    console.error(`🛑 "--platform" must be one of ${PLATFORMS.join(", ")}`)
    process.exit(1)
  }

  if (flags.arch) {
    if (FAT.includes(flags.platform)) {
      console.log(`🟢 ${flags.platform} always builds all supported architectures`)
    }

    if (!ARCHS[flags.platform]?.includes(flags.arch)) {
      console.error(
        `🛑 Supported architectures for ${flags.platform} are ${ARCHS[flags.platform]?.join(
          ", ",
        )}`,
      )
      process.exit(1)
    }
  }

  if (flags.platform !== "ios") {
    console.log(`🟢 Nothing to do! ${flags.platform} is linked automatically during build`)
    process.exit(0)
  }

  await BareKitLinker[flags.platform](flags.arch, flags.profile)
} else {
  console.log(cmd.help())
}
