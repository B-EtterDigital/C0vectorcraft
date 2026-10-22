use crate::VectorcraftApp;
use serde_json::{Value, json};
use vectorcraft_doc::{Node, NodeId, NodeKind};

fn layers(node: &Node, depth: usize, out: &mut Vec<Value>) {
    if !matches!(node.kind, NodeKind::Layer { .. }) {
        return;
    }
    out.push(json!({"id": node.id.0, "name":node.name.as_deref().unwrap_or("Layer"), "visible":node.visible, "locked":node.locked, "depth":depth}));
    if depth == 0
        && let Some(children) = node.children()
    {
        for child in children {
            layers(child, depth + 1, out);
        }
    }
}

pub fn inspect(app: &VectorcraftApp, ctx: &egui::Context) -> Value {
    let st = app.session.active();
    let active = st.and_then(|st| app.view().map(|v| v.artboard).filter(|i| *i < st.doc.artboards.len()));
    let mut rows = Vec::new();
    if let Some(st) = st {
        for node in &st.doc.layers {
            layers(node, 0, &mut rows);
        }
    }
    json!({"title":st.map(|d|d.title()), "revision":st.map_or(0,|d|d.revision), "dirty":st.is_some_and(|d|d.is_dirty()),
        "canUndo":st.is_some_and(|d|!d.history.undo.is_empty()), "canRedo":st.is_some_and(|d|!d.history.redo.is_empty()),
        "panes":super::c0_panels::snapshot(app), "theme":crate::theme::host::inspect(ctx),
        "artboards":st.map(|d|d.doc.artboards.iter().enumerate().map(|(i,a)|json!({"index":i,"name":a.name,"width":a.rect.width(),"height":a.rect.height(),"active":active==Some(i)})).collect::<Vec<_>>()).unwrap_or_default(),
        "layers":rows, "selectionCount":st.map_or(0,|d|d.selection.objects.len()), "activeArtboard":active,
        "documents":app.session.documents().iter().enumerate().map(|(i,d)|json!({"title":d.title(),"dirty":d.is_dirty(),"active":Some(i)==app.session.active_index()})).collect::<Vec<_>>(),
        "fileDialog":app.picks.waiting_kind(), "pickerPending":super::picker_pending(ctx),
        "pendingFile":app.c0.pending.is_some(), "fileResult":app.c0.file_result})
}

pub fn go_to(app: &mut VectorcraftApp, p: &Value) -> Result<Value, String> {
    let st = app.session.active().ok_or("no document is open")?;
    let board = p
        .get("artboard")
        .map(|v| v.as_u64().and_then(|n| usize::try_from(n).ok()).filter(|i| *i < st.doc.artboards.len()).ok_or("artboard is out of range"))
        .transpose()?;
    let layer = p
        .get("layer")
        .map(|v| {
            v.as_u64()
                .map(NodeId)
                .filter(|id| st.doc.node(*id).is_some_and(|n| matches!(n.kind, NodeKind::Layer { .. })))
                .ok_or("layer is not a layer id")
        })
        .transpose()?;
    if board.is_none() && layer.is_none() {
        return Err("artboard or layer is required".into());
    }
    // Validate both targets before changing either.
    if let Some(i) = board {
        app.run("view.goToArtboard", json!({"index":i}))?;
    }
    if let Some(id) = layer {
        app.run("layer.setCurrent", json!({"id":id.0}))?;
        app.run("layer.locate", json!({"id":id.0}))?;
        app.run("window.panel", json!({"panel":"layers"}))?;
    }
    Ok(json!({"artboard":app.view().map(|v|v.artboard), "layer":app.session.active().and_then(|d|d.active_layer).map(|id|id.0)}))
}
