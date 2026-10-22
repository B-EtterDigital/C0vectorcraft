//! Bounded browser picker. A cancelled request makes late chooser answers inert.
use std::sync::atomic::Ordering;
use vectorcraft_ui_egui::{VectorcraftApp, c0_host};
use wasm_bindgen_futures::{JsFuture, spawn_local};

pub fn attach(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let inbox = app.services.inbox.clone();
    let ctx = ctx.clone();
    app.services.open_async = Some(Box::new(move || {
        let Some(inbox) = inbox.clone() else { return };
        let pending = c0_host::begin_picker(&ctx);
        let ctx = ctx.clone();
        spawn_local(async move {
            let dialog =
                vectorcraft_engine::cmd::fileio::open_filters().fold(rfd::AsyncFileDialog::new(), |d, (name, exts)| d.add_filter(name, exts));
            let file = dialog.pick_file().await;
            if !pending.load(Ordering::Relaxed) {
                return;
            }
            let Some(file) = file else {
                pending.store(false, Ordering::Relaxed);
                return;
            };
            let size = file.inner().size();
            if !size.is_finite() || !(0.0..=c0_host::MAX_FILE_BYTES as f64).contains(&size) {
                pending.store(false, Ordering::Relaxed);
                log::warn!("picked file exceeds the host file limit");
                return;
            }
            let buffer = match JsFuture::from(file.inner().array_buffer()).await {
                Ok(v) => v,
                Err(e) => {
                    pending.store(false, Ordering::Relaxed);
                    log::warn!("could not read picked file: {e:?}");
                    return;
                }
            };
            let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
            let mut inbox = inbox.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let held = inbox.iter().map(|(_, b)| b.len()).fold(0, usize::saturating_add);
            if bytes.len() > c0_host::MAX_FILE_BYTES.saturating_sub(held) {
                pending.store(false, Ordering::Relaxed);
                return;
            }
            if pending.swap(false, Ordering::Relaxed) {
                inbox.push((file.file_name(), bytes));
                ctx.request_repaint();
            }
        });
    }));
}
