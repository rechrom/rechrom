// The block-only host installs only the block algorithm. These exact
// signatures fail loudly if an excluded package is reached at runtime.
// They never simulate inline, text, transform, or scroll behavior.
#![allow(non_snake_case, unused_variables)]

use font_engine::fonts::shaping::forward::{GlyphCallback, ShapeResultView};
use font_engine::Font;
use foundation::transform_operations::{
    RotateTransformOperation, ScaleTransformOperation, TranslateTransformOperation,
};
use foundation::{gfx, Length, TextDirection, TransformOperations};
use layout_style::style::content_data::ContentData;
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
    fn RotateTransformOperationApply(operation: &RotateTransformOperation, transform: &mut gfx::Transform, size: &gfx::SizeF) -> ();
    fn RotateTransformOperationEqual(a: &RotateTransformOperation, b: &RotateTransformOperation) -> bool;
    fn ScaleTransformOperationApply(operation: &ScaleTransformOperation, transform: &mut gfx::Transform, size: &gfx::SizeF) -> ();
    fn ScaleTransformOperationEqual(a: &ScaleTransformOperation, b: &ScaleTransformOperation) -> bool;
    fn ShapeResultViewDirection(view: &ShapeResultView) -> TextDirection;
    fn ShapeResultViewEndIndex(view: &ShapeResultView) -> u32;
    fn ShapeResultViewForEachGlyph(view: &ShapeResultView, advance: f32, callback: GlyphCallback, context: *mut c_void) -> f32;
    fn ShapeResultViewStartIndex(view: &ShapeResultView) -> u32;
    fn ShapeResultViewWidth(view: &ShapeResultView) -> f32;
}
