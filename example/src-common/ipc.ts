import c, { type Encoder } from "compact-encoding"

export enum Method {
  BeamReady = 0,
  BeamMessage,
  BeamFile,
  BeamSave,
  BenchEncrypt = 0xff01,
  BenchDecrypt,
  BenchEncryptStreaming,
  BenchDecryptStreaming,
}

enum Flags {
  hasFD = 0x01,
  hasName = 0x02,
}

export type ID = Uint8Array
export const ID = c.fixed32

export type Message = { other: Uint8Array; text: string; color: number }
export const Message: Encoder<Message> = {
  preencode(state, value) {
    c.fixed32.preencode(state, value.other)
    c.string.preencode(state, value.text)
    c.uint8.preencode(state, value.color)
  },
  encode(state, value) {
    c.fixed32.encode(state, value.other)
    c.string.encode(state, value.text)
    c.uint8.encode(state, value.color)
  },
  decode(state) {
    const other = c.fixed32.decode(state)
    const text = c.string.decode(state)
    const color = c.uint8.decode(state)

    return { other, text, color }
  },
}

export type File = {
  other: Uint8Array
  path: string
  fd: number | null
  name: string | null
  color: number
}
export const File: Encoder<File> = {
  preencode(state, value) {
    let flags = 0

    if (value.fd) {
      flags |= Flags.hasFD
    }

    if (value.name) {
      flags |= Flags.hasName
    }

    c.uint8.preencode(state, flags)
    c.fixed32.preencode(state, value.other)
    c.string.preencode(state, value.path)
    value.fd && c.int32.preencode(state, value.fd)
    value.name && c.string.preencode(state, value.name)
    c.uint8.preencode(state, value.color)
  },
  encode(state, value) {
    let flags = 0

    if (value.fd) {
      flags |= Flags.hasFD
    }

    if (value.name) {
      flags |= Flags.hasName
    }

    c.uint8.encode(state, flags)
    c.fixed32.encode(state, value.other)
    c.string.encode(state, value.path)
    value.fd && c.int32.encode(state, value.fd)
    value.name && c.string.encode(state, value.name)
    c.uint8.encode(state, value.color)
  },
  decode(state) {
    const flags = c.uint8.decode(state)
    const other = c.fixed32.decode(state)
    const path = c.string.decode(state)
    const fd = (flags & Flags.hasFD) !== 0 ? c.int32.decode(state) : null
    const name = (flags & Flags.hasName) !== 0 ? c.string.decode(state) : null
    const color = c.uint8.decode(state)

    return { other, path, fd, name, color }
  },
}

export type SaveRequest = {
  other: Uint8Array
  filename: string
  filepath: string
  fd: number | null
}
export const SaveRequest: Encoder<SaveRequest> = {
  preencode(state, value) {
    let flags = 0

    if (value.fd) {
      flags |= Flags.hasFD
    }

    c.uint8.preencode(state, flags)
    c.fixed32.preencode(state, value.other)
    c.string.preencode(state, value.filename)
    c.string.preencode(state, value.filepath)
    value.fd && c.int32.preencode(state, value.fd)
  },
  encode(state, value) {
    let flags = 0

    if (value.fd) {
      flags |= Flags.hasFD
    }

    c.uint8.encode(state, flags)
    c.fixed32.encode(state, value.other)
    c.string.encode(state, value.filename)
    c.string.encode(state, value.filepath)
    value.fd && c.int32.encode(state, value.fd)
  },
  decode(state) {
    const flags = c.uint8.decode(state)
    const other = c.fixed32.decode(state)
    const filename = c.string.decode(state)
    const filepath = c.string.decode(state)
    const fd = (flags & Flags.hasFD) !== 0 ? c.int32.decode(state) : null

    return { other, filename, filepath, fd }
  },
}
