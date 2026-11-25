package com.extemporecreare.app.ui.tools

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.ui.theme.EtcColors
import java.util.Locale
import kotlin.math.pow

/** The compact contextual settings window a dial tool opens (`uistandards.md` §6.2): a rounded floating panel titled [title]. */
@Composable
fun ToolSettingsWindow(title: String, modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier = modifier
            .widthIn(min = 240.dp, max = 340.dp)
            .background(EtcColors.Panel, RoundedCornerShape(16.dp))
            .border(BorderStroke(1.dp, EtcColors.Line), RoundedCornerShape(16.dp))
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Text(title, color = EtcColors.Text, fontSize = 14.sp)
        content()
    }
}

/** A short secondary text line, for state the user should know (what Crop crops, "Nothing to export"). */
@Composable
fun SettingNote(text: String, color: Color = EtcColors.Muted) {
    Text(text, color = color, fontSize = 12.sp)
}

/** Numbers in tabular figures (§3.2). */
private val Tabular = TextStyle(fontFeatureSettings = "tnum", fontSize = 12.sp, color = EtcColors.Text)

/** [value] with at most [decimals] decimals, trailing zeros dropped. */
fun formatNumber(value: Double, decimals: Int = 2): String =
    String.format(Locale.ROOT, "%.${decimals}f", value).trimEnd('0').trimEnd('.')

/**
 * A labelled slider for [value] between [min] and [max], its value and [unit] beside the label.
 * With [curve] above 1 the track gives small values more room (sizes); [whole] keeps whole numbers.
 */
@Composable
fun SettingSlider(
    label: String,
    value: Double,
    min: Double,
    max: Double,
    unit: String,
    onChange: (Double) -> Unit,
    curve: Double = 1.0,
    whole: Boolean = false,
) {
    fun toTrack(v: Double) = ((v.coerceIn(min, max) - min) / (max - min)).pow(1 / curve).toFloat()
    fun fromTrack(t: Float) = (min + (max - min) * t.toDouble().pow(curve)).let { if (whole) Math.round(it).toDouble() else it }
    Column {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, color = EtcColors.Muted, fontSize = 12.sp)
            Text("${formatNumber(value, if (whole) 0 else 1)} $unit", style = Tabular)
        }
        Slider(
            value = toTrack(value),
            onValueChange = { onChange(fromTrack(it)) },
            colors = SliderDefaults.colors(thumbColor = EtcColors.Gold, activeTrackColor = EtcColors.Gold2, inactiveTrackColor = EtcColors.Line),
        )
    }
}

/** A labelled row of [options] (their labels), [selected] highlighted in gold, choosing one with [onSelect]. */
@Composable
fun <T> SettingChoice(label: String, options: List<Pair<T, String>>, selected: T, onSelect: (T) -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(label, color = EtcColors.Muted, fontSize = 12.sp)
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            for ((option, text) in options) {
                val on = option == selected
                Box(
                    Modifier
                        .background(if (on) EtcColors.Gold else EtcColors.Panel2, RoundedCornerShape(8.dp))
                        .border(1.dp, if (on) EtcColors.Gold else EtcColors.Line, RoundedCornerShape(8.dp))
                        .clickable { onSelect(option) }
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                ) {
                    Text(text, color = if (on) EtcColors.Bg else EtcColors.Text, fontSize = 12.sp)
                }
            }
        }
    }
}

/** A labelled switch. */
@Composable
fun SettingSwitch(label: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
        Text(label, color = EtcColors.Text, fontSize = 12.sp)
        Switch(
            checked = checked,
            onCheckedChange = onChange,
            colors = SwitchDefaults.colors(checkedThumbColor = EtcColors.Bg, checkedTrackColor = EtcColors.Gold, uncheckedTrackColor = EtcColors.Panel2),
        )
    }
}

/**
 * A number field labelled [label] showing [value] in [unit]; the typed number is committed with
 * [onCommit] on Done or when the field loses focus, and only when it is a number.
 */
@Composable
fun SettingNumberField(label: String, value: Double, unit: String, onCommit: (Double) -> Unit, modifier: Modifier = Modifier.width(120.dp)) {
    var text by remember(value) { mutableStateOf(formatNumber(value)) }
    fun commit() {
        text.toDoubleOrNull()?.takeIf { it.isFinite() && it != value }?.let(onCommit)
    }
    OutlinedTextField(
        value = text,
        onValueChange = { text = it },
        label = { Text(label, fontSize = 11.sp) },
        suffix = { Text(unit, fontSize = 11.sp, color = EtcColors.Muted) },
        singleLine = true,
        textStyle = Tabular,
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal, imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { commit() }),
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = EtcColors.Gold,
            unfocusedBorderColor = EtcColors.Line,
            focusedLabelColor = EtcColors.Gold,
            unfocusedLabelColor = EtcColors.Muted,
            cursorColor = EtcColors.Gold,
        ),
        modifier = modifier.onFocusChanged { if (!it.isFocused) commit() },
    )
}

/** A compact text button, gold when [primary]. */
@Composable
fun SettingButton(text: String, onClick: () -> Unit, primary: Boolean = false, enabled: Boolean = true) {
    Box(
        Modifier
            .background(if (primary && enabled) EtcColors.Gold else EtcColors.Panel2, RoundedCornerShape(8.dp))
            .border(1.dp, if (primary && enabled) EtcColors.Gold else EtcColors.Line, RoundedCornerShape(8.dp))
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 7.dp),
    ) {
        val color = when {
            !enabled -> EtcColors.Soft
            primary -> EtcColors.Bg
            else -> EtcColors.Text
        }
        Text(text, color = color, fontSize = 12.sp)
    }
}

// GLOSSARY
// - ToolSettingsWindow(title, modifier, content): the tool's compact settings panel.
// - SettingNote(text, color): a short secondary line.
// - Tabular: tabular-figure number style.
// - formatNumber(value, decimals): a number without trailing zeros.
// - SettingSlider(label, value, min, max, unit, onChange, curve, whole): a labelled slider with its value.
// - SettingChoice(label, options, selected, onSelect): a row of choices.
// - SettingSwitch(label, checked, onChange): a labelled switch.
// - SettingNumberField(label, value, unit, onCommit, modifier): a number field committed on Done or focus loss.
// - SettingButton(text, onClick, primary, enabled): a compact button.
