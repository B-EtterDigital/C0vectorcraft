#![allow(clippy::unwrap_used)]
use egui::Color32;
use serde_json::json;
use vectorcraft_ui_egui::{
    Services, VectorcraftApp, c0_host,
    theme::{self, Brightness, Tokens, host},
};

#[test]
fn registry_covers_every_color_field() {
    let src = include_str!("../src/theme.rs");
    let fields: Vec<_> = src
        .split("pub struct Tokens {")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap()
        .lines()
        .filter(|l| l.contains(": Color32"))
        .map(|l| l.trim().strip_prefix("pub ").unwrap().split(':').next().unwrap())
        .collect();
    assert_eq!(fields, host::FIELDS);
}
#[test]
fn overrides_survive_brightness_reapplication_and_clear_to_current_palette() {
    let ctx = egui::Context::default();
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Services::default());
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"themeTokens":{"text":"#123456","panel":"#234567","accent":"#aabbcc"}})).unwrap();
    for b in [Brightness::Dark, Brightness::Light] {
        theme::apply(&ctx, b);
        let t = Tokens::get(&ctx);
        assert_eq!(t.text, Color32::from_rgb(0x12, 0x34, 0x56));
        assert_eq!(t.panel, Color32::from_rgb(0x23, 0x45, 0x67));
        assert_eq!(ctx.global_style().visuals.panel_fill, t.panel);
    }
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"themeTokens":{"text":null}})).unwrap();
    theme::apply(&ctx, Brightness::Light);
    assert_eq!(Tokens::get(&ctx).text, Tokens::for_brightness(Brightness::Light).text);
}
#[test]
fn invalid_colors_are_rejected_and_contexts_are_isolated() {
    let ctx = egui::Context::default();
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Services::default());
    for value in [json!(3), json!("#xyzxyz"), json!("#12345"), json!("123456"), json!("#ffffffé")] {
        let v = c0_host::handle(&mut app, &ctx, "c0.set", &json!({"themeTokens":{"text":value}})).unwrap();
        assert_eq!(v["invalid"], json!(["themeTokens.text"]));
    }
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"themeTokens":{"text":"#123456"}})).unwrap();
    assert_ne!(Tokens::get(&ctx).text, Tokens::get(&egui::Context::default()).text);
}

#[test]
fn host_palette_reapplication_keeps_the_saved_canvas_preference() {
    let ctx = egui::Context::default();
    let mut app = VectorcraftApp::new(vectorcraft_engine::Session::new(), Services::default());
    app.session.prefs.canvas_color = "lightGray".into();
    vectorcraft_ui_egui::prefs_dialog::apply_runtime(&mut app, &ctx);
    let saved = Tokens::get(&ctx).pasteboard;
    assert_eq!(saved, Color32::from_gray(0xc8));
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"panels":{"rulers":false}})).unwrap();
    assert_eq!(Tokens::get(&ctx).pasteboard, saved);
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"themeTokens":{"pasteboard":"#abcdef"}})).unwrap();
    app.session.prefs.ui_brightness = "light".into();
    app.session.prefs.canvas_color = "darkGray".into();
    vectorcraft_ui_egui::prefs_dialog::apply_runtime(&mut app, &ctx);
    assert_eq!(Tokens::get(&ctx).pasteboard, Color32::from_rgb(0xab, 0xcd, 0xef));
    c0_host::handle(&mut app, &ctx, "c0.set", &json!({"themeTokens":{"pasteboard":null}})).unwrap();
    assert_eq!(Tokens::get(&ctx).pasteboard, Color32::from_gray(0x5a));
}
