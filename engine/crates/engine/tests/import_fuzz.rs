//! Untrusted files never crash the app: garbage, truncated, mutated and hostile SVG, PDF, DXF and HP-GL input,
//! mutated raster images (Photoshop documents and JPEG XL images among them) placed with File → Place, Photoshop documents' slices (damaged or cut short) opened and placed, and swatch (`.vcswatches`, `.gpl`, `.ase`), graphic style (`.vcstyles`), flattener preset
//! (`.vcflattener`) and export preset (`.vcexportpresets`) libraries, shortcut sets and action sets other apps save as text (`.kys`, `.aia`), and native files (compressed, damaged, saved for older versions),
//! must load as an error or as a document that then renders and exports, without a panic; nor may
//! bitmaps, PDF and text pasted from other apps, nor EMF and WMF pictures (damaged files, records
//! of every kind with random contents) opened, placed or pasted, nor PICT pictures (damaged files,
//! opcodes with random contents) opened or placed, nor EPS and PostScript files
//! (damaged ones, hostile programs) read by the PostScript interpreter, nor the editing data of
//! Illustrator EPS and `.ai` files (the layers they carry, damaged or hostile), nor Affinity documents
//! (mutated object streams, archives and indexed PNG previews, hostile image dimensions), nor
//! `.cdr` drawings (mutated and cut short, plain, compressed and packaged, opened and placed),
//! nor `.cmx` files (mutated, cut short and garbage after the header, opened and placed),
//! nor the font files a folder search reads (damaged fonts, collections whose headers give any count of
//! faces), nor scripts written for the reference app checked without running them (garbage,
//! generated script text and mutated scripts).
//!
//! `PROPTEST_CASES=20000 cargo test -p vectorcraft-engine --test import_fuzz` runs a deeper search.
// Integration tests: unwrapping and panicking on failure is fine here, unlike in shipped code (AGENTS.md › Robustness).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use serde_json::{Value, json};
use vectorcraft_doc::Document;
use vectorcraft_testkit::catch_quiet;
use vectorcraft_testkit::fixtures::rich_session;

/// 64 cases each, unless `PROPTEST_CASES` asks for more.
fn config() -> ProptestConfig {
    let mut c = ProptestConfig { failure_persistence: None, ..ProptestConfig::default() };
    if std::env::var_os("PROPTEST_CASES").is_none() {
        c.cases = 64;
    }
    c
}

// ---------- Affinity previews ----------

/// Original pixels in a synthetic container envelope, not an Affinity document writer.
fn affinity_preview_sample() -> Vec<u8> {
    let im = image::RgbaImage::from_pixel(2, 1, image::Rgba([220, 40, 60, 255]));
    let mut png = Vec::new();
    {
        let __dyn: image::DynamicImage = im.into();
        let (__w, __h) = (__dyn.width(), __dyn.height());
        let (__layout, __sample, __raw) = match __dyn {
            image::DynamicImage::ImageLuma8(b) => (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageLumaA8(b) => (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageRgb8(b) => (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageRgba8(b) => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageLuma16(b) => (
                craft_image_codecs::ChannelLayout::Gray,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageLumaA16(b) => (
                craft_image_codecs::ChannelLayout::GrayA,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageRgb16(b) => (
                craft_image_codecs::ChannelLayout::Rgb,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageRgba16(b) => (
                craft_image_codecs::ChannelLayout::Rgba,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            other => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, other.into_rgba8().into_raw()),
        };
        let __cimg = craft_image_codecs::Image::from_raw(__w, __h, __layout, __sample, __raw).unwrap();
        craft_image_codecs::encode(&__cimg, craft_image_codecs::Format::Png, &craft_image_codecs::EncodeOptions::default())
            .map(|__b| png.extend_from_slice(&__b))
    }
    .unwrap();
    let mut b = vec![0; 72];
    b[..4].copy_from_slice(vectorcraft_affinity::MAGIC);
    b[4..6].copy_from_slice(&12u16.to_le_bytes());
    b[8..12].copy_from_slice(b"nsrP");
    b[12..16].copy_from_slice(b"#Inf");
    b[24..32].copy_from_slice(&72u64.to_le_bytes());
    b[64..68].copy_from_slice(b"Prot");
    b.extend(b"\xff\xff\xff\xffThmb");
    b.extend(1u32.to_le_bytes());
    b.extend((png.len() as u32 + 13).to_le_bytes());
    b.extend(29u32.to_le_bytes());
    b.extend(0u32.to_le_bytes());
    b.extend((png.len() as u32).to_le_bytes());
    b.push(1);
    b.extend(png);
    b
}

/// A synthetic native document exercising the reader's paths: curves (with live corners), every
/// parametric shape it knows, a compound, gradients, a stroke, text, a mask and an artboard. Not
/// an Affinity writer: the layout is the one the reader accepts.
fn affinity_native_stream() -> Vec<u8> {
    use vectorcraft_affinity::synth::{self, F, tag};
    let f32s = |v: &[f32]| F::Struct(v.iter().flat_map(|x| x.to_le_bytes()).collect());
    let rec = |x: f64, y: f64, role: u8| {
        let mut r = x.to_le_bytes().to_vec();
        r.extend(y.to_le_bytes());
        r.extend([1, role]);
        r
    };
    let mut id = 100;
    let mut next = || {
        id += 1;
        id
    };
    let colour = |id: u32| F::Def(id, vec![tag(b"RGBA")], vec![(tag(b"_col"), f32s(&[0.2, 0.4, 0.6, 0.8]))]);
    let gradient = |a: u32, b: u32, c: u32, kind: u16| {
        F::Def(
            a,
            vec![tag(b"FDsc")],
            vec![
                (
                    tag(b"FDeF"),
                    F::Def(
                        b,
                        vec![tag(b"FilG")],
                        vec![
                            (tag(b"Type"), F::Enum(kind, 0)),
                            (tag(b"Grad"), F::Obj(tag(b"Grad"), vec![(tag(b"Cols"), F::Shared(vec![colour(c), F::Ref(c)]))])),
                        ],
                    ),
                ),
                (tag(b"FDeX"), F::F64s(vec![10.0, 0.0, 5.0, 0.0, 20.0, 5.0])),
            ],
        )
    };
    let curve = F::Obj(
        tag(b"PCvD"),
        vec![(
            tag(b"Data"),
            F::Pos(vec![
                F::U8(0),
                F::U32(1),
                F::Bool(true),
                F::Records(18, vec![rec(0.0, 0.0, 0), rec(5.0, 0.0, 1), rec(10.0, 5.0, 2), rec(10.0, 10.0, 0), rec(0.0, 10.0, 0), rec(0.0, 0.0, 0)]),
            ]),
        )],
    );
    let mut kids = vec![F::Def(
        next(),
        vec![tag(b"PCrv")],
        vec![(tag(b"Crvs"), curve.clone()), (tag(b"BFFl"), F::Shared(vec![gradient(next(), next(), next(), 0)]))],
    )];
    for (i, class) in [b"ShNR", b"ShpE", b"ShPy", b"ShSt", b"ShSS", b"ShPi", b"ShpT", b"ShTz", b"ShCl"].into_iter().enumerate() {
        let fields = vec![
            (tag(b"Side"), F::U32(6)),
            (tag(b"Smth"), F::Bool(i % 2 == 0)),
            (tag(b"Curv"), F::F32(0.5)),
            (tag(b"CTyp"), F::Raw(vec![0xaa, b'p', b'y', b'T', b'C', 4, 0, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 4, 0])),
            (tag(b"ShCR"), f32s(&[0.25, 0.0, 0.0, 0.0])),
            (tag(b"AngE"), F::F32(1.0)),
        ];
        kids.push(F::Def(
            next(),
            vec![tag(b"ShpN")],
            vec![
                (tag(b"Shpe"), F::Def(next(), vec![tag(class)], fields)),
                (tag(b"ShpB"), F::F64s(vec![0.0, 0.0, 40.0 + i as f64, 30.0])),
                (tag(b"Xfrm"), F::F64s(vec![1.0, 0.2, i as f64 * 10.0, 0.0, 1.0, 5.0])),
                (tag(b"BFFl"), F::Shared(vec![gradient(next(), next(), next(), (i % 4) as u16)])),
                (tag(b"AdCh"), F::Shared(vec![F::Def(next(), vec![tag(b"PCrv")], vec![(tag(b"Crvs"), curve.clone())])])),
            ],
        ));
    }
    kids.push(F::Def(
        next(),
        vec![tag(b"Comp")],
        vec![(tag(b"Chld"), F::Shared(vec![F::Def(next(), vec![tag(b"PCrv")], vec![(tag(b"Crvs"), curve.clone()), (tag(b"ComO"), F::Enum(2, 0))])]))],
    ));
    let run = F::Def(
        next(),
        vec![tag(b"GAtt")],
        vec![(tag(b"Doub"), F::Raw(vec![0x8a, b'b', b'u', b'o', b'D', 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x28, 0x40]))],
    );
    let block = F::Def(
        next(),
        vec![tag(b"StBl")],
        vec![
            (tag(b"Glyp"), F::Obj(tag(b"GStr"), vec![(tag(b"Utf8"), F::Str("Hi\u{2029}there\0".into()))])),
            (
                tag(b"GAtt"),
                F::Obj(tag(b"GlAS"), vec![(tag(b"Runs"), F::Objs(tag(b"GlAR"), vec![vec![(tag(b"Indx"), F::I32(9)), (tag(b"Item"), run)]]))]),
            ),
        ],
    );
    kids.push(F::Def(
        next(),
        vec![tag(b"TxtA")],
        vec![
            (tag(b"StSt"), F::Def(next(), vec![tag(b"Stry")], vec![(tag(b"Blok"), F::Shared(vec![block]))])),
            (
                tag(b"TxtH"),
                F::Def(next(), vec![tag(b"ArFr")], vec![(tag(b"FrmB"), F::F64s(vec![0.0, -10.0, 30.0, 0.0])), (tag(b"ArtV"), F::F64(10.0))]),
            ),
        ],
    ));
    let board = F::Def(
        next(),
        vec![tag(b"ShpN")],
        vec![
            (tag(b"ABEn"), F::Bool(true)),
            (tag(b"Shpe"), F::Def(next(), vec![tag(b"ShNR")], vec![])),
            (tag(b"ShpB"), F::F64s(vec![0.0, 0.0, 200.0, 100.0])),
            (tag(b"Chld"), F::Shared(kids)),
        ],
    );
    synth::stream(&[
        (tag(b"UVCn"), F::Obj(tag(b"UVCn"), vec![(tag(b"UPPI"), F::F64(96.0))])),
        (
            tag(b"DocR"),
            F::Def(
                1,
                vec![tag(b"DocN")],
                vec![(tag(b"Chld"), F::Shared(vec![F::Def(2, vec![tag(b"Sprd")], vec![(tag(b"Chld"), F::Shared(vec![board]))])]))],
            ),
        ),
    ])
}

fn affinity_native(stream: &[u8], method: vectorcraft_affinity::synth::Method) -> Vec<u8> {
    vectorcraft_affinity::synth::container(&[("doc.dat", stream, method)], None)
}

#[test]
fn the_native_affinity_sample_reaches_every_reader_path() {
    let bytes = affinity_native(&affinity_native_stream(), vectorcraft_affinity::synth::Method::Zstd);
    let l = vectorcraft_engine::cmd::fileio::load("x.af", &bytes).unwrap();
    assert!(!l.preview_only, "{:?}", l.warnings);
    let kinds = l.doc.layers.iter().flat_map(|l| l.children().into_iter().flatten()).count();
    assert!(kinds > 0);
    let all = format!("{:?}", l.doc.layers);
    for want in ["Path", "Text", "Compound"].iter().take(2) {
        assert!(all.contains(want), "{want} missing");
    }
    assert_eq!(l.doc.artboards.len(), 1);
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn affinity_native_documents_with_mutated_streams_render_and_export(
        cut in 0usize..4096,
        edits in prop::collection::vec((0usize..4096, any::<u8>()), 0..24),
    ) {
        let mut stream = affinity_native_stream();
        let n = stream.len();
        for (at, b) in edits { stream[at % n] = b; }
        stream.truncate(cut.max(16));
        let bytes = affinity_native(&stream, vectorcraft_affinity::synth::Method::Stored);
        survive("Affinity native stream", || vectorcraft_engine::cmd::fileio::load("x.af", &bytes).ok().map(|l| l.doc))?;
    }

    #[test]
    fn affinity_native_containers_with_mutated_bytes_render_and_export(
        edits in prop::collection::vec((0usize..8192, any::<u8>()), 1..16),
        method in 0u8..3,
    ) {
        use vectorcraft_affinity::synth::Method;
        let m = [Method::Stored, Method::Zlib, Method::Zstd][usize::from(method)];
        let mut bytes = affinity_native(&affinity_native_stream(), m);
        let n = bytes.len();
        for (at, b) in edits { bytes[at % n] = b; }
        survive("Affinity native container", || vectorcraft_engine::cmd::fileio::load("x.af", &bytes).ok().map(|l| l.doc))?;
    }

    #[test]
    fn affinity_garbage_never_panics(tail in prop::collection::vec(any::<u8>(), 0..2048)) {
        let mut bytes = vectorcraft_affinity::MAGIC.to_vec();
        bytes.extend(tail);
        survive("Affinity garbage", || vectorcraft_engine::cmd::fileio::load("x.af", &bytes).ok().map(|l| l.doc))?;
    }

    #[test]
    fn affinity_mutated_previews_render_and_export_without_panicking(
        cut in 0usize..400,
        edits in prop::collection::vec((0usize..400, any::<u8>()), 0..16),
    ) {
        let mut bytes = affinity_preview_sample();
        for (at, b) in edits {
            if let Some(byte) = bytes.get_mut(at) { *byte = b; }
        }
        bytes.truncate(cut);
        survive("Affinity preview", || vectorcraft_engine::cmd::fileio::load("x.af", &bytes).ok().map(|l| l.doc))?;
    }

    #[test]
    fn affinity_placement_is_rejected_without_panicking(tail in prop::collection::vec(any::<u8>(), 0..2048)) {
        let mut bytes = vectorcraft_affinity::MAGIC.to_vec();
        bytes.extend(tail);
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"renamed.png", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            for cmd in ["file.place", "file.place.info"] { assert!(no_panic(s.execute(cmd, &p)).is_err()); }
            assert!(no_panic(s.execute("file.place.queue", &json!({"files":[p]}))).is_err());
        });
        prop_assert!(r.is_ok(), "Affinity placement panicked: {:?}", r.err());
    }
}

// ---------- .cdr drawings ----------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn cdr_mutated_drawings_render_and_export(
        version in prop::sample::select(vec![300u16, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300]),
        compress: bool,
        features: bool,
        cut in 12usize..8192,
        edits in prop::collection::vec((0usize..8192, any::<u8>()), 0..24),
    ) {
        // The feature sample adds a style table, a clipping container and a lens to the shapes.
        let compress = compress && version >= 700;
        let mut bytes =
            if features { craft_cdr::synth::feature_sample(version, compress) } else { craft_cdr::synth::sample(version, compress) };
        let n = bytes.len();
        for (at, b) in edits { bytes[at % n] = b; }
        bytes.truncate(cut);
        survive("CDR drawing", || vectorcraft_engine::cmd::fileio::load("x.cdr", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "CDR drawing placement panicked: {:?}", r.err());
    }

    #[test]
    fn cdr_mutated_packages_render_and_export(
        version in prop::sample::select(vec![1400u16, 1500, 1600, 2000, 2700]),
        cut in prop::option::of(22usize..16384),
        edits in prop::collection::vec((any::<usize>(), 0usize..8192, any::<u8>()), 0..24),
    ) {
        // The members are edited before they are zipped, so that the edits reach the drawing
        // rather than stop at the CRC-32; the archive may then be cut short.
        let mut members = if version < 1600 {
            vec![craft_cdr::synth::Member { name: "content/riffData.cdr".into(), data: craft_cdr::synth::sample(version, true), deflate: false }]
        } else {
            let mut body = vec![craft_cdr::synth::chunk(b"vrsn", version.to_le_bytes().to_vec())];
            body.extend(craft_cdr::synth::sample_content(version));
            craft_cdr::synth::package16_members(version, &body, false, &[])
        };
        for (m, at, b) in edits {
            let k = m % members.len();
            let data = &mut members[k].data;
            if !data.is_empty() {
                let n = data.len();
                data[at % n] = b;
            }
        }
        let mut bytes = craft_cdr::synth::zip(&members, b"");
        if let Some(cut) = cut {
            bytes.truncate(cut);
        }
        survive("CDR package", || vectorcraft_engine::cmd::fileio::load("x.cdr", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "CDR package placement panicked: {:?}", r.err());
    }

    #[test]
    fn cdr_garbage_opens_and_places_without_panicking(tail in prop::collection::vec(any::<u8>(), 0..2048)) {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend(((4 + tail.len()) as u32).to_le_bytes());
        bytes.extend(b"CDRB");
        bytes.extend(tail);
        survive("CDR garbage", || vectorcraft_engine::cmd::fileio::load("x.cdr", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "CDR placement panicked: {:?}", r.err());
    }

    // Drawings of versions 1 and 2 are not RIFF files: a header of offsets, then records.
    #[test]
    fn cdr_mutated_wl_drawings_render_and_export(
        marker in prop::sample::select(vec![b'e', b'l', b'm']),
        cut in prop::option::of(0usize..1024),
        edits in prop::collection::vec((0usize..8192, any::<u8>()), 0..24),
    ) {
        let mut bytes = craft_cdr::synth::wl_sample(marker);
        let n = bytes.len();
        for (at, b) in edits { bytes[at % n] = b; }
        if let Some(cut) = cut {
            bytes.truncate(cut);
        }
        survive("CDR drawing of version 1 or 2", || vectorcraft_engine::cmd::fileio::load("x.cdr", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "placing a CDR drawing of version 1 or 2 panicked: {:?}", r.err());
    }

    #[test]
    fn cdr_garbage_after_a_wl_header_opens_and_places_without_panicking(
        marker in prop::sample::select(vec![b'e', b'l', b'm']),
        tail in prop::collection::vec(any::<u8>(), 0..2048),
    ) {
        // Offsets that fit the file, so the garbage is walked as records.
        let mut bytes = vec![b'W', b'L', marker, 0];
        bytes.extend(28u32.to_le_bytes());
        bytes.extend(28u32.to_le_bytes());
        bytes.extend((28 + tail.len() as u32).to_le_bytes());
        bytes.extend([0; 12]);
        bytes.extend(tail);
        survive("CDR garbage of version 1 or 2", || vectorcraft_engine::cmd::fileio::load("x.cdr", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cdr", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "placing CDR garbage of version 1 or 2 panicked: {:?}", r.err());
    }
}

// ---------- .cmx files ----------

proptest! {
    #![proptest_config(config())]

    #[test]
    fn cmx_mutated_files_render_and_export(
        cut in prop::option::of(12usize..4096),
        edits in prop::collection::vec((0usize..8192, any::<u8>()), 0..24),
    ) {
        let mut bytes = craft_cdr::synth_cmx::sample();
        let n = bytes.len();
        for (at, b) in edits { bytes[at % n] = b; }
        if let Some(cut) = cut {
            bytes.truncate(cut);
        }
        survive("CMX file", || vectorcraft_engine::cmd::fileio::load("x.cmx", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cmx", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "CMX placement panicked: {:?}", r.err());
    }

    #[test]
    fn cmx_garbage_opens_and_places_without_panicking(tail in prop::collection::vec(any::<u8>(), 0..2048)) {
        // A 32-bit header, so the garbage is walked as chunks, tables and instructions.
        let mut bytes = b"RIFF".to_vec();
        let cont = craft_cdr::synth_cmx::cont(b'4', 1e-7);
        bytes.extend(((4 + cont.len() + tail.len()) as u32).to_le_bytes());
        bytes.extend(b"CMX1");
        bytes.extend(cont);
        bytes.extend(tail);
        survive("CMX garbage", || vectorcraft_engine::cmd::fileio::load("x.cmx", &bytes).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width":100,"height":100})).unwrap();
            let p = json!({"name":"x.cmx", "dataBase64":vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "CMX placement panicked: {:?}", r.err());
    }
}

// ---------- Affinity documents with every feature the reader maps ----------

#[path = "support/affinity_docs.rs"]
mod affinity_docs;
#[path = "../../affinity/tests/support/text_fixture.rs"]
mod text_fixture;

/// One sample document: its object stream and, for embedded content, the embedded document's.
struct AffinitySample {
    name: &'static str,
    stream: Vec<u8>,
    embedded: Option<Vec<u8>>,
}

/// Original synthetic documents (not an Affinity writer) reaching what the reader maps beyond
/// [`affinity_native_stream`]: layer effects and colour overlays (also over tables), stroke
/// styles, dashes and brushes, paragraph alignment, frame text, underline, strikethrough and
/// line breaks inside a paragraph, tables with merged and two-line cells, master pages, CMYK
/// pixel layers with and without the document's CMYK profile, and embedded documents.
fn affinity_feature_samples() -> Vec<AffinitySample> {
    use affinity_docs::*;
    let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let fx = vec![shadow(), effect(b"OutG", 6.0), effect(b"InnS", 3.0), effect(b"BevE", 4.0), outline(0, 3.0), outline(2, 4.0), colour_overlay(0.5)];
    let canvas = canvas_stream(vec![
        with_effects(stroked(10, 1, &[]), fx),
        stroked(11, 0, &[]),
        stroked(12, 2, &[4.0, 2.0]),
        stroked(13, 3, &[]),
        mixed_alignment(1, 2),
        two_line_frame(150.0),
        decorated_text(70),
        with_effects(table_with(30, identity, ["A", "B\u{2028}two", "C", "D", "E"]), vec![colour_overlay(1.0)]),
    ]);
    let icc = vectorcraft_color::cms::icc_bytes(vectorcraft_color::cms::GENERIC_CMYK).unwrap();
    let inner = canvas_stream(vec![with_effects(rectangle(10, (0.0, 0.0), (50.0, 50.0)), vec![colour_overlay(1.0)]), artistic_text(20, "Hi", 40.0)]);
    vec![
        AffinitySample { name: "effects, strokes, text and tables", stream: canvas, embedded: None },
        AffinitySample { name: "master pages", stream: master_stream(3), embedded: None },
        AffinitySample { name: "CMYK pixels with a profile", stream: cmyk_stream(false, Some(&icc)), embedded: None },
        AffinitySample { name: "CMYK pixels with transparency", stream: cmyk_stream(true, None), embedded: None },
        AffinitySample { name: "embedded document", stream: canvas_stream(vec![embedded_node(20)]), embedded: Some(inner) },
    ]
}

/// A sample's file: its streams stored uncompressed, so mutations reach the object graph
/// instead of failing a decompressor's checks.
fn affinity_sample_file(stream: &[u8], embedded: Option<&[u8]>) -> Vec<u8> {
    use vectorcraft_affinity::synth::Method;
    let inner = embedded.map(|e| affinity_docs::packed(e, None, Method::Stored));
    affinity_docs::packed(stream, inner.as_deref(), Method::Stored)
}

#[test]
fn the_affinity_feature_samples_reach_what_they_are_for() {
    use vectorcraft_doc::NodeKind;
    let samples = affinity_feature_samples();
    let open = |i: usize| {
        let s: &AffinitySample = &samples[i];
        let l = vectorcraft_engine::cmd::fileio::load("x.af", &affinity_sample_file(&s.stream, s.embedded.as_deref())).unwrap();
        assert!(!l.preview_only, "{}: {:?}", s.name, l.warnings);
        l
    };
    fn all(d: &Document) -> Vec<&vectorcraft_doc::Node> {
        fn walk<'a>(nodes: &'a [std::sync::Arc<vectorcraft_doc::Node>], out: &mut Vec<&'a vectorcraft_doc::Node>) {
            for n in nodes {
                out.push(n);
                if let Some(c) = n.children() {
                    walk(c, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&d.layers, &mut out);
        out
    }
    let l = open(0);
    let nodes = all(&l.doc);
    let runs: Vec<&vectorcraft_doc::text::TextRun> =
        nodes.iter().filter_map(|n| if let NodeKind::Text(t) = &n.kind { Some(t.runs.iter()) } else { None }).flatten().collect();
    assert!(runs.iter().any(|r| r.style.underline) && runs.iter().any(|r| r.style.strikethrough));
    assert!(runs.iter().any(|r| r.text.contains("two")), "the table's two-line cell");
    assert!(nodes.iter().any(|n| n.appearance.effects.iter().any(|e| e.id == "stylize.dropShadow")));
    assert!(l.warnings.iter().any(|w| w.contains("bevel")), "{:?}", l.warnings);
    // Line styles: solid, None (no stroke), dashed and brush.
    use vectorcraft_doc::appearance::{AppearanceItem, StrokeLayer};
    let strokes = |n: &vectorcraft_doc::Node| -> Vec<StrokeLayer> {
        n.appearance.items.iter().filter_map(|i| if let AppearanceItem::Stroke(s) = i { Some(s.clone()) } else { None }).collect()
    };
    let boxes: Vec<_> = nodes.iter().filter(|n| n.name.as_deref() == Some("Magenta box")).collect();
    assert_eq!(boxes.len(), 4);
    assert_eq!(boxes.iter().filter(|b| strokes(b).is_empty()).count(), 1, "line style None");
    assert!(boxes.iter().any(|b| strokes(b).iter().any(|s| s.dash.is_some())), "dashed");
    let l = open(1);
    assert_eq!(l.doc.artboards.len(), 3);
    assert!(l.doc.layers.iter().any(|n| n.name.as_deref() == Some("Master A")));
    for i in [2, 3] {
        assert!(all(&open(i).doc).iter().any(|n| matches!(n.kind, NodeKind::Image(_))), "{}", samples[i].name);
    }
    assert!(all(&open(4).doc).iter().any(|n| n.name.as_deref() == Some("Magenta box")));
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn affinity_feature_documents_with_mutated_streams_render_and_export(
        sample in 0usize..5,
        inner in any::<bool>(),
        cut in 0usize..8192,
        edits in prop::collection::vec((0usize..8192, any::<u8>()), 0..24),
    ) {
        let samples = affinity_feature_samples();
        let s = &samples[sample % samples.len()];
        let (mut stream, mut embedded) = (s.stream.clone(), s.embedded.clone());
        // Mutate the embedded document's stream instead, where there is one.
        let target = match embedded.as_mut() {
            Some(e) if inner => e,
            _ => &mut stream,
        };
        let n = target.len();
        for (at, b) in edits { target[at % n] = b; }
        target.truncate(cut.max(16));
        let bytes = affinity_sample_file(&stream, embedded.as_deref());
        survive(s.name, || vectorcraft_engine::cmd::fileio::load("x.af", &bytes).ok().map(|l| l.doc))?;
    }
}

/// Import must not panic; whatever comes back must render and export without panicking either.
fn survive(what: &str, import: impl FnOnce() -> Option<Document>) -> Result<(), TestCaseError> {
    let r = catch_quiet(|| {
        if let Some(d) = import() {
            let mut r = vectorcraft_render::Renderer::new();
            if let Some(ab) = d.artboards.first() {
                let scale = (256.0 / ab.rect.width().max(ab.rect.height()).max(1.0)).min(1.0);
                if vectorcraft_render::raster_size(ab.rect, scale).is_ok() {
                    let _ = r.render_region(&d, ab.rect, scale, true).to_png();
                }
            }
            let _ = vectorcraft_svg::export(&d, &vectorcraft_svg::ExportOptions::default());
            let _ = vectorcraft_engine::export_pdf(&d, &vectorcraft_pdf::PdfOptions::default());
            let _ = vectorcraft_format::save_file(&d);
        }
    });
    r.map_err(|msg| TestCaseError::fail(format!("{what}: panicked: {msg}")))
}

fn rich_doc() -> Document {
    (*rich_session().doc().unwrap().doc).clone()
}

fn rich_svg() -> String {
    vectorcraft_svg::export(&rich_doc(), &vectorcraft_svg::ExportOptions::default())
}

fn rich_pdf() -> Vec<u8> {
    vectorcraft_engine::export_pdf(&rich_doc(), &vectorcraft_pdf::PdfOptions::uncompressed()).unwrap()
}

/// Numbers that tend to break arithmetic: zero, negatives, huge, tiny, exponents, junk.
fn arb_num() -> impl Strategy<Value = String> {
    prop_oneof![
        (-1000.0f64..1000.0).prop_map(|v| format!("{v:.2}")),
        Just("0".to_string()),
        Just("-0".to_string()),
        Just("1e308".to_string()),
        Just("-1e308".to_string()),
        Just("1e-308".to_string()),
        Just("1e15".to_string()),
        Just("4294967296".to_string()),
        Just("-2147483649".to_string()),
        Just("NaN".to_string()),
        Just("inf".to_string()),
        Just("".to_string()),
        Just("1e".to_string()),
        Just("--1".to_string()),
    ]
}

// ---------- SVG ----------

fn arb_svg_attr() -> impl Strategy<Value = String> {
    let names = prop::sample::select(vec![
        "x",
        "y",
        "width",
        "height",
        "rx",
        "ry",
        "r",
        "cx",
        "cy",
        "x1",
        "y1",
        "x2",
        "y2",
        "dx",
        "dy",
        "font-size",
        "stroke-width",
        "stroke-dasharray",
        "stroke-dashoffset",
        "stroke-miterlimit",
        "opacity",
        "fill-opacity",
        "offset",
        "startOffset",
        "letter-spacing",
        "rotate",
        "textLength",
        "fx",
        "fy",
        "fr",
        "points",
        "viewBox",
        "transform",
        "d",
        "fill",
        "stroke",
        "stop-color",
        "mask-type",
        "data-vectorcraft-mask",
        "href",
        "baseline-shift",
        "word-spacing",
        "kerning",
        "writing-mode",
        "font-weight",
        "font-variation-settings",
        "text-anchor",
        "side",
        "path",
        // Filters, spreads, hidden objects and linked images.
        "spreadMethod",
        "display",
        "filter",
        "clip-path",
        "stdDeviation",
        "flood-color",
        "flood-opacity",
        "in",
        "in2",
        "result",
        "operator",
        "values",
        "tableValues",
        "type",
        "preserveAspectRatio",
        "overflow",
        "data-vc-blend",
        // SVG Interactivity: event attributes, kept as text.
        "onclick",
        "onload",
    ]);
    let axes = prop::sample::select(vec!["wght", "wdth", "opsz", "wd", "slnt!", ""]);
    let value = prop_oneof![
        arb_num(),
        prop::collection::vec(arb_num(), 0..7).prop_map(|v| v.join(" ")),
        prop::collection::vec(arb_num(), 0..7).prop_map(|v| format!("matrix({})", v.join(" "))),
        prop::collection::vec(arb_num(), 0..3).prop_map(|v| format!("rotate({}) scale({})", v.join(" "), v.join(","))),
        prop::collection::vec((prop::sample::select(vec!["M", "L", "C", "Q", "A", "Z", "H", "V", "S", "T", "m", "a"]), arb_num()), 0..12)
            .prop_map(|v| v.iter().map(|(c, n)| format!("{c}{n} {n}")).collect::<Vec<_>>().join(" ")),
        prop::collection::vec(arb_num(), 0..2).prop_map(|v| format!("{}%", v.join(""))),
        // Variable font axis settings, well formed or not.
        prop::collection::vec((axes, arb_num()), 0..5).prop_map(|v| v.iter().map(|(t, n)| format!("'{t}' {n}")).collect::<Vec<_>>().join(", ")),
        prop::sample::select(vec!["normal", "initial", "unset"]).prop_map(str::to_string),
        // Colours (Lab too), mask options and links.
        prop::collection::vec(arb_num(), 0..4).prop_map(|v| format!("lab({})", v.join(" "))),
        prop::sample::select(vec!["noclip invert", "invert", "noclip", "alpha", "luminance", "url(#e)", "#e", "#zz", "none", "currentColor"])
            .prop_map(str::to_string),
        prop::sample::select(vec![
            "reflect",
            "repeat",
            "SourceAlpha",
            "SourceGraphic",
            "e",
            "over",
            "in",
            "table",
            "1 0",
            "multiply",
            "img/x%2.png",
            "file:///C:/nope.png",
            "https://x/y.png",
            "data:image/svg+xml;base64,PHN2Zy8+",
        ])
        .prop_map(str::to_string),
    ];
    (names, value).prop_map(|(n, v)| format!(" {n}=\"{v}\""))
}

fn arb_svg_element(depth: u32) -> BoxedStrategy<String> {
    let tag = prop::sample::select(vec![
        "rect",
        "circle",
        "ellipse",
        "line",
        "polyline",
        "polygon",
        "path",
        "text",
        "tspan",
        "textPath",
        "g",
        "use",
        "image",
        "linearGradient",
        "radialGradient",
        "stop",
        "pattern",
        "clipPath",
        "mask",
        "symbol",
        "defs",
        "svg",
        "marker",
        "a",
        "filter",
        "feGaussianBlur",
        "feOffset",
        "feFlood",
        "feComposite",
        "feMerge",
        "feMergeNode",
        "feColorMatrix",
        "feDropShadow",
        "feComponentTransfer",
        "feFuncA",
        "script",
    ]);
    let attrs = || prop::collection::vec(arb_svg_attr(), 0..6).prop_map(|v| v.concat());
    let leaf = (tag.clone(), attrs(), "[a-z ]{0,6}").prop_map(|(t, a, txt)| format!("<{t} id=\"e\"{a} href=\"#e\">{txt}</{t}>"));
    if depth == 0 {
        return leaf.boxed();
    }
    prop_oneof![
        leaf,
        (tag, attrs(), prop::collection::vec(arb_svg_element(depth - 1), 0..4)).prop_map(|(t, a, ch)| format!("<{t}{a}>{}</{t}>", ch.concat()))
    ]
    .boxed()
}

/// A character-level edit that keeps the string valid UTF-8 (the importer takes `&str`).
fn mutate_text(src: &str, cut: usize, edits: &[(usize, char)]) -> String {
    let mut chars: Vec<char> = src.chars().collect();
    for &(i, c) in edits {
        if !chars.is_empty() {
            let n = chars.len();
            chars[i % n] = c;
        }
    }
    chars.truncate(cut.min(chars.len()));
    chars.into_iter().collect()
}

/// Type shifted two billion points down exports to PDF: an input of
/// `svg_hostile_markup_never_panics`, cut down to its type.
#[test]
fn svg_type_shifted_two_billion_points_exports_to_pdf() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><text baseline-shift="-2147483649">f</text></svg>"#;
    let d = vectorcraft_svg::import(svg).unwrap();
    assert!(vectorcraft_engine::export_pdf(&d, &vectorcraft_pdf::PdfOptions::default()).is_ok());
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn svg_garbage_never_panics(s in ".{0,300}") {
        survive("svg garbage", || vectorcraft_svg::import(&s).ok())?;
    }

    #[test]
    fn svg_hostile_markup_never_panics(
        root in prop::collection::vec(arb_svg_attr(), 0..4),
        body in prop::collection::vec(arb_svg_element(2), 0..6),
    ) {
        let svg = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"{}>{}</svg>", root.concat(), body.concat());
        survive(&svg, || vectorcraft_svg::import(&svg).ok())?;
    }

    /// The linked `<rect>` gets an id from the importer, so attribute selectors in `sheet` reach the
    /// pass that copies style sheet matches into the elements' styles.
    #[test]
    fn svg_hostile_css_never_panics(
        sheet in r#"[a-z#.>*\[\]=:;{}@ !"'/,-]{0,160}"#,
        decls in r#"[a-z0-9.%: ;!-]{0,60}"#,
        body in prop::collection::vec(arb_svg_element(1), 0..3),
    ) {
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\"><style>{sheet}</style><g class=\"a b\"><text id=\"t\" class=\"a\" x=\"1 2 3\" y=\"9\" rotate=\"5\" style=\"{decls}\">ab<tspan dy=\"3\">c</tspan></text></g><a href=\"#t\"><rect width=\"5\" height=\"5\"/></a>{}</svg>",
            body.concat()
        );
        survive(&svg, || vectorcraft_svg::import(&svg).ok())?;
    }

    /// Style sheets of descendant, child, sibling and `:not()` selectors over nested groups and the
    /// `<use>` copies of a rectangle inside them, in an SVG and in an SVG image it shows: the import
    /// measures their matching before its own cascade and the SVG reader run.
    #[test]
    fn svg_hostile_selectors_over_copies_never_panic(
        sheet in r#"([a-z.#:()>+* ]{1,24}\{fill:red\} ?){1,6}"#,
        depth in 1usize..12,
        copies in 0usize..4,
    ) {
        let groups = format!("{}<rect id=\"r\" class=\"a\" width=\"1\" height=\"1\"/>{}", "<g class=\"a\">".repeat(depth), "</g>".repeat(depth));
        let inner = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"4\"><style>{sheet}</style>{groups}{}</svg>",
            "<use href=\"#r\"/>".repeat(copies)
        );
        let image = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"4\"><image href=\"data:image/svg+xml;base64,{}\" width=\"4\" height=\"4\"/></svg>",
            vectorcraft_svg::base64_encode(inner.as_bytes())
        );
        for svg in [inner, image] {
            survive(&svg, || vectorcraft_svg::import(&svg).ok())?;
        }
    }

    /// Clip paths made of text, of `<use>` elements showing text, and of both with shapes, each with
    /// hostile attributes, clipping art that may clip by them in turn.
    #[test]
    fn svg_text_clip_paths_never_panic(
        text in prop::collection::vec(arb_svg_attr(), 0..5),
        used in prop::collection::vec(arb_svg_attr(), 0..5),
        clip in prop::collection::vec(arb_svg_attr(), 0..3),
        body in prop::collection::vec(arb_svg_element(1), 0..3),
    ) {
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><defs><text id=\"t\" font-size=\"20\"{}>ab<tspan dy=\"3\">c</tspan></text></defs><clipPath id=\"e\"{}><text y=\"30\"{}>Clip<tspan x=\"5\" y=\"60\">it</tspan></text><use href=\"#t\"{}/><rect width=\"5\" height=\"5\"/></clipPath><clipPath id=\"f\" clip-path=\"url(#e)\"><use xlink:href=\"#t\" x=\"9\"/></clipPath><rect width=\"90\" height=\"90\" clip-path=\"url(#e)\"/><g clip-path=\"url(#f)\">{}</g></svg>",
            text.concat(),
            clip.concat(),
            text.concat(),
            used.concat(),
            body.concat()
        );
        survive(&svg, || vectorcraft_svg::import(&svg).ok())?;
    }

    #[test]
    fn svg_mutated_export_never_panics(
        cut in 0usize..20_000,
        edits in prop::collection::vec((0usize..20_000, prop::sample::select(vec!['<', '>', '"', '/', '-', '9', 'e', '.', ' ', '#', '%', '&', ';', 'x'])), 0..10),
    ) {
        let svg = mutate_text(&rich_svg(), cut, &edits);
        survive("mutated svg", || vectorcraft_svg::import(&svg).ok())?;
    }
}

// ---------- PDF ----------

fn handmade_pdf(content: &str, resources: &str, media: &str) -> Vec<u8> {
    handmade_pdf_with(content, resources, media, &[])
}

/// The same, followed by the objects `more` (numbered from 5).
fn handmade_pdf_with(content: &str, resources: &str, media: &str, more: &[Vec<u8>]) -> Vec<u8> {
    let page = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox {media} /Contents 4 0 R /Resources {resources} >>"),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len() + 1),
    ];
    pdf_of(&page.into_iter().map(String::into_bytes).chain(more.iter().cloned()).collect::<Vec<_>>())
}

/// The PDF of the objects `objs` (numbered from 1, the catalog first).
fn pdf_of(objs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for off in offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

/// A file filling thousands of paths with function-based shadings, each a long PostScript
/// calculator function of fine stripes, opens in bounded time: each shading is read once, and all
/// of them within one budget of function evaluations per import (#3621 review).
#[test]
fn pdf_many_paths_filled_with_long_function_shadings_open_in_bounded_time() {
    let shadings = 20;
    // Five hundred no-op steps, then stripes: red, then white, along x.
    let program = format!("{{ pop {}dup truncate sub 0.5 le {{ 1 0 0 }} {{ 1 1 1 }} ifelse }}", "1 mul ".repeat(500));
    let patterns: String = (0..shadings)
        .map(|i| format!("/P{i} << /PatternType 2 /Shading << /ShadingType 1 /ColorSpace /DeviceRGB /Domain [0 {} 0 1] /Matrix [1 0 0 100 0 0] /Function {} 0 R >> >> ", 200 + i, 5 + i))
        .collect();
    let functions: Vec<Vec<u8>> = (0..shadings)
        .map(|i| {
            format!("<< /FunctionType 4 /Domain [0 {} 0 1] /Range [0 1 0 1 0 1] /Length {} >>\nstream\n{program}\nendstream", 200 + i, program.len())
                .into_bytes()
        })
        .collect();
    let content: String = (0..5000).map(|i| format!("/Pattern cs /P{} scn {} {} 2 2 re f\n", i % shadings, i % 100 * 2, i / 100 * 2)).collect();
    let pdf = handmade_pdf_with(&content, &format!("<< /Pattern << {patterns}>> >>"), "[0 0 200 100]", &functions);
    let start = std::time::Instant::now();
    let d = vectorcraft_pdf::import(&pdf).unwrap();
    let took = start.elapsed();
    assert!(!d.layers.is_empty());
    assert!(took < std::time::Duration::from_secs(30), "took {took:?}");
}

/// One content-stream operation with junk operands.
fn arb_pdf_op() -> impl Strategy<Value = String> {
    let op = prop::sample::select(vec![
        "m", "l", "c", "v", "y", "h", "re", "f", "f*", "F", "S", "s", "B", "B*", "b", "n", "W", "W*", "q", "Q", "cm", "w", "J", "j", "M", "d", "ri",
        "i", "gs", "g", "G", "rg", "RG", "k", "K", "cs", "CS", "sc", "SC", "scn", "SCN", "sh", "Do", "BT", "ET", "Tf", "Td", "TD", "Tm", "T*", "Tc",
        "Tw", "Tz", "TL", "Tr", "Ts", "Tj", "TJ", "'", "\"", "BMC", "BDC", "EMC", "d0", "d1",
    ]);
    let operand = prop_oneof![
        arb_num(),
        prop::collection::vec(arb_num(), 0..5).prop_map(|v| format!("[{}]", v.join(" "))),
        Just("/F1".to_string()),
        Just("/Sh0".to_string()),
        Just("/Im0".to_string()),
        Just("/GS0".to_string()),
        Just("/DeviceRGB".to_string()),
        Just("/CS0".to_string()),
        Just("/CS1".to_string()),
        Just("/P0".to_string()),
        Just("/DeviceCMYK".to_string()),
        Just("/Pattern".to_string()),
        Just("(Hi)".to_string()),
        Just("<00ff>".to_string()),
        Just("[(a) -250 (b)]".to_string()),
    ];
    (prop::collection::vec(operand, 0..7), op).prop_map(|(args, op)| format!("{} {op}", args.join(" ")))
}

fn arb_resources() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "<< >>".to_string(),
        "<< /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >>".to_string(),
        "<< /ExtGState << /GS0 << /CA 0.5 /ca -3 /BM /Multiply /SMask /None >> >> >>".to_string(),
        "<< /Shading << /Sh0 << /ShadingType 2 /ColorSpace /DeviceRGB /Coords [0 0 1e308 0] /Function << /FunctionType 2 /Domain [0 1] /C0 [1 0 0] /C1 [0 0 1] /N 1 >> >> >> >>".to_string(),
        "<< /Shading << /Sh0 << /ShadingType 3 /ColorSpace /DeviceRGB /Coords [0 0 0 0 0 -5] /Function << /FunctionType 2 /Domain [0 1] /C0 [1] /C1 [] /N -1 >> >> >> >>".to_string(),
        "<< /XObject << /Im0 << /Type /XObject /Subtype /Image /Width 4294967295 /Height 0 /BitsPerComponent 8 /ColorSpace /DeviceRGB /Length 3 >> >> >>".to_string(),
        "<< /Pattern << /P0 << /PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 0 0] /XStep 0 /YStep -0 /Resources << >> >> >> >>".to_string(),
        // Ink colour spaces with junk names, alternates and tint transforms (spot swatch import).
        "<< /ColorSpace << /CS0 [/Separation /All /DeviceCMYK << /FunctionType 2 /Domain [0 1] /C0 [0 0 0 0] /C1 [1 1 1 1] /N 1 >>] >> >>".to_string(),
        "<< /ColorSpace << /CS0 [/Separation /#00 /Lab << /FunctionType 2 /Domain [0 1] /C0 [1e308] /C1 [NaN 5] /N -1 >>] >> >>".to_string(),
        "<< /ColorSpace << /CS0 [/DeviceN [] /DeviceGray << /FunctionType 2 /Domain [0 1] /C0 [0] /C1 [1] /N 1 >>] >> >>".to_string(),
        "<< /ColorSpace << /CS0 [/DeviceN [/A /None /Cyan /All] [/ICCBased << /N 4 >>] << /FunctionType 2 /Domain [0 1] /C0 [0] /C1 [1] /N 1 >>] >> >>".to_string(),
        "<< /Shading << /Sh0 << /ShadingType 2 /ColorSpace [/Separation /Ink /DeviceCMYK << /FunctionType 2 /Domain [0 1] /C0 [0 0 0 0] /C1 [0 1 0 0] /N 1 >>] /Coords [0 0 100 0] /Function << /FunctionType 2 /Domain [0 1] /C0 [0] /C1 [1] /N 1 >> >> >> >>".to_string(),
        // Inks with the same values and other names, as color spaces, a shading pattern and a
        // shading (the page's walk tells them apart).
        "<< /ColorSpace << /CS0 [/Separation /A /DeviceRGB << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >>] /CS1 [/Separation /B /DeviceRGB << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >>] >> /Pattern << /P0 << /PatternType 2 /Shading << /ShadingType 2 /ColorSpace [/Separation /B /DeviceRGB << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >>] /Coords [0 0 100 0] /Function << /FunctionType 2 /Domain [0 1] /C0 [0] /C1 [1] /N 1 >> >> >> >> /Shading << /Sh0 << /ShadingType 2 /ColorSpace [/Separation /A /DeviceRGB << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >>] /Coords [0 0 100 0] /Function << /FunctionType 2 /Domain [0 1] /C0 [0] /C1 [1] /N 1 >> >> >> >>".to_string(),
    ])
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn pdf_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..600)) {
        let mut b = b"%PDF-1.7\n".to_vec();
        b.extend(bytes);
        survive("pdf garbage", || vectorcraft_pdf::import(&b).ok())?;
    }

    #[test]
    fn pdf_hostile_content_never_panics(
        ops in prop::collection::vec(arb_pdf_op(), 0..40),
        resources in arb_resources(),
        media in prop::sample::select(vec!["[0 0 100 100]", "[0 0 0 0]", "[0 0 1e308 1e308]", "[100 100 0 0]", "[-5 -5 -1 -1]", "[0 0 14400 14400]", "[]", "[0 0 NaN 5]"]),
    ) {
        let pdf = handmade_pdf(&ops.join("\n"), &resources, media);
        survive(&ops.join(" "), || vectorcraft_pdf::import(&pdf).ok())?;
    }

    #[test]
    fn pdf_mutated_export_never_panics(
        cut in 0usize..40_000,
        flips in prop::collection::vec((0usize..40_000, prop::sample::select(vec![b'0', b'9', b'-', b'.', b' ', b'[', b']', b'<', b'>', b'/', b'(', b'e', 0u8, 0xff])), 0..10),
    ) {
        let mut b = rich_pdf();
        for (i, c) in flips {
            let n = b.len();
            b[i % n] = c;
        }
        b.truncate(cut.min(b.len()).max(9));
        survive("mutated pdf", || vectorcraft_pdf::import(&b).ok())?;
    }
}

/// A dictionary value: one the reader reads, or one it skips over but can't parse (a name with a
/// broken `#` escape, a number with a stray `-`).
fn arb_dict_value() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec!["/Fine", "/#zz", "12-.5"])
}

/// A page's resources: `space` in the color space dictionary before a Separation space, `font`
/// in the font dictionary before a font, and a calculator function (`function`: plain, with
/// decode parameters the reader can't apply, or decoding past the cap) written as the shading's
/// function, or inside an array or a dictionary under `/Properties` (`held`). Returns the
/// resources and whether a file with them is refused.
fn unreadable_resources(space: &str, font: &str, function: &str, held: &str) -> (String, bool) {
    let program = deflated(b"{ dup 1 exch sub 0 }");
    let (function, refused) = match function {
        "Columns 0" => (calculator(&program, "/DecodeParms [null << /Predictor 2 /Columns 0 >>]"), true),
        "a huge Columns" => (calculator(&program, "/DecodeParms [null << /Predictor 12 /Colors 4 /Columns 99999999999 >>]"), true),
        "spaces past the cap" => (spaces_past_the_cap_once().to_string(), true),
        _ => (calculator(&program, ""), false),
    };
    let ramp = "<< /FunctionType 2 /Domain [0 1] /C0 [1 0 0] /C1 [0 0 1] /N 1 >>";
    let (shaded, other) = match held {
        "in an array" => (ramp, format!("/Properties << /P0 [{function}] >>")),
        "in a dictionary" => (ramp, format!("/Properties << /P0 << /K {function} >> >>")),
        _ => (function.as_str(), String::new()),
    };
    let tint = "<< /FunctionType 2 /Domain [0 1] /C0 [0 0 0 0] /C1 [0 0 1 0] /N 1 >>";
    let resources = format!(
        "<< /ColorSpace << /A {space} /CS0 [/Separation /Gold /DeviceCMYK {tint}] >> \
         /Font << /A {font} /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> \
         /Shading << /Sh0 << /ShadingType 2 /ColorSpace /DeviceRGB /Coords [0 0 100 0] /Function {shaded} >> >> {other} >>"
    );
    (resources, refused)
}

proptest! {
    #![proptest_config(config())]

    /// A page that paints with each of [`unreadable_resources`] is imported without a panic. The
    /// file opens with its art, or is refused when its calculator function decodes past the cap or
    /// has decode parameters the reader can't apply.
    #[test]
    fn pdf_resources_the_reader_cannot_parse_never_panic(
        space in arb_dict_value(),
        font in arb_dict_value(),
        function in prop::sample::select(vec!["plain", "Columns 0", "a huge Columns", "spaces past the cap"]),
        held in prop::sample::select(vec!["as the shading's function", "in an array", "in a dictionary"]),
    ) {
        let (resources, refused) = unreadable_resources(space, font, function, held);
        let pdf = handmade_pdf("/CS0 cs 1 scn 0 0 50 50 re f BT /F1 12 Tf 10 60 Td (Hi) Tj ET /Sh0 sh", &resources, "[0 0 100 100]");
        let what = format!("{space} and {font}, a function ({function}) {held}");
        let mut read = None;
        survive(&what, || {
            let r = vectorcraft_pdf::import(&pdf);
            read = Some(r.as_ref().map(|d| d.layers.iter().any(|l| l.children().is_some_and(|c| !c.is_empty()))).map_err(ToString::to_string));
            r.ok()
        })?;
        // A function written in the page's resources goes by the page's object (#4366).
        let why = if function == "spaces past the cap" {
            "a stream in it (object 3) would decode to more than 256 MB"
        } else {
            "a stream in it (object 3) has decode parameters that can't be applied"
        };
        match read.expect("the import ran") {
            Ok(art) => prop_assert!(!refused && art, "{what}: opened (its art read: {art})"),
            Err(e) => prop_assert!(refused && e.contains(why), "{what}: {e}"),
        }
    }
}

/// A page box with junk in it: inverted, empty, huge, NaN or outside the media box.
fn arb_box() -> impl Strategy<Value = Option<[f64; 4]>> {
    prop_oneof![
        Just(None),
        Just(Some([10.0, 10.0, 90.0, 90.0])),
        Just(Some([90.0, 90.0, 10.0, 10.0])),
        Just(Some([0.0, 0.0, 0.0, 0.0])),
        Just(Some([-1e308, -1e308, 1e308, 1e308])),
        Just(Some([f64::NAN, 0.0, 50.0, f64::INFINITY])),
        Just(Some([200.0, 200.0, 300.0, 300.0])),
    ]
}

proptest! {
    #![proptest_config(config())]

    /// Every page box, rotation, `UserUnit`, page pick, password and Crop To, through
    /// `document.open` and `document.pdfInfo` (with a thumbnail).
    #[test]
    fn pdf_import_options_never_panic(
        boxes in prop::collection::vec(arb_box(), 5),
        rotate in prop::sample::select(vec![0, 90, 180, 270, 45, -90, 1_000_000_000]),
        unit in prop::sample::select(vec!["", "10", "2.5", "1", "0", "-10", "0.001", "99999999999999999999", "/Ten", "(10)", "99 0 R", "huge"]),
        encrypt in any::<bool>(),
        password in prop::sample::select(vec!["", "pw", "wrong", "\u{0}\u{ff}"]),
        pages in prop::sample::select(vec!["1", "2", "1-2", "2-1", "0", "-", "1-", "all", "1,1,2", "99999999999999999999"]),
        crop in prop::sample::select(vec!["bounding", "art", "crop", "trim", "bleed", "media", "nope"]),
    ) {
        use vectorcraft_testkit::pdf::{PdfPage, pdf};
        // A `UserUnit` of 1 followed by 308 zeros (a PDF number has no exponent).
        let unit = if unit == "huge" { format!("1{}", "0".repeat(308)) } else { unit.to_string() };
        let page = PdfPage {
            media: boxes[0].unwrap_or([0.0, 0.0, 100.0, 100.0]),
            crop: boxes[1],
            bleed: boxes[2],
            trim: boxes[3],
            art: boxes[4],
            rotate,
            entries: if unit.is_empty() { String::new() } else { format!("/UserUnit {unit}") },
            ..PdfPage::new(0.0, 0.0, "0 0 1 rg 20 20 50 50 re f")
        };
        let bytes = pdf(&[page.clone(), page], encrypt.then_some("pw"));
        let b64 = vectorcraft_format::base64_encode(&bytes);
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            let p = json!({"name": "x.pdf", "dataBase64": b64, "pages": pages, "cropTo": crop, "password": password, "thumbnail": 1, "thumbnailSize": 64});
            let _ = no_panic(s.execute("document.pdfInfo", &p));
            if no_panic(s.execute("document.open", &p)).is_ok() {
                let d = (*s.doc().unwrap().doc).clone();
                survive("pdf options", || Some(d)).unwrap();
            }
        });
        r.map_err(|msg| TestCaseError::fail(format!("{pages} {crop} {rotate} {unit}: panicked: {msg}")))?;
    }
}

/// An XMP packet with every property File Info reads: arrays, language alternatives, attributes,
/// a qualified value, a URI value and dates.
const RICH_XMP: &str = "<?xpacket begin=\"\u{feff}\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?><x:xmpmeta xmlns:x=\"adobe:ns:meta/\">\
    <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"><rdf:Description rdf:about=\"\" \
    xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:xmp=\"http://ns.adobe.com/xap/1.0/\" \
    xmlns:xmpRights=\"http://ns.adobe.com/xap/1.0/rights/\" xmlns:pdf=\"http://ns.adobe.com/pdf/1.3/\" \
    xmlns:q=\"urn:example:qualifiers\" xmpRights:Marked=\"True\" xmp:ModifyDate=\"2026-10-06T09:00:00.5+05:30\">\
    <dc:creator><rdf:Seq><rdf:li rdf:parseType=\"Resource\"><rdf:value>Ada</rdf:value><q:role>Author</q:role></rdf:li><rdf:li>Grace</rdf:li></rdf:Seq></dc:creator>\
    <dc:description><rdf:Alt><rdf:li xml:lang=\"x-default\">A poster</rdf:li><rdf:li xml:lang=\"de\">Ein Plakat</rdf:li></rdf:Alt></dc:description>\
    <dc:subject><rdf:Bag><rdf:li>fair</rdf:li><rdf:li>print</rdf:li></rdf:Bag></dc:subject><pdf:Keywords>a, b; c</pdf:Keywords>\
    <dc:rights><rdf:Alt><rdf:li xml:lang=\"x-default\">(c) 2026 Ada</rdf:li></rdf:Alt></dc:rights>\
    <xmpRights:WebStatement rdf:resource=\"https://example.com/rights\"/><xmp:CreateDate>2026-10-05T14:03:00Z</xmp:CreateDate>\
    </rdf:Description></rdf:RDF></x:xmpmeta><?xpacket end=\"w\"?>";

/// A PDF whose trailer names an information dictionary with the entries `info` and whose catalog's
/// metadata stream holds `xmp`.
fn file_info_pdf(info: &str, xmp: &str) -> Vec<u8> {
    use vectorcraft_testkit::pdf::{PdfPage, first_extra, pdf_with_catalog};
    let x = first_extra(1);
    let stream = format!("<< /Type /Metadata /Subtype /XML /Length {} >>\nstream\n{xmp}\nendstream", xmp.len());
    let dict = format!("<< {info} >>");
    let pdf = pdf_with_catalog(&[PdfPage::new(100.0, 100.0, "0 0 1 rg 10 10 50 50 re f")], &[&stream, &dict], &format!("/Metadata {x} 0 R "), None);
    // The trailer follows the cross-reference table, so a longer trailer moves no object.
    String::from_utf8(pdf).unwrap().replacen("/Root 1 0 R", &format!("/Root 1 0 R /Info {} 0 R", x + 1), 1).into_bytes()
}

/// The sample `pdf_file_info_never_panics` mutates opens with the File Info of its packet.
#[test]
fn the_file_info_sample_reaches_the_reader() {
    let pdf = file_info_pdf("/Author (Info author) /Subject (Info subject)", RICH_XMP);
    let mut s = vectorcraft_engine::Session::new();
    s.execute("document.open", &json!({"name": "x.ai", "dataBase64": vectorcraft_format::base64_encode(&pdf)})).unwrap();
    let info = s.execute("file.info", &json!({})).unwrap();
    assert_eq!((info["author"].as_str(), info["description"].as_str()), (Some("Ada, Grace"), Some("A poster")));
    assert_eq!((info["copyrightStatus"].as_str(), info["keywords"].clone()), (Some("copyrighted"), json!(["fair", "print"])));
    assert_eq!(info["copyrightUrl"].as_str(), Some("https://example.com/rights"));
}

proptest! {
    #![proptest_config(config())]

    /// File Info read on open: mutated XMP packets, some with elements nested deeply, and junk
    /// information dictionaries, opened as a PDF and as a `.ai`.
    #[test]
    fn pdf_file_info_never_panics(
        cut in 0usize..2_000,
        edits in prop::collection::vec((0usize..2_000, prop::sample::select(vec!['<', '>', '"', '/', ':', '-', '+', 'T', 'Z', '9', '0', '.', ' ', '&', ';', '=', '#'])), 0..10),
        info in prop::sample::select(vec![
            "",
            "/Author (Ada) /Subject (A poster) /Keywords (a;b,,c ;) /CreationDate (D:2026) /ModDate (D:20261005140300-08'30')",
            "/CreationDate (D:99999999999999+99'99') /ModDate (D:20261301) /Author 5 /Keywords [(a)]",
            "/Subject <FEFF00> /Author <FEFFD800> /Keywords <EFBBBF> /Title (\\377\\376)",
            "/CreationDate (D:00000101000000Z) /ModDate (D:99991231235959+23'59')",
        ]),
        nest in prop::sample::select(vec![0usize, 0, 0, 251, 100_000]),
    ) {
        // `nest` elements nested in one another around the packet.
        let xmp = format!("{}{}{}", "<a>".repeat(nest), mutate_text(RICH_XMP, cut, &edits), "</a>".repeat(nest));
        let pdf = file_info_pdf(info, &xmp);
        let b64 = vectorcraft_format::base64_encode(&pdf);
        let r = catch_quiet(|| {
            let _ = vectorcraft_pdf::file_info(&pdf, None);
            let mut s = vectorcraft_engine::Session::new();
            for name in ["x.pdf", "x.ai"] {
                let _ = no_panic(s.execute("document.open", &json!({"name": name, "dataBase64": b64})));
                let _ = no_panic(s.execute("file.info", &json!({})));
            }
        });
        r.map_err(|msg| TestCaseError::fail(format!("{info}: panicked: {msg}")))?;
    }
}

/// Hostile content over layers (optional content, one group off), soft masks, tiling patterns,
/// a patch mesh, an unextended shading and fonts: the import's own readings of the file.
fn rich_resources_pdf(content: &str, config: &str) -> Vec<u8> {
    use vectorcraft_testkit::pdf::{PdfPage, first_extra, pdf_with_catalog};
    let x = first_extra(1);
    let stream = |dict: &str, data: &str| format!("<< {dict} /Length {} >>\nstream\n{data}\nendstream", data.len());
    let extra = [
        "<< /Type /OCG /Name (A) >>".to_string(),
        "<< /Type /OCG /Name <FEFF> /Usage << /Print << /PrintState /OFF >> >> >>".to_string(),
        stream(
            "/Type /XObject /Subtype /Form /BBox [0 0 100 100] /Group << /S /Transparency /CS /DeviceRGB /I true /K true >>",
            "1 g 0 0 50 50 re f /OC /MC0 BDC 0 g 0 0 5 5 re f EMC",
        ),
        stream(
            "/PatternType 1 /PaintType 2 /TilingType 1 /BBox [0 0 1e-9 10] /XStep 0.001 /YStep 10 /Resources << >>",
            "0 0 5 5 re f /P0 scn 0 0 1 1 re f",
        ),
        stream(
            "/ShadingType 6 /ColorSpace /DeviceRGB /BitsPerCoordinate 8 /BitsPerComponent 8 /BitsPerFlag 8 /Decode [0 255 0 255 0 1 0 1 0 1] /Filter /ASCIIHexDecode",
            "000A0A0A250A3F0A5A255A3F5A5A5A5A3F5A255A0A3F0A250AFF000000FF000000FFFFFFFF>",
        ),
        stream("/Type /XObject /Subtype /Form /BBox [0 0 100 100] /OC 6 0 R", "/X0 Do"),
    ];
    let (a, b, form, pat, mesh, hidden) = (x, x + 1, x + 2, x + 3, x + 4, x + 5);
    let resources = format!(
        "/Properties << /MC0 {a} 0 R /MC1 {b} 0 R >> /XObject << /X0 {form} 0 R /X1 {hidden} 0 R >> /Pattern << /P0 {pat} 0 R >> /Shading << /Sh0 {mesh} 0 R /Sh1 << /ShadingType 3 /ColorSpace /DeviceGray /Coords [50 50 0 50 50 40] /Extend [false false] /Function << /FunctionType 2 /Domain [0 1] /C0 [0] /C1 [1] /N 1 >> >> >> /ExtGState << /GS0 << /OP true /OPM 1 /SMask << /Type /Mask /S /Luminosity /G {form} 0 R /BC [1] /TR << /FunctionType 2 /Domain [0 1] /C0 [1] /C1 [0] /N 1 >> >> >> /GS1 << /OP true /op false /SMask << /Type /Mask /S /Alpha /G {form} 0 R >> >> /GS2 << /OP 7 /op (x) /OPM -3 >> /GS3 << /op true >> >> /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /ABCDEF+Odd-Name,Bold >> >> /ColorSpace << /CS0 [/Pattern /DeviceRGB] /CS1 [/Separation /A /DeviceRGB << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >>] /CS2 [/Separation /B /DeviceRGB << /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 1] /N 1 >>] >>"
    );
    let catalog = format!("/OCProperties << /OCGs [{a} 0 R {b} 0 R 99 0 R] /D << {} >> >> ", config.replace("{B}", &format!("{b} 0 R")));
    let extra: Vec<&str> = extra.iter().map(String::as_str).collect();
    pdf_with_catalog(&[PdfPage { resources, ..PdfPage::new(100.0, 100.0, content) }], &extra, &catalog, None)
}

fn arb_rich_op() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_pdf_op(),
        prop::sample::select(vec![
            "/OC /MC0 BDC",
            "/OC /MC1 BDC",
            "/OC /Nope BDC",
            "/Span << /MCID 3 >> BDC",
            "/X BMC",
            "EMC",
            "/X0 Do",
            "/X1 Do",
            "/GS0 gs",
            "/GS1 gs",
            // Overprint (#1388): set, malformed, and for fills alone.
            "/GS2 gs",
            "/GS3 gs",
            "/CS0 cs 1 0 0 /P0 scn",
            "/Pattern cs /P0 scn",
            // Inks with the same values and other names: set, and a pattern set in one.
            "/CS1 cs 1 scn",
            "/CS2 cs 0.5 scn",
            "/CS2 CS 1 SCN",
            "/CS1 cs /P0 scn",
            "/Sh0 sh",
            "/Sh1 sh",
            "BT /F1 12 Tf 10 10 Td (Hi there) Tj [(a) -900 (b)] TJ ET",
            // Lines wrapped in a frame, digits set apart by gaps, glyph outlines stroked (#508).
            "BT /F1 10 Tf 10 80 Td (Wrapped line ) Tj 0 -12 Td (and its next ) Tj 0 -12 Td (end.) Tj ET",
            "BT /F1 10 Tf 10 40 Td (3) Tj 20 0 Td (5331) Tj 40 0 Td (0106) Tj ET",
            "0.5 w 10 10 m 14 18 l 18 10 l h S",
            "7 Tr",
            "q 10 10 50 50 re W n",
            "Q",
            "0 0 100 100 re f",
            "10 10 20 20 re B",
        ])
        .prop_map(String::from),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn pdf_layers_masks_patterns_and_text_never_panic(
        ops in prop::collection::vec(arb_rich_op(), 0..40),
        config in prop::sample::select(vec!["/OFF [{B}]", "/BaseState /OFF", "/BaseState /Unchanged /ON [{B}] /Locked [{B}]", "", "/OFF {B}"]),
        layers in any::<bool>(),
        outlines in any::<bool>(),
    ) {
        let bytes = rich_resources_pdf(&ops.join("\n"), config);
        let opts = vectorcraft_pdf::ImportOptions {
            layers,
            text_as: if outlines { vectorcraft_pdf::TextAs::Outlines } else { vectorcraft_pdf::TextAs::Text },
            ..Default::default()
        };
        survive(&ops.join(" "), || vectorcraft_pdf::import_with_report(&bytes, &opts).ok().map(|r| r.document))?;
    }
}

// ---------- library files ----------

/// The `data` of a library file `cmd` writes for the rich document.
fn saved(cmd: &str, params: Value) -> String {
    rich_session().execute(cmd, &params).unwrap()["data"].as_str().unwrap().to_string()
}

/// `r`, after failing the case if the command panicked. The engine's guard reports a panic as
/// `EngineError::Internal`.
fn no_panic(r: Result<Value, vectorcraft_engine::EngineError>) -> Result<Value, vectorcraft_engine::EngineError> {
    if let Err(vectorcraft_engine::EngineError::Internal { cmd, msg }) = &r {
        panic!("{cmd} panicked: {msg}");
    }
    r
}

/// Library file `data` loaded with `load`; what it holds is then used on the rich document
/// (`then`, given the load's result), which renders and exports.
fn survive_library(what: &str, load: &str, data: &str, then: impl FnOnce(&mut vectorcraft_engine::Session, &Value)) -> Result<(), TestCaseError> {
    survive(what, || {
        let mut s = rich_session();
        let r = no_panic(s.execute(load, &json!({"data": data, "name": "fuzz"}))).ok()?;
        then(&mut s, &r);
        Some((*s.doc().ok()?.doc).clone())
    })
}

fn swatches(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "swatch.library.load", data, |s, r| {
        let _ = no_panic(s.execute("swatch.library.add", &json!({"library": r["library"], "apply": "fill"})));
    })
}

fn styles(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "graphicStyle.loadLibrary", data, |s, r| {
        let _ = no_panic(s.execute("select.all", &json!({})));
        let _ = no_panic(s.execute("graphicStyle.addFromLibrary", &json!({"library": r["library"], "apply": true})));
    })
}

/// A document with type and a character and a paragraph style, saved as pretty native text: what
/// Load All Styles reads.
fn saved_text_styles() -> String {
    static TEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TEXT.get_or_init(|| {
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width": 200, "height": 100})).unwrap();
        s.execute("text.create", &json!({"x": 10, "y": 40, "text": "Styled"})).unwrap();
        s.execute("charStyle.new", &json!({"name": "Lead", "attrs": {"size": 30, "tracking": 50, "underline": true, "leading": 36}})).unwrap();
        s.execute("paraStyle.new", &json!({"name": "Centered", "attrs": {"justify": "Center", "space_after": 6, "hyphenate": true}})).unwrap();
        let options = vectorcraft_format::SaveOptions { version: 2, pretty: true, ..vectorcraft_format::SaveOptions::default() };
        let text = String::from_utf8(vectorcraft_format::save_with(&s.doc().unwrap().doc, &options).unwrap()).unwrap();
        // Undamaged, it loads both styles.
        let r = rich_session().execute("textStyle.loadAll", &json!({"data": text, "name": "fuzz.vectorcraft"})).unwrap();
        assert_eq!((&r["charStyles"]["added"], &r["paraStyles"]["added"]), (&json!(["Lead"]), &json!(["Centered"])));
        text
    })
    .clone()
}

/// Native file text `data`'s styles loaded into the rich document by Load All Styles, and each
/// style it added applied to all of its type.
fn text_styles(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive(what, || {
        let mut s = rich_session();
        let r = no_panic(s.execute("textStyle.loadAll", &json!({"data": data, "name": "fuzz.vectorcraft"}))).ok()?;
        let _ = no_panic(s.execute("select.all", &json!({})));
        for (list, kind) in [("charStyles", "charStyle"), ("paraStyles", "paraStyle")] {
            for name in r[list]["added"].as_array().into_iter().flatten() {
                let _ = no_panic(s.execute(&format!("{kind}.apply"), &json!({"name": name})));
            }
        }
        Some((*s.doc().ok()?.doc).clone())
    })
}

/// Libraries panel library file `data` imported, then each kind's first item used on the rich
/// document, moved into its first group and next to its kind's last item, a colour and the
/// selection added into that group next to the first ones, the group moved and deleted and the
/// library exported (#926).
fn libraries(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "library.import", data, |s, r| {
        let lib = r["id"].clone();
        let Ok(got) = no_panic(s.execute("library.get", &json!({"library": lib}))) else { return };
        let _ = no_panic(s.execute("select.all", &json!({})));
        for (list, kind, key) in [
            ("graphics", "graphic", "id"),
            ("colors", "fillColor", "name"),
            ("gradients", "fillGradient", "name"),
            ("patterns", "fillPattern", "name"),
            ("charStyles", "charStyle", "name"),
            ("paraStyles", "paraStyle", "name"),
        ] {
            if let Some(item) = got[list].get(0) {
                let _ = no_panic(s.execute("library.use", &json!({"library": lib, "kind": kind, "item": item[key]})));
                let to = json!({"library": lib, "kind": kind, "item": item[key], "group": got["groups"][0]["name"]});
                let _ = no_panic(s.execute("library.moveItem", &to));
                // Next to the last item of its kind, whatever group that is in.
                let last = got[list].as_array().and_then(|a| a.last()).map(|v| v[key].clone()).unwrap_or_default();
                for side in ["before", "after"] {
                    let _ = no_panic(s.execute("library.moveItem", &json!({"library": lib, "kind": kind, "item": item[key], side: last})));
                }
            }
        }
        // Added where the panel drops it: into the first group, next to the first colour and graphic.
        let group = &got["groups"][0]["name"];
        let to = json!({"library": lib, "kind": "fillColor", "color": "#123456", "group": group, "after": got["colors"][0]["name"]});
        let _ = no_panic(s.execute("library.add", &to));
        let _ = no_panic(s.execute("library.add", &json!({"library": lib, "kind": "graphic", "group": group, "before": got["graphics"][0]["id"]})));
        if let Some(g) = got["groups"].get(0) {
            for to in [0, 1, 1_000_000] {
                let _ = no_panic(s.execute("library.moveGroup", &json!({"library": lib, "group": g["name"], "to": to})));
            }
            let _ = no_panic(s.execute("library.deleteGroup", &json!({"library": lib, "group": g["name"]})));
        }
        let _ = no_panic(s.execute("library.export", &json!({"library": lib})));
    })
}

/// A Libraries panel library of the rich document's first object (a graphic) and its gradient, a
/// colour, a pattern, its type's character and paragraph styles and a group, exported, made once.
fn saved_library() -> String {
    static TEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TEXT.get_or_init(|| {
        let mut s = rich_session();
        s.execute("library.create", &json!({"name": "Fuzz"})).unwrap();
        let first = s.doc().unwrap().doc.layers[0].children().unwrap()[0].id.0;
        let g = s.execute("library.add", &json!({"kind": "graphic", "ids": [first]})).unwrap()["id"].clone();
        s.execute("library.add", &json!({"kind": "fillColor", "color": "#336699"})).unwrap();
        s.execute("library.add", &json!({"kind": "fillGradient", "ids": [first]})).unwrap();
        s.execute("select.set", &json!({"ids": [first]})).unwrap();
        s.execute("object.pattern.make", &json!({"name": "Tiles", "width": 200, "height": 150})).unwrap();
        s.execute("object.pattern.done", &json!({})).unwrap();
        s.execute("library.add", &json!({"kind": "fillPattern", "swatch": "Tiles"})).unwrap();
        s.execute("select.all", &json!({})).unwrap();
        for kind in ["charStyle", "paraStyle"] {
            s.execute("library.add", &json!({"kind": kind})).unwrap();
        }
        s.execute(
            "library.createGroup",
            &json!({"name": "Logos", "items": [{"kind": "graphic", "item": g}, {"kind": "fillColor", "item": "#336699"}]}),
        )
        .unwrap();
        s.execute("library.export", &json!({})).unwrap()["data"].as_str().unwrap().to_string()
    })
    .clone()
}

fn flattener_presets(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "flattener.presets.import", data, |s, r| {
        let _ = no_panic(s.execute("select.all", &json!({})));
        if let Some(name) = r["imported"].get(0) {
            let _ = no_panic(s.execute("flattener.preview", &json!({"preset": name, "highlight": "allAffected"})));
            let _ = no_panic(s.execute("object.flattenTransparency", &json!({"preset": name, "lineArtPpi": 36, "gradientPpi": 36})));
        }
    })
}

fn export_presets(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "document.exportPresets.import", data, |s, r| {
        if let Some(name) = r["imported"].get(0) {
            let _ = no_panic(s.execute("assets.settings.set", &json!({"exportPreset": name})));
            let _ = no_panic(s.execute("document.exportPresets.export", &json!({"names": [name]})));
            let _ = no_panic(s.execute("document.exportForScreens", &json!({"exportPreset": name, "range": "1"})));
        }
    })
}

/// Swatch exchange (`.ase`) bytes loaded as a swatch library, then added to the rich document.
fn swatch_exchange(what: &str, bytes: &[u8]) -> Result<(), TestCaseError> {
    survive(what, || {
        let mut s = rich_session();
        let load = json!({"dataBase64": vectorcraft_format::base64_encode(bytes), "name": "fuzz.ase"});
        let r = no_panic(s.execute("swatch.library.load", &load)).ok()?;
        let _ = no_panic(s.execute("swatch.library.add", &json!({"library": r["library"], "apply": "fill"})));
        Some((*s.doc().ok()?.doc).clone())
    })
}

/// The rich document's swatches with a CMYK spot color and a global Lab color added, saved as a
/// swatch exchange file, made once.
fn saved_ase() -> Vec<u8> {
    static ASE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    ASE.get_or_init(|| {
        let mut s = rich_session();
        s.execute("swatch.new", &json!({"name": "Ink", "color": {"c": 1, "m": 0.5, "y": 0, "k": 0.2}, "spot": true})).unwrap();
        s.execute("swatch.new", &json!({"name": "Clay", "color": {"l": 50, "a": 20, "b": -30}, "global": true})).unwrap();
        let r = s.execute("swatch.library.save", &json!({"format": "ase"})).unwrap();
        vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap()
    })
    .clone()
}

/// A random swatch exchange block (type, body). The body holds a name, a color model when the block
/// is a color, and random bytes for the rest.
fn arb_ase_block() -> impl Strategy<Value = (u16, Vec<u8>)> {
    let kind = prop::sample::select(vec![0xC001u16, 0xC002, 0x0001, 0x0042]);
    let model = prop::sample::select(vec![*b"RGB ", *b"CMYK", *b"LAB ", *b"Gray", *b"HSB "]);
    (kind, "[ -~]{0,8}", model, prop::collection::vec(any::<u8>(), 0..24)).prop_map(|(kind, name, model, rest)| {
        let units: Vec<u16> = name.encode_utf16().chain([0]).collect();
        let mut body = (units.len() as u16).to_be_bytes().to_vec();
        body.extend(units.iter().flat_map(|u| u.to_be_bytes()));
        if kind == 0x0001 {
            body.extend(model);
        }
        body.extend(rest);
        (kind, body)
    })
}

/// Characters that break JSON and GPL palettes.
fn arb_edit() -> impl Strategy<Value = (usize, char)> {
    (0usize..20_000, prop::sample::select(vec!['{', '}', '[', ']', '"', ':', ',', '-', '9', 'e', '.', ' ', '\n', '#', 'n', 'x', '\t']))
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn library_garbage_never_panics(s in ".{0,300}", head in prop::sample::select(vec!["", "GIMP Palette\n","{\"format\": \"vcswatches\", ", "{\"format\": \"vcstyles\", ", "{\"format\": \"vcflattener\", ", "{\"format\": \"vcexportpresets\", ", "{\"name\": \"L\", "])) {
        let data = format!("{head}{s}");
        swatches("swatch library garbage", &data)?;
        styles("style library garbage", &data)?;
        flattener_presets("flattener preset garbage", &data)?;
        export_presets("export preset garbage", &data)?;
        libraries("library garbage", &data)?;
    }

    #[test]
    fn mutated_swatch_libraries_never_panic(cut in 0usize..20_000, edits in prop::collection::vec(arb_edit(), 0..10), gpl in any::<bool>()) {
        let text = saved("swatch.library.save", json!({"format": if gpl { "gpl" } else { "vcswatches" }}));
        swatches("mutated swatch library", &mutate_text(&text, cut, &edits))?;
    }

    #[test]
    fn mutated_style_libraries_never_panic(cut in 0usize..20_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        let text = saved("graphicStyle.saveLibrary", json!({}));
        styles("mutated style library", &mutate_text(&text, cut, &edits))?;
    }

    #[test]
    fn mutated_text_style_files_never_panic(cut in 0usize..20_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        text_styles("mutated text styles", &mutate_text(&saved_text_styles(), cut, &edits))?;
    }

    #[test]
    fn mutated_flattener_presets_never_panic(cut in 0usize..5_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        let text = saved("flattener.presets.export", json!({"names": ["high", "medium", "low"]}));
        flattener_presets("mutated flattener presets", &mutate_text(&text, cut, &edits))?;
    }

    #[test]
    fn mutated_export_presets_never_panic(cut in 0usize..5_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        let text = saved("document.exportPresets.export", json!({"names": ["Logo Package", "Print-Ready PDF"]}));
        export_presets("mutated export presets", &mutate_text(&text, cut, &edits))?;
    }

    #[test]
    fn mutated_libraries_never_panic(keep in 0.0f64..1.2, edits in prop::collection::vec(arb_edit(), 0..10)) {
        // Cut anywhere in the file (whole most of the time), the edits anywhere in it.
        let text = saved_library();
        let cut = (text.chars().count() as f64 * keep) as usize;
        libraries("mutated library", &mutate_text(&text, cut, &edits))?;
    }

    #[test]
    fn swatch_exchange_garbage_never_panics(blocks in prop::collection::vec(arb_ase_block(), 0..12), tail in prop::collection::vec(any::<u8>(), 0..16)) {
        let mut bytes = b"ASEF\0\x01\0\0".to_vec();
        bytes.extend((blocks.len() as u32).to_be_bytes());
        for (kind, body) in &blocks {
            bytes.extend(kind.to_be_bytes());
            bytes.extend((body.len() as u32).to_be_bytes());
            bytes.extend(body);
        }
        bytes.extend(tail);
        swatch_exchange("swatch exchange garbage", &bytes)?;
    }

    #[test]
    fn mutated_swatch_exchange_files_never_panic(cut in prop::option::of(0usize..200), edits in prop::collection::vec((0usize..200, any::<u8>()), 0..12)) {
        let mut bytes = vectorcraft_testkit::ase::sample();
        for (at, b) in edits {
            let n = bytes.len();
            bytes[at % n] = b;
        }
        if let Some(cut) = cut {
            bytes.truncate(cut);
        }
        swatch_exchange("mutated swatch exchange file", &bytes)?;
    }

    #[test]
    fn mutated_saved_swatch_exchange_files_never_panic(cut in prop::option::of(0usize..4_000), edits in prop::collection::vec((0usize..4_000, any::<u8>()), 0..12)) {
        let mut bytes = saved_ase();
        for (at, b) in edits {
            let n = bytes.len();
            bytes[at % n] = b;
        }
        if let Some(cut) = cut {
            bytes.truncate(cut);
        }
        swatch_exchange("mutated saved swatch exchange file", &bytes)?;
    }
}

// ---------- data set CSV files ----------

/// A document with a text, a visibility and a graph variable, each bound to an object, and two
/// data sets.
fn merge_session() -> vectorcraft_engine::Session {
    let mut s = vectorcraft_engine::Session::new();
    s.execute("file.new", &json!({"width": 200, "height": 200})).unwrap();
    let id = |r: Value| r["id"].as_u64().unwrap();
    let t = id(s.execute("text.create", &json!({"x": 10, "y": 10, "text": "One"})).unwrap());
    let r = id(s.execute("shape.rectangle", &json!({"x": 10, "y": 40, "width": 50, "height": 40})).unwrap());
    let graph = json!({"type": "column", "x": 80, "y": 10, "width": 100, "height": 80, "csv": ",Sales\nA,1\nB,2"});
    let g = id(s.execute("graph.create", &graph).unwrap());
    for (name, kind, object) in [("Title", "text", t), ("Show", "visibility", r), ("Chart", "graph", g)] {
        s.execute("variable.define", &json!({"name": name, "kind": kind})).unwrap();
        s.execute("variable.bind", &json!({"variable": name, "ids": [object]})).unwrap();
    }
    s.execute("dataset.new", &json!({"name": "A", "values": {"Title": "Two, \"quoted\"\nlines", "Show": true, "Chart": ",Q\nX,3"}})).unwrap();
    s.execute("dataset.new", &json!({"name": "B", "values": {"Title": " padded ", "Show": false}})).unwrap();
    s
}

/// [`merge_session`]'s data sets exported as CSV, made once.
fn saved_datasets() -> String {
    static TEXT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TEXT.get_or_init(|| merge_session().execute("dataset.export", &json!({})).unwrap()["csv"].as_str().unwrap().to_string()).clone()
}

/// `dataset.import` with `params` on [`merge_session`]'s document, the first three data sets it
/// read selected in turn and every data set exported again; the document then renders and exports.
fn datasets(what: &str, params: Value) -> Result<(), TestCaseError> {
    survive(what, || {
        let mut s = merge_session();
        if let Ok(r) = no_panic(s.execute("dataset.import", &params)) {
            for name in r["names"].as_array().into_iter().flatten().take(3) {
                let _ = no_panic(s.execute("dataset.select", &json!({"name": name})));
            }
        }
        let _ = no_panic(s.execute("dataset.export", &json!({})));
        Some((*s.doc().ok()?.doc).clone())
    })
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn dataset_csv_garbage_never_panics(s in ".{0,300}", head in prop::sample::select(vec!["", "name,Title,Show,Chart\n", "Title\tShow\tChart\n", "name\tTitle\n", "Show,Chart\n", "name,#Show,@Photo,New\n", "#Shown\t@Title\tChart\n", "name,%Chart,#\n"])) {
        datasets("data set CSV garbage", json!({"csv": format!("{head}{s}")}))?;
    }

    #[test]
    fn prefixed_dataset_headers_never_panic(
        cells in prop::collection::vec((prop::sample::select(vec!["", "#", "@", "%", "#@", " # "]), "[A-Za-z #@%\"]{0,8}"), 1..8),
        tabs in any::<bool>(),
        body in ".{0,200}",
    ) {
        let sep = if tabs { "\t" } else { "," };
        let header: Vec<String> = cells.iter().map(|(prefix, name)| format!("{prefix}{name}")).collect();
        datasets("prefixed data set CSV header", json!({"csv": format!("{}\n{body}", header.join(sep))}))?;
    }

    #[test]
    fn dataset_csv_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..300)) {
        datasets("data set CSV bytes", json!({"dataBase64": vectorcraft_format::base64_encode(&bytes)}))?;
    }

    #[test]
    fn mutated_dataset_csvs_never_panic(cut in 0usize..200, edits in prop::collection::vec(arb_edit(), 0..10), tabs in any::<bool>()) {
        let text = saved_datasets();
        let text = if tabs { text.replace(',', "\t") } else { text };
        datasets("mutated data set CSV", json!({"csv": mutate_text(&text, cut, &edits)}))?;
    }
}

// ---------- variable library XML files ----------

/// A variable library written from its public grammar: two text variables (`Title` is a
/// variable of [`merge_session`]'s document), a variable of another kind and two data sets.
const VARIABLE_LIBRARY: &str = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<!DOCTYPE svg [\n<!ENTITY ns_vars \"http://ns.adobe.com/Variables/1.0/\">\n<!ENTITY ns_custom \"http://ns.adobe.com/GenericCustomNamespace/1.0/\">\n]>\n\
<svg>\n<variableSets xmlns=\"&ns_vars;\">\n<variableSet varSetName=\"binding1\" locked=\"none\">\n<variables>\n\
<variable varName=\"Title\" trait=\"textcontent\"></variable>\n<variable varName=\"Note\" trait=\"textcontent\"></variable>\n\
<variable varName=\"Show\" trait=\"other\"></variable>\n</variables>\n\
<v:sampleDataSets xmlns:v=\"&ns_vars;\" xmlns=\"&ns_custom;\">\n\
<v:sampleDataSet dataSetName=\"A\">\n<Title><p>One</p><p>Two &amp; three</p></Title>\n<Note><p>x</p></Note>\n<Show><p>true</p></Show>\n</v:sampleDataSet>\n\
<v:sampleDataSet dataSetName=\"C\">\n<Title><p>Four</p></Title>\n<Note></Note>\n</v:sampleDataSet>\n\
</v:sampleDataSets>\n</variableSet>\n</variableSets>\n</svg>\n";

/// `variable.loadLibrary` with `params` on [`merge_session`]'s document, the first three data sets
/// it read selected in turn and every data set exported; the document then renders and exports.
fn variable_library(what: &str, params: Value) -> Result<(), TestCaseError> {
    survive(what, || {
        let mut s = merge_session();
        if let Ok(r) = no_panic(s.execute("variable.loadLibrary", &params)) {
            for name in r["names"].as_array().into_iter().flatten().take(3) {
                let _ = no_panic(s.execute("dataset.select", &json!({"name": name})));
            }
        }
        let _ = no_panic(s.execute("dataset.export", &json!({})));
        Some((*s.doc().ok()?.doc).clone())
    })
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn variable_library_garbage_never_panics(s in ".{0,300}", head in prop::sample::select(vec![
        "",
        "<variableSets>",
        "<variableSets><variableSet><variables><variable varName=\"Title\" trait=\"textcontent\"/></variables><sampleDataSets><sampleDataSet dataSetName=\"A\">",
        "<variableSets><variableSet><variables><variable varName=\"Title\" trait=\"textcontent\"/></variables><sampleDataSets><sampleDataSet dataSetName=\"A\"><Title><p>",
        "<!DOCTYPE svg [<!ENTITY a \"",
    ])) {
        variable_library("variable library garbage", json!({"xml": format!("{head}{s}")}))?;
    }

    #[test]
    fn variable_library_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..300)) {
        variable_library("variable library bytes", json!({"dataBase64": vectorcraft_format::base64_encode(&bytes)}))?;
    }

    /// The library cut short and with characters replaced, then with `nest` elements nested in one
    /// another inside its `svg` element.
    #[test]
    fn mutated_variable_libraries_never_panic(
        cut in 0usize..1_200,
        edits in prop::collection::vec((0usize..1_200, prop::sample::select(vec!['<', '>', '"', '/', '=', '&', ';', '!', '[', ']', 'p', 'x', ' '])), 0..10),
        nest in prop::sample::select(vec![0usize, 0, 0, 240, 100_000]),
    ) {
        let xml = mutate_text(VARIABLE_LIBRARY, cut, &edits);
        let xml = xml.replacen("<svg>", &format!("<svg>{}", "<a>".repeat(nest)), 1).replacen("</svg>", &format!("{}</svg>", "</a>".repeat(nest)), 1);
        variable_library("mutated variable library", json!({"xml": xml}))?;
    }
}

// ---------- shortcut sets ----------

/// Shortcut set `bytes` read with `shortcutSet.read`, with import matches saved to annotate it: an
/// error, or a set within the caps.
fn shortcut_set(bytes: &[u8]) -> Result<(), TestCaseError> {
    let mut s = vectorcraft_engine::Session::new();
    let matches = json!([
        {"kind": "name", "name": "A", "target": "edit.undo"},
        {"kind": "key", "name": "1", "target": "K"},
        {"kind": "modifiers", "name": "-1", "target": "Cmd"},
    ]);
    s.execute("importMatches.set", &json!({ "matches": matches })).unwrap();
    let read = json!({"dataBase64": vectorcraft_format::base64_encode(bytes), "name": "fuzz.kys"});
    if let Ok(r) = no_panic(s.execute("shortcutSet.read", &read)) {
        prop_assert!(r["entries"].as_array().map_or(0, Vec::len) <= 20_000);
        prop_assert!(r["unread"].as_array().map_or(0, Vec::len) <= 1000);
    }
    Ok(())
}

/// A token of a text shortcut set: a name (escaped spaces), a field, an integer of any size and sign,
/// a brace, a lone backslash, a line end, NUL or bytes that aren't UTF-8.
fn arb_kys_token() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        "[A-Za-z ]{0,12}".prop_map(|n| format!("/{}", n.replace(' ', "\\ ")).into_bytes()),
        prop::sample::select(vec!["/Key", "/Modifiers", "/Context", "/Represent"]).prop_map(|f| f.as_bytes().to_vec()),
        any::<i64>().prop_map(|n| n.to_string().into_bytes()),
        Just(b"99999999999999999999999".to_vec()),
        prop::sample::select(vec!["{", "}", "\\", "\r\n", "\r", "\n"]).prop_map(|t| t.as_bytes().to_vec()),
        prop::sample::select(vec![vec![0u8], vec![0xFF, 0xFE], vec![0xC3]]),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn shortcut_set_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..600), head in any::<bool>()) {
        let mut data = if head { b"/Menus {".to_vec() } else { vec![] };
        data.extend(bytes);
        shortcut_set(&data)?;
    }

    #[test]
    fn hostile_shortcut_sets_never_panic(tokens in prop::collection::vec(arb_kys_token(), 0..200), sep in prop::sample::select(vec![" ", "", "\t"])) {
        let mut data = b"/Menus {".to_vec();
        for t in tokens {
            data.extend(sep.as_bytes());
            data.extend(t);
        }
        shortcut_set(&data)?;
    }
}

/// A shortcut set past any cap (nesting, entries, name length, file size) is read up to it and
/// marked truncated.
#[test]
fn shortcut_sets_past_the_caps_are_truncated() {
    let read = |text: String| vectorcraft_engine::Session::new().execute("shortcutSet.read", &json!({ "data": text })).unwrap();
    let deep = format!("/Menus {{ {} /K {{ /Key 1 }} }}", "/A { ".repeat(40));
    let many = format!("/Menus {{ {} }}", "/E { /Key 1 } ".repeat(20_001));
    let long = format!("/Menus {{ /{} {{ /Key 1 }} }}", "n".repeat(2000));
    let big = format!("/Menus {{ /E {{ /Key 1 }} {} }}", " ".repeat(4 << 20));
    for text in [deep, many, long, big] {
        assert_eq!(read(text)["truncated"], true);
    }
}

// ---------- action sets ----------

/// Action set `bytes` read with `actionSet.read`: an error, or a set within the caps.
fn action_set(bytes: &[u8]) -> Result<(), TestCaseError> {
    let read = json!({"dataBase64": vectorcraft_format::base64_encode(bytes), "name": "fuzz.aia"});
    if let Ok(r) = no_panic(vectorcraft_engine::Session::new().execute("actionSet.read", &read)) {
        let actions = r["actions"].as_array().cloned().unwrap_or_default();
        prop_assert!(actions.len() <= 1000);
        prop_assert!(actions.iter().all(|a| a["steps"].as_array().map_or(0, Vec::len) <= 10_000));
        prop_assert!(actions.iter().map(|a| a["steps"].as_array().map_or(0, Vec::len)).sum::<usize>() <= 100_000);
        prop_assert!(r["unread"].as_array().map_or(0, Vec::len) <= 1000);
        let steps = || actions.iter().flat_map(|a| a["steps"].as_array().cloned().unwrap_or_default());
        prop_assert!(steps().all(|s| s["values"].as_array().map_or(0, Vec::len) as u64 <= s["params"].as_u64().unwrap_or(0)));
        prop_assert!(steps().map(|s| s["values"].as_array().map_or(0, Vec::len)).sum::<usize>() <= 100_000);
    }
    Ok(())
}

/// A token of a text action set: a block or field name, a number of any size and sign, text in
/// parentheses (unclosed, nested or escaped too), bracketed or angled hexadecimal, a brace, a line
/// end, NUL or bytes that aren't UTF-8.
fn arb_aia_token() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        prop::sample::select(vec![
            "/action-1",
            "/event-2",
            "/parameter-1",
            "/name",
            "/internalName",
            "/localizedName",
            "/isOn",
            "/value",
            "/key",
            "/type",
            "/x",
            "/version",
        ])
        .prop_map(|t| t.as_bytes().to_vec()),
        any::<i64>().prop_map(|n| n.to_string().into_bytes()),
        Just(b"99999999999999999999999".to_vec()),
        "[a-z ()\\\\]{0,12}".prop_map(|t| format!("({t})").into_bytes()),
        prop::sample::select(vec!["(", ")", "\\", "[ 2 4142 ]", "[ 3 41 4", "[ 1 zz ]", "[", "]", "< 2 0a0b >", "<", ">", "2.5"])
            .prop_map(|t| t.as_bytes().to_vec()),
        prop::sample::select(vec!["[ 2 c328 ]", "(ustring)", "(boolean)"]).prop_map(|t| t.as_bytes().to_vec()),
        prop::sample::select(vec!["{", "}", "\r\n", "\r", "\n"]).prop_map(|t| t.as_bytes().to_vec()),
        prop::sample::select(vec![vec![0u8], vec![0xFF, 0xFE], vec![0xC3]]),
    ]
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn action_set_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..600), head in any::<bool>()) {
        let mut data = if head { b"/version 3 ".to_vec() } else { vec![] };
        data.extend(bytes);
        action_set(&data)?;
    }

    #[test]
    fn hostile_action_sets_never_panic(tokens in prop::collection::vec(arb_aia_token(), 0..200), sep in prop::sample::select(vec![" ", "", "\t"])) {
        let mut data = b"/version 3".to_vec();
        for t in tokens {
            data.extend(sep.as_bytes());
            data.extend(t);
        }
        action_set(&data)?;
    }
}

/// An action set past any cap (nesting, actions, steps, name length, file size) is read up to it
/// and marked truncated.
#[test]
fn action_sets_past_the_caps_are_truncated() {
    let read = |text: String| vectorcraft_engine::Session::new().execute("actionSet.read", &json!({ "data": text })).unwrap();
    let deep = format!("/version 3 /action-1 {{ {} }}", "/x { ".repeat(40));
    let actions = format!("/version 3 {}", "/action-1 { } ".repeat(1001));
    let steps = format!("/version 3 /action-1 {{ {} }}", "/event-1 { } ".repeat(10_001));
    let action = format!("/action-1 {{ {} }} ", "/event-1 { } ".repeat(10_000));
    let all_steps = format!("/version 3 {}", action.repeat(11));
    let long = format!("/version 3 /name [ 2000 {} ]", "6e".repeat(2000));
    let big = format!("/version 3 /action-1 {{ }} {}", " ".repeat(8 << 20));
    let uncapped: Vec<&str> = [("deep", deep), ("actions", actions), ("steps", steps), ("long", long), ("big", big)]
        .into_iter()
        .filter(|(_, text)| read(text.clone())["truncated"] != true)
        .map(|(name, _)| name)
        .collect();
    assert!(uncapped.is_empty(), "{uncapped:?}");
    let r = read(all_steps);
    assert_eq!(r["truncated"], true);
    assert_eq!(r["actions"].as_array().unwrap().iter().map(|a| a["steps"].as_array().unwrap().len()).sum::<usize>(), 100_000);
}

/// A small image of each raster format Place reads, with resolution metadata where it has some.
fn raster_samples() -> Vec<(&'static str, Vec<u8>)> {
    let img = image::RgbImage::from_pixel(5, 3, image::Rgb([200, 40, 10]));
    let mut out = vec![];
    for (name, f) in [
        ("a.png", craft_image_codecs::Format::Png),
        ("a.jpg", craft_image_codecs::Format::Jpeg),
        ("a.tif", craft_image_codecs::Format::Tiff),
        ("a.bmp", craft_image_codecs::Format::Bmp),
    ] {
        let mut b = vec![];
        {
            let __dyn: image::DynamicImage = img.clone().into();
            let (__w, __h) = (__dyn.width(), __dyn.height());
            let (__layout, __sample, __raw) = match __dyn {
                image::DynamicImage::ImageLuma8(b) => (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8, b.into_raw()),
                image::DynamicImage::ImageLumaA8(b) => (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8, b.into_raw()),
                image::DynamicImage::ImageRgb8(b) => (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8, b.into_raw()),
                image::DynamicImage::ImageRgba8(b) => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, b.into_raw()),
                image::DynamicImage::ImageLuma16(b) => (
                    craft_image_codecs::ChannelLayout::Gray,
                    craft_image_codecs::SampleType::U16,
                    b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
                ),
                image::DynamicImage::ImageLumaA16(b) => (
                    craft_image_codecs::ChannelLayout::GrayA,
                    craft_image_codecs::SampleType::U16,
                    b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
                ),
                image::DynamicImage::ImageRgb16(b) => (
                    craft_image_codecs::ChannelLayout::Rgb,
                    craft_image_codecs::SampleType::U16,
                    b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
                ),
                image::DynamicImage::ImageRgba16(b) => (
                    craft_image_codecs::ChannelLayout::Rgba,
                    craft_image_codecs::SampleType::U16,
                    b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
                ),
                other => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, other.into_rgba8().into_raw()),
            };
            let __cimg = craft_image_codecs::Image::from_raw(__w, __h, __layout, __sample, __raw).unwrap();
            craft_image_codecs::encode(&__cimg, f, &craft_image_codecs::EncodeOptions::default()).map(|__b| b.extend_from_slice(&__b))
        }
        .unwrap();
        out.push((name, b));
    }
    out[0].1 = vectorcraft_engine::cmd::fileio::ppi::with_png_resolution(&out[0].1, (300.0, 150.0));
    // TIFFs that store one plane per channel: CMYK with a channel after the inks, and gray with
    // alpha, 16 bits a sample.
    let plane = |v: u16| vec![v; 15];
    let cmyk = [plane(200), plane(40), plane(10), plane(0), plane(90)];
    let cmyk =
        vectorcraft_testkit::tiff::Tiff { width: 5, height: 3, photometric: 5, bits: 8, extra: &[0], planar: true, ppi: Some(300), channels: &cmyk };
    out.push(("planar-cmyk.tif", vectorcraft_testkit::tiff::tiff(&cmyk)));
    let gray = [plane(30_000), plane(65_535)];
    let gray =
        vectorcraft_testkit::tiff::Tiff { width: 5, height: 3, photometric: 1, bits: 16, extra: &[2], planar: true, ppi: None, channels: &gray };
    out.push(("planar-gray.tif", vectorcraft_testkit::tiff::tiff(&gray)));
    // A layered PSD with transparency (as VectorCraft exports it), and a PackBits PSB by hand.
    let mut s = rich_session();
    let r = s.execute("document.export", &json!({"format": "psd", "ppi": 2})).unwrap();
    out.push(("a.psd", vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap()));
    let mut psb = b"8BPS\0\x02\0\0\0\0\0\0\0\x04\0\0\0\x02\0\0\0\x03\0\x08\0\x03".to_vec();
    psb.extend([0; 8]);
    psb.extend(10u64.to_be_bytes());
    psb.extend(2u64.to_be_bytes());
    psb.extend((-1i16).to_be_bytes());
    psb.extend([0, 1]);
    psb.extend((0..8).flat_map(|_| 2u32.to_be_bytes()));
    psb.extend((0..8u8).flat_map(|v| [0xfe, v * 30]));
    out.push(("a.psb", psb));
    // JPEG XL: an 8-bit RGBA codestream, and a 16-bit gray one in the container.
    use craft_jxl::{EncodeOptions, ImageRef, Layout, Samples};
    let opts = EncodeOptions { lossless: true, ..EncodeOptions::default() };
    let rgba: Vec<u8> = (0..60).map(|i| (i * 17) as u8).collect();
    out.push((
        "a.jxl",
        craft_jxl::encode(
            &ImageRef { width: 5, height: 3, layout: Layout::Rgba, samples: Samples::U8(&rgba) },
            opts,
            &craft_jxl::Metadata::default(),
        )
        .unwrap(),
    ));
    let gray_u16: Vec<u16> = (0..15).map(|i| u16::from_ne_bytes([(i * 2 * 41) as u8, ((i * 2 + 1) * 41) as u8])).collect();
    let gray = craft_jxl::encode(
        &ImageRef { width: 5, height: 3, layout: Layout::Gray, samples: Samples::U16(&gray_u16) },
        opts,
        &craft_jxl::Metadata::default(),
    )
    .unwrap();
    let mut boxed = vec![0, 0, 0, 0x0c, b'J', b'X', b'L', b' ', 0x0d, 0x0a, 0x87, 0x0a];
    boxed.extend(20u32.to_be_bytes());
    boxed.extend(b"ftypjxl \0\0\0\0jxl ");
    boxed.extend((8 + gray.len() as u32).to_be_bytes());
    boxed.extend(b"jxlc");
    boxed.extend(gray);
    out.push(("b.jxl", boxed));
    out
}

proptest! {
    #![proptest_config(config())]

    /// Mutated image headers (resolution metadata, chunk and segment lengths) never crash reading
    /// their resolution or placing them.
    #[test]
    fn mutated_images_place_without_panics(which in 0usize..8, cut in 0usize..4000, edits in prop::collection::vec((0usize..120, any::<u8>()), 0..12)) {
        let (name, mut bytes) = raster_samples().swap_remove(which);
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at) {
                *x = b;
            }
        }
        bytes.truncate(cut.max(8));
        let r = catch_quiet(|| {
            let _ = vectorcraft_engine::cmd::fileio::ppi::resolution(&bytes);
            let _ = vectorcraft_engine::cmd::fileio::ppi::with_png_resolution(&bytes, (72.0, 72.0));
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
            let p = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes), "thumbnail": 8});
            let _ = no_panic(s.execute("file.place.info", &p));
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "{name}: panicked: {:?}", r.err());
    }

    /// Mutated copies of the PSD and PSB samples never crash opening them with their layers, hidden
    /// ones too: the PSD's layer records, channel lengths and data, and the PSB's layer count.
    /// (Masks, clipping and groups are covered by `tests_psdlayers::damaged_layers_never_panic`.)
    /// Neither does `document.psdInfo`, nor placing them with their layers as objects.
    #[test]
    fn mutated_layered_psds_open_without_panics(which in 6usize..8, cut in 0usize..1_000_000, edits in prop::collection::vec((0usize..1_000_000, any::<u8>()), 0..12)) {
        let (name, mut bytes) = raster_samples().swap_remove(which);
        let len = bytes.len();
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at % len) {
                *x = b;
            }
        }
        bytes.truncate((cut % (len + 1)).max(8));
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            let p = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes), "hiddenLayers": true});
            let _ = no_panic(s.execute("document.open", &p));
            let _ = no_panic(s.execute("document.psdInfo", &p));
            let _ = no_panic(s.execute("file.new", &json!({})));
            let p = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes), "layers": true, "hiddenLayers": true});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "{name}: panicked: {:?}", r.err());
    }
}

/// A 4 × 2 pixel RGB PSD file whose clipping path resource (2999) names its saved path "P"
/// (resource 2000): a closed subpath of a linked and two unlinked knots, an open subpath, and the
/// initial fill record saying the fill starts with all pixels.
fn clipped_psd() -> Vec<u8> {
    let record = |selector: u16, body: &[i32]| {
        let mut r = selector.to_be_bytes().to_vec();
        r.extend(body.iter().flat_map(|v| v.to_be_bytes()));
        r.resize(26, 0);
        r
    };
    let one = 1 << 24;
    let mut path = record(6, &[]);
    path.extend(record(8, &[1 << 16]));
    path.extend(record(0, &[3 << 16]));
    path.extend(record(1, &[0, 0, 0, one / 2, 0, one]));
    path.extend(record(2, &[one, one, one, one, one, one]));
    path.extend(record(2, &[one, 0, one, 0, one, 0]));
    path.extend(record(3, &[2 << 16]));
    path.extend(record(5, &[-one, -one, 0, 0, 2 * one, 2 * one]));
    path.extend(record(4, &[one / 4, one / 4, one / 4, one / 4, one / 4, one / 4]));
    let mut res = b"8BIM\x07\xd0\x01P".to_vec();
    res.extend((path.len() as u32).to_be_bytes());
    res.extend(path);
    res.extend(b"8BIM\x0b\xb7\0\0");
    res.extend(8u32.to_be_bytes());
    res.extend([1, b'P', 0, 0, 0, 0, 0, 0]);
    let mut f = b"8BPS\0\x01\0\0\0\0\0\0\0\x03\0\0\0\x02\0\0\0\x04\0\x08\0\x03".to_vec();
    f.extend([0; 4]);
    f.extend((res.len() as u32).to_be_bytes());
    f.extend(res);
    f.extend([0; 4]);
    f.extend([0, 0]);
    f.extend((0..24u8).map(|v| v * 10));
    f
}

#[test]
fn the_clipped_psd_sample_places_in_a_clip_group() {
    let path = vectorcraft_engine::cmd::fileio::psdread::clipping_path(&clipped_psd()).unwrap();
    assert_eq!(path.subpaths.iter().map(|s| (s.anchors.len(), s.closed)).collect::<Vec<_>>(), [(4, true), (3, true), (2, false)]);
    let mut s = vectorcraft_engine::Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    let r = s.execute("file.place", &json!({"name": "c.psd", "dataBase64": vectorcraft_format::base64_encode(&clipped_psd())})).unwrap();
    assert!(r["image"].is_u64(), "{r}");
}

proptest! {
    #![proptest_config(config())]

    /// Mutated copies of a PSD file with a clipping path (its resources: the path records, their
    /// selectors, counts and points, and the names) never crash reading the path, placing the file
    /// or opening its merged image.
    #[test]
    fn mutated_psd_clipping_paths_never_panic(cut in 0usize..1000, edits in prop::collection::vec((0usize..300, any::<u8>()), 0..12)) {
        let mut bytes = clipped_psd();
        let len = bytes.len();
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at % len) {
                *x = b;
            }
        }
        bytes.truncate((cut % (len + 1)).max(8));
        let r = catch_quiet(|| {
            let _ = vectorcraft_engine::cmd::fileio::psdread::clipping_path(&bytes);
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
            let p = json!({"name": "c.psd", "dataBase64": vectorcraft_format::base64_encode(&bytes), "thumbnail": 8});
            let _ = no_panic(s.execute("file.place", &p));
            let _ = no_panic(s.execute("document.open", &json!({"name": "c.psd", "dataBase64": vectorcraft_format::base64_encode(&bytes), "layers": false})));
        });
        prop_assert!(r.is_ok(), "panicked: {:?}", r.err());
    }
}

// ---------- PSD slices ----------

/// A Unicode string as a PSD file stores it: a count of UTF-16 units, then the units.
fn psd_unicode(s: &str) -> Vec<u8> {
    let units: Vec<u16> = s.encode_utf16().collect();
    let mut b = (units.len() as u32).to_be_bytes().to_vec();
    b.extend(units.iter().flat_map(|u| u.to_be_bytes()));
    b
}

/// A descriptor key or class ID: length 0 and its four characters, or its length and its characters.
fn psd_key(k: &str) -> Vec<u8> {
    let mut b = if k.len() == 4 { 0u32 } else { k.len() as u32 }.to_be_bytes().to_vec();
    b.extend(k.as_bytes());
    b
}

/// A descriptor's class (an empty name and the ID `null`), then the count of its items.
fn psd_class(count: usize) -> Vec<u8> {
    let mut b = psd_unicode("");
    b.extend(psd_key("null"));
    b.extend((count as u32).to_be_bytes());
    b
}

/// A descriptor of `items`, each a key and a typed value.
fn psd_descriptor(items: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut b = psd_class(items.len());
    for (k, v) in items {
        b.extend(psd_key(k));
        b.extend(v);
    }
    b
}

/// A typed descriptor value: its four-character type, then `body`.
fn psd_value(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    [kind.as_slice(), body].concat()
}

fn psd_long(v: i32) -> Vec<u8> {
    psd_value(b"long", &v.to_be_bytes())
}

fn psd_text(t: &str) -> Vec<u8> {
    psd_value(b"TEXT", &psd_unicode(t))
}

fn psd_enum(kind: &str, v: &str) -> Vec<u8> {
    psd_value(b"enum", &[psd_key(kind), psd_key(v)].concat())
}

/// A bounds object: left, top, right and bottom (px), stored top, left, bottom, right.
fn psd_bounds([l, t, r, b]: [i32; 4]) -> Vec<u8> {
    psd_value(b"Objc", &psd_descriptor(&[("Top ", psd_long(t)), ("Left", psd_long(l)), ("Btom", psd_long(b)), ("Rght", psd_long(r))]))
}

/// The slices of the samples: origin (0: generated automatically, 1: follows a layer, 2: made by
/// the user), name, and left, top, right and bottom on the 12 × 10 px canvas. Two come in; the
/// automatic one between them is left out.
const PSD_SLICES: [(u32, &str, [i32; 4]); 3] = [(2, "a", [1, 2, 5, 6]), (0, "auto", [0, 0, 12, 1]), (1, "b", [6, 3, 11, 9])];

/// The data of an image resource 1050 holding [`PSD_SLICES`].
struct SliceSample {
    version: u32,
    data: Vec<u8>,
    /// Where the slice count starts in `data`.
    count_at: usize,
}

/// Version 6: the bounds of all the slices, the group's name, the count and one record per slice.
/// The first record is followed by a Photoshop 7 descriptor (version 16) with a nested object, which
/// the reader reads past to the second record.
fn psd_slices_v6() -> SliceSample {
    let mut b = 6u32.to_be_bytes().to_vec();
    b.extend([0i32, 0, 10, 12].iter().flat_map(|v| v.to_be_bytes()));
    b.extend(psd_unicode("group"));
    let count_at = b.len();
    b.extend((PSD_SLICES.len() as u32).to_be_bytes());
    for (i, (origin, name, rect)) in PSD_SLICES.iter().enumerate() {
        // The ID, the group ID and the origin; a slice that follows a layer gives the layer's ID.
        b.extend((i as u32 + 1).to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend(origin.to_be_bytes());
        if *origin == 1 {
            b.extend(4u32.to_be_bytes());
        }
        b.extend(psd_unicode(name));
        // An Image slice: its rectangle, link, target, message, alternative text, plain cell text,
        // alignment and background color (alpha, red, green, blue).
        b.extend(1u32.to_be_bytes());
        b.extend(rect.iter().flat_map(|v| v.to_be_bytes()));
        for t in ["https://example.com/", "_blank", "", "alt"] {
            b.extend(psd_unicode(t));
        }
        b.push(0);
        b.extend(psd_unicode(""));
        b.extend([0u8; 8]);
        b.extend([255, 10, 20, 30]);
        if i == 0 {
            b.extend(16u32.to_be_bytes());
            let color = psd_value(b"Objc", &psd_descriptor(&[("Rd  ", psd_value(b"doub", &128f64.to_be_bytes()))]));
            b.extend(psd_descriptor(&[("topOutset", psd_long(0)), ("bgColor", color)]));
        }
    }
    SliceSample { version: 6, data: b, count_at }
}

/// Versions 7 and 8: a version 16 descriptor with the slices' bounds, base name and `slices` list,
/// each slice an object with a nested bounds and background color object.
fn psd_slices_descriptor(version: u32) -> SliceSample {
    let mut b = version.to_be_bytes().to_vec();
    b.extend(16u32.to_be_bytes());
    b.extend(psd_class(3));
    b.extend(psd_key("bounds"));
    b.extend(psd_bounds([0, 0, 12, 10]));
    b.extend(psd_key("baseName"));
    b.extend(psd_text("sheet"));
    b.extend(psd_key("slices"));
    b.extend(b"VlLs");
    let count_at = b.len();
    b.extend((PSD_SLICES.len() as u32).to_be_bytes());
    for (i, (origin, name, rect)) in PSD_SLICES.iter().enumerate() {
        let origin = match origin {
            0 => "autoGenerated",
            1 => "layerGenerated",
            _ => "userGenerated",
        };
        let doub = |v: f64| psd_value(b"doub", &v.to_be_bytes());
        let color = psd_value(b"Objc", &psd_descriptor(&[("Rd  ", doub(10.0)), ("Grn ", doub(20.0)), ("Bl  ", doub(30.0))]));
        b.extend(psd_value(
            b"Objc",
            &psd_descriptor(&[
                ("sliceID", psd_long(i as i32 + 1)),
                ("groupID", psd_long(0)),
                ("origin", psd_enum("ESliceOrigin", origin)),
                ("Type", psd_enum("ESliceType", "Img ")),
                ("bounds", psd_bounds(*rect)),
                ("Nm  ", psd_text(name)),
                ("url", psd_text("https://example.com/")),
                ("bgColorType", psd_enum("ESliceBGColorType", "Clr ")),
                ("bgColor", color),
            ]),
        ));
    }
    SliceSample { version, data: b, count_at }
}

/// Resource 1050 in versions 6, 7 and 8.
fn psd_slice_samples() -> Vec<SliceSample> {
    vec![psd_slices_v6(), psd_slices_descriptor(7), psd_slices_descriptor(8)]
}

/// A flat 12 × 10 px RGB PSD file whose one image resource is 1050 holding `data`, and where
/// `data` lies in it.
fn sliced_psd(data: &[u8]) -> (Vec<u8>, std::ops::Range<usize>) {
    // The signature, version 1, reserved bytes, 3 channels, 10 rows, 12 columns, 8 bits, RGB, and
    // no color mode data.
    let mut f = b"8BPS\0\x01\0\0\0\0\0\0\0\x03".to_vec();
    f.extend(10u32.to_be_bytes());
    f.extend(12u32.to_be_bytes());
    f.extend([0, 8, 0, 3]);
    f.extend([0; 4]);
    let mut block = b"8BIM".to_vec();
    block.extend(1050u16.to_be_bytes());
    block.extend([0, 0]);
    block.extend((data.len() as u32).to_be_bytes());
    let start = f.len() + 4 + block.len();
    block.extend(data);
    block.resize(block.len().next_multiple_of(2), 0);
    f.extend((block.len() as u32).to_be_bytes());
    f.extend(block);
    // No layer and mask information, then the merged image, raw.
    f.extend([0; 4]);
    f.extend([0, 0]);
    f.extend((0..360u32).map(|v| (v * 7) as u8));
    (f, start..start + data.len())
}

/// PSD file `f` opened (the names of the document's slices, and the open's warnings), counted by
/// `document.psdInfo`, and placed embedded in a new document (the count of the slices it brings in).
fn sliced_psd_imports(f: &[u8]) -> (Vec<String>, Value, Value, usize) {
    let mut s = vectorcraft_engine::Session::new();
    let p = json!({"name": "sheet.psd", "dataBase64": vectorcraft_format::base64_encode(f)});
    let r = s.execute("document.open", &p).unwrap();
    let names = s.doc().unwrap().doc.slices.iter().map(|x| x.options.name.clone()).collect();
    let info = s.execute("document.psdInfo", &p).unwrap();
    s.execute("file.new", &json!({})).unwrap();
    let placed = s.execute("file.place", &json!({"name": "sheet.psd", "dataBase64": vectorcraft_format::base64_encode(f), "link": false})).unwrap();
    (names, r["warnings"].clone(), info["slices"].clone(), placed["slices"].as_array().map_or(0, Vec::len))
}

/// Each sample reaches the slice reader: its two user slices open and place, without a warning,
/// and `document.psdInfo` counts them; the automatic one is left out.
#[test]
fn the_sliced_psd_samples_bring_in_their_user_slices() {
    for sample in psd_slice_samples() {
        let (names, warnings, info, placed) = sliced_psd_imports(&sliced_psd(&sample.data).0);
        assert_eq!(names, ["a", "b"], "version {}: {warnings}", sample.version);
        assert_eq!(warnings.as_array().map_or(0, Vec::len), 0, "version {}: {warnings}", sample.version);
        assert_eq!((info, placed), (json!(2), 2), "version {}", sample.version);
    }
}

/// A slice count past the 10,000 read, or one larger than the slices stored, and resource data cut
/// short, open and place the file without its slices, with a warning, and `document.psdInfo`
/// counts none.
#[test]
fn sliced_psd_samples_with_huge_counts_or_cut_short_come_in_without_their_slices() {
    for sample in psd_slice_samples() {
        let mut damaged = vec![];
        for count in [10_001u32, 0x7fff_ffff, u32::MAX] {
            let mut d = sample.data.clone();
            d[sample.count_at..sample.count_at + 4].copy_from_slice(&count.to_be_bytes());
            damaged.push((format!("count {count}"), d));
        }
        damaged.push(("one byte short".into(), sample.data[..sample.data.len() - 1].to_vec()));
        damaged.push(("cut in the count".into(), sample.data[..sample.count_at + 2].to_vec()));
        for (what, d) in damaged {
            let (names, warnings, info, placed) = sliced_psd_imports(&sliced_psd(&d).0);
            let warned = warnings.as_array().is_some_and(|w| w.iter().any(|w| w.as_str().is_some_and(|w| w.contains("the slices can't be read"))));
            assert!(warned, "version {}, {what}: {warnings}", sample.version);
            assert_eq!((names.len(), info, placed), (0, json!(0), 0), "version {}, {what}", sample.version);
        }
    }
}

proptest! {
    #![proptest_config(config())]

    /// Mutated copies of the slice samples never crash opening the file, `document.psdInfo` or
    /// placing it embedded: the resource's data mutated, then kept whole, cut short with the
    /// resource's length saying so, or with the file cut short in the resource's block.
    #[test]
    fn mutated_psd_slices_never_panic(which in 0usize..3, mode in 0u8..3, cut in any::<usize>(), edits in prop::collection::vec((any::<usize>(), any::<u8>()), 0..12)) {
        let SliceSample { version, mut data, .. } = psd_slice_samples().swap_remove(which);
        let len = data.len();
        for (at, b) in edits {
            data[at % len] = b;
        }
        let bytes = match mode {
            0 => sliced_psd(&data).0,
            1 => {
                data.truncate(cut % (len + 1));
                sliced_psd(&data).0
            }
            _ => {
                let (mut f, at) = sliced_psd(&data);
                f.truncate(at.start - 12 + cut % (len + 13));
                f
            }
        };
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            let p = json!({"name": "sheet.psd", "dataBase64": vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("document.open", &p));
            let _ = no_panic(s.execute("document.psdInfo", &p));
            let _ = no_panic(s.execute("file.new", &json!({})));
            let p = json!({"name": "sheet.psd", "dataBase64": vectorcraft_format::base64_encode(&bytes), "link": false});
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "version {version}, mode {mode}: panicked: {:?}", r.err());
    }
}

/// 1-bit TIFFs compressed with CCITT Group 3: modified Huffman (Compression 2) in three strips, T.4
/// (Compression 3) with two-dimensional rows, fill bits, six EOLs at the end and each byte's lowest
/// bit first, and T.4 two-dimensional rows in 16 × 16 tiles.
fn group3_samples() -> Vec<Vec<u8>> {
    let rows: Vec<Vec<bool>> = (0..6).map(|y| (0..40).map(|x| (x * (y + 1) / 3 + y) % 4 < 2).collect()).collect();
    let tiff = |compression, t4_options, fill_order, rows_per_strip, tile| {
        let rtc = compression == 3;
        let g = vectorcraft_testkit::tiff::Group3 { rows: &rows, compression, t4_options, photometric: 0, fill_order, rows_per_strip, tile, rtc };
        vectorcraft_testkit::tiff::group3(&g)
    };
    vec![tiff(2, 0, 1, 2, None), tiff(3, 5, 2, 6, None), tiff(3, 1, 1, 6, Some((16, 16)))]
}

/// [`group3_samples`] place as images.
#[test]
fn the_group3_samples_place() {
    for bytes in group3_samples() {
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        s.execute("file.place", &json!({"name": "fax.tif", "dataBase64": vectorcraft_format::base64_encode(&bytes)})).unwrap();
    }
}

proptest! {
    #![proptest_config(config())]

    /// Mutated Group 3 TIFFs (directory values, strip offsets and counts, the coded rows) never
    /// crash reading or placing them.
    #[test]
    fn mutated_group3_tiffs_place_without_panics(which in 0usize..3, cut in 0usize..400, edits in prop::collection::vec((0usize..400, any::<u8>()), 0..12)) {
        let mut bytes = group3_samples().swap_remove(which);
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at) {
                *x = b;
            }
        }
        bytes.truncate(cut.max(8));
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
            let p = json!({"name": "fax.tif", "dataBase64": vectorcraft_format::base64_encode(&bytes), "thumbnail": 8});
            let _ = no_panic(s.execute("file.place.info", &p));
            let _ = no_panic(s.execute("file.place", &p));
        });
        prop_assert!(r.is_ok(), "panicked: {:?}", r.err());
    }
}

/// Targa files, which start with no signature: run-length encoded 24 bits (as the `image` crate
/// writes them), a 16-bit export with a one-bit alpha, and by hand a 16-bit run-length encoded file
/// stored right to left and top down, a run-length encoded color-mapped one with 16-bit entries,
/// and a 32-bit one with an extension area.
fn targa_samples() -> Vec<(&'static str, Vec<u8>)> {
    let mut rle = vec![];
    {
        let __dyn: image::DynamicImage = (image::RgbImage::from_pixel(5, 3, image::Rgb([200, 40, 10]))).into();
        let (__w, __h) = (__dyn.width(), __dyn.height());
        let (__layout, __sample, __raw) = match __dyn {
            image::DynamicImage::ImageLuma8(b) => (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageLumaA8(b) => (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageRgb8(b) => (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageRgba8(b) => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageLuma16(b) => (
                craft_image_codecs::ChannelLayout::Gray,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageLumaA16(b) => (
                craft_image_codecs::ChannelLayout::GrayA,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageRgb16(b) => (
                craft_image_codecs::ChannelLayout::Rgb,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageRgba16(b) => (
                craft_image_codecs::ChannelLayout::Rgba,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            other => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, other.into_rgba8().into_raw()),
        };
        let __cimg = craft_image_codecs::Image::from_raw(__w, __h, __layout, __sample, __raw).unwrap();
        craft_image_codecs::encode(&__cimg, craft_image_codecs::Format::Tga, &craft_image_codecs::EncodeOptions::default())
            .map(|__b| rle.extend_from_slice(&__b))
    }
    .unwrap();
    let mut s = rich_session();
    let r = s.execute("document.export", &json!({"format": "tga", "depth": 16, "ppi": 2})).unwrap();
    let export = vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap();
    let mut alpha = vec![0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 2, 0, 16, 0x31, 0x02, 0x00, 0xfc, 0x1f, 0x80, 0x00, 0x00];
    alpha.extend([0x82, 0xe0, 0x83]);
    let mut mapped = vec![0, 1, 9, 5, 0, 2, 0, 16, 0, 0, 0, 0, 3, 0, 2, 0, 8, 0x21, 0x00, 0x00, 0x00, 0xfc];
    mapped.extend([0x80, 6, 0x00, 5, 0x83, 6]);
    let mut extended = vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 32, 0x28, 50, 50, 50, 128];
    let mut area = vec![0; 495];
    area[..2].copy_from_slice(&495u16.to_le_bytes());
    area[494] = 4;
    extended.extend(area);
    extended.extend(22u32.to_le_bytes());
    extended.extend([0; 4]);
    extended.extend(b"TRUEVISION-XFILE.\0");
    vec![("a.tga", rle), ("b.tga", export), ("c.tga", alpha), ("d.vda", mapped), ("e.vst", extended)]
}

proptest! {
    #![proptest_config(config())]

    /// Mutated Targa files (headers, run-length packets, color maps, the extension area and the
    /// footer) never crash placing them.
    #[test]
    fn mutated_targa_files_place_without_panics(which in 0usize..5, cut in 0usize..600, edits in prop::collection::vec((0usize..560, any::<u8>()), 0..12)) {
        let (name, mut bytes) = targa_samples().swap_remove(which);
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at) {
                *x = b;
            }
        }
        bytes.truncate(cut.max(8));
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
            let p = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes), "thumbnail": 8});
            let _ = no_panic(s.execute("file.place.info", &p));
            let _ = no_panic(s.execute("file.place", &p));
            let _ = no_panic(s.execute("clipboard.importImage", &json!({"dataBase64": vectorcraft_format::base64_encode(&bytes)})));
        });
        prop_assert!(r.is_ok(), "{name}: panicked: {:?}", r.err());
    }
}

/// PCX files of each layout, 3 × 2 pixels at 150 dpi: 1 bit in two colors, 1 bit in four planes,
/// 2 bits with CGA palette settings, 4 bits with the header palette, 8 bits with the 256-color palette after the image data,
/// and 24 bits in three planes, with runs that cross a scan line.
fn pcx_samples() -> Vec<(&'static str, Vec<u8>)> {
    let pcx = |bits: u8, planes: u8, line: u16, data: &[u8]| {
        let mut f = vec![0x0a, 5, 1, bits, 0, 0, 0, 0, 2, 0, 1, 0, 150, 0, 150, 0];
        f.extend((0..48u8).map(|i| i.wrapping_mul(37)));
        f.extend([0, planes]);
        f.extend(line.to_le_bytes());
        f.extend(1u16.to_le_bytes());
        f.resize(128, 0);
        f.extend(data);
        f
    };
    let mut paletted = pcx(8, 1, 4, &[0xc5, 7, 0xc1, 0xff, 1, 2]);
    paletted.push(0x0c);
    paletted.extend((0..=255u8).flat_map(|i| [i, 255 - i, i / 2]));
    vec![
        ("a.pcx", pcx(1, 1, 2, &[0xa0, 0, 0x40, 0])),
        ("b.pcx", pcx(1, 4, 2, &[0xc8, 0xe0, 0x20, 0, 0x60, 0, 0xc4, 0x80])),
        ("c.pcx", pcx(2, 1, 2, &[0x1b, 0, 0xc2, 0xe4])),
        ("d.pcx", pcx(4, 1, 2, &[0xf1, 0x70, 0x23, 0x40])),
        ("e.pcx", paletted),
        ("f.pcx", pcx(8, 3, 4, &[0xc9, 200, 0xcf, 40])),
    ]
}

proptest! {
    #![proptest_config(config())]

    /// Mutated PCX files (header fields, run-length bytes and the palette after the image data)
    /// never crash placing, opening or pasting them.
    #[test]
    fn mutated_pcx_files_place_without_panics(which in 0usize..6, cut in 0usize..960, edits in prop::collection::vec((0usize..960, any::<u8>()), 0..12)) {
        let (name, mut bytes) = pcx_samples().swap_remove(which);
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at) {
                *x = b;
            }
        }
        bytes.truncate(cut.max(4));
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
            let p = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes), "thumbnail": 8});
            let _ = no_panic(s.execute("file.place.info", &p));
            let _ = no_panic(s.execute("file.place", &p));
            let _ = no_panic(s.execute("document.open", &json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes)})));
            let _ = no_panic(s.execute("clipboard.importImage", &json!({"dataBase64": vectorcraft_format::base64_encode(&bytes)})));
        });
        prop_assert!(r.is_ok(), "{name}: panicked: {:?}", r.err());
    }
}

/// Pixar picture files, 3 × 2 pixels: RGBA dumped in 8 bits, RGB dumped in 12 bits, RGBA encoded
/// with every packet kind and a block end, and gray in 2 × 1 tiles with a null tile.
fn pxr_samples() -> Vec<(&'static str, Vec<u8>)> {
    let pxr = |tile: (u16, u16), format: u16, storage: u16, tiles: &[Option<Vec<u8>>]| {
        let mut f = vec![0x80, 0xe8, 0, 0];
        f.resize(416, 0);
        for v in [2, 3, tile.1, tile.0, format, storage, 16, 1, 0, 0] {
            f.extend(u16::to_le_bytes(v));
        }
        f.resize(512, 0);
        let mut at = (512 + tiles.len() * 8).next_multiple_of(16);
        let mut data: Vec<u8> = vec![];
        for t in tiles {
            match t {
                Some(bytes) => {
                    f.extend((at as u32).to_le_bytes());
                    f.extend((bytes.len() as u32).to_le_bytes());
                    data.extend(bytes);
                    at += bytes.len();
                }
                None => f.extend([0; 8]),
            }
        }
        f.resize((512 + tiles.len() * 8).next_multiple_of(16), 0);
        f.extend(data);
        f
    };
    let rgba: Vec<u8> = (0..24u8).map(|i| i.wrapping_mul(41)).collect();
    let wide: Vec<u8> = (0..18u16).flat_map(|i| (i * 300).to_le_bytes()).collect();
    let mut packets = vec![0x01, 0x10, 1, 2, 3, 4, 5, 6, 7, 8, 0x00, 0x20, 0, 9, 9, 9, 9];
    packets.extend([0x00, 0x00]);
    packets.resize(32, 0);
    packets.extend([0x00, 0x30, 200, 1, 2, 3, 0x01, 0x40, 100, 0, 4, 5, 6, 0, 7, 8, 9]);
    vec![
        ("a.pxr", pxr((3, 2), 15, 2, &[Some(rgba)])),
        ("b.pxr", pxr((3, 2), 14, 3, &[Some(wide)])),
        ("c.pxr", pxr((3, 2), 15, 0, &[Some(packets)])),
        ("d.pxr", pxr((2, 1), 8, 2, &[Some(vec![1, 2]), Some(vec![3, 4]), None, Some(vec![5, 6])])),
    ]
}

proptest! {
    #![proptest_config(config())]

    /// Mutated Pixar picture files (header fields, the tile pointer table, samples and packets)
    /// never crash placing, opening or pasting them.
    #[test]
    fn mutated_pxr_files_place_without_panics(which in 0usize..4, cut in 0usize..640, edits in prop::collection::vec((400usize..640, any::<u8>()), 0..12)) {
        let (name, mut bytes) = pxr_samples().swap_remove(which);
        for (at, b) in edits {
            if let Some(x) = bytes.get_mut(at) {
                *x = b;
            }
        }
        bytes.truncate(cut.max(4));
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
            let p = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes), "thumbnail": 8});
            let _ = no_panic(s.execute("file.place.info", &p));
            let _ = no_panic(s.execute("file.place", &p));
            let _ = no_panic(s.execute("document.open", &json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes)})));
            let _ = no_panic(s.execute("clipboard.importImage", &json!({"dataBase64": vectorcraft_format::base64_encode(&bytes)})));
        });
        prop_assert!(r.is_ok(), "{name}: panicked: {:?}", r.err());
    }
}

// ---------- SVG saved with hidden layers and editing data ----------

proptest! {
    #![proptest_config(config())]

    /// Mutated JP2 boxes and raw codestream markers must return errors rather than unwind.
    #[test]
    fn mutated_jpeg2000_files_do_not_panic(raw in any::<bool>(), cut in 0usize..300, edits in prop::collection::vec((0usize..300, any::<u8>()), 0..8)) {
        let (name, mut bytes) = if raw {
            ("a.j2k", include_bytes!("fixtures/jpeg2000/checker-raw.j2k").to_vec())
        } else {
            ("a.jp2", include_bytes!("fixtures/jpeg2000/checker.jp2").to_vec())
        };
        for (at, byte) in edits {
            if let Some(slot) = bytes.get_mut(at) { *slot = byte; }
        }
        bytes.truncate(cut);
        let result = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            let params = json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("document.open", &params));
            let _ = no_panic(s.execute("file.place", &params));
        });
        prop_assert!(result.is_ok(), "{name}: panicked: {:?}", result.err());
    }
}

proptest! {
    #![proptest_config(config())]

    /// Mutated HEIF boxes and HEVC data must return errors rather than unwind.
    #[test]
    fn mutated_heif_files_do_not_panic(which in 0usize..4, cut in 0usize..1300, edits in prop::collection::vec((0usize..1300, any::<u8>()), 0..8)) {
        let mut bytes = match which {
            0 => include_bytes!("fixtures/heif/quadrants.heic").to_vec(),
            1 => include_bytes!("fixtures/heif/quadrants-alpha.heic").to_vec(),
            2 => include_bytes!("fixtures/heif/quadrants-grid.heic").to_vec(),
            // Its stream's VUI gives the color description.
            _ => include_bytes!("fixtures/heif/tones-p3-icc-only.heic").to_vec(),
        };
        for (at, byte) in edits {
            if let Some(slot) = bytes.get_mut(at) { *slot = byte; }
        }
        bytes.truncate(cut);
        let result = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            let params = json!({"name": "a.heic", "dataBase64": vectorcraft_format::base64_encode(&bytes)});
            let _ = no_panic(s.execute("document.open", &params));
            let _ = no_panic(s.execute("file.place", &params));
        });
        prop_assert!(result.is_ok(), "a.heic: panicked: {:?}", result.err());
    }
}

/// The rich document with a hidden layer, saved as SVG with its editing data.
fn saved_svg() -> String {
    let mut d = rich_doc();
    if let Some(l) = d.layers.first_mut() {
        std::sync::Arc::make_mut(l).visible = false;
    }
    let opts = vectorcraft_svg::ExportOptions { hidden_layers: true, preserve_editing: true, ..Default::default() };
    vectorcraft_svg::export_full(&d, &opts, Some(&vectorcraft_format::save(&d, false))).svg
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn svg_mutated_save_never_panics(
        cut in 0usize..40_000,
        edits in prop::collection::vec((0usize..40_000, prop::sample::select(vec!['<', '>', '"', '/', '-', '9', 'e', '.', ' ', '#', '%', '&', ';', 'x', ']', 'A'])), 0..10),
    ) {
        let svg = mutate_text(&saved_svg(), cut, &edits);
        survive("mutated saved svg", || vectorcraft_engine::cmd::fileio::load("saved.svg", svg.as_bytes()).ok().map(|l| l.doc))?;
    }
}

// ---------- PDF saved with editing data ----------

/// The rich document saved as a PDF-compatible .ai (the native document embedded, uncompressed).
fn saved_ai() -> Vec<u8> {
    static AI: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    AI.get_or_init(|| {
        let r = rich_session().execute("document.save", &json!({"format": "ai", "compression": {"compressText": false}})).unwrap();
        vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap()
    })
    .clone()
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn ai_mutated_save_never_panics(
        cut in 0usize..200_000,
        flips in prop::collection::vec((0usize..200_000, prop::sample::select(vec![b'0', b'9', b'-', b'.', b' ', b'[', b']', b'<', b'>', b'/', b'(', b'{', b'"', 0u8, 0xff])), 0..10),
    ) {
        let mut b = saved_ai();
        for (i, c) in flips {
            let n = b.len();
            b[i % n] = c;
        }
        b.truncate(cut.min(b.len()).max(9));
        survive("mutated ai", || vectorcraft_engine::cmd::fileio::load("saved.ai", &b).ok().map(|l| l.doc))?;
    }
}

// ---------- PDF presets files ----------

/// Import PDF presets from `data`, then export with each one imported (and save the .ai).
fn pdf_presets(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "pdf.preset.import", data, |s, r| {
        for name in r["imported"].as_array().into_iter().flatten() {
            let _ = no_panic(s.execute("document.exportPdf", &json!({"preset": name})));
            let _ = no_panic(s.execute("document.save", &json!({"format": "ai", "preset": name})));
        }
    })
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn pdf_presets_garbage_never_panics(s in ".{0,300}", head in prop::sample::select(vec!["", "{\"format\": \"vcpdfpresets\", ", "{\"format\": \"vcpdfpresets\", \"presets\": [{\"name\": \"x\", \"settings\": "])) {
        pdf_presets("pdf presets garbage", &format!("{head}{s}"))?;
    }

    #[test]
    fn mutated_pdf_presets_never_panic(cut in 0usize..20_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        let text = saved("pdf.preset.export", json!({"names": ["VectorCraft Default", "Smallest File Size", "PDF/X-4:2010"]}));
        pdf_presets("mutated pdf presets", &mutate_text(&text, cut, &edits))?;
    }
}

// ---------- what Paste takes from other apps ----------

/// Load `p` into the clipboard with `cmd`, then paste it into a new document: neither may panic.
fn paste_from_elsewhere(cmd: &str, p: Value) -> Result<(), TestCaseError> {
    let r = catch_quiet(|| {
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        if no_panic(s.execute(cmd, &p)).is_ok() {
            // Paste may refuse what it was given, e.g. a damaged PDF whose art spans more than the
            // canvas ("result would exceed the canvas"): an error is a fine outcome, a panic isn't.
            let _ = no_panic(s.execute("edit.paste", &json!({"center": [50, 50]})));
        }
    });
    prop_assert!(r.is_ok(), "{cmd}: panicked: {:?}", r.err());
    Ok(())
}

/// [`rich_pdf`], made once.
fn rich_pdf_once() -> Vec<u8> {
    static PDF: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    PDF.get_or_init(rich_pdf).clone()
}

/// `bytes` with `edits` (positions wrap around) cut to `cut` bytes, as base64.
fn mutated_b64(mut bytes: Vec<u8>, cut: usize, edits: &[(usize, u8)]) -> String {
    let n = bytes.len().max(1);
    for (at, b) in edits {
        if let Some(x) = bytes.get_mut(at % n) {
            *x = *b;
        }
    }
    bytes.truncate(cut.max(8));
    vectorcraft_format::base64_encode(&bytes)
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn pasted_bitmaps_never_panic(which in 0usize..4, cut in 0usize..400, edits in prop::collection::vec((0usize..120, any::<u8>()), 0..12)) {
        let (_, image) = raster_samples().swap_remove(which);
        paste_from_elsewhere("clipboard.importImage", json!({"dataBase64": mutated_b64(image, cut, &edits), "mime": "image/png"}))?;
    }

    #[test]
    fn pasted_pdf_never_panics(cut in 0usize..40_000, edits in prop::collection::vec((0usize..40_000, any::<u8>()), 0..12)) {
        paste_from_elsewhere("clipboard.importPdf", json!({"dataBase64": mutated_b64(rich_pdf_once(), cut, &edits)}))?;
    }

    // ASCII with control characters and line breaks, a mark and multi-byte characters (characters
    // no font has would send every layout to the slow font fallback).
    #[test]
    fn pasted_text_never_panics(text in r"[\x00-\x7f\u{300}\u{feff}\u{fffd}é€]{0,200}") {
        paste_from_elsewhere("clipboard.importText", json!({ "text": text }))?;
    }
}

// ---------- native files: compressed, and saved for older versions ----------

/// The rich document saved compressed, as v1 and as v2 (pretty).
fn native_samples() -> &'static [Vec<u8>] {
    use vectorcraft_format::{SaveOptions, save_with};
    static SAMPLES: std::sync::OnceLock<Vec<Vec<u8>>> = std::sync::OnceLock::new();
    SAMPLES.get_or_init(|| {
        let d = rich_doc();
        [
            SaveOptions { compress: true, ..SaveOptions::default() },
            SaveOptions { version: 1, ..SaveOptions::default() },
            SaveOptions { version: 2, pretty: true, ..SaveOptions::default() },
        ]
        .iter()
        .map(|o| save_with(&d, o).unwrap())
        .collect()
    })
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write as _;
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    gz.write_all(bytes).unwrap();
    gz.finish().unwrap()
}

proptest! {
    #![proptest_config(config())]

    /// Damaged native files of every kind: flipped bytes and cuts in the compressed stream, in v1
    /// and v2 text, and in the text inside an intact compressed stream.
    #[test]
    fn native_mutated_files_never_panic(which in 0usize..4, cut in 0usize..80_000, edits in prop::collection::vec((0usize..80_000, any::<u8>()), 0..10)) {
        let samples = native_samples();
        let mut bytes = samples.get(which).cloned().unwrap_or_else(|| {
            let mut f = vectorcraft_format::save(&rich_doc(), false);
            f.truncate(cut.max(16));
            f
        });
        for &(at, b) in &edits {
            let n = bytes.len();
            bytes[at % n] = b;
        }
        if which < samples.len() {
            bytes.truncate(cut.max(2));
        } else {
            bytes = gzip(&bytes);
        }
        let _ = vectorcraft_format::sniff(&bytes);
        let _ = vectorcraft_format::preview(&bytes);
        survive("mutated native file", || vectorcraft_engine::cmd::fileio::load("x.vectorcraft", &bytes).ok().map(|l| l.doc))?;
    }
}

/// A native file whose area type has 2^60 rows, 2^60 columns, or 2^32 of each (their product
/// overflows) opens with 100 in place of each count over 100, and renders and exports.
#[test]
fn native_area_type_with_2_to_the_60_rows_or_columns_opens_renders_and_exports() {
    let mut s = vectorcraft_engine::Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 300})).unwrap();
    let made = s.execute("text.create", &json!({"x": 10, "y": 10, "text": "word ".repeat(40), "area": {"width": 300, "height": 120}})).unwrap();
    let id = vectorcraft_doc::NodeId(made["id"].as_u64().unwrap());
    let mut d = (*s.doc().unwrap().doc).clone();
    for (rows, columns) in [(1 << 60, 1), (1, 1 << 60), (1 << 32, 1 << 32)] {
        let Some(vectorcraft_doc::NodeKind::Text(t)) = d.node_mut(id).map(|n| &mut n.kind) else { panic!("area type") };
        t.area.rows = rows;
        t.area.columns = columns;
        let file = vectorcraft_format::base64_encode(&vectorcraft_format::save(&d, false));
        s.execute("document.open", &json!({"name": "x.vectorcraft", "dataBase64": file})).unwrap();
        let opened = (*s.doc().unwrap().doc).clone();
        let Some(vectorcraft_doc::NodeKind::Text(t)) = opened.node(id).map(|n| &n.kind) else { panic!("area type") };
        assert_eq!((t.area.rows, t.area.columns), (rows.min(100), columns.min(100)));
        survive("area type in 2^60 rows or columns", || Some(opened)).unwrap();
    }
}

/// The rich document as EPS (with a preview, a thumbnail and the document it carries), and the
/// document read back from it.
fn eps_sample() -> Vec<u8> {
    let mut s = rich_session();
    let r = s.execute("document.export", &json!({"format": "eps", "useArtboards": false})).unwrap();
    vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap()
}

proptest! {
    #![proptest_config(config())]



    /// A damaged EPS another app copied: Paste reads it as an error or as objects.
    #[test]
    fn pasted_eps_never_panics(cut in 0usize..400_000, edits in prop::collection::vec((0usize..400_000, any::<u8>()), 0..12)) {
        static SAMPLE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        paste_from_elsewhere("clipboard.importEps", json!({"dataBase64": mutated_b64(SAMPLE.get_or_init(foreign_eps_sample).clone(), cut, &edits)}))?;
    }

    /// Hostile PostScript programs: operators in any order with any operands, opened and placed;
    /// as files in the legacy Illustrator format too (the creator line is what turns on its
    /// documented `u` … `U` groups), the groups unbalanced, deep and among clips; in a layer, which
    /// the editing data reader reads from the program too, or after a setup without a layer, which
    /// it reads with the art on one layer.
    #[test]
    fn eps_hostile_programs_never_panic(tokens in prop::collection::vec(arb_ps_token(), 0..60), illustrator in any::<bool>(), layers in any::<bool>()) {
        let head = if illustrator { "%%Creator: Adobe Illustrator(R) 8.0\n%%EndComments\n/u {} def /U {} def\n%%BeginSetup\n%%EndSetup" } else { "%%EndComments" };
        let (open, close) = if layers {
            ("/Lb {10 {pop} repeat} def /Ln {pop} def /LB {} def\n%AI5_BeginLayer\n1 1 1 1 0 0 -1 0 0 0 Lb\n(L) Ln\n", "\nLB\n%AI5_EndLayer--")
        } else {
            ("", "")
        };
        let ps = format!("%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 200 200\n{head}\n{open}{}{close}\nshowpage\n", tokens.join(" "));
        survive("hostile PostScript", || vectorcraft_engine::cmd::fileio::load("x.eps", ps.as_bytes()).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = rich_session();
            let _ = no_panic(s.execute("file.place", &json!({"name": "x.eps", "dataBase64": vectorcraft_format::base64_encode(ps.as_bytes())})));
        });
        r.map_err(|msg| TestCaseError::fail(format!("placing hostile PostScript panicked: {msg}")))?;
    }

    /// Hostile PostScript programs in `.ps` files whose header is only `%!`: opened, placed, and
    /// pasted as a copied file is.
    #[test]
    fn ps_hostile_programs_never_panic(tokens in prop::collection::vec(arb_ps_token(), 0..60)) {
        let ps = format!("%!\n{}\nshowpage\n", tokens.join(" "));
        survive("hostile .ps", || vectorcraft_engine::cmd::fileio::load("x.ps", ps.as_bytes()).ok().map(|l| l.doc))?;
        let r = catch_quiet(|| {
            let mut s = rich_session();
            let _ = no_panic(s.execute("file.place", &json!({"name": "x.ps", "dataBase64": vectorcraft_format::base64_encode(ps.as_bytes())})));
        });
        r.map_err(|msg| TestCaseError::fail(format!("placing a hostile .ps panicked: {msg}")))?;
        paste_from_elsewhere("clipboard.importEps", json!({"dataBase64": vectorcraft_format::base64_encode(ps.as_bytes())}))?;
    }







}








/// [`eps_sample`] as another app would write it: without the document it carries, so its
/// PostScript is read.
fn foreign_eps_sample() -> Vec<u8> {
    let mut bytes = eps_sample();
    let marker = b"%VectorCraft_BeginData: native";
    let at = bytes.windows(marker.len()).position(|w| w == marker).unwrap();
    bytes[at + marker.len() - 1] = b'x';
    bytes
}

/// The operators and operands of a hostile PostScript program.
fn arb_ps_token() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_num(),
        prop::sample::select(vec![
            "moveto", "lineto", "curveto", "rlineto", "rmoveto", "closepath", "fill", "eofill", "stroke", "clip", "eoclip", "newpath",
            "gsave", "grestore", "save", "restore", "concat", "translate", "scale", "rotate", "setlinewidth", "setdash", "setgray",
            "setrgbcolor", "setcmykcolor", "sethsbcolor", "setcolorspace", "setcolor", "setpattern", "makepattern", "shfill", "arc", "arcn",
            "arct", "arcto", "rectfill", "rectclip", "rectstroke", "image", "imagemask", "colorimage", "currentfile", "filter",
            "readhexstring", "readstring", "readline", "show", "stringwidth", "charpath", "findfont", "scalefont", "makefont", "setfont", "selectfont",
            "def", "bind", "load", "dup", "exch", "pop", "roll", "index", "copy", "for", "repeat", "loop", "exit", "if", "ifelse", "exec",
            "stopped", "begin", "end", "dict", "array", "string", "get", "put", "getinterval", "putinterval", "aload", "astore", "forall",
            "[", "]", "<<", ">>", "{", "}", "matrix", "currentmatrix", "setmatrix", "transform", "itransform", "invertmatrix", "pathbbox",
            "clippath", "initclip", "currentpoint", "eexec", "cleartomark", "mark", "counttomark", "showpage", "gstate", "setgstate",
            "clipsave", "cliprestore", "languagelevel", "cvx", "findresource", "defineresource", "undefineresource", "resourcestatus",
            "resourceforall", "/Category", "/Generic", "/ProcSet", "(*)", "(G?n*\\\\*?)", "{pop}", "8 string",
            "(*) {pop} 8 string /Category resourceforall", "/Generic /Category findresource dup length dict copy /Category defineresource",
            "dictstack", "8 array", "1183615869 internaldict", "cshow", "currentsystemparams", "currentuserparams", "flushfile", "xcheck",
            "systemdict", "/userdict", "SharedFontDirectory", "(1 2 add) 0 () /SubFileDecode filter cvx exec", "currentfile cvx exec",
            "/p { (p) 0 () /SubFileDecode filter cvx exec } def p", "currentfile 0 () /SubFileDecode filter cvx exec", "cvlit", "{ (a) cshow }",
            "/ASCII85Decode", "/ASCIIHexDecode", "/FlateDecode", "/RunLengthDecode", "/LZWDecode", "/DCTDecode", "/SubFileDecode",
            "/CCITTFaxDecode", "<< /K -1 /Columns 8 /Rows 3 >> /CCITTFaxDecode", "<< /K 2 /Columns 99999 /EndOfLine true /EncodedByteAlign true /BlackIs1 true >>",
            "/DeviceRGB", "/DeviceCMYK", "/DeviceGray", "/Pattern", "/x", "x", "/Helvetica", "(abc)", "<ff00>", "<~87cURD]i,\"Ebo80~>",
            "true", "false", "null", "[/Separation (S) /DeviceCMYK {dup dup dup}]", "[/Indexed /DeviceRGB 1 <ff000000ff00>]",
            "<< /ShadingType 2 /ColorSpace /DeviceRGB /Coords [0 0 1 1] /Function << /FunctionType 2 /C0 [0 0 0] /C1 [1 1 1] /N 1 >> >>",
            "<< /PatternType 2 /Shading << /ShadingType 3 /ColorSpace /DeviceGray /Coords [0 0 0 9 9 9] /Function << /FunctionType 3 /Functions [] /Bounds [] /Encode [] >> >> >>",
            "<< /ImageType 1 /Width 2 /Height 2 /BitsPerComponent 8 /ImageMatrix [2 0 0 2 0 0] /DataSource (abcdefghijkl) >>",
            "u", "U", "u u u", "U U", "{ u } 300 repeat", "1 1 300 { pop u 0 0 9 9 rectclip } for",
            // Probing, files, forms and devices.
            "status", "token", "bytesavailable", "resetfile", "pdfmark", "execform", "nulldevice", "strokepath", "pathforall",
            "currenthsbcolor", "currentcolorrendering", "gcheck", "rootfont", "setcachedevice2", "writestring", "(%stdout) (w) file",
            "//x", "{ //dup }", "/ReusableStreamDecode", "currentfile /ASCII85Decode filter /ReusableStreamDecode filter",
            "<< /Predictor 12 /Columns 2 /Colors 1 >> /FlateDecode", "<< /Predictor 2 /Colors 3 /BitsPerComponent 4 /Columns 3 >> /LZWDecode",
            "<< /Predictor 2 /BitsPerComponent 16 /Columns 5 >> /FlateDecode", "<< /FormType 1 /BBox [0 0 9 9] /PaintProc { pop 0 0 5 5 rectfill } >>",
            // Tiling patterns, uncoloured ones and patterns drawing patterns.
            "<< /PatternType 1 /PaintType 1 /XStep 5 /YStep 5 /BBox [0 0 5 5] /PaintProc { pop 0 0 3 3 rectfill } >> matrix makepattern",
            "<< /PatternType 1 /PaintType 2 /XStep 5 /YStep 5 /BBox [0 0 5 5] /PaintProc { pop dup setpattern 0 0 3 3 rectfill } >>",
            "[/Pattern /DeviceRGB] setcolorspace", "1 0 0", "setpattern",
            // Mesh and function shadings, sampled functions.
            "<< /ShadingType 4 /ColorSpace /DeviceGray /DataSource [0 0 0 0 0 9 0 1 0 0 9 1 1 9 9 0] >>",
            "<< /ShadingType 5 /ColorSpace /DeviceGray /VerticesPerRow 2 /DataSource [0 0 0 9 0 1 0 9 1 9 9 0] >>",
            "<< /ShadingType 6 /ColorSpace /DeviceGray /BitsPerCoordinate 8 /BitsPerComponent 8 /BitsPerFlag 8 /Decode [0 9 0 9 0 1] /DataSource <00ff10ff20ff30> >>",
            "<< /ShadingType 7 /ColorSpace /DeviceRGB /BitsPerCoordinate 32 /BitsPerComponent 16 /BitsPerFlag 8 /Decode [0 9 0 9 0 1 0 1 0 1] /DataSource (abc) >>",
            "<< /ShadingType 1 /ColorSpace /DeviceGray /Function << /FunctionType 0 /Domain [0 1 0 1] /Range [0 1] /Size [2 2] /BitsPerSample 8 /DataSource <00ff00ff> >> >>",
            "<< /FunctionType 0 /Domain [0 1] /Range [0 1 0 1 0 1] /Size [99999999] /BitsPerSample 32 /DataSource () >>",
            // Masked images.
            "<< /ImageType 3 /InterleaveType 1 /DataDict << /ImageType 1 /Width 2 /Height 2 /BitsPerComponent 8 /ImageMatrix [2 0 0 2 0 0] /DataSource (abcdefghijklmnop) >> /MaskDict << /ImageType 1 /Width 2 /Height 2 /BitsPerComponent 1 /ImageMatrix [2 0 0 2 0 0] >> >>",
            "<< /ImageType 3 /InterleaveType 2 /DataDict << /ImageType 1 /Width 3 /Height 1 /BitsPerComponent 8 /ImageMatrix [3 0 0 1 0 0] /DataSource (abcdefghijklmnop) >> /MaskDict << /ImageType 1 /Width 3 /Height 5 /BitsPerComponent 1 /ImageMatrix [3 0 0 5 0 0] >> >>",
            "<< /ImageType 4 /Width 2 /Height 2 /BitsPerComponent 8 /Decode [0 1 0 1 0 1 0 1] /ImageMatrix [2 0 0 2 0 0] /MaskColor [97 98 99 100] /DataSource (abcdefghijklmnop) >>",
            "2 2 8 [2 0 0 2 0 0] (abcdefghijklmnop) false 4 colorimage",
            // Type 3 fonts and the show operators that space their glyphs.
            "/T3 << /FontType 3 /FontMatrix [0.1 0 0 0.1 0 0] /FontBBox [0 0 9 9] /Encoding [/a /b] /BuildChar { pop pop 9 0 setcharwidth 0 0 5 5 rectfill } >> definefont setfont",
            "/T4 << /FontType 3 /FontMatrix [1 0 0 1 0 0] /BuildGlyph { pop pop (x) show } /Encoding [/a] >> definefont setfont",
            "glyphshow", "xshow", "xyshow", "awidthshow", "kshow", "/a", "[1 2 3]",
            // Several data sources reading one file in turn, and sources of uneven lengths.
            "/f currentfile /ASCIIHexDecode filter def", "{f 3 string readstring pop}", "{f 0 string readstring pop}", "true 3 colorimage",
            "3 2 8 [3 0 0 2 0 0] {f 5 string readstring pop} {(ab)} {f 1 string readstring pop} true 3 colorimage",
            "<< /ImageType 1 /Width 3 /Height 2 /BitsPerComponent 4 /Decode [0 1 0 1 0 1 0 1] /ImageMatrix [3 0 0 2 0 0] /MultipleDataSources true /DataSource [(a) (bc) {f 2 string readstring pop} (d)] >>",
            // Local VM restored: dictionary changes undone, saves restored twice.
            "/a 1 def save /a 2 def", "restore", "save dup restore", "1 dict save exch /k 1 put", "cachestatus", "/n 0 def save /n n 1 add store",
            // Executable strings and keys other than names (PLRM 3rd ed., `cvx`, `load`).
            "(>>) cvx", "(1 2 add) cvx exec", "(x) cvx dup exec", "0 load", "1 { } def", "(mark) cvx cvlit",
            // CID-keyed fonts found by name over predefined CMaps, horizontal and vertical.
            "/Ryumin-Light-90ms-RKSJ-H", "/Ryumin-Light-EUC-V", "/Ryumin-Light-H", "/GBT-EUC-H", "/KSCms-UHC-V", "/UniJIS-UTF16-V",
            "[/Ryumin-Light]", "composefont", "<93fa967b8540fd0d41>", "<c7d1a4a4467c>"
        ])
        .prop_map(str::to_string),
    ]
}

proptest! {
    #![proptest_config(config())]

    /// Damaged password-protected PDFs, opened with the open password, the permissions password
    /// (which reads the encryption dictionary to find the open password) or a wrong one.
    #[test]
    fn mutated_encrypted_pdfs_never_panic(
        cut in 0usize..2_000,
        edits in prop::collection::vec((0usize..2_000, any::<u8>()), 0..12),
        password in prop::sample::select(vec!["pw", "pw-owner", "wrong", ""]),
    ) {
        use vectorcraft_testkit::pdf::{PdfPage, pdf};
        let bytes = pdf(&[PdfPage::new(100.0, 100.0, "0 0 1 rg 20 20 50 50 re f")], Some("pw"));
        let b64 = mutated_b64(bytes, cut, &edits);
        let r = catch_quiet(|| {
            let mut s = vectorcraft_engine::Session::new();
            let p = json!({"name": "x.pdf", "dataBase64": b64, "password": password});
            let _ = no_panic(s.execute("document.pdfInfo", &p));
            if no_panic(s.execute("document.open", &p)).is_ok() {
                let d = (*s.doc().unwrap().doc).clone();
                survive("encrypted pdf", || Some(d)).unwrap();
            }
        });
        r.map_err(|msg| TestCaseError::fail(format!("{password}: panicked: {msg}")))?;
    }
}

// ---------- DXF ----------


/// Group-code pairs as DXF text.
fn dxf_text(pairs: &[(i32, String)]) -> String {
    pairs.iter().map(|(c, v)| format!("{c:>3}\n{v}\n")).collect()
}




proptest! {
    #![proptest_config(config())]



}

// ---------- Windows metafiles: opened, placed and pasted ----------

/// The rich document as EMF and as WMF.
fn metafile_samples() -> &'static [Vec<u8>] {
    static SAMPLES: std::sync::OnceLock<Vec<Vec<u8>>> = std::sync::OnceLock::new();
    SAMPLES.get_or_init(|| {
        let mut s = rich_session();
        ["emf", "wmf"]
            .iter()
            .map(|f| {
                let r = s.execute("document.export", &json!({ "format": f })).unwrap();
                vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap()
            })
            .collect()
    })
}

/// An EMF of a 100 × 100 mm frame on a device of 10 pixels a millimetre holding `records`.
fn emf_of(records: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut b: Vec<u8> = vec![];
    let mut head = vec![0u8; 108];
    head[..4].copy_from_slice(&1u32.to_le_bytes());
    head[4..8].copy_from_slice(&108u32.to_le_bytes());
    for (at, v) in [(32, 10_000i32), (36, 10_000), (72, 1000), (76, 1000), (80, 100), (84, 100), (100, 100_000), (104, 100_000)] {
        head[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    head[40..44].copy_from_slice(b" EMF");
    b.extend(head);
    for (kind, body) in records {
        let mut body = body.clone();
        body.resize(body.len().div_ceil(4) * 4, 0);
        b.extend(kind.to_le_bytes());
        b.extend((8 + body.len() as u32).to_le_bytes());
        b.extend(body);
    }
    b.extend([14, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 20, 0, 0, 0]);
    b
}

/// A placeable WMF of a 1 × 1 inch box holding `records`.
fn wmf_of(records: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut b: Vec<u8> = vec![0xD7, 0xCD, 0xC6, 0x9A, 0, 0, 0, 0, 0, 0, 0xA0, 0x05, 0xA0, 0x05, 0xA0, 0x05, 0, 0, 0, 0];
    let sum = b.as_chunks::<2>().0.iter().fold(0u16, |a, w| a ^ u16::from_le_bytes(*w));
    b.extend(sum.to_le_bytes());
    b.extend([1, 0, 9, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    for (function, body) in records {
        let mut body = body.clone();
        body.resize(body.len().div_ceil(2) * 2, 0);
        b.extend((3 + body.len() as u32 / 2).to_le_bytes());
        b.extend(function.to_le_bytes());
        b.extend(body);
    }
    b.extend([3, 0, 0, 0, 0, 0]);
    b
}

/// Open the bytes as `name`, and place them: neither may panic.
fn survive_metafile(what: &str, name: &str, bytes: &[u8]) -> Result<(), TestCaseError> {
    survive(what, || vectorcraft_engine::cmd::fileio::load(name, bytes).ok().map(|l| l.doc))?;
    let r = catch_quiet(|| {
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
        let _ = no_panic(s.execute("file.place", &json!({"name": name, "dataBase64": vectorcraft_format::base64_encode(bytes)})));
    });
    prop_assert!(r.is_ok(), "{what}: placing panicked: {:?}", r.err());
    Ok(())
}

/// A record body: random bytes, or small numbers (coordinates, counts and offsets that land).
fn arb_body() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        prop::collection::vec(any::<u8>(), 0..96),
        prop::collection::vec(prop_oneof![Just(0i32), Just(1), Just(-1), 0i32..200, Just(i32::MAX), Just(i32::MIN)], 0..24)
            .prop_map(|v| v.iter().flat_map(|x| x.to_le_bytes()).collect()),
    ]
}

/// WMF record functions VectorCraft reads, and one it doesn't.
const WMF_FUNCTIONS: [u16; 35] = [
    0x0324, 0x0325, 0x0538, 0x041B, 0x0418, 0x061C, 0x0817, 0x081A, 0x0830, 0x0416, 0x0415, 0x02FA, 0x02FC, 0x02FB, 0x012D, 0x01F0, 0x0521, 0x0A32,
    0x0F43, 0x0B41, 0x0940, 0x061D, 0x0213, 0x0214, 0x001E, 0x0127, 0x0106, 0x0209, 0x012E, 0x00F7, 0x01F9, 0x0142, 0x0103, 0x0104, 0x0999,
];

proptest! {
    #![proptest_config(config())]

    #[test]
    fn mutated_metafiles_never_panic(which in 0usize..2, cut in 0usize..200_000, edits in prop::collection::vec((0usize..200_000, any::<u8>()), 0..16)) {
        let mut bytes = metafile_samples()[which].clone();
        let n = bytes.len();
        for &(at, b) in &edits {
            bytes[at % n] = b;
        }
        bytes.truncate(cut.max(8));
        survive_metafile("mutated metafile", ["x.emf", "x.wmf"][which], &bytes)?;
    }

    #[test]
    fn random_emf_records_never_panic(records in prop::collection::vec((0u32..124, arb_body()), 0..32)) {
        survive_metafile("EMF records", "x.emf", &emf_of(&records))?;
    }

    #[test]
    fn random_wmf_records_never_panic(records in prop::collection::vec((prop::sample::select(WMF_FUNCTIONS.to_vec()), arb_body()), 0..32)) {
        survive_metafile("WMF records", "x.wmf", &wmf_of(&records))?;
    }

    #[test]
    fn random_headerless_wmf_records_never_panic(records in prop::collection::vec((prop::sample::select(WMF_FUNCTIONS.to_vec()), arb_body()), 0..32)) {
        // Without the placeable header, the map and mix modes the records set decide the scale.
        survive_metafile("headerless WMF records", "x.wmf", &wmf_of(&records)[22..])?;
    }

    #[test]
    fn random_emf_plus_records_never_panic(records in prop::collection::vec((0x4001u16..0x403B, any::<u16>(), arb_body()), 0..32)) {
        // One comment of EMF+ records (type, flags, data) after an EmfPlusHeader.
        let mut plus = vec![];
        let header = (0x4001u16, 0u16, [0xDBC0_1002u32, 0, 96, 96].iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>());
        for (kind, flags, data) in std::iter::once(&header).chain(&records) {
            plus.extend(kind.to_le_bytes());
            plus.extend(flags.to_le_bytes());
            plus.extend((12 + data.len() as u32).to_le_bytes());
            plus.extend((data.len() as u32).to_le_bytes());
            plus.extend(data);
        }
        let mut comment = (4 + plus.len() as u32).to_le_bytes().to_vec();
        comment.extend(b"EMF+");
        comment.extend(plus);
        survive_metafile("EMF+ records", "x.emf", &emf_of(&[(70, comment)]))?;
    }

    #[test]
    fn pasted_metafiles_never_panic(which in 0usize..2, cut in 0usize..200_000, edits in prop::collection::vec((0usize..200_000, any::<u8>()), 0..12)) {
        paste_from_elsewhere("clipboard.importEmf", json!({"dataBase64": mutated_b64(metafile_samples()[which].clone(), cut, &edits)}))?;
    }
}

// ---------- PICT pictures: opened and placed ----------

fn be16(v: &[i16]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_be_bytes()).collect()
}

/// A version 2 PICT file of a 200 × 100 pt frame holding `ops`, each padded to an even length.
fn pict_of(ops: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut b = vec![0; 512];
    b.extend(be16(&[0, 0, 0, 100, 200]));
    b.extend([0x00, 0x11, 0x02, 0xFF, 0x0C, 0x00]);
    b.extend(be16(&[-1, 0]));
    b.extend([0; 20]);
    for (op, data) in ops {
        b.extend(op.to_be_bytes());
        b.extend(data);
        if b.len() % 2 == 1 {
            b.push(0);
        }
    }
    b.extend([0x00, 0xFF]);
    b
}

/// A picture with shapes, a region, text, a pattern and a packed pixel map.
fn pict_sample() -> Vec<u8> {
    let region = [be16(&[36, 0, 0, 20, 20]), be16(&[0, 0, 20, 0x7FFF, 10, 10, 20, 0x7FFF, 20, 0, 10, 0x7FFF, 0x7FFF])].concat();
    let table = be16(&[0, 0, 0, 1, 0, -1, 0, 0, 1, 0, 0, -1]);
    let pixmap = [
        be16(&[0x8008u16 as i16, 0, 0, 2, 8]),
        be16(&[0, 0, 0, 0, 72, 0, 72, 0, 0, 8, 1, 8, 0, 0, 0, 0, 0, 0]),
        table,
        be16(&[0, 0, 2, 8, 10, 20, 14, 36, 0]),
        vec![2, 0xF9, 0, 9, 7, 1, 0, 1, 0, 1, 0, 1, 0],
    ]
    .concat();
    pict_of(&[
        (0x1A, be16(&[-1, 0, 0])),
        (0x07, be16(&[2, 2])),
        (0x31, be16(&[10, 20, 50, 80])),
        (0x30, be16(&[10, 20, 50, 80])),
        (0x0B, be16(&[8, 8])),
        (0x41, be16(&[0, 100, 40, 160])),
        (0x51, be16(&[50, 100, 90, 160])),
        (0x61, be16(&[0, 0, 60, 60, 30, 120])),
        (0x71, be16(&[22, 0, 0, 40, 40, 0, 0, 40, 40, 0, 40])),
        (0x81, region.clone()),
        (0x01, region),
        (0x20, be16(&[70, 10, 70, 190])),
        (0x21, be16(&[90, 100])),
        (0x0A, vec![0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55, 0xAA, 0x55]),
        (0x34, be16(&[0, 0, 10, 10])),
        (0x2C, [be16(&[9, 3]), vec![6], b"Geneva".to_vec()].concat()),
        (0x0D, be16(&[12])),
        (0x28, [be16(&[80, 10]), vec![3], b"abc".to_vec()].concat()),
        (0x98, pixmap),
    ])
}

/// The sample the mutations start from opens with art and an image.
#[test]
fn the_pict_sample_opens() {
    let l = vectorcraft_engine::cmd::fileio::load("x.pct", &pict_sample()).unwrap();
    assert_eq!(l.format.id, "pict");
    let (mut paths, mut images, mut texts) = (0, 0, 0);
    l.doc.walk(|n| match n.kind {
        vectorcraft_doc::NodeKind::Path { .. } => paths += 1,
        vectorcraft_doc::NodeKind::Image(_) => images += 1,
        vectorcraft_doc::NodeKind::Text(_) => texts += 1,
        _ => {}
    });
    assert!(paths > 8 && images == 1 && texts == 1, "{paths} paths, {images} images, {texts} texts, warnings {:?}", l.warnings);
}

/// PICT opcodes VectorCraft reads, and reserved ones it skips.
const PICT_OPS: [u16; 40] = [
    0x01, 0x03, 0x04, 0x07, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x12, 0x13, 0x14, 0x1A, 0x1B, 0x20, 0x21, 0x22, 0x23, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x30,
    0x31, 0x38, 0x41, 0x51, 0x61, 0x71, 0x81, 0x90, 0x91, 0x98, 0x99, 0x9A, 0x9B, 0xA1, 0x8200, 0x0024,
];

proptest! {
    #![proptest_config(config())]

    #[test]
    fn mutated_pict_pictures_never_panic(cut in 0usize..4096, edits in prop::collection::vec((0usize..4096, any::<u8>()), 0..16)) {
        let mut bytes = pict_sample();
        let n = bytes.len();
        for &(at, b) in &edits {
            bytes[512 + at % (n - 512)] = b;
        }
        bytes.truncate(cut.max(512));
        survive_metafile("mutated PICT", "x.pct", &bytes)?;
    }

    #[test]
    fn random_pict_opcodes_never_panic(ops in prop::collection::vec((prop::sample::select(PICT_OPS.to_vec()), arb_body()), 0..32)) {
        survive_metafile("PICT opcodes", "x.pct", &pict_of(&ops))?;
    }
}

// ---------- HP-GL plot files: opened and placed ----------

/// A plot file with every kind of instruction the reader plays, printer escapes around it.
const HPGL_SAMPLE: &[u8] = b"\x1b%-12345X@PJL ENTER LANGUAGE=HPGL2\r\n\x1bE\x1b%0BIN;BP1,\"job\";CO\"sample\";NP16;PC9,10,20,30;CR0,100,0,100,0,100;\
SP1;PW0.5;WU0;LA1,4,2,4,3,7;LT2,5,1;PA0,0;PD1000,0,1000,1000;PU;LT;PR;PD-500,0,0,-500;PA2000,2000;CI300,5;AA2000,2500,120;AR0,100,-90;\
AT2500,2500,3000,2000;RT100,100,200,0;BZ0,100,100,100,100,0;BR10,10,20,10,30,0;UL3,1,2,1;LT-3,2;EA3000,3000;ER100,-100;FT10,40;RA3500,3500;\
RR200,200;WG400,30,120;EW-400,0,360;PM0;PD4000,4000,4500,4000,4500,4500;PM1;PU4100,4100;PD4200,4100,4200,4200;PM2;FP1;EP;FT3,10,45;FP;\
IP0,0,10160,7620;SC0,100,0,100,1,20,80;PU10,10;PD20,20;SC0,10,0,10,2;PD5,5;SC;RO90;PD100,100;RO;IR10,10,90,90;IW0,0,5000,5000;\
DT*,1;LBlabel*SMx;PA0,0;SM;PE:\xc1<=\xbf\xc0\xc2\xc2;LB\x03PG;\x1b%0A\x1b*r1A\x1b*b4W\x01\x02\x03\x04\x1b*rC\x1b%1BPD1,1;\x1b%-12345X";

/// The sample the mutations start from opens as a plot with art.
#[test]
fn the_hpgl_sample_opens() {
    let l = vectorcraft_engine::cmd::fileio::load("x.plt", HPGL_SAMPLE).unwrap();
    assert_eq!(l.format.id, "hpgl");
    let mut paths = 0;
    l.doc.walk(|n| paths += usize::from(matches!(n.kind, vectorcraft_doc::NodeKind::Path { .. })));
    assert!(paths > 15, "{paths} paths, warnings {:?}", l.warnings);
}

/// A wedge drawn with the adaptive line type `LT-1` and a near-zero radius (a case the fuzz test
/// found) opens and places without a panic.
#[test]
fn an_adaptive_line_type_wedge_of_almost_no_radius_opens() {
    survive_metafile("HP-GL tiny adaptive wedge", "x.plt", b"IN;SP1;LT-1;EW0.000001,0,0;").unwrap();
}

/// A random instruction: a mnemonic the reader plays (or one it doesn't) with hostile numbers.
fn arb_hpgl_instruction() -> impl Strategy<Value = String> {
    let op = prop::sample::select(vec![
        "IN", "DF", "PU", "PD", "PA", "PR", "SP", "PW", "WU", "NP", "PC", "CR", "LT", "UL", "LA", "FT", "SC", "IP", "IR", "RO", "CI", "AA", "AR",
        "AT", "RT", "BZ", "BR", "EA", "ER", "RA", "RR", "EW", "WG", "PM", "FP", "EP", "LB", "DT", "SM", "CO", "PG", "IW", "PE", "ZZ",
    ]);
    let number = prop_oneof![
        Just("0".to_string()),
        Just("-1".to_string()),
        (-3000i32..3000).prop_map(|v| v.to_string()),
        Just("99999999999999999999999".to_string()),
        Just("0.000001".to_string()),
        Just("32767".to_string()),
        Just("-32768".to_string()),
        Just("360".to_string()),
    ];
    (op, prop::collection::vec(number, 0..8)).prop_map(|(o, n)| format!("{o}{};", n.join(",")))
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn random_hpgl_instructions_never_panic(instructions in prop::collection::vec(arb_hpgl_instruction(), 0..40)) {
        let bytes = format!("IN;SP1;{}", instructions.concat()).into_bytes();
        survive_metafile("HP-GL instructions", "x.plt", &bytes)?;
    }

    #[test]
    fn mutated_hpgl_files_never_panic(cut in 0usize..1200, edits in prop::collection::vec((0usize..1200, any::<u8>()), 0..16)) {
        let mut bytes = HPGL_SAMPLE.to_vec();
        let n = bytes.len();
        for &(at, b) in &edits {
            bytes[at % n] = b;
        }
        bytes.truncate(cut.max(4));
        survive_metafile("mutated HP-GL", "x.plt", &bytes)?;
    }

    #[test]
    fn hpgl_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..400)) {
        let mut b = b"IN;SP1;PD".to_vec();
        b.extend(bytes);
        survive_metafile("HP-GL garbage", "x.hpgl", &b)?;
    }
}

// ---------- print presets files ----------

/// Import print presets from `data`, then lay out (and print, when short) the rich document with
/// each one imported, by its settings and by its name.
fn print_presets(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "print.presets.import", data, |s, r| {
        let list = no_panic(s.execute("print.presets.list", &json!({}))).unwrap_or_default();
        for name in r["imported"].as_array().into_iter().flatten() {
            let Some(p) = list["presets"].as_array().into_iter().flatten().find(|p| &p["name"] == name) else { continue };
            let _ = no_panic(s.execute("print.preview", &json!({ "preset": name })));
            let settings = json!({ "settings": p["settings"] });
            let pages = no_panic(s.execute("print.preview", &settings)).ok().and_then(|v| v["pages"].as_u64());
            let _ = no_panic(s.execute("print.setup", &settings));
            if pages.is_some_and(|n| n <= 8) {
                let _ = no_panic(s.execute("file.print", &settings));
            }
        }
    })
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn print_presets_garbage_never_panics(s in ".{0,300}", head in prop::sample::select(vec!["", "{\"format\": \"vcprintpresets\", ", "{\"format\": \"vcprintpresets\", \"presets\": [{\"name\": \"x\", \"settings\": "])) {
        print_presets("print presets garbage", &format!("{head}{s}"))?;
    }

    #[test]
    fn mutated_print_presets_never_panic(cut in 0usize..20_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        let mut s = rich_session();
        s.execute("print.presets.save", &json!({"name": "Tiles", "settings": {"scaling": "tileFull", "scale": {"width": 300, "height": 300}, "marks": {"trim": true}}})).unwrap();
        s.execute("print.presets.save", &json!({"name": "Seps", "settings": {"output": {"mode": "separations", "inks": [{"name": "Cyan", "print": false}]}}})).unwrap();
        let text = s.execute("print.presets.export", &json!({"names": ["Tiles", "Seps", "[Default]"]})).unwrap()["data"].as_str().unwrap().to_string();
        print_presets("mutated print presets", &mutate_text(&text, cut, &edits))?;
    }
}

// ---------- perspective grid presets files ----------

/// Import perspective grid presets from `data`, then apply each one and draw on its grid.
fn perspective_presets(what: &str, data: &str) -> Result<(), TestCaseError> {
    survive_library(what, "perspective.presets.import", data, |s, r| {
        for name in r["imported"].as_array().into_iter().flatten() {
            let _ = no_panic(s.execute("perspective.grid.preset", &json!({"name": name})));
            let _ = no_panic(s.execute("perspective.grid.define", &json!({"name": name, "gridline": 3})));
            let _ = no_panic(
                s.execute("perspective.draw", &json!({"command": "shape.rectangle", "params": {"x": 300, "y": 400, "width": 40, "height": 30}})),
            );
        }
    })
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn perspective_presets_garbage_never_panics(s in ".{0,300}", head in prop::sample::select(vec!["", "{\"format\": \"vcperspective\", ", "{\"format\": \"vcperspective\", \"presets\": [{\"name\": \"x\", \"kind\": 3, \"scale\": "])) {
        perspective_presets("perspective presets garbage", &format!("{head}{s}"))?;
    }

    #[test]
    fn mutated_perspective_presets_never_panic(cut in 0usize..20_000, edits in prop::collection::vec(arb_edit(), 0..10)) {
        let mut s = rich_session();
        s.execute("perspective.presets.save", &json!({"name": "Tall", "kind": 3, "units": "inches", "scale": [1, 4], "angle": 25, "thirdVp": [1, 30]})).unwrap();
        s.execute("perspective.presets.save", &json!({"name": "Flat", "preset": "[1P-Low View]", "gridline": 4, "groundColor": "#00ff00"})).unwrap();
        let text = s.execute("perspective.presets.export", &json!({"names": ["Tall", "Flat", "[2P-High View]"]})).unwrap()["data"].as_str().unwrap().to_string();
        perspective_presets("mutated perspective presets", &mutate_text(&text, cut, &edits))?;
    }
}

// ---------- the editing data of Illustrator files ----------

/// `data` as ASCII85, up to the `~>`.
fn ascii85(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(4) {
        let mut x = chunk.iter().enumerate().fold(0u32, |v, (i, b)| v | u32::from(*b) << (24 - 8 * i));
        let mut d = [0u8; 5];
        for c in d.iter_mut().rev() {
            *c = (x % 85) as u8 + b'!';
            x /= 85;
        }
        out.extend(d.iter().take(chunk.len() + 1).map(|c| char::from(*c)));
    }
    out.push_str("~>");
    out
}

/// A pattern's definition in editing data: its name and tile, its art on `%_` lines, its end.
const PATTERN: &str = "%AI3_BeginPattern: (Dots)\n(Dots) 0 0 6 6\n%_0 A\n%_0 Ae\n%_u\n%_0 0 m\n%_0 6 L\n%_6 6 L\n%_6 0 L\n%_n\n%_0 0 0 1 K\n%_1 J 1 w\n%_3 3 m\n%_3 3 L\n%_S\n%_U\n%_9 () XW\nE\n%AI3_EndPattern\n";

/// The swatches in editing data: process, spot (one the art doesn't use), global, a gradient that
/// isn't defined and a pattern, a colour group, versioned content read once, and the None swatch
/// and a swatch without a name, as version 7 files write them.
const PALETTE: &str = "%AI5_BeginPalette\n0 0 Pb\nPn\nPc\n%AI17_Begin_Content_if_version_gt:24 15\n1 1 1 1 ([Registration]) 0 Xs\n([Registration])\nPc\n\
     0 0 0 1 k\n(Black)\nPc\n0.1 0.9 0.8 0 (Unused ink) 0 x\n(Unused ink)\nPc\n0 0 0 1 0 0 0 (Black global) 0 1 Xk\n(Black global)\nPc\n\
     Bb\n2 (G) 0 0 0 1 1 0 0 1 0 0 1 Bg\n0 BB\n(G)\nPc\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n(Dots)\nPc\n\
     %AI17_Alternate_Content\n0 0 0 1 k\n(Black)\nPc\n%AI17_End_Versioned_Content\n1 (Greys) 1 Pg\n0 0 0 0.5 k\n(K50)\nPc\n0.75 g\nPc\nPB\n%AI5_EndPalette\n";

/// The editing data of a file with every sort of object the layers reader knows, and some it doesn't.
fn editing_text() -> String {
    let square = |x: u32| format!("0 0 1 0 k\n{x} 10 m\n{} 10 L\n{} 14 L\n{x} 14 L\nf", x + 4, x + 4);
    let body = [
        "u".to_string(),
        square(10),
        "U".into(),
        "*u".into(),
        square(20),
        square(30),
        "*U".into(),
        "q".into(),
        square(40),
        "50 10 m 60 10 L 60 20 L 50 20 L h W f".into(),
        "Q".into(),
        // Transparency: a path's, a group's state block, and an object's opacity mask.
        "0 0 1 0 k\n2 0.5 0 0 1 Xy".into(),
        square(86),
        "0 Ae\nu".into(),
        square(90),
        "U\n0 0.4 1 1 1 Xy\n0 0 Xd\n6 () XW".into(),
        square(94),
        "%_/ArtDictionary :\n%_/Mask :\n%_X=\n%_0 g\n%_94 10 m\n%_98 10 L\n%_98 14 L\n%_f\n%_X+\n%_(Art) ,\n%_1 /Bool (Clipping) ,\n\
         %_0 /Bool (Inverted) ,\n%_0 /Bool (Disabled) ,\n%_1 /Bool (Linked) ,\n%_; (AI9_Mask) ,\n%_;"
            .into(),
        // Smooth and corner anchors by the case of each segment's operator, back to the start.
        "10 50 m 20 60 30 50 40 50 c 45 50 50 55 v 55 60 60 60 Y 60 50 l 50 40 30 40 10 50 C s".into(),
        "70 50 m 80 50 l 80 60 L 70 60 y h S".into(),
        "1 Xw".into(),
        "u".into(),
        "/AI11Text :\n0 /FreeUndo ,\n;".into(),
        "/AI11Text :\n1 /FreeUndo ,\n;\n1 (Style) XW".into(),
        "U".into(),
        "0 Xw".into(),
        "1 0 0 0 1 0 Bg".into(),
        "Bb\n1 (G) 0 0 0 1 1 0 0 1 0 0 1 Bg\n1 0 0 1 0 0 Xm\n0 0 0 0 0 0 Bm\n1 0 0 1 5 5 Bm\n0 BB".into(),
        "0 1 w 2 J 0 j 4 M [3 2]0 d 1 D".into(),
        "0 0 0 1 (Black global) 0.4 0 Xk".into(),
        "0 0 0 0 0 0 0 ([Registration]) 0.1 1 Xz".into(),
        "(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p".into(),
        square(60),
        // A 2 × 1 CMYK image with an alpha channel, its samples in hex.
        "%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceCMYK XN\n[ 4 0 0 4 10 30 ] 2 1 0 Xh\n[ 4 0 0 4 10 30 ] 0 0 2 1 2 1 8 4 1 0 0 0 4 4 0 0\n\
         %%BeginData: 18\rXI\nff000000000000ff80ff\n%%EndData\nXH\n%AI5_EndRaster\nN"
            .into(),
        // A 9 × 2 1-bit image mask (ImageMask 1) colorized by the global fill before it, in hex.
        "0 0 0 1 (Black global) 0.4 0 Xk\n%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceGray XN\n[ 4 0 0 4 10 30 ] 9 2 0 Xh\n\
         [ 4 0 0 4 10 30 ] 0 0 9 2 9 2 1 1 0 0 0 1 4 4 0 0\n%%BeginData: 8\rXI\n6080ff00\n%%EndData\nXH\n%AI5_EndRaster\nN"
            .into(),
    ]
    .join("\n");
    let layer = |name: &str, visible: u8, body: &str| {
        format!("%AI5_BeginLayer\n{visible} 1 1 1 0 0 1 0 79 128 255 0 50 0 Lb\n({name}) Ln\n{body}\nLB\n%AI5_EndLayer--\n")
    };
    format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%HiResBoundingBox: 0 0 100 100\n%AI3_Cropmarks: 0 0 100 100\n{PATTERN}{PALETTE}{}{}%%Trailer\n",
        layer("One", 1, &format!("{body}\n{}", layer("Sub", 1, &square(70)))),
        // A layer's own state block after its last compound path.
        layer("Two", 0, &format!("{}\n*u\n{}\n{}\n*U\n1 0.6 1 2 1 Xy\n0 0 Xd\n7 () XW", square(80), square(84), square(88)))
    )
}




fn zstd(text: &str) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(text.as_bytes(), ruzstd::encoding::CompressionLevel::Fastest)
}

/// A one-page `.ai` whose page's content stream has decode parameters the import refuses (#4366),
/// carrying `private` in one stream with the entries `filter`, under the key `{key}1` of its
/// `PieceInfo`; with an object stream of entries `object_stream` when they aren't empty.
fn refused_ai(private: &[u8], key: &str, filter: &str, object_stream: &str) -> Vec<u8> {
    let stream = |dict: &str, data: &[u8]| [format!("<< {dict} /Length {} >>\nstream\n", data.len()).as_bytes(), data, b"\nendstream"].concat();
    let mut objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents 4 0 R /PieceInfo << /Illustrator << /Private << /{key}1 5 0 R /NumBlock 1 >> >> >> >>")
            .into_bytes(),
        stream("/DecodeParms << /Predictor 12 /Columns 0 >>", b"0 g 10 10 20 20 re f"),
        stream(filter, private),
    ];
    if !object_stream.is_empty() {
        objects.push(stream(object_stream, b"8 0 << >>"));
    }
    let mut out = b"%PDF-1.5\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n", i + 1).bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    for off in offsets {
        out.extend(format!("{off:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
    out
}

/// An EPS drawing a square whose private data holds `editing`.
fn editing_eps(editing: &str) -> Vec<u8> {
    let lines: Vec<String> = ascii85(&zstd(editing)).as_bytes().chunks(60).map(|c| format!("%{}", String::from_utf8_lossy(c))).collect();
    format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 100\n%%EndComments\n0 0 1 0 setcmykcolor 10 10 moveto 14 10 lineto 14 14 lineto closepath fill\nshowpage\n%%EOF\n%AI9_PrivateDataBegin\n%AI24_DataStream\n{}\n%AI9_PrivateDataEnd\n",
        lines.join("\n")
    )
    .into_bytes()
}





/// The text document of area type threaded through two frames and of type on a path, as an
/// editing copy keeps it (frames on the canvas, y down).
fn frames_document_text() -> String {
    let area = |x0: f64, x1: f64| {
        let (y0, y1) = (8201.5, 8221.5);
        let corners = [(x0, y0), (x0, y1), (x1, y1), (x1, y0), (x0, y0)];
        let segments: Vec<String> = corners.windows(2).map(|w| format!("{0} {1} {0} {1} {2} {3} {2} {3}", w[0].0, w[0].1, w[1].0, w[1].1)).collect();
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 /7 18 >> >> >>", segments.join(" "))
    };
    let path = "<< /0 << /0 [ 0 0 ] /1 << /0 [ 8151.5 8171.5 8151.5 8171.5 8191.5 8161.5 8191.5 8171.5 8191.5 8171.5 8211.5 8181.5 8231.5 8171.5 8231.5 8171.5 ] >> /2 << /0 2 /6 [ 0.5 2.0 ] >> >> >>";
    let story = |text: &str, frames: &str, style: &str| {
        let n = text.chars().count() - text.matches('\\').count();
        format!(
            "<< /0 << /0 ({text}) /5 << /0 [ << /0 << /0 << /0 () /5 << /0 2 /1 4 /2 6 >> /6 0 >> >> /1 {n} >> ] >> \
             /6 << /0 [ << /0 << /0 << /0 () /5 0 /6 << {style} >> >> >> /1 {n} >> ] >> >> /1 << /0 [ {frames} ] >> >>"
        )
    };
    format!(
        "/0 << /1 << /0 [ << /0 << /0 << /0 (Helvetica) >> >> >> ] >> /8 << /0 [ {} {} {path} ] >> >>\n/1 << /1 [ {} {} ] /2 << /1 12.0 >> >>\n",
        area(8151.5, 8201.5),
        area(8211.5, 8241.5),
        story(
            "A story in two frames\\rand more\\r",
            "<< /0 0 >> << /0 1 >>",
            "/1 9.0 /8 50 /53 << /99 /CAITextPaint /0 << /0 1 /1 [ 1.0 1.0 0.0 0.0 ] >> >>"
        ),
        story("On a path\\r", "<< /0 2 >>", "/1 8.0"),
    )
}

/// The text document of vertical type (key 1 of what each frame carries at 2) as an editing copy
/// keeps it: point type in two columns with a Tate-chu-yoko run (key 35 at 3), area type and type
/// on a path.
fn vertical_document_text() -> String {
    let area = "<< /0 << /0 [ 0 0 ] /1 << /0 [ 8151.5 8171.5 8151.5 8171.5 8201.5 8171.5 8201.5 8171.5 8201.5 8171.5 8201.5 8171.5 8201.5 8221.5 8201.5 8221.5 8201.5 8221.5 8201.5 8221.5 8151.5 8221.5 8151.5 8221.5 8151.5 8221.5 8151.5 8221.5 8151.5 8171.5 8151.5 8171.5 ] >> /2 << /0 1 /1 2 >> >> >>";
    let path = "<< /0 << /0 [ 0 0 ] /1 << /0 [ 8151.5 8171.5 8151.5 8171.5 8191.5 8161.5 8191.5 8171.5 8191.5 8171.5 8211.5 8181.5 8231.5 8171.5 8231.5 8171.5 ] >> /2 << /0 2 /1 2 /6 [ 0.5 2.0 ] >> >> >>";
    let point = "<< /0 << /0 [ 0 0 ] /2 << /1 2 /2 [ 1 0 0 1 3 4 ] >> >> >>";
    let run = |n: usize, own: &str| format!("<< /0 << /0 << /0 () /5 0 /6 << /0 0 /1 12.0 {own}>> >> >> /1 {n} >>");
    let story = |text: &str, frame: usize, runs: &str, layout: &str| {
        format!("<< /0 << /0 ({text}) /6 << /0 [ {runs} ] >> >> /1 << /0 [ << /0 {frame} >> ] {layout}>> >>")
    };
    let columns = "/2 [ << /99 /PC /6 [ << /99 /F /0 << /0 [ 8200.0 8180.0 ] >> /6 [ << /99 /R /6 [ << /99 /R /6 [ \
                   << /99 /L /6 [ << /99 /S /0 << /0 [ 0.0 0.0 ] >> >> ] >> << /99 /L /0 << /0 [ -6.0 14.4 ] >> /6 [ << /99 /S /0 << /0 [ 0.0 0.0 ] >> >> ] >> \
                   ] >> ] >> ] >> ] >> ] ";
    format!(
        "/0 << /1 << /0 [ << /0 << /0 << /0 (Helvetica) >> >> >> ] >> /8 << /0 [ {point} {area} {path} ] >> >>\n\
         /1 << /1 [ {} {} {} ] /2 << /1 12.0 /35 1 /47 0 /48 0 >> >>\n",
        story("Day 12\\rnext\\r", 0, &format!("{} {} {}", run(4, ""), run(2, "/35 3 /47 -2 /48 3 "), run(6, "/35 2 ")), columns),
        story("Vertical area type wraps\\r", 1, &run(25, ""), ""),
        story("On a path\\r", 2, &run(10, "/35 3 "), ""),
    )
}

/// The text objects of [`vertical_document_text`]: one per story.
const VERTICAL_OBJECTS: &str = "/AI11Text :\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n/AI11Text :\n0 /FrameIndex ,\n1 /StoryIndex ,\n;\n/AI11Text :\n0 /FrameIndex ,\n2 /StoryIndex ,\n;\n";

/// The fuzzed vertical fixture, as it is, reads as vertical point, area and path type, with the
/// Tate-chu-yoko run marked.
#[test]
fn the_vertical_fixture_reads_as_vertical_type() {
    let l = vectorcraft_engine::cmd::fileio::load("x.eps", &text_eps_of(&vertical_document_text(), VERTICAL_OBJECTS)).unwrap();
    let mut found = vec![];
    l.doc.walk(|n| {
        if let vectorcraft_doc::NodeKind::Text(t) = &n.kind {
            let tcy: Vec<&str> = t.runs.iter().filter(|r| r.style.tate_chu_yoko).map(|r| r.text.as_str()).collect();
            let roman: Vec<&str> = t.runs.iter().filter(|r| r.style.standard_vertical_roman).map(|r| r.text.as_str()).collect();
            found.push((t.plain_text(), t.vertical, tcy.join("|"), roman.join("|")));
        }
    });
    // Key 35 at 1 in the defaults: Standard Vertical Roman Alignment, except where a run has 3 or 2.
    let want = [
        ("Day 12\nnext", true, "12", "Day "),
        ("Vertical area type wraps", true, "", "Vertical area type wraps"),
        ("On a path", true, "On a path", ""),
    ];
    assert_eq!(found, want.map(|(a, b, c, d)| (a.to_string(), b, c.to_string(), d.to_string())), "{:?}", l.warnings);
}

/// The text objects of [`frames_document_text`]: story 0 in its two frames, story 1 in its one (a
/// text object names its frame among its story's).
const FRAME_OBJECTS: &str = "/AI11Text :\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n/AI11Text :\n1 /FrameIndex ,\n0 /StoryIndex ,\n;\n/AI11Text :\n0 /FrameIndex ,\n1 /StoryIndex ,\n;\n";

/// The fuzzed fixture, as it is, reads as area type in two threaded frames and type on a path.
#[test]
fn the_frames_fixture_reads_as_threaded_area_type_and_type_on_a_path() {
    let l = vectorcraft_engine::cmd::fileio::load("x.eps", &text_eps_of(&frames_document_text(), FRAME_OBJECTS)).unwrap();
    let mut kinds = vec![];
    l.doc.walk(|n| {
        if let vectorcraft_doc::NodeKind::Text(t) = &n.kind {
            kinds.push(match t.kind {
                vectorcraft_doc::TextKind::Area { .. } => "area",
                vectorcraft_doc::TextKind::OnPath { .. } => "path",
                _ => "point",
            });
        }
    });
    assert_eq!(kinds, ["area", "area", "path"], "{:?}", l.warnings);
    assert_eq!(l.doc.text_threads.len(), 1);
}

/// The smallest failing input `eps_text_frames_never_panic` found: the first frame's bottom edge
/// has a control point at x = 8e115 (`8e0115`). That frame holds no text, and the story flows on
/// into the second frame.
#[test]
fn a_text_frame_with_a_control_point_at_8e115_opens_with_its_story_in_the_next_frame() {
    let text = mutate_text(&frames_document_text(), 1397, &[(191, '1'), (1586, 'e')]);
    assert!(text.contains(" 8e0115 "));
    let l = vectorcraft_engine::cmd::fileio::load("x.eps", &text_eps_of(&text, FRAME_OBJECTS)).unwrap();
    let mut texts = vec![];
    l.doc.walk(|n| {
        if let vectorcraft_doc::NodeKind::Text(t) = &n.kind {
            texts.push(t.plain_text());
        }
    });
    assert_eq!(texts, ["", "A story in two frames\nand more", "On a path"], "{:?}", l.warnings);
    survive("a frame with a control point at 8e115 pt", || Some(l.doc)).unwrap();
}


/// An EPS with the text `objects` on a hidden layer, their stories in `document`.
fn text_eps_of(document: &str, objects: &str) -> Vec<u8> {
    editing_eps(&text_editing_of(document, objects))
}

/// The editing data of [`text_eps_of`].
fn text_editing_of(document: &str, objects: &str) -> String {
    let lines: Vec<String> = ascii85(document.as_bytes()).as_bytes().chunks(60).map(|c| format!("%{}", String::from_utf8_lossy(c))).collect();
    format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%HiResBoundingBox: 0 0 100 100\n%AI3_Cropmarks: 0 0 100 100\n%AI3_TemplateBox: 50 50 50 50\n\
         %AI5_BeginLayer\n0 1 1 1 0 0 1 0 79 128 255 0 50 0 Lb\n(Spare) Ln\n{objects}LB\n%AI5_EndLayer--\n\
         %AI11_BeginTextDocument\n/AI11TextDocument : /ASCII85Decode ,\n{}\n%AI11_EndTextDocument\n%%Trailer\n",
        lines.join("\n")
    )
}




/// A text document of `kinds.len()` stories, story `i` in frame `i` of kind `kinds[i]` (0 point type,
/// 1 area type, 2 type on a path, others a kind the reader doesn't know), with a line break inside
/// its paragraph (`\x03`) at each of `soft` (by character, wrapping, the story's end included) and
/// its paragraph runs `runs` characters long (the whole story when there are none).
fn stories_document_text(kinds: &[u8], soft: &[usize], runs: &[usize]) -> String {
    let frames: Vec<String> = kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            // Six rows 15 apart on the page (the canvas is y down), then the same rows again a little
            // to the right.
            let (x, y) = (8150.0 + (i / 6) as f64 * 5.0, 8150.0 + (i % 6) as f64 * 15.0);
            match kind {
                0 => format!("<< /0 << /0 [ 0 0 ] /2 << /2 [ 1 0 0 1 {x} {y} ] >> >> >>"),
                1 => format!(
                    "<< /0 << /0 [ 0 0 ] /1 << /0 [ {x} {y} {x} {y} {0} {y} {0} {y} {0} {y} {0} {y} {0} {1} {0} {1} {0} {1} {0} {1} {x} {1} {x} {1} ] >> /2 << /0 1 >> >> >>",
                    x + 70.0,
                    y + 12.0
                ),
                2 => format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {x} {y} {x} {y} {0} {y} {0} {y} ] >> /2 << /0 2 /6 [ 0 0.9 ] >> >> >>", x + 70.0),
                k => format!("<< /0 << /0 [ 0 0 ] /2 << /0 {k} /2 [ 1 0 0 1 {x} {y} ] >> >> >>"),
            }
        })
        .collect();
    let stories: Vec<String> = (0..kinds.len())
        .map(|i| {
            let mut chars: Vec<char> = format!("Line {i} one two\rand three\r").chars().collect();
            for at in soft {
                chars.insert(at % (chars.len() + 1), '\x03');
            }
            let n = chars.len();
            let text: String = chars.iter().map(|c| match c {
                '\r' => "\\r".to_string(),
                '\x03' => "\\003".to_string(),
                c => c.to_string(),
            }).collect();
            let lengths: Vec<usize> = if runs.is_empty() { vec![n] } else { runs.to_vec() };
            let paras: String = lengths
                .iter()
                .enumerate()
                .map(|(j, len)| format!("<< /0 << /0 << /0 () /5 << /0 {} /2 2 /4 {j} >> /6 0 >> >> /1 {len} >> ", j % 3))
                .collect();
            format!(
                "<< /0 << /0 ({text}) /5 << /0 [ {paras}] >> /6 << /0 [ << /0 << /0 << /0 () /5 0 /6 << /1 8.0 >> >> >> /1 {n} >> ] >> >> /1 << /0 [ << /0 {i} >> ] >> >>"
            )
        })
        .collect();
    format!(
        "/0 << /1 << /0 [ << /0 << /0 << /0 (Helvetica) >> >> >> ] >> /8 << /0 [ {} ] >> >>\n/1 << /1 [ {} ] /2 << /1 12.0 >> >>\n",
        frames.join(" "),
        stories.join(" ")
    )
}

/// An EPS whose page draws `page` (PostScript), with the text `objects` on a layer that shows and
/// the stories in `document`.
fn shown_text_eps(document: &str, objects: &str, page: &str) -> Vec<u8> {
    let lines: Vec<String> = ascii85(document.as_bytes()).as_bytes().chunks(60).map(|c| format!("%{}", String::from_utf8_lossy(c))).collect();
    let editing = format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%HiResBoundingBox: 0 0 100 100\n%AI3_Cropmarks: 0 0 100 100\n%AI3_TemplateBox: 50 50 50 50\n\
         %AI5_BeginLayer\n1 1 1 1 0 0 1 0 79 128 255 0 50 0 Lb\n(Words) Ln\n{objects}LB\n%AI5_EndLayer--\n\
         %AI11_BeginTextDocument\n/AI11TextDocument : /ASCII85Decode ,\n{}\n%AI11_EndTextDocument\n%%Trailer\n",
        lines.join("\n")
    );
    let stream = ascii85(&zstd(&editing));
    let private: Vec<String> = stream.as_bytes().chunks(60).map(|c| format!("%{}", String::from_utf8_lossy(c))).collect();
    format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 100\n%%EndComments\n0 0 0 1 setcmykcolor /Helvetica findfont 8 scalefont setfont {page}\nshowpage\n%%EOF\n%AI9_PrivateDataBegin\n%AI24_DataStream\n{}\n%AI9_PrivateDataEnd\n",
        private.join("\n")
    )
    .into_bytes()
}

/// The page of [`stories_document_text`]'s first story's first line, at its place.
const FIRST_LINE: &str = "8.5 91.5 moveto (Line 0 one two) show";

/// The text objects of `pairs` (story, frame among the story's).
fn story_objects(pairs: &[(u32, u32)]) -> String {
    pairs.iter().map(|(story, frame)| format!("/AI11Text :\n{frame} /FrameIndex ,\n{story} /StoryIndex ,\n;\n")).collect()
}

/// The fixture, as it is (point type, area type and type on a path, a line break inside each
/// paragraph), opens with its layers and every story's type from its text document.
#[test]
fn the_shown_stories_fixture_opens_with_its_layers() {
    let doc = stories_document_text(&[0, 1, 2], &[4], &[]);
    // Each story's first line where the page draws it: the area type's below its frame's top, the
    // type on a path on its path.
    let page = format!("{FIRST_LINE} 8.5 69.5 moveto (Line 1) show 8.5 61.5 moveto (Line 2) show");
    let l = vectorcraft_engine::cmd::fileio::load("x.eps", &shown_text_eps(&doc, &story_objects(&[(0, 0), (1, 0), (2, 0)]), &page)).unwrap();
    let names: Vec<_> = l.doc.layers.iter().map(|l| l.name.clone()).collect();
    assert_eq!(names, [Some("Words".to_string())], "{:?}", l.warnings);
    let mut texts = vec![];
    l.doc.walk(|n| {
        if let vectorcraft_doc::NodeKind::Text(t) = &n.kind {
            texts.push(t.plain_text());
        }
    });
    assert_eq!(texts.len(), 3, "{texts:?} {:?}", l.warnings);
    assert!(texts.iter().all(|t| t.contains('\u{2028}') && t.contains('\n')), "{texts:?}");
}


proptest! {
    #![proptest_config(config())]

    /// An Illustrator EPS with text objects that show, in frames of every kind and some the reader
    /// doesn't know, naming stories and frames that may not exist, with line breaks inside their
    /// paragraphs and paragraph runs of any length: read as its layers, or as its page.
    #[test]
    fn eps_shown_stories_never_panic(
        kinds in prop::collection::vec(0u8..4, 1..20),
        pairs in prop::collection::vec((0u32..24, 0u32..3), 1..24),
        soft in prop::collection::vec(0usize..40, 0..5),
        runs in prop::collection::vec(0usize..40, 0..5),
    ) {
        let kinds: Vec<u8> = kinds.into_iter().map(|k| if k == 3 { 9 } else { k }).collect();
        let bytes = shown_text_eps(&stories_document_text(&kinds, &soft, &runs), &story_objects(&pairs), FIRST_LINE);
        survive("EPS with shown stories", || vectorcraft_engine::cmd::fileio::load("x.eps", &bytes).ok().map(|l| l.doc))?;
    }


    /// The same for area type threaded through frames and type on a path, opened as a document
    /// (its threads flow).
    #[test]
    fn eps_text_frames_never_panic(cut in 0usize..2_500, edits in prop::collection::vec((0usize..2_500, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '(', ')', '/', '[', ']', '<', '>', '\\', 'e', '1', '2'])), 0..12)) {
        let text = mutate_text(&frames_document_text(), cut, &edits);
        let bytes = text_eps_of(&text, FRAME_OBJECTS);
        survive("mutated EPS text frames", || vectorcraft_engine::cmd::fileio::load("x.eps", &bytes).ok().map(|l| l.doc))?;
    }




    /// The same for vertical type and Tate-chu-yoko runs: orientations and baseline directions of
    /// any value, type or size.
    #[test]
    fn eps_vertical_text_never_panics(cut in 0usize..2_000, edits in prop::collection::vec((0usize..2_000, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '(', ')', '/', '[', ']', '<', '>', '\\', 'e', '1', '2', '3'])), 0..12)) {
        let text = mutate_text(&vertical_document_text(), cut, &edits);
        let bytes = text_eps_of(&text, VERTICAL_OBJECTS);
        survive("mutated EPS vertical text", || vectorcraft_engine::cmd::fileio::load("x.eps", &bytes).ok().map(|l| l.doc))?;
    }








    /// A `.ai` whose PDF part can't be read (#4366), its private data damaged or hostile, under
    /// any key and filter: it opens from its editing data, or with the error of its PDF part.
    #[test]
    fn ai_whose_pdf_part_cant_be_read_never_panics(
        cut in 0usize..3_000,
        edits in prop::collection::vec((0usize..3_000, prop::sample::select(vec!['0', '9', '-', ' ', '\n', '(', ')', '/', '%', 'u', 'U', 'L'])), 0..8),
        junk in prop::collection::vec(any::<u8>(), 0..64),
        key in prop::sample::select(vec!["AIPrivateData", "AIPDFPrivateData", "AIPrivateData2", "Other"]),
        filter in prop::sample::select(vec!["", "/Filter /FlateDecode", "/Filter /Bogus", "/Filter [/FlateDecode /FlateDecode]", "/DP << /Predictor 2 >>", "/Filter 7"]),
    ) {
        let text = mutate_text(&editing_text(), cut, &edits);
        let private = [b"%AI24_ZStandard_Data".as_slice(), &zstd(&text), &junk].concat();
        let bytes = refused_ai(&private, key, filter, "");
        let plain = vectorcraft_engine::cmd::fileio::LoadOptions { editing_data: false, ..Default::default() };
        let page_error = vectorcraft_engine::cmd::fileio::load_with("x.ai", &bytes, &plain).err().map(|e| e.to_string());
        prop_assert!(page_error.is_some());
        let _ = catch_quiet(|| vectorcraft_pdf::illustrator_data_without_pdf_part(&bytes, None)).expect("the private data read panicked");
        let mut opened = None;
        survive("a .ai whose PDF part can't be read", || {
            let l = vectorcraft_engine::cmd::fileio::load("x.ai", &bytes);
            opened = Some(l.as_ref().map(|l| l.editing_data).map_err(|e| e.to_string()));
            l.ok().map(|l| l.doc)
        })?;
        match opened {
            Some(Ok(editing)) => prop_assert!(editing),
            // The page's error, followed by why the editing data couldn't be read when it says.
            Some(Err(e)) => prop_assert!(page_error.as_deref().is_some_and(|p| e == p || e.starts_with(&format!("{p}; its editing data couldn't be read either ("))), "{e}"),
            None => {}
        }
    }

    /// The same file with a damaged table of objects and an object stream the reader would fail
    /// on: refused, as before.
    #[test]
    fn ai_with_a_hostile_object_stream_is_refused(damage in prop::collection::vec((0usize..200, any::<u8>()), 0..6)) {
        let private = [&b"%AI12_CompressedData"[..], &deflated(&vectorcraft_testkit::ai::sample_data())].concat();
        let mut bytes = refused_ai(&private, "AIPrivateData", "/Filter /FlateDecode", "/Type /ObjStm /N 1 /First 4 /DecodeParms << /Predictor 12 /Columns 0 >>");
        let table = bytes.windows(5).rposition(|w| w == b"xref\n").unwrap_or(0);
        for (at, b) in damage {
            let n = bytes.len() - table;
            if let Some(x) = bytes.get_mut(table + at % n) {
                *x = b;
            }
        }
        let read = catch_quiet(|| vectorcraft_pdf::illustrator_data_without_pdf_part(&bytes, None)).expect("the private data read panicked");
        prop_assert!(read.is_none());
        let opened = catch_quiet(|| vectorcraft_engine::cmd::fileio::load("x.ai", &bytes).is_ok()).expect("the open panicked");
        prop_assert!(!opened);
    }




















}
























/// A `.ai` in PostScript form as other apps write it: the procsets that define its operators named,
/// not included, so its program can't be run and its layers are read from it.
const NAMED_PROLOG_AI: &str = "%!PS-Adobe-3.0\n%%BoundingBox: 0 0 200 100\n%%DocumentNeededResources: procset Adobe_packedarray 2.0 0\n%AI3_TemplateBox: 288 384 288 384\n%%EndComments\n\
    %%BeginProlog\n%%IncludeResource: procset Adobe_packedarray 2.0 0\nAdobe_packedarray /initialize get exec\n%%EndProlog\n%%BeginSetup\n%AI5_Begin_NonPrinting\nNp\n%AI5_End_NonPrinting--\n%%EndSetup\n\
    %AI5_BeginLayer\n1 1 1 1 0 0 -1 191 63 255 Lb\n(A::B) Ln\n0 A\n0 R\n0 0 0 1 K\n0 i 1 J 1 j 0.8 w 4 M [2 1]0 d\n0 D\n%AI3_Note:a (note)\n%AI5_NoteMore:more\n1 Ap\n10 10 m\n90 10 L\n90 90 L\n10 90 L\n10 10 L\nS\n%%Note:\n0 Ap\n\
    u\n0.5 g\n20 20 m\n40 20 L\n40 40 L\nf\nU\n%AI5_BeginLayer\n0 1 1 1 0 0 3 0 0 0 Lb\n(Sub) Ln\n110 10 m\n190 90 L\nS\nLB\n%AI5_EndLayer--\nLB\n%AI5_EndLayer--\n\
    %%PageTrailer\ngsave annotatepage grestore showpage\n%%Trailer\n%%EOF\n";

/// The fixture, as it is, opens with its layer and sublayer.
#[test]
fn the_named_prolog_fixture_opens_with_its_layers() {
    let l = vectorcraft_engine::cmd::fileio::load("x.ai", NAMED_PROLOG_AI.as_bytes()).unwrap();
    let names: Vec<_> = l.doc.layers.iter().map(|l| l.name.clone()).collect();
    assert_eq!(names, [Some("A::B".to_string())], "{:?}", l.warnings);
    assert_eq!(l.doc.layers[0].children().unwrap().len(), 3);
}

proptest! {
    #![proptest_config(config())]

    /// A `.ai` in PostScript form whose prolog is named, not included, damaged: read from its
    /// layers, or refused.
    #[test]
    fn ps_form_ai_never_panics(cut in 0usize..1_200, edits in prop::collection::vec((0usize..1_200, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '(', ')', '/', '[', ']', '%', 'u', 'U', 'L', 'b', 'B', 'i', 'A', 'K', 'p', ':', '\u{e9}'])), 0..12)) {
        let text = mutate_text(NAMED_PROLOG_AI, cut, &edits);
        survive("mutated PostScript-form .ai", || vectorcraft_engine::cmd::fileio::load("x.ai", text.as_bytes()).ok().map(|l| l.doc))?;
    }


    /// The same file with placed EPS files in the version 7 form (no `%AI5_BeginPlace`), one
    /// linked and one included whole with a nested document and counted data in it, damaged.
    #[test]
    fn ps_form_ai_with_placed_files_never_panics(cut in 0usize..1_600, edits in prop::collection::vec((0usize..1_600, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '(', ')', '[', ']', '%', '`', '~', 'E', 'D'])), 0..12)) {
        let placed = "[1 0 0 1 10 20] 10 20 50 50 (a.eps) `\n%%IncludeFile: a.eps\n~\n\
            [-1 0 0 1 100 0] 10 20 50 50 4 4 0 (b.eps) `\n%%BeginDocument: b.eps\n%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 10 20 50 50\n\
            %%BeginDocument: c.eps\n~\n%%EndDocument\n%%BeginData: 14 Binary Bytes\n%%EndDocument\n0 0 1 setrgbcolor 10 20 40 30 rectfill\n%%EndDocument\n~\n";
        let fixture = NAMED_PROLOG_AI.replacen("0 Ap\nu\n", &format!("0 Ap\n{placed}u\n"), 1);
        prop_assert!(fixture.contains("(b.eps)"));
        let text = mutate_text(&fixture, cut, &edits);
        survive("mutated PostScript-form .ai with placed files", || vectorcraft_engine::cmd::fileio::load("x.ai", text.as_bytes()).ok().map(|l| l.doc))?;
    }

    /// The same file with linked images in the version 7 form (`XG`, then `XF` with no samples),
    /// one with an embedded image between its `XG` and `XF`, damaged.
    #[test]
    fn ps_form_ai_with_linked_images_never_panics(cut in 0usize..1_400, edits in prop::collection::vec((0usize..1_400, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '(', ')', '[', ']', '%', 'X', 'F', 'G'])), 0..12)) {
        let linked = "%AI5_File:\n%AI5_BeginRaster\n(a.tif) 0 XG\n[ 2 0 0 3 10 80 ] 0 0 10 20 10 20 8 3 0 0 0 0 XF\n%AI5_EndRaster\n\
            %AI5_File:\n%AI5_BeginRaster\n(b.tif) 1 XG\n[ 1 0 0 1 0 100 ] 0 0 1 1 1 1 8 3 0 0 0 0 XI\n%FF0000\n%AI5_EndRaster\n\
            [ 0 1 -1 0 50 50 ] 0 0 4 4 4 4 8 3 0 0 0 0 XF\n";
        let fixture = NAMED_PROLOG_AI.replacen("0 Ap\nu\n", &format!("0 Ap\n{linked}u\n"), 1);
        prop_assert!(fixture.contains("(a.tif)"));
        let text = mutate_text(&fixture, cut, &edits);
        survive("mutated PostScript-form .ai with linked images", || vectorcraft_engine::cmd::fileio::load("x.ai", text.as_bytes()).ok().map(|l| l.doc))?;
    }
}

/// A `.ai` in the legacy format without layers (Illustrator 3) whose prolog is included: its
/// program runs, and its art is read on one layer.
const LAYERLESS_AI: &str = "%!PS-Adobe-3.0 EPSF-3.0\n%%Creator: a script\n%%BoundingBox: 0 0 200 100\n%AI3_Cropmarks: 0 0 200 100\n%%EndComments\n\
    %%BeginProlog\n/m {moveto} def /L {lineto} def /f {closepath fill} def /S {stroke} def /A {pop} def /k {setcmykcolor} def /u {} def /U {} def\n\
    %%EndProlog\n%%BeginSetup\n%%EndSetup\n1 A\n0 0 0 1 k\n10 10 m\n90 10 L\n90 90 L\nf\n0 A\nu\n110 10 m\n190 90 L\n150 90 L\nf\nU\n\
    %%PageTrailer\nshowpage\n%%Trailer\n%%EOF\n";

/// The fixture, as it is, opens through the reader: its first path keeps its lock.
#[test]
fn the_layerless_fixture_opens_through_the_reader() {
    let l = vectorcraft_engine::cmd::fileio::load("x.ai", LAYERLESS_AI.as_bytes()).unwrap();
    assert_eq!(l.doc.layers.len(), 1, "{:?}", l.warnings);
    let art = l.doc.layers[0].children().unwrap();
    assert_eq!(art.len(), 2, "{:?}", l.warnings);
    assert!(art[0].locked && !art[1].locked);
}

proptest! {
    #![proptest_config(config())]

    /// A `.ai` in the legacy format without layers, damaged: read through the reader, as its page,
    /// or refused.
    #[test]
    fn layerless_ai_never_panics(cut in 0usize..800, edits in prop::collection::vec((0usize..800, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '(', ')', '/', '{', '}', '%', 'u', 'U', 'L', 'f', 'A', 'k'])), 0..12)) {
        let text = mutate_text(LAYERLESS_AI, cut, &edits);
        survive("mutated layerless .ai", || vectorcraft_engine::cmd::fileio::load("x.ai", text.as_bytes()).ok().map(|l| l.doc))?;
    }
}

/// A pattern's definition in the version 7 form (§3.4): a layer whose `_ &` fills the tile's box,
/// and one whose tile string has CR-separated lines, comments and escapes.
const LEGACY_PATTERN: &str = "%AI3_BeginPattern: (P)\n(P) 0 0 10 10 [\n%AI3_Tile\n(0 O 0 R 0 0 0 0.5 k) @ _ &\n\
    (0 O 1 R 0 0 0 1 \\(Ink\\) 0 X) @ (\r%AI6_BeginPatternLayer\r1 w u\r0 5 m\\r10 5 L\rS\rU\r%AI6_EndPatternLayer\r) &\n] E\n%AI3_EndPattern\n";

/// [`NAMED_PROLOG_AI`] with `pattern` in its setup, filling the square.
fn legacy_pattern_ai(pattern: &str) -> String {
    NAMED_PROLOG_AI
        .replace("%AI5_Begin_NonPrinting", &format!("{pattern}%AI5_Begin_NonPrinting"))
        .replace("\n10 10 m\n", "\n(P) 5 5 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n10 10 m\n")
        .replace("10 10 L\nS\n", "10 10 L\nB\n")
}

/// The fixture, as it is, opens with the pattern filling the square.
#[test]
fn the_legacy_pattern_fixture_opens_with_its_pattern() {
    let l = vectorcraft_engine::cmd::fileio::load("x.ai", legacy_pattern_ai(LEGACY_PATTERN).as_bytes()).unwrap();
    assert_eq!(l.doc.pattern("P").map(|p| p.art.len()), Some(2), "{:?}", l.warnings);
    let square = &l.doc.layers[0].children().unwrap()[0];
    assert!(matches!(square.appearance.fill().map(|f| &f.paint), Some(vectorcraft_color::Paint::Pattern { .. })));
}

proptest! {
    #![proptest_config(config())]

    /// That `.ai` with its pattern's definition damaged (its strings, `@`, `&`, `_`, the list's
    /// brackets and its `E`): read, without the pattern where it can't be read whole, or refused.
    #[test]
    fn ps_form_ai_legacy_pattern_never_panics(cut in 0usize..500, edits in prop::collection::vec((0usize..500, prop::sample::select(vec!['0', '9', '-', '.', ' ', '\n', '\r', '(', ')', '\\', '[', ']', '%', '@', '&', '_', 'E', 'u', 'U', 'q', 'Q', 'm', 'f', 'S', 'k', 'X'])), 0..12)) {
        let text = legacy_pattern_ai(&mutate_text(LEGACY_PATTERN, cut, &edits));
        survive("PostScript-form .ai with a damaged version 7 pattern", || vectorcraft_engine::cmd::fileio::load("x.ai", text.as_bytes()).ok().map(|l| l.doc))?;
    }
}

/// Area type in the version 7 form (§15.5.3) whose story flows through two linked areas: a small
/// one, and one a quarter turn round, each area's characters after its `TP`.
const LINKED_AREAS: &str = "1 To\n1 0 0 1 120 90 0 Tp\n120 60 m\n120 90 L\n180 90 L\n180 60 L\n120 60 L\nn\nTP\n0 Tr\n0 g\n/_Helvetica 9 Tf\n\
    (one two three four five six seven eight nine ten ) Tx\n0 1 -1 0 190 10 0 Tp\n130 10 m\n130 55 L\n190 55 L\n190 10 L\n130 10 L\nn\nTP\n\
    (eleven twelve thirteen fourteen fifteen) Tx\n(\\r) TX\nTO\n";

/// [`NAMED_PROLOG_AI`] with `text` on its layer.
fn linked_areas_ai(text: &str) -> String {
    NAMED_PROLOG_AI.replace("u\n0.5 g\n", &format!("{text}u\n0.5 g\n"))
}

/// The fixture, as it is, opens with its area type as two threaded frames, the story flowing from
/// the first into the second.
#[test]
fn the_linked_areas_fixture_opens_as_threaded_frames() {
    let l = vectorcraft_engine::cmd::fileio::load("x.ai", linked_areas_ai(LINKED_AREAS).as_bytes()).unwrap();
    let [thread] = &l.doc.text_threads[..] else { panic!("one thread: {:?} {:?}", l.doc.text_threads, l.warnings) };
    let texts: Vec<String> = thread
        .iter()
        .map(|id| match &l.doc.node(*id).unwrap().kind {
            vectorcraft_doc::NodeKind::Text(t) => t.plain_text(),
            k => panic!("type: {k:?}"),
        })
        .collect();
    let [first, second] = texts.as_slice() else { panic!("two frames: {texts:?}") };
    assert!(!first.is_empty() && !second.is_empty(), "the story flows on into the second frame: {texts:?}");
    assert_eq!(format!("{first}{second}"), "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen");
}

proptest! {
    #![proptest_config(config())]

    /// That `.ai` with its linked areas damaged (their text paths, frames and characters): read, or
    /// refused.
    #[test]
    fn ps_form_ai_linked_areas_never_panic(cut in 0usize..400, edits in prop::collection::vec((0usize..400, prop::sample::select(vec!['0', '1', '-', '.', ' ', '\n', '(', ')', '\\', 'T', 'p', 'P', 'O', 'o', 'x', 'X', 'm', 'L', 'n'])), 0..12)) {
        let text = linked_areas_ai(&mutate_text(LINKED_AREAS, cut, &edits));
        survive("PostScript-form .ai with damaged linked areas", || vectorcraft_engine::cmd::fileio::load("x.ai", text.as_bytes()).ok().map(|l| l.doc))?;
    }
}






/// The testkit's sample editing data, compressed as older `.ai` files compress it.
fn ai_sample_compressed() -> Vec<u8> {
    [&b"%AI12_CompressedData"[..], &deflated(&vectorcraft_testkit::ai::sample_data())].concat()
}

/// `data` compressed (zlib).
fn deflated(data: &[u8]) -> Vec<u8> {
    use std::io::Write as _;
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

proptest! {
    #![proptest_config(config())]








    /// The testkit's sample damaged, as it is and compressed, in EPS, `.ai` and PDF files.
    #[test]
    fn ai_mutated_sample_never_panics(cut in 0usize..4_000, edits in prop::collection::vec((0usize..4_000, any::<u8>()), 0..12)) {
        use vectorcraft_testkit::ai;
        static PLAIN: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        static PACKED: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        for (what, sample) in [("editing data", PLAIN.get_or_init(ai::sample_data)), ("compressed editing data", PACKED.get_or_init(ai_sample_compressed))] {
            let mut bytes = sample.clone();
            for &(at, b) in &edits {
                let n = bytes.len();
                bytes[at % n] = b;
            }
            bytes.truncate(cut.max(4));
            let file = ai::ai(&bytes, ai::page_pdf());
            survive(what, || vectorcraft_engine::cmd::fileio::load("x.ai", &file).ok().map(|l| l.doc))?;
            let file = ai::pdf(&bytes, ai::page_pdf());
            survive(what, || vectorcraft_engine::cmd::fileio::load("x.pdf", &file).ok().map(|l| l.doc))?;
            let file = ai::eps(&bytes, ai::page_ps());
            survive(what, || vectorcraft_engine::cmd::fileio::load("x.eps", &file).ok().map(|l| l.doc))?;
        }
    }

}


// ---------- font files a folder search reads ----------

/// A small font file holding what a folder search reads: an outline table's tag, the `name` and
/// `OS/2` tables of the bundled Source Sans 3 renamed "Findme Sans 3", and an `fvar` table with a
/// weight axis and three named instances (named by name ids 2, 1 and 4).
fn search_font() -> Vec<u8> {
    let font = vectorcraft_testkit::fonts::renamed("Findme Sans 3");
    let be32 = |at: usize| u32::from_be_bytes(font[at..at + 4].try_into().unwrap()) as usize;
    let table = |tag: &[u8; 4]| {
        let n = u16::from_be_bytes([font[4], font[5]]) as usize;
        let r = (0..n).map(|i| 12 + 16 * i).find(|&r| &font[r..r + 4] == tag).unwrap();
        font[be32(r + 8)..be32(r + 8) + be32(r + 12)].to_vec()
    };
    // fvar 1.0: the axis array at 16, one axis of 20 bytes, three instances of 8.
    let mut fvar = vec![];
    for v in [1u16, 0, 16, 2, 1, 20, 3, 8] {
        fvar.extend(v.to_be_bytes());
    }
    fvar.extend(b"wght");
    for v in [100i32, 400, 900] {
        fvar.extend((v << 16).to_be_bytes());
    }
    fvar.extend([0, 0, 1, 0]);
    for (name, weight) in [(2u16, 300i32), (1, 600), (4, 900)] {
        fvar.extend(name.to_be_bytes());
        fvar.extend(0u16.to_be_bytes());
        fvar.extend((weight << 16).to_be_bytes());
    }
    let tables: [(&[u8; 4], Vec<u8>); 4] = [(b"OS/2", table(b"OS/2")), (b"fvar", fvar), (b"glyf", vec![0; 4]), (b"name", table(b"name"))];
    let mut out = 0x0001_0000_u32.to_be_bytes().to_vec();
    for v in [tables.len() as u16, 0, 0, 0] {
        out.extend(v.to_be_bytes());
    }
    let mut offset = 12 + 16 * tables.len();
    for (tag, data) in &tables {
        out.extend(*tag);
        for v in [0, offset as u32, data.len() as u32] {
            out.extend(v.to_be_bytes());
        }
        offset += data.len().next_multiple_of(4);
    }
    for (_, data) in &tables {
        out.extend(data);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    out
}

/// `font` as a collection of `faces % 5` faces that all read it, whose header gives `faces` as the
/// count.
fn search_collection(font: &[u8], faces: u32) -> Vec<u8> {
    let k = (faces % 5) as usize;
    let base = 12 + 4 * k;
    let mut out = b"ttcf".to_vec();
    out.extend(0x0001_0000_u32.to_be_bytes());
    out.extend(faces.to_be_bytes());
    for _ in 0..k {
        out.extend((base as u32).to_be_bytes());
    }
    // The tables' offsets count from the start of the collection.
    let mut f = font.to_vec();
    let n = u16::from_be_bytes([f[4], f[5]]) as usize;
    for i in 0..n {
        let at = 12 + 16 * i + 8;
        let offset = u32::from_be_bytes(f[at..at + 4].try_into().unwrap()) + base as u32;
        f[at..at + 4].copy_from_slice(&offset.to_be_bytes());
    }
    out.extend(f);
    out
}

/// The fonts a search for "Findme Sans 3" looks for: the face, and a style of the family as if the
/// family were installed.
fn findme_wanted() -> vectorcraft_text::WantedFonts {
    use vectorcraft_text::WantedFont;
    vectorcraft_text::WantedFonts::new(&[
        WantedFont { family: "Findme Sans 3".into(), style: "Regular".into(), installed: None },
        WantedFont { family: "FINDME SANS 3".into(), style: "Black".into(), installed: Some("Findme Sans 3".into()) },
        WantedFont { family: "SourceSans3-Regular".into(), style: "Bold".into(), installed: None },
    ])
}

/// Whether reading `bytes` as a font file a search reads panics.
fn search_reads(what: &str, bytes: &[u8]) -> Result<Vec<usize>, TestCaseError> {
    let dir = vectorcraft_testkit::TempDir::new("font-search-fuzz");
    let path = dir.join("font.ttc");
    std::fs::write(&path, bytes).unwrap();
    catch_quiet(|| findme_wanted().provided_by(&path)).map_err(|e| TestCaseError::fail(format!("{what}: a search reading the file panicked: {e}")))
}

/// The samples the fuzz properties mutate reach the matching: each provides the face it names.
#[test]
fn the_search_font_samples_are_read_whole() {
    let font = search_font();
    assert_eq!(search_reads("font", &font).unwrap(), [0, 2]);
    assert_eq!(search_reads("collection", &search_collection(&font, 3)).unwrap(), [0, 2]);
    // Two faces, and a header that gives seven.
    assert_eq!(search_reads("collection claiming more faces", &search_collection(&font, 7)).unwrap(), [0, 2]);
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn font_files_a_search_reads_never_panic(
        faces in prop::option::of(any::<u32>()),
        cut in prop::option::of(0usize..12_000),
        edits in prop::collection::vec((0usize..12_000, any::<u8>()), 0..12),
    ) {
        let font = search_font();
        let mut bytes = match faces {
            Some(n) => search_collection(&font, n),
            None => font,
        };
        for (at, b) in edits {
            let n = bytes.len();
            bytes[at % n] = b;
        }
        if let Some(cut) = cut {
            bytes.truncate(cut);
        }
        search_reads("mutated font file", &bytes)?;
    }
}

// ---------- scripts checked without running them ----------

/// A script written for these tests, with invented names: directives in both forms, member chains,
/// files, a dialog, menu commands, E4X, an operator and a message to another app.
const SCRIPT_SAMPLE: &str = concat!(
    "#target acme_2-12-en_US\n",
    "//@include \"lib/util.jsxinc\"\n",
    "#includepath \"lib;shared\"\n",
    "#targetengine session\n",
    "var doc = app.activeDocument;\n",
    "var w = new Window(\"dialog { text: 'Tidy' }\");\n",
    "var f = File.saveDialog('Save'); f.open('w'); f.write(doc.acmeName); f.close();\n",
    "app.executeMenuCommand('acmecopyall');\n",
    "var x = <root><item id=\"1\">{doc.acmeName}</item></root>;\n",
    "for each (var i in x..item) { $.writeln(i.@id); }\n",
    "Vec.prototype['+'] = function (o) { return o; };\n",
    "var bt = new BridgeTalk(); bt.target = 'acme'; bt.send();\n",
    "if (item.typename == 'AcmeShape') { alert(/a[/]b/g.test(s) ? 'yes' : \"no\"); }\n",
);

/// The pieces generated scripts are made of.
const SCRIPT_PIECES: &[&str] = &[
    "#target acme",
    "//@include 'a.jsxinc'",
    "#includepath 'x;..'",
    "#pragma",
    "app",
    "activeDocument",
    "selection",
    "var doc = ",
    ".",
    ".acmeItems",
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    ";",
    "=",
    "'str'",
    "\"open",
    "/re[/]x/g",
    "/",
    "<a b=\"{v}\">",
    "</a>",
    "<!--",
    ".@",
    "..",
    "`t",
    "=>",
    "\n",
    "\r",
    "\u{2028}",
    "@JSXBIN@",
    "executeMenuCommand",
    "new File(p)",
    "Window(",
    "import",
    "/*",
    "\\",
    "0x1F",
    " ",
];

/// `script.check` with `p`, which mustn't panic, and whose lists keep to their limit.
fn checked(what: &str, p: Value) -> Result<(), TestCaseError> {
    let r = catch_quiet(|| no_panic(vectorcraft_engine::Session::new().execute("script.check", &p)))
        .map_err(|e| TestCaseError::fail(format!("{what}: script.check panicked: {e}")))?;
    if let Ok(r) = r {
        for list in [&r["targets"], &r["directives"], &r["includes"], &r["problems"], &r["dom"]["members"], &r["menuCommands"]["strings"]] {
            prop_assert!(list.as_array().is_some_and(|l| l.len() <= 1000), "{}: {}", what, list);
        }
    }
    Ok(())
}

/// The sample the properties mutate reaches every part of the check.
#[test]
fn the_script_sample_is_checked_whole() {
    let r = vectorcraft_engine::Session::new().execute("script.check", &json!({"data": SCRIPT_SAMPLE})).unwrap();
    assert_eq!(r["kind"], "script");
    assert_eq!(r["needs"], json!(["language", "dom", "files", "menuCommands", "dialogs", "hostOnly"]));
    assert_eq!(r["problems"], json!([]));
}

proptest! {
    #![proptest_config(config())]

    #[test]
    fn script_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..2_000), compiled in any::<bool>()) {
        let name = if compiled { "garbage.jsxbin" } else { "garbage.jsx" };
        checked("script garbage", json!({"dataBase64": vectorcraft_format::base64_encode(&bytes), "name": name}))?;
    }

    #[test]
    fn generated_scripts_never_panic(pieces in prop::collection::vec(prop::sample::select(SCRIPT_PIECES.to_vec()), 0..120)) {
        checked("generated script", json!({"data": pieces.concat()}))?;
    }

    #[test]
    fn mutated_scripts_never_panic(cut in 0usize..1_200, edits in prop::collection::vec((0usize..1_200, any::<u8>()), 0..12)) {
        let mut bytes = SCRIPT_SAMPLE.as_bytes().to_vec();
        for (at, b) in edits {
            let n = bytes.len();
            bytes[at % n] = b;
        }
        bytes.truncate(cut);
        checked("mutated script", json!({"dataBase64": vectorcraft_format::base64_encode(&bytes), "name": "sample.jsx"}))?;
    }
}

// ---------- deep nesting ----------
//
// The tree is walked by recursion, so a file nested deeply enough overflowed the stack: an abort,
// which nothing catches. Every reader caps what it reads, and no document is deeper than
// `MAX_DEPTH`. These run on a thread with the stack of a Windows main thread (1 MB), where an
// overflow ends the test run.

use vectorcraft_doc::MAX_DEPTH;

/// `f` on a thread with a 1 MB stack.
fn on_a_small_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new().stack_size(1 << 20).spawn(f).unwrap().join().unwrap();
}

/// Open `bytes` as the file `name`: an error, or a document within the cap that renders and
/// exports. Returns the document and the import's warnings.
fn open_deep(name: &str, bytes: &[u8]) -> Option<(Document, Vec<String>)> {
    let loaded = vectorcraft_engine::cmd::fileio::load(name, bytes).ok()?;
    assert!(!loaded.doc.too_deep(None), "{name}: deeper than the cap");
    let doc = loaded.doc.clone();
    survive(name, move || Some(doc)).unwrap();
    Some((loaded.doc, loaded.warnings))
}

fn merged(warnings: &[String]) -> bool {
    warnings.iter().any(|w| w.contains("nested more than"))
}

fn paths_in(doc: &Document) -> usize {
    let mut paths = 0;
    doc.walk(|n| paths += usize::from(n.path_data().is_some()));
    paths
}

const SVG_NS: &str = "xmlns=\"http://www.w3.org/2000/svg\"";

/// `n` groups in one another, each named (so it is kept) and with a rectangle beside the next.
fn nested_svg(n: usize) -> String {
    let open: String = (0..n).map(|i| format!("<g id=\"g{i}\"><rect width=\"5\" height=\"5\"/>")).collect();
    format!("<svg {SVG_NS} width=\"100\" height=\"100\">{open}<rect width=\"10\" height=\"10\"/>{}</svg>", "</g>".repeat(n))
}

#[test]
fn deeply_nested_svg_is_refused_or_merged() {
    on_a_small_stack(|| {
        // Within what the reader takes: opened, the groups past the cap merged into the last.
        let (doc, warnings) = open_deep("deep.svg", nested_svg(40).as_bytes()).expect("40 groups open");
        assert!(merged(&warnings), "{warnings:?}");
        assert_eq!(paths_in(&doc), 41, "no art is lost");
        let (_, warnings) = open_deep("shallow.svg", nested_svg(20).as_bytes()).expect("20 groups open");
        assert!(!merged(&warnings), "{warnings:?}");
        // As deep as the reader goes (on a stack of its own): opened and merged the same.
        let most = vectorcraft_svg::MAX_ELEMENT_DEPTH;
        let (doc, warnings) = open_deep("deepest.svg", nested_svg(most - 2).as_bytes()).expect("opens at the reader's cap");
        assert!(merged(&warnings), "{warnings:?}");
        assert_eq!(paths_in(&doc), most - 1, "no art is lost");
        // Past it: an error that says why, whatever the depth.
        for n in [most, 2 * most, 200_000] {
            let e = vectorcraft_engine::cmd::fileio::load("deep.svg", nested_svg(n).as_bytes()).err().expect("refused").to_string();
            assert!(e.contains(&format!("nested more than {most} deep")), "{n}: {e}");
        }
        // Unnamed groups of one object aren't kept, but usvg walks them all the same.
        let plain = format!(
            "<svg {SVG_NS} width=\"9\" height=\"9\">{}<rect width=\"1\" height=\"1\"/>{}</svg>",
            "<g>".repeat(2 * most),
            "</g>".repeat(2 * most)
        );
        assert!(open_deep("plain.svg", plain.as_bytes()).is_none());
        // A chain of `<use>`: shallow XML, a deep tree.
        let links: String = (0..3000).map(|i| format!("<g id=\"u{i}\"><use href=\"#u{}\"/></g>", i + 1)).collect();
        let chain = format!(
            "<svg {SVG_NS} width=\"9\" height=\"9\"><defs>{links}<rect id=\"u3000\" width=\"1\" height=\"1\"/></defs><use href=\"#u0\"/></svg>"
        );
        assert!(open_deep("chain.svg", chain.as_bytes()).is_none());
        // SVG images in SVG images, read on the same stack: each too deep, of a type that is read
        // as SVG too (`text/plain`), or each within the cap but not all together (#3562). What is
        // past the cap is left out.
        let images = [nested_images(3, 1000), nested_images(12, most - 4)];
        for (i, file) in images.iter().enumerate() {
            let (_, warnings) = open_deep("images.svg", file.as_bytes()).expect("opens");
            assert!(warnings.iter().any(|w| w.contains("SVG images were left out")), "{i}: {warnings:?}");
        }
        // Placed and pasted, as opened.
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width": 300, "height": 200})).unwrap();
        for svg in [nested_svg(40), nested_svg(2 * most)].iter().chain(&images) {
            let p = json!({"name": "deep.svg", "dataBase64": vectorcraft_format::base64_encode(svg.as_bytes())});
            let _ = no_panic(s.execute("file.place", &p));
            let _ = no_panic(s.execute("clipboard.importSvg", &json!({"svg": svg})));
            let _ = no_panic(s.execute("edit.paste", &json!({})));
            assert!(!s.doc().unwrap().doc.too_deep(None));
        }
    });
}

/// An SVG showing a chain of `levels` SVG images from `text/plain` data URLs, each `n` groups
/// deep around the next.
fn nested_images(levels: usize, n: usize) -> String {
    let image = |svg: &str| {
        format!("<image href=\"data:text/plain;base64,{}\" width=\"9\" height=\"9\"/>", vectorcraft_format::base64_encode(svg.as_bytes()))
    };
    let mut svg = nested_svg(0);
    for _ in 0..levels {
        svg = format!("<svg {SVG_NS} width=\"9\" height=\"9\">{}{}{}</svg>", "<g>".repeat(n), image(&svg), "</g>".repeat(n));
    }
    format!("<svg {SVG_NS} width=\"9\" height=\"9\">{}</svg>", image(&svg))
}

/// A page whose content is `content`, with `resources`.
fn deep_pdf(content: &str, resources: &str) -> Vec<u8> {
    handmade_pdf(content, resources, "[0 0 100 100]")
}

/// A stream object with the entries `dict`, its `data` compressed.
fn packed(dict: &str, data: &str) -> Vec<u8> {
    packed_as_is(dict, &deflated(data.as_bytes()))
}

/// A stream object with the entries `dict`, its data `z` compressed already.
fn packed_as_is(dict: &str, z: &[u8]) -> Vec<u8> {
    // (A length the test gives stands.)
    let length = if dict.contains("/Length") { String::new() } else { format!(" /Length {}", z.len()) };
    [format!("<< {dict} /Filter /FlateDecode{length} >>\nstream\n").as_bytes(), z, b"\nendstream"].concat()
}

/// A page whose content is `content`, with `resources` and the objects `more` (numbered from 5).
fn pdf_with(content: &str, resources: &str, more: &[Vec<u8>]) -> Vec<u8> {
    handmade_pdf_with(content, resources, "[0 0 100 100]", more)
}

/// Arrays in arrays, and dictionaries in dictionaries, far deeper than any reader goes: as the
/// operands of content.
fn nested_operands() -> [String; 2] {
    let (n, d) = (100_000, 50_000);
    [format!("BT {}{} TJ ET 0 0 5 5 re f", "[".repeat(n), "]".repeat(n)), format!("/P {}{} DP 0 0 5 5 re f", "<< /A ".repeat(d), ">> ".repeat(d))]
}

/// A page drawing through the compressed stream that `container` makes of each of
/// [`nested_operands`] (object 5), one file each.
fn nested_in(content: &str, resources: &str, container: impl Fn(&str) -> Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    nested_operands().iter().map(|deep| pdf_with(content, resources, &container(deep))).collect()
}

const FORM: &str = "/Type /XObject /Subtype /Form /BBox [0 0 100 100]";

/// A Type 3 font (object 5) whose glyph `a` is object 6.
fn type3_font(resources: &str) -> Vec<u8> {
    format!(
        "<< /Type /Font /Subtype /Type3 /FontBBox [0 0 10 10] /FontMatrix [0.1 0 0 0.1 0 0] /CharProcs << /a 6 0 R >> \
         /Encoding << /Type /Encoding /Differences [97 /a] >> /FirstChar 97 /LastChar 97 /Widths [10] /Resources {resources} >>"
    )
    .into_bytes()
}

/// A page whose box is object 6, kept with object 7 (a number) in a compressed object stream
/// (object 5, with the entries `stream`): a file with a cross-reference stream, as it must be.
fn pdf_with_object_stream(boxed: &str, stream: &str) -> Vec<u8> {
    let content = "0 0 5 5 re f";
    let inside = format!("{boxed} 77");
    let index = format!("6 0 7 {} ", boxed.len() + 1);
    let objs = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox 6 0 R /Contents 4 0 R /Resources << >> >>".to_vec(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len() + 1).into_bytes(),
        packed(&format!("/Type /ObjStm /N 2 /First {} {stream}", index.len()), &format!("{index}{inside}")),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    // One entry an object: its kind, then where it is (an offset, or the stream it is in and its
    // place there).
    let mut entries = vec![[0u8, 0, 0, 0, 0, 255, 255]];
    let at = |kind: u8, a: usize, b: u16| {
        let (a, b) = ((a as u32).to_be_bytes(), b.to_be_bytes());
        [kind, a[0], a[1], a[2], a[3], b[0], b[1]]
    };
    for (i, o) in objs.iter().enumerate() {
        entries.push(at(1, out.len(), 0));
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(o);
        out.extend_from_slice(b"\nendobj\n");
    }
    entries.extend([at(2, 5, 0), at(2, 5, 1), at(1, out.len(), 0)]);
    let xref = out.len();
    out.extend_from_slice(format!("8 0 obj\n<< /Type /XRef /Size 9 /Root 1 0 R /W [1 4 2] /Length {} >>\nstream\n", entries.len() * 7).as_bytes());
    out.extend_from_slice(&entries.concat());
    out.extend_from_slice(format!("\nendstream\nendobj\nstartxref\n{xref}\n%%EOF\n").as_bytes());
    out
}

/// `n` objects from object 5 on, each made by `link` of the number of the next, then `last`.
fn chain(n: usize, link: impl Fn(usize) -> String, last: &str) -> Vec<Vec<u8>> {
    (0..n).map(|i| link(i + 6).into_bytes()).chain([last.as_bytes().to_vec()]).collect()
}

/// A page painting with the shading whose function is object 5.
fn pdf_with_function(more: &[Vec<u8>]) -> Vec<u8> {
    let shading = "/Shading << /S << /ShadingType 2 /ColorSpace /DeviceRGB /Coords [0 0 100 0] /Function 5 0 R >> >>";
    pdf_with("/S sh", &format!("<< {shading} >>"), more)
}

/// A page filling in the colour space that is object 5.
fn pdf_with_color_space(more: &[Vec<u8>]) -> Vec<u8> {
    pdf_with("/C cs 0 sc 0 0 5 5 re f /C CS 0 SC 0 0 5 5 re S", "<< /ColorSpace << /C 5 0 R >> >>", more)
}

/// Every hostile PDF here opens as an error or as a document within the cap, on a small stack.
fn open_each(name: &str, files: impl IntoIterator<Item = Vec<u8>> + Send + 'static) {
    let name = name.to_string();
    on_a_small_stack(move || {
        for (i, bytes) in files.into_iter().enumerate() {
            open_deep(&format!("{name}-{i}.pdf"), &bytes);
            // As `.ai` (the PDF part of one) and placed, too.
            open_deep(&format!("{name}-{i}.ai"), &bytes);
            let mut s = vectorcraft_engine::Session::new();
            s.execute("file.new", &json!({"width": 300, "height": 200})).unwrap();
            let _ = no_panic(s.execute("file.place", &json!({"name": "deep.pdf", "dataBase64": vectorcraft_format::base64_encode(&bytes)})));
            assert!(!s.doc().unwrap().doc.too_deep(None));
        }
    });
}

// Nesting where only the reader of the decoded data sees it: inside each kind of compressed
// stream a page draws through.

#[test]
fn pdf_nesting_inside_a_compressed_form() {
    open_each("form", nested_in("/F Do", "<< /XObject << /F 5 0 R >> >>", |deep| vec![packed(FORM, deep)]));
}

#[test]
fn pdf_nesting_inside_a_compressed_pattern() {
    let tile = "/PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 10 10] /XStep 10 /YStep 10 /Resources << >>";
    open_each("pattern", nested_in("/Pattern cs /P scn 0 0 50 50 re f", "<< /Pattern << /P 5 0 R >> >>", |deep| vec![packed(tile, deep)]));
}

#[test]
fn pdf_nesting_inside_a_compressed_glyph() {
    let glyph = |deep: &str| vec![type3_font("<< >>"), packed("", &format!("10 0 d0 {deep}"))];
    open_each("glyph", nested_in("BT /F1 12 Tf 10 10 Td (aa) Tj ET", "<< /Font << /F1 5 0 R >> >>", glyph));
}

#[test]
fn pdf_nesting_inside_a_compressed_soft_mask() {
    let group = format!("{FORM} /Group << /S /Transparency /CS /DeviceGray >>");
    let state = "<< /ExtGState << /G << /Type /ExtGState /SMask << /S /Luminosity /G 5 0 R >> >> >> >>";
    open_each("mask", nested_in("/G gs 0 0 50 50 re f", state, move |deep| vec![packed(&group, deep)]));
}

#[test]
fn pdf_nesting_inside_a_compressed_annotation() {
    let annots = "<< >> /Annots [<< /Type /Annot /Subtype /Square /Rect [0 0 50 50] /AP << /N 5 0 R >> >>]";
    open_each("annotation", nested_in("0 0 5 5 re f", annots, |deep| vec![packed(FORM, deep)]));
}

#[test]
fn pdf_nesting_inside_compressed_page_content() {
    let files = nested_operands().map(|deep| {
        let mut objs: Vec<Vec<u8>> = ["<< /Type /Catalog /Pages 2 0 R >>", "<< /Type /Pages /Kids [3 0 R] /Count 1 >>"].map(Vec::from).into();
        objs.push(b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents [4 0 R 4 0 R] /Resources << >> >>".to_vec());
        objs.push(packed("", &deep));
        pdf_of(&objs)
    });
    open_each("content", files);
}

#[test]
fn pdf_nesting_inside_an_object_stream() {
    let (n, d) = (100_000, 50_000);
    open_each(
        "objects",
        [
            // A plain one first: the file is read as it should be.
            pdf_with_object_stream("[0 0 100 100]", ""),
            pdf_with_object_stream(&format!("{}0 0 100 100{}", "[".repeat(n), "]".repeat(n)), ""),
            pdf_with_object_stream(&format!("{}{}", "<< /A ".repeat(d), ">> ".repeat(d)), ""),
            // Its length is an object kept in the stream itself.
            pdf_with_object_stream("[0 0 100 100]", "/Length 7 0 R"),
            // It extends itself.
            pdf_with_object_stream("[0 0 100 100]", "/Extends 5 0 R"),
        ],
    );
    let plain = vectorcraft_engine::cmd::fileio::load("plain.pdf", &pdf_with_object_stream("[0 0 120 80]", "")).expect("an object stream is read");
    assert_eq!(plain.doc.artboards[0].rect.width(), 120.0);
}

// What refers to itself, or to the next of its kind without end.

#[test]
fn pdf_page_trees_that_hold_themselves() {
    let page = |parent: usize| format!("<< /Type /Page /Parent {parent} 0 R /MediaBox [0 0 100 100] /Contents 4 0 R /Resources << >> >>");
    let content = "<< /Length 13 >>\nstream\n0 0 5 5 re f\nendstream";
    let own = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [2 0 R 3 0 R 2 0 R] /Count 1 >>".to_string(),
        page(2),
        content.to_string(),
    ];
    // Thousands of nodes, each holding the next; the page is the last.
    let n = 30_000;
    let mut deep = vec!["<< /Type /Catalog /Pages 5 0 R >>".to_string(), "null".to_string(), "null".to_string(), content.to_string()];
    deep.extend((0..n).map(|i| format!("<< /Type /Pages /Kids [{} 0 R] /Count 1 >>", i + 6)));
    deep.push(page(n + 4));
    // Two nodes holding each other.
    let pair = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [5 0 R] /Count 1 >>".to_string(),
        page(5),
        content.to_string(),
        "<< /Type /Pages /Kids [2 0 R 3 0 R] /Count 1 >>".to_string(),
    ];
    let bytes = |objs: &[String]| pdf_of(&objs.iter().map(|o| o.clone().into_bytes()).collect::<Vec<_>>());
    open_each("pages", [bytes(&own), bytes(&deep), bytes(&pair)]);
}

#[test]
fn pdf_functions_that_hold_themselves() {
    let stitch = |next: usize| format!("<< /FunctionType 3 /Domain [0 1] /Functions [{next} 0 R] /Bounds [] /Encode [0 1] >>");
    let ramp = "<< /FunctionType 2 /Domain [0 1] /C0 [1 0 0] /C1 [0 0 1] /N 1 >>";
    let calculator = "/FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1]";
    let n = 100_000;
    open_each(
        "functions",
        [
            pdf_with_function(&[stitch(5).into_bytes()]),
            pdf_with_function(&[stitch(6).into_bytes(), stitch(5).into_bytes()]),
            pdf_with_function(&chain(20_000, stitch, ramp)),
            // A calculator function of procedures in procedures.
            pdf_with_function(&[packed(calculator, &format!("{}{}", "{".repeat(n), "}".repeat(n)))]),
            pdf_with_function(&[packed(calculator, &format!("{{ {} 0 0 0 {} }}", "true {".repeat(n), "} if ".repeat(n)))]),
        ],
    );
}

#[test]
fn pdf_color_spaces_that_hold_themselves() {
    let indexed = |next: usize| format!("[/Indexed {next} 0 R 1 <000000ffffff>]");
    let tint = "<< /FunctionType 2 /Domain [0 1] /C0 [1 1 1] /C1 [0 0 0] /N 1 >>";
    let profile = |alternate: &str| format!("<< /N 3 /Alternate {alternate} /Length 4 >>\nstream\njunk\nendstream").into_bytes();
    open_each(
        "spaces",
        [
            pdf_with_color_space(&[indexed(5).into_bytes()]),
            pdf_with_color_space(&[indexed(6).into_bytes(), indexed(5).into_bytes()]),
            pdf_with_color_space(&chain(20_000, indexed, "/DeviceRGB")),
            pdf_with_color_space(&[b"[/Separation /Ink 5 0 R 6 0 R]".to_vec(), tint.as_bytes().to_vec()]),
            pdf_with_color_space(&[b"[/DeviceN [/Ink] 5 0 R 6 0 R]".to_vec(), tint.as_bytes().to_vec()]),
            pdf_with_color_space(&[b"[/Pattern 5 0 R]".to_vec()]),
            pdf_with_color_space(&[b"[/ICCBased 6 0 R]".to_vec(), profile("5 0 R")]),
            pdf_with_color_space(&[b"[/ICCBased 6 0 R]".to_vec(), profile("[/ICCBased 6 0 R]")]),
            // As the space of a shading and of an image.
            pdf_with(
                "/S sh",
                "<< /Shading << /S << /ShadingType 2 /ColorSpace 5 0 R /Coords [0 0 9 0] /Function 6 0 R >> >> >>",
                &[indexed(5).into_bytes(), tint.as_bytes().to_vec()],
            ),
            pdf_with(
                "q 50 0 0 50 0 0 cm /I Do Q",
                "<< /XObject << /I 6 0 R >> >>",
                &[
                    indexed(5).into_bytes(),
                    b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace 5 0 R /Length 1 >>\nstream\n0\nendstream"
                        .to_vec(),
                ],
            ),
        ],
    );
}

/// Spaces past what a stream may decode to (256 MB), compressed (about 260 kB).
fn spaces_past_the_cap() -> Vec<u8> {
    use std::io::Write as _;
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    for _ in 0..257 {
        e.write_all(&[b' '; 1 << 20]).unwrap();
    }
    e.finish().unwrap()
}

/// [`calculator`] of [`spaces_past_the_cap`], made once.
fn spaces_past_the_cap_once() -> &'static str {
    static FUNCTION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FUNCTION.get_or_init(|| calculator(&spaces_past_the_cap(), ""))
}

/// A calculator function with the deflated program `z` and the decode parameters `dp`, to write
/// inside another object. Its data is in hexadecimal digits with spaces between them, so the
/// object around it still reads.
fn calculator(z: &[u8], dp: &str) -> String {
    let data: String = z.iter().map(|b| format!("{:X} {:X} ", b >> 4, b & 15)).collect();
    let dict = format!("/FunctionType 4 /Domain [0 1] /Range [0 1 0 1 0 1] /Filter [/ASCIIHexDecode /FlateDecode] {dp}");
    format!("<< {dict} /Length {} >>\nstream\n{data}\nendstream", data.len())
}

// Streams the reader would fail on (#1700, #1701): it applies a predictor before checking its
// parameters (`Columns 0` divides by zero, a huge `Columns` allocates a row that large), and
// inflates without a ceiling.

#[test]
fn pdf_streams_the_reader_would_fail_on_are_refused() {
    let zero = "/DecodeParms << /Predictor 2 /Columns 0 >>";
    let huge = "/DecodeParms << /Predictor 12 /Colors 4 /Columns 99999999999 >>";
    // An inline image whose data is deflated, written in hexadecimal so the content stays text.
    let inline = |dp: &str| {
        let data: String = deflated(&[0; 8]).iter().map(|b| format!("{b:02x}")).collect();
        format!("q 10 0 0 10 0 0 cm BI /W 2 /H 2 /CS /G /BPC 8 /F [/AHx /Fl] /DP [null {dp}] ID {data}> EI Q")
    };
    let form = |dict: &str, content: &str| pdf_with("/F Do", "<< /XObject << /F 5 0 R >> >>", &[packed(&format!("{FORM} {dict}"), content)]);
    let held = |object: String| pdf_with("0 0 5 5 re f", "<< >>", &[object.into_bytes()]);
    let shaded = |function: &str| {
        let shading = format!("<< /ShadingType 2 /ColorSpace /DeviceRGB /Coords [0 0 100 0] /Function {function} >>");
        pdf_with("/S sh", "<< /Shading << /S 5 0 R >> >>", &[shading.into_bytes()])
    };
    let spaces = calculator(&spaces_past_the_cap(), "");
    let program = deflated(b"{ dup 1 exch sub 0 }");
    // The message names the stream, so a report of a file that can't be shared says which (#4366).
    let (parameters, cap) = ("has decode parameters that can't be applied", "would decode to more than 256 MB");
    let files = [
        ("a form with Columns 0", form(zero, "0 0 5 5 re f"), format!("a form in it (object 5) {parameters}")),
        ("a form with a huge Columns", form(huge, "0 0 5 5 re f"), format!("a form in it (object 5) {parameters}")),
        (
            "an inline image with Columns 0",
            deep_pdf(&inline("<< /Predictor 2 /Columns 0 >>"), "<< >>"),
            format!("an inline image in the content of object 4 {parameters}"),
        ),
        (
            "an inline image with a huge Columns",
            deep_pdf(&inline("<< /Predictor 15 /Columns 99999999999 >>"), "<< >>"),
            format!("an inline image in the content of object 4 {parameters}"),
        ),
        (
            "an inline image in a compressed form",
            form("", &inline("<< /Predictor 2 /Columns 0 >>")),
            format!("an inline image in the content of object 5 {parameters}"),
        ),
        // Read as the file opens, before its objects can be looked at one by one.
        (
            "an object stream with Columns 0",
            pdf_with_object_stream("[0 0 100 100]", zero),
            format!("a stream in it {cap}, or its decode parameters can't be applied"),
        ),
        (
            "a form of spaces past the cap",
            pdf_with("/F Do", "<< /XObject << /F 5 0 R >> >>", &[packed_as_is(FORM, &spaces_past_the_cap())]),
            format!("a form in it (object 5) {cap}"),
        ),
        // A function written inside object 5 goes by it.
        ("a function of spaces past the cap in an array", held(format!("[{spaces}]")), format!("a stream in it (object 5) {cap}")),
        ("a function of spaces past the cap in a dictionary", held(format!("<< /K {spaces} >>")), format!("a stream in it (object 5) {cap}")),
        ("a function of spaces past the cap in a shading", shaded(&spaces), format!("a stream in it (object 5) {cap}")),
        (
            "a function with Columns 0 in an array",
            held(format!("[{}]", calculator(&program, "/DecodeParms [null << /Predictor 2 /Columns 0 >>]"))),
            format!("a stream in it (object 5) {parameters}"),
        ),
    ];
    on_a_small_stack(move || {
        for (what, bytes, why) in &files {
            for name in ["hostile.pdf", "hostile.ai"] {
                let e = vectorcraft_engine::cmd::fileio::load(name, bytes).err().map(|e| e.to_string());
                assert!(e.as_deref().is_some_and(|e| e.contains(why.as_str())), "{what} as {name}: {e:?}");
            }
        }
    });
    // Within range, the same files open.
    let fine = "/DecodeParms << /Predictor 1 /Columns 0 >>";
    let function = calculator(&program, "");
    for bytes in [form(fine, "0 0 5 5 re f"), deep_pdf(&inline("<< /Predictor 1 >>"), "<< >>"), held(format!("[{function}]")), shaded(&function)] {
        vectorcraft_engine::cmd::fileio::load("fine.pdf", &bytes).expect("opens");
    }
}

/// A page drawing a thousand times one form that decodes to 64 MiB (of spaces, then a square):
/// each drawing reads the form again, so the import stops at the page's form content instead of
/// decoding 64 GB (#3569, which hung).
#[test]
fn pdf_a_huge_form_drawn_over_and_over_is_refused_in_bounded_time() {
    use std::io::Write as _;
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    for _ in 0..64 {
        e.write_all(&[b' '; 1 << 20]).unwrap();
    }
    e.write_all(b"0 0 5 5 re f").unwrap();
    let form = packed_as_is(FORM, &e.finish().unwrap());
    let bytes = pdf_with(&"/F Do ".repeat(1000), "<< /XObject << /F 5 0 R >> >>", &[form]);
    let start = std::time::Instant::now();
    let e = vectorcraft_engine::cmd::fileio::load("repeated.pdf", &bytes).err().map(|e| e.to_string());
    let took = start.elapsed();
    assert!(e.as_deref().is_some_and(|e| e.contains("no document was imported")), "{e:?}");
    assert!(took < std::time::Duration::from_secs(60), "took {took:?}");
}

// A large file the reader would fail on says which stream (#4366): an image with its size in
// pixels, a page's content, or the editing data an `.ai` keeps in its first page's `PieceInfo`.
#[test]
fn a_stream_past_the_cap_is_named() {
    let image = "/Type /XObject /Subtype /Image /Width 10 /Height 10 /ColorSpace /DeviceRGB /BitsPerComponent 8";
    let page = |contents: &str, entries: &str| {
        format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents {contents} /Resources << /XObject << /Im 5 0 R >> >> {entries} >>")
            .into_bytes()
    };
    let plain = |content: &str| format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len() + 1).into_bytes();
    let pdf = |page: Vec<u8>, more: Vec<Vec<u8>>| {
        let head = [b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(), b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(), page];
        pdf_of(&head.into_iter().chain(more).collect::<Vec<_>>())
    };
    let spaces = spaces_past_the_cap();
    let editing = "/PieceInfo << /Illustrator << /Private << /AIPrivateData1 6 0 R /NumBlock 1 >> >> >>";
    let files = [
        (
            "an image",
            pdf(page("4 0 R", ""), vec![plain("q 10 0 0 10 0 0 cm /Im Do Q"), packed_as_is(image, &spaces)]),
            "an image of 10 × 10 px in it (object 5) would decode to more than 256 MB",
        ),
        (
            "a page's content",
            pdf(page("4 0 R", ""), vec![packed_as_is("", &spaces), packed(image, "\0\0\0")]),
            "a page's content in it (object 4) would decode to more than 256 MB",
        ),
        (
            "a page's content in an array",
            pdf(page("[4 0 R]", ""), vec![packed_as_is("", &spaces), packed(image, "\0\0\0")]),
            "a page's content in it (object 4) would decode to more than 256 MB",
        ),
        (
            "a page's content in an array of its own",
            pdf(page("6 0 R", ""), vec![packed_as_is("", &spaces), packed(image, "\0\0\0"), b"[4 0 R]".to_vec()]),
            "a page's content in it (object 4) would decode to more than 256 MB",
        ),
        (
            "the editing data",
            pdf(page("4 0 R", editing), vec![plain("0 0 5 5 re f"), packed(image, "\0\0\0"), packed_as_is("", &spaces)]),
            "its editing data (object 6) would decode to more than 256 MB",
        ),
    ];
    on_a_small_stack(move || {
        for (what, bytes, why) in &files {
            for name in ["large.pdf", "large.ai"] {
                let e = vectorcraft_engine::cmd::fileio::load(name, bytes).err().map(|e| e.to_string());
                assert!(e.as_deref().is_some_and(|e| e.contains(why)), "{what} as {name}: {e:?}");
            }
        }
    });
}

// Pages that share one `Contents` array, the array an object of its own: its stream is named as
// a page's content, and a refused stream elsewhere is still found past the pages (#4612 review).
#[test]
fn pages_sharing_a_contents_array_name_its_stream() {
    let image = "/Type /XObject /Subtype /Image /Width 10 /Height 10 /ColorSpace /DeviceRGB /BitsPerComponent 8";
    let plain = |content: &str| format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len() + 1).into_bytes();
    let spaces = spaces_past_the_cap();
    // 1 catalog, 2 pages, 3 the shared array, 4 its stream, 5 to 7 the pages, 8 an image they draw.
    let pdf = |content: Vec<u8>, image: Vec<u8>| {
        let page = b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 100] /Contents 3 0 R /Resources << /XObject << /Im 8 0 R >> >> >>";
        let head = [
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            b"<< /Type /Pages /Kids [5 0 R 6 0 R 7 0 R] /Count 3 >>".to_vec(),
            b"[4 0 R]".to_vec(),
            content,
        ];
        pdf_of(&head.into_iter().chain([page.to_vec(), page.to_vec(), page.to_vec(), image]).collect::<Vec<_>>())
    };
    let files = [
        (pdf(packed_as_is("", &spaces), packed(image, "\0\0\0")), "a page's content in it (object 4) would decode to more than 256 MB"),
        (
            pdf(plain("q 10 0 0 10 0 0 cm /Im Do Q"), packed_as_is(image, &spaces)),
            "an image of 10 × 10 px in it (object 8) would decode to more than 256 MB",
        ),
    ];
    on_a_small_stack(move || {
        for (bytes, why) in &files {
            let e = vectorcraft_engine::cmd::fileio::load("shared.pdf", bytes).err().map(|e| e.to_string());
            assert!(e.as_deref().is_some_and(|e| e.contains(why)), "{why}: {e:?}");
        }
    });
}

#[test]
fn pdf_art_that_draws_itself() {
    let own = "<< /XObject << /F 5 0 R >> /Pattern << /P 5 0 R >> /Font << /F1 5 0 R >> /ExtGState << /G << /SMask << /S /Alpha /G 5 0 R >> >> >> >>";
    let tile = format!("/PatternType 1 /PaintType 1 /TilingType 1 /BBox [0 0 10 10] /XStep 10 /YStep 10 /Resources {own}");
    let group = format!("{FORM} /Group << /S /Transparency >> /Resources {own}");
    open_each(
        "itself",
        [
            // A form drawing itself twice, a pattern painting its own tile, a glyph showing
            // itself, and a mask masked by itself.
            pdf_with("/F Do", own, &[packed(&format!("{FORM} /Resources {own}"), "0 0 5 5 re f /F Do 1 0 0 1 3 3 cm /F Do")]),
            pdf_with("/Pattern cs /P scn 0 0 50 50 re f", own, &[packed(&tile, "/Pattern cs /P scn 0 0 5 5 re f")]),
            pdf_with("BT /F1 12 Tf (a) Tj ET", own, &[type3_font(own), packed("", "10 0 d0 0 0 5 5 re f BT /F1 1 Tf (a) Tj ET")]),
            pdf_with("/G gs 0 0 50 50 re f", own, &[packed(&group, "/G gs 0 0 5 5 re f")]),
            // An image that is its own mask, and objects that are only a reference to each other.
            pdf_with("q 50 0 0 50 0 0 cm /F Do Q", own, &[
                b"<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /BitsPerComponent 8 /ColorSpace /DeviceGray /SMask 5 0 R /Length 1 >>\nstream\n0\nendstream".to_vec(),
            ]),
            handmade_pdf_with("0 0 5 5 re f", "5 0 R", "6 0 R", &[b"6 0 R".to_vec(), b"5 0 R".to_vec()]),
            handmade_pdf_with("0 0 5 5 re f", "<< >>", "[0 0 9 9]", &[b"<< /Length 5 0 R >>\nstream\nx\nendstream".to_vec()]),
        ],
    );
}

/// An EPS whose program is `body`.
fn deep_eps(body: &str) -> Vec<u8> {
    format!("%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 100 100\n%%EndComments\n{body}\nshowpage\n%%EOF\n").into_bytes()
}

/// A test for each file: it opens as an error or as a document within the cap, on a small stack.
macro_rules! deep_files {
    ($($test:ident: $name:literal, $bytes:expr;)*) => {$(
        #[test]
        fn $test() {
            on_a_small_stack(|| {
                open_deep($name, &$bytes);
            });
        }
    )*};
}

deep_files! {
    // PDF: clips inside clips (saved states never restored); arrays in arrays as an operand and as
    // an object; dictionaries in dictionaries as a resource and inline; marked content never ended.
    pdf_clips_in_clips: "clips.pdf", deep_pdf(&format!("{} 0 0 5 5 re f", "q 0 0 50 50 re W n ".repeat(20_000)), "<< >>");
    pdf_arrays_in_an_operand: "arrays.pdf", deep_pdf(&format!("BT {}{} TJ ET 0 0 5 5 re f", "[".repeat(100_000), "]".repeat(100_000)), "<< >>");
    pdf_arrays_in_an_object: "boxes.pdf", handmade_pdf("0 0 5 5 re f", "<< >>", &format!("{}0 0 100 100{}", "[".repeat(100_000), "]".repeat(100_000)));
    pdf_dictionaries_in_resources: "dicts.pdf", deep_pdf("0 0 5 5 re f", &format!("{}{}", "<< /A ".repeat(50_000), ">> ".repeat(50_000)));
    pdf_dictionaries_in_content: "inline.pdf", deep_pdf(&format!("/P {}{} DP 0 0 5 5 re f", "<< /A ".repeat(50_000), ">> ".repeat(50_000)), "<< >>");
    pdf_marked_content_never_ended: "marked.pdf", deep_pdf(&format!("{} 0 0 5 5 re f", "/Span << /MCID 0 >> BDC ".repeat(50_000)), "<< >>");
    // PostScript: clips inside clips; procedures in procedures; arrays in arrays; a procedure that
    // calls itself; dictionaries never ended.
    eps_clips_in_clips: "clips.eps", deep_eps(&format!("{} 0 0 5 5 rectfill", "gsave 0 0 50 50 rectclip ".repeat(20_000)));
    eps_procedures_in_procedures: "procs.eps", deep_eps(&format!("{}{} 0 0 5 5 rectfill", "{".repeat(100_000), "}".repeat(100_000)));
    eps_arrays_in_arrays: "arrays.eps", deep_eps(&format!("{}{} pop 0 0 5 5 rectfill", "[".repeat(100_000), "]".repeat(100_000)));
    eps_a_procedure_calling_itself: "runaway.eps", deep_eps("/f { 0 0 5 5 rectfill f } def f");
    eps_dictionaries_never_ended: "dicts.eps", deep_eps(&format!("{} 0 0 5 5 rectfill", "1 dict begin ".repeat(100_000)));
}

#[test]
fn deeply_nested_dxf_blocks_are_read_within_the_cap() {
    on_a_small_stack(|| {
        // Block `B0` inserts `B1`, which inserts `B2`, and so on; the last one holds a line.
        let pair = |c: i32, v: &str| (c, v.to_string());
        let mut pairs = vec![pair(0, "SECTION"), pair(2, "BLOCKS")];
        for i in 0..2_000 {
            pairs.extend([pair(0, "BLOCK"), pair(2, &format!("B{i}")), pair(10, "0"), pair(20, "0")]);
            if i + 1 < 2_000 {
                pairs.extend([pair(0, "INSERT"), pair(2, &format!("B{}", i + 1)), pair(10, "1"), pair(20, "1")]);
            } else {
                pairs.extend([pair(0, "LINE"), pair(10, "0"), pair(20, "0"), pair(11, "5"), pair(21, "5")]);
            }
            pairs.push(pair(0, "ENDBLK"));
        }
        pairs.extend([pair(0, "ENDSEC"), pair(0, "SECTION"), pair(2, "ENTITIES"), pair(0, "INSERT"), pair(2, "B0"), pair(10, "0"), pair(20, "0")]);
        pairs.extend([pair(0, "LINE"), pair(10, "0"), pair(20, "0"), pair(11, "9"), pair(21, "9"), pair(0, "ENDSEC"), pair(0, "EOF")]);
        open_deep("blocks.dxf", dxf_text(&pairs).as_bytes()).expect("opens");
    });
}

#[test]
fn dxf_blocks_inserting_themselves_are_left_out() {
    on_a_small_stack(|| {
        let pair = |c: i32, v: &str| (c, v.to_string());
        // A line, then an insert of `inserts`.
        let block = |name: &str, inserts: &str| {
            let line = [pair(0, "LINE"), pair(10, "0"), pair(20, "0"), pair(11, "5"), pair(21, "5")];
            let insert = [pair(0, "INSERT"), pair(2, inserts), pair(10, "1"), pair(20, "1"), pair(0, "ENDBLK")];
            [pair(0, "BLOCK"), pair(2, name), pair(10, "0"), pair(20, "0")].into_iter().chain(line).chain(insert).collect::<Vec<_>>()
        };
        let mut pairs = vec![pair(0, "SECTION"), pair(2, "BLOCKS")];
        pairs.extend([block("SELF", "SELF"), block("A", "B"), block("B", "A")].concat());
        pairs.extend([pair(0, "ENDSEC"), pair(0, "SECTION"), pair(2, "ENTITIES")]);
        for name in ["SELF", "A"] {
            pairs.extend([pair(0, "INSERT"), pair(2, name), pair(10, "0"), pair(20, "0")]);
        }
        pairs.extend([pair(0, "ENDSEC"), pair(0, "EOF")]);
        let (_, warnings) = open_deep("self.dxf", dxf_text(&pairs).as_bytes()).expect("opens");
        assert!(warnings.iter().any(|w| w.contains("inside themselves")), "{warnings:?}");
    });
}

#[test]
fn deeply_nested_affinity_groups_are_read_within_the_cap() {
    use vectorcraft_affinity::synth::{self, F, Method, tag};
    // Groups in groups: within what the reader takes, past its cap on layers (128), past its cap
    // on the stream's own nesting (384), and far past both.
    let files: Vec<Vec<u8>> = [20u32, 120, 180, 370, 2000]
        .into_iter()
        .map(|levels| {
            let group = |id: u32, fields| F::Def(id, vec![tag(b"Grup")], fields);
            let inner = (0..levels).fold(group(10, vec![]), |inner, i| group(11 + i, vec![(tag(b"Chld"), F::Shared(vec![inner]))]));
            let spread = F::Def(2, vec![tag(b"Sprd")], vec![(tag(b"Chld"), F::Shared(vec![inner]))]);
            let stream = synth::stream(&[(tag(b"DocR"), F::Def(1, vec![tag(b"DocN")], vec![(tag(b"Chld"), F::Shared(vec![spread]))]))]);
            affinity_native(&stream, Method::Stored)
        })
        .collect();
    on_a_small_stack(move || {
        for bytes in &files {
            open_deep("deep.afdesign", bytes);
        }
    });
}

#[test]
fn metafiles_saving_states_without_end_are_read_within_the_cap() {
    on_a_small_stack(|| {
        // A saved state, a clip and a rectangle, over and over (nothing is ever restored).
        let longs = |v: [i32; 4]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let emf: Vec<(u32, Vec<u8>)> = (0..50_000).flat_map(|_| [(33, vec![]), (30, longs([0, 0, 500, 500])), (43, longs([0, 0, 50, 50]))]).collect();
        open_deep("deep.emf", &emf_of(&emf));
        let shorts = |v: [i16; 4]| v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>();
        let wmf: Vec<(u16, Vec<u8>)> =
            (0..20_000).flat_map(|_| [(0x001E, vec![]), (0x0416, shorts([500, 500, 0, 0])), (0x041B, shorts([50, 50, 0, 0]))]).collect();
        open_deep("deep.wmf", &wmf_of(&wmf));
    });
}


/// A document whose symbol `s0` and pattern `p0` are each a line stroked with the brush `loop`,
/// with an instance of the symbol and a square painted with the pattern on the layer. The brush
/// is the `kind`th of those that hold art (art, scatter, pattern), and its art is the `art`th of:
/// an instance of the symbol; that instance inside as many groups as a file holds; a square
/// painted with the pattern. So what the brush draws is drawn with the brush.
fn brushed_document(kind: usize, art: usize) -> Document {
    use vectorcraft_brush::{ArtBrush, Brush, BrushKind, PatternBrush, Scatter};
    use vectorcraft_color::Paint;
    use vectorcraft_doc::{Appearance, AppearanceItem, Node, NodeKind, PatternDef, Symbol};
    let mut doc = nested_document(1);
    let square = || vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(0.0, 0.0, 60.0, 20.0));
    let brushed = |doc: &mut Document| {
        let mut look = Appearance::basic(Paint::None, Paint::solid(vectorcraft_color::Color::rgb8(0, 0, 0)), 4.0);
        for item in &mut look.items {
            if let AppearanceItem::Stroke(s) = item {
                s.brush = Some("loop".into());
            }
        }
        Node::path(doc.alloc_id(), square(), look)
    };
    let instance =
        |doc: &mut Document| Node::new(doc.alloc_id(), NodeKind::SymbolInstance { symbol: "s0".into(), xf: vectorcraft_geom::Affine::IDENTITY });
    let painted = |doc: &mut Document| {
        let fill = Paint::Pattern { pattern: "p0".into(), xf: vectorcraft_geom::Affine::IDENTITY };
        Node::path(doc.alloc_id(), square(), Appearance::basic(fill, Paint::None, 0.0))
    };
    let (in_symbol, in_pattern) = (brushed(&mut doc), brushed(&mut doc));
    doc.symbols.push(Symbol { dynamic: false, name: "s0".into(), art: std::sync::Arc::new(in_symbol) });
    doc.patterns.push(PatternDef::new("p0", vec![std::sync::Arc::new(in_pattern)]));
    let art = match art {
        0 => instance(&mut doc),
        1 => (0..MAX_DEPTH + 4).fold(instance(&mut doc), |inner, _| Node::group(doc.alloc_id(), vec![std::sync::Arc::new(inner)])),
        _ => painted(&mut doc),
    };
    let kind = match kind {
        0 => BrushKind::Art(ArtBrush { art, ..Default::default() }),
        1 => BrushKind::Scatter(Scatter { art, ..Default::default() }),
        _ => BrushKind::Pattern(PatternBrush { side: art.clone(), start: Some(art), ..Default::default() }),
    };
    vectorcraft_brush::store(&mut doc, &[Brush { name: "loop".into(), kind }]);
    let layer = doc.layers[0].id;
    for on_layer in [instance(&mut doc), painted(&mut doc), brushed(&mut doc)] {
        doc.insert(Some(layer), usize::MAX, on_layer).unwrap();
    }
    doc
}

/// `doc` saved, opened again, drawn, exported and edited: nothing overflows.
fn art_by_name_survives(what: &str, doc: Document) {
    let (back, _) = open_deep("named.vectorcraft", &vectorcraft_format::save_file(&doc)).expect("opens");
    let mut s = vectorcraft_engine::Session::new();
    s.add_document(back, None);
    for (cmd, p) in [
        ("select.all", json!({})),
        ("document.inspect", json!({})),
        ("brush.expand", json!({})),
        ("edit.undo", json!({})),
        ("object.expand", json!({})),
        ("edit.undo", json!({})),
        ("object.outlineStroke", json!({})),
        ("edit.undo", json!({})),
        ("graph.design", json!({"paste": "deep"})),
        ("graph.create", json!({"type": "line", "x": 10, "y": 10, "width": 80, "height": 60})),
        ("graph.marker", json!({"design": "deep"})),
        ("graph.create", json!({"type": "column", "x": 10, "y": 10, "width": 80, "height": 60})),
        ("graph.column", json!({"design": "deep", "columnType": "repeating", "each": 1})),
        ("graph.design", json!({"new": "again"})),
        ("edit.copy", json!({})),
        ("edit.paste", json!({})),
    ] {
        let _ = no_panic(s.execute(cmd, &p));
        assert!(!s.doc().unwrap().doc.too_deep(None), "{what}: {cmd}");
    }
    for format in ["svg", "pdf", "eps", "png", "dxf", "emf"] {
        let _ = no_panic(s.execute("document.export", &json!({"format": format})));
    }
}

/// A test for each brush whose art is drawn with the brush itself ([`brushed_document`]).
macro_rules! brushes_drawn_with_themselves {
    ($($test:ident: $kind:literal, $art:literal;)*) => {$(
        #[test]
        fn $test() {
            on_a_small_stack(|| art_by_name_survives(stringify!($test), brushed_document($kind, $art)));
        }
    )*};
}

brushes_drawn_with_themselves! {
    an_art_brush_of_its_own_symbol: 0, 0;
    an_art_brush_of_its_own_symbol_in_deep_art: 0, 1;
    an_art_brush_painted_with_its_own_pattern: 0, 2;
    a_scatter_brush_of_its_own_symbol: 1, 0;
    a_scatter_brush_of_its_own_symbol_in_deep_art: 1, 1;
    a_scatter_brush_painted_with_its_own_pattern: 1, 2;
    a_pattern_brush_of_its_own_symbol: 2, 0;
    a_pattern_brush_of_its_own_symbol_in_deep_art: 2, 1;
    a_pattern_brush_painted_with_its_own_pattern: 2, 2;
}

#[test]
fn a_graph_design_of_deep_art_and_of_a_symbol_holding_itself_is_drawn() {
    use vectorcraft_doc::{GraphDesign, Node, NodeKind};
    on_a_small_stack(|| {
        let mut doc = symbols_document(1, 1, "s0");
        let instance = Node::new(doc.alloc_id(), NodeKind::SymbolInstance { symbol: "s0".into(), xf: vectorcraft_geom::Affine::IDENTITY });
        let art = (0..MAX_DEPTH + 4).fold(instance, |inner, _| Node::group(doc.alloc_id(), vec![std::sync::Arc::new(inner)]));
        doc.graph_designs.push(GraphDesign { name: "deep".into(), art: std::sync::Arc::new(art) });
        art_by_name_survives("a graph design of deep art", doc);
    });
}

/// A layer holding `levels` groups in one another around a rectangle, each with a second
/// rectangle beside the next: `levels + 2` deep.
fn nested_document(levels: usize) -> Document {
    use vectorcraft_doc::Node;
    use vectorcraft_testkit::fixtures::DocBuilder;
    let mut b = DocBuilder::new(100.0, 100.0);
    let rect =
        |b: &mut DocBuilder, x: f64| b.rect_node(vectorcraft_geom::Rect::new(x, 0.0, x + 10.0, 10.0), vectorcraft_color::Color::rgb8(200, 0, 0));
    let mut inner = rect(&mut b, 0.0);
    for _ in 0..levels {
        let beside = rect(&mut b, 20.0);
        inner = Node::group(b.alloc(), vec![std::sync::Arc::new(inner), std::sync::Arc::new(beside)]);
        inner.opacity = 0.9;
    }
    b.add(inner);
    b.build()
}

#[test]
fn a_document_at_the_cap_saves_opens_and_exports() {
    on_a_small_stack(|| {
        let doc = nested_document(MAX_DEPTH - 2);
        assert!(!doc.too_deep(None));
        let back = vectorcraft_format::load_file(&vectorcraft_format::save_file(&doc)).expect("what can be made opens again");
        assert!(back.depth_notes.is_empty());
        assert_eq!(back.doc.layers, doc.layers);
        survive("at the cap", || Some(doc.clone())).unwrap();
        // Every format that keeps the nesting reads its own file back within the cap.
        let mut s = vectorcraft_engine::Session::new();
        s.add_document(doc, None);
        for format in ["svg", "pdf", "eps", "ai"] {
            let r = s.execute("document.export", &json!({"format": format})).unwrap();
            let bytes = vectorcraft_format::base64_decode(r["dataBase64"].as_str().unwrap()).unwrap();
            open_deep(&format!("cap.{format}"), &bytes).unwrap_or_else(|| panic!("{format} opens"));
        }
        for cmd in ["select.all", "document.inspect", "document.info", "edit.copy", "edit.paste", "object.ungroup", "edit.undo"] {
            no_panic(s.execute(cmd, &json!({}))).unwrap_or_else(|e| panic!("{cmd}: {e}"));
        }
    });
}

#[test]
fn a_native_file_nested_too_deep_is_merged_or_refused() {
    on_a_small_stack(|| {
        // Saved before the cap by a version that could still open it: opened, the deepest merged.
        let old = vectorcraft_format::save_file(&nested_document(MAX_DEPTH + 2));
        let (doc, warnings) = open_deep("old.vectorcraft", &old).expect("opens");
        assert!(merged(&warnings), "{warnings:?}");
        assert_eq!(paths_in(&doc), MAX_DEPTH + 3, "no art is lost");
        // Deeper than any version could open, and a hostile file: an error that says why. Written
        // by hand: a tree this deep can't be built and saved.
        let text = String::from_utf8(vectorcraft_format::save(&nested_document(0), false)).unwrap();
        let (open, close) = ("{\"id\":900000,\"kind\":{\"type\":\"group\",\"children\":[", "]}}");
        let at = text.find("\"children\":[").expect("the layer's children") + "\"children\":[".len();
        for levels in [100, 50_000] {
            let deep = format!("{}{}{},{}", &text[..at], open.repeat(levels), close.repeat(levels), &text[at..]);
            let e = vectorcraft_engine::cmd::fileio::load("deep.vectorcraft", deep.as_bytes()).err().expect("refused").to_string();
            assert!(e.contains("nested too deep"), "{levels}: {e}");
        }
    });
}

#[test]
fn commands_cannot_nest_deeper_than_the_cap() {
    on_a_small_stack(|| {
        let mut s = vectorcraft_engine::Session::new();
        s.execute("file.new", &json!({"width": 200, "height": 200})).unwrap();
        s.execute("shape.rectangle", &json!({"x": 10, "y": 10, "width": 50, "height": 50})).unwrap();
        s.execute("select.all", &json!({})).unwrap();
        let mut grouped = 0;
        for _ in 0..10_000 {
            match no_panic(s.execute("object.group", &json!({}))) {
                Ok(_) => grouped += 1,
                Err(e) => assert!(e.to_string().contains(vectorcraft_engine::TOO_DEEP), "after {grouped} groups: {e}"),
            }
        }
        // The layer, the groups and the rectangle.
        assert_eq!(grouped, MAX_DEPTH - 2);
        assert!(!s.doc().unwrap().doc.too_deep(None));
        // Other ways down: a clipping mask, a copy pasted, a symbol made of it.
        for (cmd, p) in [
            ("object.clippingMask.make", json!({})),
            ("edit.copy", json!({})),
            ("edit.pasteInFront", json!({})),
            ("symbol.new", json!({"name": "deep"})),
        ] {
            let _ = no_panic(s.execute(cmd, &p));
            assert!(!s.doc().unwrap().doc.too_deep(None), "{cmd}");
        }
        survive("grouped to the cap", || Some((*s.doc().unwrap().doc).clone())).unwrap();
    });
}

/// A document whose one object is an instance of the first of `count` symbols: each symbol's art
/// is `levels` groups around an instance of the next symbol, the last one of `last`.
fn symbols_document(count: usize, levels: usize, last: &str) -> Document {
    use vectorcraft_doc::{Node, NodeKind, Symbol};
    let instance =
        |doc: &mut Document, of: String| Node::new(doc.alloc_id(), NodeKind::SymbolInstance { symbol: of, xf: vectorcraft_geom::Affine::IDENTITY });
    let mut doc = nested_document(1);
    for i in 0..count {
        let mut art = instance(&mut doc, if i + 1 < count { format!("s{}", i + 1) } else { last.to_string() });
        for _ in 0..levels {
            art = Node::group(doc.alloc_id(), vec![std::sync::Arc::new(art)]);
        }
        doc.symbols.push(Symbol { dynamic: false, name: format!("s{i}"), art: std::sync::Arc::new(art) });
    }
    let first = instance(&mut doc, "s0".into());
    let layer = doc.layers[0].id;
    doc.insert(Some(layer), usize::MAX, first).unwrap();
    doc
}

/// `doc` saved (as an older version, or a hostile file, could have) and opened again: within the
/// caps (`kept`: as it was), drawn, exported and edited.
fn symbols_survive(what: &str, doc: Document, kept: bool) {
    let (back, warnings) = open_deep("symbols.vectorcraft", &vectorcraft_format::save_file(&doc)).expect("opens");
    assert_eq!(!warnings.iter().any(|w| w.contains("symbol instances")), kept, "{what}: {warnings:?}");
    let mut s = vectorcraft_engine::Session::new();
    s.add_document(back, None);
    for cmd in [
        "select.all",
        "document.inspect",
        "document.info",
        "object.expand",
        "symbol.breakLink",
        "object.flattenTransparency",
        "edit.copy",
        "edit.paste",
    ] {
        let _ = no_panic(s.execute(cmd, &json!({})));
        assert!(!s.doc().unwrap().doc.too_deep(None), "{what}: {cmd}");
    }
    for format in ["svg", "pdf", "eps", "png"] {
        let _ = no_panic(s.execute("document.export", &json!({"format": format})));
    }
}

#[test]
fn a_symbol_holding_itself_opens_without_the_instance() {
    on_a_small_stack(|| {
        symbols_survive("a symbol holding itself", symbols_document(1, 1, "s0"), false);
        symbols_survive("symbols holding one another", symbols_document(3, 2, "s0"), false);
    });
}

#[test]
fn symbols_in_symbols_are_drawn_only_so_deep() {
    on_a_small_stack(|| {
        // As deep as symbols draw: each one's art as deep as art goes.
        symbols_survive("symbols at the cap", symbols_document(3, MAX_DEPTH - 2, "none"), true);
        symbols_survive("symbols in symbols", symbols_document(40, MAX_DEPTH - 1, "none"), false);
        // A symbol can't be made to hold itself, or one that holds it, by editing either.
        let mut s = vectorcraft_engine::Session::new();
        s.add_document(symbols_document(2, 1, "none"), None);
        s.execute("select.all", &json!({})).unwrap();
        for (cmd, p) in [
            ("symbol.new", json!({"name": "s1"})),
            ("symbol.update", json!({"name": "s1"})),
            ("symbol.update", json!({"name": "s0"})),
            ("symbol.replace", json!({"name": "s1"})),
        ] {
            let _ = no_panic(s.execute(cmd, &p));
            assert!(!s.doc().unwrap().doc.too_deep(None), "{cmd} {p}");
        }
    });
}

#[test]
fn a_pattern_painting_its_own_tile_opens_without_the_paint() {
    on_a_small_stack(|| {
        use vectorcraft_color::Paint;
        use vectorcraft_doc::{Appearance, PatternDef};
        let painted = |doc: &mut Document, with: &str| {
            let fill = Paint::Pattern { pattern: with.into(), xf: vectorcraft_geom::Affine::IDENTITY };
            let square = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(0.0, 0.0, 20.0, 20.0));
            vectorcraft_doc::Node::path(doc.alloc_id(), square, Appearance::basic(fill, Paint::None, 0.0))
        };
        // One pattern painting its own tile, and two painting each other's.
        for names in [["p", "p"], ["p", "q"]] {
            let mut doc = nested_document(1);
            let (first, second) = (painted(&mut doc, names[1]), painted(&mut doc, names[0]));
            doc.patterns.push(PatternDef::new(names[0], vec![std::sync::Arc::new(first)]));
            if names[0] != names[1] {
                doc.patterns.push(PatternDef::new(names[1], vec![std::sync::Arc::new(second)]));
            }
            let on_layer = painted(&mut doc, "p");
            let layer = doc.layers[0].id;
            doc.insert(Some(layer), usize::MAX, on_layer).unwrap();
            symbols_survive("a pattern painting its own tile", doc, false);
        }
    });
}

/// A .ai cut short inside an array or dictionary of its PDF part (found by
/// `ai_mutated_save_never_panics`): a value can't be read whole, and hayro's iterators over the
/// array's items or the dictionary's entries would panic on it.
#[test]
fn ai_cut_inside_an_array_or_dictionary_opens_without_panicking() {
    for flip in [None, Some((12262, b'/'))] {
        let mut b = saved_ai();
        if let Some((at, c)) = flip {
            b[at] = c;
        }
        b.truncate(44902.min(b.len()));
        survive("ai cut inside an array or dictionary", || vectorcraft_engine::cmd::fileio::load("saved.ai", &b).ok().map(|l| l.doc)).unwrap();
    }
}

/// Found by the fuzzer above: a shape 3.3 million points long and 50 high, with a drop shadow,
/// made the PDF export rasterise its shadow wider than the renderer can address (within the
/// effect image's area limit, past any side), which panicked.
#[test]
fn a_long_thin_shape_with_a_shadow_renders_and_exports() {
    use affinity_docs::*;
    let stream = canvas_stream(vec![with_effects(rectangle(10, (0.0, 0.0), (3_276_800.0, 50.0)), vec![shadow()])]);
    let bytes = affinity_sample_file(&stream, None);
    let doc = vectorcraft_engine::cmd::fileio::load("long.af", &bytes).unwrap().doc;
    survive("long shape", || Some(doc)).unwrap();
}





proptest! {
    #![proptest_config(config())]


}


proptest! {
    #![proptest_config(config())]

}


proptest! {
    #![proptest_config(config())]

}

proptest! {
    #![proptest_config(config())]

}
