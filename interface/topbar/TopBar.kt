package com.extemporecreare.app.ui.topbar

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Redo
import androidx.compose.material.icons.automirrored.outlined.Undo
import androidx.compose.material.icons.outlined.Flip
import androidx.compose.material.icons.outlined.IosShare
import androidx.compose.material.icons.outlined.Menu
import androidx.compose.material.icons.outlined.Palette
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors

/** The main menu's entries. */
data class MenuEntry(val label: String, val action: () -> Unit)

/**
 * The top bar (`uistandards.md` §4): Menu at the far left (a rounded square opening [menu]), the
 * essential actions Save, Flip, Undo, Redo and Export as compact pills, and the brand at the
 * right end.
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
        modifier.fillMaxWidth().height(BAR_HEIGHT).background(EtcColors.Shell).padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        var open by remember { mutableStateOf(false) }
        Box {
            Box(
                Modifier.size(38.dp).background(EtcColors.Panel2, RoundedCornerShape(10.dp)).border(1.dp, EtcColors.Line, RoundedCornerShape(10.dp))
                    .clickable { open = true }.semantics { contentDescription = "Menu" },
                contentAlignment = Alignment.Center,
            ) { Icon(Icons.Outlined.Menu, null, tint = EtcColors.Text, modifier = Modifier.size(19.dp)) }
            DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
                for (entry in menu) DropdownMenuItem(text = { Text(entry.label) }, onClick = { open = false; entry.action() })
            }
        }
        Pill(Icons.Outlined.Save, "Save", onSave)
        Pill(Icons.Outlined.Flip, "Flip", onFlip, active = flipped)
        Pill(Icons.AutoMirrored.Outlined.Undo, null, onUndo, description = "Undo")
        Pill(Icons.AutoMirrored.Outlined.Redo, null, onRedo, description = "Redo")
        Pill(Icons.Outlined.IosShare, "Export", onExport, active = true)
        Spacer(Modifier.weight(1f))
        Box(
            Modifier.size(30.dp).border(1.dp, EtcColors.Gold2, CircleShape),
            contentAlignment = Alignment.Center,
        ) { Icon(Icons.Outlined.Palette, null, tint = EtcColors.Gold, modifier = Modifier.size(16.dp)) }
        Spacer(Modifier.width(2.dp))
        Text("Ex-Tempore ", color = EtcColors.Text, fontFamily = FontFamily.Serif, fontStyle = FontStyle.Italic, fontSize = 15.sp)
        Text("Creare", color = EtcColors.Gold, fontFamily = FontFamily.Serif, fontStyle = FontStyle.Italic, fontSize = 15.sp)
    }
}

/** A compact pill button: an icon and an optional label, gold-tinted when [active]. */
@Composable
private fun Pill(icon: ImageVector, label: String?, onClick: () -> Unit, active: Boolean = false, description: String? = label) {
    Row(
        Modifier
            .height(34.dp)
            .background(if (active) EtcColors.Panel2 else EtcColors.Shell, RoundedCornerShape(17.dp))
            .border(1.dp, if (active) EtcColors.Gold2 else EtcColors.Line, RoundedCornerShape(17.dp))
            .clickable(onClick = onClick)
            .semantics { contentDescription = description ?: "" }
            .padding(horizontal = if (label == null) 10.dp else 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Icon(icon, null, tint = if (active) EtcColors.Gold else EtcColors.Text, modifier = Modifier.size(17.dp))
        if (label != null) Text(label, color = EtcColors.Text, fontSize = 13.sp)
    }
}

/** The top bar's height (about 51 px in the mockup, §4). */
val BAR_HEIGHT = 52.dp

// GLOSSARY
// - MenuEntry: a main menu entry (label, action).
// - TopBar(menu, onSave, onFlip, flipped, onUndo, onRedo, onExport, modifier): the top bar.
// - Pill(icon, label, onClick, active, description): a compact pill button.
// - BAR_HEIGHT: the top bar's height, 52 dp.
