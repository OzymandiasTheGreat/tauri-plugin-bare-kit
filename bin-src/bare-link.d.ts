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
  export default async function link(
    base: string,
    options: {
      target?: string
      preset?: Preset
      needs?: string[]
      out?: string
    },
  ): Promise<void>
}
