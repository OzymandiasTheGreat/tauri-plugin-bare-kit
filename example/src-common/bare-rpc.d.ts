import type RPC from "bare-rpc"
import type { Encoder } from "compact-encoding"

declare module "bare-rpc" {
  class CommandRouter {
    respond(
      command: number,
      handler: (req: RPC.IncomingRequest, data: Uint8Array) => void | Promise<void>,
    ): void
    respond<V, Q = V, R = V>(
      command: number,
      options: {
        valueEncoding?: Encoder<V> | null
        requestEncoding?: Encoder<Q> | null
        responseEncoding?: Encoder<R> | null
      },
      handler: (req: RPC.IncomingRequest, data: V | Q) => V | R | void | Promise<V | R | void>,
    ): void
  }
}

namespace IPC {
  export {
    IncomingEvent,
    IncomingRequest,
    IncomingStream,
    OutgoingEvent,
    OutgoingRequest,
    OutgoingStream,
  } from "bare-rpc"
}

export interface IPC {
  event: RPC["event"]
  request: RPC["request"]
  respond: RPC.CommandRouter["respond"]
}
