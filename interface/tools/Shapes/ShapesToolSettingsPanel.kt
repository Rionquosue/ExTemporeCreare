package com.extemporecreare.app.ui.tools.shapes

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.shapes.ShapeKind
import com.extemporecreare.app.tools.shapes.ShapesTool
import com.extemporecreare.app.tools.shapes.ShapesToolSettings
import com.extemporecreare.app.ui.tools.SettingChoice
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/** The Shapes tool's settings window: the shape drawn, and a polygon's sides or a star's points. */
@Composable
fun ShapesToolSettingsPanel(tool: ShapesTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    fun set(changed: ShapesToolSettings) {
        s = changed
        onResult(tool.update(engine, changed))
    }
    ToolSettingsWindow(ShapesTool.NAME) {
        SettingChoice(
            "Shape",
            listOf(
                ShapeKind.RECTANGLE to "Rectangle",
                ShapeKind.ELLIPSE to "Ellipse",
                ShapeKind.POLYGON to "Polygon",
                ShapeKind.STAR to "Star",
                ShapeKind.LINE to "Line",
            ),
            s.kind,
        ) { set(s.copy(kind = it)) }
        when (s.kind) {
            ShapeKind.POLYGON -> SettingSlider("Sides", s.sides.toDouble(), 3.0, MAX_COUNT, "", { set(s.copy(sides = it.toInt())) }, whole = true)
            ShapeKind.STAR -> SettingSlider("Points", s.points.toDouble(), 3.0, MAX_COUNT, "", { set(s.copy(points = it.toInt())) }, whole = true)
            else -> {}
        }
    }
}

/** The sliders' largest side and point count. */
private const val MAX_COUNT = 100.0

// GLOSSARY
// - ShapesToolSettingsPanel(tool, engine, onResult): the Shapes settings window.
// - MAX_COUNT: the sliders' largest count, 100.
