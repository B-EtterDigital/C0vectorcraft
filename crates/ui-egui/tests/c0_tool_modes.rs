#![allow(clippy::unwrap_used)]
//! The tool modes (`toolbar`, `toolbar_double`, `toolbar_floating`) and `right_dock_collapsed` of
//! `c0.set {panels}` / `c0.inspect.panes`: setting, reading back, upstream defaults, bad input.
use serde_json::{Value, json};
use vectorcraft_ui_egui::{Services, UiState, VectorcraftApp, c0_host};

fn app() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Services::default());
    app.session.prefs.interface_language = "en".into();
    app
}
fn set(app: &mut VectorcraftApp, panels: Value) -> Value {
    c0_host::handle(app, &egui::Context::default(), "c0.set", &json!({"panels": panels})).unwrap()
}
fn panes(app: &mut VectorcraftApp) -> Value {
    c0_host::handle(app, &egui::Context::default(), "c0.inspect", &json!({})).unwrap()["panes"].clone()
}
fn frames(app: &mut VectorcraftApp, ctx: &egui::Context) {
    for _ in 0..3 {
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 900.0))), ..Default::default() },
            |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
        );
        out.textures_delta.clear();
    }
}
fn floating_rect(ctx: &egui::Context) -> Option<egui::Rect> {
    ctx.memory(|m| m.area_rect(egui::Id::new("toolbar-floating")))
}

#[test]
fn defaults_equal_upstream_and_inspect_reports_all_keys() {
    let mut app = app();
    let ui = UiState::default();
    let p = panes(&mut app);
    assert_eq!(p["toolbar"], json!(ui.toolbar));
    assert_eq!(p["toolbar_double"], json!(ui.toolbar_double));
    assert_eq!(p["toolbar_floating"], json!(ui.toolbar_pos.is_some()));
    assert_eq!(p["right_dock_collapsed"], json!(ui.dock_collapsed));
    assert_eq!((ui.toolbar_double, ui.toolbar_pos, ui.dock_collapsed), (false, None, false));
    // Reading changes nothing.
    assert_eq!(panes(&mut app), p);
}

#[test]
fn each_tool_mode_is_set_and_read_back() {
    let mut app = app();
    // (toolbar_double, toolbar_floating): docked one column, docked two, floating one, floating two.
    for (double, floating) in [(true, false), (false, true), (true, true), (false, false)] {
        let r = set(&mut app, json!({"toolbar_double": double, "toolbar_floating": floating}));
        assert_eq!(r["applied"], json!(["panels.toolbar_double", "panels.toolbar_floating"]));
        assert_eq!((r["unknown"].clone(), r["invalid"].clone()), (json!([]), json!([])));
        let p = panes(&mut app);
        assert_eq!((p["toolbar_double"].as_bool(), p["toolbar_floating"].as_bool()), (Some(double), Some(floating)));
        assert_eq!(app.ui.toolbar_double, double);
        assert_eq!(app.ui.toolbar_pos.is_some(), floating);
    }
}

#[test]
fn floating_keeps_a_dragged_position_and_docking_clears_it() {
    let mut app = app();
    app.ui.toolbar_pos = Some([300.0, 200.0]);
    set(&mut app, json!({"toolbar_floating": true}));
    assert_eq!(app.ui.toolbar_pos, Some([300.0, 200.0]));
    set(&mut app, json!({"toolbar_floating": false}));
    assert_eq!(app.ui.toolbar_pos, None);
    set(&mut app, json!({"toolbar_floating": true}));
    assert!(app.ui.toolbar_pos.is_some());
}

#[test]
fn the_apps_own_toolbar_code_draws_each_mode() {
    let mut app = app();
    let ctx = egui::Context::default();
    frames(&mut app, &ctx);
    assert!(floating_rect(&ctx).is_none(), "docked: no palette");
    set(&mut app, json!({"toolbar_floating": true}));
    frames(&mut app, &ctx);
    let one = floating_rect(&ctx).expect("floating one column draws the palette");
    set(&mut app, json!({"toolbar_double": true}));
    frames(&mut app, &ctx);
    let two = floating_rect(&ctx).expect("floating two columns draws the palette");
    assert!(two.width() > one.width(), "two columns are wider: {one:?} {two:?}");
}

#[test]
fn modes_are_session_only_and_leave_preferences_alone() {
    let mut app = app();
    let prefs = serde_json::to_value(&app.session.prefs).unwrap();
    set(&mut app, json!({"toolbar_double": true, "toolbar_floating": true, "right_dock_collapsed": true}));
    assert_eq!(serde_json::to_value(&app.session.prefs).unwrap(), prefs);
}

#[test]
fn right_dock_folds_and_unfolds_without_touching_the_dock_switch() {
    let mut app = app();
    let ctx = egui::Context::default();
    frames(&mut app, &ctx);
    let r = set(&mut app, json!({"right_dock_collapsed": true}));
    assert_eq!(r["applied"], json!(["panels.right_dock_collapsed"]));
    assert_eq!(panes(&mut app)["right_dock_collapsed"], json!(true));
    frames(&mut app, &ctx);
    assert_eq!(panes(&mut app)["dock"], json!(true));
    set(&mut app, json!({"right_dock_collapsed": false}));
    assert_eq!(panes(&mut app)["right_dock_collapsed"], json!(false));
    // A hidden dock stays hidden whichever order the two keys arrive in.
    set(&mut app, json!({"dock": false, "right_dock_collapsed": true}));
    assert!(!app.ui.dock && app.ui.dock_collapsed);
    set(&mut app, json!({"right_dock_collapsed": false}));
    assert!(!app.ui.dock && !app.ui.dock_collapsed);
    frames(&mut app, &ctx);
}

#[test]
fn a_collapsed_dock_still_opens_a_panel_from_its_icon_and_unfolds_to_it() {
    let mut app = app();
    set(&mut app, json!({"right_dock_collapsed": true}));
    app.ui.open_panel = Some("layers".into());
    set(&mut app, json!({"right_dock_collapsed": false}));
    assert_eq!(app.ui.open_panel, None);
    assert_eq!(app.ui.dock_tab, vectorcraft_ui_egui::state::DockTab::Layers);
}

#[test]
fn unknown_and_invalid_keys_are_reported_and_valid_ones_still_apply() {
    let mut app = app();
    let r = set(&mut app, json!({"toolbar_triple": true, "toolbar_double": "yes", "right_dock_collapsed": true, "right_dock": true}));
    assert_eq!(r["applied"], json!(["panels.right_dock_collapsed"]));
    let mut unknown: Vec<_> = r["unknown"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
    unknown.sort();
    assert_eq!(unknown, ["panels.right_dock", "panels.toolbar_triple"]);
    assert_eq!(r["invalid"], json!(["panels.toolbar_double"]));
    assert!(!app.ui.toolbar_double && app.ui.dock_collapsed);
}
