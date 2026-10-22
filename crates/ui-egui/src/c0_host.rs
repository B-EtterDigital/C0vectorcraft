//! Session-only guest contract. Defaults retain the upstream desktop interface.
use crate::VectorcraftApp;
use serde_json::{Value, json};

mod c0_files;
mod c0_panels;
mod c0_state;
pub use c0_files::{begin_picker, picker_pending};

pub const MAX_FILE_BYTES: usize = 72 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 100 * 1024 * 1024;
pub const MAX_FILE_NAME_BYTES: usize = 255;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chrome {
    pub app_bar: bool,
    pub start_screen: bool,
    pub community_links: bool,
}
impl Default for Chrome {
    fn default() -> Self {
        Self { app_bar: true, start_screen: true, community_links: true }
    }
}

#[derive(Default)]
pub struct Host {
    pub chrome: Chrome,
    pub page_attached: bool,
    pending: Option<c0_files::Pending>,
    pub file_result: Option<Value>,
}

pub fn allow(app: &VectorcraftApp, id: &str) -> Result<(), String> {
    if !app.c0.chrome.community_links && matches!(id, "help.discord" | "help.website" | "help.appPage" | "help.github" | "help.links") {
        return Err("community links are owned by the host".into());
    }
    if !app.c0.chrome.start_screen && id == "app.home" {
        return Err("Home is owned by the host".into());
    }
    Ok(())
}

pub fn handle(app: &mut VectorcraftApp, ctx: &egui::Context, method: &str, params: &Value) -> Result<Value, String> {
    if !params.is_object() {
        return Err("params must be an object".into());
    }
    if method != "c0.documentBytes" {
        tick(app);
    }
    match method {
        "c0.inspect" => Ok(c0_state::inspect(app, ctx)),
        "c0.goTo" => c0_state::go_to(app, params),
        "c0.documentBytes" => c0_files::document_bytes(app, params),
        "c0.openFile" => c0_files::open(app, ctx, params),
        "c0.newDocument" => c0_files::new_document(app, ctx, params),
        "c0.cancelFileDialog" => Ok(c0_files::cancel(app, ctx)),
        "c0.set" => Ok(c0_panels::set(app, ctx, params)),
        _ => Err(format!("unknown host method `{method}`")),
    }
}

pub fn tick(app: &mut VectorcraftApp) {
    c0_files::tick(app);
}

pub fn palette_match(app: &VectorcraftApp, id: &str, haystack: &str, query: &str) -> bool {
    allow(app, id).is_ok() && query.split_whitespace().all(|word| haystack.contains(word))
}

pub(super) fn error_result(error: impl std::fmt::Display) -> Value {
    json!({"ok": false, "error": error.to_string()})
}
