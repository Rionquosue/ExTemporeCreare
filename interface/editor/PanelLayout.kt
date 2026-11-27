package com.extemporecreare.app.ui.editor

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/**
 * The floating panels' layout, kept in the app's preferences (`uistandards.md` §10, §12): where each
 * panel is (docked to an edge, or floating at a point), which are expanded, and whether canvas focus
 * mode hides them. [reset] restores the default layout without touching the artwork.
 */
class PanelLayout(context: Context) {

    /** Where a panel is: docked at [dock] (`topStart`, `topEnd`, `bottomEnd`) or, with dock `float`, at [x], [y] (pixels in the workspace). */
    data class Place(val dock: String, val x: Float = 0f, val y: Float = 0f)

    /** Each panel's place, by its key ([TEXTURES], [LAYERS], [BOARD]). */
    val places = mutableStateMapOf<String, Place>()

    private val prefs = context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    /** Whether each panel is expanded. */
    var layersOpen by mutableStateOf(prefs.getBoolean("layersOpen", true))
        private set
    var texturesOpen by mutableStateOf(prefs.getBoolean("texturesOpen", false))
        private set
    var boardOpen by mutableStateOf(prefs.getBoolean("boardOpen", false))
        private set

    /** How far the dial has been moved from its corner by its grip, in dp. */
    var dialX by mutableStateOf(prefs.getFloat("dialX", 0f))
        private set
    var dialY by mutableStateOf(prefs.getFloat("dialY", 0f))
        private set

    /** Move the dial by ([dx], [dy]) dp, kept within [maxX], [maxY] dp of its corner. */
    fun moveDial(dx: Float, dy: Float, maxX: Float, maxY: Float) {
        dialX = (dialX + dx).coerceIn(0f, maxX)
        dialY = (dialY + dy).coerceIn(-maxY, 0f)
        keep()
    }

    /** Move the dial by ([dx], [dy]) dp while its grip is dragged: its grip stays within a [w] × [h] dp workspace. */
    fun moveDialFreely(dx: Float, dy: Float, w: Float, h: Float) {
        dialX = (dialX + dx).coerceIn(-GRIP_X_DP, w - GRIP_X_DP)
        dialY = (dialY + dy).coerceIn(GRIP_FROM_BOTTOM_DP - h, GRIP_FROM_BOTTOM_DP)
    }

    /** Put the dial's offset at ([x], [y]) dp, kept within [maxX], [maxY] dp of its corner. */
    fun placeDial(x: Float, y: Float, maxX: Float, maxY: Float) {
        dialX = x.coerceIn(0f, maxX)
        dialY = y.coerceIn(-maxY, 0f)
        keep()
    }

    /**
     * The edge the dial is docked to as a toolbar ([EDGES]: `left`, `right`, `top`, `bottom`), or
     * null for the rotary dial.
     */
    var toolbarDock by mutableStateOf(prefs.getString("toolbarDock", null)?.takeIf { it in EDGES })
        private set

    /** While the dial's grip is dragged, the edge it would dock to on release, or null. */
    var toolbarPreview by mutableStateOf<String?>(null)

    /** Dock the dial as a toolbar at [edge], or make it the rotary dial again (null). */
    fun dockToolbar(edge: String?) {
        toolbarDock = edge?.takeIf { it in EDGES }
        keep()
    }

    /** Canvas focus mode: the floating panels hidden, the dial and essential actions kept (§4). */
    var focus by mutableStateOf(false)
        private set

    init {
        for ((key, dock) in DEFAULT_DOCKS) {
            places[key] = Place(prefs.getString("$key.dock", dock) ?: dock, prefs.getFloat("$key.x", 0f), prefs.getFloat("$key.y", 0f))
        }
    }

    /** Put panel [key] at [place]. */
    fun place(key: String, place: Place) {
        places[key] = place
        keep()
    }

    fun toggleLayers() { layersOpen = !layersOpen; keep() }
    fun toggleTextures() { texturesOpen = !texturesOpen; keep() }
    fun toggleBoard() { boardOpen = !boardOpen; keep() }

    /** Enter or leave focus mode; leaving restores the panels as they were. */
    fun toggleFocus() {
        focus = !focus
    }

    /** The default layout. */
    fun reset() {
        for ((key, dock) in DEFAULT_DOCKS) places[key] = Place(dock)
        dialX = 0f
        dialY = 0f
        toolbarDock = null
        layersOpen = true
        texturesOpen = false
        boardOpen = false
        focus = false
        keep()
    }

    private fun keep() {
        val e = prefs.edit().putBoolean("layersOpen", layersOpen).putBoolean("texturesOpen", texturesOpen).putBoolean("boardOpen", boardOpen)
            .putFloat("dialX", dialX).putFloat("dialY", dialY).putString("toolbarDock", toolbarDock)
        for ((key, p) in places) e.putString("$key.dock", p.dock).putFloat("$key.x", p.x).putFloat("$key.y", p.y)
        e.apply()
    }

    companion object {
        /** The preferences file. */
        const val FILE = "panel-layout"

        /** The panels' keys. */
        const val TEXTURES = "textures"
        const val LAYERS = "layers"
        const val BOARD = "board"

        /** The rotary dial's grip centre before the dial is moved: dp from the left, and from the bottom. */
        const val GRIP_X_DP = 40f
        const val GRIP_FROM_BOTTOM_DP = 272f

        /** The edges the dial docks to as a toolbar. */
        val EDGES = listOf("left", "right", "top", "bottom")

        /** Where each panel docks by default (`uistandards.md` §4). */
        val DEFAULT_DOCKS = mapOf(TEXTURES to "topStart", LAYERS to "topEnd", BOARD to "bottomEnd")
    }
}

// GLOSSARY
// - PanelLayout: the panels' places, expanded states and focus mode, kept in the preferences.
// - Place: docked at an edge or floating at a point. places: each panel's place. place(key, place): move a panel.
// - layersOpen, texturesOpen, boardOpen: whether each panel is expanded. focus: canvas focus mode.
// - toggleLayers(), toggleTextures(), toggleBoard(), toggleFocus(): change them.
// - dialX, dialY, moveDial(dx, dy, maxX, maxY), placeDial(x, y, maxX, maxY), moveDialFreely(dx, dy, w, h): the dial's offset from its corner, moved by its grip.
// - toolbarDock, dockToolbar(edge), toolbarPreview, EDGES: the dial docked to an edge as a toolbar, and the edge a grip drag would dock it to.
// - reset(): the default layout. keep(): save it.
// - GRIP_X_DP, GRIP_FROM_BOTTOM_DP: where the rotary dial's grip sits before it is moved.
// - FILE: "panel-layout". TEXTURES, LAYERS, BOARD: the panels' keys. DEFAULT_DOCKS: their default docks.
