import MDI from "@expo/vector-icons/MaterialCommunityIcons"
import { DarkTheme, DefaultTheme, ThemeProvider } from "@react-navigation/native"
import { Stack } from "expo-router"
import { useCallback, useEffect } from "react"
import { Pressable } from "react-native"
import "react-native-reanimated"

import bench from "@/api/bench"
import { setIPC } from "@/api/store"
import bundle from "@/bundle"
import Colors from "@/constants/colors"
import { useIPC, useWorklet } from "@/hooks/useBareKit"
import useColorScheme from "@/hooks/useColorScheme"

export {
  // Catch any errors thrown by the Layout component.
  ErrorBoundary,
} from "expo-router"

export const unstable_settings = {
  // Ensure that reloading on `/modal` keeps a back button present.
  initialRouteName: "index",
}

export default function RootLayout() {
  const colorScheme = useColorScheme()
  const worklet = useWorklet({ source: bundle })
  const ipc = useIPC(worklet)

  const runBench = useCallback(() => bench(ipc!), [ipc])

  useEffect(() => setIPC(ipc), [ipc])

  return (
    <ThemeProvider value={colorScheme === "dark" ? DarkTheme : DefaultTheme}>
      <Stack>
        <Stack.Screen
          name="index"
          options={{
            title: "",
            headerRight: () => (
              <Pressable onPress={runBench}>
                {({ pressed }) => (
                  <MDI
                    name="timer-outline"
                    size={24}
                    color={Colors[colorScheme].text}
                    style={{ marginRight: 16, opacity: pressed ? 0.5 : 1 }}
                  />
                )}
              </Pressable>
            ),
          }}
        />
      </Stack>
    </ThemeProvider>
  )
}
