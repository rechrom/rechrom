//! `CanvasContextCreationAttributesCore` translated from Blink's
//! `core/html/canvas/canvas_context_creation_attributes_core.idl`.

#![allow(non_snake_case)]

/// Web-exposed power preference. The default is intentionally distinct from
/// `LowPower`: Blink leaves backend selection unspecified unless the caller
/// asks for a preference.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CanvasPowerPreference {
    #[default]
    Default,
    LowPower,
    HighPerformance,
}

/// Backend-independent attributes shared by CanvasRenderingContext creation.
///
/// The defaults correspond to the WebGL context-attribute dictionary used by
/// Blink. Keeping the dictionary in the canvas package prevents WebIDL glue
/// and a GPU implementation from owning competing copies of context state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasContextCreationAttributesCore {
    pub alpha: bool,
    pub depth: bool,
    pub stencil: bool,
    pub antialias: bool,
    pub premultiplied_alpha: bool,
    pub preserve_drawing_buffer: bool,
    pub power_preference: CanvasPowerPreference,
    pub fail_if_major_performance_caveat: bool,
    pub desynchronized: bool,
    /// Source canvas_context_creation_attributes_core.h:34-39. Providers may
    /// disable the effective attribute while retaining the requested value for
    /// getContextAttributes().
    pub desynchronized_specified: bool,
    pub xr_compatible: bool,
}

impl Default for CanvasContextCreationAttributesCore {
    fn default() -> Self {
        Self {
            alpha: true,
            depth: true,
            stencil: false,
            antialias: true,
            premultiplied_alpha: true,
            preserve_drawing_buffer: false,
            power_preference: CanvasPowerPreference::Default,
            fail_if_major_performance_caveat: false,
            desynchronized: false,
            desynchronized_specified: false,
            xr_compatible: false,
        }
    }
}
