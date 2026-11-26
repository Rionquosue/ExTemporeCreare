package com.extemporecreare.app.components.tools

import com.extemporecreare.app.components.engine.EngineBridge
import org.json.JSONArray
import org.json.JSONObject

/**
 * Finishes the objects a stroke tool drew: their stroke weight, and the hardness as the engine's
 * Feather effect (`stylize.feather`): below 100 % hardness, a feather radius of
 * `size / 2 × (1 − hardness / 100)` points.
 */
object StrokeFinish {

    /** The feather radius in points for a stroke [size] points wide at [hardness] percent. */
    fun featherRadius(size: Double, hardness: Double): Double = size / 2 * (1 - hardness.coerceIn(0.0, 100.0) / 100)

    /**
     * Give [created] the stroke [weight] (when not `null`), a feather of [radius] points (when above
     * 0), then each of [effects] (`effect id` to its parameters) → an error message, or `null`.
     */
    fun apply(engine: EngineBridge, created: JSONArray, weight: Double?, radius: Double, effects: List<Pair<String, JSONObject>> = emptyList()): String? {
        if (created.length() == 0) return null
        if (weight != null) {
            val params = JSONObject().put("ids", created).put("weight", weight)
            EngineAnswer(engine.execute("stroke.set", params.toString())).error?.let { return it }
        }
        if (radius > 0) {
            val params = JSONObject().put("ids", created).put("effect", "stylize.feather")
                .put("params", JSONObject().put("radius", radius))
            EngineAnswer(engine.execute("effect.apply", params.toString())).error?.let { return it }
        }
        for ((effect, values) in effects) {
            val params = JSONObject().put("ids", created).put("effect", effect).put("params", values)
            EngineAnswer(engine.execute("effect.apply", params.toString())).error?.let { return it }
        }
        return null
    }
}

// GLOSSARY
// - StrokeFinish: stroke weight and feather (hardness) for what a stroke tool drew.
// - featherRadius(size, hardness): the feather radius for a size and hardness.
// - apply(engine, created, weight, radius, effects): set the weight, feather and further effects of the created objects.
