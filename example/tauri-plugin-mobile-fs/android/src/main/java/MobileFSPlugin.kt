package sh.quince.mobilefs

import android.annotation.SuppressLint
import android.app.Activity
import android.provider.OpenableColumns
import androidx.core.net.toUri
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import kotlin.math.max

@InvokeArg
class PingArgs {
  var value: String? = null
}

@InvokeArg
class GetFileDescriptorArgs {
    lateinit var uri: String
    lateinit var mode: String
}

@InvokeArg
class GetFileNameArgs {
    lateinit var uri: String
}

@TauriPlugin
class MobileFSPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun ping(invoke: Invoke) {
        val args = invoke.parseArgs(PingArgs::class.java)

        val ret = JSObject()
        ret.put("value", args.value)
        invoke.resolve(ret)
    }

    @SuppressLint("Recycle")
    @Command
    fun getFileDescriptor(invoke: Invoke) {
        val args = invoke.parseArgs(GetFileDescriptorArgs::class.java)
        val res = JSObject()
        val fd = activity.contentResolver.openAssetFileDescriptor(args.uri.toUri(), args.mode)?.parcelFileDescriptor?.detachFd()

        res.put("fd", fd)
        invoke.resolve(res)
    }

    @Command
    fun getFileName(invoke: Invoke) {
        val uri = invoke.parseArgs(GetFileNameArgs::class.java).uri.toUri()
        val res = JSObject()
        var name: String? = null

        if (uri.scheme == "content") {
            val projection = arrayOf(OpenableColumns.DISPLAY_NAME)
            val cursor = activity.contentResolver.query(uri, projection, null, null, null)

            try {
                if (cursor != null && cursor.moveToFirst()) {
                    name = cursor.getString(max(0, cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)))
                }
            } finally {
                cursor?.close()
            }
        }

        if (name == null) {
            name = uri.lastPathSegment
        }

        res.put("filename", name)
        invoke.resolve(res)
    }
}
