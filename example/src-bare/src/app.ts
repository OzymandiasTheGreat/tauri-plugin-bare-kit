import RPC from "bare-rpc"
import c from "compact-encoding"

import type {} from "./common/bare-rpc"
import { Method } from "./common/ipc"
import { bench, benchStreaming } from "./bench"

const { IPC } = BareKit

const router = new RPC.CommandRouter()

router.respond(Method.BenchEncrypt, { valueEncoding: c.raw }, bench)
router.respond(Method.BenchDecrypt, { valueEncoding: c.raw }, bench)
// Need to pass null as encoding to avoid errors for streaming requests/replies
router.respond(Method.BenchEncryptStreaming, { valueEncoding: null as any }, benchStreaming)
router.respond(Method.BenchDecryptStreaming, { valueEncoding: null as any }, benchStreaming)

const rpc = new RPC(IPC, router as any)
