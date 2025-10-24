import { invoke, transformCallback } from "@tauri-apps/api/core"
import b4a from "b4a"

type on_poll_callback = ((data: { readable: boolean; writable: boolean }) => void) | null

export default class NativeBareKit {
  static async invalidate(): Promise<void> {
    return invoke(format("bare_invalidate"))
  }

  static async init(
    memoryLimit: number,
    assets: string | null,
    pollCallback: on_poll_callback,
  ): Promise<number> {
    const onPoll = pollCallback != null ? transformCallback(pollCallback as any, false) : null
    return invoke<number>(format("bare_init"), {
      payload: { memoryLimit, assets, onPoll },
    })
  }

  static async startFile(id: number, filename: string, args: string[] = []): Promise<void> {
    return invoke(format("bare_start_file"), { payload: { id, filename, args } })
  }

  static async startUTF8(
    id: number,
    filename: string,
    source: string,
    args: string[] = [],
  ): Promise<void> {
    return invoke(format("bare_start_utf8"), { payload: { id, filename, source, args } })
  }

  static async startBytes(
    id: number,
    filename: string,
    source: Uint8Array,
    args: string[] = [],
  ): Promise<void> {
    return invoke(format("bare_start_bytes"), { payload: { id, filename, source, args } })
  }

  static async read(id: number): Promise<Uint8Array | null> {
    return invoke<ArrayBuffer>(format("bare_read"), { payload: { id } }).then((res) => {
      if (res.byteLength === 0) {
        return null
      }
      return b4a.from(res)
    })
  }

  static async write(id: number, data: Uint8Array | null): Promise<number> {
    const payload = data ? b4a.concat([b4a.alloc(1, id), data]) : b4a.allocUnsafe(0)
    return invoke<number>(format("bare_write"), payload)
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

  static async wakeup(id: number, deadline: number): Promise<void> {
    return invoke(format("bare_wakeup"), { payload: { id, deadline } })
  }

  static async terminate(id: number): Promise<void> {
    return invoke(format("bare_terminate"), { payload: { id } })
  }
}

function format(fn: string): string {
  return `plugin:bare-kit|${fn}`
}
