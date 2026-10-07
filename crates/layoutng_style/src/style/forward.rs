// Untranslated engine classes remain pointer-facing opaque declarations.
// Image and CSSValue have a real resolved-gradient implementation below.

// cpp: layoutng_style/style/computed_style.h:99-101
#[repr(C)]
pub struct CSSAnimationData {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CSSTransitionData {
    _private: [u8; 0],
}

// cpp: layoutng_style/style/computed_style_base.h:133-138
pub use super::style_path::StylePath;

#[repr(C)]
pub struct StyleRule {
    _private: [u8; 0],
}

// cpp: layoutng_style/style/computed_style_base.h:129-131
pub use foundation::{
    RotateTransformOperation, ScaleTransformOperation, TranslateTransformOperation,
};

// cpp: layoutng_style/style/computed_style.h:91-106
#[repr(C)]
pub struct Element {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Longhand {
    _private: [u8; 0],
}

// cpp: layoutng_style/style/style_image.h:39-45
pub use super::generated_gradient_image::Image;

#[repr(C)]
pub struct ImageResourceContent {
    _private: [u8; 0],
}

pub use super::image_resource_observer::ImageResourceObserver;

#[repr(C)]
pub struct Node {
    _private: [u8; 0],
}

// C++ declares this enum with its default int underlying type; its values
// come from the external CSS package.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CSSValuePhase(i32);

// cpp: layoutng_style/style/content_data.h:39-45
#[repr(C)]
pub struct CSSSymbolsValue {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CounterStyle {
    _private: [u8; 0],
}

#[repr(C)]
pub struct LayoutObject {
    _private: [u8; 0],
}

#[repr(C)]
pub struct StyleEngine {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CountersAttachmentContext {
    _private: [u8; 0],
}

// The native CSS operation value owner now supplies this formerly forward-
// declared base; dependent headers keep their original declaration path.
pub use super::filter_operation::FilterOperation;

#[repr(C)]
pub struct SVGResourceClient {
    _private: [u8; 0],
}

// cpp: layoutng_style/style/style_svg_resource.h:16-17
#[repr(C)]
pub struct SVGResource {
    _private: [u8; 0],
}

// cpp: layoutng_style/style/style_variables.h:27-28
// The AMT stores pointers to these externally defined values. Hash and deep
// equality are required by that source algorithm, but their definitions are
// absent from this checkout, so those calls remain link-time requirements.
pub use super::generated_gradient_image::CSSValue;

#[repr(C)]
pub struct CSSVariableData {
    _private: [u8; 0],
}

#[allow(non_snake_case)]
impl CSSVariableData {
    pub fn Hash(&self) -> u32 {
        unsafe { CSSVariableDataHash(self) }
    }
}

impl PartialEq for CSSVariableData {
    fn eq(&self, other: &Self) -> bool {
        unsafe { CSSVariableDataEquals(self, other) }
    }
}

unsafe extern "Rust" {
    fn CSSVariableDataHash(value: &CSSVariableData) -> u32;
    fn CSSVariableDataEquals(left: &CSSVariableData, right: &CSSVariableData) -> bool;
}
