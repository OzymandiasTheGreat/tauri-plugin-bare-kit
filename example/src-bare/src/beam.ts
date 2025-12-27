import RPC from "@hyperswarm/rpc"
import Buffer from "bare-buffer"
import fs from "bare-fs"
import fsp from "bare-fs/promises"
import os from "bare-os"
import path from "bare-path"
import type { Writable } from "bare-stream"
import c, { Encoder } from "compact-encoding"
import ReadyResource from "ready-resource"
import sodium from "sodium-native"
import z32 from "z32"

import { IPC } from "./common/bare-rpc"
import { File, ID, Message, Method as IPCMethod, SaveRequest } from "./common/ipc"

enum RPCMethod {
  text = "text",
  fileOpen = "file-open",
  fileChunk = "file-chunk",
  fileClose = "file-close",
}

type FileRequest = { name: string; size: number }
const FileRequest: Encoder<FileRequest> = {
  preencode(state, value) {
    c.string.preencode(state, value.name)
    c.uint32.preencode(state, value.size)
  },
  encode(state, value) {
    c.string.encode(state, value.name)
    c.uint32.encode(state, value.size)
  },
  decode(state) {
    const name = c.string.decode(state)
    const size = c.uint32.decode(state)

    return { name, size }
  },
}

type FileChunk = { name: string; data: Uint8Array }
const FileChunk: Encoder<FileChunk> = {
  preencode(state, value) {
    c.string.preencode(state, value.name)
    c.buffer.preencode(state, value.data)
  },
  encode(state, value) {
    c.string.encode(state, value.name)
    c.buffer.encode(state, value.data)
  },
  decode(state) {
    const name = c.string.decode(state)
    const data = c.buffer.decode(state)

    return { name, data }
  },
}

export default class Beam extends ReadyResource {
  ipc: IPC
  rpc: RPC
  server: any
  tmp: string
  incoming: Map<string, Writable> = new Map()

  constructor(ipc: IPC) {
    super()

    this.ipc = ipc
    this.rpc = new RPC()
    this.server = this.rpc.createServer()
    this.tmp = path.join(os.tmpdir(), `bare-beam-${randString()}`)

    this.ipc.respond(
      IPCMethod.BeamMessage,
      { requestEncoding: Message, responseEncoding: null },
      this.onsendmessage.bind(this),
    )
    this.ipc.respond(
      IPCMethod.BeamFile,
      { requestEncoding: File, responseEncoding: null },
      this.onsendfile.bind(this),
    )
    this.ipc.respond(
      IPCMethod.BeamSave,
      { requestEncoding: SaveRequest, responseEncoding: null },
      this.onsavefile.bind(this),
    )

    this.server.respond(
      RPCMethod.text,
      { requestEncoding: c.string },
      this.onrecvmessage.bind(this),
    )
    this.server.respond(
      RPCMethod.fileOpen,
      { requestEncoding: FileRequest },
      this.onrecvfileopen.bind(this),
    )
    this.server.respond(
      RPCMethod.fileChunk,
      { requestEncoding: FileChunk },
      this.onrecvfilechunk.bind(this),
    )
    this.server.respond(
      RPCMethod.fileClose,
      { requestEncoding: c.string },
      this.onrecvfileclose.bind(this),
    )

    this.ready().then(async () => {
      const req = this.ipc.request(IPCMethod.BeamReady)
      req.send(this.publicKey!, ID as any)
      await req.reply()
    })
  }

  get publicKey(): Buffer | null {
    if (!this.opened || this.closed) return null

    return this.server.address().publicKey
  }

  protected async _open(): Promise<void> {
    await fsp.mkdir(this.tmp, { recursive: true })
    await this.server.listen()
  }

  protected async _close(): Promise<void> {
    await this.server.close()
    await this.rpc.destroy()

    for (const stream of this.incoming.values()) {
      stream.destroy()
    }

    await fsp.rm(this.tmp, { force: true, recursive: true })
  }

  async onsendmessage(req: IPC.IncomingRequest, data: Message) {
    await this.rpc.request(data.other, RPCMethod.text, data.text, {
      requestEncoding: c.string,
    })
  }

  async onrecvmessage(text: string, rpc: any) {
    const other = rpc.stream.remotePublicKey
    const color = colorIndex(other)
    const req = this.ipc.request(IPCMethod.BeamMessage)
    req.send({ other, text, color } as any, Message as any)
    await req.reply()
  }

  async onsendfile(req: IPC.IncomingRequest, data: File) {
    const stat = await fsp.stat(data.path)
    const name = path.basename(data.path)

    await this.rpc.request(
      data.other,
      RPCMethod.fileOpen,
      {
        name,
        size: stat.size,
      },
      {
        requestEncoding: FileRequest,
      },
    )

    for await (const chunk of fs.createReadStream(data.path)) {
      await this.rpc.request(
        data.other,
        RPCMethod.fileChunk,
        {
          name,
          data: chunk,
        },
        {
          requestEncoding: FileChunk,
        },
      )
    }

    await this.rpc.request(data.other, RPCMethod.fileClose, name, { requestEncoding: c.string })
  }

  async onrecvfileopen(file: FileRequest, rpc: any) {
    const other = rpc.stream.remotePublicKey
    const key = fileKey(other, file.name)
    const filepath = path.join(this.tmp, z32.encode(other), file.name)
    await fsp.mkdir(path.dirname(filepath), { recursive: true })
    const stream = fs.createWriteStream(filepath)
    this.incoming.set(key, stream)
  }

  async onrecvfilechunk(file: FileChunk, rpc: any) {
    const other = rpc.stream.remotePublicKey
    const key = fileKey(other, file.name)
    const stream = this.incoming.get(key)
    stream?.write(file.data)
  }

  async onrecvfileclose(filename: string, rpc: any) {
    const other = rpc.stream.remotePublicKey
    const key = fileKey(other, filename)
    const filepath = path.join(this.tmp, z32.encode(other), filename)
    const color = colorIndex(other)
    const stream = this.incoming.get(key)

    this.incoming.delete(key)
    stream?.end()

    const req = this.ipc.request(IPCMethod.BeamFile)
    req.send({ path: filepath, other, color } as any, File as any)
    await req.reply()
  }

  async onsavefile(req: IPC.IncomingRequest, data: SaveRequest) {
    const source = path.join(this.tmp, z32.encode(data.other), data.filename)

    await fsp.copyFile(source, data.filepath)
  }
}

function randString() {
  const out = Buffer.allocUnsafe(4)
  sodium.randombytes_buf(out)
  const num = out.readUint32LE()
  return num.toString(16)
}

function colorIndex(seed: Buffer): number {
  const out = Buffer.allocUnsafe(1)
  sodium.randombytes_buf_deterministic(out, seed)
  const num = out.readUint8()
  return Math.floor(num / 32)
}

function fileKey(other: Uint8Array, filename: string): string {
  return `${z32.encode(other)}::${filename}`
}
