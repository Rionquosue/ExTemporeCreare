use super::*;
use vectorcraft_color::Color;

fn paint_op(f: &mut Filter, op: &str, vals: &[V]) {
    super::paint_op(f, op, vals, &mut Names::default());
}


























#[test]
fn a_style_list_keeps_its_strings_in_order() {
    let list = art_style_list(b"(A)\n(B\\)C)\n(D(E))\n/Name 3 x\n(Caf\\303\\251)\n(F");
    let want: [&[u8]; 4] = [b"A", b"B)C", b"D(E)", "Caf\u{e9}".as_bytes()];
    assert_eq!(list, want, "the string the data ends inside is left out");
    assert!(art_style_list(b"").is_empty());
    let unended: [&[u8]; 1] = [b"A"];
    assert_eq!(art_style_list(b"(A)\n(B\\)"), unended, "an escaped `)` doesn't end the string");
    assert_eq!(art_style_list(b"(A)\n(B(C)"), unended, "nor does one that closes a nested pair");
    assert_eq!(art_style_list(format!("({})", "x".repeat(MAX_LISTED_NAME + 1)).as_bytes()), Vec::<Vec<u8>>::new());
    assert_eq!(art_style_list("(x)\n".repeat(MAX_LISTED + 5).as_bytes()).len(), MAX_LISTED);
}

#[test]
fn hostile_dash_offsets_and_width_profiles_are_tamed() {
    let mut f = Filter::default();
    paint_op(&mut f, "d", &[V::Arr(vec![V::Num(5.0)]), V::Num(f64::NAN)]);
    assert_eq!(f.dash.as_ref().map(|d| d.offset), Some(0.0));
    // Positions outside 0..=1 and out of order, a negative width, a non-finite one.
    let arrays = HashMap::from([
        ("BSVariableWidthTValuesArray".to_string(), vec![1.5, -0.2, 0.5, 0.7]),
        ("BSCenteredWidthsArray".to_string(), vec![-1.0, 2.0, 0.5, f64::INFINITY]),
    ]);
    assert_eq!(width_profile(&arrays).map(|p| p.points), Some(vec![(0.0, 2.0, 2.0), (0.5, 0.5, 0.5), (1.0, 0.0, 0.0)]));
    let ok = (0..64).map(|_| V::Num(2.0)).collect::<Vec<_>>();
    paint_op(&mut f, "d", &[V::Arr(ok), V::Num(0.0)]);
    assert_eq!(f.dash.as_ref().map(|d| d.pattern.len()), Some(64));
    let huge = (0..65).map(|_| V::Num(1.0)).collect::<Vec<_>>();
    paint_op(&mut f, "d", &[V::Arr(huge), V::Num(0.0)]);
    assert_eq!(f.dash, None);
    let long = (0..1025).map(|i| i as f64 / 1024.0).collect::<Vec<_>>();
    let wide = vec![1.0; 1025];
    let huge = HashMap::from([("BSVariableWidthTValuesArray".to_string(), long), ("BSCenteredWidthsArray".to_string(), wide)]);
    assert_eq!(width_profile(&huge), None);
}

#[test]
fn hostile_stroke_widths_are_sanitized() {
    let mut f = Filter::default();
    paint_op(&mut f, "w", &[V::Num(12.0)]);
    assert_eq!(f.width, 12.0);
    paint_op(&mut f, "w", &[V::Num(-1.0)]);
    assert_eq!(f.width, 12.0);
    paint_op(&mut f, "w", &[V::Num(f64::INFINITY)]);
    assert_eq!(f.width, 12.0);
}

#[test]
fn hostile_stroke_paint_flags_ignore_short_operands() {
    let mut f = Filter { paints: (true, true), ..Filter::default() };
    paint_op(&mut f, "Xd", &[V::Num(1.0)]);
    assert_eq!(f.paints, (true, true));
    paint_op(&mut f, "Xd", &[V::Num(0.0), V::Num(1.0)]);
    assert_eq!(f.paints, (false, true));
    paint_op(&mut f, "Xd", &[V::Num(2.0), V::Num(0.0)]);
    assert_eq!(f.paints, (true, false));
}

#[test]
fn hostile_stroke_miter_limits_are_capped() {
    let mut f = Filter::default();
    paint_op(&mut f, "M", &[V::Num(900.0)]);
    assert_eq!(f.miter, 500.0);
    paint_op(&mut f, "M", &[V::Num(0.5)]);
    assert_eq!(f.miter, 500.0);
}

#[test]
fn hostile_stroke_caps_and_joins_ignore_invalid_values() {
    let mut f = Filter::default();
    paint_op(&mut f, "J", &[V::Num(99.0)]);
    assert_eq!(f.cap, LineCap::Butt);
    paint_op(&mut f, "J", &[V::Num(2.0)]);
    assert_eq!(f.cap, LineCap::Square);
    paint_op(&mut f, "j", &[V::Num(99.0)]);
    assert_eq!(f.join, LineJoin::Miter);
    paint_op(&mut f, "j", &[V::Num(1.0)]);
    assert_eq!(f.join, LineJoin::Round);
}

#[test]
fn hostile_blend_opacities_are_clamped() {
    let mut f = Filter::default();
    paint_op(&mut f, "Xy", &[V::Num(3.0), V::Num(1.5)]);
    assert_eq!(f.xy, Some((3, 1.0)));
    paint_op(&mut f, "Xy", &[V::Num(-1.0), V::Num(0.25)]);
    assert_eq!(f.xy, Some((0, 0.25)));
    paint_op(&mut f, "Xy", &[V::Num(0.0), V::Num(f64::NAN)]);
    assert_eq!(f.xy, Some((0, 0.25)));
}












































































/// Only a style that is one paint holds a pattern for type: the same pattern paint in a style that
/// also has filters gives none, whichever comes first.
#[test]
fn only_a_style_of_one_pattern_paint_holds_a_pattern_for_type() {
    let simple = "/SimpleStyle :\n0 O\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 1 0 0 0 Xy\n0 J 0 j 1 w 10 M []0 d\n0 XR\n0 1 Xd\n/Paint ;\n";
    let active = "/ActiveStyle :\n/CompoundFilter :\n(Chain Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
        /BasicFilter :\n(Fill Style Filter) 0 0 /Filter ,\n1 /Visible ,\n1 /FillOrStroke ,\n/Dictionary : /NotRecorded ,\n\
        /FillStyle : 0 O\n1 0 0 0 k\n/Def ;\n (FillStyle) ,\n; /Dict ;\n /Part ,\n;\n";
    let styles = |body: &str| art_styles_in(format!("/KnownStyle :\n(S) /Name ,\n{body} /Def ;\n").as_bytes(), &mut Names::default());
    assert!(styles(simple).get("S").is_some_and(|s| s.simple_pattern.is_some()));
    for body in [format!("{simple}{active}"), format!("{active}{simple}")] {
        assert!(styles(&body).get("S").is_none_or(|s| s.simple_pattern.is_none()), "{body}");
    }
}

/// A style that is one paint reads as the fill and stroke its `Xd` turns on, in the paint's colors
/// and the stroke's style; one that turns on a paint not read here, or has no `Xd`, isn't read.
#[test]
fn a_style_of_one_paint_reads_as_the_fill_and_stroke_its_xd_turns_on() {
    let one = |paint: &str| {
        let data = format!("/KnownStyle :\n(S) /Name ,\n/SimpleStyle :\n{paint}/Paint ;\n/Def ;\n");
        art_styles_in(data.as_bytes(), &mut Names::default()).remove("S").and_then(|s| s.one_paint)
    };
    let stroke = "0 R\n0 1 1 0 K\n1 J 2 j 3 w 4 M [2 1]0 d\n";
    let Some(Ok(items)) = one(&format!("0 O\n0 0 1 0 k\n{stroke}1 1 Xd\n")) else { panic!("{:?}", one(stroke)) };
    let [
        StyleItem::Fill(fill),
        StyleItem::Stroke { paint, width, cap, join, miter, dash, profile: None, align: StrokeAlign::Center, brush: None, arrows: _ },
    ] = items.as_slice()
    else {
        panic!("{items:?}")
    };
    assert_eq!((fill.color(), paint.color()), (Some(Color::cmyk(0.0, 0.0, 1.0, 0.0)), Some(Color::cmyk(0.0, 1.0, 1.0, 0.0))));
    assert_eq!((*width, *cap, *join, *miter), (3.0, LineCap::Round, LineJoin::Bevel, 4.0));
    assert_eq!(dash.as_ref().map(|d| d.pattern.clone()), Some(vec![2.0, 1.0]));
    // Only what `Xd` turns on; nothing when it turns on neither.
    assert!(matches!(one(&format!("0 0 1 0 k\n{stroke}1 0 Xd\n")), Some(Ok(i)) if matches!(i.as_slice(), [StyleItem::Stroke { .. }])));
    assert_eq!(one(&format!("0 0 1 0 k\n{stroke}0 0 Xd\n")), Some(Ok(vec![])));
    // The last paint of each one counts: a solid color after a pattern or a gradient replaces it.
    let solid_fill = |before: &str| one(&format!("{before}\n0 0 1 0 k\n0 1 Xd\n"));
    for before in ["(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p", "0 Bb\n2 (Sky) 0 0 0 100 1 0 0 1 0 0 Bg\n0 BB"] {
        assert!(matches!(solid_fill(before), Some(Ok(i)) if matches!(i.as_slice(), [StyleItem::Fill(_)])), "{before}");
    }
    let solid_stroke = one("(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] P\n0 1 1 0 K\n1 0 Xd\n");
    assert!(matches!(solid_stroke, Some(Ok(i)) if matches!(i.as_slice(), [StyleItem::Stroke { .. }])));
    // A gradient instance's flag says which one it paints: a stroke gradient leaves a solid fill read.
    let fill_and_stroke_gradient = one("0 0 1 0 k\n1 Bb\n2 (Ramp) 0 0 90 1 1 0 0 1 0 0 1 Bg\n0 BB\n0 1 Xd\n");
    assert!(matches!(fill_and_stroke_gradient, Some(Ok(i)) if matches!(i.as_slice(), [StyleItem::Fill(_)])));
    // A gradient instance's fill is its swatch at its angle; its stroke, after a solid one too, is a
    // gradient naming its swatch at its angle, at the stroke's weight.
    let gradient_fill = one("0 Bb\n2 (Sky) 0 0 30 100 1 0 0 1 0 0 Bg\n0 BB\n0 1 Xd\n");
    assert_eq!(gradient_fill, Some(Ok(vec![StyleItem::Gradient { name: Some("Sky".into()), angle: 30.0 }])));
    let gradient_stroke = one("0 R\n0 1 1 0 K\n1 Bb\n0 0 0 0 Bh\n2 (Ramp) 0 0 90 1 1 0 0 1 0 0 1 Bg\n0 BB\n2 w\n1 0 Xd\n");
    let Some(Ok(items)) = &gradient_stroke else { panic!("{gradient_stroke:?}") };
    let [StyleItem::Stroke { paint: Paint::Gradient(g), width, .. }] = items.as_slice() else { panic!("{items:?}") };
    assert_eq!((g.swatch.as_deref(), g.angle, g.geom, *width), (Some("Ramp"), 90.0, None, 2.0));
    // Not read: no `Xd`, a pattern fill, a pattern stroke, a gradient outside a gradient instance, a
    // paint turned on with no color.
    for paint in [
        format!("0 0 1 0 k\n{stroke}"),
        "(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 1 Xd\n".to_string(),
        "(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] P\n1 0 Xd\n".to_string(),
        "2 (Sky) 0 0 0 100 1 0 0 1 0 0 Bg\n0 1 Xd\n".to_string(),
        "0 Bb\n2 (Sky) 0 0 0 100 1 0 0 1 0 0 Bg\n0 BB\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 1 Xd\n".to_string(),
        "0 0 1 0 k\n1 1 Xd\n".to_string(),
    ] {
        assert_eq!(one(&paint), Some(Err(())), "{paint}");
    }
}


