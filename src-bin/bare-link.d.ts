declare module "bare-link" {
  type Preset =
    | "android"
    | "apple"
    | "darwin"
    | "desktop"
    | "ios"
    | "linux"
    | "mobile"
    | "win32"
  export default async function* link(
    base: string,
    options: {
      target?: string[]
      out: string
      preset?: Preset
      sign?: boolean

      // Apple signing options
      identity?: string
      keychain?: string

      // Windows signing options
      subject?: string
      subjectName?: string
      thumbprint?: string
    },
  ): AsyncGenerator<void>
}
