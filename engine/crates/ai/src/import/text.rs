//! Fonts and type: fonts are found by name (their programs aren't run) and type becomes point
//! type in the app's font of that name, sized and turned as the font matrix and the current
//! transform say.

use std::sync::Arc;

/// A word of a style as PostScript names abbreviate it (`AkzidenzGroteskPro-BoldCnIt`), in full.
fn style_word(w: &str) -> String {
    match w {
        "Lt" => "Light",
        "Md" | "Med" => "Medium",
        "Bd" => "Bold",
        "XBd" => "Extra Bold",
        "Blk" => "Black",
        "Hv" => "Heavy",
        "Th" => "Thin",
        "Rg" => "Regular",
        "Sb" | "Smbd" => "Semibold",
        "Cn" | "Cond" => "Condensed",
        "Ext" | "Ex" => "Extended",
        "It" => "Italic",
        "Obl" => "Oblique",
        w => w,
    }
    .to_string()
}

/// A font's PostScript name without a subset prefix (`ABCDEF+`) or the `*1` of a re-encoded copy
/// (`ABCDEF+RollerBabyBV*1` is `RollerBabyBV`).
fn bare_name(name: &str) -> &str {
    let name = match name.split_once('+') {
        Some((prefix, rest)) if prefix.len() == 6 && prefix.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => name,
    };
    match name.rsplit_once('*') {
        Some((base, n)) if !base.is_empty() && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) => base,
        _ => name,
    }
}

/// The font of type that a text document's font record names: by `name`, its PostScript name, and
/// `version`, the version string (name ID 5) the record keeps, if any. An installed face of that
/// name gives its font ([`named_font`]). Without one, `name` is split ([`family_style`]) and the
/// family spelled as an available family is ([`FontDb::find_family`]: `MicrosoftYaHei` is
/// Microsoft YaHei). → (family, style, version).
///
/// [`FontDb::find_family`]: vectorcraft_text::FontDb::find_family
pub(crate) fn document_font(name: &str, version: Option<&str>) -> (String, String, Option<String>) {
    named_font(name, version).unwrap_or_else(|| {
        let (family, style) = family_style(name);
        let family = vectorcraft_text::FontDb::global().find_family(&family).unwrap_or(family);
        (family, style, None)
    })
}

/// The font of type set in the installed face of PostScript name `name` (a subset prefix and a
/// re-encoded copy's `*1` aside) in `version` ([`postscript_face`]): that face's own family and
/// style, and its version when the family and style alone resolve to another installed copy of
/// the font. A library can keep two copies under one family and style that differ in their
/// PostScript names or their versions, and draw the same text at different widths. → (family,
/// style, version); `None` when no installed face has the name.
pub(crate) fn named_font(name: &str, version: Option<&str>) -> Option<(String, String, Option<String>)> {
    let face = postscript_face(bare_name(name), version)?;
    // As the PDF import names a version: only one that the family and style don't resolve to.
    let default = vectorcraft_text::FontDb::global().face(&face.family, &face.style);
    let version = default.filter(|d| d.version != face.version).map(|_| face.version.clone());
    Some((face.family.clone(), face.style.clone(), version))
}

/// The installed face of PostScript name `name` (without a subset prefix), among the installed
/// faces that answer to it ([`FontDb::postscript_faces`]): those of exactly that name when there
/// are any (PostScript names are case-sensitive, so `SampleSans` isn't the face named
/// `SAMPLESANS`, and `SampleSerif-Bold` isn't `SampleSerifBold`), and of those the one in
/// `version` when it is installed, else the first. `None` when no installed face has the name.
///
/// [`FontDb::postscript_faces`]: vectorcraft_text::FontDb::postscript_faces
pub(crate) fn postscript_face(name: &str, version: Option<&str>) -> Option<Arc<vectorcraft_text::FontFace>> {
    let db = vectorcraft_text::FontDb::global();
    let found = db.find_postscript(name)?;
    if found.postscript_name() == name && version.is_none_or(|v| v == found.version) {
        return Some(found);
    }
    let faces = db.postscript_faces(name);
    let named: Vec<_> = faces.iter().filter(|f| f.postscript_name() == name).cloned().collect();
    let pool = if named.is_empty() { faces } else { named };
    let in_version = version.and_then(|v| pool.iter().find(|f| f.version == v).cloned());
    Some(in_version.or_else(|| pool.into_iter().next()).unwrap_or(found))
}

/// A font's family and style from its PostScript name, without a subset prefix (`ABCDEF+`) or the
/// `*1` of a re-encoded copy (`ABCDEF+RollerBabyBV*1`): an installed or bundled face of that
/// exact name gives its own (a family can hold hyphens: `rounded-x-mplus-1c-black` is Rounded-X
/// M+ 1c, black); else the name is split, `Helvetica-BoldOblique` → (`Helvetica`, `Bold
/// Oblique`), `TimesNewRomanPS-BoldMT` → (`Times New Roman`, `Bold`).
pub fn family_style(name: &str) -> (String, String) {
    let name = bare_name(name);
    if let Some(face) = vectorcraft_text::FontDb::global().find_postscript(name) {
        return (face.family.clone(), face.style.clone());
    }
    let (family, style) = name.split_once('-').unwrap_or((name, ""));
    let trim = |s: &str| -> String {
        let s = ["PSMT", "MT", "PS"].iter().find_map(|suffix| s.strip_suffix(suffix)).unwrap_or(s);
        // Words: a capital after a small letter starts one.
        let mut out = String::with_capacity(s.len() + 4);
        let mut prev_lower = false;
        for ch in s.chars() {
            if ch.is_ascii_uppercase() && prev_lower {
                out.push(' ');
            }
            prev_lower = ch.is_ascii_lowercase();
            out.push(ch);
        }
        out
    };
    let style = match trim(style).as_str() {
        "" | "Roman" | "Regular" | "Book" | "Normal" => "Regular".to_string(),
        s => s.split(' ').map(style_word).collect::<Vec<_>>().join(" "),
    };
    (trim(family), style)
}

