import { useMemo } from "react"
import { StyleSheet } from "react-native"

import Colors from "@/constants/colors"
import useColorScheme from "@/hooks/useColorScheme"

export default function createThemedStyleSheet<
  T extends StyleSheet.NamedStyles<T> | StyleSheet.NamedStyles<any>,
>(builder: (theme: (typeof Colors)["dark"]) => T & StyleSheet.NamedStyles<any>): () => T {
  return () => {
    const theme = useColorScheme()
    const stylesheet = useMemo(() => StyleSheet.create(builder(Colors[theme])), [theme])
    return stylesheet
  }
}
