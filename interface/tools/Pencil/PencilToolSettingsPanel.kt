package com.extemporecreare.app.ui.tools.pencil

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.StrokeMode
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.pencil.PencilTip
import com.extemporecreare.app.tools.pencil.PencilTool
import com.extemporecreare.app.tools.pencil.PencilToolSettings
import com.extemporecreare.app.ui.tools.SettingChoice
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/**
 * The Pencil tool's settings window: mode, size and hardness; the tip (cone or mechanical) and, for
 * a cone, how sharp it is; and the paper grain's scale and strength. Each change is saved and applied
 * through [PencilTool.update]; [onResult] shows its errors.
 */
@Composable
fun PencilToolSettingsPanel(tool: PencilTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    fun set(changed: PencilToolSettings) {
        s = changed
        onResult(tool.update(engine, changed))
    }
    ToolSettingsWindow(PencilTool.NAME) {
        SettingChoice("Mode", listOf(StrokeMode.PAINTBRUSH to "Paintbrush", StrokeMode.BLOB to "Blob Brush"), s.mode) { set(s.copy(mode = it)) }
        SettingSlider("Size", s.size, 0.1, 1000.0, "pt", { set(s.copy(size = it)) }, curve = 3.0)
        SettingSlider("Hardness", s.hardness, 0.0, 100.0, "%", { set(s.copy(hardness = it)) }, whole = true)
        SettingChoice("Tip", listOf(PencilTip.CONE to "Cone", PencilTip.MECHANICAL to "Mechanical"), s.tip) { set(s.copy(tip = it)) }
        if (s.tip == PencilTip.CONE) {
            SettingSlider("Cone sharpness", s.coneAngle, 1.0, 80.0, "°", { set(s.copy(coneAngle = it)) }, whole = true)
        }
        SettingSlider("Grain size", s.grainScale, 0.05, 50.0, "pt", { set(s.copy(grainScale = it)) }, curve = 3.0)
        SettingSlider("Grain strength", s.grainStrength, 0.0, 100.0, "%", { set(s.copy(grainStrength = it)) }, whole = true)
    }
}

// GLOSSARY
// - PencilToolSettingsPanel(tool, engine, onResult): the Pencil settings window.
