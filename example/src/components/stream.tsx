import MDI from "@expo/vector-icons/MaterialCommunityIcons"
import { save } from "@tauri-apps/plugin-dialog"
import { useCallback } from "react"
import { FlatList, ListRenderItem, Pressable, View, Text } from "react-native"

import { type Message } from "@/api/store"
import useStore from "@/hooks/useStore"
import createThemedStyleSheet from "@/hooks/useTheme"

export default function Stream() {
  const styles = useStyles()
  const store = useStore()

  const saveFile = useCallback(
    (other: string, filename: string) =>
      save({ defaultPath: filename }).then((filepath) => {
        if (filepath) {
          store.saveFile(other, filename, filepath)
        }
      }),
    [],
  )

  const keyExtractor = useCallback((item, index) => `#${index}`, [])
  const renderItem = useCallback<ListRenderItem<Message>>(
    ({ item }) => {
      return (
        <View
          style={[
            styles.bubbleWrapper,
            { justifyContent: item.isLocal ? "flex-end" : "flex-start" },
          ]}
        >
          <View style={[styles.bubble, !item.isLocal && { backgroundColor: item.color }]}>
            <Text style={styles.id} numberOfLines={1} ellipsizeMode="tail">
              {item.isLocal ? item.recipient : item.sender}
            </Text>
            {item.isFile ? (
              <Pressable
                pointerEvents={item.isLocal ? "none" : "auto"}
                onPress={() => saveFile(item.sender, item.text)}
              >
                {({ pressed }) => (
                  <View style={[styles.fileBox, { opacity: pressed ? 0.5 : 1 }]}>
                    <MDI name="file" size={24} color={styles.fileIcon.color} />
                    <Text style={styles.fileText} numberOfLines={1} ellipsizeMode="middle">
                      {item.text}
                    </Text>
                  </View>
                )}
              </Pressable>
            ) : (
              <Text style={styles.text}>{item.text}</Text>
            )}
          </View>
        </View>
      )
    },
    [saveFile],
  )

  return (
    <View style={styles.container}>
      <FlatList
        data={store.messages}
        extraData={store.messages.$length?.value}
        keyExtractor={keyExtractor}
        renderItem={renderItem}
      />
    </View>
  )
}

const useStyles = createThemedStyleSheet((theme) => ({
  bubble: {
    backgroundColor: theme.self,
    gap: 8,
    padding: 8,
    borderRadius: 8,
    maxWidth: "65%",
  },
  bubbleWrapper: {
    flexDirection: "row",
    padding: 8,
  },
  container: {
    flex: 1,
    backgroundColor: theme.background,
  },
  fileBox: {
    flexDirection: "row",
    alignItems: "center",
    gap: 8,
    padding: 8,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: theme.text,
  },
  fileIcon: {
    color: theme.text,
  },
  fileText: {
    color: theme.text,
    fontSize: 14,
    opacity: 0.8,
  },
  id: {
    color: theme.text,
    fontSize: 12,
    opacity: 0.6,
  },
  text: {
    color: theme.text,
    fontSize: 16,
  },
}))
