package com.extemporecreare.app.ui.tools.brush

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.brush.BrushTool
import com.extemporecreare.app.tools.brush.BrushToolSettings
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.SettingSwitch
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/** The raster Brush tool's settings window: size, hardness, spacing, flow, opacity and pressure sizing. */
@Composable
fun BrushToolSettingsPanel(tool: BrushTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    fun set(changed: BrushToolSettings) {
        s = changed
        onResult(tool.update(engine, changed))
    }
    ToolSettingsWindow(BrushTool.NAME) {
        SettingSlider("Size", s.size, 0.1, 5000.0, "pt", { set(s.copy(size = it)) }, curve = 3.0)
        SettingSlider("Hardness", s.hardness, 0.0, 100.0, "%", { set(s.copy(hardness = it)) }, whole = true)
        SettingSlider("Spacing", s.spacing, 1.0, 1000.0, "%", { set(s.copy(spacing = it)) }, curve = 2.0, whole = true)
        SettingSlider("Flow", s.flow, 1.0, 100.0, "%", { set(s.copy(flow = it)) }, whole = true)
        SettingSlider("Opacity", s.opacity, 1.0, 100.0, "%", { set(s.copy(opacity = it)) }, whole = true)
        SettingSwitch("Pressure sets size", s.pressureSize) { set(s.copy(pressureSize = it)) }
    }
}

// GLOSSARY
// - BrushToolSettingsPanel(tool, engine, onResult): the raster Brush settings window.
