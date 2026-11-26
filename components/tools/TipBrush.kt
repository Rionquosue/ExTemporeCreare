package com.extemporecreare.app.components.tools

import com.extemporecreare.app.components.engine.EngineBridge
import org.json.JSONObject

/**
 * A stroke tool's tip as a Calligraphic brush in the document: set to the tool's settings before
 * each stroke (made when the document has none) and made the brush the Paintbrush paints with.
 * Run inside the stroke's undo group, so the brush change is part of the stroke's undo step.
 */
object TipBrush {

    /** Set the document's Calligraphic brush [name] to [params] (made when missing) and make it current → an error message, or `null`. */
    fun use(engine: EngineBridge, name: String, params: JSONObject): String? {
        val edited = EngineAnswer(engine.execute("brush.options", JSONObject().put("name", name).put("params", params).toString()))
        if (!edited.isOk) {
            val made = JSONObject().put("type", "calligraphic").put("name", name).put("params", params)
            EngineAnswer(engine.execute("brush.new", made.toString())).error?.let { return it }
        }
        return EngineAnswer(engine.execute("brush.setCurrent", JSONObject().put("name", name).toString())).error
    }
}

// GLOSSARY
// - TipBrush: a stroke tool's tip as a document Calligraphic brush.
// - use(engine, name, params): set or make the brush and make it current.
