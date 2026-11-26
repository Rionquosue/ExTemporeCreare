package com.extemporecreare.app.ui.document

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.extemporecreare.app.components.documents.DocumentSettings
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingChoice
import com.extemporecreare.app.ui.tools.SettingNote
import com.extemporecreare.app.ui.tools.SettingNumberField
import com.extemporecreare.app.ui.tools.formatNumber
import kotlin.math.roundToLong

/**
 * New Document (`uistandards.md` §5.3): name, canvas mode (finite or infinite), a preset that fills
 * the editable settings, width and height in px, in, cm or mm (finite only) with an orientation
 * swap, resolution with the resulting pixel size, colour mode (what the engine supports; 8 bits
 * per channel), and background; Create or Cancel. An infinite canvas has no outer size: export
 * slices give its finite output. The dialog scrolls on small screens.
 */
@Composable
fun NewDocumentDialog(initial: DocumentSettings, onCreate: (DocumentSettings) -> Unit, onCancel: () -> Unit) {
    var s by remember { mutableStateOf(initial) }
    Dialog(onDismissRequest = onCancel) {
        Column(
            Modifier
                .widthIn(max = 520.dp)
                .background(EtcColors.Shell, RoundedCornerShape(16.dp))
                .border(1.dp, EtcColors.Line, RoundedCornerShape(16.dp))
                .verticalScroll(rememberScrollState())
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("New Document", color = EtcColors.Text, fontSize = 18.sp)
            OutlinedTextField(s.name, { s = s.copy(name = it) }, label = { Text("Name") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            PresetPicker { s = it(s) }
            SettingChoice("Canvas", listOf(false to "Finite", true to "Infinite"), s.infinite) { s = s.copy(infinite = it) }
            if (s.infinite) {
                SettingNote("An infinite canvas has no outer size. Export slices (the Export tool) give it a finite output.")
            } else {
                SettingChoice("Units", listOf("px" to "px", "in" to "in", "cm" to "cm", "mm" to "mm"), s.units) { u -> s = convertUnits(s, u) }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    SettingNumberField("Width", s.width, s.units, { if (it > 0) s = s.copy(width = it) })
                    SettingNumberField("Height", s.height, s.units, { if (it > 0) s = s.copy(height = it) })
                    SettingButton("Swap", { s = s.copy(width = s.height, height = s.width) })
                }
            }
            SettingNumberField("Resolution", s.ppi, "ppi", { if (it in 1.0..2400.0) s = s.copy(ppi = it) })
            if (!s.infinite) {
                val (w, h) = s.pixels
                SettingNote("Result: ${w.roundToLong()} × ${h.roundToLong()} px at ${formatNumber(s.ppi)} ppi")
            }
            SettingChoice("Colour", listOf("rgb" to "RGB", "cmyk" to "CMYK"), s.colorMode) { s = s.copy(colorMode = it) }
            SettingNote("8 bits per channel (the engine's).")
            BackgroundPicker(s.background) { s = s.copy(background = it) }
            val (w, h) = s.pixels
            val tooLarge = !s.infinite && (w > DocumentSettings.MAX_SIDE || h > DocumentSettings.MAX_SIDE)
            if (tooLarge) SettingNote("Too large: the most a side can be is ${DocumentSettings.MAX_SIDE.roundToLong()} px.", EtcColors.Gold2)
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End)) {
                SettingButton("Cancel", onCancel)
                SettingButton("Create", { onCreate(s) }, primary = true, enabled = !tooLarge && s.name.isNotBlank())
            }
        }
    }
}

/** [s] in units [u], its width and height converted so the size stays the same. */
private fun convertUnits(s: DocumentSettings, u: String): DocumentSettings {
    val (w, h) = s.pixels
    val back = { px: Double -> when (u) { "in" -> px / s.ppi; "cm" -> px / s.ppi * 2.54; "mm" -> px / s.ppi * 25.4; else -> px } }
    return s.copy(units = u, width = back(w), height = back(h))
}

/** The preset dropdown. */
@Composable
private fun PresetPicker(onPick: ((DocumentSettings) -> DocumentSettings) -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box {
        SettingButton("Preset…", { open = true })
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            for ((label, apply) in DOCUMENT_PRESETS) DropdownMenuItem(text = { Text(label) }, onClick = { open = false; onPick(apply) })
        }
    }
}

/** Transparent, white, or a solid colour (`#rrggbb`). */
@Composable
private fun BackgroundPicker(current: String, onPick: (String) -> Unit) {
    val kind = if (current.startsWith("#")) "colour" else current
    SettingChoice("Background", listOf("transparent" to "Transparent", "white" to "White", "colour" to "Colour"), kind) {
        onPick(if (it == "colour") "#E7DDC6" else it)
    }
    if (kind == "colour") {
        var text by remember(current) { mutableStateOf(current) }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            OutlinedTextField(text, { text = it; if (Regex("#[0-9a-fA-F]{6}").matches(it)) onPick(it) }, label = { Text("Colour (#rrggbb)") }, singleLine = true)
            Box(
                Modifier.padding(4.dp).widthIn(min = 32.dp).background(runCatching { androidx.compose.ui.graphics.Color(android.graphics.Color.parseColor(current)) }.getOrDefault(EtcColors.Panel2), RoundedCornerShape(6.dp))
                    .padding(14.dp),
            )
        }
    }
}

// GLOSSARY
// - NewDocumentDialog(initial, onCreate, onCancel): the New Document dialog.
// - convertUnits(s, u): settings in other units, the same size.
// - PresetPicker(onPick): the preset dropdown.
// - BackgroundPicker(current, onPick): transparent, white or a colour.
