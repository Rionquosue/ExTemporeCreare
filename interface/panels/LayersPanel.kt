package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AutoAwesome
import androidx.compose.material.icons.outlined.Brush
import androidx.compose.material.icons.outlined.Category
import androidx.compose.material.icons.outlined.CreateNewFolder
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.material.icons.outlined.Texture
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors
import org.json.JSONArray

/**
 * What the Layers panel shows that outlives it while Export Settings takes its place (§9.1): its
 * scroll position and which folders are collapsed.
 */
class LayersViewState(val scroll: ScrollState, val folders: SnapshotStateMap<Long, Boolean>)

/** What the Layers panel can ask of the document. */
class LayersActions(
    val card: LayerCardActions,
    val newVector: () -> Unit,
    val newRaster: () -> Unit,
    val newTexture: () -> Unit,
    /** The filters a filter layer can apply, as (effect id, label). */
    val filters: () -> List<Pair<String, String>>,
    val newFilter: (String) -> Unit,
    val newFolder: () -> Unit,
    val deleteCurrent: () -> Unit,
    val move: (Long, Long, String) -> Unit,
)

/**
 * The Layers panel (`uistandards.md` §7.1): the creation row (Vector, Raster, Filter, Texture) as a strip of text buttons,
 * the compositing stack topmost first as cards ([LayerCard]), which a card's grip drags above,
 * below or into another (a folder's middle), and the footer with Delete and Add Folder.
 * Texture opens the Texture library, where a texture is picked; Filter offers the filters a filter
 * layer can apply (it filters everything below it on the page, `uistandards.md` §7.3).
 */
@Composable
fun LayersPanel(
    rows: JSONArray,
    masks: MaskContext,
    expanded: Boolean,
    onToggle: () -> Unit,
    actions: LayersActions,
    view: LayersViewState,
    modifier: Modifier = Modifier,
    headerModifier: Modifier = Modifier,
) {
    FloatingPanel("Layers", Icons.Outlined.Layers, count(rows), expanded, onToggle, 368.dp, modifier, headerModifier) {
        // The creation row as a strip of text buttons under the header.
        Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp), horizontalArrangement = Arrangement.SpaceBetween) {
            Create(Icons.Outlined.Category, "Vector", actions.newVector)
            Create(Icons.Outlined.Brush, "Raster", actions.newRaster)
            Box {
                var menu by remember { mutableStateOf(false) }
                Create(Icons.Outlined.AutoAwesome, "Filter") { menu = true }
                DropdownMenu(expanded = menu, onDismissRequest = { menu = false }, modifier = Modifier.heightIn(max = 420.dp)) {
                    for ((effect, label) in actions.filters()) {
                        DropdownMenuItem(text = { Text(label) }, onClick = { menu = false; actions.newFilter(effect) })
                    }
                }
            }
            Create(Icons.Outlined.Texture, "Texture", actions.newTexture)
        }
        Box(Modifier.fillMaxWidth().padding(top = 4.dp, bottom = 10.dp).height(1.dp).background(EtcColors.LineSoft))
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
            Modifier.heightIn(max = 520.dp).verticalScroll(view.scroll).padding(horizontal = 10.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            for (i in 0 until rows.length()) rows.optJSONObject(i)?.let { LayerCard(it, 0, actions.card, drag, view.folders, masks) }
        }
        Row(Modifier.fillMaxWidth().padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            FooterButton(Icons.Outlined.Delete, "Delete", actions.deleteCurrent)
            Spacer(Modifier.weight(1f))
            FooterButton(Icons.Outlined.CreateNewFolder, "Add folder", actions.newFolder, accent = true)
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

/** A creation row button: a small icon and the kind it makes, as a text button (`New <label>`). */
@Composable
private fun Create(icon: ImageVector, label: String, onClick: () -> Unit) {
    Row(
        Modifier
            .clip(RoundedCornerShape(8.dp))
            .clickable(onClick = onClick)
            .semantics { contentDescription = "New $label" }
            .padding(horizontal = 8.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(5.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(icon, null, tint = EtcColors.Muted, modifier = Modifier.size(15.dp))
        Text(label, color = EtcColors.Muted, fontSize = 14.sp)
    }
}

/** A footer icon button, gold-edged when [accent]. */
@Composable
private fun FooterButton(icon: ImageVector, description: String, onClick: () -> Unit, accent: Boolean = false) {
    Box(
        Modifier.size(44.dp)
            .background(EtcColors.Panel2, RoundedCornerShape(12.dp))
            .border(1.dp, if (accent) EtcColors.Gold2 else EtcColors.Line, RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .semantics { contentDescription = description },
        contentAlignment = Alignment.Center,
    ) { Icon(icon, null, tint = if (accent) EtcColors.Gold else EtcColors.Text, modifier = Modifier.size(22.dp)) }
}

// GLOSSARY
// - LayersActions: what the panel asks of the document (cards, new vector/raster layer, folder, delete, move).
// - LayersViewState: the panel's scroll and collapsed folders, kept while Export Settings shows.
// - LayersPanel(rows, expanded, onToggle, actions, view, modifier, headerModifier): the Layers panel.
// - dropAt(bounds, dragged, y): where a dragged card would go.
// - count(rows): the rows, folders' included.
// - Create(icon, label, onClick): a creation row text button.
// - FooterButton(icon, description, onClick, accent): a footer icon button.
