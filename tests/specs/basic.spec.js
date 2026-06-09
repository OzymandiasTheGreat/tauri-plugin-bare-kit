describe("bare-kit", () => {
  it("suspend resume", async () => {
    const result = await browser.executeAsync(async (done) => {
      const { b4a, BareKit } = window

      let suspend = false
      let resume = false

      const worklet = await BareKit.Worklet.init()

      worklet.on("suspend", () => (suspend = true))
      worklet.on("resume", () => (resume = true))

      await worklet.start("/app.js", `console.log("Hello, World!")`)
      await worklet.suspend(10)
      await new Promise((resolve) => setTimeout(resolve, 100))
      await worklet.resume()
      await new Promise((resolve) => setTimeout(resolve, 100))
      await worklet.terminate()

      done({ suspend, resume })
    })

    expect(result.suspend).toBe(true)
    expect(result.resume).toBe(true)
  })

  it("ipc", async () => {
    const result = await browser.executeAsync(async (done) => {
      const { b4a, BareKit } = window

      const payload = "Hello, World!"
      let matches = 0

      const worklet = await BareKit.Worklet.init()
      const IPC = worklet.IPC

      IPC.on("error", () => done(0))
      IPC.on("data", (data) => {
        if (b4a.toString(data) === payload) {
          matches++
        }

        if (matches < 3) {
          IPC.write(data)
        }
      })

      await worklet.start(
        "/app.js",
        `BareKit.IPC.on("data", (data) => BareKit.IPC.write(data)).write("${payload}")`,
      )
      await new Promise((resolve) => setTimeout(resolve, 100))
      await worklet.terminate()

      done(matches)
    })

    expect(result).toBe(3)
  })

  it("ipc large write", async () => {
    const result = await browser.executeAsync(async (done) => {
      const { b4a, BareKit } = window

      const payload = b4a.alloc(4_194_304, 13)
      const received = []

      const worklet = await BareKit.Worklet.init()
      const IPC = worklet.IPC

      IPC.on("error", () => done(false))
      IPC.on("data", (data) => received.push(data))

      await worklet.start(
        "/app.js",
        `BareKit.IPC.on("data", (data) => BareKit.IPC.write(data))`,
      )
      IPC.write(payload)
      await new Promise((resolve) => setTimeout(resolve, 3_000))
      await worklet.terminate()

      const data = b4a.concat(received)

      done(b4a.equals(data, payload))
    })

    expect(result).toBe(true)
  })
})
