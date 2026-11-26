package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.Category
import androidx.compose.material.icons.outlined.CreateNewFolder
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.material.icons.outlined.Texture
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors
import org.json.JSONArray

/** What the Layers panel can ask of the document. */
class LayersActions(
    val card: LayerCardActions,
    val newVector: () -> Unit,
    val newRaster: () -> Unit,
    val newFolder: () -> Unit,
    val deleteCurrent: () -> Unit,
    val move: (Long, Long, String) -> Unit,
)

/**
 * The Layers panel (`uistandards.md` §7.1): the creation row (Vector, Raster, Filter, Texture),
 * the compositing stack topmost first as cards ([LayerCard]), which a card's grip drags above,
 * below or into another (a folder's middle), and the footer with Delete, the hint and Add Folder. Filter and Texture layers have no engine object behind them yet and are shown
 * disabled.
 */
@Composable
fun LayersPanel(rows: JSONArray, expanded: Boolean, onToggle: () -> Unit, actions: LayersActions, modifier: Modifier = Modifier) {
    FloatingPanel("Layers", "%02d".format(count(rows)), expanded, onToggle, modifier) {
        Row(Modifier.padding(horizontal = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Create(Icons.Outlined.Category, "Vector", Modifier.weight(1f), actions.newVector)
            Create(Icons.Outlined.Layers, "Raster", Modifier.weight(1f), actions.newRaster)
            Create(Icons.Outlined.AutoAwesome, "Filter", Modifier.weight(1f), null)
            Create(Icons.Outlined.Texture, "Texture", Modifier.weight(1f), null)
        }
        Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp)) {
            Text("COMPOSITING STACK", color = EtcColors.Soft, fontSize = 10.sp, modifier = Modifier.weight(1f))
            Text("TOP ↓ BOTTOM", color = EtcColors.Muted, fontSize = 10.sp)
        }
        val bounds = remember { mutableMapOf<Long, Pair<Rect, Boolean>>() }
        var dragging by remember { mutableStateOf<Long?>(null) }
        var hint by remember { mutableStateOf<DropHint?>(null) }
        val drag = CardDrag(
            onBounds = { id, r, folder -> bounds[id] = r to folder },
            onStart = { id, _ -> dragging = id },
            onMove = { y -> hint = dragging?.let { dropAt(bounds, it, y) } },
            onEnd = {
                val from = dragging
                val to = hint
                dragging = null
                hint = null
                if (from != null && to != null) actions.move(from, to.id, to.place)
            },
            hint = hint,
        )
        Column(
            Modifier.heightIn(max = 520.dp).verticalScroll(rememberScrollState()).padding(horizontal = 10.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            for (i in 0 until rows.length()) rows.optJSONObject(i)?.let { LayerCard(it, 0, actions.card, drag) }
        }
        Row(Modifier.fillMaxWidth().padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
            FooterButton(Icons.Outlined.Delete, "Delete", actions.deleteCurrent)
            Text("Drag rows to reorder · filters cascade downward", color = EtcColors.Soft, fontSize = 10.sp, modifier = Modifier.weight(1f).padding(horizontal = 8.dp))
            FooterButton(Icons.Outlined.CreateNewFolder, "Add folder", actions.newFolder)
        }
    }
}

/**
 * Where a card dragged to [y] (root coordinates) would go: above a card when over its upper part,
 * below when over its lower part, and inside a folder when over its middle third. Not onto itself.
 */
private fun dropAt(bounds: Map<Long, Pair<Rect, Boolean>>, dragged: Long, y: Float): DropHint? {
    val (id, value) = bounds.entries.filter { it.key != dragged && y >= it.value.first.top && y <= it.value.first.bottom }.minByOrNull { it.value.first.height }?.toPair() ?: return null
    val (r, folder) = value
    val t = (y - r.top) / r.height.coerceAtLeast(1f)
    val place = when {
        folder && t in 0.33f..0.67f -> "inside"
        t < 0.5f -> "above"
        else -> "below"
    }
    return DropHint(id, place)
}

/** How many rows the stack has, folders' rows included. */
private fun count(rows: JSONArray): Int = (0 until rows.length()).sumOf { i ->
    val r = rows.optJSONObject(i)
    1 + (r?.optJSONArray("children")?.let { count(it) } ?: 0)
}

/** A creation row button; disabled when [onClick] is null. */
@Composable
private fun Create(icon: ImageVector, label: String, modifier: Modifier, onClick: (() -> Unit)?) {
    val enabled = onClick != null
    Row(
        modifier
            .border(1.dp, EtcColors.Line, RoundedCornerShape(10.dp))
            .clickable(enabled = enabled) { onClick?.invoke() }
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(icon, null, tint = if (enabled) EtcColors.Text else EtcColors.Soft, modifier = Modifier.size(15.dp))
        Text(" $label", color = if (enabled) EtcColors.Text else EtcColors.Soft, fontSize = 12.sp)
    }
}

/** A footer icon button. */
@Composable
private fun FooterButton(icon: ImageVector, description: String, onClick: () -> Unit) {
    androidx.compose.foundation.layout.Box(
        Modifier.size(34.dp).border(1.dp, EtcColors.Line, RoundedCornerShape(9.dp)).clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) { Icon(icon, description, tint = EtcColors.Text, modifier = Modifier.size(18.dp)) }
}

// GLOSSARY
// - LayersActions: what the panel asks of the document (cards, new vector/raster layer, folder, delete, move).
// - LayersPanel(rows, expanded, onToggle, actions, modifier): the Layers panel.
// - dropAt(bounds, dragged, y): where a dragged card would go.
// - count(rows): the rows, folders' included.
// - Create(icon, label, modifier, onClick): a creation row button.
// - FooterButton(icon, description, onClick): a footer icon button.
