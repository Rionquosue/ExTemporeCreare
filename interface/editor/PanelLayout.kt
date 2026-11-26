package com.extemporecreare.app.ui.editor

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/**
 * The floating panels' layout, kept in the app's preferences (`uistandards.md` §12): which panels
 * are expanded, and whether canvas focus mode hides them. [reset] restores the default layout
 * without touching the artwork.
 */
class PanelLayout(context: Context) {

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

    fun toggleLayers() { layersOpen = !layersOpen; keep() }
    fun toggleTextures() { texturesOpen = !texturesOpen; keep() }
    fun toggleBoard() { boardOpen = !boardOpen; keep() }

    /** Enter or leave focus mode; leaving restores the panels as they were. */
    fun toggleFocus() {
        focus = !focus
    }

    /** The default layout. */
    fun reset() {
        layersOpen = true
        texturesOpen = false
        boardOpen = false
        focus = false
        keep()
    }

    private fun keep() {
        prefs.edit().putBoolean("layersOpen", layersOpen).putBoolean("texturesOpen", texturesOpen).putBoolean("boardOpen", boardOpen).apply()
    }

    companion object {
        /** The preferences file. */
        const val FILE = "panel-layout"
    }
}

// GLOSSARY
// - PanelLayout: the panels' expanded states and focus mode, kept in the preferences.
// - layersOpen, texturesOpen, boardOpen: whether each panel is expanded. focus: canvas focus mode.
// - toggleLayers(), toggleTextures(), toggleBoard(), toggleFocus(): change them.
// - reset(): the default layout. keep(): save it.
// - FILE: "panel-layout".
