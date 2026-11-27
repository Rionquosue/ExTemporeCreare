package com.extemporecreare.app.ui.dial

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * The dial docked to an edge as a toolbar (the grip dragged to the edge, [DialGrip]): vertical on
 * the left or right edge, horizontal on the top or bottom, with the grip ([grip]), the colour
 * control ([colors], shown smaller) and the tools in the dial's order. Tapping a tool chooses it
 * ([onChoose]); tapping the active one opens or closes its settings ([onActiveTap]). It scrolls
 * when the edge is too short for all of it.
 */
@Composable
fun DialToolbar(
    edge: String,
    names: List<String>,
    active: Int,
    onChoose: (Int) -> Unit,
    onActiveTap: () -> Unit,
    grip: @Composable () -> Unit,
    colors: @Composable (Modifier) -> Unit,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(16.dp)
    val surface = modifier.background(EtcColors.Panel, shape).border(1.dp, EtcColors.Line, shape).padding(6.dp)
    val content: @Composable () -> Unit = {
        grip()
        Box(Modifier.size(COLOR_SIZE), contentAlignment = Alignment.Center) {
            colors(Modifier.requiredSize(COLOR_CONTROL_SIZE).scale(COLOR_SIZE / COLOR_CONTROL_SIZE))
        }
        names.forEachIndexed { i, name ->
            val on = i == active
            Box(
                Modifier.size(TOOL_SIZE)
                    .background(if (on) EtcColors.Panel2 else EtcColors.Panel, RoundedCornerShape(10.dp))
                    .border(if (on) 2.dp else 1.dp, if (on) EtcColors.Gold else EtcColors.LineSoft, RoundedCornerShape(10.dp))
                    .clickable(role = Role.Tab) { if (on) onActiveTap() else onChoose(i) }
                    .semantics { contentDescription = name; selected = on },
                contentAlignment = Alignment.Center,
            ) {
                Icon(iconOf(name), null, tint = if (on) EtcColors.Gold else EtcColors.Muted, modifier = Modifier.size(22.dp))
            }
        }
    }
    if (edge == "left" || edge == "right") {
        Column(surface.verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(6.dp), horizontalAlignment = Alignment.CenterHorizontally) { content() }
    } else {
        Row(surface.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) { content() }
    }
}

/** A tool button's size, and the colour control's in the toolbar and its own full size. */
private val TOOL_SIZE = 48.dp
private val COLOR_SIZE = 52.dp
private val COLOR_CONTROL_SIZE = 76.dp

/** How far the docked toolbar reaches in from its edge (its thickness and margin), for what is placed beside it. */
val TOOLBAR_REACH = 76.dp

// GLOSSARY
// - DialToolbar(edge, names, active, onChoose, onActiveTap, grip, colors, modifier): the dial docked to an edge as a toolbar.
// - TOOL_SIZE, COLOR_SIZE, COLOR_CONTROL_SIZE: button and colour control sizes. TOOLBAR_REACH: how far it reaches in from its edge.
