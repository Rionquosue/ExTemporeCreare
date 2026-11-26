package com.extemporecreare.app.ui.splash

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingNote

/**
 * The splash's Settings: the New Document defaults (the last settings a document was made with),
 * clearing the recent files, and resetting the interface layout.
 */
@Composable
fun SettingsDialog(defaultsSummary: String, onEditDefaults: () -> Unit, onClearRecents: () -> Unit, onResetLayout: () -> Unit, onClose: () -> Unit) {
    Dialog(onDismissRequest = onClose) {
        Column(
            Modifier.widthIn(max = 460.dp).background(EtcColors.Shell, RoundedCornerShape(16.dp)).border(1.dp, EtcColors.Line, RoundedCornerShape(16.dp)).padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("Settings", color = EtcColors.Text, fontSize = 18.sp)
            Text("New Document defaults", color = EtcColors.Muted, fontSize = 12.sp)
            SettingNote(defaultsSummary)
            SettingButton("Edit defaults…", onEditDefaults)
            Text("Recent files", color = EtcColors.Muted, fontSize = 12.sp)
            SettingButton("Clear recent files", onClearRecents)
            Text("Interface", color = EtcColors.Muted, fontSize = 12.sp)
            SettingButton("Reset panel layout", onResetLayout)
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End, verticalAlignment = Alignment.CenterVertically) {
                SettingButton("Done", onClose, primary = true)
            }
        }
    }
}

// GLOSSARY
// - SettingsDialog(defaultsSummary, onEditDefaults, onClearRecents, onResetLayout, onClose): the splash's Settings.
