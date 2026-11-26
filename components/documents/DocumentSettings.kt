package com.extemporecreare.app.components.documents

import android.content.Context
import org.json.JSONObject

/**
 * A new document's settings (`uistandards.md` §5.3): its [name], whether the canvas is [infinite],
 * its [width] × [height] in [units] (finite only), its [ppi], [colorMode] (`rgb` or `cmyk`, what the
 * engine supports) and [background] (`transparent`, `white` or `#rrggbb`). Document points are
 * pixels: a length in inches, centimetres or millimetres is converted to pixels at the resolution.
 */
data class DocumentSettings(
    val name: String = "Untitled",
    val infinite: Boolean = false,
    val width: Double = 1920.0,
    val height: Double = 1080.0,
    val units: String = "px",
    val ppi: Double = 144.0,
    val colorMode: String = "rgb",
    val background: String = "transparent",
) {
    /** [value] in [units] as pixels at [ppi]. */
    fun toPixels(value: Double): Double = when (units) {
        "in" -> value * ppi
        "cm" -> value / 2.54 * ppi
        "mm" -> value / 25.4 * ppi
        else -> value
    }

    /** The width and height in pixels. */
    val pixels: Pair<Double, Double> get() = toPixels(width) to toPixels(height)

    /** What the engine's `newDocument` takes. */
    fun toEngineJson(): JSONObject {
        val (w, h) = pixels
        val o = JSONObject().put("name", name.ifBlank { "Untitled" }).put("infinite", infinite).put("colorMode", colorMode)
            .put("ppi", ppi.coerceIn(1.0, 2400.0)).put("background", background).put("units", "Pixels")
        if (!infinite) o.put("width", w.coerceIn(1.0, MAX_SIDE)).put("height", h.coerceIn(1.0, MAX_SIDE))
        return o
    }

    /** The settings as JSON, to keep as the defaults. */
    fun toJson(): JSONObject = JSONObject().put("name", name).put("infinite", infinite).put("width", width).put("height", height)
        .put("units", units).put("ppi", ppi).put("colorMode", colorMode).put("background", background)

    companion object {
        /** The largest side the engine takes, in points. */
        const val MAX_SIDE = 3_999_856.0

        /** The settings in [json], defaults for what is missing. */
        fun fromJson(json: JSONObject?): DocumentSettings {
            val d = DocumentSettings()
            if (json == null) return d
            return DocumentSettings(
                name = json.optString("name", d.name),
                infinite = json.optBoolean("infinite", d.infinite),
                width = json.optDouble("width", d.width),
                height = json.optDouble("height", d.height),
                units = json.optString("units", d.units),
                ppi = json.optDouble("ppi", d.ppi),
                colorMode = json.optString("colorMode", d.colorMode),
                background = json.optString("background", d.background),
            )
        }

        /** The reusable defaults New Document starts from (the last settings made), kept in the app's preferences. */
        fun defaults(context: Context): DocumentSettings =
            fromJson(context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY, null)?.let { runCatching { JSONObject(it) }.getOrNull() })

        /** Keep [settings] as the defaults. */
        fun keepDefaults(context: Context, settings: DocumentSettings) {
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit().putString(KEY, settings.copy(name = "Untitled").toJson().toString()).apply()
        }

        /** The preferences file and key of the defaults. */
        const val PREFS = "document-defaults"
        const val KEY = "settings"
    }
}

// GLOSSARY
// - DocumentSettings: a new document's name, canvas mode, size, units, ppi, colour mode and background.
// - toPixels(value), pixels: lengths in pixels at the resolution.
// - toEngineJson(): what EngineBridge.newDocument takes.
// - toJson(), fromJson(json): the settings as JSON.
// - defaults(context), keepDefaults(context, settings): the reusable New Document defaults.
// - MAX_SIDE: 3 999 856 pt. PREFS, KEY: "document-defaults", "settings".
