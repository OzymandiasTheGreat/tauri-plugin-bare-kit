import b4a from "b4a"
import BareKit from "tauri-plugin-bare-kit-api"

window.b4a = b4a
window.BareKit = { Worklet: BareKit.Worklet }

window.Deferred = class {
  constructor() {
    this.promise = new Promise((resolve, reject) => {
      this.resolve = resolve
      this.reject = reject
    })
  }
}
