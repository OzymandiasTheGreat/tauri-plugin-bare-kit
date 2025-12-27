import RPC from "bare-rpc"
import { useEffect, useMemo, useState } from "react"
import { Worklet } from "tauri-plugin-bare-kit-api"

import type { IPC } from "@/common/bare-rpc"

interface WorkletOptions {
  filename?: string
  source?: string | Uint8Array
  args?: string[]
  memoryLimit?: number
  assets?: string | null
}

export type { Worklet, IPC }

export function useWorklet(options: WorkletOptions = {}): Worklet | null {
  const { filename = "/app.bundle", source, args = [], memoryLimit, assets } = options
  const [worklet, setWorklet] = useState<Worklet | null>(null)

  useEffect(() => {
    let _worklet: Worklet | null = null

    Worklet.init({ memoryLimit, assets }).then(async (w) => {
      _worklet = w

      _worklet.on("terminate", () => {
        _worklet = null
        setWorklet(null)
      })

      await _worklet.start(filename, source ?? null, args)
      setWorklet(_worklet)
    })

    return () => {
      setWorklet(null)
      _worklet?.terminate()
    }
  }, [])

  return worklet
}

export function useIPC(worklet: Worklet | null): IPC | null {
  const [rpc, setRPC] = useState<RPC | null>(null)
  const [router, setRouter] = useState<RPC.CommandRouter | null>(null)
  const ipc: IPC | null = useMemo(() => {
    if (rpc && router) {
      return {
        event: rpc.event.bind(rpc),
        request: rpc.request.bind(rpc),
        respond: router.respond.bind(router),
      }
    } else {
      return null
    }
  }, [rpc, router])

  useEffect(() => {
    if (worklet) {
      const _router = new RPC.CommandRouter()
      const _rpc = new RPC(worklet.IPC, _router as any)

      setRPC(_rpc)
      setRouter(_router)
    } else {
      setRPC(null)
      setRouter(null)
    }
  }, [worklet])

  return ipc
}
