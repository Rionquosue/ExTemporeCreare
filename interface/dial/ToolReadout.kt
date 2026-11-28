package com.extemporecreare.app.ui.dial

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.formatNumber

/**
 * The tool readout beside the dial (`uistandards.md` §6.2): the active tool's name and, for tools
 * with a size, the size; dragging it up or down adjusts the size ([onSize], by a share of the size
 * per point dragged). Tapping it opens the settings ([onTap]).
 */
@Composable
fun ToolReadout(name: String, size: Pair<Double, String>?, onSize: (Double) -> Unit, onTap: () -> Unit, modifier: Modifier = Modifier) {
    Column(
        modifier.pointerInput(size?.first) {
            detectVerticalDragGestures { _, dy ->
                val s = size ?: return@detectVerticalDragGestures
                onSize(s.first * (1.0 - dy / 200.0))
            }
        }.pointerInput(Unit) { detectTapGestures { onTap() } },
    ) {
        Text(name, color = EtcColors.Text, fontSize = 22.sp, fontWeight = FontWeight.Medium)
        if (size != null) {
            Row(verticalAlignment = Alignment.Bottom) {
                Text(formatNumber(size.first, 1), color = EtcColors.Text, fontSize = 20.sp)
                Text(" ${size.second}", color = EtcColors.Muted, fontSize = 18.sp)
            }
        }
    }
}

// GLOSSARY
// - ToolReadout(name, size, onSize, onTap, modifier): the active tool's name and size, with drag-to-adjust.
