package com.extemporecreare.app.tools.export

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.DragBox
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.NoToolSettings
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import org.json.JSONObject

/**
 * The Export tool: export slice editing (`uistandards.md` §9.3, `applicationstandard.md` §4.2).
 * While it is active a drag on the canvas draws a new slice instead of painting; on release the
 * slice is created and selected and [revealDimensions] asks the UI to show its ratio and
 * dimensions. A selected slice's centre (inside) moves it and its edges and corners resize it.
 * With grid snapping on, edited edges land on the grid in document coordinates. Each change is
 * one undo step. With no slices, [autoSlice] is the automatic content slice (§9.4); editing it
 * makes it a manual slice. The engine's Hand tool stays chosen underneath and gets no input.
 */
class ExportTool(store: ToolSettingsStore) : EngineTool<NoToolSettings>(NAME, store, { NoToolSettings }) {

    /** The document's slices, grid and content bounds, as last read. */
    var slices: ExportSliceList = ExportSliceList.EMPTY
        private set

    /** The selected slice's id. */
    var selectedId: String? = null
        private set

    /** The box being drawn or edited, for the canvas to draw in place of the slice. */
    var editing: DragBox? = null
        private set

    /** Set when a slice was just drawn: the UI reveals its ratio and dimensions editor, then calls [dimensionsShown]. */
    var revealDimensions: Boolean = false
        private set

    /** What an export writes: all slices when there are any, else the automatic content slice, until the user chooses ([chooseScope]). */
    var scope: ExportScope = ExportScope.AUTO_CONTENT
        private set

    /** Whether the user chose [scope]; until then it follows whether slices exist. */
    private var scopeChosen = false

    /** The automatic content slice: the content bounds while there are no slices, else `null`. */
    val autoSlice: DragBox? get() = if (slices.slices.isEmpty()) slices.contentBounds else null

    /** What the drag in progress does: its grip, the slice it edits (`null`: a new slice), and where it was last. */
    private var drag: Triple<DragBox.Grip, String?, Pair<Double, Double>>? = null

    override fun engineToolId(): String = "hand"

    override fun activate(engine: EngineBridge, viewJson: String): ToolResult {
        val result = super.activate(engine, viewJson)
        return result.error?.let { ToolResult.failed(it) } ?: refresh(engine)
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        drag = null
        editing = null
        return super.deactivate(engine)
    }

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult = when (event.kind) {
        ToolPointer.Kind.DOWN -> press(event, viewJson)
        ToolPointer.Kind.DRAG -> dragTo(event)
        ToolPointer.Kind.UP -> dragTo(event).error?.let { ToolResult.failed(it) } ?: release(engine)
        ToolPointer.Kind.MOVE -> ToolResult.NONE
    }

    /** Read the slices again from the document (after undo, redo or a document switch). */
    fun refresh(engine: EngineBridge): ToolResult = attempt {
        slices = ExportSliceCommands.list(engine)
        if (slices.slices.none { it.id == selectedId }) selectedId = null
    }

    /** Choose what an export writes. */
    fun chooseScope(chosen: ExportScope) {
        scope = chosen
        scopeChosen = true
    }

    /** Select slice [id] (`null`: none). */
    fun select(id: String?) {
        selectedId = id?.takeIf { s -> slices.slices.any { it.id == s } }
    }

    /** The UI has shown the new slice's ratio and dimensions editor. */
    fun dimensionsShown() {
        revealDimensions = false
    }

    /** Give the selected slice the aspect ratio [x]:[y], resizing it about its centre (width kept). */
    fun setAspect(engine: EngineBridge, x: Double, y: Double): ToolResult {
        val slice = selected() ?: return ToolResult.failed("No slice is selected")
        if (!(x > 0 && y > 0)) return ToolResult.failed("Both ratio values must be above 0")
        val box = slice.box().apply { resizeAboutCentre(width, width * y / x) }
        return change(engine, slice.id, rectFields(box).put("aspectRatio", JSONObject().put("x", x).put("y", y)))
    }

    /** Give the selected slice [width] × [height] document pixels about its centre; the ratio follows. */
    fun setDimensions(engine: EngineBridge, width: Double, height: Double): ToolResult {
        val slice = selected() ?: return ToolResult.failed("No slice is selected")
        if (!(width > 0 && height > 0)) return ToolResult.failed("The width and height must be above 0")
        val box = slice.box().apply { resizeAboutCentre(width, height) }
        return change(engine, slice.id, rectFields(box).put("aspectRatio", JSONObject().put("x", width).put("y", height)))
    }

    /** Change slice [id]'s output settings ([fields]: `name`, `format`, `scale`, `exportProfileRef`, `visible`). */
    fun setOutput(engine: EngineBridge, id: String, fields: JSONObject): ToolResult = change(engine, id, fields)

    /** Copy slice [id]; the copy is selected. */
    fun duplicate(engine: EngineBridge, id: String): ToolResult = attempt {
        selectedId = ExportSliceCommands.duplicate(engine, id)
        slices = ExportSliceCommands.list(engine)
    }

    /** Remove slice [id]. */
    fun delete(engine: EngineBridge, id: String): ToolResult = attempt {
        ExportSliceCommands.delete(engine, id)
        if (selectedId == id) selectedId = null
        slices = ExportSliceCommands.list(engine)
    }

    /** Set the export grid. */
    fun setGrid(engine: EngineBridge, grid: ExportGrid): ToolResult = attempt {
        ExportSliceCommands.setGrid(engine, grid)
        slices = ExportSliceCommands.list(engine)
    }

    /** Start a drag: on the selected slice's edges, corners or centre, on another slice (selecting it), on the automatic slice, or a new slice. */
    private fun press(event: ToolPointer, viewJson: String): ToolResult {
        val reach = DragBox.reach(REACH_PX, viewJson)
        val at = event.x to event.y
        val chosen = selected()?.let { s -> s.box().gripAt(event.x, event.y, reach)?.let { s to it } }
            ?: slices.slices.lastOrNull { it.visible && it.box().contains(event.x, event.y) }?.let { it to DragBox.Grip.INSIDE }
        if (chosen != null) {
            selectedId = chosen.first.id
            editing = chosen.first.box()
            drag = Triple(chosen.second, chosen.first.id, at)
            return ToolResult.NONE
        }
        val auto = autoSlice?.copy()
        val autoGrip = auto?.gripAt(event.x, event.y, reach)
        if (auto != null && autoGrip != null) {
            editing = auto
            drag = Triple(autoGrip, null, at)
            return ToolResult.NONE
        }
        editing = DragBox(event.x, event.y, event.x + DragBox.MIN, event.y + DragBox.MIN)
        drag = Triple(DragBox.Grip.BOTTOM_RIGHT, null, at)
        return ToolResult.NONE
    }

    /** Move the dragged edges to [event], on the grid when snapping is on. */
    private fun dragTo(event: ToolPointer): ToolResult {
        val (grip, id, last) = drag ?: return ToolResult.NONE
        val box = editing ?: return ToolResult.NONE
        box.drag(grip, event.x - last.first, event.y - last.second)
        if (slices.grid.snapEnabled) box.snap(grip, slices.grid.spacingPx)
        drag = Triple(grip, id, event.x to event.y)
        return ToolResult.NONE
    }

    /** End the drag: create the drawn slice, or store the edited one, as one undo step. */
    private fun release(engine: EngineBridge): ToolResult {
        val (_, id, _) = drag ?: return ToolResult.NONE
        val box = editing ?: return ToolResult.NONE
        drag = null
        editing = null
        return attempt {
            if (id == null) {
                selectedId = ExportSliceCommands.add(engine, box)
                revealDimensions = true
            } else if (slices.slices.firstOrNull { it.id == id }?.box()?.sameAs(box) != true) {
                ExportSliceCommands.move(engine, id, box)
            }
            slices = ExportSliceCommands.list(engine)
        }
    }

    /** Until the user chooses a scope: all slices when there are any, else the automatic content slice. */
    private fun followScope() {
        if (!scopeChosen) scope = if (slices.slices.isEmpty()) ExportScope.AUTO_CONTENT else ExportScope.ALL_SLICES
    }

    /** The selected slice, or `null`. */
    private fun selected(): ExportSlice? = slices.slices.firstOrNull { it.id == selectedId }

    /** Change slice [id] by [fields] and read the slices again. */
    private fun change(engine: EngineBridge, id: String, fields: JSONObject): ToolResult = attempt {
        ExportSliceCommands.update(engine, id, fields)
        slices = ExportSliceCommands.list(engine)
    }

    /** [box]'s region as slice fields. */
    private fun rectFields(box: DragBox): JSONObject =
        JSONObject().put("x", box.left).put("y", box.top).put("width", box.width).put("height", box.height)

    /** Run [block], turning an engine failure into a [ToolResult] error. */
    private fun attempt(block: () -> Unit): ToolResult = try {
        block()
        followScope()
        ToolResult.NONE
    } catch (e: IllegalStateException) {
        ToolResult.failed(e.message ?: "export slice change failed")
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Export"

        /** How near an edge or corner a press grips it, in screen pixels. */
        const val REACH_PX = 24.0
    }
}

// GLOSSARY
// - ExportTool: export slice editing on the canvas (draw, select, move, resize, snap), one undo step per change.
// - slices: the slices, grid and content bounds as last read.
// - selectedId: the selected slice.
// - editing: the box being drawn or edited, for drawing.
// - revealDimensions: a slice was just drawn; the UI shows its ratio and dimensions.
// - scope: what an export writes; scopeChosen: whether the user chose it.
// - autoSlice: the automatic content slice while there are no slices.
// - drag: the drag's grip, slice (null: new) and last position.
// - engineToolId(): "hand" (gets no input).
// - activate, deactivate, pointer: CanvasTool.
// - refresh(engine): read the slices again.
// - chooseScope(chosen): choose what an export writes.
// - followScope(): all slices or the automatic slice until a scope is chosen.
// - select(id): select a slice.
// - dimensionsShown(): the UI showed the new slice's editor.
// - setAspect(engine, x, y): ratio about the centre.
// - setDimensions(engine, width, height): size about the centre.
// - setOutput(engine, id, fields): name, format, scale, profile, visibility.
// - duplicate(engine, id), delete(engine, id): copy or remove a slice.
// - setGrid(engine, grid): the export grid.
// - press(event, viewJson), dragTo(event), release(engine): the drag.
// - selected(): the selected slice.
// - change(engine, id, fields): update a slice and read again.
// - rectFields(box): a region as slice fields.
// - attempt(block): run, turning failures into errors.
// - NAME: "Export". REACH_PX: grip reach, 24 screen px.
