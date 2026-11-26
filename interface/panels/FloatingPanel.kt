package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.DragIndicator
import androidx.compose.material.icons.outlined.ExpandLess
import androidx.compose.material.icons.outlined.ExpandMore
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * A floating panel (`uistandards.md` §3.2, §10): a rounded translucent surface with a thin border,
 * its header a grip, the [title], a [count] and a collapse chevron. Tapping the header collapses or
 * restores it ([onToggle]); the body ([content]) shows only while [expanded]. [actions] sit before
 * the chevron.
 */
@Composable
fun FloatingPanel(
    title: String,
    count: String?,
    expanded: Boolean,
    onToggle: () -> Unit,
    modifier: Modifier = Modifier,
    actions: @Composable () -> Unit = {},
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier
            .background(EtcColors.Panel, RoundedCornerShape(16.dp))
            .border(1.dp, EtcColors.Line, RoundedCornerShape(16.dp)),
    ) {
        Row(
            Modifier.clickable(onClick = onToggle).padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(Icons.Outlined.DragIndicator, null, tint = EtcColors.Soft, modifier = Modifier.size(16.dp))
            Text(title, color = EtcColors.Text, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
            if (count != null) Text(count, color = EtcColors.Muted, fontSize = 11.sp)
            Spacer(Modifier.width(4.dp).weight(1f, fill = false))
            actions()
            Icon(if (expanded) Icons.Outlined.ExpandLess else Icons.Outlined.ExpandMore, if (expanded) "Collapse" else "Expand", tint = EtcColors.Muted, modifier = Modifier.size(20.dp))
        }
        if (expanded) content()
    }
}

// GLOSSARY
// - FloatingPanel(title, count, expanded, onToggle, modifier, actions, content): a floating panel with a collapsible header.
