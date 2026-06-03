const b4a = require("b4a")
const { Duplex } = require("streamx")
const EventEmitter = require("bare-events")
const NativeBareKit = require("./module")

const constants = {
  STARTED: 0x1,
  TERMINATED: 0x2,
  SUSPENDED: 0x4,
}

class BareKitIPC extends Duplex {
  constructor(worklet) {
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

  _open(cb) {
    if (this._worklet.started) cb(null)
    else this._pendingOpen = cb
  }

  async _update() {
    await NativeBareKit.update(
      this._worklet._handle,
      this._pendingRead !== null,
      this._pendingWrite !== null,
    )
  }

  async _poll({ readable, writable }) {
    if (this._worklet.terminated) return
    if (readable) await this._continueRead()
    if (writable) await this._continueWrite()

    await NativeBareKit.notify(this._worklet._handle)
  }

  async _read(cb) {
    const data = await NativeBareKit.read(this._worklet._handle)

    if (data) {
      this.push(data)
      cb(null)
    } else {
      this._pendingRead = cb
      await this._update()
    }
  }

  async _write(data, cb) {
    const written = await NativeBareKit.write(this._worklet._handle, data)

    if (data == null || written === data.byteLength) cb(null)
    else {
      this._pendingWrite = [data.subarray(written), cb]
      await this._update()
    }
  }

  _continueOpen(err) {
    if (this._pendingOpen === null) {
      if (err) this.destroy(err)
    } else {
      const cb = this._pendingOpen
      this._pendingOpen = null
      cb(err)
    }
  }

  async _continueRead() {
    if (this._pendingRead === null) return
    const cb = this._pendingRead
    this._pendingRead = null
    await this._update()
    await this._read(cb)
  }

  async _continueWrite() {
    if (this._pendingWrite === null) return
    const [data, cb] = this._pendingWrite
    this._pendingWrite = null
    await this._update()
    await this._write(data, cb)
  }
}

class BareKitWorklet extends EventEmitter {
  static _worklets = new Set()

  constructor() {
    super()

    this._state = 0
    this._handle = null
    this._ipc = new BareKitIPC(this)
  }

  static async optimizeForMemory(enabled) {
    await NativeBareKit.optimizeForMemory(enabled)
  }

  static async init(options = {}) {
    const { memoryLimit = 0, assets = null } = options

    if (typeof memoryLimit !== "number") {
      throw new TypeError(
        "Memory limit must be a number. Received type " +
          typeof memoryLimit +
          " (" +
          memoryLimit +
          ")",
      )
    }

    if (typeof assets !== "string" && assets !== null) {
      throw new TypeError(
        "Asset path must be a string. Received type " + typeof assets + " (" + assets + ")",
      )
    }

    const worklet = new BareKitWorklet()

    worklet._handle = await NativeBareKit.newWorklet(
      memoryLimit,
      assets,
      worklet._ipc._poll,
      worklet._onsuspend.bind(worklet),
      worklet._onwakeup.bind(worklet),
      worklet._onidle.bind(worklet),
      worklet._onresume.bind(worklet),
    )

    return worklet
  }

  get IPC() {
    return this._ipc
  }

  get started() {
    return (this._state & constants.STARTED) !== 0
  }

  get terminated() {
    return (this._state & constants.TERMINATED) !== 0
  }

  get suspended() {
    return (this._state & constants.SUSPENDED) !== 0
  }

  async start(filename, source, args = []) {
    if (this.started) throw new Error("Worklet has already been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    if (typeof filename !== "string") {
      throw new TypeError(
        "Filename must be a string. Received type " + typeof filename + " (" + filename + ")",
      )
    }

    if (Array.isArray(source)) {
      args = source
      source = null
    }

    if (source !== null && typeof source !== "string" && !ArrayBuffer.isView(source)) {
      throw new TypeError(
        "Source must be a string or TypedArray. Received type " +
          typeof source +
          " (" +
          source +
          ")",
      )
    }

    for (const arg of args) {
      if (typeof arg !== "string") {
        throw new TypeError(
          "Argument must be a string. Received type " + typeof arg + " (" + arg + ")",
        )
      }
    }

    let err = null
    try {
      if (source === null) {
        await NativeBareKit.startFile(this._handle, filename, args)
      } else if (typeof source === "string") {
        await NativeBareKit.startUTF8(this._handle, filename, source, args)
      } else {
        await NativeBareKit.startBytes(this._handle, filename, source, args)
      }

      this._state |= constants.STARTED

      BareKitWorklet._worklets.add(this)
    } catch (e) {
      err = e
    }

    this._ipc._continueOpen(err)

    if (err) throw err

    await this.update()
  }

  async suspend(linger = -1) {
    if (!this.started) throw new Error("Worklet has not been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    if (typeof linger !== "number") {
      throw new TypeError(
        "Linger time must be a number. Received type " + typeof linger + " (" + linger + ")",
      )
    }

    await NativeBareKit.suspend(this._handle, linger)

    this._state |= constants.SUSPENDED
  }

  static async suspend(linger) {
    for (const worklet of this._worklets) {
      await worklet.suspend(linger)
    }
  }

  async resume() {
    if (!this.started) throw new Error("Worklet has not been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    await NativeBareKit.resume(this._handle)

    this._state &= ~constants.SUSPENDED
  }

  async wakeup(deadline = 0) {
    if (!this.started) throw new Error("Worklet has not been started")
    if (this.terminated) throw new Error("Worklet has been terminated")

    if (typeof deadline !== "number") {
      throw new TypeError(
        "Deadline time must be a number. Received type " +
          typeof deadline +
          " (" +
          deadline +
          ")",
      )
    }

    await NativeBareKit.wakeup(this._handle, deadline)
  }

  static async wakeup(deadline) {
    for (const worklet of this._worklets) {
      await worklet.wakeup(deadline)
    }
  }

  static async resume() {
    for (const worklet of this._worklets) {
      await worklet.resume()
    }
  }

  // TODO: tauri lifecycle events
  async update(state) {
    // switch (state) {
    //   case "active":
    return this.resume()
    //   case "background":
    //     return this.suspend()
    // }
  }

  static async update(state) {
    for (const worklet of this._worklets) {
      await worklet.update(state)
    }
  }

  async terminate() {
    if (this.terminated) return

    this._ipc.destroy()

    if (this.started) await NativeBareKit.terminate(this._handle)

    this._state |= constants.TERMINATED
    this._handle = null

    BareKitWorklet._worklets.delete(this)
  }

  toJSON() {
    return {
      started: this.started,
      terminated: this.terminated,
      suspended: this.suspended,
    }
  }

  _onsuspend(linger) {
    this.emit("suspend", linger)
  }

  _onwakeup(deadline) {
    this.emit("wakeup", deadline)
  }

  _onidle() {
    this.emit("idle")
  }

  _onresume() {
    this.emit("resume")
  }
}

module.exports.Worklet = BareKitWorklet
