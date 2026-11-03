//! Resolved text-decoration metrics shared by painting and ink overflow.

#![allow(non_snake_case)]

use crate::{
    decorating_box::DecoratingBox, inline_paint_context::InlinePaintContext,
    line_relative_rect::LineRelativeOffset,
};
use font_engine::{FontBaseline, FontVerticalPositionType};
pub use foundation::ResolvedUnderlinePosition;
use foundation::{
    gfx, Color, DecorationGeometry, EBoxDecorationBreak, ETextDecorationStyle, FloatValueForLength,
    LayoutUnit, Length, StrokeStyle, TextDecorationLine, TextDirection, WaveDefinition,
};
use layoutng_assembly::{
    fragment_item::FragmentItem,
    inline_cursor::InlineCursor,
    internal::{
        layout_theme::LayoutTheme, text_decoration_offset::TextDecorationOffset,
        used_font::UsedFont,
    },
};
use layoutng_style::style::{
    applied_text_decoration::AppliedTextDecoration, computed_style::ComputedStyle,
};

const USCRIPT_HANGUL: i32 = 18;
const USCRIPT_KATAKANA_OR_HIRAGANA: i32 = 54;

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:27-57.
fn ResolveUnderlinePosition(style: &ComputedStyle) -> ResolvedUnderlinePosition {
    use layoutng_style::style::computed_style_constants::TextUnderlinePosition;

    let position = style.GetTextUnderlinePosition();
    if style.GetFontBaseline() != FontBaseline::kCentralBaseline {
        if position.value() & TextUnderlinePosition::kUnder.value() != 0 {
            return ResolvedUnderlinePosition::kUnder;
        }
        if position.value() & TextUnderlinePosition::kFromFont.value() != 0 {
            return ResolvedUnderlinePosition::kNearAlphabeticBaselineFromFont;
        }
        return ResolvedUnderlinePosition::kNearAlphabeticBaselineAuto;
    }
    let script = style.GetFontDescription().GetScript();
    if script == USCRIPT_KATAKANA_OR_HIRAGANA || script == USCRIPT_HANGUL {
        if position.value() & TextUnderlinePosition::kLeft.value() != 0 {
            return ResolvedUnderlinePosition::kUnder;
        }
        return ResolvedUnderlinePosition::kOver;
    }
    if position.value() & TextUnderlinePosition::kRight.value() != 0 {
        return ResolvedUnderlinePosition::kOver;
    }
    ResolvedUnderlinePosition::kUnder
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:69-71.
fn InlineSizeOfItem(item: &FragmentItem) -> LayoutUnit {
    if item.IsHorizontal() {
        item.Size().width
    } else {
        item.Size().height
    }
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:73-89.
fn ContinuesDecoratedRun(
    item: *const FragmentItem,
    decoration_index: usize,
    decoration: &AppliedTextDecoration,
) -> bool {
    if item.is_null() || !unsafe { &*item }.IsText() {
        return false;
    }
    let decorations = unsafe { &*item }.Style().AppliedTextDecorations();
    decoration_index < decorations.len() && decorations[decoration_index] == *decoration
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:91-101.
fn HasSameDecorationAtIndexOnLine(
    item: *const FragmentItem,
    decoration_index: usize,
    decoration: &AppliedTextDecoration,
) -> bool {
    if item.is_null() {
        return false;
    }
    let item = unsafe { &*item };
    debug_assert!(item.IsText());
    debug_assert!(!item.IsLineBreak());
    ContinuesDecoratedRun(item, decoration_index, decoration)
        && InlineSizeOfItem(item) > LayoutUnit::default()
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.h:44.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IsSvgText(pub bool);

// Source: third_party/blink/renderer/core/paint/text_decoration_info.h:46-57.
#[derive(Clone, Copy, Default)]
pub struct TextDecorationFragmentContext {
    pub previous_fragment_on_line: *const FragmentItem,
    pub next_fragment_on_line: *const FragmentItem,
    pub fragment_cursor: *const InlineCursor,
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:179-202.
pub fn ComputeTextDecorationFragmentContext(
    cursor: &InlineCursor,
) -> TextDecorationFragmentContext {
    let mut fragment_context = TextDecorationFragmentContext {
        fragment_cursor: cursor,
        ..TextDecorationFragmentContext::default()
    };
    let mut line_cursor = cursor.clone();
    line_cursor.ExpandRootToContainingBlock();
    let current = cursor.CurrentItem();
    debug_assert!(!current.is_null());
    line_cursor.MoveToFragmentItem(unsafe { &*current });

    let mut previous_cursor = line_cursor.clone();
    previous_cursor.MoveToPreviousInlineLeafOnLine();
    let previous = previous_cursor.CurrentItem();
    if !previous.is_null() && unsafe { &*previous }.IsText() && !unsafe { &*previous }.IsLineBreak()
    {
        fragment_context.previous_fragment_on_line = previous;
    }
    let mut next_cursor = line_cursor;
    next_cursor.MoveToNextInlineLeafOnLine();
    let next = next_cursor.CurrentItem();
    if !next.is_null() && unsafe { &*next }.IsText() && !unsafe { &*next }.IsLineBreak() {
        fragment_context.next_fragment_on_line = next;
    }
    fragment_context
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.h:62-104.
pub struct ResolvedDecoration<'a> {
    pub applied_text_decoration: &'a AppliedTextDecoration,
    pub used_font: UsedFont,
    pub lines: TextDecorationLine,
    pub resolved_thickness: f32,
    pub line_left_inset: f32,
    pub line_right_inset: f32,
    pub effective_zoom: f32,
    pub offset_from_decorating_box: LayoutUnit,
    pub underline_position: ResolvedUnderlinePosition,
    pub has_underline: bool,
    pub has_overline: bool,
    pub is_flipped_underline_and_overline: bool,
}

impl<'a> ResolvedDecoration<'a> {
    pub fn new(font: &UsedFont, decoration: &'a AppliedTextDecoration) -> Self {
        Self {
            applied_text_decoration: decoration,
            used_font: UsedFont::new(font.GetFont(), font.ScalingFactor()),
            lines: TextDecorationLine::kNone,
            resolved_thickness: 0.0,
            line_left_inset: 0.0,
            line_right_inset: 0.0,
            effective_zoom: 1.0,
            offset_from_decorating_box: LayoutUnit::default(),
            underline_position: ResolvedUnderlinePosition::default(),
            has_underline: false,
            has_overline: false,
            is_flipped_underline_and_overline: false,
        }
    }

    pub fn HasUnderline(&self) -> bool {
        self.has_underline
    }

    pub fn HasOverline(&self) -> bool {
        self.has_overline
    }

    pub fn HasLineThrough(&self) -> bool {
        self.lines.bits() & TextDecorationLine::kLineThrough.bits() != 0
    }

    pub fn HasSpellingError(&self) -> bool {
        self.lines.bits() & TextDecorationLine::kSpellingError.bits() != 0
    }

    pub fn HasGrammarError(&self) -> bool {
        self.lines.bits() & TextDecorationLine::kGrammarError.bits() != 0
    }

    pub fn HasSpellingOrGrammarError(&self) -> bool {
        self.HasSpellingError() || self.HasGrammarError()
    }

    pub fn HasFontData(&self) -> bool {
        !self.used_font.PrimaryFont().is_null()
    }
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:103-108.
fn ResolveInsetForFragment(inset: f32, preceding_size: f32) -> f32 {
    if inset <= 0.0 {
        return if preceding_size == 0.0 { inset } else { 0.0 };
    }
    (inset - preceding_size).max(0.0)
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:110-117.
fn NeedsFragmentContextForDecoration(decoration: &AppliedTextDecoration) -> bool {
    let inset = decoration.DecorationInset();
    if inset.GetStart().IsAuto() {
        return false;
    }
    !inset.GetStart().IsZero() || !inset.GetEnd().IsZero()
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:119-146.
pub fn ComputeDecorationThickness(
    text_decoration_thickness: &foundation::TextDecorationThickness,
    used_font: &UsedFont,
) -> f32 {
    let used_font_size = used_font.UsedSize();
    let auto_underline_thickness = used_font_size / 10.0;
    if text_decoration_thickness.IsAuto() {
        return auto_underline_thickness;
    }
    if used_font.PrimaryFont().is_null() {
        return auto_underline_thickness;
    }
    if text_decoration_thickness.IsFromFont() {
        return used_font
            .UnderlineThickness()
            .unwrap_or(auto_underline_thickness);
    }
    FloatValueForLength(text_decoration_thickness.Thickness(), used_font_size).round()
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:148-162.
fn TextDecorationStyleToStrokeStyle(decoration_style: ETextDecorationStyle) -> StrokeStyle {
    match decoration_style {
        ETextDecorationStyle::kSolid => StrokeStyle::kSolidStroke,
        ETextDecorationStyle::kDouble => StrokeStyle::kDoubleStroke,
        ETextDecorationStyle::kDotted => StrokeStyle::kDottedStroke,
        ETextDecorationStyle::kDashed => StrokeStyle::kDashedStroke,
        ETextDecorationStyle::kWavy => StrokeStyle::kWavyStroke,
    }
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:164-175.
#[cfg(not(target_vendor = "apple"))]
fn MakeSpellingGrammarWave(effective_zoom: f32) -> WaveDefinition {
    let wavelength = 6.0 * effective_zoom;
    WaveDefinition {
        wavelength,
        control_point_distance: 5.0 * effective_zoom,
        phase: -0.75 * wavelength,
    }
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.h:106-248.
pub struct TextDecorationInfo<'a> {
    target_style_: &'a ComputedStyle,
    decorating_box_style_: *const ComputedStyle,
    inline_context_: Option<&'a InlinePaintContext>,
    target_used_font_: UsedFont,
    selection_decoration_line_: TextDecorationLine,
    selection_decoration_color_: Color,
    decoration_override_: Option<&'a AppliedTextDecoration>,
    local_origin_: LineRelativeOffset,
    width_: LayoutUnit,
    target_ascent_: f32,
    svg_resource_scaling_factor_: f32,
    fragment_context_: TextDecorationFragmentContext,
    conservative_inset_bounds_: bool,
    union_all_lines_: TextDecorationLine,
    original_underline_position_: ResolvedUnderlinePosition,
    flip_underline_and_overline_: bool,
    use_decorating_box_: bool,
    is_svg_text_: bool,
    antialias_: bool,
    highlight_override_: Option<Color>,
}

impl<'a> TextDecorationInfo<'a> {
    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:204-243.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        local_origin: LineRelativeOffset,
        width: LayoutUnit,
        target_style: &'a ComputedStyle,
        target_font: &UsedFont,
        inline_context: Option<&'a InlinePaintContext>,
        selection_decoration_line: TextDecorationLine,
        selection_decoration_color: Color,
        decoration_override: Option<&'a AppliedTextDecoration>,
        is_svg_text: IsSvgText,
        svg_resource_scaling_factor: f32,
        fragment_context: TextDecorationFragmentContext,
        conservative_inset_bounds: bool,
    ) -> Self {
        let use_decorating_box = inline_context.is_some()
            && decoration_override.is_none()
            && !is_svg_text.0
            && !layoutng_style::style::computed_style_constants::IsHighlightPseudoElement(
                target_style.StyleType(),
            );
        let mut result = Self {
            target_style_: target_style,
            decorating_box_style_: std::ptr::null(),
            inline_context_: inline_context,
            target_used_font_: UsedFont::new(target_font.GetFont(), target_font.ScalingFactor()),
            selection_decoration_line_: selection_decoration_line,
            selection_decoration_color_: selection_decoration_color,
            decoration_override_: decoration_override,
            local_origin_: local_origin,
            width_: width,
            target_ascent_: target_font.FloatAscent(),
            svg_resource_scaling_factor_: svg_resource_scaling_factor,
            fragment_context_: fragment_context,
            conservative_inset_bounds_: conservative_inset_bounds,
            union_all_lines_: TextDecorationLine::kNone,
            original_underline_position_: ResolvedUnderlinePosition::default(),
            flip_underline_and_overline_: false,
            use_decorating_box_: use_decorating_box,
            is_svg_text_: is_svg_text.0,
            antialias_: false,
            highlight_override_: None,
        };
        for index in 0..result.AppliedDecorationCount() {
            let decoration = result.AppliedDecoration(index);
            let lines = decoration.Lines();
            let style = decoration.Style();
            result.union_all_lines_ |= lines;
            if !result.antialias_
                && matches!(
                    style,
                    ETextDecorationStyle::kDotted | ETextDecorationStyle::kDashed
                )
            {
                result.antialias_ = true;
            }
        }
        result
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:245-256.
    pub fn NeedsFragmentContextForInset(style: &ComputedStyle) -> bool {
        debug_assert!(style.HasAppliedTextDecorations());
        style
            .AppliedTextDecorations()
            .iter()
            .any(NeedsFragmentContextForDecoration)
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:321-332.
    pub fn AppliedDecorationCount(&self) -> usize {
        if self.HasDecorationOverride() {
            1
        } else {
            self.target_style_.AppliedTextDecorations().len()
        }
    }

    pub fn AppliedDecoration(&self, index: usize) -> &'a AppliedTextDecoration {
        if let Some(decoration) = self.decoration_override_ {
            decoration
        } else {
            &self.target_style_.AppliedTextDecorations()[index]
        }
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.h:128-136.
    pub fn HasDecorationOverride(&self) -> bool {
        self.decoration_override_.is_some()
    }

    pub fn HasAnyLine(&self, lines: TextDecorationLine) -> bool {
        self.union_all_lines_.bits() & lines.bits() != 0
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:334-413.
    pub fn ResolveDecorationAt(&mut self, decoration_index: usize) -> ResolvedDecoration<'a> {
        debug_assert!(decoration_index < self.AppliedDecorationCount());

        let mut decoration = ResolvedDecoration::new(
            &self.target_used_font_,
            self.AppliedDecoration(decoration_index),
        );
        decoration.lines = decoration.applied_text_decoration.Lines();
        decoration.has_underline =
            decoration.lines.bits() & TextDecorationLine::kUnderline.bits() != 0;
        decoration.has_overline =
            decoration.lines.bits() & TextDecorationLine::kOverline.bits() != 0;

        let mut decorating_box: *const DecoratingBox = std::ptr::null();
        let mut decorating_box_style: *const ComputedStyle = self.target_style_;
        if self.use_decorating_box_ {
            let context = self
                .inline_context_
                .expect("decorating-box mode requires InlinePaintContext");
            debug_assert_eq!(
                context.DecoratingBoxes().len(),
                self.AppliedDecorationCount()
            );
            let disable_decorating_box = if decoration_index >= context.DecoratingBoxes().len() {
                true
            } else {
                decorating_box = &context.DecoratingBoxes()[decoration_index];
                let style = unsafe { &*decorating_box }.Style();
                decorating_box_style = style;
                !style.IsHorizontalWritingMode()
            };
            if disable_decorating_box {
                self.use_decorating_box_ = false;
                decorating_box = std::ptr::null();
                decorating_box_style = self.target_style_;
            }
        }

        if decorating_box_style != self.decorating_box_style_ {
            self.decorating_box_style_ = decorating_box_style;
            self.original_underline_position_ =
                ResolveUnderlinePosition(unsafe { &*decorating_box_style });
            self.flip_underline_and_overline_ =
                self.original_underline_position_ == ResolvedUnderlinePosition::kOver;
        }

        if self.flip_underline_and_overline_ {
            decoration.underline_position = ResolvedUnderlinePosition::kUnder;
            std::mem::swap(&mut decoration.has_underline, &mut decoration.has_overline);
        } else {
            decoration.underline_position = self.original_underline_position_;
        }
        decoration.is_flipped_underline_and_overline = self.flip_underline_and_overline_;

        if !self.is_svg_text_ && !decorating_box.is_null() {
            let used_font = unsafe { &*decorating_box }.GetUsedFont();
            decoration.used_font = UsedFont::new(used_font.GetFont(), used_font.ScalingFactor());
        }
        let decorating_box_style = unsafe { &*self.decorating_box_style_ };
        decoration.effective_zoom = decorating_box_style.EffectiveZoom();
        decoration.offset_from_decorating_box =
            if decoration.HasUnderline() && !decorating_box.is_null() {
                self.OffsetFromDecoratingBox(unsafe { &*decorating_box })
            } else {
                LayoutUnit::default()
            };
        decoration.resolved_thickness = self.ComputeThickness(&decoration);
        self.ResolveDecorationInsets(decoration_index, &mut decoration);
        decoration
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:258-319.
    fn ComputeDecoratedRunMetrics(
        &self,
        decoration: &ResolvedDecoration<'_>,
        decoration_index: usize,
    ) -> DecoratedRunMetrics {
        let mut size_before = LayoutUnit::default();
        let mut size_after = LayoutUnit::default();
        if !self.fragment_context_.fragment_cursor.is_null() {
            let fragment_cursor = unsafe { &*self.fragment_context_.fragment_cursor };
            if fragment_cursor.Current().IsNotNull() {
                let mut line_cursor = fragment_cursor.clone();
                line_cursor.ExpandRootToContainingBlock();
                let current = fragment_cursor.CurrentItem();
                debug_assert!(!current.is_null());
                line_cursor.MoveToFragmentItem(unsafe { &*current });

                let applied = decoration.applied_text_decoration;
                let mut cursor = line_cursor.clone();
                loop {
                    cursor.MoveToPreviousInlineLeaf();
                    let item = cursor.CurrentItem();
                    if !ContinuesDecoratedRun(item, decoration_index, applied) {
                        break;
                    }
                    size_before += InlineSizeOfItem(unsafe { &*item });
                }
                let mut cursor = line_cursor.clone();
                loop {
                    cursor.MoveToNextInlineLeaf();
                    let item = cursor.CurrentItem();
                    if !ContinuesDecoratedRun(item, decoration_index, applied) {
                        break;
                    }
                    size_after += InlineSizeOfItem(unsafe { &*item });
                }

                let current_item = line_cursor.CurrentItem();
                debug_assert!(!current_item.is_null());
                let layout_object = unsafe { &*current_item }.GetLayoutObject();
                debug_assert!(!layout_object.is_null());
                let mut size_before_in_node = LayoutUnit::default();
                let mut size_after_in_node = LayoutUnit::default();
                let mut found_current = false;
                let mut cursor = line_cursor.clone();
                cursor.MoveToLayoutObject(unsafe { &*layout_object });
                while cursor.Current().IsNotNull() {
                    let item = cursor.CurrentItem();
                    if item == current_item {
                        found_current = true;
                    } else if found_current {
                        size_after_in_node += InlineSizeOfItem(unsafe { &*item });
                    } else {
                        size_before_in_node += InlineSizeOfItem(unsafe { &*item });
                    }
                    cursor.MoveToNextForSameLayoutObject();
                }
                debug_assert!(found_current);
                size_before = size_before.max(size_before_in_node);
                size_after = size_after.max(size_after_in_node);
            }
        }
        DecoratedRunMetrics {
            size_before: size_before.ToFloat(),
            size_after: size_after.ToFloat(),
            total_size: (size_before + self.width_ + size_after).ToFloat(),
        }
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:486-550.
    fn ResolveDecorationInsets(
        &self,
        decoration_index: usize,
        decoration: &mut ResolvedDecoration<'_>,
    ) {
        let applied = decoration.applied_text_decoration;
        let inset = applied.DecorationInset();
        let is_auto = inset.GetStart().IsAuto();
        if !is_auto && inset.GetStart().IsZero() && inset.GetEnd().IsZero() {
            return;
        }

        let (mut start_inset, mut end_inset) = if is_auto {
            let auto_inset = (decoration.resolved_thickness / 2.0).clamp(1.0, 2.0);
            (auto_inset, auto_inset)
        } else if applied.BoxDecorationBreak() == EBoxDecorationBreak::kSlice {
            let run_metrics = self.ComputeDecoratedRunMetrics(decoration, decoration_index);
            let resolved_start = FloatValueForLength(inset.GetStart(), run_metrics.total_size);
            let resolved_end = FloatValueForLength(inset.GetEnd(), run_metrics.total_size);
            (
                ResolveInsetForFragment(resolved_start, run_metrics.size_before),
                ResolveInsetForFragment(resolved_end, run_metrics.size_after),
            )
        } else {
            let has_previous_same_decoration = HasSameDecorationAtIndexOnLine(
                self.fragment_context_.previous_fragment_on_line,
                decoration_index,
                applied,
            );
            let has_next_same_decoration = HasSameDecorationAtIndexOnLine(
                self.fragment_context_.next_fragment_on_line,
                decoration_index,
                applied,
            );
            if has_previous_same_decoration && has_next_same_decoration {
                return;
            }
            let percentage_reference = self.width_.ToFloat();
            (
                if has_previous_same_decoration {
                    0.0
                } else {
                    FloatValueForLength(inset.GetStart(), percentage_reference)
                },
                if has_next_same_decoration {
                    0.0
                } else {
                    FloatValueForLength(inset.GetEnd(), percentage_reference)
                },
            )
        };

        if self.conservative_inset_bounds_ && !is_auto {
            start_inset = start_inset.min(0.0);
            end_inset = end_inset.min(0.0);
        }
        if self.TargetStyle().Direction() == TextDirection::kRtl {
            std::mem::swap(&mut start_inset, &mut end_inset);
        }
        decoration.line_left_inset = start_inset;
        decoration.line_right_inset = end_inset;
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:552-564.
    fn OffsetFromDecoratingBox(&self, decorating_box: &DecoratingBox) -> LayoutUnit {
        debug_assert!(self.use_decorating_box_);
        let inline_context = self
            .inline_context_
            .expect("decorating-box mode requires InlinePaintContext");
        let decorating_box_paint_offset =
            decorating_box.ContentOffsetInContainer().top + inline_context.PaintOffset().top;
        decorating_box_paint_offset - self.local_origin_.line_over
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:658-681.
    fn ComputeThickness(&self, decoration: &ResolvedDecoration<'_>) -> f32 {
        if decoration.HasSpellingOrGrammarError() {
            #[cfg(target_os = "android")]
            {
                unimplemented!("Android source spellcheck feature state is not bridged")
            }
            #[cfg(target_vendor = "apple")]
            {
                return 2.0 * decoration.effective_zoom;
            }
            #[cfg(all(not(target_os = "android"), not(target_vendor = "apple")))]
            {
                return decoration.effective_zoom;
            }
        }
        let thickness = ComputeDecorationThickness(
            &decoration.applied_text_decoration.Thickness(),
            &decoration.used_font,
        );
        thickness.max(if self.is_svg_text_ { 0.0 } else { 1.0 })
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:415-484.
    pub fn ComputeLineData(
        &self,
        decoration: &ResolvedDecoration<'_>,
        line: TextDecorationLine,
        line_offset: f32,
    ) -> DecorationGeometry {
        let double_offset_from_thickness = decoration.resolved_thickness + 1.0;
        let (double_offset, wavy_offset) = if line == TextDecorationLine::kUnderline
            || line == TextDecorationLine::kSpellingError
            || line == TextDecorationLine::kGrammarError
        {
            (double_offset_from_thickness, double_offset_from_thickness)
        } else if line == TextDecorationLine::kOverline {
            (-double_offset_from_thickness, -double_offset_from_thickness)
        } else if line == TextDecorationLine::kLineThrough {
            (double_offset_from_thickness.floor(), 0.0)
        } else {
            unreachable!("TextDecorationLine::kNone/kBlink has no line geometry")
        };

        let mut custom_wave = None;
        let mut antialias = self.antialias_;
        let style = if line == TextDecorationLine::kSpellingError
            || line == TextDecorationLine::kGrammarError
        {
            #[cfg(target_vendor = "apple")]
            {
                antialias = true;
                StrokeStyle::kDottedStroke
            }
            #[cfg(all(not(target_vendor = "apple"), not(target_os = "android")))]
            {
                custom_wave = Some(MakeSpellingGrammarWave(decoration.effective_zoom));
                StrokeStyle::kWavyStroke
            }
            #[cfg(target_os = "android")]
            {
                unimplemented!("Android source spellcheck feature state is not bridged")
            }
        } else {
            TextDecorationStyleToStrokeStyle(decoration.applied_text_decoration.Style())
        };

        let decoration_rect = gfx::RectF::new(
            gfx::PointF::new(
                self.local_origin_.line_left.ToFloat() + decoration.line_left_inset,
                self.local_origin_.line_over.ToFloat() + line_offset,
            ),
            gfx::SizeF::new(
                (self.width_.ToFloat() - decoration.line_left_inset - decoration.line_right_inset)
                    .max(0.0),
                decoration.resolved_thickness,
            ),
        );
        let mut geometry = DecorationGeometry::Make(
            style,
            &decoration_rect,
            double_offset,
            wavy_offset,
            custom_wave.as_ref(),
        );
        geometry.antialias = antialias;
        geometry
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:566-584.
    pub fn ComputeUnderlineLineData(
        &self,
        decoration: &ResolvedDecoration<'_>,
        decoration_offset: &TextDecorationOffset<'_>,
    ) -> DecorationGeometry {
        debug_assert!(decoration.HasUnderline());
        let line_offset = if decoration.is_flipped_underline_and_overline {
            Length::default()
        } else {
            decoration.applied_text_decoration.UnderlineOffset()
        };
        let paint_underline_offset = decoration_offset.ComputeUnderlineOffset(
            decoration.underline_position,
            decoration.used_font.ComputedSize(),
            &decoration.used_font,
            &line_offset,
            decoration.resolved_thickness,
        ) as f32
            + decoration.offset_from_decorating_box.ToFloat();
        self.ComputeLineData(
            decoration,
            TextDecorationLine::kUnderline,
            paint_underline_offset,
        )
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:586-606.
    pub fn ComputeOverlineLineData(
        &self,
        decoration: &ResolvedDecoration<'_>,
        decoration_offset: &TextDecorationOffset<'_>,
    ) -> DecorationGeometry {
        debug_assert!(decoration.HasOverline());
        let (line_offset, position) = if decoration.is_flipped_underline_and_overline {
            (
                decoration.applied_text_decoration.UnderlineOffset(),
                FontVerticalPositionType::TopOfEmHeight,
            )
        } else {
            (Length::default(), FontVerticalPositionType::TextTop)
        };
        let paint_overline_offset = decoration_offset.ComputeUnderlineOffsetForUnder(
            &line_offset,
            self.TargetStyle().ComputedFontSize(),
            &decoration.used_font,
            decoration.resolved_thickness,
            position,
        );
        self.ComputeLineData(
            decoration,
            TextDecorationLine::kOverline,
            paint_overline_offset as f32,
        )
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:608-618.
    pub fn ComputeLineThroughLineData(
        &self,
        decoration: &ResolvedDecoration<'_>,
    ) -> DecorationGeometry {
        debug_assert!(decoration.HasLineThrough());
        let line_through_offset =
            2.0 * decoration.used_font.FloatAscent() / 3.0 - decoration.resolved_thickness / 2.0;
        self.ComputeLineData(
            decoration,
            TextDecorationLine::kLineThrough,
            line_through_offset,
        )
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:620-635.
    pub fn ComputeSpellingOrGrammarErrorLineData(
        &self,
        decoration: &ResolvedDecoration<'_>,
        decoration_offset: &TextDecorationOffset<'_>,
    ) -> DecorationGeometry {
        debug_assert!(decoration.HasSpellingOrGrammarError());
        debug_assert!(!decoration.HasUnderline());
        debug_assert!(!decoration.HasOverline());
        debug_assert!(!decoration.HasLineThrough());
        let paint_underline_offset = decoration_offset.ComputeUnderlineOffset(
            decoration.underline_position,
            self.TargetStyle().ComputedFontSize(),
            &decoration.used_font,
            &Length::default(),
            decoration.resolved_thickness,
        );
        self.ComputeLineData(
            decoration,
            if decoration.HasSpellingError() {
                TextDecorationLine::kSpellingError
            } else {
                TextDecorationLine::kGrammarError
            },
            paint_underline_offset as f32,
        )
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:637-656.
    pub fn LineColor(&self, decoration: &ResolvedDecoration<'_>) -> Color {
        if decoration.HasSpellingError() {
            return LayoutTheme::GetTheme().PlatformSpellingMarkerUnderlineColor();
        }
        if decoration.HasGrammarError() {
            return LayoutTheme::GetTheme().PlatformGrammarMarkerUnderlineColor();
        }
        if let Some(color) = self.highlight_override_ {
            return color;
        }
        if decoration.lines == self.selection_decoration_line_ {
            return self.selection_decoration_color_;
        }
        decoration.applied_text_decoration.GetColor()
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.h:162-171.
    pub fn TargetStyle(&self) -> &ComputedStyle {
        self.target_style_
    }

    pub fn SvgResourceScalingFactor(&self) -> f32 {
        self.svg_resource_scaling_factor_
    }

    pub fn BaselineForInkSkip(&self) -> f32 {
        self.local_origin_.line_over.ToFloat() + self.target_ascent_
    }

    // Source: third_party/blink/renderer/core/paint/text_decoration_info.cc:683-686.
    pub fn SetHighlightOverrideColor(&mut self, color: Option<Color>) {
        self.highlight_override_ = color;
    }
}

// Source: third_party/blink/renderer/core/paint/text_decoration_info.h:190-194.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct DecoratedRunMetrics {
    size_before: f32,
    size_after: f32,
    total_size: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use font_engine::Font;
    use foundation::{
        EBoxDecorationBreak, LayoutHeapScope, Length, MakeGarbageCollected, TextDecorationThickness,
    };
    use layoutng_style::style::text_decoration_inset::TextDecorationInset;

    fn decoration(lines: TextDecorationLine, style: ETextDecorationStyle) -> AppliedTextDecoration {
        AppliedTextDecoration::new(
            lines,
            style,
            Color::default(),
            TextDecorationThickness::new(&Length::Fixed(2.0)),
            Length::Fixed(0.0),
            TextDecorationInset::default(),
            EBoxDecorationBreak::kSlice,
        )
    }

    fn decoration_with_inset(
        lines: TextDecorationLine,
        start: Length,
        end: Length,
    ) -> AppliedTextDecoration {
        AppliedTextDecoration::new(
            lines,
            ETextDecorationStyle::kSolid,
            Color::default(),
            TextDecorationThickness::new(&Length::Fixed(2.0)),
            Length::Fixed(0.0),
            TextDecorationInset::new(&start, &end),
            EBoxDecorationBreak::kSlice,
        )
    }

    fn font() -> UsedFont {
        let font = unsafe { &*MakeGarbageCollected(Font::default()) };
        UsedFont::new(font, 1.0)
    }

    #[test]
    fn resolved_decoration_predicates_follow_line_bits() {
        let _heap = LayoutHeapScope::new();
        let applied = decoration(
            TextDecorationLine::kUnderline | TextDecorationLine::kLineThrough,
            ETextDecorationStyle::kSolid,
        );
        let mut resolved = ResolvedDecoration::new(&font(), &applied);
        resolved.lines = applied.Lines();
        resolved.has_underline = true;
        assert!(resolved.HasUnderline());
        assert!(resolved.HasLineThrough());
        assert!(!resolved.HasGrammarError());
    }

    #[test]
    fn line_data_uses_style_insets_offsets_and_thickness() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let used_font = font();
        let applied = decoration(
            TextDecorationLine::kUnderline,
            ETextDecorationStyle::kDouble,
        );
        let info = TextDecorationInfo::new(
            LineRelativeOffset::new(LayoutUnit::from_f32(10.0), LayoutUnit::from_f32(20.0)),
            LayoutUnit::from_f32(40.0),
            &style,
            &used_font,
            None,
            TextDecorationLine::kNone,
            Color::default(),
            Some(&applied),
            IsSvgText(false),
            1.0,
            TextDecorationFragmentContext::default(),
            false,
        );
        let mut resolved = ResolvedDecoration::new(&used_font, &applied);
        resolved.resolved_thickness = 2.0;
        resolved.line_left_inset = 3.0;
        resolved.line_right_inset = 5.0;
        let geometry = info.ComputeLineData(&resolved, TextDecorationLine::kUnderline, 4.0);
        assert_eq!(geometry.style, StrokeStyle::kDoubleStroke);
        assert_eq!(geometry.line.x(), 13.0);
        assert_eq!(geometry.line.y(), 24.0);
        assert_eq!(geometry.line.width(), 32.0);
        assert_eq!(geometry.line.height(), 2.0);
        assert_eq!(geometry.double_offset, 3.0);
    }

    #[test]
    fn inset_fragment_resolution_consumes_preceding_run_size() {
        assert_eq!(ResolveInsetForFragment(12.0, 5.0), 7.0);
        assert_eq!(ResolveInsetForFragment(3.0, 5.0), 0.0);
        assert_eq!(ResolveInsetForFragment(-2.0, 0.0), -2.0);
        assert_eq!(ResolveInsetForFragment(-2.0, 5.0), 0.0);
    }

    #[test]
    fn resolve_decoration_uses_real_style_data_and_conservative_insets() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let used_font = font();
        let applied = decoration_with_inset(
            TextDecorationLine::kUnderline | TextDecorationLine::kLineThrough,
            Length::Fixed(-3.0),
            Length::Fixed(5.0),
        );
        let mut info = TextDecorationInfo::new(
            LineRelativeOffset::default(),
            LayoutUnit::from_f32(40.0),
            &style,
            &used_font,
            None,
            TextDecorationLine::kNone,
            Color::default(),
            Some(&applied),
            IsSvgText(false),
            1.0,
            TextDecorationFragmentContext::default(),
            true,
        );
        let resolved = info.ResolveDecorationAt(0);
        assert!(resolved.HasUnderline());
        assert!(resolved.HasLineThrough());
        assert_eq!(resolved.line_left_inset, -3.0);
        assert_eq!(resolved.line_right_inset, 0.0);
        assert!(resolved.resolved_thickness >= 1.0);
    }

    #[test]
    fn line_color_uses_marker_highlight_selection_and_applied_precedence() {
        let _heap = LayoutHeapScope::new();
        let style = ComputedStyle::default();
        let used_font = font();
        let mut applied = decoration(TextDecorationLine::kUnderline, ETextDecorationStyle::kSolid);
        let applied_color = Color::FromRGB(10, 20, 30);
        let selection_color = Color::FromRGB(40, 50, 60);
        let highlight_color = Color::FromRGB(70, 80, 90);
        applied.SetColor(applied_color);
        let mut info = TextDecorationInfo::new(
            LineRelativeOffset::default(),
            LayoutUnit::from_f32(40.0),
            &style,
            &used_font,
            None,
            TextDecorationLine::kOverline,
            selection_color,
            Some(&applied),
            IsSvgText(false),
            1.0,
            TextDecorationFragmentContext::default(),
            false,
        );
        let mut resolved = ResolvedDecoration::new(&used_font, &applied);

        resolved.lines = TextDecorationLine::kUnderline;
        assert_eq!(info.LineColor(&resolved), applied_color);
        resolved.lines = TextDecorationLine::kOverline;
        assert_eq!(info.LineColor(&resolved), selection_color);
        info.SetHighlightOverrideColor(Some(highlight_color));
        assert_eq!(info.LineColor(&resolved), highlight_color);

        resolved.lines = TextDecorationLine::kSpellingError;
        assert_eq!(
            info.LineColor(&resolved),
            LayoutTheme::GetTheme().PlatformSpellingMarkerUnderlineColor()
        );
        resolved.lines = TextDecorationLine::kGrammarError;
        assert_eq!(
            info.LineColor(&resolved),
            LayoutTheme::GetTheme().PlatformGrammarMarkerUnderlineColor()
        );
    }
}
