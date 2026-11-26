package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingNote

/**
 * The Reference board (`uistandards.md` §8.2): a collapsible lower-right panel for the `.etcr`
 * board, its header showing how many references it holds and an add button ([onAdd]). The board's
 * own view is not built yet.
 */
@Composable
fun ReferenceBoardPanel(references: Int, expanded: Boolean, onToggle: () -> Unit, onAdd: () -> Unit, modifier: Modifier = Modifier) {
    FloatingPanel(
        "Reference board",
        "%02d REFS".format(references),
        expanded,
        onToggle,
        modifier.width(270.dp),
        actions = { Icon(Icons.Outlined.Add, "Add reference", tint = EtcColors.Text, modifier = Modifier.size(20.dp).clickable(onClick = onAdd)) },
    ) {
        Box(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp)) {
            SettingNote("The board's view is not built yet. References live in a separate .etcr board.")
        }
    }
}

// GLOSSARY
// - ReferenceBoardPanel(references, expanded, onToggle, onAdd, modifier): the Reference board panel.
