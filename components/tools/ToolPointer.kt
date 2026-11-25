package com.extemporecreare.app.components.tools

import org.json.JSONObject

/**
 * One pointer event for a tool, in document points: what `EngineBridge.pointer` takes as JSON.
 * The canvas converts touch and stylus input into these.
 */
data class ToolPointer(val kind: Kind, val x: Double, val y: Double, val pressure: Float = 1f) {

    /** The event kinds, with the names the engine reads. A cancelled touch is sent as [UP]. */
    enum class Kind(val json: String) { DOWN("down"), DRAG("drag"), UP("up"), MOVE("move") }

    /** The event as the engine's pointer event JSON. */
    fun toJson(): String = JSONObject()
        .put("kind", kind.json)
        .put("pos", JSONObject().put("x", x).put("y", y))
        .put("pressure", pressure.toDouble())
        .toString()
}

// GLOSSARY
// - ToolPointer: one pointer event for a tool, in document points.
// - Kind: down, drag, up, move (engine names in `json`).
// - toJson(): the engine's pointer event JSON.
