import b4a from "b4a"

import type { IPC } from "@/hooks/useBareKit"
import { Method } from "@/common/ipc"

const MAX_PACKET = 65_536
const MAX_PAYLOAD = 65_523
const CHUNK_SIZE = 16 * 1024

let running = false

export default async function bench(ipc: IPC) {
  if (running) return

  try {
    running = true
    console.time("benchmark")

    for (let i = 1; i <= 24; i++) {
      const length = 2 ** i

      if (length <= MAX_PACKET) {
        const payload = b4a.alloc(length === MAX_PACKET ? MAX_PAYLOAD : length, i)
        const encryptRequest = ipc.request(Method.BenchEncrypt)

        encryptRequest.send(payload)

        const encryptResponse = await encryptRequest.reply()
        const decryptRequest = ipc.request(Method.BenchDecrypt)

        decryptRequest.send(encryptResponse)

        const decryptResponse = await decryptRequest.reply()
        console.assert(
          b4a.equals(payload, decryptResponse),
          `Encrypt/decrypt failed for iteration ${i}`,
        )
      } else {
        const payload: Uint8Array[] = []
        const decryptRequest = ipc.request(Method.BenchDecryptStreaming)
        const decryptRequestStream = decryptRequest.createRequestStream()
        const decryptResponseStream = decryptRequest.createResponseStream()
        const decryptResponse: Uint8Array[] = []

        decryptRequestStream.on("error", console.error)
        decryptResponseStream.on("error", console.error)
        decryptResponseStream.on("data", (data: Uint8Array) => decryptResponse.push(data))

        const promise = new Promise<void>((resolve) =>
          decryptResponseStream.on("end", () => {
            console.assert(
              b4a.equals(b4a.concat(payload), b4a.concat(decryptResponse)),
              `Encrypt/decrypt streaming failed for iteration ${i}`,
            )
            resolve()
          }),
        )
        const encryptRequest = ipc.request(Method.BenchEncryptStreaming)
        const encryptRequestStream = encryptRequest.createRequestStream()
        const encryptResponseStream = encryptRequest.createResponseStream()
        const encryptResponse: Uint8Array[] = []

        encryptRequestStream.on("error", console.error)
        encryptResponseStream.on("error", console.error)
        encryptResponseStream.on("data", (data: Uint8Array) => {
          encryptResponse.push(data)
          decryptRequestStream.write(data)
        })
        encryptResponseStream.on("end", () => decryptRequestStream.end())

        for (let j = CHUNK_SIZE; j <= length; j += CHUNK_SIZE) {
          const chunk = b4a.alloc(CHUNK_SIZE, i)

          payload.push(chunk)
          encryptRequestStream.write(chunk)
        }

        encryptRequestStream.end()
        await promise
      }
    }
  } finally {
    console.timeEnd("benchmark")
    running = false
  }
}
