// @ts-ignore
import b4a from "b4a"
import EventEmitter from "bare-events"
// @ts-ignore
import type { Duplex } from "bare-stream"
// @ts-ignore
import { Duplex } from "streamx"
import NativeBareKit from "./module"

enum CONSTANTS {
  STARTED = 0x1,
  TERMINATED = 0x2,
  SUSPENDED = 0x4,
}

type Callback = (err: Error | null) => void

// @ts-ignore
class BareKitIPC extends Duplex {
  protected _worklet: BareKitWorklet

  protected _pendingOpen: Callback | null
  protected _pendingRead: Callback | null
  protected _pendingWrite: [Uint8Array, Callback] | null

  constructor(worklet: BareKitWorklet) {
    super()

    this._worklet = worklet
    this._poll = this._poll.bind(this)

    this._pendingOpen = null
    this._pendingRead = null
    this._pendingWrite = null
  }

  get worklet() {
    return this._worklet
  }

  toJSON() {
    return {
      worklet: this.worklet,
    }
  }

  _open(callback: Callback) {
    if (this._worklet.started) {
      callback(null)
    } else {
      this._pendingOpen = callback
    }
  }

  async _update() {
    return NativeBareKit.update(
      this._worklet.handle,
      this._pendingRead !== null,
      this._pendingWrite !== null,
    )
  }

  async _poll(data: { readable: boolean; writable: boolean }) {
    if (this._worklet.terminated) return
    if (data.readable) await this._continueRead()
    if (data.writable) await this._continueWrite()

    await NativeBareKit.notify(this._worklet.handle)
  }

  // @ts-ignore
  async _read(callback: Callback) {
    const data = await NativeBareKit.read(this._worklet.handle)

    if (data) {
      this.push(data)
      callback(null)
    } else {
      this._pendingRead = callback
      await this._update()
    }
  }

  // @ts-ignore
  async _write(data: Uint8Array, callback: Callback) {
    if (!b4a.isBuffer(data)) {
      data = b4a.from(data)
    }

    const written = await NativeBareKit.write(this._worklet.handle, data)

    if (written === data.byteLength) callback(null)
    else {
      this._pendingWrite = [data.subarray(written), callback]
      await this._update()
    }
  }

  _continueOpen(err: Error | null) {
    if (this._pendingOpen === null) {
      if (err) this.destroy(err)
    } else {
      const callback = this._pendingOpen
      this._pendingOpen = null
      callback(err)
    }
  }

  async _continueRead() {
    if (this._pendingRead === null) return
    const callback = this._pendingRead
    this._pendingRead = null
    await this._update()
    await this._read(callback)
  }

  async _continueWrite() {
    if (this._pendingWrite === null) return
    const [data, callback] = this._pendingWrite
    this._pendingWrite = null
    await this._update()
    await this._write(data, callback)
  }
}

class BareKitWorklet extends EventEmitter {
  protected static _worklets = new Set<BareKitWorklet>()

  protected _state: number
  protected _ipc: BareKitIPC
  protected _handle!: number

  constructor() {
    super()

    this._state = 0
    this._ipc = new BareKitIPC(this)
  }

  static async init(
    options: { memoryLimit?: number; assets?: string | null } = {},
  ): Promise<BareKitWorklet> {
    const { memoryLimit = 0, assets = null } = options

    if (typeof memoryLimit !== "number") {
      throw new TypeError(
        `Memory limit must be a number. Received type ${typeof memoryLimit} (${memoryLimit})`,
      )
    }

    if (typeof assets !== "string" && assets !== null) {
      throw new TypeError(
        `Asset path must be a string. Received type ${typeof assets} (${assets})`,
      )
    }

    const worklet = new BareKitWorklet()
    worklet._handle = await NativeBareKit.init(memoryLimit, assets, worklet._ipc._poll)

    return worklet
  }

  get handle() {
    return this._handle
  }

  get IPC() {
    return this._ipc
  }

  get started() {
    return (this._state & CONSTANTS.STARTED) !== 0
  }

  get terminated() {
    return (this._state & CONSTANTS.TERMINATED) !== 0
  }

  get suspended() {
    return (this._state & CONSTANTS.SUSPENDED) !== 0
  }

  async start(filename: string, source: string | Uint8Array | null, args: string[] = []) {
    if (this.started) throw new Error("Worklet has already been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    if (typeof filename !== "string") {
      throw new TypeError(
        `Filename must be a string. Received type ${typeof filename} (${filename})`,
      )
    }

    if (source !== null && typeof source !== "string" && !ArrayBuffer.isView(source)) {
      throw new TypeError(
        `Source must be a string or TypedArray. Received type ${typeof source} (${source})`,
      )
    }

    for (const arg of args) {
      if (typeof arg !== "string") {
        throw new TypeError(`Argument must be a string. Received type ${typeof arg} (${arg})`)
      }
    }

    let err: any = null
    try {
      if (typeof source === "string") {
        await NativeBareKit.startUTF8(this._handle, filename, source, args)
      } else if (source == null) {
        await NativeBareKit.startFile(this._handle, filename, args)
      } else {
        await NativeBareKit.startBytes(this._handle, filename, source, args)
      }

      this._state |= CONSTANTS.STARTED

      this.emit("start")

      BareKitWorklet._worklets.add(this)
    } catch (e) {
      err = e
    }

    this._ipc._continueOpen(err)

    if (err) throw err

    await this.resume()
  }

  async suspend(linger = -1) {
    if (!this.started) throw new Error("Worklet has not been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    if (typeof linger !== "number") {
      throw new TypeError(`Linger must be a number. Received type ${typeof linger} (${linger})`)
    }

    await NativeBareKit.suspend(this._handle, linger)

    this._state |= CONSTANTS.SUSPENDED

    this.emit("suspend")
  }

  static async suspend(linger?: number) {
    for (const worklet of this._worklets) {
      await worklet.suspend(linger)
    }
  }

  async resume() {
    if (!this.started) throw new Error("Worklet has not been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    await NativeBareKit.resume(this._handle)

    this._state &= ~CONSTANTS.SUSPENDED

    this.emit("resume")
  }

  static async resume() {
    for (const worklet of this._worklets) {
      await worklet.resume()
    }
  }

  async wakeup(deadline: number = 0) {
    if (!this.started) throw new Error("Worklet has not been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    if (typeof deadline !== "number") {
      throw new TypeError(
        `Deadline time must be a number. Received type ${typeof deadline} (${deadline})`,
      )
    }

    await NativeBareKit.wakeup(this._handle, deadline)

    this.emit("wakeup")
  }

  static async wakeup(deadline: number) {
    for (const worklet of this._worklets) {
      await worklet.wakeup(deadline)
    }
  }

  async terminate() {
    if (this.terminated) return

    this._ipc.destroy()

    if (this.started) await NativeBareKit.terminate(this._handle)

    this._state |= CONSTANTS.TERMINATED
    this._handle = -1

    BareKitWorklet._worklets.delete(this)

    this.emit("terminate")
  }

  toJSON() {
    return {
      started: this.started,
      terminated: this.terminated,
      suspended: this.suspended,
    }
  }
}

export const Worklet = BareKitWorklet
export type Worklet = BareKitWorklet
