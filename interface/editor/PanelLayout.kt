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
        layersOpen = true
        texturesOpen = false
        boardOpen = false
        focus = false
        keep()
    }

    private fun keep() {
        val e = prefs.edit().putBoolean("layersOpen", layersOpen).putBoolean("texturesOpen", texturesOpen).putBoolean("boardOpen", boardOpen)
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

        /** Where each panel docks by default (`uistandards.md` §4). */
        val DEFAULT_DOCKS = mapOf(TEXTURES to "topStart", LAYERS to "topEnd", BOARD to "bottomEnd")
    }
}

// GLOSSARY
// - PanelLayout: the panels' places, expanded states and focus mode, kept in the preferences.
// - Place: docked at an edge or floating at a point. places: each panel's place. place(key, place): move a panel.
// - layersOpen, texturesOpen, boardOpen: whether each panel is expanded. focus: canvas focus mode.
// - toggleLayers(), toggleTextures(), toggleBoard(), toggleFocus(): change them.
// - reset(): the default layout. keep(): save it.
// - FILE: "panel-layout". TEXTURES, LAYERS, BOARD: the panels' keys. DEFAULT_DOCKS: their default docks.
