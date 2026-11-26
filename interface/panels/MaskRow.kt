package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Contrast
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors
import org.json.JSONObject

/** What the mask controls ask of the document (`uistandards.md` §7.1, as in Photoshop). */
class MaskActions(
    /** Give layer [id] a mask: `reveal` (white), `hide` (black) or `selection`. */
    val add: (Long, String) -> Unit,
    /** Replace layer [id]'s mask with the selection. */
    val fromSelection: (Long) -> Unit,
    /** Start or stop painting layer [id]'s mask. */
    val paint: (Long) -> Unit,
    val invert: (Long) -> Unit,
    val setEnabled: (Long, Boolean) -> Unit,
    val delete: (Long) -> Unit,
    /** Replace layer [id]'s mask with an all-white (`reveal`) or all-black (`hide`) one. */
    val reset: (Long, String) -> Unit,
)

/** What the mask controls need to know: how many objects are selected and whose mask is being painted. */
data class MaskContext(val selected: Int, val editing: Long?)

/**
 * The card's mask button: without a mask, a tap adds one, from the selection when something is
 * selected (as Photoshop's Add Mask), else Reveal All; holding it offers Reveal All, Hide All and
 * From Selection. With a mask it shows lit.
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun MaskButton(row: JSONObject, masks: MaskContext, actions: MaskActions) {
    val id = row.optLong("id")
    val has = !row.isNull("mask") && row.optJSONObject("mask") != null
    var menu by remember { mutableStateOf(false) }
    Box {
        Box(
            Modifier.size(30.dp)
                .border(1.dp, if (has) EtcColors.Gold else EtcColors.LineSoft, RoundedCornerShape(8.dp))
                .semantics { contentDescription = if (has) "Mask" else "Add mask" }
                .combinedClickable(
                    onClick = { if (has) menu = true else actions.add(id, if (masks.selected > 0) "selection" else "reveal") },
                    onLongClick = { menu = true },
                ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(Icons.Outlined.Contrast, null, tint = if (has) EtcColors.Gold else EtcColors.Muted, modifier = Modifier.size(16.dp))
        }
        MaskMenu(menu, { menu = false }, row, masks, actions)
    }
}

/**
 * A layer's mask as its sublayer: a black or white swatch (its background, inverted when the mask
 * is), "Mask" and its state (Painting, Inverted, Disabled). Tapping it with something selected
 * makes the mask the selection; with nothing selected it starts or stops painting the mask (white
 * shows the layer, black hides it, greys partly show it). Its menu holds the quick operations.
 */
@Composable
fun MaskRow(row: JSONObject, masks: MaskContext, actions: MaskActions) {
    val id = row.optLong("id")
    val mask = row.optJSONObject("mask") ?: return
    val editing = masks.editing == id
    val enabled = mask.optBoolean("enabled", true)
    val inverted = mask.optBoolean("inverted")
    val white = (mask.optString("background") == "white") != inverted
    var menu by remember { mutableStateOf(false) }
    Row(
        Modifier.fillMaxWidth()
            .padding(start = 14.dp)
            .background(if (editing) EtcColors.Panel2 else Color.Transparent, RoundedCornerShape(8.dp))
            .border(1.dp, if (editing) EtcColors.Gold else EtcColors.LineSoft, RoundedCornerShape(8.dp))
            .clickable { if (masks.selected > 0 && !editing) actions.fromSelection(id) else actions.paint(id) }
            .padding(horizontal = 8.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(Modifier.size(22.dp).background(if (white) Color.White else Color.Black, RoundedCornerShape(4.dp)).border(1.dp, EtcColors.Line, RoundedCornerShape(4.dp)))
        Text("Mask", color = if (enabled) EtcColors.Text else EtcColors.Soft, fontSize = 12.sp)
        val status = listOfNotNull("Painting".takeIf { editing }, "Inverted".takeIf { inverted }, "Disabled".takeIf { !enabled }).joinToString(" · ")
        Text(
            status.ifEmpty { if (masks.selected > 0) "Tap: use selection" else "Tap: paint" },
            color = EtcColors.Soft,
            fontSize = 10.sp,
            modifier = Modifier.weight(1f),
        )
        Box {
            Icon(Icons.Outlined.MoreVert, "Mask options", tint = EtcColors.Muted, modifier = Modifier.size(20.dp).clickable { menu = true })
            MaskMenu(menu, { menu = false }, row, masks, actions)
        }
    }
}

/** The mask's quick operations: without a mask, the ways to add one; with one, the rest. */
@Composable
private fun MaskMenu(open: Boolean, close: () -> Unit, row: JSONObject, masks: MaskContext, actions: MaskActions) {
    val id = row.optLong("id")
    val mask = row.optJSONObject("mask")
    DropdownMenu(expanded = open, onDismissRequest = close) {
        if (mask == null) {
            DropdownMenuItem(text = { Text("Reveal All") }, onClick = { close(); actions.add(id, "reveal") })
            DropdownMenuItem(text = { Text("Hide All") }, onClick = { close(); actions.add(id, "hide") })
            DropdownMenuItem(text = { Text("From Selection") }, onClick = { close(); actions.add(id, "selection") }, enabled = masks.selected > 0)
        } else {
            val enabled = mask.optBoolean("enabled", true)
            DropdownMenuItem(text = { Text(if (masks.editing == id) "Stop Painting Mask" else "Paint Mask") }, onClick = { close(); actions.paint(id) })
            DropdownMenuItem(text = { Text("Mask from Selection") }, onClick = { close(); actions.fromSelection(id) }, enabled = masks.selected > 0)
            DropdownMenuItem(text = { Text("Invert") }, onClick = { close(); actions.invert(id) })
            DropdownMenuItem(text = { Text(if (enabled) "Disable Mask" else "Enable Mask") }, onClick = { close(); actions.setEnabled(id, !enabled) })
            DropdownMenuItem(text = { Text("Reset to Reveal All") }, onClick = { close(); actions.reset(id, "reveal") })
            DropdownMenuItem(text = { Text("Reset to Hide All") }, onClick = { close(); actions.reset(id, "hide") })
            DropdownMenuItem(text = { Text("Delete Mask") }, onClick = { close(); actions.delete(id) })
        }
    }
}

// GLOSSARY
// - MaskActions(add, fromSelection, paint, invert, setEnabled, delete, reset): what the mask controls ask of the document.
// - MaskContext(selected, editing): how many objects are selected and whose mask is being painted.
// - MaskButton(row, masks, actions): the card's add-mask / mask button.
// - MaskRow(row, masks, actions): a layer's mask as its sublayer.
// - MaskMenu(open, close, row, masks, actions): the mask's quick operations.
