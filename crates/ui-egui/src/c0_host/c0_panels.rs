use crate::{VectorcraftApp, theme};
use serde_json::{Value, json};

pub const KEYS: &[&str] = &[
    "app_bar",
    "start_screen",
    "community_links",
    "toolbar",
    "toolbar_double",
    "toolbar_floating",
    "right_dock_collapsed",
    "control_bar",
    "task_bar",
    "dock",
    "status_bar",
    "rulers",
];

/// Where the floating Tools palette appears when the host floats it: the app's own default for
/// `window.panel.float` on the tools (`floating::TOOLS_AT`), in screen points.
const FLOAT_AT: [f32; 2] = [60.0, 120.0];

/// The dock folded to its icon strip. Like `window.collapseDock`, except that it leaves the
/// `dock` switch alone, so the two keys do not depend on the order they arrive in.
fn fold_dock(app: &mut VectorcraftApp, folded: bool) {
    let shown = app.ui.dock;
    crate::dock::set_collapsed(app, folded);
    app.ui.dock = shown;
}

pub fn snapshot(app: &VectorcraftApp) -> Value {
    json!({"app_bar":app.c0.chrome.app_bar, "start_screen":app.c0.chrome.start_screen,
        "community_links":app.c0.chrome.community_links, "toolbar":app.ui.toolbar,
        "toolbar_double":app.ui.toolbar_double, "toolbar_floating":app.ui.toolbar_pos.is_some(),
        "right_dock_collapsed":app.ui.dock_collapsed,
        "control_bar":app.ui.control_bar, "task_bar":app.ui.task_bar, "dock":app.ui.dock,
        "status_bar":app.ui.status_bar, "rulers":app.ui.view.rulers})
}

pub fn set(app: &mut VectorcraftApp, ctx: &egui::Context, params: &Value) -> Value {
    let (mut applied, mut unknown, mut invalid) = (Vec::<String>::new(), Vec::<String>::new(), Vec::<String>::new());
    for (key, value) in params.as_object().into_iter().flatten() {
        match key.as_str() {
            "panels" => {
                let Some(fields) = value.as_object() else {
                    invalid.push(key.clone());
                    continue;
                };
                for (key, value) in fields {
                    let path = format!("panels.{key}");
                    if !KEYS.contains(&key.as_str()) {
                        unknown.push(path);
                        continue;
                    }
                    let Some(on) = value.as_bool() else {
                        invalid.push(path);
                        continue;
                    };
                    match key.as_str() {
                        "app_bar" => app.c0.chrome.app_bar = on,
                        "start_screen" => {
                            app.c0.chrome.start_screen = on;
                            if !on {
                                app.ui.home = None;
                            }
                        }
                        "community_links" => app.c0.chrome.community_links = on,
                        "toolbar" => app.ui.toolbar = on,
                        "toolbar_double" => app.ui.toolbar_double = on,
                        "toolbar_floating" => {
                            // Floating keeps a position the user already dragged it to.
                            app.ui.toolbar_pos = if on { app.ui.toolbar_pos.or(Some(FLOAT_AT)) } else { None };
                        }
                        "right_dock_collapsed" => fold_dock(app, on),
                        "control_bar" => app.ui.control_bar = on,
                        "task_bar" => app.ui.task_bar = on,
                        "dock" => app.ui.dock = on,
                        "status_bar" => app.ui.status_bar = on,
                        "rulers" => app.ui.view.rulers = on,
                        _ => {
                            unknown.push(path);
                            continue;
                        }
                    }
                    applied.push(path);
                }
            }
            "themeTokens" => theme::host::set(ctx, value, &mut applied, &mut unknown, &mut invalid),
            _ => unknown.push(key.clone()),
        }
    }
    theme::apply_with(
        ctx,
        app.ui.brightness,
        theme::Accent::parse(&app.session.prefs.ui_accent).unwrap_or_default(),
        theme::Scheme::parse(&app.session.prefs.ui_color_scheme).unwrap_or_default(),
    );
    theme::host::restore_canvas_preference(ctx);
    ctx.request_repaint();
    json!({"applied":applied, "unknown":unknown, "invalid":invalid})
}
