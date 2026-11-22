//! Window → Attributes: centre point display, image map, URL and note
//! (`attributes.set`), the fill rule (`path.setFillRule`) and, through `path.reverse`, the path
//! direction. The panel reads everything back with `attributes.info` ([`attributes`]).

use std::collections::BTreeSet;

use serde_json::{Value, json};
use vectorcraft_doc::{Document, ImageMap, Node, NodeKind, ObjectAttributes};
use vectorcraft_geom::FillRule;

use super::edit_counted;
use super::path::paths_in;
use super::{CommandSpec, NodeId, Result, Session, bad, cmd, has_doc, str_param, targets};

/// How many URLs the Attributes panel's recent list keeps: the most its Panel Options lets the
/// URL menu list.
pub const RECENT_URLS: usize = 30;

/// The Attributes panel's values ([`Session::attributes_info`], `attributes.info`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AttributesInfo {
    /// The targets (the selection by default).
    pub ids: Vec<NodeId>,
    /// The rest of the panel; `None` where the targets differ.
    pub show_center: Option<bool>,
    pub image_map: Option<ImageMap>,
    pub url: Option<String>,
    pub note: Option<String>,
    /// Fill rule of the paths and compound paths in the targets.
    pub fill_rule: Option<FillRule>,
    /// Whether their subpaths run counter-clockwise on screen (Reverse Path Direction On).
    pub reversed: Option<bool>,
    /// [`Session::recent_urls`].
    pub recent_urls: Vec<String>,
}

impl AttributesInfo {
    fn to_json(&self) -> Value {
        json!({
            "ids": self.ids.iter().map(|id| id.0).collect::<Vec<_>>(),
            "showCenter": self.show_center,
            "imageMap": self.image_map,
            "url": self.url,
            "note": self.note,
            "fillRule": self.fill_rule.map(rule_name),
            "reversed": self.reversed,
            "recentUrls": self.recent_urls,
        })
    }
}

impl Session {
    /// The Attributes panel's values for the selection.
    pub fn attributes_info(&self) -> AttributesInfo {
        attributes(self, &Value::Null).unwrap_or_default()
    }
}

fn attributes(s: &Session, p: &Value) -> Result<AttributesInfo> {
    let d = &s.doc()?.doc;
    let mut info = AttributesInfo { ids: targets(s, p)?, recent_urls: s.recent_urls.clone(), ..Default::default() };
    fill_info(d, &mut info);
    Ok(info)
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "attributes.info",
            "Attributes",
            [],
            None,
            "{ids?|id?, target?: \"object\"|\"contents\"} → {ids, showCenter, imageMap: \"none\"|\"rectangle\"|\"polygon\", url, note, fillRule: \"nonZero\"|\"evenOdd\" (of the paths and compound paths in them), reversed (their subpaths run counter-clockwise on screen: Reverse Path Direction On), recentUrls: [newest first, at most 30]} the Attributes panel's values for `ids` or the selection; a value is null where they differ (or nothing has a path)",
            has_doc,
            |s, p| Ok(attributes(s, p)?.to_json())
        ),
        cmd!(
            "attributes.set",
            "Attributes",
            [],
            None,
            "{showCenter?: bool (the canvas shows the selected object's centre point), imageMap?: \"none\"|\"rectangle\"|\"polygon\", url?: string (\"\" removes it; SVG, PDF and .ai export link the object), note?: string, ids?|id?, target?: \"object\"|\"contents\"} set the Attributes panel's values on ids or the selection, as one undo step; a URL given joins the recent URLs (attributes.info, newest first, at most 30) → {changed: objects}",
            has_doc,
            set
        ),
        cmd!(
            "path.setFillRule",
            "Fill Rule",
            [],
            None,
            "{rule: \"nonZero\"|\"evenOdd\", ids?|id?} the fill rule of the paths and compound paths in ids or the selection (groups: those inside), as one undo step → {changed}",
            has_doc,
            set_fill_rule
        ),
    ]
}

/// A fill rule as the commands name it.
pub(super) fn rule_name(r: FillRule) -> &'static str {
    match r {
        FillRule::NonZero => "nonZero",
        FillRule::EvenOdd => "evenOdd",
    }
}

fn parse_rule(s: &str) -> Option<FillRule> {
    match s.to_ascii_lowercase().replace(['-', ' ', '_'], "").as_str() {
        "nonzero" | "nonzerowinding" => Some(FillRule::NonZero),
        "evenodd" => Some(FillRule::EvenOdd),
        _ => None,
    }
}

/// The one value every item of `it` shares, if any.
fn same<T: PartialEq>(mut it: impl Iterator<Item = T>) -> Option<T> {
    let first = it.next()?;
    it.all(|v| v == first).then_some(first)
}

/// The paths and compound paths in `n`'s subtree that have a fill rule of their own with it (not a
/// compound path's members, which fill with its rule).
fn ruled<'a>(n: &'a Node, out: &mut Vec<(&'a Node, FillRule)>) {
    match &n.kind {
        NodeKind::Path { rule, guide: false, .. } | NodeKind::Compound { rule, .. } => out.push((n, *rule)),
        _ => n.children().into_iter().flatten().for_each(|c| ruled(c, out)),
    }
}

/// A directly selected compound member fills with its compound's rule. Its other attributes
/// and path direction still belong to the member itself.
fn fill_rule_owner(d: &Document, id: NodeId) -> Option<&Node> {
    let n = d.node(id)?;
    match d.parent_of(id).and_then(|parent| d.node(parent)) {
        Some(parent) if matches!(parent.kind, NodeKind::Compound { .. }) => Some(parent),
        _ => Some(n),
    }
}

/// The Attributes panel's values of `info.ids`.
pub(super) fn fill_info(d: &Document, info: &mut AttributesInfo) {
    let nodes: Vec<&Node> = info.ids.iter().filter_map(|id| d.node(*id)).collect();
    let attrs = || nodes.iter().map(|n| n.extras().attrs.as_deref().cloned().unwrap_or_default());
    info.show_center = same(nodes.iter().map(|n| n.shows_center()));
    info.image_map = same(attrs().map(|a| a.image_map));
    info.url = same(attrs().map(|a| a.url));
    info.note = same(attrs().map(|a| a.note));
    let mut all = vec![];
    nodes.iter().filter_map(|n| fill_rule_owner(d, n.id)).for_each(|n| ruled(n, &mut all));
    info.fill_rule = same(all.into_iter().map(|(_, r)| r));
    let paths = paths_in(d, &info.ids);
    let subpaths = paths.iter().filter_map(|id| d.node(*id)?.path_data()).flat_map(|p| &p.subpaths);
    info.reversed = same(subpaths.map(|sp| sp.signed_area() < 0.0));
}

/// The attribute edits `p` asks for, applied to `a`.
#[derive(Default)]
struct Edits {
    show_center: Option<bool>,
    image_map: Option<ImageMap>,
    url: Option<String>,
    note: Option<String>,
}

impl Edits {
    fn parse(p: &Value) -> Result<Self> {
        const C: &str = "attributes.set";
        let text = |k: &str| -> Result<Option<String>> {
            p.get(k).map(|v| v.as_str().map(|s| s.trim().to_string()).ok_or_else(|| bad(C, format!("`{k}` must be a string")))).transpose()
        };
        Ok(Self {
            show_center: p.get("showCenter").map(|v| v.as_bool().ok_or_else(|| bad(C, "`showCenter` must be true or false"))).transpose()?,
            image_map: text("imageMap")?
                .map(|s| ImageMap::parse(&s).ok_or_else(|| bad(C, "`imageMap` must be none, rectangle or polygon")))
                .transpose()?,
            url: text("url")?,
            note: text("note")?,
        })
    }

    fn is_empty(&self) -> bool {
        self.show_center.is_none() && self.image_map.is_none() && self.url.is_none() && self.note.is_none()
    }

    /// `n`'s attributes with the edits applied (Show Center kept only where it isn't the default).
    fn apply(&self, n: &Node) -> ObjectAttributes {
        let mut a = n.extras().attrs.as_deref().cloned().unwrap_or_default();
        if let Some(v) = self.show_center {
            a.show_center = (v != n.shows_center_by_default()).then_some(v);
        }
        if let Some(v) = self.image_map {
            a.image_map = v;
        }
        if let Some(v) = &self.url {
            a.url.clone_from(v);
        }
        if let Some(v) = &self.note {
            a.note.clone_from(v);
        }
        a
    }
}

fn set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "attributes.set";
    let edits = Edits::parse(p)?;
    if edits.is_empty() {
        return Err(bad(C, "give showCenter, imageMap, url or note"));
    }
    let ids = targets(s, p)?;
    let changed = edit_counted(s, "Attributes", |d| {
        let mut changed = BTreeSet::new();
        if !edits.is_empty() {
            for id in &ids {
                let Some(n) = d.node(*id) else { continue };
                let new = edits.apply(n);
                if new != n.extras().attrs.as_deref().cloned().unwrap_or_default()
                    && let Some(n) = d.node_mut(*id)
                {
                    n.edit_attrs(|a| *a = new);
                    changed.insert(*id);
                }
            }
        }
        Ok(changed.len())
    })?;
    if let Some(url) = edits.url.filter(|u| !u.is_empty() && !ids.is_empty()) {
        s.recent_urls.retain(|u| *u != url);
        s.recent_urls.insert(0, url);
        s.recent_urls.truncate(RECENT_URLS);
    }
    Ok(json!({ "changed": changed }))
}

fn set_fill_rule(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "path.setFillRule";
    let rule = str_param(p, "rule").and_then(parse_rule).ok_or_else(|| bad(C, "`rule` must be nonZero or evenOdd"))?;
    let ids = targets(s, p)?;
    let changed = edit_counted(s, "Fill Rule", |d| {
        let mut all = vec![];
        ids.iter().filter_map(|id| fill_rule_owner(d, *id)).for_each(|n| ruled(n, &mut all));
        let todo: BTreeSet<_> = all.into_iter().filter(|(_, r)| *r != rule).map(|(n, _)| n.id).collect();
        for id in &todo {
            if let Some(NodeKind::Path { rule: r, .. } | NodeKind::Compound { rule: r, .. }) = d.node_mut(*id).map(|n| &mut n.kind) {
                *r = rule;
            }
        }
        Ok(todo.len())
    })?;
    Ok(json!({ "changed": changed }))
}
