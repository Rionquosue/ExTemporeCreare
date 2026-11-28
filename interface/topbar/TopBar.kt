package com.extemporecreare.app.ui.topbar

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Redo
import androidx.compose.material.icons.automirrored.outlined.Undo
import androidx.compose.material.icons.outlined.Flip
import androidx.compose.material.icons.outlined.IosShare
import androidx.compose.material.icons.outlined.Menu
import androidx.compose.material.icons.outlined.Save
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors

/** The main menu's entries. */
data class MenuEntry(val label: String, val action: () -> Unit)

/**
 * The top bar (`uistandards.md` §4): Menu at the far left (a rounded square opening [menu]), the
 * essential actions Save, Undo, Redo, Flip and Export as pills with an icon and a label. No
 * branding while the app is in use (the owner's decision; the splash keeps it).
 */
@Composable
fun TopBar(
    menu: List<MenuEntry>,
    onSave: () -> Unit,
    onFlip: () -> Unit,
    flipped: Boolean,
    onUndo: () -> Unit,
    onRedo: () -> Unit,
    onExport: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier.fillMaxWidth().height(BAR_HEIGHT).background(EtcColors.Shell).padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        var open by remember { mutableStateOf(false) }
        Box {
            val shape = RoundedCornerShape(14.dp)
            Box(
                Modifier.size(48.dp).shadow(6.dp, shape, ambientColor = EtcColors.Bg, spotColor = EtcColors.Bg)
                    .background(EtcColors.Panel2, shape).border(1.dp, EtcColors.Line, shape)
                    .clickable { open = true }.semantics { contentDescription = "Menu" },
                contentAlignment = Alignment.Center,
            ) { Icon(Icons.Outlined.Menu, null, tint = EtcColors.Text, modifier = Modifier.size(24.dp)) }
            DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                for (entry in menu) DropdownMenuItem(text = { Text(entry.label) }, onClick = { open = false; entry.action() })
            }
        }
        Pill(Icons.Outlined.Save, "Save", onSave)
        Pill(Icons.AutoMirrored.Outlined.Undo, "Undo", onUndo)
        Pill(Icons.AutoMirrored.Outlined.Redo, "Redo", onRedo)
        Pill(Icons.Outlined.Flip, "Flip", onFlip, active = flipped)
        Pill(Icons.Outlined.IosShare, "Export", onExport)
    }
}

/** A pill button: an icon and a label, gold-edged when [active] (Flip while the view is flipped). */
@Composable
private fun Pill(icon: ImageVector, label: String, onClick: () -> Unit, active: Boolean = false) {
    val shape = RoundedCornerShape(22.dp)
    Row(
        Modifier
            .height(44.dp)
            .shadow(6.dp, shape, ambientColor = EtcColors.Bg, spotColor = EtcColors.Bg)
            .background(EtcColors.Panel2, shape)
            .border(1.dp, if (active) EtcColors.Gold else EtcColors.Line, shape)
            .clickable(onClick = onClick)
            .semantics { contentDescription = label }
            .padding(horizontal = 18.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Icon(icon, null, tint = if (active) EtcColors.Gold else EtcColors.Text, modifier = Modifier.size(20.dp))
        Text(label, color = if (active) EtcColors.Gold else EtcColors.Text, fontSize = 15.sp)
    }
}

/** The top bar's height: room for its 48 dp controls (§4: starting proportions, subject to touch readability). */
val BAR_HEIGHT = 68.dp

// GLOSSARY
// - MenuEntry: a main menu entry (label, action).
// - TopBar(menu, onSave, onFlip, flipped, onUndo, onRedo, onExport, modifier): the top bar.
// - Pill(icon, label, onClick, active): a pill button with an icon and a label.
// - BAR_HEIGHT: the top bar's height, 68 dp.
