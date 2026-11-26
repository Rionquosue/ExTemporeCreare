package com.extemporecreare.app.ui.editor

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.color.ColorLibrary
import com.extemporecreare.app.components.debug.DebugMode
import com.extemporecreare.app.components.documents.DocumentSettings
import com.extemporecreare.app.components.documents.RecentFiles
import com.extemporecreare.app.components.debug.PenReading
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.engine.EngineFiles
import com.extemporecreare.app.components.engine.EnginePresets
import com.extemporecreare.app.components.tools.CanvasTool
import com.extemporecreare.app.components.tools.DragBox
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import com.extemporecreare.app.tools.brush.BrushTool
import com.extemporecreare.app.tools.crop.CropTool
import com.extemporecreare.app.tools.eraser.EraserTool
import com.extemporecreare.app.tools.export.ExportSliceWriter
import com.extemporecreare.app.tools.export.ExportTool
import com.extemporecreare.app.tools.fill.FillTool
import com.extemporecreare.app.tools.marker.MarkerTool
import com.extemporecreare.app.tools.move.MoveTool
import com.extemporecreare.app.tools.pen.PenTool
import com.extemporecreare.app.tools.pencil.PencilTool
import com.extemporecreare.app.tools.selection.SelectionTool
import com.extemporecreare.app.tools.shapes.ShapesTool
import com.extemporecreare.app.ui.canvas.Camera
import com.extemporecreare.app.ui.common.Confirmation
import org.json.JSONObject

/**
 * The editor's state: the engine session, the dial's tools in order, the active tool, the camera,
 * the document's revision (which the canvas and panels follow), the colours, the open document's
 * URI and the message to show. The UI reads it and calls its actions.
 */
class EditorState(private val context: Context) {

    companion object {
        /** The recovery copies' folder in the app's files, and how often a copy is written (ms). */
        const val RECOVERY_FOLDER = "recovery"
        const val RECOVERY_EVERY_MS = 120_000L
    }

    /** Which screen shows: the opening splash or the editor (`uistandards.md` §5.1). */
    enum class Screen { SPLASH, EDITOR }

    /** The screen showing. */
    var screen by mutableStateOf(Screen.SPLASH)
        private set

    /** The recently used documents. */
    val recents = RecentFiles(context)

    /** Goes up when the recent files change. */
    var recentsRevision by mutableIntStateOf(0)
        private set

    /** The floating panels' layout. */
    val layout = PanelLayout(context)

    /** The engine session. */
    val engine = EngineBridge()

    /** Each tool's saved settings. */
    private val store = ToolSettingsStore(context)

    /** The dial's tools, in dial order (`uistandards.md` §6.1, with Marker and Pencil added). */
    val tools: List<CanvasTool> = listOf(
        BrushTool(store), MarkerTool(store), PencilTool(store), PenTool(store), EraserTool(store), FillTool(store),
        SelectionTool(store), ShapesTool(store), MoveTool(store), CropTool(store), ExportTool(store),
    )

    /** The active tool's index in [tools]. */
    var activeIndex by mutableIntStateOf(0)
        private set

    /** The active tool. */
    val activeTool: CanvasTool get() = tools[activeIndex]

    /** The artwork camera. */
    var camera by mutableStateOf(Camera())

    /** Goes up whenever the document may have changed: the canvas renders again and panels read again. */
    var revision by mutableIntStateOf(0)
        private set

    /** The primary (stroke) and alternate (fill) colours, `#rrggbb` or null. */
    var primary by mutableStateOf<String?>("#000000")
        private set
    var alternate by mutableStateOf<String?>("#ffffff")
        private set

    /** The colour workspace's swatches and history, and whether it is open. */
    val colors = ColorLibrary(context)
    var colorWorkspaceOpen by mutableStateOf(false)

    /** A question to confirm before a destructive action, or null. */
    var confirm by mutableStateOf<Confirmation?>(null)

    /** A message to show (an error or a result), or null. */
    var message by mutableStateOf<String?>(null)

    /** Debug mode: whether it is on (followed by the editor), the last pointer reading and the last render's time. */
    var debugOn by mutableStateOf(false)
    var penReading by mutableStateOf<PenReading?>(null)
    var renderMs by mutableStateOf(0L)

    /** Whether the active tool's settings window is open. */
    var settingsOpen by mutableStateOf(false)

    /** The open document's URI (null: never saved). */
    var documentUri: Uri? = null
        private set

    /** The canvas view's size in pixels. */
    private var viewW = 0
    private var viewH = 0

    /**
     * Install the presets and set up data recovery; a cold launch shows the splash (§5.1), unless a
     * session was interrupted: its recovery copies are opened and the editor shows them.
     */
    fun start() {
        EngineBridge.setPresetsRoot(EnginePresets.install(context).path)
        val folder = java.io.File(context.filesDir, RECOVERY_FOLDER).apply { mkdirs() }
        show(EngineAnswer(engine.execute("prefs.set", JSONObject().put("values", JSONObject().put("recoveryFolder", folder.path).put("autosaveRecovery", true)).toString())).error)
        recoverInterrupted()
    }

    /** Open the recovery copies an interrupted session left, if any (§5.1); a failure is shown. */
    private fun recoverInterrupted() {
        val copies = EngineAnswer(engine.execute("file.recovery.list", "{}")).okObject?.optJSONArray("copies") ?: return
        val left = (0 until copies.length()).mapNotNull { copies.optJSONObject(it) }.filter { !it.optBoolean("open") && !it.optBoolean("running") }
        if (left.isEmpty()) return
        val restored = EngineAnswer(engine.execute("file.recovery.restore", "{}"))
        val ok = restored.okObject
        if (ok == null) {
            message = "Couldn't recover the interrupted session: ${restored.error}"
            return
        }
        val failed = ok.optJSONArray("failed")?.length() ?: 0
        if ((ok.optJSONArray("restored")?.length() ?: 0) == 0) {
            message = "Couldn't recover the interrupted session"
            return
        }
        documentUri = null
        afterDocumentChange()
        message = if (failed > 0) "Recovered the session; $failed recovery copies were damaged" else "Recovered your unsaved work: save it to keep it"
    }

    /** Write a recovery copy of the document's unsaved changes (the app does this every few minutes and when it goes to the background). */
    fun saveRecovery() {
        if (screen != Screen.EDITOR) return
        val error = EngineAnswer(engine.execute("file.recovery.save", "{}")).error
        DebugMode.log("engine", "file.recovery.save → ${error ?: "ok"}")
    }

    /** A new document from [settings] in place of the open one, the settings kept as the defaults. */
    fun newDocument(settings: DocumentSettings) {
        activeTool.deactivate(engine)
        val answer = EngineAnswer(engine.newDocument(settings.toEngineJson().toString()))
        if (!answer.isOk) {
            show(answer.error)
            return
        }
        DocumentSettings.keepDefaults(context, settings)
        documentUri = null
        afterDocumentChange()
    }

    /** Back to the splash (Home): the document stays open until another replaces it. */
    fun home() {
        keepCamera()
        screen = Screen.SPLASH
        recentsRevision++
    }

    /** Take [uri] off the recent files. */
    fun forget(uri: Uri) {
        recents.remove(uri)
        recentsRevision++
    }

    /** Empty the recent files. */
    fun clearRecents() {
        recents.clear()
        recentsRevision++
    }

    /** The canvas view's size: fit the artwork the first time it is known. */
    fun onViewSize(w: Int, h: Int) {
        val first = viewW == 0
        viewW = w
        viewH = h
        if (first) restoreCamera()
    }

    /** Fit the first artboard in the view; on an infinite canvas, the artwork (else the origin at 100 %). */
    fun fitArtwork() {
        if (viewW == 0) return
        val a = EngineAnswer(engine.state()).okObject?.optJSONArray("artboards")?.optJSONObject(0)
            ?: EngineAnswer(engine.execute("export.slice.list", "{}")).okObject?.optJSONObject("contentBounds")
        camera = if (a == null) {
            Camera(0.0, 0.0, 1.0, camera.flipped)
        } else {
            Camera.fit(a.optDouble("x"), a.optDouble("y"), a.optDouble("width"), a.optDouble("height"), viewW, viewH).copy(flipped = camera.flipped)
        }
    }

    /** Zoom the view by [factor] about its centre. */
    fun zoomBy(factor: Double) {
        if (viewW == 0) return
        camera = camera.zoomedAbout(factor, viewW / 2f, viewH / 2f, viewW, viewH)
    }

    /** The view at 100 % (one screen pixel a point), its centre kept. */
    fun actualSize() {
        camera = camera.copy(zoom = 1.0)
    }

    /** Keep the camera with the document, so a save writes it (§12). */
    private fun keepCamera() {
        engine.setView(JSONObject().put("x", camera.cx).put("y", camera.cy).put("zoom", camera.zoom).toString())
    }

    /** The camera the document keeps, else fit the artwork. */
    private fun restoreCamera() {
        val v = runCatching { JSONObject(engine.view()).optJSONObject("ok") }.getOrNull()
        if (v != null && viewW > 0) camera = Camera(v.optDouble("x"), v.optDouble("y"), v.optDouble("zoom", 1.0), camera.flipped) else fitArtwork()
    }

    /** The view state the tools take: the zoom and the view's frame in document points. */
    fun viewJson(): String {
        val v = camera.viewport(viewW.coerceAtLeast(1), viewH.coerceAtLeast(1))
        val frame = JSONObject().put("x", v[0]).put("y", v[1]).put("width", v[2]).put("height", v[3])
        return JSONObject().put("zoom", camera.zoom).put("viewport", frame).toString()
    }

    /** Make tool [index] the active one (the dial turned it into the sight). */
    fun chooseTool(index: Int) {
        if (index == activeIndex) return
        show(activeTool.deactivate(engine))
        DebugMode.log("tool", "chose ${tools[index].name}")
        activeIndex = index
        settingsOpen = false
        show(activeTool.activate(engine, viewJson()))
        revision++
    }

    /** Feed a canvas pointer event to the active tool. */
    fun pointer(event: ToolPointer) {
        val started = System.nanoTime()
        val r = activeTool.pointer(engine, event, viewJson())
        if (event.kind != ToolPointer.Kind.DRAG || r.error != null) {
            DebugMode.log("tool", "${activeTool.name} ${event.kind} (${"%.2f".format(event.x)}, ${"%.2f".format(event.y)}) in ${(System.nanoTime() - started) / 1_000_000} ms → created ${r.created} changed ${r.changed} error ${r.error}")
        }
        show(r)
        if (event.kind == ToolPointer.Kind.UP || event.kind == ToolPointer.Kind.DOWN || r.changed) revision++
    }

    /** Undo or redo one step. */
    fun undo() = step("edit.undo")
    fun redo() = step("edit.redo")

    private fun step(command: String) {
        show(EngineAnswer(engine.execute(command, "{}")).error)
        (activeTool as? ExportTool)?.refresh(engine)
        revision++
    }

    /** View Flip: mirror the view only. */
    fun flip() {
        camera = camera.copy(flipped = !camera.flipped)
    }

    /** Swap the primary and alternate colours (the colour control's tap). */
    fun swapColors() {
        val values = JSONObject().put("stroke", alternate ?: JSONObject.NULL).put("fill", primary ?: JSONObject.NULL)
        readColors(engine.setPaintDefaults(values.toString()))
    }

    /** Set the [which] colour ("primary" or "alternate") to [hex], noted in the colour history. */
    fun setColor(which: String, hex: String) {
        val key = if (which == "primary") "stroke" else "fill"
        readColors(engine.setPaintDefaults(JSONObject().put(key, hex).toString()))
        colors.used(hex)
    }

    /** Read the colours from an answer of the paint defaults (else ask the engine). */
    private fun readColors(answer: String = engine.paintDefaults()) {
        val c = EngineAnswer(answer).okObject ?: return
        primary = c.optString("stroke").takeIf { !c.isNull("stroke") && it.isNotEmpty() }
        alternate = c.optString("fill").takeIf { !c.isNull("fill") && it.isNotEmpty() }
    }

    /** Run an engine command for a panel and read the document again → whether it worked. */
    fun run(command: String, params: JSONObject): Boolean {
        val error = EngineAnswer(engine.execute(command, params.toString())).error
        DebugMode.log("engine", "$command $params → ${error ?: "ok"}")
        show(error)
        revision++
        return error == null
    }

    /** Save to [uri] (the document's own, or a new one from Save As). */
    fun save(uri: Uri) {
        keepCamera()
        val answer = EngineAnswer(EngineFiles(context, engine).save(uri))
        if (answer.isOk) {
            documentUri = uri
            recents.add(uri, displayName(uri))
            recentsRevision++
            message = "Saved"
        } else {
            show(answer.error)
        }
    }

    /** Open the document at [uri] in place of the open one. */
    fun open(uri: Uri) {
        activeTool.deactivate(engine)
        val answer = EngineAnswer(EngineFiles(context, engine).open(uri))
        if (!answer.isOk) {
            show(answer.error)
            return
        }
        documentUri = uri
        recents.add(uri, displayName(uri))
        recentsRevision++
        afterDocumentChange()
    }

    /** [uri]'s display name. */
    private fun displayName(uri: Uri): String =
        com.extemporecreare.app.components.engine.UriReplace(context).displayName(uri).ifEmpty { uri.lastPathSegment ?: "Document" }

    /** Export the regions the Export tool's scope writes into folder [tree] (the top bar's Export). */
    fun exportTo(tree: Uri) {
        val export = tools.filterIsInstance<ExportTool>().first()
        export.refresh(engine)
        val artboards = EngineAnswer(engine.state()).okObject?.optJSONArray("artboards")
        val boxes = (0 until (artboards?.length() ?: 0)).mapNotNull { i ->
            artboards?.optJSONObject(i)?.let { DragBox.of(it.optDouble("x"), it.optDouble("y"), it.optDouble("width"), it.optDouble("height")) }
        }
        val writer = ExportSliceWriter(context)
        val regions = writer.regions(export.slices, export.scope, export.selectedId, boxes)
        if (regions.isEmpty()) {
            message = "Nothing to export"
            return
        }
        val parent = DocumentsContract.buildDocumentUriUsingTree(tree, DocumentsContract.getTreeDocumentId(tree))
        var written = 0
        for (region in regions) {
            val (mime, ext) = when (region.format) {
                "jpg" -> "image/jpeg" to "jpg"
                "webp" -> "image/webp" to "webp"
                else -> "image/png" to "png"
            }
            val target = runCatching { DocumentsContract.createDocument(context.contentResolver, parent, mime, "${region.name}.$ext") }.getOrNull()
            val error = if (target == null) "can't create ${region.name}.$ext" else writer.write(engine, region, target)
            if (error != null) show(error) else written++
        }
        if (written == regions.size) message = "Exported $written ${if (written == 1) "file" else "files"}"
    }

    /** After a new or opened document: colours, tool, camera and panels again. */
    private fun afterDocumentChange() {
        readColors()
        show(activeTool.activate(engine, viewJson()))
        restoreCamera()
        screen = Screen.EDITOR
        revision++
    }

    /** Something outside the tools changed the document (a panel, a settings window): read it again. */
    fun touched() {
        revision++
    }

    /** Step the dial one tool on ([by] 1) or back (−1), the stepped alternative to turning it (§13.2). */
    fun stepTool(by: Int) = chooseTool((activeIndex + by).mod(tools.size))

    /** Show [r]'s error, if any. */
    fun show(r: ToolResult) = show(r.error)

    /** Show [error], if not null. */
    fun show(error: String?) {
        if (error != null) message = error
    }
}

// GLOSSARY
// - EditorState: the editor's engine session, tools, active tool, camera, revision, colours, document URI and message.
// - engine: the engine session. store: the tools' saved settings.
// - tools: the dial's tools in order. activeIndex, activeTool: the active one.
// - camera: the artwork camera. revision: goes up when the document may have changed.
// - primary, alternate: the stroke and fill defaults. message: what to show. settingsOpen: the tool settings window.
// - documentUri: the open document's URI. viewW, viewH: the canvas size.
// - debugOn, penReading, renderMs: debug mode's state for the readout.
// - Screen, screen: the splash or the editor. recents, recentsRevision: the recent files. layout: the panels' layout.
// - RECOVERY_FOLDER, RECOVERY_EVERY_MS: the recovery folder and interval (2 min).
// - start(): presets, recovery setup, an interrupted session recovered. recoverInterrupted(): open its copies. saveRecovery(): write a copy.
// - stepTool(by): step the dial. newDocument(settings): a new finite or infinite document. home(): back to the splash.
// - forget(uri), clearRecents(): change the recent files.
// - zoomBy(factor), actualSize(): view controls. keepCamera(), restoreCamera(): the camera kept with the document.
// - displayName(uri): a document's name.
// - onViewSize(w, h): the canvas size; restores the camera the first time. fitArtwork(): fit the first artboard or the artwork.
// - viewJson(): the zoom and view frame for the tools.
// - chooseTool(index): make a tool active. pointer(event): feed a canvas event to the active tool.
// - undo(), redo(), step(command): one step back or forward.
// - colors, colorWorkspaceOpen: the colour workspace's library and whether it is open. setColor(which, hex): set primary or alternate.
// - confirm: a question to confirm before a destructive action.
// - flip(): mirror the view. swapColors(): swap primary and alternate. readColors(answer): read the defaults.
// - run(command, params): a panel's engine command.
// - save(uri), open(uri): the document's file. exportTo(tree): write the export regions into a folder.
// - afterDocumentChange(): colours, tool, camera and panels after a new or opened document.
// - touched(): the document changed outside the tools.
// - show(r), show(error): show an error.
