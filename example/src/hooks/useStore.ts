import { useSignals } from "@preact/signals-react/runtime"
import type { DeepSignal } from "deepsignal/react"

import { store, type Store } from "@/api/store"

export default function useStore(): DeepSignal<Store> {
  useSignals()

  return store
}
