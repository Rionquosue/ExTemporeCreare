package com.extemporecreare.app.ui.editor

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.DragHandle
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateMapOf
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
import com.extemporecreare.app.ui.color.ColorWorkspace
import com.extemporecreare.app.ui.common.ConfirmDialog
import com.extemporecreare.app.ui.common.Confirmation
import com.extemporecreare.app.ui.debug.DebugOverlay
import com.extemporecreare.app.ui.dial.ColorControl
import com.extemporecreare.app.ui.dial.RotaryDial
import com.extemporecreare.app.ui.dial.ToolReadout
import com.extemporecreare.app.ui.dial.setSize
import com.extemporecreare.app.ui.dial.sizeOf
import com.extemporecreare.app.ui.panels.LayerCardActions
import com.extemporecreare.app.ui.panels.LayersActions
import com.extemporecreare.app.ui.panels.LayersPanel
import com.extemporecreare.app.ui.panels.LayersViewState
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
 * tool's settings window above the readout. The panels move by holding their headers and dock at
 * the workspace's edges ([MovablePanel]); focus mode hides them. While debug mode is on (switched from ADB,
 * [DebugMode]) its readout shows at the top.
 */
@Composable
fun EditorScreen(state: EditorState, requests: EditorRequests) {
    val layout = state.layout
    // The Layers panel's scroll and collapsed folders, kept while Export Settings takes its place.
    val layersView = remember { LayersViewState(ScrollState(0), mutableStateMapOf()) }
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
                BoxWithConstraints(Modifier.fillMaxSize()) {
                    MovablePanel(PanelLayout.TEXTURES, layout) { header ->
                        TextureLibraryPanel(layout.texturesOpen, layout::toggleTextures, headerModifier = header)
                    }
                    MovablePanel(PanelLayout.LAYERS, layout) { header ->
                        val export = state.activeTool as? ExportTool
                        Box(Modifier.width(368.dp)) {
                            if (export != null) {
                                Column {
                                    Box(
                                        header.fillMaxWidth().height(18.dp).background(EtcColors.Panel2, RoundedCornerShape(topStart = 12.dp, topEnd = 12.dp)),
                                        contentAlignment = Alignment.Center,
                                    ) { Icon(Icons.Outlined.DragHandle, "Hold to move Export Settings", tint = EtcColors.Soft, modifier = Modifier.size(16.dp)) }
                                    ExportToolSettingsPanel(export, state.engine, revision) { state.show(it); state.touched() }
                                }
                            } else {
                                val rows = remember(revision) { JSONObject(state.engine.layers()).optJSONObject("ok")?.optJSONArray("rows") ?: JSONArray() }
                                LayersPanel(rows, layout.layersOpen, layout::toggleLayers, layersActions(state), layersView, headerModifier = header)
                            }
                        }
                    }
                    MovablePanel(PanelLayout.BOARD, layout) { header ->
                        ReferenceBoardPanel(0, layout.boardOpen, layout::toggleBoard, requests.openBoard, headerModifier = header)
                    }
                }
            }
            RotaryDial(
                names = state.tools.map { it.name },
                active = state.activeIndex,
                onChoose = { state.chooseTool(it) },
                onActiveTap = { state.settingsOpen = !state.settingsOpen },
                modifier = Modifier.align(Alignment.BottomStart).offset((-110).dp, 110.dp),
            ) {
                ColorControl(state.primary, state.alternate, { state.swapColors() }, { state.colorWorkspaceOpen = !state.colorWorkspaceOpen })
            }
            ToolReadout(
                name = state.activeTool.name,
                size = remember(revision, state.activeIndex, state.settingsOpen) { sizeOf(state.activeTool) },
                onSize = { state.show(setSize(state.engine, state.activeTool, it)); state.touched() },
                onTap = { state.settingsOpen = !state.settingsOpen },
                modifier = Modifier.align(Alignment.BottomStart).padding(start = 210.dp, bottom = 60.dp),
            )
            if (state.colorWorkspaceOpen) {
                ColorWorkspace(state.primary, state.alternate, state.colors, state::setColor, { state.colorWorkspaceOpen = false }, Modifier.align(Alignment.BottomStart).padding(start = 16.dp, bottom = 190.dp))
            }
            if (state.settingsOpen) {
                Box(Modifier.align(Alignment.BottomStart).padding(start = 210.dp, bottom = 130.dp)) { ToolSettingsHost(state) { requests.saveAs() } }
            }
            if (state.debugOn) {
                DebugOverlay(state.penReading, state.activeTool.name, state.camera.zoom, state.renderMs, Modifier.align(Alignment.TopCenter).padding(top = 12.dp))
            }
            state.confirm?.let { c -> ConfirmDialog(c) { state.confirm = null } }
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
            addStyle = { id, command, params ->
                params.remove("_label")
                state.run(command, params.put("ids", ids(id)))
            },
        ),
        move = { id, target, place -> state.run("layer.move", JSONObject().put("ids", ids(id)).put("target", target).put("place", place)) },
        newVector = { state.run("layer.new", JSONObject().put("name", "Vector layer")) },
        newRaster = { state.run("layer.new", JSONObject().put("name", "Raster layer")) },
        newFolder = { state.run("layer.newSublayer", JSONObject().put("name", "Folder")) },
        deleteCurrent = {
            val rows = EngineAnswer(state.engine.layers()).okObject?.optJSONArray("rows")
            val current = rows?.let { findCurrent(it) }
            when {
                current == null -> state.message = "Select a row to delete"
                (current.optJSONArray("children")?.length() ?: 0) > 0 -> state.confirm = Confirmation(
                    "Delete “${current.optString("name")}” and everything in it?",
                    "Delete",
                ) { state.run("layer.delete", JSONObject().put("ids", ids(current.optLong("id")))) }
                else -> state.run("layer.delete", JSONObject().put("ids", ids(current.optLong("id"))))
            }
        },
    )
}

/** The current row in [rows] (depth first), or null. */
private fun findCurrent(rows: JSONArray): JSONObject? {
    for (i in 0 until rows.length()) {
        val r = rows.optJSONObject(i) ?: continue
        if (r.optBoolean("current")) return r
        r.optJSONArray("children")?.let { findCurrent(it) }?.let { return it }
    }
    return null
}

// GLOSSARY
// - EditorRequests: the system pickers the editor asks the activity for.
// - EditorScreen(state, requests): the editor workspace.
// - layersActions(state): the Layers panel's engine commands.
// - findCurrent(rows): the current row.
