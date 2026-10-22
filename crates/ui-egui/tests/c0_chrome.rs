#![allow(clippy::unwrap_used)]
use serde_json::json;
use vectorcraft_ui_egui::{ControlRequest, Services, UiState, VectorcraftApp, c0_host, menus};

fn app() -> VectorcraftApp {
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Services::default());
    app.session.prefs.interface_language = "en".into();
    app
}
fn texts(shapes: &[egui::epaint::ClippedShape]) -> String {
    fn visit(shape: &egui::Shape, out: &mut String) {
        match shape {
            egui::Shape::Text(t) => {
                out.push_str(t.galley.text());
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    visit(s, out);
                }
            }
            _ => {}
        }
    }
    let mut out = String::new();
    for s in shapes {
        visit(&s.shape, &mut out);
    }
    out
}
fn render(app: &mut VectorcraftApp, ctx: &egui::Context) -> String {
    let mut out = String::new();
    for _ in 0..3 {
        let mut frame = ctx.run_ui(
            egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 900.0))), ..Default::default() },
            |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            },
        );
        out = texts(&frame.shapes);
        // This headless pass inspects shapes without uploading textures to a renderer.
        frame.textures_delta.clear();
    }
    out
}
#[test]
fn chrome_defaults_keep_upstream_interface_and_preferences() {
    let mut app = app();
    let ctx = egui::Context::default();
    let prefs = serde_json::to_value(&app.session.prefs).unwrap();
    assert_eq!(app.c0.chrome, c0_host::Chrome::default());
    assert!(app.c0.chrome.app_bar && app.c0.chrome.start_screen && app.c0.chrome.community_links);
    let ui = UiState::default();
    assert_eq!(
        (app.ui.toolbar, app.ui.control_bar, app.ui.task_bar, app.ui.dock, app.ui.status_bar),
        (ui.toolbar, ui.control_bar, ui.task_bar, ui.dock, ui.status_bar)
    );
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"panels":{"start_screen":false,"community_links":false,"app_bar":false}})).unwrap();
    assert_eq!(serde_json::to_value(&app.session.prefs).unwrap(), prefs);
    assert!(serde_json::to_value(&app.ui).unwrap().get("c0").is_none());
}
#[test]
fn community_menu_entries_and_commands_follow_switch() {
    let mut app = app();
    let ctx = egui::Context::default();
    let ids = ["help.discord", "help.website", "help.appPage", "help.github"];
    for id in ids {
        assert!(menus::menu_entries(&app).iter().any(|e| e.command.as_deref() == Some(id)));
        assert!(app.run(id, json!({})).is_ok());
    }
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"panels":{"community_links":false}})).unwrap();
    for id in ids.into_iter().chain(["help.links"]) {
        assert!(!menus::menu_entries(&app).iter().any(|e| e.command.as_deref() == Some(id)));
        assert!(app.run(id, json!({})).is_err());
        assert!(!c0_host::palette_match(&app, id, "discord github", ""));
        assert!(!vectorcraft_ui_egui::control::all_commands(&app).as_array().unwrap().iter().any(|entry| entry["id"] == id));
    }
    assert!(app.run("help.about", json!({})).is_ok());
    assert!(c0_host::palette_match(&app, "help.about", "about vectorcraft", "vector"));
}
#[test]
fn rendered_home_about_and_credits_keep_names_without_promotions() {
    let mut app = app();
    let ctx = egui::Context::default();
    let before = render(&mut app, &ctx);
    assert!(before.contains("Community"));
    assert!(before.contains("Discord"));
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"panels":{"community_links":false}})).unwrap();
    let home = render(&mut app, &ctx);
    assert!(!home.contains("Community"));
    app.ui.about = true;
    let about = render(&mut app, &ctx);
    assert!(about.contains("VectorCraft"));
    assert!(about.contains("MIT OR Apache-2.0"));
    for text in [&home, &about] {
        for banned in ["Discord", "ArtCraft", "getartcraft", "GitHub"] {
            assert!(!text.contains(banned), "{banned}: {text}");
        }
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("about_tab"), 1u8));
    let credits = render(&mut app, &ctx);
    assert!(credits.contains("Contributors"));
    for banned in ["https://github.com", "Discord", "getartcraft"] {
        assert!(!credits.contains(banned));
    }
}
#[test]
fn hidden_home_does_not_reappear_after_close_or_explicit_command() {
    let mut app = app();
    let ctx = egui::Context::default();
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"panels":{"app_bar":false,"start_screen":false}})).unwrap();
    assert!(app.run("app.home", json!({})).is_err());
    let empty = render(&mut app, &ctx);
    assert!(!empty.contains("Welcome to VectorCraft"));
    app.run("file.new", json!({})).unwrap();
    app.run("file.close", json!({})).unwrap();
    assert!(!render(&mut app, &ctx).contains("Welcome to VectorCraft"));
}
#[test]
fn control_queue_answers_all_host_calls_in_unpresented_logic_pass() {
    let ctx = egui::Context::default();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut app = app().with_control(rx);
    for (method, p) in [
        ("c0.inspect", json!({})),
        ("c0.set", json!({"panels":{"app_bar":false}})),
        ("c0.newDocument", json!({})),
        ("c0.goTo", json!({"artboard":0})),
        ("c0.documentBytes", json!({})),
        ("c0.cancelFileDialog", json!({})),
        (
            "c0.openFile",
            json!({"name":"file.svg","bytes":vectorcraft_format::base64_encode(br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="20"/>"#)}),
        ),
    ] {
        let (req, reply) = ControlRequest::new(method, p);
        tx.send(req).unwrap();
        ctx.run_ui(egui::RawInput::default(), |ui| app.logic(ui.ctx())).textures_delta.clear();
        let v = reply.try_recv().unwrap();
        assert_eq!(v["ok"], true, "{method}: {v}");
    }
}
