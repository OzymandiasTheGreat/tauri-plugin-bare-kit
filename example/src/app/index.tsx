import { useCallback } from "react"
import { FlatList, ListRenderItem, Text, View } from "react-native"

import createThemedStyleSheet from "@/hooks/useTheme"

const Template = "This is a very long hash string"
const data: string[] = []

for (let i = 1; i <= 4; i++) {
  data.push(`${i}: ${Template}`)
}

export default function HomeScreen() {
  const styles = useStyles()

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
      <FlatList data={data} keyExtractor={keyExtractor} renderItem={renderItem} />
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
}))
