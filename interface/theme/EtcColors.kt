package com.extemporecreare.app.ui.theme

import androidx.compose.ui.graphics.Color

/** The colour tokens of `uistandards.md` §3.1. */
object EtcColors {
    /** Deep workspace background (`--bg`). */
    val Bg = Color(0xFF0C131B)

    /** Application shell (`--shell`). */
    val Shell = Color(0xFF141E29)

    /**
     * Floating panel (`--panel`, #17222E). The mockup's translucency (EB) needs a backdrop blur,
     * which Compose doesn't offer, so the panels use the opaque fallback `uistandards.md` §3.2 asks
     * for, keeping labels readable over artwork.
     */
    val Panel = Color(0xFF17222E)

    /** Secondary panel surface (`--panel2`). */
    val Panel2 = Color(0xFF1D2A37)

    /** Main border (`--line`). */
    val Line = Color(0xFF34404F)

    /** Subtle divider (`--line-soft`). */
    val LineSoft = Color(0xFF2A3644)

    /** Primary text (`--text`). */
    val Text = Color(0xFFF1EEE9)

    /** Secondary text (`--muted`). */
    val Muted = Color(0xFFA4AEB7)

    /** Subdued text (`--soft`). */
    val Soft = Color(0xFF778492)

    /** Active / primary accent (`--gold`). */
    val Gold = Color(0xFFEFC98C)

    /** Secondary gold (`--gold2`). */
    val Gold2 = Color(0xFFDBA767)

    /** Positive status accent (`--teal`). */
    val Teal = Color(0xFF83C5BB)
}

// GLOSSARY
// - EtcColors: the §3.1 colour tokens: Bg, Shell, Panel, Panel2, Line, LineSoft, Text, Muted, Soft, Gold, Gold2, Teal.
