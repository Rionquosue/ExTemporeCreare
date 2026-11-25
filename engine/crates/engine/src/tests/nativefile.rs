//! Native `.etc` files: the thumbnail, atomic writes, unused images left out, keys from newer
//! versions kept, and saves that are byte-identical for the same document.

use std::path::PathBuf;

use serde_json::{Value, json};
use etcfile::sniff;

use crate::*;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 400, "height": 200})).unwrap();
    s.execute("shape.rectangle", &json!({"x": 10, "y": 10, "width": 100, "height": 50})).unwrap();
    s
}

fn tmp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("vc-native-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn bytes_of(r: &Value) -> Vec<u8> {
    etcfile::base64::decode(r["dataBase64"].as_str().expect("dataBase64")).unwrap()
}

/// The file `document.save` (no path) gives back with `p`.
fn saved(s: &mut Session, p: Value) -> Vec<u8> {
    bytes_of(&s.execute("document.save", &p).unwrap())
}

fn open(s: &mut Session, name: &str, bytes: &[u8]) -> Value {
    s.execute("document.open", &json!({"name": name, "dataBase64": etcfile::base64::encode(bytes)})).unwrap()
}

/// The file's `project.json`.
fn json_of(bytes: &[u8]) -> Value {
    serde_json::from_slice(&etcfile::project_json(bytes).expect("an .etc file")).unwrap()
}

/// `bytes` with its `project.json` replaced by `project`, every other entry kept.
fn with_project(bytes: &[u8], project: &Value) -> Vec<u8> {
    use etcfile::zipwrite::{Method, ZipWriter};
    let zip = etcfile::zipread::ZipArchive::new(bytes).unwrap();
    let mut w = ZipWriter::new(Vec::new());
    for e in zip.entries() {
        let data = if e.name == etcfile::PROJECT_JSON { serde_json::to_vec(project).unwrap() } else { zip.read_entry(e, u64::MAX).unwrap() };
        w.add(&e.name, &data, if e.method == 0 { Method::Stored } else { Method::Deflate }).unwrap();
    }
    w.finish().unwrap()
}





#[test]
fn preview_is_at_most_256_pixels() {
    let mut s = session();
    let bytes = saved(&mut s, json!({}));
    let png = etcfile::thumbnail(&bytes).expect("a thumbnail");
    let img = craft_image_codecs::decode(&png)
        .map(|__cimg| {
            let (__w, __h) = __cimg.dimensions();
            match (__cimg.layout(), __cimg.sample_type()) {
                (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageLumaA8(image::GrayAlphaImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                _ => image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(__w, __h, __cimg.to_rgba8()).unwrap()),
            }
        })
        .unwrap();
    assert_eq!((img.width(), img.height()), (256, 128), "the 400×200 artboard fitted into 256 px");
    assert!(etcfile::merged_image(&bytes).is_some(), "every save carries the flattened image");
}

#[test]
fn saving_writes_through_a_temporary_file() {
    let d = tmp_dir("atomic");
    let path = d.join("doc.etc");
    std::fs::write(&path, b"an older version of the file").unwrap();
    let mut s = session();
    s.execute("document.save", &json!({"path": path.to_string_lossy()})).unwrap();
    assert!(sniff(&std::fs::read(&path).unwrap()));
    let names: Vec<String> = std::fs::read_dir(&d).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
    assert_eq!(names, ["doc.etc"], "no temporary file left behind");
    assert!(!s.doc().unwrap().is_dirty());
    // A save that can't be written fails, and the document stays modified.
    s.execute("shape.rectangle", &json!({"x": 0, "y": 0, "width": 5, "height": 5})).unwrap();
    let missing = d.join("missing").join("doc.etc");
    assert!(s.execute("document.save", &json!({"path": missing.to_string_lossy()})).is_err());
    assert!(s.doc().unwrap().is_dirty());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn unused_images_are_left_out_of_saves() {
    let mut s = session();
    let mut png = vec![];
    {
        let __dyn: image::DynamicImage = (image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]))).into();
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
    let r = s.execute("file.place", &json!({"name": "a.png", "dataBase64": etcfile::base64::encode(&png)})).unwrap();
    let id = r["ids"][0].as_u64().unwrap();
    assert_eq!(json_of(&saved(&mut s, json!({})))["assets"]["images"].as_object().unwrap().len(), 1);
    s.execute("edit.clear", &json!({"ids": [id]})).unwrap();
    assert_eq!(s.doc().unwrap().doc.images.len(), 1, "kept in memory (undo can bring the image back)");
    assert!(json_of(&saved(&mut s, json!({})))["assets"]["images"].as_object().unwrap().is_empty());
    s.execute("edit.undo", &json!({})).unwrap();
    let bytes = saved(&mut s, json!({}));
    open(&mut s, "back.etc", &bytes);
    assert_eq!(s.doc().unwrap().doc.images.len(), 1);
}

#[test]
fn newer_keys_survive_open_and_save() {
    let mut s = session();
    let bytes = saved(&mut s, json!({}));
    let mut j = json_of(&bytes);
    j["document"]["fromTheFuture"] = json!({"x": 1});
    open(&mut s, "newer.etc", &with_project(&bytes, &j));
    let again = json_of(&saved(&mut s, json!({})));
    assert_eq!(again["document"]["fromTheFuture"], json!({"x": 1}));
}

/// A save records the date the document was made in, so the same commands twice gave different
/// bytes (#2880). A pinned date (`file.new {created}`) makes the saves of a document identical.
#[test]
fn a_pinned_date_makes_saves_byte_identical() {
    let session_at = |t: i64| {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 400, "height": 200, "created": t})).unwrap();
        s.execute("shape.rectangle", &json!({"x": 10, "y": 10, "width": 100, "height": 50})).unwrap();
        s
    };
    let at = 1_791_208_980;
    let mut s = session_at(at);
    assert_eq!(saved(&mut s, json!({})), saved(&mut s, json!({})), "the same document and date give the same bytes");
    // The date really is in the file, so the comparison above is not vacuous.
    let created = |s: &mut Session| json_of(&saved(s, json!({})))["document"]["metadata"].clone();
    assert_ne!(created(&mut s), created(&mut session_at(at + 1)), "another date shows in the file");
}
