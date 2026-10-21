#![allow(non_snake_case)]

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::WeakMember;
use layoutng_assembly::internal::layout_block::LayoutBlock;
use layoutng_assembly::internal::layout_invalidation_reason::kStyleChange;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/block_style.h:9-9
// cpp: layoutng_svg/block_style.cc:12-18
pub fn SquaredLocalSvgTextScale(block: &LayoutBlock) -> f64 {
    let transform = block.TransformForLayout();
    if transform.is_null() {
        return 1.0;
    }
    let affine = AffineTransform::FromTransform(unsafe { &*transform });
    affine.A() * affine.A()
        + affine.B() * affine.B()
        + affine.C() * affine.C()
        + affine.D() * affine.D()
}

// cpp: layoutng_svg/block_style.h:10-12
// cpp: layoutng_svg/block_style.cc:20-34
pub fn UpdateSvgBlockStyle(
    block: &mut LayoutBlock,
    old_squared_scale: f64,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
) {
    if old_squared_scale == SquaredLocalSvgTextScale(block) {
        return;
    }
    let stacking_context_changed = !old_style.is_null()
        && (block.IsStackingContextWithStyle(unsafe { &*old_style })
            != block.IsStackingContextWithStyle(new_style));
    let document_element = block.IsDocumentElement();
    let key = WeakMember::from_ptr(block as *mut LayoutBlock);
    let view = unsafe { &mut *block.View() };
    let descendants = view
        .SvgTextDescendantsMap()
        .get(&key)
        .expect("SVG text descendants map has no entry for block")
        .Get();
    for member in unsafe { &*descendants }.iter() {
        let box_ = unsafe { &mut *member.Get() };
        box_.SetNeedsTextMetricsUpdate();
        if document_element || stacking_context_changed {
            box_.SetNeedsLayout(&raw const kStyleChange);
        }
    }
}
