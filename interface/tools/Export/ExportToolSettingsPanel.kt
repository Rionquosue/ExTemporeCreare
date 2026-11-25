package com.extemporecreare.app.ui.tools.export

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.export.ExportScope
import com.extemporecreare.app.tools.export.ExportSlice
import com.extemporecreare.app.tools.export.ExportTool
import com.extemporecreare.app.tools.export.ExportUnits
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingChoice
import com.extemporecreare.app.ui.tools.SettingNote
import com.extemporecreare.app.ui.tools.SettingNumberField
import com.extemporecreare.app.ui.tools.SettingSwitch
import com.extemporecreare.app.ui.tools.ToolSettingsWindow
import com.extemporecreare.app.ui.tools.formatNumber
import org.json.JSONObject
import kotlin.math.roundToInt

/**
 * Export Settings (`uistandards.md` §9.2), shown in the Layers panel's place while the Export tool
 * is active: scope, the slice list (select, show or hide, rename, duplicate, delete), the selected
 * slice's ratio and dimensions (px, in, cm, mm) and output (format, scale, final pixels), and the
 * dot grid. A slice just drawn opens its dimensions editor (§9.3). [revision] changes whenever the
 * canvas changed the slices, so the panel reads them again.
 */
@Composable
fun ExportToolSettingsPanel(tool: ExportTool, engine: EngineBridge, revision: Int, onResult: (ToolResult) -> Unit) {
    var tick by remember { mutableIntStateOf(0) }
    var units by remember { mutableStateOf(ExportUnits.PX) }
    val finite = remember(revision, tick) { EngineAnswer(engine.state()).okObject?.optString("canvas") == "finite" }
    fun done(r: ToolResult) {
        tick++
        onResult(r)
    }
    LaunchedEffect(revision) { if (tool.revealDimensions) tool.dimensionsShown() }
    val list = tool.slices
    val selected = list.slices.firstOrNull { it.id == tool.selectedId }
    ToolSettingsWindow("Export Settings", Modifier.verticalScroll(rememberScrollState())) {
        val scopes = ExportScope.entries.filter { it != ExportScope.WHOLE_DOCUMENT || finite }
        SettingChoice("Scope", scopes.map { it to it.label }, tool.scope) { tool.chooseScope(it); tick++ }
        if (tool.scope == ExportScope.AUTO_CONTENT && list.contentBounds == null) {
            SettingNote("Nothing to export. Drag on the canvas to create a slice.", EtcColors.Gold2)
        }
        Text("Slices", color = EtcColors.Muted, fontSize = 12.sp)
        if (list.slices.isEmpty()) SettingNote("No slices yet")
        for (slice in list.slices) {
            SliceRow(slice, slice.id == tool.selectedId, onSelect = { tool.select(slice.id); tick++ }, onVisible = {
                done(tool.setOutput(engine, slice.id, JSONObject().put("visible", it)))
            })
        }
        if (selected != null) SliceEditor(tool, engine, selected, units, { units = it }, ::done)
        Text("Grid", color = EtcColors.Muted, fontSize = 12.sp)
        SettingSwitch("Show dot grid", list.grid.visible) { done(tool.setGrid(engine, list.grid.copy(visible = it))) }
        SettingSwitch("Snap to grid", list.grid.snapEnabled) { done(tool.setGrid(engine, list.grid.copy(snapEnabled = it))) }
        SettingNumberField("Grid size", list.grid.spacingPx, "px", { if (it > 0) done(tool.setGrid(engine, list.grid.copy(spacingPx = it))) })
    }
}

/** One slice in the list: its name (tap to select) and its visibility. */
@Composable
private fun SliceRow(slice: ExportSlice, selected: Boolean, onSelect: () -> Unit, onVisible: (Boolean) -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .background(if (selected) EtcColors.Panel2 else EtcColors.Panel, RoundedCornerShape(8.dp))
            .border(1.dp, if (selected) EtcColors.Gold else EtcColors.LineSoft, RoundedCornerShape(8.dp))
            .clickable(onClick = onSelect)
            .padding(horizontal = 10.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(slice.name, color = if (slice.visible) EtcColors.Text else EtcColors.Soft, fontSize = 12.sp)
        SettingSwitch("", slice.visible, onVisible)
    }
}

/** The selected slice: name, duplicate and delete, ratio, dimensions in [units], format, scale and final pixels. */
@Composable
private fun SliceEditor(
    tool: ExportTool,
    engine: EngineBridge,
    slice: ExportSlice,
    units: ExportUnits,
    onUnits: (ExportUnits) -> Unit,
    done: (ToolResult) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        var name by remember(slice.id, slice.name) { mutableStateOf(slice.name) }
        androidx.compose.material3.OutlinedTextField(
            value = name,
            onValueChange = { name = it },
            label = { Text("Name", fontSize = 11.sp) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SettingButton("Rename", { if (name.isNotBlank()) done(tool.setOutput(engine, slice.id, JSONObject().put("name", name.trim()))) }, enabled = name.isNotBlank() && name != slice.name)
            SettingButton("Duplicate", { done(tool.duplicate(engine, slice.id)) })
            SettingButton("Delete", { done(tool.delete(engine, slice.id)) })
        }
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
            SettingNumberField("Ratio", slice.aspectX, "", { done(tool.setAspect(engine, it, slice.aspectY)) }, Modifier.width(90.dp))
            Text(":", color = EtcColors.Text)
            SettingNumberField("", slice.aspectY, "", { done(tool.setAspect(engine, slice.aspectX, it)) }, Modifier.width(90.dp))
        }
        SettingChoice("Units", ExportUnits.entries.map { it to it.label }, units, onUnits)
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            SettingNumberField("W", units.fromPx(slice.width), units.label, { done(tool.setDimensions(engine, units.toPx(it), slice.height)) }, Modifier.width(120.dp))
            SettingNumberField("H", units.fromPx(slice.height), units.label, { done(tool.setDimensions(engine, slice.width, units.toPx(it))) }, Modifier.width(120.dp))
        }
        SettingChoice("Format", listOf("png" to "PNG", "jpg" to "JPEG", "webp" to "WebP"), slice.format) {
            done(tool.setOutput(engine, slice.id, JSONObject().put("format", it)))
        }
        SettingNumberField("Scale", slice.scale, "×", { if (it > 0) done(tool.setOutput(engine, slice.id, JSONObject().put("scale", it))) })
        SettingNote("Output: ${(slice.width * slice.scale).roundToInt()} × ${(slice.height * slice.scale).roundToInt()} px" + if (slice.format == "jpg") ", on white" else "")
        SettingNote("Region: ${formatNumber(slice.x)}, ${formatNumber(slice.y)} px")
    }
}

// GLOSSARY
// - ExportToolSettingsPanel(tool, engine, revision, onResult): Export Settings in the Layers panel's place.
// - SliceRow(slice, selected, onSelect, onVisible): one slice in the list.
// - SliceEditor(tool, engine, slice, units, onUnits, done): the selected slice's name, ratio, dimensions and output.
