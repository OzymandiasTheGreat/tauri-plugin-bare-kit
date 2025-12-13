import { useEffect, useState } from "react"
import { useSignals } from "@preact/signals-react/runtime"

import type { IPC, Worklet } from "@/hooks/useBareKit"
import { Store } from "@/api/store"

export { useSignals } from "@preact/signals-react/runtime"

export function useCreateStore(worklet: Worklet | null, ipc: IPC | null) {
  useEffect(() => {
    if (worklet && ipc) {
      new Store(worklet, ipc)
    }
  }, [worklet, ipc])
}

export const useStore = (): Store | null => {
  useSignals()

  const [store, setStore] = useState<Store | null>(null)

  useEffect(() => {
    Store.ready.then(() => setStore(Store.instance))
  }, [])

  return store
}
