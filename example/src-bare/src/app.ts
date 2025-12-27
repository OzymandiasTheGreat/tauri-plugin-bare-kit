import Process from "bare-process"
import RPC from "bare-rpc"
import c from "compact-encoding"

// Shim process for node compatibility
global.process = Process as unknown as NodeJS.Process

import type {} from "./common/bare-rpc"
import { Method } from "./common/ipc"
import Beam from "./beam"
import { bench, benchStreaming } from "./bench"

const { IPC } = BareKit

const router = new RPC.CommandRouter()
const rpc = new RPC(IPC, router as any)
const ipc = {
  event: rpc.event.bind(rpc),
  request: rpc.request.bind(rpc),
  respond: router.respond.bind(router),
}
const beam = new Beam(ipc)

router.respond(Method.BenchEncrypt, { valueEncoding: c.raw }, bench)
router.respond(Method.BenchDecrypt, { valueEncoding: c.raw }, bench)
// Need to pass null as encoding to avoid errors for streaming requests/replies
router.respond(Method.BenchEncryptStreaming, { valueEncoding: null as any }, benchStreaming)
router.respond(Method.BenchDecryptStreaming, { valueEncoding: null as any }, benchStreaming)
