var Bare: {
  platform: "android" | "darwin" | "ios" | "linux" | "win32"
  arch: "arm" | "arm64" | "ia32" | "mips" | "mipsel" | "x64"
  simulator: boolean
  argv: string[]
  pid: number
  exitCode: number
  version: string
  versions: Record<string, string>

  exit(code?: number): void
  suspend(linger?: number): void
  wakeup(deadline?: number): void
  idle(): void
  resume(): void

  on(event: "uncaughtException", handler: (err: Error) => void): Bare
  on(
    event: "unhandledRejection",
    handler: (reason: any, promise: Promise<unknown>) => void,
  ): Bare
  on(event: "beforeExit", handler: (code: number) => void): Bare
  on(event: "exit", handler: (code: number) => void): Bare
  on(event: "suspend", handler: (linger: number) => void): Bare
  on(event: "wakeup", handler: (deadline: number) => void): Bare
  on(event: "idle", handler: () => void): Bare
  on(event: "resume", handler: () => void): Bare
}

var BareKit: {
  IPC: import("bare-stream").Duplex
}
