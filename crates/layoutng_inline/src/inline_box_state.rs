// C++: layoutng_inline/inline_box_state.h. Out-of-line definitions in
// inline_box_state.cc are pending in this same package.
#![allow(non_snake_case)]

use font_engine::fonts::font_baseline::FontBaseline;
use font_engine::fonts::font_height::FontHeight;
use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::Font;
use foundation::{
    AtomicString, DynamicTo, EAlignmentBaseline, EBoxDecorationBreak, EDominantBaseline,
    GCedHeapVector, HeapVector, IsFlippedLinesWritingMode, IsRtl, LayoutUnit, MakeGarbageCollected,
    Member, PaintInvalidationReason, RubyPosition, RuntimeEnabledFeatures, TextDirection,
    TextEmphasisMark, To, Traceable, UnsupportedLayout, ValueForLength, ValuesEquivalent, Visitor,
    WritingDirectionMode, WtfSizeT,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::inline_item_result::InlineItemResult;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::text_fit_scale::TextFitBlockScale;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::logical_line_item::{LogicalLineItem, LogicalLineItems};
use layoutng_fragment_tree::physical_fragment::BoxType;
use layoutng_geometry::geometry::box_sides::LineLogicalBoxSides;
use layoutng_geometry::geometry::box_strut::{BoxStrut, LineBoxStrut};
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{
    EBaselineShiftType, EVerticalAlign, LineLogicalSide, PseudoId,
};
use layoutng_style::style::text_box_edge::TextBoxEdgeType;
use layoutng_style::style::text_fit::{TextFitTarget, TextFitType};

use crate::line_info::LineInfo;
use crate::line_utils::{
    CalculateLeadingSpace, ComputeRelativeOffsetForInline, ComputeRelativeOffsetForOOFInInline,
};
use layoutng::internal::used_font::UsedFont;

// cpp: layoutng_inline/inline_box_state.h:33-40
pub struct PendingPositions {
    pub fragment_start: u32,
    pub fragment_end: u32,
    pub metrics: FontHeight,
    pub vertical_align: EVerticalAlign,
}

// cpp: layoutng_inline/inline_box_state.h:42-104
pub struct InlineBoxState {
    pub fragment_start: u32,
    pub item: Member<InlineItem>,
    pub style: Member<ComputedStyle>,
    pub font: Member<Font>,
    pub scaled_font: Member<Font>,
    pub scaling_factor: f32,
    pub text_fit_scale: f32,
    pub metrics: FontHeight,
    pub text_metrics: FontHeight,
    pub text_top: LayoutUnit,
    pub text_height: LayoutUnit,
    pub alignment_type: FontBaseline,
    pub has_start_edge: bool,
    pub has_end_edge: bool,
    pub margins: LineBoxStrut,
    pub borders: LineBoxStrut,
    pub padding: LineBoxStrut,
    pub pending_descendants: Vec<PendingPositions>,
    pub include_used_fonts: bool,
    pub has_box_placeholder: bool,
    pub needs_box_fragment: bool,
    pub is_svg_text: bool,
}

// cpp: layoutng_inline/inline_box_state.h:104-108
// cpp: layoutng_inline/inline_box_state.cc:38-60
// The C++ move constructor copies scalars and GC pointers and moves the
// pending-descendant vector. Rust ownership transfer does the same; Copy/Clone
// are intentionally absent.
impl Default for InlineBoxState {
    fn default() -> Self {
        Self {
            fragment_start: 0,
            item: Member::default(),
            style: Member::default(),
            font: Member::default(),
            scaled_font: Member::default(),
            scaling_factor: 0.0,
            text_fit_scale: 1.0,
            metrics: FontHeight::Empty(),
            text_metrics: FontHeight::Empty(),
            text_top: LayoutUnit::default(),
            text_height: LayoutUnit::default(),
            alignment_type: FontBaseline::kAlphabeticBaseline,
            has_start_edge: false,
            has_end_edge: false,
            margins: LineBoxStrut::default(),
            borders: LineBoxStrut::default(),
            padding: LineBoxStrut::default(),
            pending_descendants: Vec::new(),
            include_used_fonts: false,
            has_box_placeholder: false,
            needs_box_fragment: false,
            is_svg_text: false,
        }
    }
}

impl InlineBoxState {
    // cpp: layoutng_inline/inline_box_state.h:110-115
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.item);
        visitor.Trace(&self.style);
        visitor.Trace(&self.font);
        visitor.Trace(&self.scaled_font);
    }

    // cpp: layoutng_inline/inline_box_state.h:123-127
    pub fn HasMetrics(&self) -> bool {
        !self.metrics.IsEmpty() || !self.pending_descendants.is_empty()
    }

    // cpp: layoutng_inline/inline_box_state.h:117-121,129-165
    pub fn ResetStyle(&mut self, style: &ComputedStyle, is_svg: bool, object: &LayoutObject) {
        // cpp: layoutng_inline/inline_box_state.cc:62-112
        self.style = Member::from_ptr(style as *const _ as *mut _);
        self.is_svg_text = is_svg;
        self.text_fit_scale = 1.0;
        if !self.is_svg_text {
            self.scaling_factor = 1.0;
            self.scaled_font = Member::default();
            self.font = Member::from_ptr(style.GetFont());
            return;
        }
        let algorithms = LayoutPassScope::Algorithms();
        let compute_scaled_font = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.svg_support.compute_scaled_font
        };
        let Some(compute_scaled_font) = compute_scaled_font else {
            std::panic::panic_any(UnsupportedLayout::new(
                "SVG font scaling module is not installed",
            ));
        };
        self.scaled_font =
            Member::from_ptr(compute_scaled_font(object, &mut self.scaling_factor).cast_mut());
        self.font = self.scaled_font;
        self.alignment_type = match style.AlignmentBaseline() {
            EAlignmentBaseline::kAuto | EAlignmentBaseline::kBaseline => style.GetFontBaseline(),
            EAlignmentBaseline::kBeforeEdge | EAlignmentBaseline::kTextBeforeEdge => {
                FontBaseline::kTextOverBaseline
            }
            EAlignmentBaseline::kMiddle => FontBaseline::kXMiddleBaseline,
            EAlignmentBaseline::kCentral => FontBaseline::kCentralBaseline,
            EAlignmentBaseline::kAfterEdge | EAlignmentBaseline::kTextAfterEdge => {
                FontBaseline::kTextUnderBaseline
            }
            EAlignmentBaseline::kIdeographic => FontBaseline::kIdeographicUnderBaseline,
            EAlignmentBaseline::kAlphabetic => FontBaseline::kAlphabeticBaseline,
            EAlignmentBaseline::kHanging => FontBaseline::kHangingBaseline,
            EAlignmentBaseline::kMathematical => FontBaseline::kMathBaseline,
        };
    }
    pub fn ComputeTextMetrics(
        &mut self,
        style: &ComputedStyle,
        font: &Font,
        baseline: FontBaseline,
        scale: *const TextFitBlockScale,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:114-169
        let baseline_type = if style.CssDominantBaseline() == EDominantBaseline::kAuto {
            baseline
        } else {
            style.GetFontBaseline()
        };
        let scale_ref = if scale.is_null() {
            None
        } else {
            Some(unsafe { &*scale })
        };
        let base_font = scale_ref
            .and_then(|scale| unsafe { scale.scaled_font.as_ref() })
            .unwrap_or(font);
        let paint_scale = scale_ref.map_or(1.0, |scale| scale.paint_scale);
        let font_data = base_font.PrimaryFont();
        if !font_data.is_null() {
            let font_metrics = unsafe { &*font_data }.GetFontMetrics();
            if paint_scale != 1.0 {
                self.text_metrics = font_metrics.GetFloatFontHeight(baseline_type);
                self.text_metrics.ascent *= paint_scale;
                self.text_metrics.descent *= paint_scale;
            } else if self.is_svg_text {
                self.text_metrics = font_metrics.GetFloatFontHeight(baseline_type);
            } else {
                self.text_metrics = font_metrics.GetFontHeight(baseline_type);
            }
        } else {
            self.text_metrics = FontHeight::default();
        }
        self.text_top = -self.text_metrics.ascent;
        self.text_height = self.text_metrics.LineHeight();

        let emphasis_marks_outsets = if RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled() {
            FontHeight::Empty()
        } else {
            Self::ComputeEmphasisMarkOutsets(style, &UsedFont::new(base_font, paint_scale))
        };
        let mut line_height = style.ComputedLineHeightAsFixedForFont(base_font);
        if !style.LineHeight().IsFixed() && paint_scale != 1.0 {
            line_height *= paint_scale;
        }
        let leading_space = CalculateLeadingSpace(&line_height, &self.text_metrics);
        if emphasis_marks_outsets.IsEmpty() {
            self.text_metrics.AddLeading(&leading_space);
        } else {
            let mut emphasis_marks_metrics = self.text_metrics;
            emphasis_marks_metrics += &emphasis_marks_outsets;
            self.text_metrics.AddLeading(&leading_space);
            self.text_metrics.Unite(&emphasis_marks_metrics);
        }
        self.metrics.Unite(&self.text_metrics);
        self.include_used_fonts = style.LineHeight().IsAuto();
    }
    pub fn EnsureTextMetrics(
        &mut self,
        style: &ComputedStyle,
        font: &Font,
        baseline: FontBaseline,
        scale: *const TextFitBlockScale,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:246-252
        if self.text_metrics.IsEmpty() {
            self.ComputeTextMetrics(style, font, baseline, scale);
        }
    }
    pub fn ResetTextMetrics(&mut self) {
        // cpp: layoutng_inline/inline_box_state.cc:241-244
        self.metrics = FontHeight::Empty();
        self.text_metrics = FontHeight::Empty();
        self.text_top = LayoutUnit::default();
        self.text_height = LayoutUnit::default();
    }
    pub fn AccumulateUsedFonts(&mut self, shape: *const ShapeResultView) {
        self.AccumulateUsedFontsAtScale(shape, 1.0)
    }
    pub fn AccumulateUsedFontsAtScale(&mut self, shape: *const ShapeResultView, scale: f32) {
        // cpp: layoutng_inline/inline_box_state.cc:254-270
        let baseline_type = unsafe { &*self.style.Get() }.GetFontBaseline();
        // Rust's owned set drops at scope exit, matching ClearCollectionScope.
        let used_fonts = unsafe { &*shape }.UsedFonts();
        for used_font in used_fonts.iter() {
            let font_metrics = unsafe { &*used_font.Get() }.GetFontMetrics();
            let mut used_metrics = font_metrics.GetFontHeight(baseline_type);
            let leading_space =
                CalculateLeadingSpace(&font_metrics.FixedLineSpacing(), &used_metrics);
            used_metrics.AddLeading(&leading_space);
            used_metrics.ascent *= scale;
            used_metrics.descent *= scale;
            self.metrics.Unite(&used_metrics);
        }
    }
    pub fn TextTop(&self, baseline: FontBaseline) -> LayoutUnit {
        // cpp: layoutng_inline/inline_box_state.cc:272-278
        if !self.text_metrics.IsEmpty() {
            return self.text_top;
        }
        let font_data = unsafe { &*self.font.Get() }.PrimaryFont();
        if !font_data.is_null() {
            return -unsafe { &*font_data }
                .GetFontMetrics()
                .FixedAscent(baseline);
        }
        unreachable!("InlineBoxState::TextTop requires a primary font")
    }
    pub fn CanAddTextOfStyle(&self, style: &ComputedStyle) -> bool {
        // cpp: layoutng_inline/inline_box_state.cc:280-290
        if style.VerticalAlign() != EVerticalAlign::kBaseline {
            return false;
        }
        debug_assert!(!self.style.Get().is_null());
        let box_style = unsafe { &*self.style.Get() };
        if std::ptr::eq(box_style, style)
            || ValuesEquivalent(box_style.GetFont(), style.GetFont())
            || unsafe { &*box_style.GetFont() }.PrimaryFont()
                == unsafe { &*style.GetFont() }.PrimaryFont()
        {
            return true;
        }
        false
    }
    pub fn AdjustEdges(
        style: &ComputedStyle,
        font: &Font,
        baseline: FontBaseline,
        apply_over: bool,
        apply_under: bool,
        metrics: &mut FontHeight,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:171-223
        debug_assert!(apply_over || apply_under);
        let font_data = font.PrimaryFont();
        if font_data.is_null() {
            return;
        }
        let font_metrics = unsafe { &*font_data }.GetFontMetrics();
        let edge = style.GetTextBoxEdge();
        if apply_over {
            metrics.ascent = match *edge.Over() {
                TextBoxEdgeType::kAuto | TextBoxEdgeType::kText => {
                    font_metrics.FixedAscent(baseline)
                }
                TextBoxEdgeType::kCap => font_metrics.FixedCapHeight(baseline),
                TextBoxEdgeType::kEx => font_metrics.FixedXHeight(baseline),
                TextBoxEdgeType::kAlphabetic => unreachable!("alphabetic over edge"),
                _ => unreachable!("invalid text-box over edge"),
            };
        }
        if apply_under {
            metrics.descent = match *edge.Under() {
                TextBoxEdgeType::kAuto | TextBoxEdgeType::kText => {
                    font_metrics.FixedDescent(baseline)
                }
                TextBoxEdgeType::kAlphabetic => -font_metrics.FixedAlphabetic(baseline),
                TextBoxEdgeType::kCap | TextBoxEdgeType::kEx => {
                    unreachable!("cap or ex under edge")
                }
                _ => unreachable!("invalid text-box under edge"),
            };
        }
    }
    pub fn ComputeEmphasisMarkOutsets(style: &ComputedStyle, used_font: &UsedFont) -> FontHeight {
        // cpp: layoutng_inline/inline_box_state.cc:225-239
        if style.GetTextEmphasisMark() == TextEmphasisMark::kNone {
            return FontHeight::Empty();
        }
        let height = LayoutUnit::FromFloatRound(
            unsafe { FontEmphasisMarkHeight(used_font.GetFont(), style.TextEmphasisMarkString()) }
                .ToFloat()
                * used_font.ScalingFactor(),
        );
        debug_assert!(height >= LayoutUnit::default());
        if style.GetTextEmphasisLineLogicalSide() == LineLogicalSide::kOver {
            FontHeight::new(height, LayoutUnit::default())
        } else {
            FontHeight::new(LayoutUnit::default(), height)
        }
    }
    #[cfg(debug_assertions)]
    pub fn CheckSame(&self, other: &Self) {
        // cpp: layoutng_inline/inline_box_state.cc:1466-1493
        debug_assert_eq!(self.fragment_start, other.fragment_start);
        debug_assert!(self.item == other.item);
        debug_assert!(self.style == other.style);
        debug_assert_eq!(self.metrics, other.metrics);
        debug_assert_eq!(self.text_metrics, other.text_metrics);
        debug_assert_eq!(self.text_top, other.text_top);
        debug_assert_eq!(self.text_height, other.text_height);
        if !self.text_metrics.IsEmpty() {
            debug_assert_eq!(self.include_used_fonts, other.include_used_fonts);
        }
        debug_assert_eq!(self.needs_box_fragment, other.needs_box_fragment);
        debug_assert_eq!(self.has_start_edge, other.has_start_edge);
        debug_assert_eq!(self.margins, other.margins);
        debug_assert_eq!(self.borders, other.borders);
        debug_assert_eq!(self.padding, other.padding);
        debug_assert_eq!(self.pending_descendants.len(), 0);
        debug_assert_eq!(other.pending_descendants.len(), 0);
    }
}

// cpp: layoutng_inline/inline_box_state.h:168-175,386-393
// Vec-backed HeapVector keeps stable element order while the transient stack
// is rebuilt after fragmentation, as in the source.
#[derive(Default)]
pub struct InlineLayoutStateStack {
    stack_: HeapVector<InlineBoxState, 4>,
    box_data_list_: HeapVector<BoxData, 4>,
    ruby_column_list_: HeapVector<Member<LogicalRubyColumn>>,
    is_empty_line_: bool,
    has_block_in_inline_: bool,
    is_svg_text_: bool,
}

impl InlineLayoutStateStack {
    // cpp: layoutng_inline/inline_box_state.h:176-180
    pub fn Trace(&self, visitor: &mut Visitor) {
        // cpp: layoutng_inline/inline_box_state.cc:292-296
        visitor.Trace(&self.stack_);
        visitor.Trace(&self.box_data_list_);
        visitor.Trace(&self.ruby_column_list_);
    }
    pub fn LineBoxState(&mut self) -> &mut InlineBoxState {
        &mut self.stack_[0]
    }
    pub fn SetIsEmptyLine(&mut self, is_empty_line: bool) {
        self.is_empty_line_ = is_empty_line;
    }

    // cpp: layoutng_inline/inline_box_state.h:182-216
    pub fn OnBeginPlaceItems(
        &mut self,
        node: &InlineNode,
        line_info: &LineInfo,
        baseline: FontBaseline,
        line_height_quirk: bool,
        should_scale_line_height: bool,
        line_box: *mut LogicalLineItems,
    ) -> *mut InlineBoxState {
        // cpp: layoutng_inline/inline_box_state.cc:298-377
        let line_style = line_info.LineStyle();
        self.has_block_in_inline_ = false;
        self.is_svg_text_ = node.IsSvgText();
        if self.stack_.is_empty() {
            self.stack_.push_back(InlineBoxState::default());
            let box_state = self.stack_.last_mut().unwrap();
            box_state.fragment_start = 0;
        } else {
            for i in 0..self.stack_.size() {
                let box_state = &mut self.stack_[i as usize];
                box_state.fragment_start = unsafe { &*line_box }.size();
                if box_state.needs_box_fragment {
                    debug_assert_ne!(i, 0);
                    let text_scale = TextFitBlockScale {
                        paint_scale: line_info.TextFitScale(),
                        scaled_font: std::ptr::null(),
                    };
                    let box_ptr = box_state as *mut InlineBoxState;
                    self.AddBoxFragmentPlaceholder(box_ptr, &text_scale, line_box, baseline);
                }
                let box_state = &mut self.stack_[i as usize];
                if !line_height_quirk {
                    box_state.metrics = box_state.text_metrics;
                } else {
                    box_state.ResetTextMetrics();
                }
                if box_state.has_start_edge {
                    box_state.has_start_edge = box_state.needs_box_fragment
                        && unsafe { &*box_state.style.Get() }.BoxDecorationBreak()
                            == EBoxDecorationBreak::kClone;
                }
                debug_assert!(box_state.pending_descendants.is_empty());
            }
        }
        debug_assert!(self.box_data_list_.is_empty());

        let line_box_state = self.LineBoxState();
        if line_box_state.style.Get() != line_style as *const _ as *mut _
            || (line_style.GetTextFit().Type() != TextFitType::kNone
                && line_style.GetTextFit().Target() != TextFitTarget::kConsistent)
        {
            line_box_state.ResetStyle(line_style, node.IsSvgText(), unsafe {
                &*node.GetLayoutBox()
            });
            let text_scale = TextFitBlockScale {
                paint_scale: line_info.TextFitScale(),
                scaled_font: std::ptr::null(),
            };
            line_box_state.text_fit_scale =
                text_scale.TotalScale(unsafe { &*line_box_state.font.Get() });
            if !line_height_quirk {
                line_box_state.ComputeTextMetrics(
                    line_style,
                    unsafe { &*line_box_state.font.Get() },
                    baseline,
                    &text_scale,
                );
                if RuntimeEnabledFeatures::FirstLineTextMetricsEnabled()
                    && line_style.StyleType() == PseudoId::kPseudoIdFirstLine
                    && node.Style().ComputedLineHeight() > line_style.ComputedLineHeight()
                {
                    let mut first_line_box_state = InlineBoxState::default();
                    first_line_box_state.ResetStyle(node.Style(), node.IsSvgText(), unsafe {
                        &*node.GetLayoutBox()
                    });
                    first_line_box_state.ComputeTextMetrics(
                        node.Style(),
                        unsafe { &*node.Style().GetFont() },
                        baseline,
                        &text_scale,
                    );
                    line_box_state.text_metrics = first_line_box_state.text_metrics;
                    line_box_state.metrics = first_line_box_state.metrics;
                }
            }
        }
        self.stack_.last_mut().unwrap() as *mut InlineBoxState
    }
    pub fn OnOpenTag(
        &mut self,
        space: &ConstraintSpace,
        item: &InlineItem,
        result: &InlineItemResult,
        baseline: FontBaseline,
        line_box: &LogicalLineItems,
    ) -> *mut InlineBoxState {
        // cpp: layoutng_inline/inline_box_state.cc:395-423
        let style_ptr = item.Style();
        debug_assert!(!style_ptr.is_null());
        let style = unsafe { &*style_ptr };
        let parent_scale = self.stack_.last().map(|state| state.text_fit_scale);
        self.stack_.push_back(InlineBoxState::default());
        let box_state = self.stack_.last_mut().unwrap();
        box_state.fragment_start = line_box.size();
        box_state.ResetStyle(style, self.is_svg_text_, unsafe {
            &*item.GetLayoutObject()
        });
        if let Some(parent_scale) = parent_scale {
            box_state.text_fit_scale = parent_scale;
        }
        box_state.item = Member::from_ptr(item as *const _ as *mut _);
        box_state.has_start_edge = true;
        box_state.margins = result.margins;
        box_state.borders = result.borders;
        box_state.padding = result.padding;
        if space.IsInsideRepeatableContent() {
            let layout_inline = DynamicTo::<LayoutInline>(item.GetLayoutObject());
            if !layout_inline.is_null() {
                unsafe { &mut *layout_inline }.SetShouldCreateBoxFragmentDefault();
            }
        }
        box_state as *mut InlineBoxState
    }
    pub fn OnOpenTagWithPlaceholder(
        &mut self,
        space: &ConstraintSpace,
        item: &InlineItem,
        result: &InlineItemResult,
        baseline: FontBaseline,
        text_scale: &TextFitBlockScale,
        line_box: *mut LogicalLineItems,
    ) -> *mut InlineBoxState {
        // cpp: layoutng_inline/inline_box_state.cc:379-393
        let box_state = self.OnOpenTag(space, item, result, baseline, unsafe { &*line_box });
        let box_ref = unsafe { &mut *box_state };
        box_ref.text_fit_scale = text_scale.TotalScale(unsafe { &*box_ref.font.Get() });
        box_ref.needs_box_fragment = item.ShouldCreateBoxFragment();
        if box_ref.needs_box_fragment {
            self.AddBoxFragmentPlaceholder(box_state, text_scale, line_box, baseline);
        }
        box_state
    }
    pub fn OnCloseTag(
        &mut self,
        space: &ConstraintSpace,
        line_box: *mut LogicalLineItems,
        state: *mut InlineBoxState,
        baseline: FontBaseline,
    ) -> *mut InlineBoxState {
        // cpp: layoutng_inline/inline_box_state.cc:425-437
        debug_assert_eq!(
            state,
            self.stack_.last_mut().unwrap() as *mut InlineBoxState
        );
        unsafe { &mut *state }.has_end_edge = true;
        self.EndBoxState(space, self.stack_.size() - 1, line_box, baseline);
        self.stack_.pop();
        self.stack_.last_mut().unwrap() as *mut InlineBoxState
    }
    pub fn OnEndPlaceItems(
        &mut self,
        space: &ConstraintSpace,
        line_box: *mut LogicalLineItems,
        baseline: FontBaseline,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:439-461
        let mut i = self.stack_.size();
        while i > 0 {
            i -= 1;
            let box_state = &mut self.stack_[i as usize];
            if !box_state.has_end_edge
                && box_state.needs_box_fragment
                && unsafe { &*box_state.style.Get() }.BoxDecorationBreak()
                    == EBoxDecorationBreak::kClone
            {
                box_state.has_end_edge = true;
            }
            self.EndBoxState(space, i, line_box, baseline);
        }
        for box_data in &mut self.box_data_list_ {
            let placeholder = &unsafe { &*line_box }[box_data.fragment_start as usize];
            debug_assert!(placeholder.IsPlaceholder());
            box_data.rect.offset = placeholder.rect.offset;
        }
    }
    pub fn OnBlockInInline(&mut self, metrics: &FontHeight, line_box: *mut LogicalLineItems) {
        // cpp: layoutng_inline/inline_box_state.cc:485-500
        debug_assert!(!self.has_block_in_inline_);
        self.has_block_in_inline_ = true;
        for box_state in &mut self.stack_ {
            box_state.metrics = *metrics;
        }
        let line_height = metrics.LineHeight();
        for item in unsafe { &mut *line_box }.iter_mut() {
            debug_assert!(item.IsPlaceholder());
            item.rect.offset.block_offset = LayoutUnit::default();
            item.rect.size.block_size = line_height;
        }
    }

    // cpp: layoutng_inline/inline_box_state.h:218-225
    pub fn CreateRubyColumn(&mut self) -> &mut LogicalRubyColumn {
        // cpp: layoutng_inline/inline_box_state.cc:1448-1451
        let column = MakeGarbageCollected(LogicalRubyColumn::default());
        self.ruby_column_list_.push_back(Member::from_ptr(column));
        unsafe { &mut *column }
    }
    pub fn RubyColumnAt(&self, index: WtfSizeT) -> &LogicalRubyColumn {
        unsafe { &*self.ruby_column_list_[index as usize].Get() }
    }
    pub fn RubyColumnList(&mut self) -> &mut HeapVector<Member<LogicalRubyColumn>> {
        &mut self.ruby_column_list_
    }
    pub fn ClearRubyColumnList(&mut self) {
        self.ruby_column_list_.Shrink(0);
    }

    // cpp: layoutng_inline/inline_box_state.h:227-273
    pub fn HasBoxFragments(&self) -> bool {
        !self.box_data_list_.is_empty()
    }
    pub fn AnnotationBoxBlockAxisMargins(&self) -> Option<(LayoutUnit, LayoutUnit)> {
        // cpp: layoutng_inline/inline_box_state.cc:633-646
        if !self.HasBoxFragments() || self.box_data_list_[0].fragment_start != 0 {
            return None;
        }
        let data = &self.box_data_list_[0];
        if data.padding.BlockSum() == LayoutUnit::default()
            && data.borders.BlockSum() == LayoutUnit::default()
            && data.margin_line_over == LayoutUnit::default()
            && data.margin_line_under == LayoutUnit::default()
        {
            return None;
        }
        Some((data.margin_line_over, data.margin_line_under))
    }
    pub fn ChildInserted(&mut self, index: u32) {
        // cpp: layoutng_inline/inline_box_state.cc:648-660
        for state in &mut self.stack_ {
            if state.fragment_start >= index {
                state.fragment_start += 1;
            }
            debug_assert!(state.pending_descendants.is_empty());
        }
        for box_data in &mut self.box_data_list_ {
            if box_data.fragment_start >= index {
                box_data.fragment_start += 1;
            }
            if box_data.fragment_end >= index {
                box_data.fragment_end += 1;
            }
        }
    }
    pub fn PrepareForReorder(&mut self, line_box: *mut LogicalLineItems) {
        // cpp: layoutng_inline/inline_box_state.cc:662-692
        if self.box_data_list_.is_empty() {
            return;
        }
        let line_box = unsafe { &mut *line_box };
        let mut box_data_index = 0u32;
        for data_index in 0..self.box_data_list_.len() {
            box_data_index += 1;
            let start = self.box_data_list_[data_index].fragment_start;
            let end = self.box_data_list_[data_index].fragment_end;
            debug_assert!(line_box[start as usize].IsPlaceholder());
            for i in start..end {
                let child = &mut line_box[i as usize];
                let mut child_box_data_index = child.box_data_index;
                if child_box_data_index == 0 {
                    child.box_data_index = box_data_index;
                    continue;
                }
                while child_box_data_index != box_data_index {
                    let child_data = &mut self.box_data_list_[(child_box_data_index - 1) as usize];
                    child_box_data_index = child_data.parent_box_data_index;
                    if child_box_data_index == 0 {
                        child_data.parent_box_data_index = box_data_index;
                        break;
                    }
                }
            }
        }
    }
    pub fn UpdateAfterReorder(&mut self, line_box: *mut LogicalLineItems) {
        // cpp: layoutng_inline/inline_box_state.cc:694-726
        if self.box_data_list_.is_empty() {
            return;
        }
        for box_data in &mut self.box_data_list_ {
            box_data.fragment_start = 0;
            box_data.fragment_end = 0;
        }
        let mut fragmented_boxes = HeapVector::<BoxData>::default();
        let mut index = 0u32;
        while index < unsafe { &*line_box }.size() {
            index = self.UpdateBoxDataFragmentRange(line_box, index, &mut fragmented_boxes);
        }
        if !fragmented_boxes.is_empty() {
            self.UpdateFragmentedBoxDataEdges(&mut fragmented_boxes);
        }
        #[cfg(debug_assertions)]
        {
            for box_data in &self.box_data_list_ {
                debug_assert_ne!(box_data.fragment_end, 0);
                debug_assert!(box_data.fragment_end > box_data.fragment_start);
            }
            for child in unsafe { &*line_box }.iter() {
                debug_assert_eq!(child.box_data_index, 0);
            }
        }
    }
    pub fn ComputeInlinePositions(
        &mut self,
        line_box: *mut LogicalLineItems,
        mut position: LayoutUnit,
        ignore_edges: bool,
    ) -> LayoutUnit {
        // cpp: layoutng_inline/inline_box_state.cc:847-930
        let line_box = unsafe { &mut *line_box };
        for child in line_box.iter_mut() {
            child.margin_line_left = child.rect.offset.inline_offset;
            child.rect.offset.inline_offset += position;
            if !child.HasFragment() && !child.IsRubyLinePlaceholder() {
                continue;
            }
            position += child.inline_size;
        }
        if self.box_data_list_.is_empty() {
            return position;
        }
        let line_size = line_box.size();
        if !ignore_edges {
            for box_data in &self.box_data_list_ {
                let start = box_data.fragment_start;
                let end = box_data.fragment_end;
                debug_assert!(end > start);
                if box_data.margin_border_padding_line_left != LayoutUnit::default() {
                    line_box.MoveInInlineDirectionRange(
                        box_data.margin_border_padding_line_left,
                        start,
                        line_size,
                    );
                    position += box_data.margin_border_padding_line_left;
                }
                if box_data.margin_border_padding_line_right != LayoutUnit::default() {
                    line_box.MoveInInlineDirectionRange(
                        box_data.margin_border_padding_line_right,
                        end,
                        line_size,
                    );
                    position += box_data.margin_border_padding_line_right;
                }
            }
        }
        #[derive(Clone, Copy, Default)]
        struct LinePadding {
            line_left: LayoutUnit,
            line_right: LayoutUnit,
        }
        let mut accumulated_padding = vec![LinePadding::default(); line_size as usize];
        for box_data in &mut self.box_data_list_ {
            let start = box_data.fragment_start as usize;
            let start_child = &line_box[start];
            let mut line_left_offset =
                start_child.rect.offset.inline_offset - start_child.margin_line_left;
            debug_assert!(box_data.fragment_end > box_data.fragment_start);
            let last = (box_data.fragment_end - 1) as usize;
            let last_child = &line_box[last];
            let mut line_right_offset = last_child.rect.offset.inline_offset
                - last_child.margin_line_left
                + last_child.inline_size;
            if !ignore_edges {
                accumulated_padding[start].line_left += box_data.margin_border_padding_line_left;
                accumulated_padding[last].line_right += box_data.margin_border_padding_line_right;
                line_left_offset += box_data.margin_line_left;
                line_right_offset -= box_data.margin_line_right;
            }
            line_left_offset -= accumulated_padding[start].line_left;
            line_right_offset += accumulated_padding[last].line_right;
            box_data.rect.offset.inline_offset = line_left_offset;
            box_data.rect.size.inline_size = line_right_offset - line_left_offset;
        }
        position
    }
    pub fn MoveBoxDataInBlockDirection(&mut self, diff: LayoutUnit) {
        // cpp: layoutng_inline/inline_box_state.cc:932-936
        for box_data in &mut self.box_data_list_ {
            box_data.rect.offset.block_offset += diff;
        }
    }
    pub fn MoveBoxDataInInlineDirection(&mut self, diff: LayoutUnit) {
        // cpp: layoutng_inline/inline_box_state.cc:938-942
        for box_data in &mut self.box_data_list_ {
            box_data.rect.offset.inline_offset += diff;
        }
    }
    pub fn ApplyRelativePositioning(
        &mut self,
        space: &ConstraintSpace,
        line_box: *mut LogicalLineItems,
        parent_offset: *const LogicalOffset,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:944-988
        if self.box_data_list_.is_empty()
            && self.ruby_column_list_.is_empty()
            && parent_offset.is_null()
        {
            return;
        }
        let line_box = unsafe { &mut *line_box };
        let mut accumulated_offsets = vec![LogicalOffset::default(); line_box.size() as usize];
        if !parent_offset.is_null() {
            for index in 0..line_box.size() as usize {
                line_box[index].rect.offset += unsafe { *parent_offset };
                accumulated_offsets[index] = unsafe { *parent_offset };
            }
        }
        for box_data in &mut self.box_data_list_ {
            let start = box_data.fragment_start;
            let end = box_data.fragment_end;
            let style = unsafe { &*(&*box_data.item.Get()).Style() };
            let relative_offset = ComputeRelativeOffsetForInline(space, style);
            for index in start..end {
                line_box[index as usize].rect.offset += relative_offset;
                accumulated_offsets[index as usize] += relative_offset;
            }
        }
        for box_data in &mut self.box_data_list_ {
            box_data.rect.offset += accumulated_offsets[box_data.fragment_start as usize];
        }
        for column in &mut self.ruby_column_list_ {
            let column = unsafe { &mut *column.Get() };
            column.state_stack.ApplyRelativePositioning(
                space,
                column.annotation_items.Get(),
                &accumulated_offsets[column.start_index as usize],
            );
        }
    }
    pub fn CreateBoxFragments(
        &mut self,
        space: &ConstraintSpace,
        line_box: *mut LogicalLineItems,
        line_height: LayoutUnit,
        opaque: bool,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:990-1027
        for column in &mut self.ruby_column_list_ {
            let column = unsafe { &mut *column.Get() };
            column.state_stack.CreateBoxFragments(
                space,
                column.annotation_items.Get(),
                line_height,
                false,
            );
        }
        if !self.HasBoxFragments() {
            return;
        }
        let line_box = unsafe { &mut *line_box };
        for data_index in 0..self.box_data_list_.len() {
            let box_data = &mut self.box_data_list_[data_index];
            let start = box_data.fragment_start;
            let end = box_data.fragment_end;
            debug_assert!(end > start);
            debug_assert!(unsafe { &*box_data.item.Get() }.ShouldCreateBoxFragment());
            let box_fragment = box_data.CreateBoxFragment(space, line_box, line_height, opaque);
            if line_box[start as usize].IsPlaceholder() {
                let child = &mut line_box[start as usize];
                child.layout_result = Member::from_ptr(box_fragment.cast_mut());
                child.rect = box_data.rect;
                child.children_count = end - start;
                continue;
            }
            line_box.InsertChildLayoutResult(start, box_fragment, &box_data.rect, end - start + 1);
            self.ChildInserted(start + 1);
        }
        self.box_data_list_.clear();
    }
    #[cfg(debug_assertions)]
    pub fn CheckSame(&self, other: &Self) {
        // cpp: layoutng_inline/inline_box_state.cc:1453-1465
        debug_assert_eq!(self.box_data_list_.size(), 0);
        debug_assert_eq!(other.box_data_list_.size(), 0);
        debug_assert_eq!(self.stack_.size(), other.stack_.size());
        for i in 0..self.stack_.len() {
            self.stack_[i].CheckSame(&other.stack_[i]);
        }
    }

    // cpp: layoutng_inline/inline_box_state.h:275-311,372-384
    // Private methods are exposed crate-wide for the pending .cc translation.
    pub(crate) fn EndBoxState(
        &mut self,
        space: &ConstraintSpace,
        index: WtfSizeT,
        line_box: *mut LogicalLineItems,
        baseline: FontBaseline,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:463-483
        let box_state = &self.stack_[index as usize] as *const InlineBoxState;
        if unsafe { &*box_state }.needs_box_fragment {
            self.AddBoxData(space, box_state, line_box);
        }
        let pending = self.ApplyBaselineShift(index, line_box, baseline);
        if index == 0 {
            return;
        }
        if pending == PositionPending::kPositionNotPending {
            self.stack_[(index - 1) as usize]
                .metrics
                .Unite(&unsafe { &*box_state }.metrics);
        }
    }
    pub(crate) fn AddBoxFragmentPlaceholder(
        &mut self,
        state: *mut InlineBoxState,
        scale: &TextFitBlockScale,
        line_box: *mut LogicalLineItems,
        baseline: FontBaseline,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:503-548
        debug_assert_ne!(state, self.stack_.as_ptr() as *mut InlineBoxState);
        let state = unsafe { &mut *state };
        debug_assert_ne!(
            unsafe { &*state.item.Get() }.Type(),
            InlineItemType::kAtomicInline
        );
        state.has_box_placeholder = true;
        let mut block_offset = LayoutUnit::default();
        let mut block_size = LayoutUnit::default();
        if !self.is_empty_line_ {
            let mut metrics = FontHeight::default();
            let font = if !scale.scaled_font.is_null() {
                scale.scaled_font
            } else {
                state.font.Get()
            };
            let font_data = unsafe { &*font }.PrimaryFont();
            if !font_data.is_null() {
                let font_metrics = unsafe { &*font_data }.GetFontMetrics();
                let paint_scale = scale.paint_scale;
                if paint_scale != 1.0 {
                    metrics = font_metrics.GetFloatFontHeight(baseline);
                    metrics.ascent *= paint_scale;
                    metrics.descent *= paint_scale;
                } else {
                    metrics = if self.is_svg_text_ {
                        font_metrics.GetFloatFontHeight(baseline)
                    } else {
                        font_metrics.GetFontHeight(baseline)
                    };
                }
            }
            block_offset = -metrics.ascent - (state.borders.line_over + state.padding.line_over);
            block_size = metrics.LineHeight() + state.borders.BlockSum() + state.padding.BlockSum();
        }
        let line_box = unsafe { &mut *line_box };
        line_box.AddChild(LogicalLineItem::placeholder(block_offset, block_size));
        debug_assert!(line_box[(line_box.size() - 1) as usize].IsPlaceholder());
    }
    pub(crate) fn AddBoxData(
        &mut self,
        space: &ConstraintSpace,
        state: *const InlineBoxState,
        line_box: *mut LogicalLineItems,
    ) {
        // cpp: layoutng_inline/inline_box_state.cc:550-631
        let state = unsafe { &*state };
        debug_assert!(state.needs_box_fragment);
        debug_assert!(!state.style.Get().is_null());
        let style = unsafe { &*state.style.Get() };
        let line_box = unsafe { &mut *line_box };
        let start = state.fragment_start;
        let placeholder = &line_box[start as usize];
        debug_assert!(placeholder.IsPlaceholder());
        let fragment_end = line_box.size();
        debug_assert!(!state.item.Get().is_null());
        self.box_data_list_.push_back(BoxData::new(
            start,
            fragment_end,
            state.item.Get(),
            *placeholder.Size(),
        ));
        let box_data = self.box_data_list_.last_mut().unwrap();
        box_data.borders = state.borders;
        box_data.padding = state.padding;
        box_data.margin_line_over = state.margins.line_over;
        box_data.margin_line_under = state.margins.line_under;
        if state.has_start_edge {
            box_data.has_line_left_edge = true;
            box_data.margin_line_left = state.margins.inline_start;
            box_data.margin_border_padding_line_left = state.margins.inline_start
                + state.borders.inline_start
                + state.padding.inline_start;
        } else {
            box_data.borders.inline_start = LayoutUnit::default();
            box_data.padding.inline_start = LayoutUnit::default();
        }
        if state.has_end_edge {
            box_data.has_line_right_edge = true;
            box_data.margin_line_right = state.margins.inline_end;
            box_data.margin_border_padding_line_right =
                state.margins.inline_end + state.borders.inline_end + state.padding.inline_end;
        } else {
            box_data.borders.inline_end = LayoutUnit::default();
            box_data.padding.inline_end = LayoutUnit::default();
        }
        if IsRtl(style.Direction()) {
            std::mem::swap(
                &mut box_data.has_line_left_edge,
                &mut box_data.has_line_right_edge,
            );
            std::mem::swap(
                &mut box_data.margin_line_left,
                &mut box_data.margin_line_right,
            );
            std::mem::swap(
                &mut box_data.margin_border_padding_line_left,
                &mut box_data.margin_border_padding_line_right,
            );
        }
        for column_ptr in &self.ruby_column_list_ {
            let column = unsafe { &*column_ptr.Get() };
            if column.annotation_items.Get().is_null() {
                continue;
            }
            if start <= column.start_index && column.EndIndex() <= fragment_end {
                if box_data.ruby_column_list.Get().is_null() {
                    let list = MakeGarbageCollected(
                        GCedHeapVector::<Member<LogicalRubyColumn>>::default(),
                    );
                    box_data.ruby_column_list = Member::from_ptr(list);
                }
                unsafe { &mut *box_data.ruby_column_list.Get() }.push_back(*column_ptr);
            }
        }
        debug_assert!(line_box[start as usize].IsPlaceholder());
        debug_assert!(fragment_end > start);
        if fragment_end > start + 1 {
            return;
        }
        let placeholder = &mut line_box[start as usize];
        placeholder.rect.offset.inline_offset += box_data.margin_line_left;
        let item_style = unsafe { &*(&*box_data.item.Get()).Style() };
        placeholder.rect.offset += ComputeRelativeOffsetForInline(space, item_style);
        let advance =
            box_data.margin_border_padding_line_left + box_data.margin_border_padding_line_right;
        box_data.rect.size.inline_size =
            advance - box_data.margin_line_left - box_data.margin_line_right;
        let result = box_data.CreateBoxFragment(space, line_box, LayoutUnit::default(), false);
        line_box[start as usize].layout_result = Member::from_ptr(result.cast_mut());
        line_box[start as usize].inline_size = advance;
        debug_assert_eq!(line_box[start as usize].children_count, 0);
        self.box_data_list_.pop();
    }
    pub(crate) fn ApplyBaselineShift(
        &mut self,
        index: WtfSizeT,
        line_box: *mut LogicalLineItems,
        baseline: FontBaseline,
    ) -> PositionPending {
        // cpp: layoutng_inline/inline_box_state.cc:1144-1362
        let box_index = index as usize;
        if self.has_block_in_inline_ {
            debug_assert!(self.stack_[box_index].pending_descendants.is_empty());
            return PositionPending::kPositionNotPending;
        }
        let mut baseline_shift = LayoutUnit::default();
        if !self.stack_[box_index].pending_descendants.is_empty() {
            let mut has_top_or_bottom = false;
            for child_index in 0..self.stack_[box_index].pending_descendants.len() {
                let child = &self.stack_[box_index].pending_descendants[child_index];
                let child_metrics = if child.metrics.IsEmpty() {
                    FontHeight::default()
                } else {
                    child.metrics
                };
                let child_align = child.vertical_align;
                let (start, end) = (child.fragment_start, child.fragment_end);
                self.stack_[box_index].pending_descendants[child_index].metrics = child_metrics;
                baseline_shift = match child_align {
                    EVerticalAlign::kTextTop => {
                        child_metrics.ascent + self.stack_[box_index].TextTop(baseline)
                    }
                    EVerticalAlign::kTextBottom => {
                        let font_data =
                            unsafe { &*self.stack_[box_index].font.Get() }.PrimaryFont();
                        if font_data.is_null() {
                            baseline_shift
                        } else {
                            unsafe { &*font_data }
                                .GetFontMetrics()
                                .FixedDescent(baseline)
                                - child_metrics.descent
                        }
                    }
                    EVerticalAlign::kTop | EVerticalAlign::kBottom => {
                        has_top_or_bottom = true;
                        continue;
                    }
                    _ => unreachable!("unexpected pending vertical alignment"),
                };
                let child = &mut self.stack_[box_index].pending_descendants[child_index];
                child.metrics.Move(baseline_shift);
                let adjusted_metrics = child.metrics;
                self.stack_[box_index].metrics.Unite(&adjusted_metrics);
                unsafe { &mut *line_box }.MoveInBlockDirectionRange(baseline_shift, start, end);
            }
            if has_top_or_bottom {
                let max_metrics = self
                    .MetricsForTopAndBottomAlign(&self.stack_[box_index], unsafe { &*line_box });
                for child_index in 0..self.stack_[box_index].pending_descendants.len() {
                    let child = &self.stack_[box_index].pending_descendants[child_index];
                    let (start, end, metrics) =
                        (child.fragment_start, child.fragment_end, child.metrics);
                    baseline_shift = match child.vertical_align {
                        EVerticalAlign::kTop => metrics.ascent - max_metrics.ascent,
                        EVerticalAlign::kBottom => max_metrics.descent - metrics.descent,
                        EVerticalAlign::kTextTop | EVerticalAlign::kTextBottom => continue,
                        _ => unreachable!("unexpected pending vertical alignment"),
                    };
                    let child = &mut self.stack_[box_index].pending_descendants[child_index];
                    child.metrics.Move(baseline_shift);
                    let adjusted_metrics = child.metrics;
                    self.stack_[box_index].metrics.Unite(&adjusted_metrics);
                    unsafe { &mut *line_box }.MoveInBlockDirectionRange(baseline_shift, start, end);
                }
            }
            self.stack_[box_index].pending_descendants.clear();
        }
        let style = unsafe { &*self.stack_[box_index].style.Get() };
        let vertical_align = style.VerticalAlign();
        if !self.is_svg_text_ && vertical_align == EVerticalAlign::kBaseline {
            return PositionPending::kPositionNotPending;
        }
        let item = self.stack_[box_index].item.Get();
        if !item.is_null() && unsafe { &*(&*item).GetLayoutObject() }.IsLayoutTextCombine() {
            return PositionPending::kPositionNotPending;
        }
        let fragment_end = unsafe { &*line_box }.size();
        let fragment_start = self.stack_[box_index].fragment_start;
        if fragment_start == fragment_end {
            return PositionPending::kPositionNotPending;
        }
        if self.is_svg_text_ {
            match style.BaselineShiftType() {
                EBaselineShiftType::kLength => {
                    let font = unsafe { &*self.stack_[box_index].font.Get() };
                    let computed_font_size = font.GetFontDescription().ComputedPixelSize() as f32
                        / self.stack_[box_index].scaling_factor;
                    let algorithms = LayoutPassScope::Algorithms();
                    let resolve_length = if algorithms.is_null() {
                        None
                    } else {
                        unsafe { &*algorithms }.svg_support.resolve_length
                    };
                    let Some(resolve_length) = resolve_length else {
                        std::panic::panic_any(UnsupportedLayout::new(
                            "SVG length module is not installed",
                        ));
                    };
                    baseline_shift = LayoutUnit::from_f32(
                        -resolve_length(style.BaselineShift(), style, computed_font_size)
                            * self.stack_[box_index].scaling_factor,
                    );
                }
                EBaselineShiftType::kSub | EBaselineShiftType::kSuper => {
                    let font_data = unsafe { &*self.stack_[box_index].font.Get() }.PrimaryFont();
                    if !font_data.is_null() {
                        let half_height =
                            unsafe { &*font_data }.GetFontMetrics().FloatHeight() / 2.0;
                        baseline_shift = LayoutUnit::from_f32(
                            if style.BaselineShiftType() == EBaselineShiftType::kSub {
                                half_height
                            } else {
                                -half_height
                            },
                        );
                    }
                }
            }
            baseline_shift += self.ComputeAlignmentBaselineShift(index);
            if !self.stack_[box_index].metrics.IsEmpty() {
                self.stack_[box_index].metrics.Move(baseline_shift);
            }
            unsafe { &mut *line_box }.MoveInBlockDirectionRange(
                baseline_shift,
                fragment_start,
                fragment_end,
            );
            return PositionPending::kPositionNotPending;
        }
        if index == 0 {
            return PositionPending::kPositionNotPending;
        }
        let parent_index = box_index - 1;
        match vertical_align {
            EVerticalAlign::kSub | EVerticalAlign::kSuper => {
                let parent = &self.stack_[parent_index];
                let mut font_size = unsafe { &*parent.style.Get() }.ComputedFontSizeAsFixedValue();
                if parent.text_fit_scale != 1.0 {
                    font_size = LayoutUnit::from_f32(font_size.ToFloat() * parent.text_fit_scale);
                }
                baseline_shift = if vertical_align == EVerticalAlign::kSub {
                    font_size / 5i32 + LayoutUnit::from_signed(1)
                } else {
                    -(font_size / 3i32 + LayoutUnit::from_signed(1))
                };
            }
            EVerticalAlign::kLength => {
                let length = style.GetVerticalAlignLength();
                let line_height = if length.HasPercent() {
                    let mut line_height = style.ComputedLineHeightAsFixed();
                    if !style.LineHeight().IsFixed() && self.stack_[box_index].text_fit_scale != 1.0
                    {
                        line_height = LayoutUnit::from_f32(
                            line_height.ToFloat() * self.stack_[box_index].text_fit_scale,
                        );
                    }
                    line_height
                } else {
                    self.stack_[box_index].text_metrics.LineHeight()
                };
                baseline_shift = -ValueForLength(length, line_height);
            }
            EVerticalAlign::kMiddle | EVerticalAlign::kBaselineMiddle => {
                let metrics = self.stack_[box_index].metrics;
                baseline_shift = (metrics.ascent - metrics.descent) / 2;
                if vertical_align == EVerticalAlign::kMiddle {
                    let font = unsafe { &*self.stack_[parent_index].style.Get() }.GetFont();
                    let font_data = unsafe { &*font }.PrimaryFont();
                    if !font_data.is_null() {
                        baseline_shift -= LayoutUnit::FromFloatRound(
                            unsafe { &*font_data }.GetFontMetrics().XHeight() / 2.0,
                        );
                    }
                }
            }
            EVerticalAlign::kTop | EVerticalAlign::kBottom => {
                let mut ancestor_index = parent_index;
                while ancestor_index > 0 {
                    let align =
                        unsafe { &*self.stack_[ancestor_index].style.Get() }.VerticalAlign();
                    if align == EVerticalAlign::kTop || align == EVerticalAlign::kBottom {
                        break;
                    }
                    ancestor_index -= 1;
                }
                let metrics = self.stack_[box_index].metrics;
                self.stack_[ancestor_index]
                    .pending_descendants
                    .push(PendingPositions {
                        fragment_start,
                        fragment_end,
                        metrics,
                        vertical_align,
                    });
                return PositionPending::kPositionPending;
            }
            _ => {
                let metrics = self.stack_[box_index].metrics;
                self.stack_[parent_index]
                    .pending_descendants
                    .push(PendingPositions {
                        fragment_start,
                        fragment_end,
                        metrics,
                        vertical_align,
                    });
                return PositionPending::kPositionPending;
            }
        }
        if !self.stack_[box_index].metrics.IsEmpty() {
            self.stack_[box_index].metrics.Move(baseline_shift);
        }
        unsafe { &mut *line_box }.MoveInBlockDirectionRange(
            baseline_shift,
            fragment_start,
            fragment_end,
        );
        PositionPending::kPositionNotPending
    }
    pub(crate) fn ComputeAlignmentBaselineShift(&self, index: WtfSizeT) -> LayoutUnit {
        // cpp: layoutng_inline/inline_box_state.cc:1364-1385
        let box_state = &self.stack_[index as usize];
        let mut result = LayoutUnit::default();
        let font_data = unsafe { &*box_state.font.Get() }.PrimaryFont();
        if !font_data.is_null() {
            let metrics = unsafe { &*font_data }.GetFontMetrics();
            result = metrics.FixedAscent(unsafe { &*box_state.style.Get() }.GetFontBaseline())
                - metrics.FixedAscent(box_state.alignment_type);
        }
        if index == 0 {
            return result;
        }
        let parent = &self.stack_[(index - 1) as usize];
        let font_data = unsafe { &*parent.font.Get() }.PrimaryFont();
        if !font_data.is_null() {
            let parent_metrics = unsafe { &*font_data }.GetFontMetrics();
            result -= parent_metrics.FixedAscent(unsafe { &*parent.style.Get() }.GetFontBaseline())
                - parent_metrics.FixedAscent(parent.alignment_type);
        }
        result
    }
    pub(crate) fn MetricsForTopAndBottomAlign(
        &self,
        state: &InlineBoxState,
        line_box: &LogicalLineItems,
    ) -> FontHeight {
        // cpp: layoutng_inline/inline_box_state.cc:1387-1446
        debug_assert!(!state.pending_descendants.is_empty());
        let mut metrics = state.metrics;
        for box_data in &self.box_data_list_ {
            let style_ptr = unsafe { &*box_data.item.Get() }.Style();
            debug_assert!(!style_ptr.is_null());
            let style = unsafe { &*style_ptr };
            let vertical_align = style.VerticalAlign();
            if vertical_align == EVerticalAlign::kTop || vertical_align == EVerticalAlign::kBottom {
                continue;
            }
            let placeholder = &line_box[box_data.fragment_start as usize];
            debug_assert!(placeholder.IsPlaceholder());
            let box_ascent = -placeholder.rect.offset.block_offset;
            let mut box_metrics =
                FontHeight::new(box_ascent, box_data.rect.size.block_size - box_ascent);
            box_metrics.ascent -= box_data.padding.line_over;
            box_metrics.descent -= box_data.padding.line_under;
            let leading_space =
                CalculateLeadingSpace(&style.ComputedLineHeightAsFixed(), &box_metrics);
            box_metrics.AddLeading(&leading_space);
            metrics.Unite(&box_metrics);
        }
        if metrics.IsEmpty() {
            metrics = FontHeight::default();
        }
        let mut max_metrics = metrics;
        for child in &state.pending_descendants {
            if matches!(
                child.vertical_align,
                EVerticalAlign::kTop | EVerticalAlign::kBottom
            ) && child.metrics.LineHeight() > max_metrics.LineHeight()
            {
                if child.vertical_align == EVerticalAlign::kTop {
                    max_metrics = FontHeight::new(
                        metrics.ascent,
                        child.metrics.LineHeight() - metrics.ascent,
                    );
                } else {
                    max_metrics = FontHeight::new(
                        child.metrics.LineHeight() - metrics.descent,
                        metrics.descent,
                    );
                }
            }
        }
        max_metrics
    }
    pub(crate) fn UpdateBoxDataFragmentRange(
        &mut self,
        line_box: *mut LogicalLineItems,
        mut index: u32,
        fragmented: *mut HeapVector<BoxData>,
    ) -> u32 {
        // cpp: layoutng_inline/inline_box_state.cc:728-784
        let line_size = unsafe { &*line_box }.size();
        while index < line_size {
            let box_data_index = unsafe { &*line_box }[index as usize].box_data_index;
            if box_data_index == 0 {
                index += 1;
                continue;
            }
            let data_index = (box_data_index - 1) as usize;
            (unsafe { &mut *line_box })[index as usize].box_data_index =
                self.box_data_list_[data_index].parent_box_data_index;
            let start_index = index;
            index += 1;
            while index < line_size {
                while {
                    let end_index = unsafe { &*line_box }[index as usize].box_data_index;
                    end_index != 0 && end_index < box_data_index
                } {
                    self.UpdateBoxDataFragmentRange(line_box, index, fragmented);
                }
                if box_data_index != unsafe { &*line_box }[index as usize].box_data_index {
                    break;
                }
                (unsafe { &mut *line_box })[index as usize].box_data_index =
                    self.box_data_list_[data_index].parent_box_data_index;
                index += 1;
            }
            if self.box_data_list_[data_index].fragment_end == 0 {
                self.box_data_list_[data_index].SetFragmentRange(start_index, index);
            } else {
                let mut fragmented_box =
                    BoxData::from_other(&self.box_data_list_[data_index], start_index, index);
                fragmented_box.fragmented_box_data_index = box_data_index;
                unsafe { &mut *fragmented }.push_back(fragmented_box);
            }
            if self.box_data_list_[data_index].parent_box_data_index != 0 {
                return start_index;
            }
            return index;
        }
        index
    }
    pub(crate) fn UpdateFragmentedBoxDataEdges(&mut self, fragmented: *mut HeapVector<BoxData>) {
        // cpp: layoutng_inline/inline_box_state.cc:786-827
        let fragmented = unsafe { &mut *fragmented };
        debug_assert!(!fragmented.is_empty());
        fragmented.sort_by(|a, b| {
            a.fragmented_box_data_index
                .cmp(&b.fragmented_box_data_index)
                .then_with(|| {
                    debug_assert_ne!(a.fragment_start, b.fragment_start);
                    a.fragment_start.cmp(&b.fragment_start)
                })
        });
        let sorted = std::mem::take(fragmented);
        for mut fragmented_box in sorted.into_iter().rev() {
            let insert_at = fragmented_box.fragmented_box_data_index;
            debug_assert!(insert_at > 0);
            fragmented_box.fragmented_box_data_index = 0;
            self.box_data_list_
                .insert(insert_at as usize, fragmented_box);
            for box_data in &mut self.box_data_list_ {
                if box_data.fragmented_box_data_index >= insert_at {
                    box_data.fragmented_box_data_index += 1;
                }
            }
            let fragmented_from = (insert_at - 1) as usize;
            if self.box_data_list_[fragmented_from].fragmented_box_data_index == 0 {
                self.box_data_list_[fragmented_from].fragmented_box_data_index = insert_at;
            }
        }
        for index in 0..self.box_data_list_.len() {
            if self.box_data_list_[index].fragmented_box_data_index != 0 {
                BoxData::UpdateFragmentEdges(&mut self.box_data_list_, index);
            }
        }
    }
}

// cpp: layoutng_inline/inline_box_state.h:291
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PositionPending {
    kPositionNotPending,
    kPositionPending,
}

// cpp: layoutng_inline/inline_box_state.h:313-370
pub struct BoxData {
    pub fragment_start: u32,
    pub fragment_end: u32,
    pub ruby_column_list: Member<GCedHeapVector<Member<LogicalRubyColumn>>>,
    pub item: Member<InlineItem>,
    pub rect: LogicalRect,
    pub has_line_left_edge: bool,
    pub has_line_right_edge: bool,
    pub borders: LineBoxStrut,
    pub padding: LineBoxStrut,
    pub margin_line_over: LayoutUnit,
    pub margin_line_under: LayoutUnit,
    pub margin_line_left: LayoutUnit,
    pub margin_line_right: LayoutUnit,
    pub margin_border_padding_line_left: LayoutUnit,
    pub margin_border_padding_line_right: LayoutUnit,
    pub parent_box_data_index: u32,
    pub fragmented_box_data_index: u32,
}

impl BoxData {
    // cpp: layoutng_inline/inline_box_state.h:319-336
    pub fn new(start: u32, end: u32, item: *const InlineItem, size: LogicalSize) -> Self {
        Self {
            fragment_start: start,
            fragment_end: end,
            ruby_column_list: Member::default(),
            item: Member::from_ptr(item.cast_mut()),
            rect: LogicalRect::new(LogicalOffset::default(), size),
            has_line_left_edge: false,
            has_line_right_edge: false,
            borders: LineBoxStrut::default(),
            padding: LineBoxStrut::default(),
            margin_line_over: LayoutUnit::default(),
            margin_line_under: LayoutUnit::default(),
            margin_line_left: LayoutUnit::default(),
            margin_line_right: LayoutUnit::default(),
            margin_border_padding_line_left: LayoutUnit::default(),
            margin_border_padding_line_right: LayoutUnit::default(),
            parent_box_data_index: 0,
            fragmented_box_data_index: 0,
        }
    }
    pub fn from_other(other: &Self, start: u32, end: u32) -> Self {
        let mut result = Self::new(start, end, other.item.Get(), other.rect.size);
        result.rect = other.rect;
        result
    }
    pub fn SetFragmentRange(&mut self, start: u32, end: u32) {
        self.fragment_start = start;
        self.fragment_end = end;
    }

    // cpp: layoutng_inline/inline_box_state.h:363-369
    // cpp: layoutng_inline/inline_box_state.cc:829-845
    // The source passes `this` and its containing vector together; an index
    // gives Rust two disjoint mutable slots without aliased &mut references.
    pub fn UpdateFragmentEdges(list: &mut HeapVector<BoxData, 4>, index: usize) {
        let last_index = list[index].fragmented_box_data_index as usize;
        debug_assert!(last_index != 0);
        if list[index].has_line_right_edge {
            let margin_line_right = list[index].margin_line_right;
            let margin_border_padding_line_right = list[index].margin_border_padding_line_right;
            let padding_inline_end = list[index].padding.inline_end;
            let last = &mut list[last_index];
            last.has_line_right_edge = true;
            last.margin_line_right = margin_line_right;
            last.margin_border_padding_line_right = margin_border_padding_line_right;
            last.padding.inline_end = padding_inline_end;
            let current = &mut list[index];
            current.has_line_right_edge = false;
            current.margin_line_right = LayoutUnit::default();
            current.margin_border_padding_line_right = LayoutUnit::default();
            current.padding.inline_end = LayoutUnit::default();
        }
    }
    pub fn CreateBoxFragment(
        &mut self,
        space: &ConstraintSpace,
        items: *mut LogicalLineItems,
        line_height: LayoutUnit,
        opaque: bool,
    ) -> *const LayoutResult {
        // cpp: layoutng_inline/inline_box_state.cc:1029-1137
        let item = unsafe { &*self.item.Get() };
        let style_ptr = item.Style();
        debug_assert!(!style_ptr.is_null());
        let style = unsafe { &*style_ptr };
        let flipped = IsFlippedLinesWritingMode(style.GetWritingMode());
        let geometry = FragmentGeometry {
            border_box_size: LogicalSize::new(
                self.rect.size.inline_size.ClampNegativeToZero(),
                self.rect.size.block_size,
            ),
            border: BoxStrut::from_line(&self.borders, flipped),
            padding: BoxStrut::from_line(&self.padding, flipped),
            ..FragmentGeometry::default()
        };
        let mut box_builder = BoxFragmentBuilder::new_for_layout_object(
            item.GetLayoutObject(),
            style_ptr,
            space,
            WritingDirectionMode::new(style.GetWritingMode(), TextDirection::kLtr),
        );
        box_builder.SetInitialFragmentGeometry(&geometry);
        box_builder.SetBoxType(BoxType::kInlineBox);
        box_builder.SetStyleVariant(item.GetStyleVariant());
        if opaque {
            box_builder.SetIsOpaque();
            box_builder.SetSidesToInclude(LineLogicalBoxSides::new(false, false, false, false));
        } else {
            box_builder.SetSidesToInclude(LineLogicalBoxSides::new(
                true,
                self.has_line_right_edge,
                true,
                self.has_line_left_edge,
            ));
        }
        box_builder.SetIsMonolithic(!space.HasBlockFragmentation());

        let rect_offset = self.rect.offset;
        let mut handle_box_child = |child: &mut LogicalLineItem| {
            if !child.out_of_flow_positioned_box.Get().is_null() {
                debug_assert!(unsafe { &*item.GetLayoutObject() }.IsLayoutInline());
                let oof_box =
                    BlockNode::new(To::<LayoutBox>(child.out_of_flow_positioned_box.Get()));
                let static_offset = LogicalOffset::from(child.rect.offset - rect_offset);
                box_builder.AddOutOfFlowInlineChildCandidate(
                    oof_box,
                    &static_offset,
                    child.container_writing_direction,
                    line_height,
                );
                child.out_of_flow_positioned_box.Clear();
                return;
            }
            let child_result = child.layout_result.Get();
            if !child_result.is_null() {
                // LogicalLineItem::Style reads the physical fragment style.
                let child_style = unsafe { &*child.Style() };
                let relative_offset_for_child = ComputeRelativeOffsetForInline(space, child_style);
                box_builder.PropagateFromLayoutResultAndFragment(
                    unsafe { &*child_result },
                    LogicalOffset::new(
                        child.rect.offset.inline_offset
                            - rect_offset.inline_offset
                            - relative_offset_for_child.inline_offset,
                        child.rect.offset.block_offset
                            - rect_offset.block_offset
                            - relative_offset_for_child.block_offset,
                    ),
                    ComputeRelativeOffsetForOOFInInline(space, child_style),
                    std::ptr::null(),
                );
            }
        };
        let line_box = unsafe { &mut *items };
        let mut i = self.fragment_start;
        while i < self.fragment_end {
            let child = &mut line_box[i as usize];
            if child.children_count != 0 {
                i += child.children_count - 1;
            }
            handle_box_child(child);
            i += 1;
        }
        let ruby_columns = self.ruby_column_list.Get();
        if !ruby_columns.is_null() {
            for column_ptr in unsafe { &*ruby_columns } {
                let column = unsafe { &mut *column_ptr.Get() };
                let annotation_items = unsafe { &mut *column.annotation_items.Get() };
                if annotation_items.WasPropagated() {
                    continue;
                }
                let mut i = 0;
                while i < annotation_items.size() {
                    let child = &mut annotation_items[i as usize];
                    if child.children_count != 0 {
                        i += child.children_count - 1;
                    }
                    handle_box_child(child);
                    i += 1;
                }
                annotation_items.SetPropagated();
            }
            self.ruby_column_list.Clear();
        }
        unsafe { &mut *item.GetLayoutObject() }
            .SetShouldDoFullPaintInvalidationWithReason(PaintInvalidationReason::kLayout);
        box_builder.MoveOutOfFlowDescendantCandidatesToDescendants();
        box_builder.ToInlineBoxFragment()
    }
    pub fn Trace(&self, visitor: &mut Visitor) {
        // cpp: layoutng_inline/inline_box_state.cc:1139-1142
        visitor.Trace(&self.ruby_column_list);
        visitor.Trace(&self.item);
    }
}

// cpp: layoutng_inline/inline_box_state.h:395-429
pub struct LogicalRubyColumn {
    pub start_index: u32,
    pub size: u32,
    pub base_insets: (LayoutUnit, LayoutUnit),
    pub annotation_items: Member<LogicalLineItems>,
    pub annotation_metrics: FontHeight,
    pub layout_annotation_metrics: FontHeight,
    pub ruby_position: RubyPosition,
    pub state_stack: InlineLayoutStateStack,
}

impl Default for LogicalRubyColumn {
    fn default() -> Self {
        Self {
            start_index: 0,
            size: 0,
            base_insets: (LayoutUnit::default(), LayoutUnit::default()),
            annotation_items: Member::default(),
            annotation_metrics: FontHeight::default(),
            layout_annotation_metrics: FontHeight::default(),
            ruby_position: RubyPosition::kOver,
            state_stack: InlineLayoutStateStack::default(),
        }
    }
}

impl LogicalRubyColumn {
    // cpp: layoutng_inline/inline_box_state.h:423-428
    pub fn Trace(&self, visitor: &mut Visitor) {
        // cpp: layoutng_inline/inline_box_state.cc:33-36
        visitor.Trace(&self.annotation_items);
        visitor.Trace(&self.state_stack);
    }
    pub fn EndIndex(&self) -> u32 {
        self.start_index + self.size
    }
    pub fn RubyColumnList(&mut self) -> &mut HeapVector<Member<LogicalRubyColumn>> {
        self.state_stack.RubyColumnList()
    }
}

impl Traceable for InlineBoxState {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        InlineBoxState::Trace(self, visitor);
    }
}

impl Traceable for InlineLayoutStateStack {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        InlineLayoutStateStack::Trace(self, visitor);
    }
}

impl Traceable for BoxData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        BoxData::Trace(self, visitor);
    }
}

impl Traceable for LogicalRubyColumn {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LogicalRubyColumn::Trace(self, visitor);
    }
}

// cpp: layoutng_inline/inline_box_state.h:433-435
// HeapVector owns its slots in Rust and drops moved-out values normally.

// These declarations are implemented by inline_box_state.cc in this crate.
unsafe extern "Rust" {
    // Owned by font_engine/fonts/font_services.cc; that package must provide it.
    fn FontEmphasisMarkHeight(font: &Font, mark: &AtomicString) -> LayoutUnit;
}
