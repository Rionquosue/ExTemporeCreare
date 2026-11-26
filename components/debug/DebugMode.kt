package com.extemporecreare.app.components.debug

import android.util.Log

/**
 * The app's debug mode, switched on and off from ADB without a rebuild through Android's own
 * per-tag log level:
 *
 * - on: `adb shell setprop log.tag.ETCDebug DEBUG`
 * - off: `adb shell setprop log.tag.ETCDebug INFO`
 * - read: `adb logcat -s ETCDebug`
 *
 * While it is on, [log] writes to logcat under [TAG] and the editor shows its on-screen readout.
 */
object DebugMode {

    /** The logcat tag, and the name of the log level property that switches debug mode. */
    const val TAG = "ETCDebug"

    /** Whether debug mode is on (read from the system property each time). */
    val isOn: Boolean get() = Log.isLoggable(TAG, Log.DEBUG)

    /** Log [message] under [area] (`pen`, `tool`, `engine`, `render`…) while debug mode is on. */
    fun log(area: String, message: String) {
        if (isOn) Log.d(TAG, "[$area] $message")
    }
}

// GLOSSARY
// - DebugMode: the debug mode switched from ADB with the log.tag.ETCDebug property.
// - TAG: "ETCDebug".
// - isOn: whether debug mode is on.
// - log(area, message): log to logcat while on.
