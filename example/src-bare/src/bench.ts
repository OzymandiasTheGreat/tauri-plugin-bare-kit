import b4a from "b4a"
import type { IncomingRequest } from "bare-rpc"
import sodium from "sodium-native"

const KEY = b4a.allocUnsafe(sodium.crypto_stream_KEYBYTES)
const NONCE = b4a.allocUnsafe(sodium.crypto_stream_NONCEBYTES)
sodium.randombytes_buf(KEY)
sodium.randombytes_buf(NONCE)

export async function bench(req: IncomingRequest, data: Uint8Array): Promise<Uint8Array> {
  const response = b4a.allocUnsafe(data.byteLength)
  sodium.crypto_stream_xor(response, data, NONCE, KEY)
  return response
}

export async function benchStreaming(req: IncomingRequest, data: unknown): Promise<unknown> {
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
  await new Promise<void>((resolve) => responseStream.on("close", resolve))
  return null
}
