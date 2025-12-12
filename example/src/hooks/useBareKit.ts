import RPC from "bare-rpc"
import type { Encoder } from "compact-encoding"
import { useEffect, useMemo, useState } from "react"

import { Worklet } from "tauri-plugin-bare-kit-api"

declare module "bare-rpc" {
  class CommandRouter {
    respond(
      command: number,
      handler: (req: RPC.IncomingRequest, data: Uint8Array) => void | Promise<void>,
    ): void
    respond<V, Q = V, R = V>(
      command: number,
      options: {
        valueEncoding?: Encoder<V>
        requestEncoding?: Encoder<Q>
        responseEncoding?: Encoder<R>
      },
      handler: (req: RPC.IncomingRequest, data: V | Q) => V | R | void | Promise<V | R | void>,
    ): void
  }
}

interface WorkletOptions {
  filename?: string
  source?: string | Uint8Array
  args?: string[]
  memoryLimit?: number
  assets?: string | null
}

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

export type IPC = { request: RPC["request"]; respond: RPC.CommandRouter["respond"] }

export function useIPC(worklet: Worklet | null): IPC | null {
  const [rpc, setRPC] = useState<RPC | null>(null)
  const [router, setRouter] = useState<RPC.CommandRouter | null>(null)
  const iface = useMemo(() => {
    if (router && rpc) {
      return {
        respond: router.respond.bind(router),
        request: rpc.request.bind(rpc),
      }
    } else {
      return null
    }
  }, [rpc, router])

  useEffect(() => {
    if (worklet) {
      const { IPC } = worklet
      const _router = new RPC.CommandRouter()
      const _rpc = new RPC(IPC, _router as any)

      setRouter(_router)
      setRPC(_rpc)
    } else {
      setRouter(null)
      setRPC(null)
    }
  }, [worklet])

  return iface
}
