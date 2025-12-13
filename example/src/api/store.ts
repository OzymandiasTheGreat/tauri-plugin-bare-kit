import { signal, type Signal } from "@preact/signals-react"

import Deferred from "@/api/deferred"
import type { IPC, Worklet } from "@/hooks/useBareKit"

export class Store {
  private static _instance: Store | null = null
  private static _ready = new Deferred()

  private _worklet: Worklet
  private _ipc: IPC

  private _messages: Signal<Record<string, Signal<string[]>>>
  private _peers: Signal<string[]>

  constructor(worklet: Worklet, ipc: IPC) {
    if (Store._instance) {
      return Store._instance
    }

    this._worklet = worklet
    this._ipc = ipc

    this._messages = signal({})
    this._peers = signal([])

    Store._instance = this
    Store._ready.resolve()
  }

  static get ready(): Promise<void> {
    return this._ready.promise
  }

  static get instance(): Store {
    if (!this._instance) {
      throw new Error("Store hasn't been initialized yet")
    }

    return this._instance
  }

  get messages(): Signal<Record<string, Signal<string[]>>> {
    return this._messages
  }

  get peers(): Signal<string[]> {
    return this._peers
  }

  addPeer(peer: string) {
    if (!this._messages.peek()[peer]) {
      this._messages.value = { ...this._messages.value, [peer]: signal([]) }
    }

    this._peers.value = [peer, ...this._peers.value.filter((p) => p !== peer)]
  }

  addMessage(peer: string, message: string) {
    if (this._messages.value[peer]) {
      this._messages.value[peer].value = [...this._messages.value[peer].value, message]
    } else {
      this._messages.value = { ...this._messages.value, [peer]: signal([message]) }
    }

    this._peers.value = [peer, ...this._peers.value.filter((p) => p !== peer)]
  }

  getMessages(peer: string): Signal<string[]> {
    return this._messages.peek()[peer]
  }
}
