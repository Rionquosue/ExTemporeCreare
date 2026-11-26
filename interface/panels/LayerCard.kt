package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ChevronRight
import androidx.compose.material.icons.outlined.DragIndicator
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.material.icons.outlined.Menu
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors
import org.json.JSONObject

/** The blend modes the cards offer (`uistandards.md` §7.2), as (engine name, label). */
val BLEND_MODES = listOf(
    "Normal" to "Normal", "Multiply" to "Multiply", "Screen" to "Screen", "Overlay" to "Overlay", "SoftLight" to "Soft Light",
    "HardLight" to "Hard Light", "ColorDodge" to "Color Dodge", "ColorBurn" to "Color Burn", "Darken" to "Darken", "Lighten" to "Lighten",
    "Difference" to "Difference", "Hue" to "Hue", "Saturation" to "Saturation", "Color" to "Color", "Luminosity" to "Luminosity",
)

/** What a card can ask of the document. */
class LayerCardActions(
    val select: (Long) -> Unit,
    val setVisible: (Long, Boolean) -> Unit,
    val setOpacity: (Long, Int) -> Unit,
    val setBlend: (Long, String) -> Unit,
)

/**
 * One row of the stack as a card (`uistandards.md` §7.1): a menu glyph, the name, its type chip,
 * visibility and a drag grip; below, the type icon, blend mode, opacity (a percentage with its
 * slider) and fill. A folder has a chevron that shows its rows (indented, [depth]). The current
 * layer's card is gold-bordered. Fill and the mask control have no engine property behind them yet
 * and are shown disabled; reordering by the grip is not wired yet.
 */
@Composable
fun LayerCard(row: JSONObject, depth: Int, actions: LayerCardActions) {
    val id = row.optLong("id")
    val kind = row.optString("kind")
    val folder = kind == "folder"
    var open by remember(id) { mutableStateOf(true) }
    val selected = row.optBoolean("current")
    Column(
        Modifier
            .fillMaxWidth()
            .padding(start = (depth * 14).dp)
            .background(if (selected) EtcColors.Panel2 else EtcColors.Panel, RoundedCornerShape(12.dp))
            .border(1.dp, if (selected) EtcColors.Gold else EtcColors.LineSoft, RoundedCornerShape(12.dp))
            .clickable { actions.select(id) }
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Icon(Icons.Outlined.Menu, null, tint = EtcColors.Muted, modifier = Modifier.size(16.dp))
            if (folder) {
                Icon(
                    if (open) Icons.Outlined.ExpandMore else Icons.Outlined.ChevronRight,
                    if (open) "Collapse" else "Expand",
                    tint = EtcColors.Muted,
                    modifier = Modifier.size(18.dp).clickable { open = !open },
                )
            }
            Text(row.optString("name"), color = EtcColors.Text, fontSize = 13.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
            Chip(chipLabel(kind, row))
            val visible = row.optBoolean("visible", true)
            Icon(
                if (visible) Icons.Outlined.Visibility else Icons.Outlined.VisibilityOff,
                if (visible) "Hide" else "Show",
                tint = if (visible) EtcColors.Muted else EtcColors.Soft,
                modifier = Modifier.size(18.dp).clickable { actions.setVisible(id, !visible) },
            )
            Icon(Icons.Outlined.DragIndicator, null, tint = EtcColors.Soft, modifier = Modifier.size(16.dp))
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Box(Modifier.size(30.dp).border(1.dp, EtcColors.Line, RoundedCornerShape(8.dp)), contentAlignment = Alignment.Center) {
                Icon(iconOf(kind), null, tint = EtcColors.Muted, modifier = Modifier.size(16.dp))
            }
            BlendPicker(row.optString("blend", "Normal")) { actions.setBlend(id, it) }
            PercentSlider("Opacity", row.optDouble("opacity", 100.0).toFloat(), Modifier.weight(1f)) { actions.setOpacity(id, it) }
            PercentSlider("Fill", 100f, Modifier.weight(1f), enabled = false) {}
        }
    }
    if (folder && open) {
        val children = row.optJSONArray("children")
        for (i in 0 until (children?.length() ?: 0)) children?.optJSONObject(i)?.let { LayerCard(it, depth + 1, actions) }
    }
}

/** A card's type chip: Folder · count, Raster or Vector. */
private fun chipLabel(kind: String, row: JSONObject): String = when (kind) {
    "folder" -> "Folder · ${row.optJSONArray("children")?.length() ?: 0}"
    "raster" -> "Raster"
    "guide" -> "Guide"
    else -> "Vector"
}

/** A row type's icon. */
private fun iconOf(kind: String): ImageVector = when (kind) {
    "folder" -> Icons.Outlined.Folder
    "raster" -> Icons.Outlined.Image
    else -> Icons.Outlined.Layers
}

/** A small outlined label. */
@Composable
private fun Chip(text: String) {
    Text(
        text,
        color = EtcColors.Muted,
        fontSize = 10.sp,
        modifier = Modifier.border(1.dp, EtcColors.Line, RoundedCornerShape(6.dp)).padding(horizontal = 6.dp, vertical = 2.dp),
    )
}

/** The blend mode dropdown. */
@Composable
private fun BlendPicker(current: String, onPick: (String) -> Unit) {
    var open by remember { mutableStateOf(false) }
    Column {
        Text("Blend mode", color = EtcColors.Soft, fontSize = 10.sp)
        Box {
            Text(
                BLEND_MODES.firstOrNull { it.first == current }?.second ?: current,
                color = EtcColors.Text,
                fontSize = 12.sp,
                modifier = Modifier.width(96.dp).border(1.dp, EtcColors.Line, RoundedCornerShape(6.dp)).clickable { open = true }.padding(horizontal = 8.dp, vertical = 5.dp),
            )
            DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                for ((name, label) in BLEND_MODES) DropdownMenuItem(text = { Text(label) }, onClick = { open = false; onPick(name) })
            }
        }
    }
}

/** A labelled 0–100 % slider with its value; the value is set when the drag ends. */
@Composable
private fun PercentSlider(label: String, value: Float, modifier: Modifier, enabled: Boolean = true, onSet: (Int) -> Unit) {
    var v by remember(value) { mutableFloatStateOf(value) }
    Column(modifier.semantics { contentDescription = "$label ${v.toInt()} percent" }) {
        Text(label, color = EtcColors.Soft, fontSize = 10.sp)
        Row(verticalAlignment = Alignment.CenterVertically) {
            Slider(
                value = v,
                onValueChange = { v = it },
                onValueChangeFinished = { onSet(v.toInt()) },
                valueRange = 0f..100f,
                enabled = enabled,
                colors = SliderDefaults.colors(thumbColor = EtcColors.Gold, activeTrackColor = EtcColors.Gold2, inactiveTrackColor = EtcColors.Line),
                modifier = Modifier.weight(1f),
            )
            Text("${v.toInt()}%", color = if (enabled) EtcColors.Text else EtcColors.Soft, fontSize = 11.sp)
        }
    }
}

// GLOSSARY
// - BLEND_MODES: the cards' blend modes (engine name, label).
// - LayerCardActions: what a card asks of the document (select, visibility, opacity, blend).
// - LayerCard(row, depth, actions): one row as a card, a folder's rows below it.
// - chipLabel(kind, row), iconOf(kind): a row's chip and icon.
// - Chip(text): a small outlined label.
// - BlendPicker(current, onPick): the blend mode dropdown.
// - PercentSlider(label, value, modifier, enabled, onSet): a 0–100 % slider with its value.
