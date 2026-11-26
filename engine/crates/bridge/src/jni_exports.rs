//! The JNI functions of `com.extemporecreare.app.components.engine.EngineBridge`
//! (`components/engine/EngineBridge.kt`). Each one converts its Java arguments and calls
//! [`crate::calls`]; strings cross as UTF-8 JSON text.
//!
//! JNI requires exported symbols with fixed names (`#[unsafe(no_mangle)]`), which the workspace's
//! `unsafe_code` lint covers; it is allowed in this module only. No other unsafe code is used.
#![allow(unsafe_code)]

use jni::JNIEnv;
use jni::objects::{JByteBuffer, JClass, JString};
use jni::sys::{jboolean, jdouble, jint, jlong, jstring};

use crate::{boards, calls, files, paint, sessions};

/// A Java string as Rust text (empty when it can't be read).
fn text(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(Into::into).unwrap_or_default()
}

/// Rust text as a new Java string (null when it can't be made).
fn jtext(env: &mut JNIEnv, s: &str) -> jstring {
    env.new_string(s).map(|j| j.into_raw()).unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeCreate(_env: JNIEnv, _class: JClass) -> jlong {
    sessions::create()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeDestroy(_env: JNIEnv, _class: JClass, handle: jlong) {
    sessions::destroy(handle);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeSetPresetsRoot(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) {
    let path = text(&mut env, &path);
    calls::set_presets_root(&path);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeExecute(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    command: JString,
    params: JString,
) -> jstring {
    let (command, params) = (text(&mut env, &command), text(&mut env, &params));
    let out = calls::execute(handle, &command, &params);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativePointer(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    event: JString,
    view: JString,
) -> jstring {
    let (event, view) = (text(&mut env, &event), text(&mut env, &view));
    let out = calls::pointer(handle, &event, &view);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeSelectTool(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    id: JString,
    view: JString,
) -> jstring {
    let (id, view) = (text(&mut env, &id), text(&mut env, &view));
    let out = calls::select_tool(handle, &id, &view);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeToolOptions(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let out = calls::tool_options(handle);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeSetToolOption(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    key: JString,
    value: JString,
) -> jstring {
    let (key, value) = (text(&mut env, &key), text(&mut env, &value));
    let out = calls::set_tool_option(handle, &key, &value);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeState(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
) -> jstring {
    let out = calls::state(handle);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeOverlays(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    view: JString,
) -> jstring {
    let view = text(&mut env, &view);
    let out = calls::overlays(handle, &view);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeOpen(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    temp: JString,
    name: JString,
    uri: JString,
) -> jstring {
    let (temp, name, uri) = (text(&mut env, &temp), text(&mut env, &name), text(&mut env, &uri));
    let out = files::open(handle, &temp, &name, &uri);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeSave(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    mode: JString,
    params: JString,
    temp_dir: JString,
) -> jstring {
    let (mode, params, temp_dir) = (text(&mut env, &mode), text(&mut env, &params), text(&mut env, &temp_dir));
    let out = files::save(handle, &mode, &params, &temp_dir);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeSaveFinish(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    ok: jboolean,
    final_uri: JString,
) -> jstring {
    let final_uri = text(&mut env, &final_uri);
    let out = files::save_finish(handle, ok != 0, &final_uri);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeValidate(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jstring {
    let path = text(&mut env, &path);
    let out = files::validate(&path);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeBoardNew(
    mut env: JNIEnv,
    _class: JClass,
    title: JString,
) -> jstring {
    let title = text(&mut env, &title);
    let out = boards::new_board(&title);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeBoardRead(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
    board_dir: JString,
) -> jstring {
    let (path, board_dir) = (text(&mut env, &path), text(&mut env, &board_dir));
    let out = boards::read(&path, &board_dir);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeBoardWrite(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
    board: JString,
    thumbnail: JString,
) -> jstring {
    let (path, board, thumbnail) = (text(&mut env, &path), text(&mut env, &board), text(&mut env, &thumbnail));
    let out = boards::write(&path, &board, &thumbnail);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeBoardLinks(
    mut env: JNIEnv,
    _class: JClass,
    board: JString,
    board_dir: JString,
) -> jstring {
    let (board, board_dir) = (text(&mut env, &board), text(&mut env, &board_dir));
    let out = boards::links(&board, &board_dir);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeBoardRelink(
    mut env: JNIEnv,
    _class: JClass,
    board: JString,
    id: JString,
    file: JString,
    board_dir: JString,
) -> jstring {
    let (board, id, file, board_dir) = (text(&mut env, &board), text(&mut env, &id), text(&mut env, &file), text(&mut env, &board_dir));
    let out = boards::relink_image(&board, &id, &file, &board_dir);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeBoardFindMissing(
    mut env: JNIEnv,
    _class: JClass,
    board: JString,
    board_dir: JString,
    folders: JString,
) -> jstring {
    let (board, board_dir, folders) = (text(&mut env, &board), text(&mut env, &board_dir), text(&mut env, &folders));
    let out = boards::find(&board, &board_dir, &folders);
    jtext(&mut env, &out)
}

/// Render into `out`, a direct buffer of at least `width × height × 4` bytes → whether it was
/// filled.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativeRender(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jdouble,
    y: jdouble,
    width: jint,
    height: jint,
    scale: jdouble,
    out: JByteBuffer,
) -> jboolean {
    let (Ok(w), Ok(h)) = (u32::try_from(width), u32::try_from(height)) else { return 0 };
    let Some(pixels) = calls::render(handle, x, y, w, h, scale) else { return 0 };
    let (Ok(addr), Ok(cap)) = (env.get_direct_buffer_address(&out), env.get_direct_buffer_capacity(&out)) else { return 0 };
    if addr.is_null() || cap < pixels.len() {
        return 0;
    }
    // SAFETY: `addr` is the start of a direct buffer of `cap` bytes, at least `pixels.len()`, that
    // the JVM keeps alive for this call; nothing else writes to it during the call.
    let dest = unsafe { std::slice::from_raw_parts_mut(addr, pixels.len()) };
    dest.copy_from_slice(&pixels);
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativePaintBegin(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    params: JString,
) -> jstring {
    let params = text(&mut env, &params);
    let out = paint::begin(handle, &params);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativePaintAdd(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    points: JString,
) -> jstring {
    let points = text(&mut env, &points);
    let out = paint::add(handle, &points);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativePaintPixels(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    x: jint,
    y: jint,
    width: jint,
    height: jint,
    out: JByteBuffer,
) -> jboolean {
    let (Ok(x), Ok(y), Ok(w), Ok(h)) = (u32::try_from(x), u32::try_from(y), u32::try_from(width), u32::try_from(height)) else { return 0 };
    let Some(pixels) = paint::pixels(handle, x, y, w, h) else { return 0 };
    let (Ok(addr), Ok(cap)) = (env.get_direct_buffer_address(&out), env.get_direct_buffer_capacity(&out)) else { return 0 };
    if addr.is_null() || cap < pixels.len() {
        return 0;
    }
    // SAFETY: `addr` is the start of a direct buffer of `cap` bytes, at least `pixels.len()`, that
    // the JVM keeps alive for this call; nothing else writes to it during the call.
    let dest = unsafe { std::slice::from_raw_parts_mut(addr, pixels.len()) };
    dest.copy_from_slice(&pixels);
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativePaintEnd(mut env: JNIEnv, _class: JClass, handle: jlong) -> jstring {
    let out = paint::end(handle);
    jtext(&mut env, &out)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_extemporecreare_app_components_engine_EngineBridge_nativePaintCancel(mut env: JNIEnv, _class: JClass, handle: jlong) -> jstring {
    let out = paint::cancel(handle);
    jtext(&mut env, &out)
}

// GLOSSARY
// - `text`: a Java string as Rust text.
// - `jtext`: Rust text as a Java string.
// - `Java_com_extemporecreare_app_components_engine_EngineBridge_nativeCreate`: open a session → handle.
// - `..._nativeDestroy`: close a session.
// - `..._nativeSetPresetsRoot`: set the built-in presets folder.
// - `..._nativeExecute`: run a command → JSON answer.
// - `..._nativePointer`: feed a pointer event → JSON answer with the tool's UI requests and the objects it added.
// - `..._nativeSelectTool`: make a tool active → JSON answer.
// - `..._nativeToolOptions`: the active tool's options → JSON answer.
// - `..._nativeSetToolOption`: set one option of the active tool → JSON answer.
// - `..._nativeState`: canvas mode, artboards and selection for the app's tools → JSON answer.
// - `..._nativeOverlays`: what the active tool draws over the canvas → JSON answer.
// - `..._nativeRender`: render a region of the active document into a direct byte buffer.
// - `..._nativePaintBegin`, `..._nativePaintAdd`, `..._nativePaintEnd`, `..._nativePaintCancel`: `paint::begin`, `add`, `end`, `cancel` → JSON answer.
// - `..._nativePaintPixels`: the live Brush stroke's layer over a splotch region into a direct byte buffer.
// - `..._nativeOpen`: open a copied file with its URI as the path → JSON answer.
// - `..._nativeSave`: write a save into temporary files and keep it pending → JSON answer with the files.
// - `..._nativeSaveFinish`: complete or drop the pending save → JSON answer.
// - `..._nativeValidate`: whether a file is a valid `.etc` → JSON answer.
// - `..._nativeBoardNew`: `boards::new_board` → JSON answer.
// - `..._nativeBoardRead`: `boards::read` → JSON answer.
// - `..._nativeBoardWrite`: `boards::write` → JSON answer.
// - `..._nativeBoardLinks`: `boards::links` → JSON answer.
// - `..._nativeBoardRelink`: `boards::relink_image` → JSON answer.
// - `..._nativeBoardFindMissing`: `boards::find` → JSON answer.
