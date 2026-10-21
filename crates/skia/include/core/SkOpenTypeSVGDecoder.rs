// Copyright 2022 Google LLC. BSD-style license.
use crate::include::core::SkTypes::SkGlyphID;
use crate::src::core::{SkCanvas::SkCanvas, SkColor::SkColor};

// Source: include/core/SkOpenTypeSVGDecoder.h:19-28
// The native class has only virtual operations and no stored fields. A
// Box<dyn SkOpenTypeSVGDecoder> owns its concrete implementation; dropping that
// box invokes the concrete destructor, including its retained SVG DOM.
pub trait SkOpenTypeSVGDecoder {
    // Source: include/core/SkOpenTypeSVGDecoder.h:21-24
    /// Report retained memory, including an implementation's SVG DOM.
    fn approximateSize(&mut self) -> usize;

    // Source: include/core/SkOpenTypeSVGDecoder.h:25-26
    // SkSpan<SkColor> is a borrowed mutable contiguous palette. Neither the
    // canvas nor palette ownership is transferred to the decoder.
    fn render(
        &mut self,
        canvas: &mut SkCanvas<'_>,
        upem: i32,
        glyphId: SkGlyphID,
        foregroundColor: SkColor,
        palette: &mut [SkColor],
    ) -> bool;
    // Source: include/core/SkOpenTypeSVGDecoder.h:27-27
    // Native default virtual destruction maps to concrete Rust drop glue;
    // no additional virtual provider method or refcount is introduced.
}
