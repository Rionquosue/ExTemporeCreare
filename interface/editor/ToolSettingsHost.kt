package com.extemporecreare.app.ui.editor

import androidx.compose.runtime.Composable
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.brush.BrushTool
import com.extemporecreare.app.tools.crop.CropTool
import com.extemporecreare.app.tools.eraser.EraserTool
import com.extemporecreare.app.tools.fill.FillTool
import com.extemporecreare.app.tools.marker.MarkerTool
import com.extemporecreare.app.tools.pen.PenTool
import com.extemporecreare.app.tools.pencil.PencilTool
import com.extemporecreare.app.tools.shapes.ShapesTool
import com.extemporecreare.app.ui.tools.brush.BrushToolSettingsPanel
import com.extemporecreare.app.ui.tools.crop.CropToolSettingsPanel
import com.extemporecreare.app.ui.tools.eraser.EraserToolSettingsPanel
import com.extemporecreare.app.ui.tools.fill.FillToolSettingsPanel
import com.extemporecreare.app.ui.tools.marker.MarkerToolSettingsPanel
import com.extemporecreare.app.ui.tools.pen.PenToolSettingsPanel
import com.extemporecreare.app.ui.tools.pencil.PencilToolSettingsPanel
import com.extemporecreare.app.ui.tools.shapes.ShapesToolSettingsPanel

/**
 * The active tool's settings window (`uistandards.md` §6.2), or nothing for a tool without one
 * (Selection, Move; Export's settings take the Layers panel's place instead). [requestSave] is what
 * the Crop tool's warning offers before cropping an image.
 */
@Composable
fun ToolSettingsHost(state: EditorState, requestSave: () -> Unit) {
    val show = { r: ToolResult -> state.show(r) }
    when (val tool = state.activeTool) {
        is BrushTool -> BrushToolSettingsPanel(tool, state.engine, show)
        is MarkerTool -> MarkerToolSettingsPanel(tool, state.engine, show)
        is PencilTool -> PencilToolSettingsPanel(tool, state.engine, show)
        is PenTool -> PenToolSettingsPanel(tool, state.engine, show)
        is EraserTool -> EraserToolSettingsPanel(tool, state.engine, show)
        is FillTool -> FillToolSettingsPanel(tool, state.engine, show)
        is ShapesTool -> ShapesToolSettingsPanel(tool, state.engine, show)
        is CropTool -> CropToolSettingsPanel(tool, state.engine, state.viewJson(), { state.show(it); state.touched() }, requestSave)
        else -> {}
    }
}

// GLOSSARY
// - ToolSettingsHost(state, requestSave): the active tool's settings window.
