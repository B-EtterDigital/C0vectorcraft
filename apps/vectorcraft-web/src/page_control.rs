//! Opt-in JSON envelope over window.vectorcraft.request; no second control queue.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
use serde_json::{Value, json};

#[cfg(target_arch = "wasm32")]
mod bridge;
#[cfg(target_arch = "wasm32")]
mod files;
#[cfg(target_arch = "wasm32")]
pub use bridge::{attach, runner};

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
pub const MAX_INLINE_PASSES: usize = 40;
pub const MAX_CLICK_COUNT: u64 = 16;
pub const MAX_DRAG_STEPS: u64 = 32;

pub fn parse(text: &str) -> Result<(String, Value), String> {
    if text.len() > vectorcraft_ui_egui::c0_host::MAX_REQUEST_BYTES {
        return Err("request exceeds the host request limit".into());
    }
    let value: Value = serde_json::from_str(text).map_err(|e| format!("request is not JSON: {e}"))?;
    let method = value.get("method").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or("request has no method")?;
    if !method.starts_with("c0.") && text.len() > MAX_REQUEST_BYTES {
        return Err("request is larger than 1 MiB".into());
    }
    let params = value.get("params").filter(|v| !v.is_null()).cloned().unwrap_or(json!({}));
    if !params.is_object() {
        return Err("params must be an object".into());
    }
    let bound = match method {
        "ui.click" => Some(("count", MAX_CLICK_COUNT)),
        "ui.drag" => Some(("steps", MAX_DRAG_STEPS)),
        _ => None,
    };
    if let Some((key, cap)) = bound
        && let Some(v) = params.get(key)
        && v.as_u64().is_none_or(|n| n > cap)
    {
        return Err(format!("{key} exceeds the page input limit of {cap}"));
    }
    Ok((method.to_owned(), params))
}

pub fn inline_passes(method: &str) -> usize {
    if matches!(method, "ui.click" | "ui.move" | "ui.drag" | "ui.key" | "ui.text") { MAX_INLINE_PASSES } else { 0 }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn envelope_validation() {
        assert_eq!(parse(r#"{"method":"c0.inspect"}"#).unwrap(), ("c0.inspect".into(), json!({})));
        for s in ["bad", "[]", "{}", r#"{"method":"c0.set","params":4}"#] {
            assert!(parse(s).is_err());
        }
    }
    #[test]
    fn bounded_requests_and_events() {
        assert!(parse(&json!({"method":"ui.text","params":{"text":"x".repeat(MAX_REQUEST_BYTES)}}).to_string()).is_err());
        assert!(parse(&json!({"method":"c0.openFile","params":{"bytes":"x".repeat(MAX_REQUEST_BYTES)}}).to_string()).is_ok());
        for (m, k, n) in [("ui.click", "count", MAX_CLICK_COUNT), ("ui.drag", "steps", MAX_DRAG_STEPS)] {
            assert!(parse(&json!({"method":m,"params":{k:n}}).to_string()).is_ok());
            assert!(parse(&json!({"method":m,"params":{k:n+1}}).to_string()).is_err());
        }
    }
    #[test]
    fn mutations_can_advance_input_without_presentation() {
        assert_eq!(inline_passes("c0.inspect"), 0);
        assert_eq!(inline_passes("ui.click"), MAX_INLINE_PASSES);
    }
}
