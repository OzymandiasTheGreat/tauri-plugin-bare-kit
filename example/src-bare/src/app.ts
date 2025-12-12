import b4a from "b4a"
import RPC from "bare-rpc"
import sodium from "sodium-native"

import { Method } from "./common/ipc"

const { IPC } = BareKit

const KEY = b4a.allocUnsafe(sodium.crypto_stream_KEYBYTES)
const NONCE = b4a.allocUnsafe(sodium.crypto_stream_NONCEBYTES)
sodium.randombytes_buf(KEY)
sodium.randombytes_buf(NONCE)

const rpc = new RPC(IPC, (req) => {
  if (req.command === Method.BenchEncrypt || req.command === Method.BenchDecrypt) {
    const response = b4a.allocUnsafe(req.data.byteLength)
    sodium.crypto_stream_xor(response, req.data, NONCE, KEY)
    req.reply(response)
  } else if (
    req.command === Method.BenchEncryptStreaming ||
    req.command === Method.BenchDecryptStreaming
  ) {
    const requestStream = req.createRequestStream()
    const responseStream = req.createResponseStream()

    requestStream.on("error", console.error)
    responseStream.on("error", console.error)
    requestStream.on("data", (data: Uint8Array) => {
      const response = b4a.allocUnsafe(data.byteLength)
      sodium.crypto_stream_xor(response, data, NONCE, KEY)
      responseStream.write(response)
    })
    requestStream.on("end", () => responseStream.end())
  }
})
