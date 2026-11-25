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

use crate::{calls, sessions};

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

// GLOSSARY
// - `text`: a Java string as Rust text.
// - `jtext`: Rust text as a Java string.
// - `Java_com_extemporecreare_app_components_engine_EngineBridge_nativeCreate`: open a session → handle.
// - `..._nativeDestroy`: close a session.
// - `..._nativeSetPresetsRoot`: set the built-in presets folder.
// - `..._nativeExecute`: run a command → JSON answer.
// - `..._nativePointer`: feed a pointer event → JSON answer with the tool's UI requests.
// - `..._nativeSelectTool`: make a tool active → JSON answer.
// - `..._nativeRender`: render a region of the active document into a direct byte buffer.
