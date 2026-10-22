#![allow(clippy::unwrap_used, clippy::panic)]
use serde_json::{Value, json};
use vectorcraft_engine::Session;
use vectorcraft_ui_egui::{ControlRequest, Services, VectorcraftApp, control};

fn app() -> VectorcraftApp {
    VectorcraftApp::new(Session::new(), Services::default())
}
fn call(app: &mut VectorcraftApp, ctx: &egui::Context, m: &str, p: Value) -> Value {
    let (req, _) = ControlRequest::new(m, p);
    let control::Outcome::Done(reply) = control::handle(app, ctx, &req) else { panic!("host reply needs presentation") };
    reply
}
fn ok(app: &mut VectorcraftApp, ctx: &egui::Context, m: &str, p: Value) -> Value {
    let reply = call(app, ctx, m, p);
    assert_eq!(reply["ok"], true, "{reply}");
    reply["result"].clone()
}
fn new(app: &mut VectorcraftApp) {
    app.run("file.new", json!({"width":120,"height":80})).unwrap();
}
fn dirty(app: &mut VectorcraftApp) {
    app.run("shape.rectangle", json!({"x":1,"y":2,"width":10,"height":20})).unwrap();
}
fn svg() -> Value {
    json!({"name":"real.svg","bytes":vectorcraft_format::base64_encode(br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="60"><rect x="2" y="3" width="12" height="14" fill="#28a"/></svg>"##)})
}

#[test]
fn empty_inspection_has_complete_host_mirror() {
    let (mut app, ctx) = (app(), egui::Context::default());
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    for k in ["revision", "dirty", "canUndo", "canRedo", "panes", "theme", "artboards", "layers", "selectionCount", "activeArtboard", "documents"] {
        assert!(v.get(k).is_some(), "{k}");
    }
    assert_eq!(v["documents"], json!([]));
    assert_eq!(v["dirty"], false);
    assert_eq!(v["activeArtboard"], Value::Null);
}

#[test]
fn inspect_tracks_history_selection_layers_and_documents() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    dirty(&mut app);
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["artboards"][0]["width"], 120.0);
    assert_eq!(v["artboards"][0]["active"], true);
    assert!(v["layers"][0]["id"].is_u64());
    assert_eq!(v["layers"][0]["depth"], 0);
    assert_eq!(v["selectionCount"], 1);
    assert_eq!(v["dirty"], true);
    assert_eq!(v["canUndo"], true);
    assert_eq!(v["canRedo"], false);
    app.run("edit.undo", json!({})).unwrap();
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["canRedo"], true);
    new(&mut app);
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["documents"].as_array().unwrap().len(), 2);
    assert_eq!(v["documents"][1]["active"], true);
}

#[test]
fn navigation_validates_both_targets_and_uses_app_commands() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    app.run("artboard.new", json!({"name":"Second","x":150,"y":0,"width":50,"height":60})).unwrap();
    let id = app.session.active().unwrap().doc.layers[0].id.0;
    let v = ok(&mut app, &ctx, "c0.goTo", json!({"artboard":1,"layer":id}));
    assert_eq!(v["artboard"], 1);
    assert_eq!(v["layer"], id);
    assert_eq!(call(&mut app, &ctx, "c0.goTo", json!({"artboard":0,"layer":u64::MAX}))["ok"], false);
    assert_eq!(app.view().unwrap().artboard, 1);
    for p in [json!({}), json!({"artboard":-1}), json!({"artboard":10}), json!({"layer":"x"})] {
        assert_eq!(call(&mut app, &ctx, "c0.goTo", p)["ok"], false);
    }
}

#[test]
fn layer_mirror_includes_one_nesting_level() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    let root = app.session.active().unwrap().doc.layers[0].id.0;
    let child = app.run("layer.newSublayer", json!({"parent":root,"name":"Child"})).unwrap()["id"].as_u64().unwrap();
    app.run("layer.newSublayer", json!({"parent":child,"name":"Grandchild"})).unwrap();
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["layers"].as_array().unwrap().len(), 2);
    assert_eq!(v["layers"][1]["name"], "Child");
    assert_eq!(v["layers"][1]["depth"], 1);
}

#[test]
fn document_bytes_are_read_only_and_all_formats_open() {
    for format in ["vectorcraft", "svg", "pdf"] {
        let (mut app, ctx) = (app(), egui::Context::default());
        new(&mut app);
        dirty(&mut app);
        app.run("edit.undo", json!({})).unwrap();
        let st = app.session.active().unwrap();
        let before = (st.revision, st.is_dirty(), st.history.undo.len(), st.history.redo.len(), st.path.clone());
        let bytes = ok(&mut app, &ctx, "c0.documentBytes", json!({"format":format}));
        let st = app.session.active().unwrap();
        assert_eq!(before, (st.revision, st.is_dirty(), st.history.undo.len(), st.history.redo.len(), st.path.clone()));
        let mut other = self::app();
        ok(&mut other, &ctx, "c0.openFile", json!({"name":bytes["name"],"bytes":bytes["bytes"]}));
        assert_eq!(other.session.documents().len(), 1);
        assert_eq!(other.session.active().unwrap().doc.artboards[0].rect.width(), 120.0);
    }
}

#[test]
fn invalid_files_do_not_discard_work_or_open_questions() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    dirty(&mut app);
    let uid = app.session.active().unwrap().uid;
    for p in [json!({}), json!({"name":"../x.svg","bytes":""}), json!({"name":"x.svg","bytes":"???"}), json!({"name":"x.svg","bytes":"bm90IHN2Zw=="})]
    {
        assert_eq!(call(&mut app, &ctx, "c0.openFile", p)["ok"], false);
        assert_eq!(app.session.active().unwrap().uid, uid);
        assert!(app.ui.dialog.is_none());
    }
}

#[test]
fn snapshot_name_is_a_bounded_basename_even_for_an_unusual_document_title() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    std::sync::Arc::make_mut(&mut app.session.active_mut().unwrap().doc).title = format!("folder/{}\0.svg", "é".repeat(300));
    let v = ok(&mut app, &ctx, "c0.documentBytes", json!({}));
    let name = v["name"].as_str().unwrap();
    assert!(name.len() <= vectorcraft_ui_egui::c0_host::MAX_FILE_NAME_BYTES);
    assert!(!name.contains(['/', '\\', '\0']));
    let mut other = self::app();
    ok(&mut other, &ctx, "c0.openFile", json!({"name":name,"bytes":v["bytes"]}));
}

#[test]
fn snapshot_does_not_commit_or_cancel_a_tool_preview() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    app.select_tool("rectangle");
    ok(&mut app, &ctx, "ui.pointer", json!({"events":[{"kind":"down","x":10,"y":10},{"kind":"drag","x":40,"y":30}]}));
    assert!(app.session.tool_busy());
    let st = app.session.active().unwrap();
    let before = (st.revision, st.history.undo.len(), st.history.redo.len(), st.doc.clone());
    ok(&mut app, &ctx, "c0.documentBytes", json!({}));
    let st = app.session.active().unwrap();
    assert_eq!((st.revision, st.history.undo.len(), st.history.redo.len()), (before.0, before.1, before.2));
    assert!(std::sync::Arc::ptr_eq(&st.doc, &before.3));
    assert!(app.session.tool_busy());
}

#[test]
fn clean_replacement_and_new_template_use_upstream_loader() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    ok(&mut app, &ctx, "c0.openFile", svg());
    assert_eq!(app.session.documents().len(), 1);
    assert_eq!(app.session.active().unwrap().doc.artboards[0].rect.width(), 40.0);
    ok(&mut app, &ctx, "c0.newDocument", json!({"template":"A4"}));
    assert_eq!(app.session.documents().len(), 1);
    assert!(app.session.active().unwrap().doc.artboards[0].rect.width() > 500.0);
    assert_eq!(call(&mut app, &ctx, "c0.newDocument", json!({"template":"missing-preset"}))["ok"], false);
}

#[test]
fn dirty_replacement_waits_for_discard_and_cancel_preserves_document() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    dirty(&mut app);
    let uid = app.session.active().unwrap().uid;
    assert_eq!(ok(&mut app, &ctx, "c0.openFile", svg())["pending"], "saveChanges");
    assert_eq!(app.session.active().unwrap().uid, uid);
    ok(&mut app, &ctx, "ui.dialog.cancel", json!({}));
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["pendingFile"], false);
    assert_eq!(app.session.active().unwrap().uid, uid);
    ok(&mut app, &ctx, "c0.openFile", svg());
    ok(&mut app, &ctx, "ui.dialog.set", json!({"field":"discard","value":true}));
    ok(&mut app, &ctx, "ui.dialog.confirm", json!({}));
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["pendingFile"], false);
    assert_eq!(v["fileResult"]["ok"], true);
    assert_eq!(app.session.documents().len(), 1);
    assert_ne!(app.session.active().unwrap().uid, uid);
}

#[test]
fn replacement_keeps_other_tabs_including_their_unsaved_work() {
    let (mut app, ctx) = (app(), egui::Context::default());
    new(&mut app);
    dirty(&mut app);
    let background = app.session.active().unwrap().uid;
    new(&mut app);
    let replaced = app.session.active().unwrap().uid;
    ok(&mut app, &ctx, "c0.openFile", svg());
    assert!(app.ui.dialog.is_none());
    assert_eq!(app.session.documents().len(), 2);
    assert!(app.session.documents().iter().any(|d| d.uid == background && d.is_dirty()));
    assert!(!app.session.documents().iter().any(|d| d.uid == replaced));
    ok(&mut app, &ctx, "c0.newDocument", json!({"template":"A4"}));
    assert_eq!(app.session.documents().len(), 2);
    assert!(app.session.documents().iter().any(|d| d.uid == background && d.is_dirty()));
    assert_ne!(app.session.active().unwrap().uid, background);
}

#[test]
fn save_before_replacement_and_cancelled_save_preserve_lifecycle() {
    let (mut app, ctx) = (app(), egui::Context::default());
    app.services.pick_save = Some(Box::new(|_| Some("/tmp/c0-test.vectorcraft".into())));
    app.services.write = Some(Box::new(|_, _| Ok(())));
    new(&mut app);
    dirty(&mut app);
    ok(&mut app, &ctx, "c0.newDocument", json!({}));
    ok(&mut app, &ctx, "ui.dialog.confirm", json!({}));
    let v = ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(v["pendingFile"], false);
    assert_eq!(v["fileResult"]["ok"], true);
    dirty(&mut app);
    let uid = app.session.active().unwrap().uid;
    app.services.pick_save = Some(Box::new(|_| None));
    ok(&mut app, &ctx, "c0.openFile", svg());
    assert_eq!(call(&mut app, &ctx, "ui.dialog.confirm", json!({}))["ok"], false);
    ok(&mut app, &ctx, "c0.inspect", json!({}));
    assert_eq!(app.session.active().unwrap().uid, uid);
}

#[test]
fn cancel_invalidates_late_picker_and_pending_replacement() {
    let (mut app, ctx) = (app(), egui::Context::default());
    let token = vectorcraft_ui_egui::c0_host::begin_picker(&ctx);
    assert_eq!(ok(&mut app, &ctx, "c0.cancelFileDialog", json!({}))["cancelled"], true);
    assert!(!token.load(std::sync::atomic::Ordering::Relaxed));
    assert_eq!(ok(&mut app, &ctx, "c0.cancelFileDialog", json!({}))["cancelled"], false);
    new(&mut app);
    dirty(&mut app);
    ok(&mut app, &ctx, "c0.openFile", svg());
    assert_eq!(ok(&mut app, &ctx, "c0.cancelFileDialog", json!({}))["cancelled"], true);
    assert!(app.ui.dialog.is_none());
    assert!(app.session.active().unwrap().is_dirty());
}

#[test]
fn settings_report_every_applied_unknown_and_invalid_path() {
    let (mut app, ctx) = (app(), egui::Context::default());
    let mut v = ok(
        &mut app,
        &ctx,
        "c0.set",
        json!({"panels":{"app_bar":false,"toolbar":false,"rulers":false,"dock":"no","new":true},"themeTokens":{"text":"#123456","panel":"bad","nope":"#123456"},"extra":2}),
    );
    // JSON object iteration order is not part of the host contract.
    for key in ["applied", "unknown", "invalid"] {
        v.get_mut(key).and_then(Value::as_array_mut).unwrap().sort_by_key(|value| value.as_str().unwrap_or_default().to_owned());
    }
    assert_eq!(v["applied"], json!(["panels.app_bar", "panels.rulers", "panels.toolbar", "themeTokens.text"]));
    assert_eq!(v["unknown"], json!(["extra", "panels.new", "themeTokens.nope"]));
    assert_eq!(v["invalid"], json!(["panels.dock", "themeTokens.panel"]));
    assert!(!app.c0.chrome.app_bar);
    assert!(!app.ui.toolbar);
    assert!(!app.ui.view.rulers);
    assert_eq!(call(&mut app, &ctx, "c0.missing", json!({}))["ok"], false);
    assert_eq!(call(&mut app, &ctx, "c0.set", json!([]))["ok"], false);
    assert_eq!(call(&mut app, &ctx, "c0.documentBytes", json!({}))["ok"], false);
}
