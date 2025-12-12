import { Link, Stack } from "expo-router"
import { Text, View } from "react-native"

import createThemedStyleSheet from "@/hooks/useTheme"

export default function NotFoundScreen() {
  const styles = useStyles()

  return (
    <>
      <Stack.Screen options={{ title: "Oops!" }} />
      <View style={styles.container}>
        <Text style={styles.title}>This screen doesn't exist.</Text>

        <Link href="/" style={styles.link}>
          <Text style={styles.linkText}>Go to home screen!</Text>
        </Link>
      </View>
    </>
  )
}

const useStyles = createThemedStyleSheet((theme) => ({
  container: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: theme.background,
    padding: 20,
  },
  title: {
    color: theme.text,
    fontSize: 20,
    fontWeight: "bold",
  },
  link: {
    marginTop: 15,
    paddingVertical: 15,
  },
  linkText: {
    fontSize: 14,
    color: "#2e78b7",
  },
}))
