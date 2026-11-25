package com.extemporecreare.app.ui.tools.pen

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.pen.PenTool
import com.extemporecreare.app.tools.pen.PenToolSettings
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/** The Pen tool's settings window: size (the new path's stroke weight) and hardness. */
@Composable
fun PenToolSettingsPanel(tool: PenTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    fun set(changed: PenToolSettings) {
        s = changed
        onResult(tool.update(engine, changed))
    }
    ToolSettingsWindow(PenTool.NAME) {
        SettingSlider("Size", s.size, 0.1, 1000.0, "pt", { set(s.copy(size = it)) }, curve = 3.0)
        SettingSlider("Hardness", s.hardness, 0.0, 100.0, "%", { set(s.copy(hardness = it)) }, whole = true)
    }
}

// GLOSSARY
// - PenToolSettingsPanel(tool, engine, onResult): the Pen settings window.
