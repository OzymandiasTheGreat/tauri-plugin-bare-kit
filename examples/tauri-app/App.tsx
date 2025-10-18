import React, { useCallback, useState } from "react"
import { Button, StyleSheet, Text, TextInput, View } from "react-native"
import { StatusBar } from "expo-status-bar"
import { ping } from "tauri-plugin-bare-kit-api"

export default function App() {
  const [request, setRequest] = useState("")
  const [response, setResponse] = useState("")

  const onPress = useCallback(() => {
    const payload = request.trim() ? request.trim() : "PING"
    ping(payload)
      .then((res) => setResponse(`[${new Date().toLocaleTimeString()}] ${res}`))
      .catch((err) => setResponse(`ERROR: ${err.message}`))
  }, [request])

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
