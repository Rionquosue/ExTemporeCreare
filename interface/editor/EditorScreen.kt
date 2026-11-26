package com.extemporecreare.app.ui.editor

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.tools.export.ExportTool
import com.extemporecreare.app.components.debug.DebugMode
import com.extemporecreare.app.ui.canvas.EditorCanvasView
import com.extemporecreare.app.ui.debug.DebugOverlay
import com.extemporecreare.app.ui.dial.ColorControl
import com.extemporecreare.app.ui.dial.RotaryDial
import com.extemporecreare.app.ui.dial.ToolReadout
import com.extemporecreare.app.ui.dial.setSize
import com.extemporecreare.app.ui.dial.sizeOf
import com.extemporecreare.app.ui.panels.LayerCardActions
import com.extemporecreare.app.ui.panels.LayersActions
import com.extemporecreare.app.ui.panels.LayersPanel
import com.extemporecreare.app.ui.panels.ReferenceBoardPanel
import com.extemporecreare.app.ui.panels.TextureLibraryPanel
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.export.ExportToolSettingsPanel
import com.extemporecreare.app.ui.topbar.BAR_HEIGHT
import com.extemporecreare.app.ui.topbar.MenuEntry
import com.extemporecreare.app.ui.topbar.TopBar
import kotlinx.coroutines.delay
import org.json.JSONArray
import org.json.JSONObject

/** What the editor asks of the activity: the system pickers. */
class EditorRequests(val newDocument: () -> Unit, val saveAs: () -> Unit, val open: () -> Unit, val exportFolder: () -> Unit, val openBoard: () -> Unit)

/**
 * The editor workspace (`uistandards.md` §4): the artwork canvas filling the screen under the top
 * bar; the Texture library upper left; Layers (or, while the Export tool is active, Export
 * Settings in its place) upper right; the Reference board lower right; and the rotary dial cropped
 * into the lower-left corner with its colour control, the tool readout beside it and the active
 * tool's settings window above the readout. While debug mode is on (switched from ADB,
 * [DebugMode]) its readout shows at the top.
 */
@Composable
fun EditorScreen(state: EditorState, requests: EditorRequests) {
    val layout = state.layout
    val revision = state.revision
    // Debug mode follows its ADB switch (DebugMode), checked every second.
    LaunchedEffect(Unit) {
        while (true) {
            val on = DebugMode.isOn
            if (on != state.debugOn) {
                state.debugOn = on
                if (!on) state.penReading = null
            }
            delay(1000)
        }
    }
    Box(Modifier.fillMaxSize().background(EtcColors.Bg)) {
        AndroidView(
            factory = { EditorCanvasView(it, state) },
            update = { it.follow(state.revision, state.camera) },
            modifier = Modifier.fillMaxSize().padding(top = BAR_HEIGHT),
        )
        TopBar(
            menu = listOf(
                MenuEntry("Home") { state.home() },
                MenuEntry("New document…") { requests.newDocument() },
                MenuEntry("Open…") { requests.open() },
                MenuEntry("Save as…") { requests.saveAs() },
                MenuEntry("Zoom in") { state.zoomBy(1.25) },
                MenuEntry("Zoom out") { state.zoomBy(0.8) },
                MenuEntry("Fit artwork") { state.fitArtwork() },
                MenuEntry("Actual size (100 %)") { state.actualSize() },
                MenuEntry(if (layout.focus) "Leave focus mode" else "Focus mode") { layout.toggleFocus() },
                MenuEntry("Reset panel layout") { layout.reset() },
            ),
            onSave = { state.documentUri?.let { state.save(it) } ?: requests.saveAs() },
            onFlip = { state.flip() },
            flipped = state.camera.flipped,
            onUndo = { state.undo() },
            onRedo = { state.redo() },
            onExport = { requests.exportFolder() },
        )
        Box(Modifier.fillMaxSize().padding(top = BAR_HEIGHT)) {
            if (!layout.focus) {
                TextureLibraryPanel(layout.texturesOpen, layout::toggleTextures, Modifier.align(Alignment.TopStart).padding(16.dp))
                val export = state.activeTool as? ExportTool
                Box(Modifier.align(Alignment.TopEnd).padding(12.dp).width(368.dp)) {
                    if (export != null) {
                        ExportToolSettingsPanel(export, state.engine, revision) { state.show(it); state.touched() }
                    } else {
                        val rows = remember(revision) { JSONObject(state.engine.layers()).optJSONObject("ok")?.optJSONArray("rows") ?: JSONArray() }
                        LayersPanel(rows, layout.layersOpen, layout::toggleLayers, layersActions(state))
                    }
                }
                ReferenceBoardPanel(0, layout.boardOpen, layout::toggleBoard, requests.openBoard, Modifier.align(Alignment.BottomEnd).padding(16.dp))
            }
            RotaryDial(
                names = state.tools.map { it.name },
                active = state.activeIndex,
                onChoose = { state.chooseTool(it) },
                onActiveTap = { state.settingsOpen = !state.settingsOpen },
                modifier = Modifier.align(Alignment.BottomStart).offset((-110).dp, 110.dp),
            ) {
                ColorControl(state.primary, state.alternate, { state.swapColors() }, { state.message = "The colour workspace is not built yet" })
            }
            ToolReadout(
                name = state.activeTool.name,
                size = remember(revision, state.activeIndex, state.settingsOpen) { sizeOf(state.activeTool) },
                onSize = { state.show(setSize(state.engine, state.activeTool, it)); state.touched() },
                onTap = { state.settingsOpen = !state.settingsOpen },
                modifier = Modifier.align(Alignment.BottomStart).padding(start = 210.dp, bottom = 60.dp),
            )
            if (state.settingsOpen) {
                Box(Modifier.align(Alignment.BottomStart).padding(start = 210.dp, bottom = 130.dp)) { ToolSettingsHost(state) { requests.saveAs() } }
            }
            if (state.debugOn) {
                DebugOverlay(state.penReading, state.activeTool.name, state.camera.zoom, state.renderMs, Modifier.align(Alignment.TopCenter).padding(top = 12.dp))
            }
            state.message?.let { text ->
                LaunchedEffect(text) {
                    delay(3500)
                    if (state.message == text) state.message = null
                }
                Text(
                    text,
                    color = EtcColors.Text,
                    fontSize = 13.sp,
                    modifier = Modifier.align(Alignment.BottomCenter).padding(bottom = 24.dp)
                        .background(EtcColors.Panel2, RoundedCornerShape(10.dp)).padding(horizontal = 14.dp, vertical = 8.dp),
                )
            }
        }
    }
}

/** The Layers panel's actions on the document through engine commands. */
private fun layersActions(state: EditorState): LayersActions {
    val ids = { id: Long -> JSONArray().put(id) }
    return LayersActions(
        card = LayerCardActions(
            select = { id -> state.run("layer.setCurrent", JSONObject().put("id", id)) },
            setVisible = { id, on -> state.run("layer.setProps", JSONObject().put("id", id).put("visible", on)) },
            setOpacity = { id, v -> state.run("transparency.set", JSONObject().put("ids", ids(id)).put("opacity", v)) },
            setBlend = { id, mode -> state.run("transparency.set", JSONObject().put("ids", ids(id)).put("blend", mode)) },
        ),
        newVector = { state.run("layer.new", JSONObject().put("name", "Vector layer")) },
        newRaster = { state.run("layer.new", JSONObject().put("name", "Raster layer")) },
        newFolder = { state.run("layer.newSublayer", JSONObject().put("name", "Folder")) },
        deleteCurrent = {
            val current = EngineAnswer(state.engine.layers()).okObject?.optJSONArray("rows")?.let { findCurrent(it) }
            if (current == null) state.message = "Select a row to delete" else state.run("layer.delete", JSONObject().put("ids", ids(current)))
        },
    )
}

/** The current row's id in [rows] (depth first), or null. */
private fun findCurrent(rows: JSONArray): Long? {
    for (i in 0 until rows.length()) {
        val r = rows.optJSONObject(i) ?: continue
        if (r.optBoolean("current")) return r.optLong("id")
        r.optJSONArray("children")?.let { findCurrent(it) }?.let { return it }
    }
    return null
}

// GLOSSARY
// - EditorRequests: the system pickers the editor asks the activity for.
// - EditorScreen(state, requests): the editor workspace.
// - layersActions(state): the Layers panel's engine commands.
// - findCurrent(rows): the current row's id.
