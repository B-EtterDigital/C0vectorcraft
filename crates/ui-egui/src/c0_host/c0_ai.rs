//! `c0.selection` and `c0.place`: what the C0VIBE AI actions read from the drawing and put back.
//!
//! Not upstream's: part of the C0VIBE host shell (`c0/vectorcraft-host/README.md`). Every edit goes
//! through the engine's own commands (`layer.new`, `file.place`), so history, dirty state and saving
//! behave as for a user edit, and a whole placement is ONE undo step (an engine undo scope).
//!
//! | Method | Params | Result |
//! |---|---|---|
//! | `c0.selection` | `{}` | `{kind:"artboard", artboard:{index,name,width,height}, selectedCount, selectedNames:[string], png:base64}` |
//! | `c0.place` | exactly one of `{svg, name}`, `{png, name}`, `{jpeg, name}` | `{ok:true, placed:"svg"\|"png"\|"jpeg", layerId, id, name, width, height, warnings}` |

use crate::VectorcraftApp;
use serde_json::{Value, json};
use vectorcraft_doc::NodeKind;

/// The biggest picture (or SVG text) a request may carry, decoded.
pub const MAX_PICTURE_BYTES: usize = 40 * 1024 * 1024;
/// The longest side of the picture `c0.selection` returns, in pixels.
pub const MAX_SELECTION_SIDE: u32 = 1536;
const MAX_NAMES: usize = 200;

fn png_size(b: &[u8]) -> Option<(u32, u32)> {
    if b.len() < 24 || b[..8] != [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a] {
        return None;
    }
    let n = |o: usize| u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]]);
    Some((n(16), n(20))).filter(|d| d.0 > 0 && d.1 > 0)
}

fn jpeg_ok(b: &[u8]) -> bool {
    b.len() > 4 && b[..2] == [0xff, 0xd8]
}

fn label(node: &vectorcraft_doc::Node) -> String {
    if let Some(n) = node.name.as_deref().filter(|n| !n.trim().is_empty()) {
        return n.to_owned();
    }
    match node.kind {
        NodeKind::Layer { .. } => "Layer",
        NodeKind::Group { .. } => "Group",
        NodeKind::Path { .. } => "Path",
        NodeKind::Compound { .. } => "Compound Path",
        NodeKind::Text(_) => "Text",
        NodeKind::Image(_) => "Image",
        _ => "Object",
    }
    .to_owned()
}

// ---------- c0.selection ----------

fn render(app: &mut VectorcraftApp, cmd: &str, base: Value, longest: f64) -> Result<String, String> {
    let mut scale = if longest > 0.0 { (f64::from(MAX_SELECTION_SIDE) - 2.0) / longest } else { 1.0 }.clamp(0.01, 2.0);
    for _ in 0..3 {
        let mut p = base.clone();
        p["format"] = json!("png");
        p["scale"] = json!(scale);
        p["background"] = json!("white");
        // The engine directly: the app-level command would ask for a file to write.
        let out = app.session.execute(cmd, &p).map_err(|e| e.to_string())?;
        let data = out["dataBase64"].as_str().ok_or("the picture could not be made")?.to_owned();
        let bytes = vectorcraft_format::base64_decode(&data).ok_or("the picture could not be made")?;
        let (w, h) = png_size(&bytes).ok_or("the picture could not be made")?;
        let side = w.max(h);
        if side <= MAX_SELECTION_SIDE {
            return Ok(data);
        }
        scale *= (f64::from(MAX_SELECTION_SIDE) - 2.0) / f64::from(side);
    }
    Err("the picture is too large to read".into())
}

pub fn selection(app: &mut VectorcraftApp) -> Result<Value, String> {
    let st = app.session.active().ok_or("no document is open")?;
    if st.doc.artboards.is_empty() {
        return Err("the document has no artboard".into());
    }
    let index = app.view().map_or(0, |v| v.artboard).min(st.doc.artboards.len() - 1);
    let board = &st.doc.artboards[index];
    let (name, width, height, rect) = (board.name.clone(), board.rect.width(), board.rect.height(), board.rect);
    let symbols = st.doc.symbol_boxes();
    let nodes: Vec<_> = st.selection.objects.iter().filter_map(|id| st.doc.node(*id)).collect();
    let names: Vec<String> = nodes.iter().take(MAX_NAMES).map(|n| label(n)).collect();
    let count = nodes.len();
    let bounds = nodes.iter().filter_map(|n| n.geometric_bounds_with(&symbols)).reduce(|a, b| a.union(b));
    let png = match (count, bounds) {
        (0, _) | (_, None) => render(app, "document.export", json!({"artboard": index}), rect.width().max(rect.height()))?,
        (_, Some(b)) => {
            // Effects reach past the geometry: leave room, then check the real size.
            let longest = b.width().max(b.height()) * 1.1 + 4.0;
            render(app, "document.exportSelection", json!({}), longest)?
        }
    };
    Ok(json!({
        "kind": "artboard",
        "artboard": {"index": index, "name": name, "width": width, "height": height},
        "selectedCount": count, "selectedNames": names, "png": png,
    }))
}

// ---------- c0.place ----------

fn name_param(p: &Value) -> Result<String, String> {
    match p.get("name") {
        Some(Value::String(n)) if !n.trim().is_empty() && n.len() <= super::MAX_FILE_NAME_BYTES && !n.contains(['/', '\\', '\0']) => Ok(n.trim().to_owned()),
        _ => Err("`name` must be a short name without slashes".into()),
    }
}

pub fn place(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let kinds: Vec<&str> = ["svg", "png", "jpeg"].into_iter().filter(|k| p.get(*k).is_some_and(|v| !v.is_null())).collect();
    let kind = match kinds.as_slice() {
        [k] => *k,
        [] => return Err("nothing to place: give `svg`, `png` or `jpeg`".into()),
        _ => return Err("place one thing at a time: give only one of `svg`, `png` or `jpeg`".into()),
    };
    let text = p[kind].as_str().ok_or_else(|| format!("`{kind}` must be {}", if kind == "svg" { "SVG text" } else { "base64 text" }))?;
    if text.len() > MAX_PICTURE_BYTES / 3 * 4 + 1024 {
        return Err("the data is larger than 40 MB".into());
    }
    let name = name_param(p)?;
    // The file's bytes, base64, in the form `file.place` reads.
    let (file_name, data) = if kind == "svg" {
        if text.len() > MAX_PICTURE_BYTES {
            return Err("the SVG is larger than 40 MB".into());
        }
        if !text.contains("<svg") {
            return Err("`svg` is not SVG markup".into());
        }
        (format!("{name}.svg"), vectorcraft_format::base64_encode(text.as_bytes()))
    } else {
        let bytes = vectorcraft_format::base64_decode(text).filter(|b| !b.is_empty()).ok_or("`png`/`jpeg` is not valid base64")?;
        if bytes.len() > MAX_PICTURE_BYTES {
            return Err("the picture is larger than 40 MB".into());
        }
        let readable = if kind == "png" { png_size(&bytes).is_some() } else { jpeg_ok(&bytes) };
        if !readable {
            return Err(format!("the data is not a readable {}", kind.to_uppercase()));
        }
        (format!("{name}.{}", if kind == "png" { "png" } else { "jpg" }), text.to_owned())
    };
    let st = app.session.active().ok_or("no document is open")?;
    if st.doc.artboards.is_empty() {
        return Err("the document has no artboard".into());
    }
    let board = st.doc.artboards[app.view().map_or(0, |v| v.artboard).min(st.doc.artboards.len() - 1)].rect;
    let rect = [board.x0, board.y0, board.width(), board.height()];
    let scope = app.session.begin_undo_scope().ok_or("no document is open")?;
    let result = (|| {
        let layer = app.run("layer.new", json!({"name": name, "top": true}))?;
        let placed = app.run("file.place", json!({"name": file_name, "dataBase64": data, "rect": rect}))?;
        let id = placed["ids"][0].clone();
        Ok(json!({
            "ok": true, "placed": kind, "layerId": layer["id"], "id": id, "name": name,
            "width": placed["width"], "height": placed["height"], "warnings": placed["warnings"],
        }))
    })();
    app.session.end_undo_scope(scope, result.is_err());
    result
}
