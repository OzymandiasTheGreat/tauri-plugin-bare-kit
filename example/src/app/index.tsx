import { useWindowDimensions, View } from "react-native"

import Composer from "@/components/composer"
import Stream from "@/components/stream"
import createThemedStyleSheet from "@/hooks/useTheme"

export default function HomeScreen() {
  const styles = useStyles()
  const { height, width } = useWindowDimensions()
  const landscape = height < width

  return (
    <View style={[styles.container, { flexDirection: landscape ? "row" : "column-reverse" }]}>
      <Composer />
      <View style={[styles.splitter, landscape ? { width: 3 } : { height: 3 }]} />
      <Stream />
    </View>
  )
}

const useStyles = createThemedStyleSheet((theme) => ({
  container: {
    flex: 1,
  },
  splitter: {
    backgroundColor: theme.input,
  },
}))
