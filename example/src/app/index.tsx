import { useCallback } from "react"
import { Button, FlatList, ListRenderItem, Text, View } from "react-native"

import { useStore } from "@/hooks/useStore"
import createThemedStyleSheet from "@/hooks/useTheme"

const Template = "This is a very long hash string"

function randint(min = 1, max = Number.MAX_SAFE_INTEGER): number {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min + 1)) + min
}

export default function HomeScreen() {
  const styles = useStyles()
  const store = useStore()

  const addRandomPeer = useCallback(() => store?.addPeer(`${randint()}: ${Template}`), [store])

  const keyExtractor = useCallback((item: string, index: number) => item, [])
  const renderItem: ListRenderItem<string> = useCallback(
    ({ item }) => (
      <View style={styles.listItem}>
        <Text style={styles.listItemText}>{item}</Text>
      </View>
    ),
    [],
  )

  return (
    <View style={styles.container}>
      <FlatList data={store?.peers.value} keyExtractor={keyExtractor} renderItem={renderItem} />
      <View>
        <Button title="Add Peer" onPress={addRandomPeer} />
      </View>
    </View>
  )
}

const useStyles = createThemedStyleSheet((theme) => ({
  container: {
    flex: 1,
    backgroundColor: theme.background,
  },
  listItem: {
    padding: 16,
  },
  listItemText: {
    color: theme.text,
    fontSize: 18,
  },
  toolbar: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "flex-end",
    height: 96,
    padding: 16,
  },
}))
