package com.extemporecreare.app.ui.tools.eraser

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.eraser.EraserTool
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/** The Eraser tool's settings window: its tip size. */
@Composable
fun EraserToolSettingsPanel(tool: EraserTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    ToolSettingsWindow(EraserTool.NAME) {
        SettingSlider("Size", s.size, 0.1, 1000.0, "pt", {
            s = s.copy(size = it)
            onResult(tool.update(engine, s))
        }, curve = 3.0)
    }
}

// GLOSSARY
// - EraserToolSettingsPanel(tool, engine, onResult): the Eraser settings window.
