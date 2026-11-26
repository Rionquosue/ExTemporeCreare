package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ArrowDropDown
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.ChevronRight
import androidx.compose.material.icons.outlined.Contrast
import androidx.compose.material.icons.outlined.DragIndicator
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.material.icons.outlined.Menu
import androidx.compose.material.icons.outlined.Texture
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
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
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

/**
 * The modifier menu's layer styles (`uistandards.md` §7.2), as (label, engine command and params
 * to apply, or null where the engine has no matching operation, shown unavailable).
 */
val LAYER_STYLES: List<Pair<String, Pair<String, JSONObject>?>> = listOf(
    "Stroke" to ("effect.apply" to JSONObject().put("effect", "style.stroke")),
    "Bevel & Emboss" to ("effect.apply" to JSONObject().put("effect", "style.bevelEmboss")),
    "Color Overlay" to ("effect.apply" to JSONObject().put("effect", "style.colorOverlay")),
    "Gradient Overlay" to ("effect.apply" to JSONObject().put("effect", "style.gradientOverlay")),
    "Drop Shadow" to ("effect.apply" to JSONObject().put("effect", "stylize.dropShadow")),
    "Outer Glow" to ("effect.apply" to JSONObject().put("effect", "stylize.outerGlow")),
)

/** What a card can ask of the document. */
class LayerCardActions(
    val select: (Long) -> Unit,
    val setVisible: (Long, Boolean) -> Unit,
    val setOpacity: (Long, Int) -> Unit,
    val setFill: (Long, Int) -> Unit,
    /** A texture layer's scale, in percent (100: one image pixel a point). */
    val setTextureScale: (Long, Int) -> Unit,
    val setBlend: (Long, String) -> Unit,
    val addStyle: (Long, String, JSONObject) -> Unit,
    val mask: MaskActions,
)

/** Where a dragged card would go: beside or inside the card [id]. */
data class DropHint(val id: Long, val place: String)

/** The drag of cards by their grips, shared by the panel's cards. */
class CardDrag(
    val onBounds: (Long, Rect, Boolean) -> Unit,
    val onStart: (Long, Float) -> Unit,
    val onMove: (Float) -> Unit,
    val onEnd: () -> Unit,
    val hint: DropHint?,
)

/**
 * One row of the stack as a card (`uistandards.md` §7.1): the modifier menu, the name, its type
 * chip, visibility and the drag grip; below, the type icon, blend mode, Opacity (typed percentage
 * and a larger slider from its arrow, kept in step), Fill (as Photoshop's: the content's opacity,
 * apart from the layer styles) and the mask button ([MaskButton]); a layer's mask shows below as its
 * sublayer ([MaskRow]). A folder has a chevron that shows its rows,
 * indented. The current layer's card is gold-bordered; a drop line shows where a dragged card would go.
 */
@Composable
fun LayerCard(row: JSONObject, depth: Int, actions: LayerCardActions, drag: CardDrag, folders: SnapshotStateMap<Long, Boolean>, masks: MaskContext) {
    val id = row.optLong("id")
    val kind = row.optString("kind")
    val folder = kind == "folder"
    val open = folders[id] ?: true
    val selected = row.optBoolean("current")
    val hint = drag.hint?.takeIf { it.id == id }
    Column(
        Modifier
            .fillMaxWidth()
            .padding(start = (depth * 14).dp)
            .onGloballyPositioned { drag.onBounds(id, it.boundsInRoot(), folder) }
            .background(if (selected) EtcColors.Panel2 else EtcColors.Panel, RoundedCornerShape(12.dp))
            .border(if (hint?.place == "inside") 2.dp else 1.dp, if (hint?.place == "inside" || selected) EtcColors.Gold else EtcColors.LineSoft, RoundedCornerShape(12.dp))
            .clickable { actions.select(id) }
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        if (hint?.place == "above") DropLine()
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            StyleMenu { label, command, params -> actions.addStyle(id, command, params.put("_label", label)) }
            if (folder) {
                Icon(
                    if (open) Icons.Outlined.ExpandMore else Icons.Outlined.ChevronRight,
                    if (open) "Collapse" else "Expand",
                    tint = EtcColors.Muted,
                    modifier = Modifier.size(22.dp).clickable { folders[id] = !open },
                )
            }
            Text(row.optString("name"), color = EtcColors.Text, fontSize = 13.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
            Chip(chipLabel(kind, row))
            val visible = row.optBoolean("visible", true)
            Icon(
                if (visible) Icons.Outlined.Visibility else Icons.Outlined.VisibilityOff,
                if (visible) "Hide" else "Show",
                tint = if (visible) EtcColors.Muted else EtcColors.Soft,
                modifier = Modifier.size(22.dp).clickable { actions.setVisible(id, !visible) },
            )
            var gripY by remember { mutableFloatStateOf(0f) }
            Icon(
                Icons.Outlined.DragIndicator,
                "Drag to reorder",
                tint = EtcColors.Soft,
                modifier = Modifier.size(22.dp)
                    .onGloballyPositioned { gripY = it.boundsInRoot().center.y }
                    .pointerInput(id) {
                        var y = 0f
                        detectDragGestures(
                            onDragStart = { y = gripY; drag.onStart(id, y) },
                            onDragEnd = { drag.onEnd() },
                            onDragCancel = { drag.onEnd() },
                        ) { change, delta ->
                            change.consume()
                            y += delta.y
                            drag.onMove(y)
                        }
                    },
            )
        }
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Box(Modifier.size(30.dp).border(1.dp, EtcColors.Line, RoundedCornerShape(8.dp)), contentAlignment = Alignment.Center) {
                Icon(iconOf(kind), null, tint = EtcColors.Muted, modifier = Modifier.size(16.dp))
            }
            // A filter layer filters what is below it, mixed at its Opacity; it has no blend, Fill or mask of its own.
            val filter = kind == "filter"
            if (!filter) BlendPicker(row.optString("blend", "Normal")) { actions.setBlend(id, it) }
            PercentControl("Opacity", row.optDouble("opacity", 100.0).toInt(), Modifier.weight(1f)) { actions.setOpacity(id, it) }
            if (!filter) {
                PercentControl("Fill", row.optDouble("fill", 100.0).toInt(), Modifier.weight(1f)) { actions.setFill(id, it) }
                MaskButton(row, masks, actions.mask)
            }
        }
        if (kind == "texture") {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PercentControl("Scale", (row.optDouble("scale", 1.0) * 100).toInt(), Modifier.weight(1f), max = 1000) { actions.setTextureScale(id, it.coerceAtLeast(1)) }
            }
        }
        MaskRow(row, masks, actions.mask)
        if (hint?.place == "below") DropLine()
    }
    if (folder && open) {
        val children = row.optJSONArray("children")
        for (i in 0 until (children?.length() ?: 0)) children?.optJSONObject(i)?.let { LayerCard(it, depth + 1, actions, drag, folders, masks) }
    }
}

/** The drop indicator line. */
@Composable
private fun DropLine() {
    Box(Modifier.fillMaxWidth().height(3.dp).background(EtcColors.Gold, RoundedCornerShape(2.dp)))
}

/** A card's type chip: Folder · count, Raster or Vector. */
private fun chipLabel(kind: String, row: JSONObject): String = when (kind) {
    "folder" -> "Folder · ${row.optJSONArray("children")?.length() ?: 0}"
    "raster" -> "Raster"
    "guide" -> "Guide"
    "texture" -> "Texture"
    "filter" -> "Filter"
    else -> "Vector"
}

/** A row type's icon. */
private fun iconOf(kind: String): ImageVector = when (kind) {
    "folder" -> Icons.Outlined.Folder
    "raster" -> Icons.Outlined.Image
    "texture" -> Icons.Outlined.Texture
    "filter" -> Icons.Outlined.AutoAwesome
    else -> Icons.Outlined.Layers
}

/** A small outlined label. */
@Composable
private fun Chip(text: String) {
    Text(text, color = EtcColors.Muted, fontSize = 10.sp, modifier = Modifier.border(1.dp, EtcColors.Line, RoundedCornerShape(6.dp)).padding(horizontal = 6.dp, vertical = 2.dp))
}

/** The modifier menu: the layer styles, those without an engine operation shown unavailable. */
@Composable
private fun StyleMenu(onPick: (String, String, JSONObject) -> Unit) {
    var open by remember { mutableStateOf(false) }
    Box {
        Icon(Icons.Outlined.Menu, "Layer styles", tint = EtcColors.Muted, modifier = Modifier.size(22.dp).clickable { open = true })
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            for ((label, op) in LAYER_STYLES) {
                DropdownMenuItem(
                    text = { Text(if (op == null) "$label (not available yet)" else label) },
                    enabled = op != null,
                    onClick = { open = false; op?.let { (command, params) -> onPick(label, command, JSONObject(params.toString())) } },
                )
            }
        }
    }
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

/**
 * A percentage property, 0–[max] % (100 by default; `uistandards.md` §7.1): the value typed
 * directly (committed on Done or when the field loses focus), the % sign, and an arrow opening a
 * larger slider that names the property; both show the same value.
 */
@Composable
private fun PercentControl(label: String, value: Int, modifier: Modifier, enabled: Boolean = true, max: Int = 100, onSet: (Int) -> Unit) {
    var v by remember(value) { mutableFloatStateOf(value.toFloat()) }
    var text by remember(value) { mutableStateOf(value.toString()) }
    var open by remember { mutableStateOf(false) }
    val commit = { text.toIntOrNull()?.coerceIn(0, max)?.let { n -> v = n.toFloat(); text = n.toString(); if (n != value) onSet(n) } }
    Column(modifier) {
        Text(label, color = EtcColors.Soft, fontSize = 10.sp)
        Row(
            Modifier.border(1.dp, EtcColors.Line, RoundedCornerShape(6.dp)).padding(start = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicTextField(
                value = text,
                onValueChange = { t -> text = t.filter { it.isDigit() }.take(max.toString().length) },
                enabled = enabled,
                singleLine = true,
                textStyle = TextStyle(color = if (enabled) EtcColors.Text else EtcColors.Soft, fontSize = 12.sp),
                cursorBrush = SolidColor(EtcColors.Gold),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number, imeAction = ImeAction.Done),
                keyboardActions = KeyboardActions(onDone = { commit() }),
                modifier = Modifier.width(30.dp).onFocusChanged { if (!it.isFocused) commit() }.semantics { contentDescription = "$label percent" },
            )
            Text("%", color = EtcColors.Muted, fontSize = 12.sp)
            Box {
                Icon(Icons.Outlined.ArrowDropDown, "$label slider", tint = if (enabled) EtcColors.Text else EtcColors.Soft, modifier = Modifier.size(28.dp).clickable(enabled = enabled) { open = true })
                DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                    Column(Modifier.width(240.dp).padding(horizontal = 14.dp, vertical = 6.dp)) {
                        Text("$label ${v.toInt()}%", color = EtcColors.Text, fontSize = 13.sp)
                        Slider(
                            value = v,
                            onValueChange = { v = it; text = it.toInt().toString() },
                            onValueChangeFinished = { onSet(v.toInt()) },
                            valueRange = 0f..max.toFloat(),
                            colors = SliderDefaults.colors(thumbColor = EtcColors.Gold, activeTrackColor = EtcColors.Gold2, inactiveTrackColor = EtcColors.Line),
                        )
                    }
                }
            }
        }
    }
}

// GLOSSARY
// - BLEND_MODES: the cards' blend modes (engine name, label).
// - LAYER_STYLES: the modifier menu's styles (label, engine command and params, or null when unavailable).
// - LayerCardActions: what a card asks of the document (select, visibility, opacity, Fill, blend, add a style, mask).
// - DropHint: where a dragged card would go. CardDrag: the grip drag shared by the cards.
// - LayerCard(row, depth, actions, drag, folders): one row as a card, a folder's rows below it unless collapsed in folders.
// - DropLine(): the drop indicator.
// - chipLabel(kind, row), iconOf(kind): a row's chip and icon. Chip(text): a small outlined label.
// - StyleMenu(onPick): the modifier menu.
// - BlendPicker(current, onPick): the blend mode dropdown.
// - PercentControl(label, value, modifier, enabled, max, onSet): typed percentage with a slider popover.
