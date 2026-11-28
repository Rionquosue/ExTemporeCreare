package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.DragIndicator
import androidx.compose.material.icons.outlined.ExpandLess
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * A floating panel (`uistandards.md` §3.2, §10): a rounded translucent surface with a thin border
 * and a soft shadow. Expanded, [width] wide: its header (a grip, the [title] underlined in gold, the
 * [count] in a badge, [actions] and a collapse chevron) over the body ([content]). Collapsed: a
 * compact pill with the [icon], the title and the count. Tapping the header or the pill collapses
 * or restores it ([onToggle]); holding it starts a move ([headerModifier], from the panel's host).
 */
@Composable
fun FloatingPanel(
    title: String,
    icon: ImageVector,
    count: Int?,
    expanded: Boolean,
    onToggle: () -> Unit,
    width: Dp,
    modifier: Modifier = Modifier,
    headerModifier: Modifier = Modifier,
    actions: @Composable () -> Unit = {},
    content: @Composable ColumnScope.() -> Unit,
) {
    val shape = RoundedCornerShape(PANEL_RADIUS)
    if (!expanded) {
        Row(
            modifier
                .shadow(10.dp, shape, ambientColor = EtcColors.Bg, spotColor = EtcColors.Bg)
                .background(EtcColors.Panel, shape)
                .border(1.dp, EtcColors.Line, shape)
                .then(headerModifier)
                .clickable(onClick = onToggle)
                .semantics { contentDescription = "$title, collapsed: tap to open" }
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(Modifier.size(36.dp).border(1.dp, EtcColors.Line, RoundedCornerShape(10.dp)), contentAlignment = Alignment.Center) {
                Icon(icon, null, tint = EtcColors.Text, modifier = Modifier.size(20.dp))
            }
            Text(title, color = EtcColors.Text, fontSize = 16.sp, fontWeight = FontWeight.Medium)
            if (count != null) CountBadge(count)
        }
        return
    }
    Column(
        modifier
            .width(width)
            .shadow(14.dp, shape, ambientColor = EtcColors.Bg, spotColor = EtcColors.Bg)
            .background(EtcColors.Panel, shape)
            .border(1.dp, EtcColors.Line, shape),
    ) {
        Row(
            headerModifier.clickable(onClick = onToggle).padding(start = 10.dp, end = 12.dp, top = 12.dp, bottom = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(Icons.Outlined.DragIndicator, "Hold to move", tint = EtcColors.Soft, modifier = Modifier.size(18.dp))
            Column {
                Text(title, color = EtcColors.Gold, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                Box(Modifier.padding(top = 4.dp).width(28.dp).height(2.dp).background(EtcColors.Gold, RoundedCornerShape(1.dp)))
            }
            if (count != null) CountBadge(count)
            Spacer(Modifier.weight(1f))
            actions()
            Icon(Icons.Outlined.ExpandLess, "Collapse", tint = EtcColors.Muted, modifier = Modifier.size(22.dp))
        }
        content()
    }
}

/** A small round badge with [count]. */
@Composable
private fun CountBadge(count: Int) {
    Box(
        Modifier.defaultMinSize(24.dp, 24.dp).background(EtcColors.Panel2, CircleShape).border(1.dp, EtcColors.Line, CircleShape).padding(horizontal = 7.dp),
        contentAlignment = Alignment.Center,
    ) { Text(count.toString(), color = EtcColors.Text, fontSize = 12.sp) }
}

/** The floating panels' corner radius (`uistandards.md` §3.2: 16 px). */
val PANEL_RADIUS = 16.dp

// GLOSSARY
// - FloatingPanel(title, icon, count, expanded, onToggle, width, modifier, headerModifier, actions, content): a floating panel; collapsed, a pill.
// - CountBadge(count): a round count badge.
// - PANEL_RADIUS: the panels' corner radius, 16 dp.
