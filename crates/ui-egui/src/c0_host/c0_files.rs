use super::{MAX_FILE_BYTES, MAX_FILE_NAME_BYTES};
use crate::VectorcraftApp;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use vectorcraft_engine::cmd::fileio;

pub(super) struct Pending {
    previous: u64,
    action: Action,
}

enum Action {
    Open(Box<fileio::ReadFile>),
    New(Value),
}

pub fn document_bytes(app: &VectorcraftApp, p: &Value) -> Result<Value, String> {
    let format = match p.get("format") {
        None => "vectorcraft",
        Some(v) => v.as_str().filter(|s| matches!(*s, "vectorcraft" | "svg" | "pdf")).ok_or("format must be vectorcraft, svg or pdf")?,
    };
    let st = app.session.active().ok_or("no document is open")?;
    let (bytes, warnings) = fileio::encode_with_warnings(&st.doc, format, &json!({})).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("document exceeds the host file limit".into());
    }
    Ok(json!({"name":snapshot_name(&st.title(), format),
        "format":format,"bytes":vectorcraft_format::base64_encode(&bytes),"revision":st.revision,"warnings":warnings}))
}

fn snapshot_name(title: &str, format: &str) -> String {
    let base = title.rsplit(['/', '\\']).find(|s| !s.is_empty()).unwrap_or("Untitled");
    let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
    let limit = MAX_FILE_NAME_BYTES.saturating_sub(format.len() + 1);
    let mut name = String::new();
    for c in stem.chars().filter(|c| *c != '\0') {
        if name.len() + c.len_utf8() > limit {
            break;
        }
        name.push(c);
    }
    if name.is_empty() {
        name.push_str("Untitled");
    }
    format!("{name}.{format}")
}

fn available(app: &VectorcraftApp) -> Result<(), String> {
    if app.c0.pending.is_some() || app.ui.dialog.is_some() || app.picks.is_waiting() {
        return Err("finish or cancel the current dialog first".into());
    }
    Ok(())
}

pub fn open(app: &mut VectorcraftApp, ctx: &egui::Context, p: &Value) -> Result<Value, String> {
    available(app)?;
    let name = p.get("name").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or("name is required")?;
    if name.len() > MAX_FILE_NAME_BYTES || name.contains(['/', '\\', '\0']) || matches!(name, "." | "..") {
        return Err(format!("name must be a file basename of at most {MAX_FILE_NAME_BYTES} bytes"));
    }
    let text = p.get("bytes").and_then(Value::as_str).ok_or("bytes must be base64 text")?;
    if text.len() > MAX_FILE_BYTES.div_ceil(3) * 4 {
        return Err("file exceeds the host file limit".into());
    }
    let bytes = vectorcraft_format::base64_decode(text).ok_or("bytes must be valid standard base64")?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err("file exceeds the host file limit".into());
    }
    if vectorcraft_format::base64_encode(&bytes) != text {
        return Err("bytes must be valid standard base64".into());
    }
    let file = fileio::read_for_open(name, &bytes, None, &json!({}), false).map_err(|e| e.to_string())?;
    cancel_picker(ctx);
    replace(app, Action::Open(Box::new(file)))
}

pub fn new_document(app: &mut VectorcraftApp, ctx: &egui::Context, p: &Value) -> Result<Value, String> {
    available(app)?;
    let mut params = json!({});
    if let Some(template) = p.get("template") {
        let name = template.as_str().ok_or("template must be a preset name")?;
        params = json!({"preset":name});
    }
    // Use the command to validate preset names/defaults in an isolated session before closing work.
    let mut scratch = vectorcraft_engine::Session::new();
    scratch.prefs = app.session.prefs.clone();
    scratch.execute("file.new", &params).map_err(|e| e.to_string())?;
    cancel_picker(ctx);
    replace(app, Action::New(params))
}

fn replace(app: &mut VectorcraftApp, action: Action) -> Result<Value, String> {
    app.c0.file_result = None;
    let Some(previous) = app.session.active().map(|d| d.uid) else { return finish(app, action) };
    app.run("file.close", json!({}))?;
    if !app.session.documents().iter().any(|d| d.uid == previous) {
        return finish(app, action);
    }
    app.c0.pending = Some(Pending { previous, action });
    Ok(json!({"pending":"saveChanges"}))
}

fn finish(app: &mut VectorcraftApp, action: Action) -> Result<Value, String> {
    let result = match action {
        Action::Open(file) => {
            let result = fileio::open_read(&mut app.session, *file).map_err(|e| e.to_string())?;
            crate::io::opened(app, &result);
            result
        }
        Action::New(params) => app.run("file.new", params)?,
    };
    app.ui.home = None;
    app.c0.file_result = Some(json!({"ok":true,"result":result}));
    Ok(result)
}

pub fn tick(app: &mut VectorcraftApp) {
    let Some(previous) = app.c0.pending.as_ref().map(|p| p.previous) else { return };
    if !app.session.documents().iter().any(|d| d.uid == previous) {
        if let Some(pending) = app.c0.pending.take()
            && let Err(e) = finish(app, pending.action)
        {
            app.status(e.clone());
            app.c0.file_result = Some(super::error_result(e));
        }
    } else if app.ui.dialog.is_none() && !app.picks.is_waiting() && app.background.jobs.is_empty() {
        app.c0.pending = None;
        app.c0.file_result = Some(json!({"cancelled":true}));
    }
}

fn picker_id() -> egui::Id {
    egui::Id::new("vectorcraft-host-picker")
}
pub fn begin_picker(ctx: &egui::Context) -> Arc<AtomicBool> {
    cancel_picker(ctx);
    let token = Arc::new(AtomicBool::new(true));
    ctx.data_mut(|d| d.insert_temp(picker_id(), token.clone()));
    token
}
pub fn picker_pending(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<Arc<AtomicBool>>(picker_id())).is_some_and(|t| t.load(Ordering::Relaxed))
}
fn cancel_picker(ctx: &egui::Context) -> bool {
    ctx.data_mut(|d| d.remove_temp::<Arc<AtomicBool>>(picker_id())).is_some_and(|t| t.swap(false, Ordering::Relaxed))
}
pub fn cancel(app: &mut VectorcraftApp, ctx: &egui::Context) -> Value {
    let picker = cancel_picker(ctx);
    let pending = app.c0.pending.take();
    if pending.as_ref().is_some_and(|pending| {
        app.ui
            .dialog
            .as_ref()
            .is_some_and(|d| d.kind == crate::unsaved::KIND && d.fields.get("uid").and_then(Value::as_u64) == Some(pending.previous))
    }) {
        app.ui.dialog = None;
    }
    if pending.is_some() {
        app.c0.file_result = Some(json!({"cancelled":true}));
    }
    // Dropping receivers invalidates late native chooser replies without trying to close OS UI.
    let native = app.picks.is_waiting();
    if native {
        app.picks = Default::default();
    }
    json!({"cancelled":picker||pending.is_some()||native})
}
