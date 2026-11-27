package com.extemporecreare.app.ui.editor

import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.DragHandle
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.extemporecreare.app.ui.dial.TOOLBAR_REACH
import com.extemporecreare.app.ui.dial.DialToolbar
import androidx.compose.runtime.setValue
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.geometry.Offset
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
import com.extemporecreare.app.ui.dial.DialGrip
import com.extemporecreare.app.ui.dial.DialStepper
import com.extemporecreare.app.ui.dial.RotaryDial
import com.extemporecreare.app.ui.dial.ToolReadout
import com.extemporecreare.app.ui.dial.setSize
import com.extemporecreare.app.ui.dial.sizeOf
import com.extemporecreare.app.ui.panels.LayerCardActions
import com.extemporecreare.app.ui.panels.LayersActions
import com.extemporecreare.app.ui.panels.LayersPanel
import com.extemporecreare.app.ui.panels.LayersViewState
import com.extemporecreare.app.ui.panels.BoardActions
import com.extemporecreare.app.ui.panels.MaskActions
import com.extemporecreare.app.ui.panels.MaskContext
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
class EditorRequests(val newDocument: () -> Unit, val saveAs: () -> Unit, val open: () -> Unit, val exportFolder: () -> Unit)

/**
 * The editor workspace (`uistandards.md` §4): the artwork canvas filling the screen under the top
 * bar; the Texture library upper left; Layers (or, while the Export tool is active, Export
 * Settings in its place) upper right; the Reference board lower right; and the rotary dial cropped
 * into the lower-left corner with its colour control, the tool readout beside it and the active
 * tool's settings window above the readout; its grip dragged to an edge docks it there as a toolbar
 * ([DialToolbar]), with the readout, settings and colour workspace beside it. The panels move by holding their headers and dock at
 * the workspace's edges ([MovablePanel]); focus mode hides them. While debug mode is on (switched from ADB,
 * [DebugMode]) its readout shows at the top.
 */
@Composable
fun EditorScreen(state: EditorState, requests: EditorRequests, board: BoardActions) {
    val layout = state.layout
    // The Layers panel's scroll and collapsed folders, kept while Export Settings takes its place.
    val layersView = remember { LayersViewState(ScrollState(0), mutableStateMapOf()) }
    val revision = state.revision
    // The workspace under the top bar: its place in the root and its size, in pixels (for the dial's grip).
    var workspaceOrigin by remember { mutableStateOf(Offset.Zero) }
    var workspaceSize by remember { mutableStateOf(IntSize.Zero) }
    // A recovery copy of unsaved work every few minutes (§5.1).
    LaunchedEffect(Unit) {
        while (true) {
            delay(EditorState.RECOVERY_EVERY_MS)
            state.saveRecovery()
        }
    }
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
        Box(
            Modifier.fillMaxSize().padding(top = BAR_HEIGHT).onGloballyPositioned {
                workspaceOrigin = it.boundsInRoot().topLeft
                workspaceSize = it.size
            },
        ) {
            if (!layout.focus) {
                BoxWithConstraints(Modifier.fillMaxSize()) {
                    MovablePanel(PanelLayout.TEXTURES, layout) { header ->
                        TextureLibraryPanel(state.textures, layout.texturesOpen, layout::toggleTextures, { state.insertTexture(it) }, headerModifier = header)
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
                                val stack = remember(revision) { JSONObject(state.engine.layers()).optJSONObject("ok") ?: JSONObject() }
                                val rows = stack.optJSONArray("rows") ?: JSONArray()
                                val masks = MaskContext(stack.optInt("selected"), if (stack.isNull("editingMask")) null else stack.optLong("editingMask"))
                                LayersPanel(rows, masks, layout.layersOpen, layout::toggleLayers, layersActions(state, layout), layersView, headerModifier = header)
                            }
                        }
                    }
                    MovablePanel(PanelLayout.BOARD, layout) { header ->
                        ReferenceBoardPanel(state.board, state.boardRevision, state.boardSelected, layout.boardOpen, layout::toggleBoard, board, headerModifier = header)
                    }
                }
            }
            val dock = layout.toolbarDock
            val readout = @Composable { m: Modifier ->
                ToolReadout(
                    name = state.activeTool.name,
                    size = remember(revision, state.activeIndex, state.settingsOpen) { sizeOf(state.activeTool) },
                    onSize = { state.show(setSize(state.engine, state.activeTool, it)); state.touched() },
                    onTap = { state.settingsOpen = !state.settingsOpen },
                    modifier = m,
                )
            }
            val colors = @Composable { m: Modifier ->
                ColorControl(state.primary, state.alternate, { state.swapColors() }, { state.colorWorkspaceOpen = !state.colorWorkspaceOpen }, m)
            }
            val workspace = @Composable { m: Modifier ->
                ColorWorkspace(state.primary, state.alternate, state.colors, state::setColor, { state.colorWorkspaceOpen = false }, m)
            }
            val settings = @Composable { m: Modifier -> Box(m) { ToolSettingsHost(state) { requests.saveAs() } } }
            // While the grip is dragged near an edge: where the toolbar would dock.
            layout.toolbarPreview?.takeIf { it != dock }?.let { edge ->
                val band = when (edge) {
                    "left" -> Modifier.align(Alignment.CenterStart).fillMaxHeight().width(TOOLBAR_REACH)
                    "right" -> Modifier.align(Alignment.CenterEnd).fillMaxHeight().width(TOOLBAR_REACH)
                    "top" -> Modifier.align(Alignment.TopCenter).fillMaxWidth().height(TOOLBAR_REACH)
                    else -> Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(TOOLBAR_REACH)
                }
                Box(band.padding(8.dp).background(EtcColors.Gold.copy(alpha = 0.18f), RoundedCornerShape(16.dp)).border(2.dp, EtcColors.Gold, RoundedCornerShape(16.dp)))
            }
            if (dock == null) {
                val dx = layout.dialX.dp
                val dy = layout.dialY.dp
                RotaryDial(
                    names = state.tools.map { it.name },
                    active = state.activeIndex,
                    onChoose = { state.chooseTool(it) },
                    onActiveTap = { state.settingsOpen = !state.settingsOpen },
                    modifier = Modifier.align(Alignment.BottomStart).offset((-110).dp + dx, 110.dp + dy),
                ) { colors(Modifier) }
                DialGrip(layout, workspaceOrigin, workspaceSize, Modifier.align(Alignment.BottomStart).offset(18.dp + dx, (-250).dp + dy))
                DialStepper({ state.stepTool(-1) }, { state.stepTool(1) }, Modifier.align(Alignment.BottomStart).offset(206.dp + dx, (-16).dp + dy))
                readout(Modifier.align(Alignment.BottomStart).offset(210.dp + dx, (-64).dp + dy))
                if (state.colorWorkspaceOpen) workspace(Modifier.align(Alignment.BottomStart).offset(16.dp + dx, (-190).dp + dy))
                if (state.settingsOpen) settings(Modifier.align(Alignment.BottomStart).offset(210.dp + dx, (-130).dp + dy))
            } else {
                // The dial docked as a toolbar: its readout, colour workspace and settings beside it.
                val (bar, beside, above) = when (dock) {
                    "left" -> Triple(Alignment.CenterStart, Alignment.BottomStart, Alignment.TopStart)
                    "right" -> Triple(Alignment.CenterEnd, Alignment.BottomEnd, Alignment.TopEnd)
                    "top" -> Triple(Alignment.TopCenter, Alignment.TopStart, Alignment.TopStart)
                    else -> Triple(Alignment.BottomCenter, Alignment.BottomStart, Alignment.BottomStart)
                }
                val inward = when (dock) {
                    "left" -> DpOffset(TOOLBAR_REACH, 0.dp)
                    "right" -> DpOffset(-TOOLBAR_REACH, 0.dp)
                    "top" -> DpOffset(0.dp, TOOLBAR_REACH)
                    else -> DpOffset(0.dp, -TOOLBAR_REACH)
                }
                val along = if (dock == "top") 1 else -1
                DialToolbar(
                    edge = dock,
                    names = state.tools.map { it.name },
                    active = state.activeIndex,
                    onChoose = { state.chooseTool(it) },
                    onActiveTap = { state.settingsOpen = !state.settingsOpen },
                    grip = { DialGrip(layout, workspaceOrigin, workspaceSize) },
                    colors = colors,
                    modifier = Modifier.align(bar).padding(8.dp),
                )
                val side = if (dock == "top" || dock == "bottom") 16.dp else 0.dp
                readout(Modifier.align(beside).offset(inward.x + side, inward.y + (16 * along).dp))
                if (state.settingsOpen) settings(Modifier.align(beside).offset(inward.x + side, inward.y + (72 * along).dp))
                if (state.colorWorkspaceOpen) workspace(Modifier.align(above).offset(inward.x + if (dock == "top" || dock == "bottom") 240.dp else 0.dp, inward.y + if (dock == "left" || dock == "right") 16.dp else 0.dp))
            }
            if (state.debugOn) {
                DebugOverlay(state.penReading, state.activeTool.name, state.camera.zoom, state.renderMs, Modifier.align(Alignment.TopCenter).padding(top = 12.dp))
            }
            state.confirm?.let { c -> ConfirmDialog(c) { state.confirm = null } }
            Column(Modifier.align(Alignment.BottomCenter).padding(bottom = if (layout.toolbarDock == "bottom") 24.dp + TOOLBAR_REACH else 24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                state.error?.let { text ->
                    Row(
                        Modifier.widthIn(max = 520.dp).background(EtcColors.Panel2, RoundedCornerShape(10.dp)).border(1.dp, EtcColors.Gold2, RoundedCornerShape(10.dp))
                            .padding(start = 14.dp, end = 6.dp, top = 4.dp, bottom = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(text, color = EtcColors.Text, fontSize = 13.sp, modifier = Modifier.weight(1f, fill = false))
                        Icon(Icons.Outlined.Close, "Dismiss", tint = EtcColors.Muted, modifier = Modifier.padding(start = 8.dp).size(36.dp).clickable { state.error = null }.padding(8.dp))
                    }
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
                        modifier = Modifier.background(EtcColors.Panel2, RoundedCornerShape(10.dp)).padding(horizontal = 14.dp, vertical = 8.dp),
                    )
                }
            }
        }
    }
}

/** The Layers panel's actions on the document through engine commands. */
private fun layersActions(state: EditorState, layout: PanelLayout): LayersActions {
    val ids = { id: Long -> JSONArray().put(id) }
    return LayersActions(
        card = LayerCardActions(
            select = { id -> state.run("layer.setCurrent", JSONObject().put("id", id)) },
            setVisible = { id, on -> state.run("layer.setProps", JSONObject().put("id", id).put("visible", on)) },
            setOpacity = { id, v -> state.run("transparency.set", JSONObject().put("ids", ids(id)).put("opacity", v)) },
            setFill = { id, v -> state.run("layer.setFill", JSONObject().put("ids", ids(id)).put("fill", v)) },
            setTextureScale = { id, percent -> state.run("texture.set", JSONObject().put("id", id).put("scale", percent / 100.0)) },
            mask = MaskActions(
                add = { id, mode -> state.run("layer.mask.add", JSONObject().put("id", id).put("mode", mode)) },
                fromSelection = { id -> state.run("layer.mask.fromSelection", JSONObject().put("id", id)) },
                paint = { id -> state.paintMask(id) },
                invert = { id -> state.run("layer.mask.invert", JSONObject().put("id", id)) },
                setEnabled = { id, on -> state.run("layer.mask.setEnabled", JSONObject().put("id", id).put("enabled", on)) },
                delete = { id -> state.run("layer.mask.delete", JSONObject().put("id", id)) },
                reset = { id, mode ->
                    state.run("layer.mask.delete", JSONObject().put("id", id))
                    state.run("layer.mask.add", JSONObject().put("id", id).put("mode", mode))
                },
            ),
            setBlend = { id, mode -> state.run("transparency.set", JSONObject().put("ids", ids(id)).put("blend", mode)) },
            addStyle = { id, command, params ->
                params.remove("_label")
                state.run(command, params.put("ids", ids(id)))
            },
        ),
        move = { id, target, place -> state.run("layer.move", JSONObject().put("ids", ids(id)).put("target", target).put("place", place)) },
        newVector = { state.run("layer.new", JSONObject().put("name", "Vector layer")) },
        newRaster = { state.run("layer.new", JSONObject().put("name", "Raster layer")) },
        newTexture = { if (!layout.texturesOpen) layout.toggleTextures() },
        filters = { state.filters },
        newFilter = { effect -> state.run("filter.new", JSONObject().put("effect", effect)) },
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
// - EditorScreen(state, requests, board): the editor workspace; board: the Reference board panel's actions.
// - layersActions(state, layout): the Layers panel's engine commands (Texture opens the Texture library).
// - findCurrent(rows): the current row.
