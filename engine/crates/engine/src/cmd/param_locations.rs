//! Which command params name a file or a folder on disk ([`key_location`], [`location_in`],
//! [`location_keys`]), walked through nested params with [`ParamsPlace`].

use serde_json::Value;

use super::CommandSpec;

/// Param keys that name a file or a folder, each by its whole name, the folder preferences
/// `prefs.set {values}` takes included: a command that adds a key naming one adds it here. A key
/// that only ends like one (`alignToPath`, `linksFolder`, `sortPaths`) names none, nor do `folders`
/// (how an export groups its files) and `url` (a web link).
const FILE_KEYS: &[&str] = &[
    "path",
    "paths",
    "file",
    "files",
    "folder",
    "dir",
    "directory",
    "destinationFolder",
    "sourceFolder",
    "diffPath",
    "fontsFolder",
    "recoveryFolder",
    "templatesFolder",
];

/// The keys in [`FILE_KEYS`] that name a folder.
const FOLDER_KEYS: &[&str] =
    &["folder", "dir", "directory", "destinationFolder", "sourceFolder", "fontsFolder", "recoveryFolder", "templatesFolder"];

/// What a param key that names a place on disk names there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    /// A file (or a list of files).
    File,
    /// A folder (or a list of folders).
    Folder,
}

/// What `key` names on disk when a command takes it ([`FILE_KEYS`]), or `None` when it never
/// names a file or a folder.
pub fn key_location(key: &str) -> Option<Location> {
    if FOLDER_KEYS.contains(&key) {
        Some(Location::Folder)
    } else if FILE_KEYS.contains(&key) {
        Some(Location::File)
    } else {
        None
    }
}

/// What `key` names on disk in command `spec`: [`key_location`], or `None` when [`NOT_FILES`]
/// names it for `spec` (it names part of the document there).
pub fn location_in(spec: &CommandSpec, key: &str) -> Option<Location> {
    ParamsPlace::top(Some(spec)).member(key, None).1
}

/// A place in the params of a command. What a key names on disk depends on the command and on the
/// keys above it: [`key_location`], less a key [`NOT_FILES`] names for the command at the top of its
/// params and one at the end of a [`NESTED_NOT_FILES`] key list. A walk of params starts at
/// [`ParamsPlace::top`] and goes down with [`ParamsPlace::member`] and [`ParamsPlace::item`].
#[derive(Clone, Copy, Debug)]
pub struct ParamsPlace {
    /// The command whose params these are; `None` for a command that isn't known.
    command: Option<&'static str>,
    /// Whether this is the top of the params.
    top: bool,
    /// The keys of a [`NESTED_NOT_FILES`] list still to follow below this place.
    trail: Option<&'static [&'static str]>,
}

impl ParamsPlace {
    /// The top of the params of `spec`, or of a command that isn't known (`None`), where every key in
    /// [`FILE_KEYS`] names a file or a folder.
    pub fn top(spec: Option<&CommandSpec>) -> ParamsPlace {
        ParamsPlace { command: spec.map(|s| s.id), top: true, trail: None }
    }

    /// The top of the params of the command with id `id` ([`ParamsPlace::top`]).
    pub fn of(id: &str) -> ParamsPlace {
        ParamsPlace::top(super::find_command(id))
    }

    /// The top of the params of the command that object `fields` calls, when it is a call
    /// (`{command | id: <command id>, params}`, as a `command.batch` step or a control request is):
    /// the place of its `params` member ([`ParamsPlace::member`]).
    pub fn carried(fields: &serde_json::Map<String, Value>) -> Option<ParamsPlace> {
        if !fields.contains_key("params") {
            return None;
        }
        let id = fields.get("command").or_else(|| fields.get("id")).and_then(Value::as_str)?;
        super::find_command(id).map(|spec| ParamsPlace::top(Some(spec)))
    }

    /// The top of the params of the command that array `items` records, when it is a recorded step
    /// (`[command id, params]`): the place of its item 1 ([`ParamsPlace::item`]).
    pub fn step(items: &[Value]) -> Option<ParamsPlace> {
        match items {
            [Value::String(id), Value::Object(_)] => super::find_command(id).map(|spec| ParamsPlace::top(Some(spec))),
            _ => None,
        }
    }

    /// The place of member `key` of an object here, with what `key` names on disk there. `carried` is
    /// [`ParamsPlace::carried`] of that object: its `params` member is then the top of that command's
    /// params, and names no file.
    pub fn member(self, key: &str, carried: Option<ParamsPlace>) -> (ParamsPlace, Option<Location>) {
        if key == "params"
            && let Some(inner) = carried
        {
            return (inner, None);
        }
        let rest = if self.top {
            let id = self.command;
            NESTED_NOT_FILES.iter().find(|(c, keys)| Some(*c) == id && keys.first() == Some(&key)).map(|(_, keys)| keys.get(1..).unwrap_or_default())
        } else {
            self.trail.filter(|keys| keys.first() == Some(&key)).map(|keys| keys.get(1..).unwrap_or_default())
        };
        let document = (self.top && self.command.is_some_and(|id| NOT_FILES.contains(&(id, key)))) || rest.is_some_and(<[&str]>::is_empty);
        let place = ParamsPlace { command: self.command, top: false, trail: rest.filter(|keys| !keys.is_empty()) };
        (place, if document { None } else { key_location(key) })
    }

    /// The place of item `index` of an array here. `step` is [`ParamsPlace::step`] of that array: its
    /// item 1 is then the top of that command's params.
    pub fn item(self, index: usize, step: Option<ParamsPlace>) -> ParamsPlace {
        match step {
            Some(inner) if index == 1 => inner,
            _ => ParamsPlace { command: self.command, top: false, trail: None },
        }
    }
}

/// The param keys of `spec` that name a file or a folder, with what they name: the keys in
/// [`FILE_KEYS`] that its params description names ([`super::params::doc_keys`]), with those of the
/// commands its `…` entries name ([`super::params::takes_key`]), and its schema's params, less those
/// [`NOT_FILES`] names for it. Sorted, each key once.
pub fn location_keys(spec: &CommandSpec) -> Vec<(&'static str, Location)> {
    let described = super::params::doc_keys(spec.params).map(|p| p.keys).unwrap_or_default();
    let in_schema = spec.schema.map(|s| s.params.iter().map(|p| p.name)).into_iter().flatten();
    let taken = FILE_KEYS.iter().copied().filter(|k| super::params::takes_key(spec.params, k));
    let mut keys: Vec<(&'static str, Location)> =
        described.into_iter().chain(in_schema).chain(taken).filter_map(|k| Some((k, location_in(spec, k)?))).collect();
    keys.sort_unstable_by_key(|(k, _)| *k);
    keys.dedup();
    keys
}

/// Commands and the param of theirs, a key in [`FILE_KEYS`], that names part of the document (a path
/// of its art, a setting or an attribute it stores), not a file.
pub(crate) const NOT_FILES: &[(&str, &str)] = &[
    ("text.createInPath", "path"),
    ("symbol.instance.setAppearance", "path"),
    ("symbol.instance.selectPart", "path"),
    ("object.objectsOnPath.attach", "path"),
];

/// Commands and the keys from the top of their params down to a key in [`FILE_KEYS`] that, there,
/// names part of the document, not a file: a custom arrowhead's outline in `stroke.set`, and the
/// path fidelity setting (0 to 100) of Image Trace's tracing settings.
const NESTED_NOT_FILES: &[(&str, &[&str])] = &[
    ("stroke.set", &["startArrow", "Custom", "path"]),
    ("stroke.set", &["endArrow", "Custom", "path"]),
    ("imageTrace.make", &["params", "paths"]),
    ("imageTrace.make", &["savedPreset", "paths"]),
    ("imageTrace.makeAndExpand", &["params", "paths"]),
    ("imageTrace.makeAndExpand", &["savedPreset", "paths"]),
];
