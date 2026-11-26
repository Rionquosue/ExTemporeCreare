package com.extemporecreare.app.components.tools

import org.json.JSONArray
import org.json.JSONObject

/**
 * One pointer event for a tool, in document points: what `EngineBridge.pointer` takes as JSON.
 * The canvas converts touch and stylus input into these. A stylus also gives its tilt ([tiltX],
 * [tiltY]: degrees, each −90..90, in document axes, x positive leaning right and y positive leaning
 * down the page) and its barrel [rotation] (degrees clockwise, 0..360), when it reads them.
 */
data class ToolPointer(
    val kind: Kind,
    val x: Double,
    val y: Double,
    val pressure: Float = 1f,
    val tiltX: Float? = null,
    val tiltY: Float? = null,
    val rotation: Float? = null,
) {

    /** The event kinds, with the names the engine reads. A cancelled touch is sent as [UP]. */
    enum class Kind(val json: String) { DOWN("down"), DRAG("drag"), UP("up"), MOVE("move") }

    /** The event as the engine's pointer event JSON. */
    fun toJson(): String {
        val json = JSONObject()
            .put("kind", kind.json)
            .put("pos", JSONObject().put("x", x).put("y", y))
            .put("pressure", pressure.toDouble())
        if ((tiltX != null && tiltY != null) || rotation != null) {
            val pen = JSONObject()
            if (tiltX != null && tiltY != null) pen.put("tilt", JSONArray().put(tiltX.toDouble()).put(tiltY.toDouble()))
            if (rotation != null) pen.put("rotation", rotation.toDouble())
            json.put("pen", pen)
        }
        return json.toString()
    }
}

// GLOSSARY
// - ToolPointer: one pointer event for a tool, in document points, with a stylus's tilt and rotation when read.
// - Kind: down, drag, up, move (engine names in `json`).
// - toJson(): the engine's pointer event JSON (pen pose as "pen": {"tilt", "rotation"}).
