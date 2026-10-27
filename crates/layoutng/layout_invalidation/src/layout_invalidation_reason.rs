#![allow(non_upper_case_globals)]

use std::ffi::c_char;

// The source declares these unsized character arrays but supplies no
// definitions. Each Rust static names the first byte of the corresponding
// external array; callers pass its address and never read it as a scalar.
// A C-ABI bridge must export the symbols when the external implementation is
// connected. No trace text is invented here.
// cpp: layoutng/internal/layout_invalidation_reason.h:12-54
extern "C" {
    pub static kUnknown: c_char;
    pub static kSizeChanged: c_char;
    pub static kAncestorMoved: c_char;
    pub static kStyleChange: c_char;
    pub static kDomChanged: c_char;
    pub static kTextChanged: c_char;
    pub static kPrintingChanged: c_char;
    pub static kPaintPreview: c_char;
    pub static kAttributeChanged: c_char;
    pub static kColumnsChanged: c_char;
    pub static kChildAnonymousBlockChanged: c_char;
    pub static kAnonymousBlockChange: c_char;
    pub static kFontsChanged: c_char;
    pub static kFullscreen: c_char;
    pub static kChildChanged: c_char;
    pub static kListValueChange: c_char;
    pub static kListStyleTypeChange: c_char;
    pub static kCounterStyleChange: c_char;
    pub static kImageChanged: c_char;
    pub static kSliderValueChanged: c_char;
    pub static kAncestorMarginCollapsing: c_char;
    pub static kFieldsetChanged: c_char;
    pub static kSvgResourceInvalidated: c_char;
    pub static kFloatDescendantChanged: c_char;
    pub static kCountersChanged: c_char;
    pub static kGridChanged: c_char;
    pub static kMenuOptionsChanged: c_char;
    pub static kRemovedFromLayout: c_char;
    pub static kAddedToLayout: c_char;
    pub static kTableChanged: c_char;
    pub static kPaddingChanged: c_char;
    pub static kTextControlChanged: c_char;
    pub static kSvgChanged: c_char;
    pub static kScrollbarChanged: c_char;
    pub static kDisplayLock: c_char;
    pub static kDevtools: c_char;
    pub static kAnchorPositioning: c_char;
    pub static kScrollMarkersChanged: c_char;
    pub static kOutOfFlowAlignmentChanged: c_char;
}

// cpp: layoutng/internal/layout_invalidation_reason.h:56-58
pub type LayoutInvalidationReasonForTracing = [c_char];
