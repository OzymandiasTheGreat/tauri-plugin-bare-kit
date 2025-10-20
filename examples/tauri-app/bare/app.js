const { IPC } = BareKit

IPC.on("error", (err) => console.error(err))
IPC.on("data", (data) => console.log("BARE RECEIVED", data))
IPC.write("Hello from Bare!")

// const b4a = require("b4a")
// const RPC = require("bare-rpc")
// const sodium = require("sodium-native")

// const METHOD = {
//   ENCRYPT: 0,
//   DECRYPT: 1,
//   ENCRYPT_STREAMING: 2,
//   DECRYPT_STREAMING: 3,
// }
// const KEY = b4a.alloc(sodium.crypto_stream_KEYBYTES)
// const NONCE = b4a.alloc(sodium.crypto_stream_NONCEBYTES)
// sodium.randombytes_buf(KEY)
// sodium.randombytes_buf(NONCE)

// const { IPC } = BareKit
// console.log("Hello, World, from BareKit!")
// const rpc = new RPC(IPC, (req) => {
//   if (req.command === METHOD.ENCRYPT || req.command === METHOD.DECRYPT) {
//     console.log(`BARE DATA ${req.data}`)
//     const response = b4a.alloc(req.data.byteLength)
//     sodium.crypto_stream_xor(response, req.data, NONCE, KEY)
//     req.reply(response)
//   } else if (
//     req.command === METHOD.ENCRYPT_STREAMING ||
//     req.command === METHOD.DECRYPT_STREAMING
//   ) {
//     const request_stream = req.createRequestStream()
//     const response_stream = req.createResponseStream()

//     request_stream.on("error", console.error)
//     response_stream.on("error", console.error)
//     request_stream.on("data", (data) => {
//       const response = b4a.alloc(data.byteLength)
//       sodium.crypto_stream_xor(response, data, NONCE, KEY)
//       response_stream.write(response)
//     })
//     request_stream.on("end", () => response_stream.end())
//   }
// })
