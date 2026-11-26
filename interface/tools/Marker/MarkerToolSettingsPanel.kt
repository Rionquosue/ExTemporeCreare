package com.extemporecreare.app.ui.tools.marker

import androidx.compose.foundation.layout.Box
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.marker.MarkerMode
import com.extemporecreare.app.tools.marker.MarkerTip
import com.extemporecreare.app.tools.marker.MarkerTool
import com.extemporecreare.app.tools.marker.MarkerToolSettings
import com.extemporecreare.app.tools.marker.TipOrientation
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingChoice
import com.extemporecreare.app.ui.tools.SettingNote
import com.extemporecreare.app.ui.tools.SettingSlider
import com.extemporecreare.app.ui.tools.ToolSettingsWindow
import org.json.JSONObject

/**
 * The Marker tool's settings window: mode, size and hardness; the tip (round, oval or chisel) and,
 * for oval and chisel tips, their angle, thickness and what turns them (fixed, pen tilt, barrel
 * rotation); and for the Paintbrush the library brush it paints with (a dropdown of the built-in
 * brush libraries' brushes; "Plain stroke": none), which draws in place of the tip when chosen.
 * Each change is saved and applied through [MarkerTool.update]; [onResult] shows its errors.
 */
@Composable
fun MarkerToolSettingsPanel(tool: MarkerTool, engine: EngineBridge, onResult: (ToolResult) -> Unit) {
    var s by remember { mutableStateOf(tool.settings) }
    fun set(changed: MarkerToolSettings) {
        s = changed
        onResult(tool.update(engine, changed))
    }
    ToolSettingsWindow(MarkerTool.NAME) {
        SettingChoice("Mode", listOf(MarkerMode.PAINTBRUSH to "Paintbrush", MarkerMode.BLOB to "Blob Brush"), s.mode) { set(s.copy(mode = it)) }
        SettingSlider("Size", s.size, 0.1, 1000.0, "pt", { set(s.copy(size = it)) }, curve = 3.0)
        SettingSlider("Hardness", s.hardness, 0.0, 100.0, "%", { set(s.copy(hardness = it)) }, whole = true)
        TipControls(s, ::set)
        if (s.mode == MarkerMode.PAINTBRUSH) {
            LibraryBrushPicker(engine, s) { library, name -> set(s.copy(brushLibrary = library, brushName = name)) }
        }
    }
}

/** The tip: its shape, and for an oval or chisel tip its angle, thickness and what turns it. */
@Composable
private fun TipControls(s: MarkerToolSettings, set: (MarkerToolSettings) -> Unit) {
    SettingChoice("Tip", listOf(MarkerTip.ROUND to "Round", MarkerTip.OVAL to "Oval", MarkerTip.CHISEL to "Chisel"), s.tip) { set(s.copy(tip = it)) }
    if (s.tip == MarkerTip.ROUND) return
    SettingSlider("Tip angle", s.tipAngle, -180.0, 180.0, "°", { set(s.copy(tipAngle = it)) }, whole = true)
    SettingSlider("Tip thickness", s.tipThickness, 1.0, 100.0, "%", { set(s.copy(tipThickness = it)) }, whole = true)
    SettingChoice(
        "Turns with",
        listOf(TipOrientation.FIXED to "Fixed", TipOrientation.TILT to "Pen tilt", TipOrientation.ROTATION to "Rotation"),
        s.orientation,
    ) { set(s.copy(orientation = it)) }
    if (s.mode == MarkerMode.PAINTBRUSH && s.brushName != null) SettingNote("The library brush draws in place of the tip")
}

/** A dropdown of the built-in library brushes (`brush.libraries`, `brush.library`), with "Plain stroke" first. */
@Composable
private fun LibraryBrushPicker(engine: EngineBridge, s: MarkerToolSettings, onPick: (String?, String?) -> Unit) {
    var open by remember { mutableStateOf(false) }
    val brushes = remember(open) { if (open) libraryBrushes(engine) else emptyList() }
    SettingNote("Brush: ${s.brushName ?: "Plain stroke"}")
    Box {
        SettingButton("Choose brush…", { open = true })
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(text = { Text("Plain stroke") }, onClick = { open = false; onPick(null, null) })
            for ((library, name, label) in brushes) {
                DropdownMenuItem(text = { Text(label) }, onClick = { open = false; onPick(library, name) })
            }
        }
    }
}

/** Every built-in library brush as (library id, brush name, "Library › Brush"). */
private fun libraryBrushes(engine: EngineBridge): List<Triple<String, String, String>> {
    val libraries = EngineAnswer(engine.execute("brush.libraries", "{}")).okObject?.optJSONArray("libraries") ?: return emptyList()
    val out = mutableListOf<Triple<String, String, String>>()
    for (i in 0 until libraries.length()) {
        val lib = libraries.getJSONObject(i)
        if (lib.optString("category") != "builtIn") continue
        val id = lib.optString("id")
        val detail = EngineAnswer(engine.execute("brush.library", JSONObject().put("library", id).toString())).okObject ?: continue
        val brushes = detail.optJSONArray("brushes") ?: continue
        for (j in 0 until brushes.length()) {
            val name = brushes.getJSONObject(j).optString("name")
            out.add(Triple(id, name, "${lib.optString("name")} › $name"))
        }
    }
    return out
}

// GLOSSARY
// - MarkerToolSettingsPanel(tool, engine, onResult): the Marker settings window.
// - TipControls(s, set): the tip shape, angle, thickness and what turns it.
// - LibraryBrushPicker(engine, s, onPick): the library brush dropdown.
// - libraryBrushes(engine): the built-in library brushes.
