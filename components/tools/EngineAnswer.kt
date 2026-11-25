package com.extemporecreare.app.components.tools

import org.json.JSONObject

/**
 * One answer of the engine bridge (`{"ok": …}` or `{"error": {"kind", "message"}}`), read once.
 * Text that is not JSON reads as an error.
 */
class EngineAnswer(text: String) {

    /** The answer as JSON. */
    private val json: JSONObject = runCatching { JSONObject(text) }.getOrElse {
        JSONObject().put("error", JSONObject().put("kind", "bridge").put("message", "unreadable engine answer"))
    }

    /** Whether the call succeeded. */
    val isOk: Boolean get() = json.has("ok")

    /** The result when it is an object, else `null`. */
    val okObject: JSONObject? get() = json.optJSONObject("ok")

    /** The error's message, or `null` when the call succeeded. */
    val error: String? get() = if (isOk) null else json.optJSONObject("error")?.optString("message") ?: "engine call failed"
}

// GLOSSARY
// - EngineAnswer: one engine bridge answer, parsed.
// - json: the answer as JSON.
// - isOk: whether the call succeeded.
// - okObject: the result when it is an object.
// - error: the error message, or null on success.
