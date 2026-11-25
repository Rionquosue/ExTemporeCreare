package com.extemporecreare.app.components.tools

import org.json.JSONArray

/**
 * What a tool's call left for the UI: the engine tool's requests (`{"type", …}`), the objects a
 * gesture added and whether it changed the document (on release), and an error message to show, if any.
 */
data class ToolResult(
    val requests: JSONArray = JSONArray(),
    val created: JSONArray = JSONArray(),
    val error: String? = null,
    val changed: Boolean = false,
) {
    companion object {
        /** Nothing to show. */
        val NONE = ToolResult()

        /** A failure with [message]. */
        fun failed(message: String) = ToolResult(error = message)

        /** [message] as a failure, or nothing when it is `null`. */
        fun failedOrNone(message: String?) = if (message == null) NONE else failed(message)

        /** The result of an `EngineBridge.pointer` answer. */
        fun fromPointer(text: String): ToolResult {
            val answer = EngineAnswer(text)
            val ok = answer.okObject ?: return failed(answer.error ?: "pointer call failed")
            return ToolResult(ok.optJSONArray("requests") ?: JSONArray(), ok.optJSONArray("created") ?: JSONArray(), changed = ok.optBoolean("changed"))
        }
    }
}

// GLOSSARY
// - ToolResult: requests, created objects, whether the document changed, and an error from a tool call.
// - NONE: nothing to show.
// - failed(message): a failure.
// - failedOrNone(message): a failure when message is not null.
// - fromPointer(text): the result of a pointer answer.
