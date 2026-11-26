package com.extemporecreare.app.ui.dial

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.BorderColor
import androidx.compose.material.icons.outlined.Brush
import androidx.compose.material.icons.outlined.Category
import androidx.compose.material.icons.outlined.CleaningServices
import androidx.compose.material.icons.outlined.Crop
import androidx.compose.material.icons.outlined.Draw
import androidx.compose.material.icons.outlined.Edit
import androidx.compose.material.icons.outlined.FormatColorFill
import androidx.compose.material.icons.outlined.IosShare
import androidx.compose.material.icons.outlined.NearMe
import androidx.compose.material.icons.outlined.OpenWith
import androidx.compose.ui.graphics.vector.ImageVector
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.CanvasTool
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.brush.BrushTool
import com.extemporecreare.app.tools.eraser.EraserTool
import com.extemporecreare.app.tools.marker.MarkerTool
import com.extemporecreare.app.tools.pen.PenTool
import com.extemporecreare.app.tools.pencil.PencilTool

/** The dial's icon for each tool name. */
fun iconOf(name: String): ImageVector = when (name) {
    "Brush" -> Icons.Outlined.Brush
    "Marker" -> Icons.Outlined.BorderColor
    "Pencil" -> Icons.Outlined.Edit
    "Pen" -> Icons.Outlined.Draw
    "Eraser" -> Icons.Outlined.CleaningServices
    "Fill" -> Icons.Outlined.FormatColorFill
    "Selection" -> Icons.Outlined.NearMe
    "Shapes" -> Icons.Outlined.Category
    "Move" -> Icons.Outlined.OpenWith
    "Crop" -> Icons.Outlined.Crop
    else -> Icons.Outlined.IosShare
}

/** The size the readout shows for [tool] with its unit, or null for a tool without a size (§6.2). */
fun sizeOf(tool: CanvasTool): Pair<Double, String>? = when (tool) {
    is BrushTool -> tool.settings.size to "px"
    is MarkerTool -> tool.settings.size to "pt"
    is PencilTool -> tool.settings.size to "pt"
    is PenTool -> tool.settings.size to "pt"
    is EraserTool -> tool.settings.size to "pt"
    else -> null
}

/** Give [tool] the size [value] (kept within its range), as the readout's drag does. */
fun setSize(engine: EngineBridge, tool: CanvasTool, value: Double): ToolResult = when (tool) {
    is BrushTool -> tool.update(engine, tool.settings.copy(size = value.coerceIn(0.5, 5000.0)))
    is MarkerTool -> tool.update(engine, tool.settings.copy(size = value.coerceIn(0.1, 1000.0)))
    is PencilTool -> tool.update(engine, tool.settings.copy(size = value.coerceIn(0.1, 1000.0)))
    is PenTool -> tool.update(engine, tool.settings.copy(size = value.coerceIn(0.1, 1000.0)))
    is EraserTool -> tool.update(engine, tool.settings.copy(size = value.coerceIn(0.1, 1000.0)))
    else -> ToolResult.NONE
}

// GLOSSARY
// - iconOf(name): the dial icon of a tool.
// - sizeOf(tool): the size and unit the readout shows, null for tools without one.
// - setSize(engine, tool, value): set a tool's size from the readout.
