import { Duplex } from "streamx"
import EventEmitter, { EventMap } from "bare-events"

declare class IPC extends Duplex {
  readonly worklet: Worklet
}

export interface WorkletEvents extends EventMap {
  suspend: [linger: number]
  wakeup: [deadline: number]
  idle: []
  resume: []
}

export interface WorkletOptions {
  memoryLimit?: number
  assets?: string
}

export class Worklet extends EventEmitter<WorkletEvents> {
  static optimizeForMemory(enabled: boolean): Promise<void>
  static init(options?: WorkletOptions): Promise<Worklet>
  readonly IPC: IPC

  start(filename: string, args?: string[]): Promise<void>
  start(filename: string, source: Uint8Array, args?: string[]): Promise<void>
  start(filename: string, source: string, args?: string[]): Promise<void>

  suspend(linger?: number): Promise<void>
  resume(): Promise<void>
  wakeup(deadline?: number): Promise<void>
  update(state?: unknown): Promise<void>
  terminate(): Promise<void>

  static suspend(linger?: number): Promise<void>
  static resume(): Promise<void>
  static wakeup(deadline?: number): Promise<void>
  static update(state?: unknown): Promise<void>
}
