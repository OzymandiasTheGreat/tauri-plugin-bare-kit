import React, { useCallback, useEffect, useState } from "react"
import { Button, StyleSheet, Text, TextInput, View } from "react-native"
import { StatusBar } from "expo-status-bar"
import b4a from "b4a"
import RPC from "bare-rpc"
import { ping, Worklet } from "tauri-plugin-bare-kit-api"
import bundleSource from "./bare/app.bundle.json"

const METHOD = {
  ENCRYPT: 0,
  DECRYPT: 1,
  ENCRYPT_STREAMING: 2,
  DECRYPT_STREAMING: 3,
}
const MAX_BYTES = 65_536
const MAX_PAYLOAD = 65_523
const CHUNK_SIZE = 1024 * 16

async function run(rpc: RPC) {
  for (let i = 1; i <= 24; i++) {
    const length = 2 ** i

    if (length <= MAX_BYTES) {
      const payload = b4a.alloc(length === MAX_BYTES ? MAX_PAYLOAD : length, i)
      const encrypt_request = rpc.request(METHOD.ENCRYPT)

      encrypt_request.send(payload)

      const encrypt_response = await encrypt_request.reply()

      console.log(
        `Encrypt result ${i} with length ${length}: ${!b4a.equals(payload, encrypt_response)}`,
      )

      const decrypt_request = rpc.request(METHOD.DECRYPT)

      decrypt_request.send(encrypt_response)

      const decrypt_response = await decrypt_request.reply()

      console.log(
        `Decrypt result ${i} with length ${length}: ${b4a.equals(payload, decrypt_response)}`,
      )
    } else {
      const payload: Uint8Array[] = []

      const decrypt_request = rpc.request(METHOD.DECRYPT_STREAMING)
      const decrypt_request_stream = decrypt_request.createRequestStream()
      const decrypt_response_stream = decrypt_request.createResponseStream()
      const decrypt_response: Uint8Array[] = []

      decrypt_request_stream.on("error", console.error)
      decrypt_response_stream.on("error", console.error)
      decrypt_response_stream.on("data", (data: any) => decrypt_response.push(data))
      decrypt_response_stream.on("end", () => {
        console.log(
          `Decrypt result ${i} with length ${length} streaming: ${b4a.equals(
            b4a.concat(payload),
            b4a.concat(decrypt_response),
          )}`,
        )
      })

      const encrypt_request = rpc.request(METHOD.ENCRYPT_STREAMING)
      const encrypt_request_stream = encrypt_request.createRequestStream()
      const encrypt_response_stream = encrypt_request.createResponseStream()
      const encrypt_response: Uint8Array[] = []

      encrypt_request_stream.on("error", console.error)
      encrypt_response_stream.on("error", console.error)
      encrypt_response_stream.on("data", (data: any) => {
        encrypt_response.push(data)
        decrypt_request_stream.write(data)
      })
      encrypt_response_stream.on("end", () => {
        decrypt_request_stream.end()
        console.log(
          `Encrypt result ${i} with length ${length} chunks ${
            encrypt_response.length
          } streaming: ${!b4a.equals(b4a.concat(payload), b4a.concat(encrypt_response))}`,
        )
      })

      for (let j = CHUNK_SIZE; j <= length; j += CHUNK_SIZE) {
        const chunk = b4a.alloc(CHUNK_SIZE, i)

        payload.push(chunk)
        encrypt_request_stream.write(chunk)
      }

      encrypt_request_stream.end()
    }
  }
}

export default function App() {
  const [request, setRequest] = useState("")
  const [response, setResponse] = useState("")

  const onPress = useCallback(() => {
    const payload = request.trim() ? request.trim() : "PING"
    ping(payload)
      .then((res) => setResponse(`[${new Date().toLocaleTimeString()}] ${res}`))
      .catch((err) => setResponse(`ERROR: ${err.message}`))
  }, [request])

  useEffect(() => {
    let worklet: Worklet | null = null

    Worklet.init().then(async (w) => {
      worklet = w
      await worklet.start("/app.bundle", bundleSource)

      const { IPC } = worklet
      // IPC.on("error", (err: Error) => console.error(err))
      // IPC.on("data", (data: string) => console.log("TAURI RECEIVED", data))
      // IPC.write("Hello from React Native!")
      const rpc = new RPC(IPC as any, (req) => {})

      await run(rpc)
    })

    return () => {
      worklet?.terminate()
    }
  })

  return (
    <View style={styles.container}>
      <TextInput onChangeText={setRequest} value={request} style={styles.input} />
      <Button title="PING" onPress={onPress} />
      <Text>{response}</Text>
      <StatusBar style="auto" />
    </View>
  )
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#fff",
    alignItems: "center",
    justifyContent: "center",
    gap: 16,
  },
  input: {
    width: 256,
    height: 24,
    borderColor: "#000",
    borderWidth: 1,
    borderRadius: 8,
  },
})
