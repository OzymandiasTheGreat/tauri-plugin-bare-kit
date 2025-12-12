import { useEffect, useState } from "react"
import { getCurrentWindow } from "@tauri-apps/api/window"

export default function useColorScheme(): "light" | "dark" {
  const initial = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
  const [scheme, setScheme] = useState<"light" | "dark">(initial)

  useEffect(() => {
    const wnd = getCurrentWindow()
    let unlisten: (() => void) | null = null

    wnd.theme().then((theme) => setScheme(theme))
    wnd.onThemeChanged(({ payload: theme }) => setScheme(theme)).then((fn) => (unlisten = fn))

    return () => unlisten?.()
  }, [])

  return scheme
}
