package com.extemporecreare.app.ui.color

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.color.ColorLibrary
import com.extemporecreare.app.components.color.ColorMath
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingChoice

/**
 * The colour workspace (`uistandards.md` §8.1), opened near the dial: a saturation/value square and
 * a hue bar for the colour being edited (primary or alternate), its `#rrggbb` and R, G, B values to
 * type, the saved swatches (tap: use, long press: remove, + saves the colour), the recent colours,
 * and the mixing pad. [onSet] gives the edited colour ("primary"/"alternate", `#rrggbb`).
 */
@Composable
fun ColorWorkspace(primary: String?, alternate: String?, library: ColorLibrary, onSet: (String, String) -> Unit, onClose: () -> Unit, modifier: Modifier = Modifier) {
    var target by remember { mutableStateOf("primary") }
    var libraryVersion by remember { mutableIntStateOf(0) }
    val current = (if (target == "primary") primary else alternate) ?: "#000000"
    val hsv = remember(current) { ColorMath.toHsv(ColorMath.parse(current) ?: intArrayOf(0, 0, 0)) }
    var hue by remember(current) { mutableFloatStateOf(hsv[0]) }
    val set = { hex: String -> onSet(target, hex) }
    Column(
        modifier
            .width(320.dp)
            .background(EtcColors.Panel, RoundedCornerShape(16.dp))
            .border(1.dp, EtcColors.Line, RoundedCornerShape(16.dp))
            .verticalScroll(rememberScrollState())
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("Colour", color = EtcColors.Text, fontSize = 14.sp, modifier = Modifier.weight(1f))
            SettingButton("Close", onClose)
        }
        SettingChoice("Editing", listOf("primary" to "Primary", "alternate" to "Alternate"), target) { target = it }
        SvSquare(hue, hsv[1], hsv[2]) { s, v -> set(ColorMath.fromHsv(hue, s, v)) }
        HueBar(hue) { h -> hue = h; set(ColorMath.fromHsv(h, hsv[1].coerceAtLeast(0.01f), hsv[2].coerceAtLeast(0.01f))) }
        NumericInput(current, set)
        Text("Swatches", color = EtcColors.Muted, fontSize = 11.sp)
        val swatches = remember(libraryVersion) { library.swatches() }
        ChipRow(swatches, onTap = set, onLongPress = { library.removeSwatch(it); libraryVersion++ }) {
            Box(
                Modifier.size(26.dp).border(1.dp, EtcColors.Line, CircleShape).pointerInput(current) { detectTapGestures { library.addSwatch(current); libraryVersion++ } }
                    .semantics { contentDescription = "Save the colour as a swatch" },
                contentAlignment = Alignment.Center,
            ) { Text("+", color = EtcColors.Text, fontSize = 14.sp) }
        }
        Text("Recent", color = EtcColors.Muted, fontSize = 11.sp)
        ChipRow(remember(current, libraryVersion) { library.history() }, onTap = set, onLongPress = {}) {}
        MixingPad(primary, alternate) { onSet("primary", it) }
    }
}

/** The saturation/value square for [hue], its marker at [s], [v]; dragging sets them. */
@Composable
private fun SvSquare(hue: Float, s: Float, v: Float, onPick: (Float, Float) -> Unit) {
    val pure = Color(android.graphics.Color.parseColor(ColorMath.fromHsv(hue, 1f, 1f)))
    Canvas(
        Modifier.fillMaxWidth().height(160.dp).pointerInput(hue) {
            val pick = { p: Offset -> onPick((p.x / size.width).coerceIn(0f, 1f), 1f - (p.y / size.height).coerceIn(0f, 1f)) }
            detectDragGestures(onDragStart = pick) { change, _ -> pick(change.position) }
        }.pointerInput(hue) { detectTapGestures { p -> onPick((p.x / size.width).coerceIn(0f, 1f), 1f - (p.y / size.height).coerceIn(0f, 1f)) } },
    ) {
        drawRect(Brush.horizontalGradient(listOf(Color.White, pure)))
        drawRect(Brush.verticalGradient(listOf(Color.Transparent, Color.Black)))
        drawCircle(Color.White, radius = 7.dp.toPx(), center = Offset(s * size.width, (1 - v) * size.height), style = Stroke(2.dp.toPx()))
    }
}

/** The hue bar, its marker at [hue]; dragging sets it. */
@Composable
private fun HueBar(hue: Float, onPick: (Float) -> Unit) {
    val stops = (0..6).map { Color(android.graphics.Color.parseColor(ColorMath.fromHsv(it * 60f % 360f, 1f, 1f))) }
    Canvas(
        Modifier.fillMaxWidth().height(18.dp).pointerInput(Unit) {
            val pick = { p: Offset -> onPick((p.x / size.width).coerceIn(0f, 1f) * 359.9f) }
            detectDragGestures(onDragStart = pick) { change, _ -> pick(change.position) }
        },
    ) {
        drawRect(Brush.horizontalGradient(stops))
        val x = hue / 360f * size.width
        drawLine(Color.White, Offset(x, 0f), Offset(x, size.height), strokeWidth = 3.dp.toPx())
    }
}

/** `#rrggbb` and R, G, B fields; a valid entry sets the colour. */
@Composable
private fun NumericInput(current: String, set: (String) -> Unit) {
    var text by remember(current) { mutableStateOf(current) }
    val rgb = ColorMath.parse(current) ?: intArrayOf(0, 0, 0)
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
        OutlinedTextField(text, { text = it; if (ColorMath.parse(it) != null) set(if (it.startsWith("#")) it.lowercase() else "#" + it.lowercase()) }, label = { Text("Hex") }, singleLine = true, modifier = Modifier.width(110.dp))
        for ((i, label) in listOf("R", "G", "B").withIndex()) {
            var part by remember(current) { mutableStateOf(rgb[i].toString()) }
            OutlinedTextField(part, { t ->
                part = t
                t.toIntOrNull()?.takeIf { it in 0..255 }?.let { n -> val c = rgb.copyOf(); c[i] = n; set(ColorMath.hex(c[0], c[1], c[2])) }
            }, label = { Text(label) }, singleLine = true, modifier = Modifier.width(58.dp))
        }
    }
}

/** A wrapping row of colour chips (tap, long press) and [trailing]. */
@Composable
private fun ChipRow(colors: List<String>, onTap: (String) -> Unit, onLongPress: (String) -> Unit, trailing: @Composable () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        val rows = (colors.map { it as String? } + listOf(null)).chunked(8)
        for (row in rows) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                for (hex in row) {
                    if (hex == null) {
                        trailing()
                    } else {
                        Box(
                            Modifier.size(26.dp).background(Color(android.graphics.Color.parseColor(hex)), CircleShape).border(1.dp, EtcColors.Line, CircleShape)
                                .pointerInput(hex) { detectTapGestures(onTap = { onTap(hex) }, onLongPress = { onLongPress(hex) }) }
                                .semantics { contentDescription = "Colour $hex" },
                        )
                    }
                }
            }
        }
    }
}

// GLOSSARY
// - ColorWorkspace(primary, alternate, library, onSet, onClose, modifier): the colour workspace.
// - SvSquare(hue, s, v, onPick): saturation/value square.
// - HueBar(hue, onPick): the hue bar.
// - NumericInput(current, set): hex and R, G, B entry.
// - ChipRow(colors, onTap, onLongPress, trailing): rows of colour chips.
