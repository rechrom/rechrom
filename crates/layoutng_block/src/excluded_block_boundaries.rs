// The block-only host installs only the block algorithm. These exact
// signatures fail loudly if an excluded package is reached at runtime.
// They never simulate inline, text, transform, or scroll behavior.
#![allow(non_snake_case, unused_variables)]

use font_engine::fonts::shaping::forward::{GlyphCallback, ShapeResultView};
use font_engine::Font;
use foundation::transform_operations::{
    RotateTransformOperation, ScaleTransformOperation, TranslateTransformOperation,
};
use foundation::{gfx, Length, PhysicalRect, TextDirection, TransformOperations};
use layoutng::break_token::BreakToken;
use layoutng::internal::algorithm_forward::InlineChildLayoutContext;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_object::LayoutObject;
#[cfg(not(feature = "replaced_extension"))]
use layoutng::internal::layout_replaced::LayoutReplaced;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::scroll_layout_scope::ScrollOffsetClampTarget;
use layoutng_style::style::content_data::ContentData;
use std::ffi::c_void;

macro_rules! excluded {
    ($(fn $name:ident($($arg:ident: $ty:ty),* $(,)?) -> $result:ty;)*) => {$ (
        #[unsafe(no_mangle)]
        pub extern "Rust" fn $name($($arg: $ty),*) -> $result {
            panic!(concat!(stringify!($name), " requires a package excluded from block-only rendering"))
        }
    )*};
}

excluded! {
    fn ContentDataClone(value: &dyn ContentData) -> *mut dyn ContentData;
    fn DispatchLayoutInlinePhysicalLinesBoundingBox(inline: *const LayoutObject) -> PhysicalRect;
    fn RotateTransformOperationApply(operation: &RotateTransformOperation, transform: &mut gfx::Transform, size: &gfx::SizeF) -> ();
    fn RotateTransformOperationEqual(a: &RotateTransformOperation, b: &RotateTransformOperation) -> bool;
    fn ScaleTransformOperationApply(operation: &ScaleTransformOperation, transform: &mut gfx::Transform, size: &gfx::SizeF) -> ();
    fn ScaleTransformOperationEqual(a: &ScaleTransformOperation, b: &ScaleTransformOperation) -> bool;
    fn ScrollOffsetClampTargetClampScrollOffsetAfterOverflowChange(target: &mut ScrollOffsetClampTarget) -> ();
    fn ShapeResultViewDirection(view: &ShapeResultView) -> TextDirection;
    fn ShapeResultViewEndIndex(view: &ShapeResultView) -> u32;
    fn ShapeResultViewForEachGlyph(view: &ShapeResultView, advance: f32, callback: GlyphCallback, context: *mut c_void) -> f32;
    fn ShapeResultViewStartIndex(view: &ShapeResultView) -> u32;
    fn ShapeResultViewWidth(view: &ShapeResultView) -> f32;
}

#[cfg(not(feature = "replaced_extension"))]
excluded! {
    fn LayoutReplacedContentRect(replaced: &LayoutReplaced) -> PhysicalRect;
}

#[cfg(not(feature = "inline_extension"))]
excluded! {
    fn LayoutInlineHasInlineFragments(inline: &LayoutInline) -> bool;
    fn LayoutTextScaledFont(text: &LayoutText) -> *const Font;
    fn LayoutTextTransformAndSecureOriginalText(text: &mut LayoutText) -> ();
}

#[cfg(not(feature = "inline_extension"))]
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineChildLayoutContextParallelFlowBreakTokens<'a>(
    context: &'a InlineChildLayoutContext,
) -> &'a foundation::HeapVector<foundation::Member<BreakToken>> {
    panic!("InlineChildLayoutContext requires the excluded inline package")
}
