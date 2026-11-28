package com.extemporecreare.app.ui.panels

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Texture
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.textures.TextureLibrary
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingNote
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * The Texture library (`uistandards.md` §7.4): a collapsible floating panel of named preview
 * tiles, the textures in `tools/Libraries/textures`. Tapping one inserts it as a texture layer
 * ([onPick]), editable and not baked into the artwork.
 */
@Composable
fun TextureLibraryPanel(library: TextureLibrary, expanded: Boolean, onToggle: () -> Unit, onPick: (TextureLibrary.Texture) -> Unit, modifier: Modifier = Modifier, headerModifier: Modifier = Modifier) {
    val textures = remember(library) { library.list() }
    val previews = remember(library) { mutableStateMapOf<String, Bitmap?>() }
    LaunchedEffect(expanded) {
        if (!expanded) return@LaunchedEffect
        for (t in textures) {
            if (t.asset in previews) continue
            previews[t.asset] = withContext(Dispatchers.IO) { library.preview(t) }
        }
    }
    FloatingPanel("Texture library", Icons.Outlined.Texture, textures.size, expanded, onToggle, 260.dp, modifier, headerModifier) {
        if (textures.isEmpty()) {
            Box(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp)) {
                SettingNote("No textures yet: add images to tools/Libraries/textures.")
            }
            return@FloatingPanel
        }
        Column(
            Modifier.heightIn(max = 420.dp).verticalScroll(rememberScrollState()).padding(start = 12.dp, end = 12.dp, bottom = 12.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            for (row in textures.chunked(COLUMNS)) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    for (t in row) {
                        Column(Modifier.weight(1f).clickable { onPick(t) }) {
                            Box(
                                Modifier.fillMaxWidth().aspectRatio(1f).clip(RoundedCornerShape(8.dp)).background(EtcColors.Panel2)
                                    .border(1.dp, EtcColors.LineSoft, RoundedCornerShape(8.dp)),
                            ) {
                                previews[t.asset]?.let { Image(it.asImageBitmap(), t.name, Modifier.fillMaxWidth().aspectRatio(1f), contentScale = ContentScale.Crop) }
                            }
                            Text(t.name, color = EtcColors.Text, fontSize = 11.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 4.dp))
                        }
                    }
                    repeat(COLUMNS - row.size) { Box(Modifier.weight(1f)) }
                }
            }
        }
    }
}

/** Preview tiles a row. */
private const val COLUMNS = 3

// GLOSSARY
// - TextureLibraryPanel(library, expanded, onToggle, onPick, modifier, headerModifier): the Texture library panel.
// - COLUMNS: preview tiles a row.
