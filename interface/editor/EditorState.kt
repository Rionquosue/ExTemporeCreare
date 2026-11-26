package com.extemporecreare.app.ui.editor

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.debug.DebugMode
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
import org.json.JSONObject

/**
 * The editor's state: the engine session, the dial's tools in order, the active tool, the camera,
 * the document's revision (which the canvas and panels follow), the colours, the open document's
 * URI and the message to show. The UI reads it and calls its actions.
 */
class EditorState(private val context: Context) {

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

    /** Install the presets, make a new document and choose the first tool. */
    fun start() {
        EngineBridge.setPresetsRoot(EnginePresets.install(context).path)
        newDocument()
    }

    /** A new document (1920 × 1080 points) in place of the open one. */
    fun newDocument() {
        activeTool.deactivate(engine)
        show(EngineAnswer(engine.execute("file.new", JSONObject().put("width", 1920).put("height", 1080).toString())).error)
        documentUri = null
        afterDocumentChange()
    }

    /** The canvas view's size: fit the artwork the first time it is known. */
    fun onViewSize(w: Int, h: Int) {
        val first = viewW == 0
        viewW = w
        viewH = h
        if (first) fitArtwork()
    }

    /** Fit the first artboard in the view. */
    fun fitArtwork() {
        if (viewW == 0) return
        val a = EngineAnswer(engine.state()).okObject?.optJSONArray("artboards")?.optJSONObject(0) ?: return
        camera = Camera.fit(a.optDouble("x"), a.optDouble("y"), a.optDouble("width"), a.optDouble("height"), viewW, viewH).copy(flipped = camera.flipped)
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
        val answer = EngineAnswer(EngineFiles(context, engine).save(uri))
        if (answer.isOk) {
            documentUri = uri
            message = "Saved"
        } else {
            show(answer.error)
        }
    }

    /** Open the document at [uri] in place of the open one. */
    fun open(uri: Uri) {
        activeTool.deactivate(engine)
        val answer = EngineAnswer(EngineFiles(context, engine).open(uri))
        if (answer.isOk) documentUri = uri else show(answer.error)
        afterDocumentChange()
    }

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
        fitArtwork()
        revision++
    }

    /** Something outside the tools changed the document (a panel, a settings window): read it again. */
    fun touched() {
        revision++
    }

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
// - start(): presets, a new document, the first tool. newDocument(): a new 1920 × 1080 document.
// - onViewSize(w, h): the canvas size; fits the artwork the first time. fitArtwork(): fit the first artboard.
// - viewJson(): the zoom and view frame for the tools.
// - chooseTool(index): make a tool active. pointer(event): feed a canvas event to the active tool.
// - undo(), redo(), step(command): one step back or forward.
// - flip(): mirror the view. swapColors(): swap primary and alternate. readColors(answer): read the defaults.
// - run(command, params): a panel's engine command.
// - save(uri), open(uri): the document's file. exportTo(tree): write the export regions into a folder.
// - afterDocumentChange(): colours, tool, camera and panels after a new or opened document.
// - touched(): the document changed outside the tools.
// - show(r), show(error): show an error.
