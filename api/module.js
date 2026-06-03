const { invoke, transformCallback } = require("@tauri-apps/api/core")
const { emit } = require("@tauri-apps/api/event")
const b4a = require("b4a")

const PLATFORM = window.__TAURI_BARE_KIT_PLUGIN_INTERNALS__.platform

module.exports.notify = async function (id) {
  if (PLATFORM !== "android") emit(`bare-kit:worklet:${id}`)
}

module.exports.optimizeForMemory = async function (enabled) {
  return invoke(format("bare_optimize_for_memory"), { payload: { enabled } })
}

module.exports.newWorklet = async function (
  memoryLimit,
  assets,
  pollCallback,
  suspendCallback,
  wakeupCallback,
  idleCallback,
  resumeCallback,
) {
  const onPoll = pollCallback != null ? transformCallback(pollCallback, false) : null
  const onSuspend = suspendCallback != null ? transformCallback(suspendCallback, false) : null
  const onWakeup = wakeupCallback != null ? transformCallback(wakeupCallback, false) : null
  const onIdle = idleCallback != null ? transformCallback(idleCallback, false) : null
  const onResume = resumeCallback != null ? transformCallback(resumeCallback, false) : null

  return invoke(format("bare_new_worklet"), {
    payload: {
      memoryLimit,
      assets,
      onPoll,
      onSuspend,
      onWakeup,
      onIdle,
      onResume,
    },
  })
}

module.exports.startFile = async function (id, filename, args = []) {
  return invoke(format("bare_start_file"), { payload: { id, filename, args } })
}

module.exports.startUTF8 = async function (id, filename, source, args = []) {
  return invoke(format("bare_start_utf8"), { payload: { id, filename, source, args } })
}

module.exports.startBytes = async function (id, filename, source, args = []) {
  return invoke(format("bare_start_bytes"), { payload: { id, filename, source, args } })
}

module.exports.read = async function (id) {
  let data = await invoke(format("bare_read"), { payload: { id } })

  if (data && PLATFORM === "android") {
    data = b4a.from(data, "base64")
  }

  if (data?.byteLength > 0) {
    return b4a.from(data)
  }

  return null
}

module.exports.write = async function (id, data) {
  let payload = b4a.concat([b4a.alloc(1, id), data ? data : b4a.allocUnsafe(0)])

  if (PLATFORM === "android") {
    payload = { payload: b4a.toString(payload, "base64") }
  }

  return invoke(format("bare_write"), payload)
}

module.exports.update = async function (id, readable, writable) {
  return invoke(format("bare_update"), { payload: { id, readable, writable } })
}

module.exports.suspend = async function (id, linger) {
  return invoke(format("bare_suspend"), { payload: { id, linger } })
}

module.exports.resume = async function (id) {
  return invoke(format("bare_resume"), { payload: { id } })
}

module.exports.wakeup = async function (id, deadline) {
  return invoke(format("bare_wakeup"), { payload: { id, deadline } })
}

module.exports.terminate = async function (id) {
  return invoke(format("bare_terminate"), { payload: { id } })
}

function format(fn) {
  return `plugin:bare-kit|${fn}`
}
