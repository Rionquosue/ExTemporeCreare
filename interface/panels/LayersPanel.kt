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
)

/**
 * The Layers panel (`uistandards.md` §7.1): the creation row (Vector, Raster, Filter, Texture),
 * the compositing stack topmost first as cards ([LayerCard]), and the footer with Delete, the hint
 * and Add Folder. Filter and Texture layers have no engine object behind them yet and are shown
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
        Column(
            Modifier.heightIn(max = 520.dp).verticalScroll(rememberScrollState()).padding(horizontal = 10.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            for (i in 0 until rows.length()) rows.optJSONObject(i)?.let { LayerCard(it, 0, actions.card) }
        }
        Row(Modifier.fillMaxWidth().padding(10.dp), verticalAlignment = Alignment.CenterVertically) {
            FooterButton(Icons.Outlined.Delete, "Delete", actions.deleteCurrent)
            Text("Drag rows to reorder · filters cascade downward", color = EtcColors.Soft, fontSize = 10.sp, modifier = Modifier.weight(1f).padding(horizontal = 8.dp))
            FooterButton(Icons.Outlined.CreateNewFolder, "Add folder", actions.newFolder)
        }
    }
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
// - LayersActions: what the panel asks of the document (cards, new vector/raster layer, folder, delete).
// - LayersPanel(rows, expanded, onToggle, actions, modifier): the Layers panel.
// - count(rows): the rows, folders' included.
// - Create(icon, label, modifier, onClick): a creation row button.
// - FooterButton(icon, description, onClick): a footer icon button.
