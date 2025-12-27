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

export type File = { other: Uint8Array; path: string; color: number }
export const File: Encoder<File> = {
  preencode(state, value) {
    c.fixed32.preencode(state, value.other)
    c.string.preencode(state, value.path)
    c.uint8.preencode(state, value.color)
  },
  encode(state, value) {
    c.fixed32.encode(state, value.other)
    c.string.encode(state, value.path)
    c.uint8.encode(state, value.color)
  },
  decode(state) {
    const other = c.fixed32.decode(state)
    const path = c.string.decode(state)
    const color = c.uint8.decode(state)

    return { other, path, color }
  },
}

export type SaveRequest = { other: Uint8Array; filename: string; filepath: string }
export const SaveRequest: Encoder<SaveRequest> = {
  preencode(state, value) {
    c.fixed32.preencode(state, value.other)
    c.string.preencode(state, value.filename)
    c.string.preencode(state, value.filepath)
  },
  encode(state, value) {
    c.fixed32.encode(state, value.other)
    c.string.encode(state, value.filename)
    c.string.encode(state, value.filepath)
  },
  decode(state) {
    const other = c.fixed32.decode(state)
    const filename = c.string.decode(state)
    const filepath = c.string.decode(state)

    return { other, filename, filepath }
  },
}
