//! The engine sessions the UI holds, by handle. A handle is a plain number the Kotlin side keeps;
//! the session itself stays here, so no pointer crosses the JNI boundary.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use vectorcraft_engine::Session;

/// One open engine session, with the save it is waiting to finish ([`crate::files::save`]) and the
/// objects the document had when the pointer was last pressed ([`crate::calls::pointer`]).
pub struct Bridge {
    pub session: Session,
    pub pending: Option<crate::files::PendingSave>,
    pub gesture_ids: Option<std::collections::HashSet<u64>>,
}

/// The next handle to give out (0 is never a valid handle).
static NEXT: AtomicI64 = AtomicI64::new(1);

fn table() -> &'static Mutex<HashMap<i64, Arc<Mutex<Bridge>>>> {
    static TABLE: OnceLock<Mutex<HashMap<i64, Arc<Mutex<Bridge>>>>> = OnceLock::new();
    TABLE.get_or_init(Default::default)
}

/// Open a new session; returns its handle.
pub fn create() -> i64 {
    let handle = NEXT.fetch_add(1, Ordering::Relaxed);
    let bridge = Bridge { session: Session::new(), pending: None, gesture_ids: None };
    table().lock().unwrap_or_else(|e| e.into_inner()).insert(handle, Arc::new(Mutex::new(bridge)));
    handle
}

/// Close session `handle` (an unknown handle is ignored).
pub fn destroy(handle: i64) {
    table().lock().unwrap_or_else(|e| e.into_inner()).remove(&handle);
}

/// Run `f` on session `handle`; `None` when there is no such session.
pub fn with<T>(handle: i64, f: impl FnOnce(&mut Bridge) -> T) -> Option<T> {
    let bridge = table().lock().unwrap_or_else(|e| e.into_inner()).get(&handle).cloned()?;
    let mut guard = bridge.lock().unwrap_or_else(|e| e.into_inner());
    Some(f(&mut guard))
}

// GLOSSARY
// - `Bridge`: one open engine session and its pending save.
// - `NEXT`: the next handle to give out.
// - `table`: the open sessions by handle.
// - `create`: open a session and return its handle.
// - `destroy`: close a session.
// - `with`: run a closure on a session by handle.
