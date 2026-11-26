package com.extemporecreare.app.ui.debug

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.debug.PenReading
import com.extemporecreare.app.ui.theme.EtcColors
import java.util.Locale

/**
 * Debug mode's on-screen readout: the last pointer reading (tool type, pressure, tilt,
 * orientation, hover distance, buttons, the tilt sent to the tools), the active tool, the zoom and
 * the last render's time. Shown while debug mode is on.
 */
@Composable
fun DebugOverlay(reading: PenReading?, tool: String, zoom: Double, renderMs: Long, modifier: Modifier = Modifier) {
    Column(
        modifier
            .widthIn(max = 420.dp)
            .background(EtcColors.Panel, RoundedCornerShape(10.dp))
            .border(1.dp, EtcColors.Gold2, RoundedCornerShape(10.dp))
            .padding(10.dp),
    ) {
        line("DEBUG  tool $tool  zoom ${String.format(Locale.ROOT, "%.3f", zoom)}  render ${renderMs} ms")
        if (reading == null) {
            line("touch or hover the canvas with a pen")
            return@Column
        }
        line("${reading.action}  ${reading.toolType}  buttons ${reading.buttons}")
        line(String.format(Locale.ROOT, "pressure %.3f   distance %.2f", reading.pressure, reading.distance))
        line(String.format(Locale.ROOT, "tilt %.1f°   orientation %.1f°", reading.tilt, reading.orientation))
        line("tools tilt x ${reading.tiltX?.let { String.format(Locale.ROOT, "%.1f°", it) } ?: "-"}  y ${reading.tiltY?.let { String.format(Locale.ROOT, "%.1f°", it) } ?: "-"}  rotation: not reported by Android")
        line(String.format(Locale.ROOT, "screen %.1f, %.1f   doc %.2f, %.2f", reading.x, reading.y, reading.docX, reading.docY))
    }
}

/** One monospaced line of the readout. */
@Composable
private fun line(text: String) {
    Text(text, color = EtcColors.Text, fontSize = 11.sp, fontFamily = FontFamily.Monospace)
}

// GLOSSARY
// - DebugOverlay(reading, tool, zoom, renderMs, modifier): debug mode's on-screen readout.
// - line(text): one monospaced line.
