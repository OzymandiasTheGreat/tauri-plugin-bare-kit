import EventEmitter from "bare-events"
import { Duplex } from "bare-stream"

declare class BareIPC extends Duplex {
  readonly worklet: BareWorklet

  toJSON(): { worklet: { started: boolean; terminated: boolean; suspended: boolean } }
}

declare class BareWorklet extends EventEmitter<{
  start: []
  terminate: []
  suspend: []
  resume: []
  wakeup: []
}> {
  static init(options?: { memoryLimit?: number; assets?: string | null }): Promise<BareWorklet>

  readonly handle: number
  readonly IPC: BareIPC
  readonly started: boolean
  readonly terminated: boolean
  readonly suspended: boolean

  start(filename: string, source: string | Uint8Array | null, args?: string[]): Promise<void>
  suspend(linger?: number): Promise<void>
  static suspend(linger?: number): Promise<void>
  resume(): Promise<void>
  static resume(): Promise<void>
  wakeup(deadline?: number): Promise<void>
  static wakeup(deadline?: number): Promise<void>
  terminate(): Promise<void>
  toJSON(): { started: boolean; terminated: boolean; suspended: boolean }
}

export declare const Worklet: typeof BareWorklet
export type Worklet = BareWorklet
export {}
