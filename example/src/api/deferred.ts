export default class Deferred<T = void> {
  private _promise: Promise<T>
  private _resolved = false
  private _resolve: (value: T) => void
  private _reject: (err: Error) => void

  constructor() {
    this._promise = new Promise<T>((resolve, reject) => {
      this._resolve = resolve
      this._reject = reject
    })
  }

  get promise(): Promise<T> {
    return this._promise
  }

  resolve(value: T) {
    if (this._resolved) {
      return
    }

    this._resolved = true
    this._resolve(value)
  }

  reject(err: Error) {
    if (this._resolved) {
      return
    }

    this._resolved = true
    this._reject(err)
  }
}
