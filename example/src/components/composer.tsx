import MDI from "@react-native-vector-icons/material-design-icons"
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager"
import { open } from "@tauri-apps/plugin-dialog"
import { useCallback, useState } from "react"
import { ActivityIndicator, Button, Pressable, Text, TextInput, View } from "react-native"
import { getFileDescriptor, getFileName } from "tauri-plugin-mobile-fs-api"

import useStore from "@/hooks/useStore"
import createThemedStyleSheet from "@/hooks/useTheme"

export default function Composer() {
  const styles = useStyles()
  const store = useStore()

  const [recipient, setRecipient] = useState("")
  const [message, setMessage] = useState("")

  const copyId = useCallback(() => {
    writeText(store.id!)
  }, [store.id])
  const pasteRecipient = useCallback(() => {
    readText().then(setRecipient)
  }, [])
  const sendMessage = useCallback(() => {
    store.sendMessage(recipient, message)
    setMessage("")
  }, [recipient, message])
  const sendFile = useCallback(() => {
    open({ directory: false, multiple: false }).then(async (path) => {
      if (path) {
        const fd = await getFileDescriptor(path, "r")
        const name = await getFileName(path)

        store.sendFile(recipient, path, fd, name)
      }
    })
  }, [recipient])

  if (!store.id) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size={256} />
      </View>
    )
  }

  return (
    <View style={styles.container}>
      {store.lastError && (
        <Pressable style={styles.errorWrapper} onPress={store.clearError}>
          {({ pressed }) => (
            <View style={[styles.errorBubble, { opacity: pressed ? 0.5 : 1 }]}>
              <MDI name="alert-circle-outline" size={48} color={styles.errorIcon.color} />
              <View style={styles.errorContent}>
                <Text style={styles.errorMessage}>{store.lastError?.message}</Text>
                {store.lastError?.reason && (
                  <Text style={styles.errorReason}>{store.lastError?.reason}</Text>
                )}
              </View>
            </View>
          )}
        </Pressable>
      )}
      <View style={styles.inputWrapper}>
        <TextInput
          style={[styles.input, styles.id]}
          value={store.id}
          readOnly
          selectTextOnFocus
        />
        <Pressable style={styles.iconWrapper} onPress={copyId}>
          {({ pressed }) => (
            <MDI
              name="content-copy"
              size={24}
              color={styles.icon.color}
              style={{ opacity: pressed ? 0.5 : 1 }}
            />
          )}
        </Pressable>
      </View>

      <View style={styles.inputWrapper}>
        <TextInput
          style={[styles.input]}
          value={recipient}
          onChangeText={setRecipient}
          selectTextOnFocus
        />
        <Pressable style={styles.iconWrapper} onPress={pasteRecipient}>
          {({ pressed }) => (
            <MDI
              name="content-paste"
              size={24}
              color={styles.icon.color}
              style={{ opacity: pressed ? 0.5 : 1 }}
            />
          )}
        </Pressable>
      </View>

      <View style={styles.inputWrapper}>
        <TextInput
          style={[styles.input]}
          multiline
          value={message}
          onChangeText={setMessage}
          numberOfLines={5}
        />
      </View>

      <View style={styles.toolbar}>
        <Button title="Send File" onPress={sendFile} />
        <Button title="Send" onPress={sendMessage} />
      </View>
    </View>
  )
}

const useStyles = createThemedStyleSheet((theme) => ({
  container: {
    flex: 1,
    backgroundColor: theme.background,
    alignItems: "center",
    justifyContent: "flex-end",
    gap: 16,
    paddingBottom: 16,
  },
  errorBubble: {
    backgroundColor: theme.error,
    borderRadius: 8,
    flexDirection: "row",
    alignItems: "center",
    padding: 8,
  },
  errorContent: {
    gap: 8,
    padding: 8,
  },
  errorIcon: {
    color: theme.text,
  },
  errorMessage: {
    color: theme.text,
    fontSize: 14,
    width: "40%",
  },
  errorReason: {
    color: theme.text,
    fontSize: 12,
    opacity: 0.6,
    width: "40%",
  },
  errorWrapper: {
    position: "absolute",
    top: 8,
    left: "10%",
    width: "80%",
  },
  icon: {
    color: theme.text,
  },
  iconWrapper: {
    position: "absolute",
    bottom: 4,
    right: 8,
  },
  input: {
    color: theme.text,
    backgroundColor: theme.input,
    width: "100%",
    outlineStyle: "none" as any,
  },
  inputWrapper: {
    flexDirection: "row",
    backgroundColor: theme.input,
    width: "80%",
    padding: 8,
    borderRadius: 8,
  },
  id: {
    opacity: 0.7,
  },
  toolbar: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "flex-end",
    gap: 8,
    width: "80%",
  },
}))
