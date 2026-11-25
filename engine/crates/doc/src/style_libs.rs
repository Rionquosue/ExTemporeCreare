//! Graphic style libraries: named, read-only sets of graphic styles (Window → Graphic Style
//! Libraries). The built-in ones are data files the engine reads at run time; user libraries
//! are `.vcstyles` files ([`write`], [`read`]): JSON holding the styles and the pattern
//! definitions they paint with.
//!
//! Library styles stand alone: no id, and no links to the swatches of the document they came from
//! ([`StyleLibrary::from_document`]).

use serde::{Deserialize, Serialize};
use vectorcraft_color::Paint;

use crate::{Document, GraphicStyle, PatternDef};

/// A graphic style library: its name, its styles and the patterns they paint with.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleLibrary {
    pub name: String,
    #[serde(default)]
    pub styles: Vec<GraphicStyle>,
    /// The pattern swatches the styles' fills and strokes paint with.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternDef>,
}

impl StyleLibrary {
    pub fn len(&self) -> usize {
        self.styles.len()
    }
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
    pub fn style(&self, name: &str) -> Option<&GraphicStyle> {
        self.styles.iter().find(|g| g.name == name)
    }

    /// The document's styles `names` (empty: all of them, in panel order) as a library named
    /// `name`, standing alone ([`GraphicStyle::standalone`]) with the patterns they paint with.
    pub fn from_document(d: &Document, names: &[String], name: String) -> Result<Self, String> {
        if let Some(n) = names.iter().find(|n| d.graphic_style(n).is_none()) {
            return Err(format!("no graphic style `{n}`"));
        }
        let styles: Vec<GraphicStyle> =
            d.graphic_styles.iter().filter(|g| names.is_empty() || names.contains(&g.name)).map(GraphicStyle::standalone).collect();
        let mut patterns: Vec<PatternDef> = vec![];
        for p in styles.iter().flat_map(GraphicStyle::patterns) {
            if !patterns.iter().any(|x| x.name == p)
                && let Some(def) = d.pattern(p)
            {
                patterns.push(def.clone());
            }
        }
        Ok(Self { name, styles, patterns })
    }
}

impl GraphicStyle {
    /// Do both styles look the same (appearance and transparency; names and ids aside)?
    pub fn same_look(&self, other: &GraphicStyle) -> bool {
        (&self.appearance, self.opacity, self.blend, self.isolate, self.knockout, self.unit_box)
            == (&other.appearance, other.opacity, other.blend, other.isolate, other.knockout, other.unit_box)
    }

    /// The style as a library holds it: no id, and its paints unlinked from swatches.
    pub fn standalone(&self) -> GraphicStyle {
        let mut g = GraphicStyle { id: 0, ..self.clone() };
        for it in &mut g.appearance.items {
            match it.paint_mut() {
                // The colour shown stays, unlinked.
                Paint::Solid { swatch, tint, .. } => (*swatch, *tint) = (None, 1.0),
                Paint::Gradient(gp) => {
                    gp.swatch = None;
                    gp.gradient.stops.iter_mut().for_each(|st| st.set_color(st.color, None));
                }
                Paint::None | Paint::Pattern { .. } => {}
            }
        }
        g
    }

    /// The names of the patterns the style's fills and strokes paint with.
    pub fn patterns(&self) -> impl Iterator<Item = &str> {
        self.appearance.items.iter().filter_map(|it| match it.paint() {
            Paint::Pattern { pattern, .. } => Some(pattern.as_str()),
            _ => None,
        })
    }
}

// ---------- .vcstyles files ----------

/// The extension (and `format` header) of graphic style library files.
pub const STYLES_EXT: &str = "vcstyles";

/// The version of the `.vcstyles` files [`write`] writes. Version 2: a positive Twist angle turns
/// clockwise ([`crate::twist`]).
const STYLES_VERSION: u32 = crate::twist::TWIST_CLOCKWISE_SINCE;

/// The file: a header around the library.
#[derive(Serialize, Deserialize)]
struct NativeFile {
    format: String,
    version: u32,
    #[serde(flatten)]
    library: StyleLibrary,
}

/// `lib` as a `.vcstyles` file.
pub fn write(lib: &StyleLibrary) -> String {
    let file = NativeFile { format: STYLES_EXT.into(), version: STYLES_VERSION, library: lib.clone() };
    serde_json::to_string_pretty(&file).unwrap_or_default()
}

/// Read a `.vcstyles` file; an unnamed library is called `name`. A version 1 file was written by
/// an app that turned Twist counterclockwise: its Twist angles are negated.
pub fn read(text: &str, name: &str) -> Result<StyleLibrary, String> {
    let f: NativeFile = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|e| format!("not a graphic style library: {e}"))?;
    if f.format != STYLES_EXT {
        return Err(format!("not a graphic style library (format `{}`)", f.format));
    }
    let mut lib = f.library;
    if f.version < STYLES_VERSION {
        crate::twist::negate_library(&mut lib);
    }
    if lib.name.trim().is_empty() {
        lib.name = name.into();
    }
    Ok(lib)
}

/// Does `text` look like a `.vcstyles` file (its header names the format)?
pub fn sniff(text: &str) -> bool {
    let t = text.trim_start_matches('\u{feff}').trim_start();
    let head = t.char_indices().nth(256).map_or(t, |(i, _)| &t[..i]);
    t.starts_with('{') && head.contains(&format!("\"{STYLES_EXT}\""))
}
