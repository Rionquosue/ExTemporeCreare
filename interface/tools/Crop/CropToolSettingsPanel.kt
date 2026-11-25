package com.extemporecreare.app.ui.tools.crop

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.tools.crop.CropMode
import com.extemporecreare.app.tools.crop.CropTool
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingNote
import com.extemporecreare.app.ui.tools.ToolSettingsWindow

/**
 * The Crop tool's settings window: what it crops, Apply and Cancel. Cropping an image removes its
 * pixels outside the box, so Apply first warns and offers to save ([requestSave]), as destructive
 * tools must (`script_structure.md`, UI rule 4); cropping the canvas leaves the art in place.
 */
@Composable
fun CropToolSettingsPanel(tool: CropTool, engine: EngineBridge, viewJson: String, onResult: (ToolResult) -> Unit, requestSave: () -> Unit) {
    var warn by remember { mutableStateOf(false) }
    val mode = tool.mode
    ToolSettingsWindow(CropTool.NAME) {
        SettingNote(
            when (mode) {
                CropMode.IMAGE -> "Crops the selected image"
                CropMode.CANVAS -> "Crops the canvas; the artwork stays where it is"
                CropMode.UNAVAILABLE -> "Select one image to crop it. An infinite canvas has no bounds to crop."
            },
        )
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            SettingButton("Cancel", { onResult(tool.cancel(engine)) }, enabled = mode != CropMode.UNAVAILABLE)
            SettingButton("Apply", {
                if (mode == CropMode.IMAGE) warn = true else onResult(tool.apply(engine, viewJson))
            }, primary = true, enabled = mode != CropMode.UNAVAILABLE)
        }
    }
    if (warn) {
        AlertDialog(
            onDismissRequest = { warn = false },
            title = { Text("Crop the image?") },
            text = { Text("The image's pixels outside the box are removed. You can undo it, or save first to keep a copy.") },
            confirmButton = { TextButton(onClick = { warn = false; onResult(tool.apply(engine, viewJson)) }) { Text("Crop") } },
            dismissButton = {
                Row {
                    TextButton(onClick = { warn = false; requestSave() }) { Text("Save first") }
                    TextButton(onClick = { warn = false }) { Text("Cancel") }
                }
            },
        )
    }
}

// GLOSSARY
// - CropToolSettingsPanel(tool, engine, viewJson, onResult, requestSave): the Crop settings window with Apply, Cancel and the image-crop warning.
