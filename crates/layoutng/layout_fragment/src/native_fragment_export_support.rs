//! Shared native fragment-export conversions and map vocabulary.
//!
//! These assembly-only helpers are consumed by fragment owner modules and
//! remain invisible to the standalone package API.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:695-695
pub(super) fn Number(value: LayoutUnit) -> f64 {
    value.ToDouble()
}

#[inline(never)]
pub(super) fn ExportAppliedTextDecorations(
    style: &NativeComputedStyle,
) -> Vec<TextDecorationPaint> {
    let skip_ink = style.TextDecorationSkipInk() != ETextDecorationSkipInk::kNone;
    let decorations = style.AppliedTextDecorationData();
    if decorations.is_null() {
        return Vec::new();
    }
    unsafe { &*decorations }
        .iter()
        .map(|applied| {
            let lines = applied.Lines();
            let native_color = applied.GetColor();
            let (red, green, blue) = native_color.ToSRGB();
            let thickness = applied.Thickness();
            let underline_offset = applied.UnderlineOffset();
            TextDecorationPaint {
                underline: (lines & TextDecorationLine::kUnderline) != TextDecorationLine::kNone,
                overline: (lines & TextDecorationLine::kOverline) != TextDecorationLine::kNone,
                line_through: (lines & TextDecorationLine::kLineThrough)
                    != TextDecorationLine::kNone,
                color: Some(crate::internal::layout_input_types::Color {
                    red,
                    green,
                    blue,
                    alpha: native_color.Alpha(),
                }),
                style: match applied.Style() {
                    NativeTextDecorationStyle::kSolid => TextDecorationStyle::kSolid,
                    NativeTextDecorationStyle::kDouble => TextDecorationStyle::kDouble,
                    NativeTextDecorationStyle::kDotted => TextDecorationStyle::kDotted,
                    NativeTextDecorationStyle::kDashed => TextDecorationStyle::kDashed,
                    NativeTextDecorationStyle::kWavy => TextDecorationStyle::kWavy,
                },
                thickness: if thickness.IsAuto() {
                    None
                } else if thickness.IsFromFont() {
                    None
                } else if thickness.Thickness().IsFixed() {
                    Some(f64::from(thickness.Thickness().Pixels()))
                } else {
                    None
                },
                underline_offset: if underline_offset.IsFixed() {
                    f64::from(underline_offset.Pixels())
                } else {
                    0.0
                },
                underline_offset_auto: underline_offset.IsAuto(),
                skip_ink,
            }
        })
        .collect()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:697-703
pub(super) fn InputId(object: *const LayoutObject) -> u64 {
    if !object.is_null() {
        let node = unsafe { &*object }.GetNode();
        if !node.is_null() {
            return unsafe { &*node }.InputId();
        }
    }
    0
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2533-2543
pub(super) fn PublicWritingMode(mode: NativeWritingMode) -> WritingMode {
    mode.into()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:705-712
pub(super) type SourceEntry = LayoutBoundarySourceEntry;
pub(super) type SourceMap = LayoutBoundaryHashMap<*const LayoutObject, SourceEntry>;
pub(super) type StyleSourceMap =
    LayoutBoundaryHashMap<*const NativeComputedStyle, *const ComputedStyle>;
pub(super) type ExportedStyleCache = LayoutBoundaryHashMap<(usize, usize), PaintStyle>;
pub(super) type DebugNameCache = LayoutBoundaryHashMap<*const LayoutObject, Arc<str>>;

pub(super) fn TraceInlineExport() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_TRACE_INLINE_EXPORT").is_some())
}
