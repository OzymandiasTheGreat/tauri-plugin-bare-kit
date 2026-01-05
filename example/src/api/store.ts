import { basename } from "@tauri-apps/api/path"
import { type DeepSignal, deepSignal, peek } from "deepsignal/react"
import z32 from "z32"

import type { IPC } from "@/common/bare-rpc"
import { ID, File, Message as MessageEncoding, Method, SaveRequest } from "@/common/ipc"
import { Bubbles } from "@/constants/colors"

export interface BeamError {
  message: string
  reason?: string
}

export interface Message {
  sender: string
  recipient: string
  isFile: boolean
  isLocal: boolean
  text: string
  color: string
}

export interface Store {
  id: string | null
  messages: Message[]
  lastError: BeamError | null

  clearError(): void
  sendMessage(recipient: string, text: string): void
  sendFile(recipient: string, path: string, fd: number | null, name: string | null): void
  saveFile(sender: string, filename: string, filepath: string, fd: number | null): void
}

let ipc: IPC | null = null

export const store: DeepSignal<Store> = deepSignal({
  id: null,
  lastError: null,
  messages: [],

  clearError() {
    store.lastError = null
  },

  sendMessage(recipient, text) {
    const req = ipc?.request(Method.BeamMessage)
    req?.send(
      {
        other: z32.decode(recipient),
        text,
        color: 0,
      } as any,
      MessageEncoding as any,
    )
    req
      ?.reply()
      .then(() =>
        store.messages.push({
          sender: peek(store, "id")!,
          recipient,
          isFile: false,
          isLocal: true,
          text,
          color: "",
        }),
      )
      .catch((err) => {
        console.error(err)
        store.lastError = {
          message: `Sending message to ${recipient} failed`,
          reason: err.message,
        }
      })
  },

  sendFile(recipient, path, fd, name) {
    const req = ipc?.request(Method.BeamFile)
    req?.send(
      {
        other: z32.decode(recipient),
        path,
        fd,
        name,
        color: 0,
      } as any,
      File as any,
    )
    req
      ?.reply()
      .then(async () =>
        store.messages.push({
          sender: peek(store, "id")!,
          recipient,
          isFile: true,
          isLocal: true,
          text: await basename(path),
          color: "",
        }),
      )
      .catch((err) => {
        console.error(err)
        store.lastError = {
          message: `Sending file to ${recipient} failed`,
          reason: err.message,
        }
      })
  },

  saveFile(sender, filename, filepath, fd) {
    const req = ipc?.request(Method.BeamSave)
    req?.send(
      {
        other: z32.decode(sender),
        filename,
        filepath,
        fd,
      } as any,
      SaveRequest as any,
    )
    req?.reply()
  },
})

export function setIPC(_ipc: IPC | null) {
  ipc = _ipc

  ipc?.respond(
    Method.BeamReady,
    { requestEncoding: ID, responseEncoding: null },
    (req, data) => {
      const id = z32.encode(data)
      store.id = id
    },
  )

  ipc?.respond(
    Method.BeamMessage,
    { requestEncoding: MessageEncoding, responseEncoding: null },
    (req, data: MessageEncoding) => {
      store.messages.push({
        sender: z32.encode(data.other),
        recipient: peek(store, "id")!,
        isFile: false,
        isLocal: false,
        text: data.text,
        color: Bubbles[data.color],
      })
    },
  )

  ipc?.respond(
    Method.BeamFile,
    { requestEncoding: File, responseEncoding: null },
    async (req, data: File) => {
      store.messages.push({
        sender: z32.encode(data.other),
        recipient: peek(store, "id")!,
        isFile: true,
        isLocal: false,
        text: await basename(data.path),
        color: Bubbles[data.color],
      })
    },
  )
}
