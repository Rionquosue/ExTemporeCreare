package com.extemporecreare.app.ui.splash

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.documents.RecentFiles
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingButton
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.text.DateFormat
import java.util.Date

/** What the splash asks for. */
class SplashActions(
    val newDocument: () -> Unit,
    val open: () -> Unit,
    val openRecent: (RecentFiles.Recent) -> Unit,
    val locate: (RecentFiles.Recent) -> Unit,
    val remove: (RecentFiles.Recent) -> Unit,
    val settings: () -> Unit,
)

/**
 * The opening splash (`uistandards.md` §5.2): the identity with New Document and Open; up to ten
 * recent files newest first, in two rows of five on a wide screen (fewer a row on a narrow one,
 * order kept), each with its thumbnail, name and date, empty places shown faintly; a missing file
 * offers Locate and Remove; and Settings below.
 */
@Composable
fun SplashScreen(recents: RecentFiles, revision: Int, actions: SplashActions) {
    val items = remember(revision) { recents.list() }
    Column(
        Modifier.fillMaxSize().background(EtcColors.Bg).verticalScroll(rememberScrollState()).padding(32.dp),
        verticalArrangement = Arrangement.spacedBy(22.dp),
    ) {
        Row(verticalAlignment = Alignment.Bottom) {
            Text("Ex-Tempore ", color = EtcColors.Text, fontFamily = FontFamily.Serif, fontStyle = FontStyle.Italic, fontSize = 34.sp)
            Text("Creare", color = EtcColors.Gold, fontFamily = FontFamily.Serif, fontStyle = FontStyle.Italic, fontSize = 34.sp)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            SettingButton("New Document", actions.newDocument, primary = true)
            SettingButton("Open…", actions.open)
        }
        Text("Recent", color = EtcColors.Muted, fontSize = 13.sp)
        BoxWithConstraints(Modifier.fillMaxWidth()) {
            val perRow = if (maxWidth >= 900.dp) 5 else if (maxWidth >= 560.dp) 3 else 2
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                for (rowStart in 0 until RecentFiles.MAX step perRow) {
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        for (i in rowStart until minOf(rowStart + perRow, RecentFiles.MAX)) {
                            Box(Modifier.weight(1f)) { RecentTile(recents, items.getOrNull(i), actions) }
                        }
                    }
                }
            }
        }
        SettingButton("Settings", actions.settings)
    }
}

/** One recent file, or an empty place. */
@Composable
private fun RecentTile(recents: RecentFiles, item: RecentFiles.Recent?, actions: SplashActions) {
    val shape = RoundedCornerShape(12.dp)
    if (item == null) {
        Box(Modifier.fillMaxWidth().aspectRatio(1.1f).border(1.dp, EtcColors.LineSoft, shape))
        return
    }
    var thumb by remember(item.uri) { mutableStateOf<Bitmap?>(null) }
    var missing by remember(item.uri) { mutableStateOf(false) }
    LaunchedEffect(item.uri) {
        withContext(Dispatchers.IO) {
            missing = !recents.exists(item.uri)
            if (!missing) thumb = recents.thumbnail(item.uri)
        }
    }
    Column(
        Modifier.fillMaxWidth().background(EtcColors.Panel2, shape).border(1.dp, EtcColors.Line, shape)
            .clickable(enabled = !missing) { actions.openRecent(item) }.padding(8.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        Box(Modifier.fillMaxWidth().aspectRatio(1.4f).background(EtcColors.Shell, RoundedCornerShape(8.dp)), contentAlignment = Alignment.Center) {
            thumb?.let { Image(it.asImageBitmap(), null, contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize()) }
                ?: Text(if (missing) "Missing" else "", color = EtcColors.Soft, fontSize = 11.sp)
        }
        Text(item.name, color = EtcColors.Text, fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(item.usedAt)), color = EtcColors.Muted, fontSize = 10.sp)
        if (missing) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                SettingButton("Locate", { actions.locate(item) })
                SettingButton("Remove", { actions.remove(item) })
            }
        }
    }
}

// GLOSSARY
// - SplashActions: what the splash asks for (new, open, open a recent, locate, remove, settings).
// - SplashScreen(recents, revision, actions): the opening splash.
// - RecentTile(recents, item, actions): one recent file or an empty place.
