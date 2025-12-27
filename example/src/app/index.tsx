import { View } from "react-native"

import Composer from "@/components/composer"
import Stream from "@/components/stream"
import createThemedStyleSheet from "@/hooks/useTheme"

export default function HomeScreen() {
  const styles = useStyles()

  return (
    <View style={styles.container}>
      <Composer />
      <View style={styles.splitter} />
      <Stream />
    </View>
  )
}

const useStyles = createThemedStyleSheet((theme) => ({
  container: {
    flex: 1,
    flexDirection: "row",
  },
  splitter: {
    backgroundColor: theme.input,
    width: 3,
  },
}))
