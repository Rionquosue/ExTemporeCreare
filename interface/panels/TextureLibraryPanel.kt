package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.tools.SettingNote

/**
 * The Texture library (`uistandards.md` §7.4): a collapsible floating panel for texture presets.
 * The engine has no texture layer yet, so it lists none.
 */
@Composable
fun TextureLibraryPanel(expanded: Boolean, onToggle: () -> Unit, modifier: Modifier = Modifier) {
    FloatingPanel("Texture library", "00", expanded, onToggle, modifier.width(260.dp)) {
        Box(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp)) {
            SettingNote("No texture presets yet: texture layers are not in the engine yet.")
        }
    }
}

// GLOSSARY
// - TextureLibraryPanel(expanded, onToggle, modifier): the Texture library panel.
