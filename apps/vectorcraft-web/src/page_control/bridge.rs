use serde_json::json;
use std::cell::RefCell;
use vectorcraft_ui_egui::VectorcraftApp;
use wasm_bindgen::{JsCast as _, prelude::*};
use wasm_bindgen_futures::{JsFuture, future_to_promise};

const FUNCTION: &str = "__vectorcraftControl";
const EVENT: &str = "vectorcraft-control-pass";
thread_local! {
    static RUNNER: RefCell<Option<eframe::WebRunner>> = const { RefCell::new(None) };
    static PUMP: RefCell<Option<web_sys::EventTarget>> = const { RefCell::new(None) };
}

pub fn runner(runner: &eframe::WebRunner) {
    RUNNER.with(|r| *r.borrow_mut() = Some(runner.clone()));
    PUMP.with(|p| *p.borrow_mut() = None);
}
fn pass() {
    let target = PUMP.with(|p| p.borrow().clone()).or_else(|| {
        let runner = RUNNER.with(|r| r.borrow().clone())?;
        let target = web_sys::EventTarget::new().ok()?;
        runner
            .add_event_listener::<web_sys::Event>(&target, EVENT, |_, app_runner| {
                app_runner.logic();
            })
            .ok()?;
        PUMP.with(|p| *p.borrow_mut() = Some(target.clone()));
        Some(target)
    });
    if let Some(target) = target
        && let Ok(event) = web_sys::Event::new(EVENT)
    {
        // Event dispatch synchronously lends the runner, even in an unpresented/background page.
        let _ = target.dispatch_event(&event);
    }
}

pub fn attach(mut app: VectorcraftApp, ctx: &egui::Context) -> VectorcraftApp {
    if !app.c0.page_attached {
        app.c0.chrome.app_bar = false;
        app.c0.chrome.start_screen = false;
        app.c0.chrome.community_links = false;
        app.c0.page_attached = true;
    }
    super::files::attach(&mut app, ctx);
    install();
    app
}

fn install() {
    let Some(window) = web_sys::window() else { return };
    let call = Closure::<dyn Fn(JsValue) -> js_sys::Promise>::new(move |input: JsValue| {
        let parsed = input.as_string().ok_or("request must be JSON text".to_owned()).and_then(|s| super::parse(&s));
        let (method, params) = match parsed {
            Ok(v) => v,
            Err(e) => return reply(false, json!(e)),
        };
        let requested = (|| {
            let window = web_sys::window().ok_or(JsValue::from_str("no window"))?;
            let api = js_sys::Reflect::get(&window, &"vectorcraft".into())?;
            let request = js_sys::Reflect::get(&api, &"request".into())?.dyn_into::<js_sys::Function>()?;
            let params = js_sys::JSON::parse(&params.to_string())?;
            let promise = request.call2(&api, &method.clone().into(), &params)?.dyn_into::<js_sys::Promise>()?;
            Ok::<_, JsValue>(promise)
        })();
        let promise = match requested {
            Ok(p) => p,
            Err(e) => return reply(false, json!(message(e))),
        };
        pass();
        for _ in 0..super::inline_passes(&method) {
            pass();
        }
        future_to_promise(async move {
            let value = match JsFuture::from(promise).await {
                Ok(v) => {
                    json!({"ok":true,"result":js_sys::JSON::stringify(&v).ok().and_then(|s|s.as_string()).and_then(|s|serde_json::from_str::<serde_json::Value>(&s).ok())})
                }
                Err(e) => json!({"ok":false,"error":message(e)}),
            };
            Ok(JsValue::from_str(&value.to_string()))
        })
    });
    if js_sys::Reflect::set(&window, &FUNCTION.into(), call.as_ref()).is_ok() {
        call.forget();
    }
}
fn message(e: JsValue) -> String {
    js_sys::Reflect::get(&e, &"message".into()).ok().and_then(|v| v.as_string()).or_else(|| e.as_string()).unwrap_or_else(|| format!("{e:?}"))
}
fn reply(ok: bool, value: serde_json::Value) -> js_sys::Promise {
    let response = if ok { json!({"ok":true,"result":value}) } else { json!({"ok":false,"error":value}) };
    js_sys::Promise::resolve(&JsValue::from_str(&response.to_string()))
}
