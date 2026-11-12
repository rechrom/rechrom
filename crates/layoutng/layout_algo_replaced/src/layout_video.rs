//! LayoutVideo derived lifecycle over the resident LayoutMedia/LayoutImage bases.
//! Source: core/layout/layout_video.cc:42-220 (6c1d401fcca5).
#![allow(non_snake_case)]
use foundation::{LayoutUnit, PhysicalSize};
use layoutng_assembly::internal::layout_invalidation_reason::kSizeChanged;
use layoutng_assembly::internal::layout_object::{MarkingBehavior, StyleChangeContext};
use layoutng_assembly::internal::layout_replaced::{LayoutVideo, VideoDisplayMode};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::natural_sizing_info::PhysicalNaturalSizingInfo;
use layoutng_style::style::style_difference::StyleDifference;

// Source: layout_video.cc:78-105. Poster dimensions belong to the actual image
// resource cache, while player dimensions belong to that same live player.
pub fn NaturalDimensions(this: &LayoutVideo) -> PhysicalNaturalSizingInfo {
    let state = this.ElementState();
    let mut mode = this.GetDisplayMode();
    if mode == VideoDisplayMode::Poster && state.is_default_poster {
        mode = VideoDisplayMode::Video;
    }
    match mode {
        VideoDisplayMode::Poster => {
            let element = unsafe {
                &*foundation::To::<layoutng_assembly::internal::layout_node_metadata::Element>(
                    this.GetNode(),
                )
            };
            if !element
                .InputElementData()
                .as_ref()
                .expect("video input")
                .image_resource_error
            {
                return this.PosterNaturalDimensions();
            }
        }
        VideoDisplayMode::Video => {
            if let Some(size) = state.player_natural_size.filter(|size| !size.IsEmpty()) {
                let zoom = this.StyleRef().EffectiveZoom() as f64;
                return PhysicalNaturalSizingInfo::MakeFixed(&PhysicalSize::new(
                    LayoutUnit::from_f64(size.width() as f64 * zoom),
                    LayoutUnit::from_f64(size.height() as f64 * zoom),
                ));
            }
        }
    }
    PhysicalNaturalSizingInfo::None()
}
// Source: layout_video.cc:56-76.
pub fn UpdateNaturalSize(this: &mut LayoutVideo) {
    let sizing = NaturalDimensions(this);
    if (sizing.size.IsEmpty() && this.ElementState().is_media_document)
        || sizing == this.CachedNaturalDimensions()
    {
        return;
    }
    this.SetCachedNaturalDimensions(sizing);
    this.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkContainerChain);
    this.SetNeedsLayoutAndFullPaintInvalidation(&raw const kSizeChanged);
}
// Source: layout_video.cc:175-185 + Element::SetNeedsCompositingUpdate,
// core/dom/element.cc:7368-7388. All flags target the original native layers.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutVideoInvalidateCompositing(this: &mut LayoutVideo) {
    let state = this.ElementState();
    if state.player_natural_size.is_none() || !state.in_active_document {
        return;
    }
    let painting_layer = this.PaintingLayerDefault();
    if !painting_layer.is_null() {
        unsafe { &mut *painting_layer }.SetNeedsRepaint();
    }
    this.SetNeedsPaintPropertyUpdate();
    if this.HasLayer() {
        unsafe { &mut *this.Layer() }.SetNeedsCompositingInputsUpdate();
    }
}
// Source: layout_video.cc:165-173. The inherited UpdateFromElement is the
// source base implementation; derived invalidation must always follow it.
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutVideoUpdateFromElement(this: &mut LayoutVideo) {
    this.UpdateFromElementBase();
    this.RefreshPosterNaturalDimensions();
    LayoutVideoInvalidateCompositing(this);
    UpdateNaturalSize(this);
    this.SetShouldDoFullPaintInvalidation();
}
pub fn StyleDidChange(
    this: &mut LayoutVideo,
    difference: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    context: &StyleChangeContext,
) {
    // Source: layout_video.cc:155-163. Image resource/style lifecycle runs first.
    // The original ComputedStyle continues through to VideoPainter's resident
    // foreign-layer filter/dynamic-range producer (html_video_element.cc:1308).
    crate::layout_replaced::LayoutInputReplacedStyleDidChange(
        this, difference, old_style, new_style, context,
    );
    UpdateNaturalSize(this);
}
