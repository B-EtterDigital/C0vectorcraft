//! Context-local colours over the current upstream palette, including egui widget visuals.
use super::Tokens;
use egui::Color32;
use serde_json::Value;
use std::collections::BTreeMap;

type Overrides = BTreeMap<String, Color32>;
fn id() -> egui::Id {
    egui::Id::new("vectorcraft-host-theme")
}

fn canvas_id() -> egui::Id {
    egui::Id::new("vectorcraft-host-canvas-preference")
}

/// Remember the native preference, retaining the original preference application behaviour.
pub fn canvas_preference(ctx: &egui::Context, color: Option<Color32>) {
    ctx.data_mut(|d| d.insert_temp(canvas_id(), color));
    if let Some(color) = color {
        store_canvas(ctx, color);
    }
}

fn store_canvas(ctx: &egui::Context, color: Color32) {
    let mut tokens = Tokens::get(ctx);
    tokens.pasteboard = color;
    ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, tokens));
}

/// Host palette reapplication must also retain the user's custom canvas preference.
pub fn restore_canvas_preference(ctx: &egui::Context) {
    if let Some(color) = ctx.data(|d| d.get_temp::<Option<Color32>>(canvas_id())).flatten() {
        store_canvas(ctx, color);
    }
}

macro_rules! colors {
    ($($field:ident),+ $(,)?) => {
        pub const FIELDS: &[&str] = &[$(stringify!($field)),+];
        pub fn overlay(ctx: &egui::Context, mut tokens: Tokens) -> Tokens {
            if let Some(values) = ctx.data(|d| d.get_temp::<Overrides>(id())) {
                $(if let Some(color) = values.get(stringify!($field)) { tokens.$field = *color; })+
            }
            tokens
        }
    };
}
colors!(
    app_bar,
    panel,
    panel_darker,
    input,
    input_border,
    border,
    divider,
    text,
    text_dim,
    text_disabled,
    icon,
    hover,
    tool_active,
    accent,
    accent_strong,
    row_selected,
    pasteboard,
    ruler,
    ruler_tick,
    smart_guide,
    guide,
    measure_bg,
    button,
    text_strong,
    tab_strip,
    button_border,
    selection,
    anchor_selected,
    flyout_active,
    caption_close,
    caption_close_text,
    highlight,
    bleed,
    error,
    warning,
    axis_x,
    axis_y,
    axis_z
);

fn parse(value: &Value) -> Option<Color32> {
    let s = value.as_str()?.strip_prefix('#')?;
    if !matches!(s.len(), 6 | 8) || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(s, 16).ok()?;
    Some(if s.len() == 6 {
        Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8)
    } else {
        Color32::from_rgba_unmultiplied((n >> 24) as u8, (n >> 16) as u8, (n >> 8) as u8, n as u8)
    })
}

pub fn set(ctx: &egui::Context, value: &Value, applied: &mut Vec<String>, unknown: &mut Vec<String>, invalid: &mut Vec<String>) {
    let Some(fields) = value.as_object() else {
        invalid.push("themeTokens".into());
        return;
    };
    let mut values = ctx.data(|d| d.get_temp::<Overrides>(id())).unwrap_or_default();
    for (key, value) in fields {
        let path = format!("themeTokens.{key}");
        if !FIELDS.contains(&key.as_str()) {
            unknown.push(path);
            continue;
        }
        if value.is_null() {
            values.remove(key);
        } else if let Some(color) = parse(value) {
            values.insert(key.clone(), color);
        } else {
            invalid.push(path);
            continue;
        }
        applied.push(path);
    }
    ctx.data_mut(|d| d.insert_temp(id(), values));
}

pub fn inspect(ctx: &egui::Context) -> Value {
    let values = ctx.data(|d| d.get_temp::<Overrides>(id())).unwrap_or_default();
    serde_json::json!({"overrides": values.into_iter().map(|(k, c)| { let [r,g,b,a] = c.to_srgba_unmultiplied(); (k, format!("#{r:02x}{g:02x}{b:02x}{a:02x}")) }).collect::<BTreeMap<_,_>>(),
        "dark": Tokens::get(ctx).dark})
}
