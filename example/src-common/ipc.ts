import c from "compact-encoding"

export enum Method {
  MessengerReady = 0,
  MessengerPeer,
  MessengerMessage,
  BenchEncrypt = 0xff01,
  BenchDecrypt,
  BenchEncryptStreaming,
  BenchDecryptStreaming,
}

export const Peer = c.fixed32
