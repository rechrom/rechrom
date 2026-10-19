// Translation status: complete
// Copyright 2019 The Chromium Authors. BSD-style license in LICENSE.
// Source: third_party/blink/renderer/platform/graphics/compositing/layers_as_json.h:1-72.
// Source: third_party/blink/renderer/platform/graphics/compositing/layers_as_json.cc:1-187.
use super::content_layer_client_impl::{ContentLayerClientRef, LayerTreeFlags};
use crate::{
    adapter::{
        geometry::Transform,
        main_layer::{LayerContext, MainLayerTypes},
        paint_artifact_compositor::PaintArtifactCompositorTypes,
        paint_geometry::TransformCacheNode,
        pending_layer::PendingTransformNode,
    },
    cc::layers::layer::{Layer, LayerRef},
};
use serde_json::{json, Value};
use std::collections::HashMap;
/// Only the external Color native formatting primitive is injected. No Layer
/// allocation/reference is passed to the embedding or retained by a service.
pub trait LayersAsJsonColor {
    fn ColorNameForLayoutTree(&self, color: crate::adapter::layer_tree_impl::Color4f) -> String;
}
// Source: layers_as_json.cc:22-25.
fn RoundCloseToZero(value: f64) -> f64 {
    if value.abs() < 1e-7 {
        0.
    } else {
        value
    }
}
// Source: layers_as_json.cc:27-36.
fn TransformAsJSONArray<T: Transform>(t: &T) -> Value {
    Value::Array(
        (0..4)
            .map(|c| {
                Value::Array(
                    (0..4)
                        .map(|r| json!(RoundCloseToZero(t.rc(r, c))))
                        .collect(),
                )
            })
            .collect(),
    )
}
// Source: layers_as_json.cc:40-103.
pub fn CCLayerAsJSON<B: PaintArtifactCompositorTypes>(
    layer: &Layer<B>,
    flags: LayerTreeFlags,
    c: &LayerContext<'_, B>,
    color: &dyn LayersAsJsonColor,
) -> Value {
    let mut out = json!({});
    if flags.includes_debug_info() {
        out["this"] = json!(format!("{layer:p}"));
        out["ccLayerId"] = json!(layer.id());
    }
    let name = layer.DebugName(c);
    out["name"] = json!(name);
    let offset = layer.offset_to_transform_parent(c);
    if offset != Default::default() {
        out["position"] = json!([offset.x(), offset.y()]);
    }
    let size = *layer.bounds(c);
    if size != Default::default() {
        out["bounds"] = json!([size.width(), size.height()]);
    }
    if layer.contents_opaque(c) {
        out["contentsOpaque"] = json!(true);
    } else if layer.contents_opaque_for_text(c) {
        out["contentsOpaqueForText"] = json!(true);
    }
    if !layer.draws_content(c) {
        out["drawsContent"] = json!(false);
    }
    if layer.should_check_backface_visibility(c) {
        out["backfaceVisibility"] = json!("hidden");
    }
    let bg = layer.background_color(c);
    if bg.alpha != 0.
        && (flags.includes_debug_info() || (name != "Caret" && !name.contains("Scroll corner of")))
    {
        out["backgroundColor"] = json!(color.ColorNameForLayoutTree(bg));
    }
    if flags.0 & ((1 << 0) | (1 << 5)) != 0 {
        if let Some(info) = layer.debug_info(c) {
            out["compositingReasons"] = json!(info.compositing_reasons);
        }
    }
    if flags.includes_debug_info()
        && layer.hit_test_opaqueness(c)
            != crate::cc::input::hit_test_opaqueness::HitTestOpaqueness::kOpaque
    {
        out["hitTestOpaqueness"] = json!(
            crate::cc::input::hit_test_opaqueness::HitTestOpaquenessToString(
                layer.hit_test_opaqueness(c)
            )
        );
    }
    out
}
// Source: layers_as_json.h:45-64; layers_as_json.cc:105-109.
pub struct LayersAsJSON<B: PaintArtifactCompositorTypes> {
    flags_: LayerTreeFlags,
    next_transform_id_: i32,
    layers_json_: Vec<Value>,
    transform_id_map_: HashMap<B::TransformNode, i32>,
    transforms_json_: Vec<Value>,
    rendering_context_map_: HashMap<i32, i32>,
}
impl<B: PaintArtifactCompositorTypes<Json = Value>> LayersAsJSON<B> {
    pub fn new(flags: LayerTreeFlags) -> Self {
        Self {
            flags_: flags,
            next_transform_id_: 1,
            layers_json_: Vec::new(),
            transform_id_map_: HashMap::new(),
            transforms_json_: Vec::new(),
            rendering_context_map_: HashMap::new(),
        }
    }
    // Source: layers_as_json.cc:111-153.
    fn AddTransformJSON(&mut self, transform: &B::TransformNode) -> i32 {
        if let Some(id) = self.transform_id_map_.get(transform) {
            return *id;
        }
        let parent_id = transform
            .UnaliasedParent()
            .map_or(0, |p| self.AddTransformJSON(&p));
        let matrix = transform.Matrix();
        let identity =
            crate::adapter::paint_chunks_to_cc_layer::ConversionTransform::IsIdentity(&matrix);
        if identity && transform.RenderingContextId() == 0 {
            self.transform_id_map_.insert(transform.clone(), parent_id);
            return parent_id;
        }
        let mut out = json!({});
        let id = self.next_transform_id_;
        self.next_transform_id_ += 1;
        self.transform_id_map_.insert(transform.clone(), id);
        out["id"] = json!(id);
        if parent_id != 0 {
            out["parent"] = json!(parent_id);
        }
        if !identity {
            out["transform"] = TransformAsJSONArray(&matrix);
        }
        if !matrix.IsIdentityOrTranslation() {
            let p = transform.Origin();
            out["origin"] = json!([p.x(), p.y(), p.z()]);
        }
        if !transform.FlattensInheritedTransform() {
            out["flattenInheritedTransform"] = json!(false);
        }
        let context = transform.RenderingContextId();
        if context != 0 {
            let next = self.rendering_context_map_.len() as i32 + 1;
            let value = *self.rendering_context_map_.entry(context).or_insert(next);
            out["renderingContext"] = json!(value);
        }
        self.transforms_json_.push(out);
        id
    }
    // Source: layers_as_json.cc:155-177. Client locking is a call-scoped borrow
    // of the one actual resident allocation; the builder owns diagnostic JSON only.
    pub fn AddLayer(
        &mut self,
        layer: &LayerRef<B>,
        transform: &B::TransformNode,
        client: Option<&ContentLayerClientRef<B>>,
        c: &LayerContext<'_, B>,
        services: &dyn crate::adapter::content_layer_client_impl::ContentLayerClientServices<B>,
        color: &dyn LayersAsJsonColor,
    ) {
        if self.flags_.0 & (1 << 4) == 0 && !layer.draws_content(c) {
            let name = layer.DebugName(c);
            if matches!(
                name.as_str(),
                "LayoutView #document" | "Inner Viewport Scroll Layer" | "Scrolling Contents Layer"
            ) {
                return;
            }
        }
        let mut out = CCLayerAsJSON(layer, self.flags_, c, color);
        if let Some(client) = client {
            client.AppendAdditionalInfoAsJSON(self.flags_, layer, &mut out, c, services);
        }
        let id = self.AddTransformJSON(transform);
        if id != 0 {
            out["transform"] = json!(id);
        }
        self.layers_json_.push(out);
    }
    // Source: layers_as_json.cc:179-185.
    pub fn Finalize(self) -> Value {
        let mut out = json!({"layers":self.layers_json_});
        if !self.transforms_json_.is_empty() {
            out["transforms"] = Value::Array(self.transforms_json_);
        }
        out
    }
}
