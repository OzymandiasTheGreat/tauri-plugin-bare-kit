import { invoke, transformCallback } from "@tauri-apps/api/core"

type on_poll_callback = ((readable: boolean, writable: boolean) => void) | null

export default class NativeBareKit {
  static async invalidate(): Promise<void> {
    return invoke(format("bare_invalidate"))
  }

  static async init(memoryLimit: number, assets: string | null, pollCallback: on_poll_callback): Promise<number> {
    const on_poll = pollCallback != null ? transformCallback(pollCallback as any, false) : null
    return invoke<{ data: number }>(format("bare_new"), { payload: { memoryLimit, assets, on_poll }}).then((res) => res.data)
  }

  static async start(id: number, filename: string, source: Uint8Array | null, argv: string[]): Promise<void> {
    return invoke(format("bare_start"), { payload: { id, filename, source, argv } })
  }

  static async read(id: number): Promise<Uint8Array | null> {
    return invoke<{ data: Uint8Array | null }>(format("bare_read"), { payload: { id } }).then((res) => res.data)
  }

  static async write(id: number, data: Uint8Array | null): Promise<number> {
    return invoke<{ data: number }>(format("bare_write"), { payload: { id, data } }).then((res) => res.data)
  }

  static async update(id: number, readable: boolean, writable: boolean): Promise<void> {
    return invoke(format("bare_update"), { payload: { id, readable, writable } })
  }

  static async suspend(id: number, linger: number): Promise<void> {
    return invoke(format("bare_suspend"), { payload: { id, linger } })
  }

  static async resume(id: number): Promise<void> {
    return invoke(format("bare_resume"), { payload: { id } })
  }

  static async terminate(id: number): Promise<void> {
    return invoke(format("bare_terminate"), { payload: { id } })
  }
}

function format(fn: string): string {
  return `plugin:bare-kit|${fn}`
}
