package com.extemporecreare.app.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.boards.ReferenceBoard
import com.extemporecreare.app.ui.board.BoardCanvas
import com.extemporecreare.app.ui.theme.EtcColors
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** What the Reference board panel asks of the app: pickers and its board's file. */
class BoardActions(
    val add: () -> Unit,
    val newBoard: () -> Unit,
    val open: () -> Unit,
    val save: () -> Unit,
    val relink: (String) -> Unit,
    val select: (String?) -> Unit,
    val changed: () -> Unit,
)

/**
 * The Reference board (`uistandards.md` §8.2): a collapsible lower-right panel showing the open
 * `.etcr` [board] ([BoardCanvas]), its header counting the references with an add button. Below
 * the view: the board's file name (• when unsaved), New, Open and Save, and for the [selected]
 * image Front, Remove and, when its file is missing, Relink. Collapsing keeps the board, its
 * arrangement and camera. Pictures are decoded off the main thread as they are needed.
 */
@Composable
fun ReferenceBoardPanel(
    board: ReferenceBoard,
    revision: Int,
    selected: String?,
    expanded: Boolean,
    onToggle: () -> Unit,
    actions: BoardActions,
    modifier: Modifier = Modifier,
    headerModifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    LaunchedEffect(revision, expanded) {
        if (!expanded) return@LaunchedEffect
        for (image in board.images()) {
            val id = image.optString("id")
            if (id in board.pictures) continue
            val file = board.fileOf(id) ?: continue
            val picture = withContext(Dispatchers.IO) { ReferenceBoard.decode(context, file) }
            board.pictures[id] = picture
            actions.changed()
        }
    }
    FloatingPanel(
        "Reference board",
        "%02d REFS".format(board.images().size),
        expanded,
        onToggle,
        modifier.width(320.dp),
        headerModifier = headerModifier,
        actions = { Icon(Icons.Outlined.Add, "Add reference", tint = EtcColors.Text, modifier = Modifier.size(20.dp).clickable(onClick = actions.add)) },
    ) {
        Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            BoardCanvas(board, revision, selected, actions.select, actions.changed, Modifier.fillMaxWidth().height(240.dp).clip(RoundedCornerShape(10.dp)))
            val name = board.fileName?.takeIf { it.isNotEmpty() } ?: "Unsaved board"
            Text(name + if (board.modified) " •" else "", color = EtcColors.Muted, fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                BoardButton("New", actions.newBoard)
                BoardButton("Open", actions.open)
                BoardButton("Save", actions.save)
            }
            if (selected != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    BoardButton("Front") { board.toFront(selected); actions.changed() }
                    BoardButton("Remove") { board.remove(selected); actions.select(null); actions.changed() }
                    if (board.isMissing(selected)) BoardButton("Relink") { actions.relink(selected) }
                }
            }
        }
    }
}

/** A small text button in the board panel. */
@Composable
private fun BoardButton(label: String, onClick: () -> Unit) {
    Text(
        label,
        color = EtcColors.Text,
        fontSize = 12.sp,
        modifier = Modifier.clip(RoundedCornerShape(8.dp)).background(EtcColors.Panel2).clickable(onClick = onClick).padding(horizontal = 10.dp, vertical = 6.dp),
    )
}

// GLOSSARY
// - BoardActions(add, newBoard, open, save, relink, select, changed): what the board panel asks of the app.
// - ReferenceBoardPanel(board, revision, selected, expanded, onToggle, actions, modifier, headerModifier): the Reference board panel.
// - BoardButton(label, onClick): a small text button in the board panel.
