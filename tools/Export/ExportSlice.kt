package com.extemporecreare.app.tools.export

import com.extemporecreare.app.components.tools.DragBox
import org.json.JSONObject

/**
 * One export slice as the engine keeps it (`filestandard.md` §7A): a named output region in
 * document pixels (one per point) with its aspect ratio and output settings.
 */
data class ExportSlice(
    val id: String,
    val name: String,
    val x: Double,
    val y: Double,
    val width: Double,
    val height: Double,
    val aspectX: Double,
    val aspectY: Double,
    val format: String,
    val scale: Double,
    val profileRef: String,
    val visible: Boolean,
) {
    /** The slice's region as a [DragBox]. */
    fun box(): DragBox = DragBox.of(x, y, width, height)

    companion object {
        /** The slice in [json] (an `export.slice.list` entry), or `null` when it has no id or region. */
        fun fromJson(json: JSONObject): ExportSlice? {
            val rect = json.optJSONObject("rectPx") ?: return null
            val ratio = json.optJSONObject("aspectRatio") ?: JSONObject()
            return ExportSlice(
                id = json.optString("id").takeIf { it.isNotEmpty() } ?: return null,
                name = json.optString("name"),
                x = rect.optDouble("x", 0.0),
                y = rect.optDouble("y", 0.0),
                width = rect.optDouble("width", 1.0),
                height = rect.optDouble("height", 1.0),
                aspectX = ratio.optDouble("x", 1.0),
                aspectY = ratio.optDouble("y", 1.0),
                format = json.optString("format", "png"),
                scale = json.optDouble("scale", 1.0),
                profileRef = json.optString("exportProfileRef", "builtin:sRGB"),
                visible = json.optBoolean("visible", true),
            )
        }
    }
}

/** The export dot grid (§7A `export.grid`): shown or not, snapping or not, and its spacing in document pixels. */
data class ExportGrid(val visible: Boolean = false, val snapEnabled: Boolean = false, val spacingPx: Double = 32.0) {
    companion object {
        /** The grid in [json], defaults for what is missing. */
        fun fromJson(json: JSONObject?): ExportGrid {
            val d = ExportGrid()
            if (json == null) return d
            return ExportGrid(json.optBoolean("visible", d.visible), json.optBoolean("snapEnabled", d.snapEnabled), json.optDouble("spacingPx", d.spacingPx))
        }
    }
}

/** What `export.slice.list` answers: the slices in order, the grid, and the bounds of the art an export draws (`null`: nothing). */
data class ExportSliceList(val slices: List<ExportSlice>, val grid: ExportGrid, val contentBounds: DragBox?) {
    companion object {
        /** The list in [json]. */
        fun fromJson(json: JSONObject): ExportSliceList {
            val arr = json.optJSONArray("slices")
            val slices = (0 until (arr?.length() ?: 0)).mapNotNull { i -> arr?.optJSONObject(i)?.let(ExportSlice::fromJson) }
            val b = json.optJSONObject("contentBounds")
            val bounds = b?.let { DragBox.of(it.optDouble("x"), it.optDouble("y"), it.optDouble("width"), it.optDouble("height")) }
            return ExportSliceList(slices, ExportGrid.fromJson(json.optJSONObject("grid")), bounds)
        }

        /** No slices, the default grid, nothing to export. */
        val EMPTY = ExportSliceList(emptyList(), ExportGrid(), null)
    }
}

// GLOSSARY
// - ExportSlice: one export slice (id, name, region, aspect ratio, format, scale, profile, visibility).
// - box(): the slice's region as a DragBox.
// - ExportGrid: the export dot grid (visible, snapEnabled, spacingPx).
// - ExportSliceList: the slices, the grid and the content bounds from export.slice.list.
// - fromJson(json): each read from the engine's JSON.
// - EMPTY: no slices, default grid, nothing to export.
