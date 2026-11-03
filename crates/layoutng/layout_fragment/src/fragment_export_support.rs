//! Shared source fragment-export conversions and map vocabulary.
//!
//! These assembly-only helpers are consumed by fragment owner modules and
//! remain invisible to the standalone package API.

#![allow(non_snake_case)]

use super::*;

pub(super) fn Number(value: LayoutUnit) -> f64 {
    value.ToDouble()
}

#[inline(never)]
pub(super) fn ExportAppliedTextDecorations(style: &StyleComputedStyle) -> Vec<TextDecorationPaint> {
    let skip_ink = style.TextDecorationSkipInk() != ETextDecorationSkipInk::kNone;
    let decorations = style.AppliedTextDecorationData();
    if decorations.is_null() {
        return Vec::new();
    }
    unsafe { &*decorations }
        .iter()
        .map(|applied| ExportAppliedTextDecoration(applied, skip_ink))
        .collect()
}

pub(crate) fn ExportAppliedTextDecoration(
    applied: &AppliedTextDecoration,
    skip_ink: bool,
) -> TextDecorationPaint {
    let lines = applied.Lines();
    let color = applied.GetColor();
    let (red, green, blue) = color.ToSRGB();
    let thickness = applied.Thickness();
    let underline_offset = applied.UnderlineOffset();
    TextDecorationPaint {
        underline: (lines & TextDecorationLine::kUnderline) != TextDecorationLine::kNone,
        overline: (lines & TextDecorationLine::kOverline) != TextDecorationLine::kNone,
        line_through: (lines & TextDecorationLine::kLineThrough) != TextDecorationLine::kNone,
        color: Some(crate::internal::layout_input_types::Color {
            red,
            green,
            blue,
            alpha: color.Alpha(),
        }),
        style: match applied.Style() {
            StyleTextDecorationStyle::kSolid => TextDecorationStyle::kSolid,
            StyleTextDecorationStyle::kDouble => TextDecorationStyle::kDouble,
            StyleTextDecorationStyle::kDotted => TextDecorationStyle::kDotted,
            StyleTextDecorationStyle::kDashed => TextDecorationStyle::kDashed,
            StyleTextDecorationStyle::kWavy => TextDecorationStyle::kWavy,
        },
        thickness,
        underline_offset: if underline_offset.IsFixed() {
            f64::from(underline_offset.Pixels())
        } else {
            0.0
        },
        underline_offset_auto: underline_offset.IsAuto(),
        skip_ink,
    }
}

fn SameDecorationLines(a: &TextDecorationPaint, b: &TextDecorationPaint) -> bool {
    a.underline == b.underline && a.overline == b.overline && a.line_through == b.line_through
}

// Source: core/highlight/highlight_style_utils.cc:38-43,126-167,169-187,
// 240-251,405-535. Freeze the selection layer while source ComputedStyle and
// LayoutTheme color resolution are still available. In particular, the macOS
// theme deliberately keeps the originating foreground for default selection.
pub(super) fn ExportSelectionPaintStyle(style: &StyleComputedStyle) -> PaintStyle {
    let selection_style = style.HighlightData().Selection();
    let selection_style_ref = unsafe { selection_style.as_ref() };
    // HighlightPaintingStyle starts with the previous (originating) layer and
    // replaces only the properties admitted by the highlight cascade. In
    // particular, stroke color and paint order stay on the originating layer;
    // only a pseudo style's stroke width is copied (lines 508-511).
    let mut result =
        crate::internal::boundary::paint_style_export::ExportDetachedStyle(style, None);
    if let Some(selection_style) = selection_style_ref {
        let selection = crate::internal::boundary::paint_style_export::ExportDetachedStyle(
            selection_style,
            None,
        );
        result.color = selection.color;
        result.text_fill_color = selection.text_fill_color;
        result.background_color = selection.background_color;
        result.text_stroke_width = selection.text_stroke_width;
        result.text_shadows = selection.text_shadows.clone();
    } else {
        // HighlightPaintingStyle resets shadows before applying the pseudo
        // style; a missing ::selection rule therefore does not inherit them.
        result.text_shadows.clear();
    }

    if selection_style_ref.is_none_or(|pseudo| !pseudo.HasAuthorHighlightColors()) {
        result.background_color = crate::internal::boundary::paint_style_export::ColorValue(
            crate::internal::layout_theme::LayoutTheme::GetTheme().ActiveSelectionBackgroundColor(),
        );
        #[cfg(target_vendor = "apple")]
        let foreground =
            crate::internal::boundary::paint_style_export::StyleColor(style, style.Color());
        #[cfg(not(target_vendor = "apple"))]
        let foreground = crate::internal::boundary::paint_style_export::ColorValue(
            crate::internal::layout_theme::LayoutTheme::GetTheme().ActiveSelectionForegroundColor(),
        );
        result.color = foreground;
        result.text_fill_color = foreground;
    }
    result
}

// Source: core/highlight/highlight_style_utils.cc:385-402. The selected
// decoration retains the originating last decoration unless ::selection has a
// last decoration with the same line set, in which case its resolved color is
// used by TextDecorationInfo::LineColor.
pub(super) fn ExportSelectionTextDecoration(
    style: &StyleComputedStyle,
) -> Option<TextDecorationPaint> {
    let selection_style = style.HighlightData().Selection();
    if selection_style.is_null() {
        return None;
    }
    let selection_style = unsafe { &*selection_style };
    let mut decoration = ExportAppliedTextDecorations(style).pop()?;
    if let Some(selection_decoration) = ExportAppliedTextDecorations(selection_style).pop() {
        if SameDecorationLines(&decoration, &selection_decoration) {
            decoration = selection_decoration;
        }
    }
    decoration.color = Some(crate::internal::boundary::paint_style_export::StyleColor(
        selection_style,
        selection_style.TextDecorationColor(),
    ));
    Some(decoration)
}

pub(super) fn InputId(object: *const LayoutObject) -> u64 {
    if !object.is_null() {
        let node = unsafe { &*object }.GetNode();
        if !node.is_null() {
            return unsafe { &*node }.InputId();
        }
    }
    0
}

pub(super) fn PublicWritingMode(mode: StyleWritingMode) -> WritingMode {
    mode.into()
}

pub(super) type SourceEntry = LayoutBoundarySourceEntry;
pub(super) type SourceMap = LayoutBoundaryHashMap<*const LayoutObject, SourceEntry>;
pub(super) type StyleSourceEntry = LayoutBoundaryStyleSourceEntry;
pub(super) type StyleSourceMap = LayoutBoundaryHashMap<*const StyleComputedStyle, StyleSourceEntry>;
pub(super) type ExportedStyleCache = LayoutBoundaryHashMap<(usize, usize), PaintStyle>;
pub(super) type DebugNameCache = LayoutBoundaryHashMap<*const LayoutObject, Arc<str>>;
