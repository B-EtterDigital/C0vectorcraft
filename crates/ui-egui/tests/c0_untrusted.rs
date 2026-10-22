#![allow(clippy::unwrap_used, clippy::panic)]
//! Bounded protocol corpus: run handlers directly so the outer panic guard cannot hide a bug.
use serde_json::{Value, json};
use vectorcraft_ui_egui::{ControlRequest, Services, VectorcraftApp, control};

fn app() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Services::default());
    app.run("file.new", json!({})).unwrap();
    app.run("shape.rectangle", json!({"x":1,"y":2,"width":3,"height":4})).unwrap();
    app
}

#[test]
fn parameter_type_sweep_preserves_unsaved_work() {
    let values = [Value::Null, json!(true), json!(-1), json!(u64::MAX), json!("\0é"), json!([]), json!({}), json!([{}, null])];
    for method in ["c0.inspect", "c0.set", "c0.goTo", "c0.documentBytes", "c0.openFile", "c0.newDocument", "c0.cancelFileDialog"] {
        for value in &values {
            let mut app = app();
            let uid = app.session.active().unwrap().uid;
            let document = app.session.active().unwrap().doc.clone();
            let ctx = egui::Context::default();
            for params in [
                value.clone(),
                json!({"name":value,"bytes":value,"format":value,"template":value,"artboard":value,"layer":value,"panels":value,"themeTokens":value}),
            ] {
                let (request, _) = ControlRequest::new(method, params);
                let control::Outcome::Done(reply) = control::handle(&mut app, &ctx, &request) else { panic!("host handler deferred presentation") };
                assert!(reply["ok"].is_boolean(), "{method}: {reply}");
                let st = app.session.active().unwrap();
                assert_eq!(st.uid, uid, "{method}: {value}");
                assert!(std::sync::Arc::ptr_eq(&st.doc, &document), "{method}: {value}");
                assert!(st.is_dirty());
            }
        }
    }
}

#[test]
fn malformed_svg_byte_sweep_never_discards_the_current_document() {
    let mut app = app();
    let uid = app.session.active().unwrap().uid;
    let ctx = egui::Context::default();
    for seed in 0u8..128 {
        let bytes: Vec<_> = (0..usize::from(seed)).map(|i| seed.wrapping_add(i as u8)).collect();
        let (request, _) = ControlRequest::new("c0.openFile", json!({"name":"input.svg","bytes":vectorcraft_format::base64_encode(&bytes)}));
        let control::Outcome::Done(reply) = control::handle(&mut app, &ctx, &request) else { panic!("host handler deferred presentation") };
        assert_eq!(reply["ok"], false, "seed {seed}: {reply}");
        assert_eq!(app.session.active().unwrap().uid, uid);
        assert!(app.session.active().unwrap().is_dirty());
        assert!(app.ui.dialog.is_none());
    }
}
