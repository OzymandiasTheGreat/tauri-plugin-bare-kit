const { invoke } = require("@tauri-apps/api/core")

module.exports.ping = async function (value) {
  return invoke(format("ping"), { payload: { value } }).then((r) => (r.value ? r.value : null))
}

module.exports.getFileDescriptor = async function (uri, mode) {
  return invoke(format("get_file_descriptor"), { payload: { uri, mode } }).then((r) =>
    r.fd ? r.fd : null,
  )
}

module.exports.getFileName = async function (uri) {
  return invoke(format("get_file_name"), { payload: { uri } }).then((r) =>
    r.filename ? r.filename : null,
  )
}

function format(command) {
  return `plugin:mobile-fs|${command}`
}
