package com.extemporecreare.app.ui.panels

import android.graphics.Bitmap
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
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
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.boards.ReferenceBoard
import com.extemporecreare.app.ui.board.BoardCanvas
import com.extemporecreare.app.ui.theme.EtcColors
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/** What the Reference board panel asks of the app: pickers, its board's file, Embed and pictures. */
class BoardActions(
    val add: () -> Unit,
    val newBoard: () -> Unit,
    val open: () -> Unit,
    val save: () -> Unit,
    val relink: (String) -> Unit,
    val embed: (String) -> Unit,
    val select: (String?) -> Unit,
    val changed: () -> Unit,
    /** Image id → its picture (scaled down), read off the main thread; null when it can't be read. */
    val picture: (String) -> Bitmap?,
)

/**
 * The Reference board (`uistandards.md` §8.2): a collapsible lower-right panel showing the open
 * `.etcr` [board] ([BoardCanvas]), its header counting the references with an add button. Below
 * the view: the board's file name (• when unsaved), New, Open and Save, and for the [selected]
 * image Front, Remove and, when its file is missing, Relink. Holding an image opens its menu:
 * Delete, Embed (store the image inside the `.etcr`), Flip (Horizontal, Vertical) and Crop, which
 * shows the whole picture to crop with its corners until Done. Collapsing keeps the board, its
 * arrangement and camera. Pictures are read off the main thread as they are needed.
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
    var menuFor by remember { mutableStateOf<String?>(null) }
    var menuAt by remember { mutableStateOf(Offset.Zero) }
    var flipMenu by remember { mutableStateOf(false) }
    var cropping by remember { mutableStateOf<String?>(null) }
    if (cropping != null && board.images().none { it.optString("id") == cropping }) cropping = null
    LaunchedEffect(revision, expanded) {
        if (!expanded) return@LaunchedEffect
        for (image in board.images()) {
            val id = image.optString("id")
            if (id in board.pictures || board.fileOf(id) == null) continue
            val picture = withContext(Dispatchers.IO) { actions.picture(id) }
            board.pictures[id] = picture
            actions.changed()
        }
    }
    FloatingPanel(
        "Reference",
        Icons.Outlined.Image,
        board.images().size,
        expanded,
        onToggle,
        320.dp,
        modifier,
        headerModifier = headerModifier,
        actions = { Icon(Icons.Outlined.Add, "Add reference", tint = EtcColors.Text, modifier = Modifier.size(20.dp).clickable(onClick = actions.add)) },
    ) {
        Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            val density = LocalDensity.current
            Box {
                BoardCanvas(
                    board, revision, selected, cropping,
                    onSelect = actions.select,
                    onMenu = { id, at -> actions.select(id); menuFor = id; menuAt = at; flipMenu = false },
                    onChanged = actions.changed,
                    modifier = Modifier.fillMaxWidth().height(BOARD_HEIGHT).clip(RoundedCornerShape(10.dp)),
                )
                val close = { menuFor = null; flipMenu = false }
                DropdownMenu(
                    expanded = menuFor != null,
                    onDismissRequest = close,
                    offset = with(density) { DpOffset(menuAt.x.toDp(), menuAt.y.toDp() - BOARD_HEIGHT) },
                ) {
                    val id = menuFor
                    if (id != null && !flipMenu) {
                        DropdownMenuItem(text = { Text("Delete") }, onClick = { board.remove(id); actions.select(null); actions.changed(); close() })
                        DropdownMenuItem(text = { Text("Embed") }, onClick = { close(); actions.embed(id) }, enabled = !board.isEmbedded(id))
                        DropdownMenuItem(text = { Text("Flip ›") }, onClick = { flipMenu = true })
                        DropdownMenuItem(text = { Text("Crop") }, onClick = { cropping = id; close() })
                    } else if (id != null) {
                        DropdownMenuItem(text = { Text("Horizontal") }, onClick = { board.flip(id, true); board.checkpoint(); actions.changed(); close() })
                        DropdownMenuItem(text = { Text("Vertical") }, onClick = { board.flip(id, false); board.checkpoint(); actions.changed(); close() })
                    }
                }
            }
            val name = board.fileName?.takeIf { it.isNotEmpty() } ?: "Unsaved board"
            Text(name + if (board.modified) " •" else "", color = EtcColors.Muted, fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (cropping != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text("Drag the corners to crop", color = EtcColors.Soft, fontSize = 11.sp, modifier = Modifier.padding(top = 6.dp))
                    BoardButton("Done") { cropping = null; board.checkpoint(); actions.changed() }
                }
            } else {
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
}

/** The board view's height. */
private val BOARD_HEIGHT = 240.dp

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
// - BoardActions(add, newBoard, open, save, relink, embed, select, changed, picture): what the board panel asks of the app.
// - ReferenceBoardPanel(board, revision, selected, expanded, onToggle, actions, modifier, headerModifier): the Reference board panel.
// - menuFor, menuAt, flipMenu, cropping: the image menu's image and place, its Flip submenu, the image being cropped.
// - BOARD_HEIGHT: the board view's height. BoardButton(label, onClick): a small text button in the board panel.
