package com.extemporecreare.app.ui.tools.fill

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.fill.FillTool
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/** The Fill tool's settings window: its tolerance, the widest gap between paths that still closes an area. */
@Composable
fun FillToolSettingsPanel(tool: FillTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    ToolSettingsWindow(FillTool.NAME) {
        SettingSlider("Tolerance", s.tolerance, 0.0, MAX_TOLERANCE, "pt", {
            s = s.copy(tolerance = it)
            onResult(tool.update(engine, s))
        }, curve = 2.0)
    }
}

/** The slider's largest tolerance, in points. */
private const val MAX_TOLERANCE = 72.0

// GLOSSARY
// - FillToolSettingsPanel(tool, engine, onResult): the Fill settings window.
// - MAX_TOLERANCE: the slider's largest tolerance, 72 pt.
