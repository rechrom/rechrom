// C++: layoutng_inline/inline_layout_algorithm.h/.cc. Source bodies are
// mapped; same-package callees and the shared inline assembly remain pending.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::harfbuzz_shaper::HarfBuzzShaper;
use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::text::native::character::Character;
use font_engine::{FontBaseline, FontHeight, ShapeResult};
use foundation::style_constants::{EBlockEllipsis, EFloat, ETextAlign, TextWrapStyle};
use foundation::{
    kIndefiniteSize, ClearCollectionScope, DynamicTo, IsFlippedLinesWritingMode, IsLtr, LayoutUnit,
    MarginStrut, Member, RuntimeEnabledFeatures, String, TextDirection, To, WritingDirectionMode,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::{AdjoiningObjectTypeValue, ConstraintSpace};
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::exclusions::layout_opportunity::LayoutOpportunity;
use layoutng::internal::exclusions::line_layout_opportunity::LineLayoutOpportunity;
use layoutng::internal::form_control_types::FormControlType;
use layoutng::internal::inline_item::{CollapseType, InlineItem, InlineItemType, InlineItems};
use layoutng::internal::inline_item_result::InlineItemResult;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_algorithm::LayoutAlgorithm;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng::internal::layout_node_metadata::Element;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::length_utils::{
    ComputeLineBorders, ComputeLineMarginsForSelf, ComputeLinePadding, LineOffsetForTextAlign,
};
use layoutng::internal::min_max_sizes::MinMaxSizesResult;
use layoutng::internal::positioned_float::PositionedFloat;
use layoutng::internal::space_utils::AdjustToClearance;
use layoutng::internal::unpositioned_float::UnpositionedFloat;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::layout_result::{EStatus, LayoutResult};
use layoutng_fragment_tree::logical_box_fragment::LogicalBoxFragment;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::logical_line_container::LogicalLineContainer;
use layoutng_fragment_tree::logical_line_item::{LogicalLineItem, LogicalLineItems};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_strut::{BoxStrut, LineBoxStrut};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_style::css::white_space::ShouldWrapLineGreedy;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::inline_box_state::{InlineBoxState, InlineLayoutStateStack};
use crate::inline_child_layout_context::InlineChildLayoutContext;
use crate::justification_utils::{ApplyJustification, JustificationTarget};
use crate::leading_floats::LeadingFloats;
use crate::line_box_fragment_builder::LineBoxFragmentBuilder;
use crate::line_breaker::LineBreaker;
use crate::line_info::LineInfo;
use crate::line_utils::ComputeRelativeOffsetForInline;
use crate::line_widths::LineWidths;
use crate::logical_line_builder::LogicalLineBuilder;
use crate::ruby_utils::{
    ComputeAnnotationOverflow, RubyBlockPositionCalculator, SetTextEmphasisAnnotationMetrics,
    UpdateRubyColumnInlinePositions,
};
use crate::score_line_break_context::ScoreLineBreakContext;
use crate::text_fit_utils::{LineFitter, ShouldApplyTextFit};

// C++ keeps a nullable pointer to a context owned by InlineChildLayoutContext.
// Option<*mut dyn Trait> retains that relation without a fake fat-pointer null.
// cpp: layoutng_inline/inline_layout_algorithm.cc:55-215
struct LineBreakStrategy {
    initiate_balancing_: bool,
    use_score_line_break_: bool,
    score_line_break_context_: Option<*mut dyn ScoreLineBreakContext>,
}

// The strategy and its context are both owned by the same Layout invocation.
// The pointer is never retained after that invocation; erasing the borrow's
// lifetime models the source's non-owning ScoreLineBreakContext* field.
fn score_context_pointer(
    context: &mut dyn ScoreLineBreakContext,
) -> *mut dyn ScoreLineBreakContext {
    unsafe {
        std::mem::transmute::<*mut (dyn ScoreLineBreakContext + '_), *mut dyn ScoreLineBreakContext>(
            context as *mut dyn ScoreLineBreakContext,
        )
    }
}

impl LineBreakStrategy {
    // cpp: layoutng_inline/inline_layout_algorithm.cc:62-99
    fn new(
        context: &mut InlineChildLayoutContext,
        node: &InlineNode,
        block_style: &ComputedStyle,
        break_token: *const InlineBreakToken,
        column_spanner_path: *const ColumnSpannerPath,
    ) -> Self {
        let mut strategy = Self {
            initiate_balancing_: false,
            use_score_line_break_: false,
            score_line_break_context_: None,
        };
        if column_spanner_path.is_null() {
            let text_wrap = block_style.GetTextWrapStyle();
            if text_wrap == TextWrapStyle::kBalance {
                strategy.score_line_break_context_ = context
                    .GetScoreLineBreakContext()
                    .map(score_context_pointer);
                strategy.initiate_balancing_ = break_token.is_null();
                if strategy.initiate_balancing_ {
                    debug_assert!(strategy
                        .score_context()
                        .is_none_or(|score| score.IsActive()));
                    strategy.use_score_line_break_ = strategy.score_line_break_context_.is_some();
                }
            } else if text_wrap == TextWrapStyle::kPretty {
                strategy.score_line_break_context_ = context
                    .GetScoreLineBreakContext()
                    .map(score_context_pointer);
                strategy.use_score_line_break_ = strategy
                    .score_context()
                    .is_some_and(|score| score.IsActive());
            } else {
                debug_assert!(ShouldWrapLineGreedy(text_wrap));
            }
        }
        #[cfg(feature = "expensive_dchecks")]
        {
            debug_assert!(
                context.GetScoreLineBreakContext().is_none() || !node.IsScoreLineBreakDisabled()
            );
            debug_assert!(!strategy.use_score_line_break_ || !node.IsScoreLineBreakDisabled());
        }
        strategy
    }

    fn score_context(&self) -> Option<&dyn ScoreLineBreakContext> {
        self.score_line_break_context_.map(|ptr| unsafe { &*ptr })
    }

    fn score_context_mut(&mut self) -> Option<&mut dyn ScoreLineBreakContext> {
        self.score_line_break_context_
            .map(|ptr| unsafe { &mut *ptr })
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:101-103
    fn NeedsToPrepare(&self) -> bool {
        self.initiate_balancing_ || self.use_score_line_break_
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:105-121
    fn Prepare(
        &mut self,
        context: &mut InlineChildLayoutContext,
        node: &InlineNode,
        space: &ConstraintSpace,
        opportunities: &[LayoutOpportunity],
        line_opportunity: &LineLayoutOpportunity,
        leading_floats: &LeadingFloats,
        break_token: *const InlineBreakToken,
        exclusion_space: &mut ExclusionSpace,
    ) {
        if self.initiate_balancing_ {
            self.Balance(
                context,
                node,
                space,
                opportunities,
                line_opportunity,
                leading_floats,
                break_token,
                exclusion_space,
            );
        } else if self.use_score_line_break_ {
            self.Optimize(
                node,
                space,
                opportunities,
                leading_floats,
                break_token,
                exclusion_space,
            );
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:123-138
    fn SetupLineBreaker(
        &mut self,
        context: &InlineChildLayoutContext,
        line_breaker: &mut LineBreaker,
    ) {
        if let Some(width) = *context.BalancedAvailableWidth() {
            debug_assert!(self
                .score_context()
                .is_none_or(|score| { score.CurrentLineBreakPoint().is_null() }));
            line_breaker.OverrideAvailableWidth(width);
        } else if let Some(score) = self.score_context() {
            let point = score.CurrentLineBreakPoint();
            if !point.is_null() {
                line_breaker.SetBreakAt(unsafe { &*point });
            }
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:140-144
    fn DidCreateLine(&mut self, is_end_paragraph: bool) {
        if let Some(score) = self.score_context_mut() {
            score.DidCreateLine(is_end_paragraph);
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:147-188
    fn Balance(
        &mut self,
        context: &mut InlineChildLayoutContext,
        node: &InlineNode,
        space: &ConstraintSpace,
        opportunities: &[LayoutOpportunity],
        line_opportunity: &LineLayoutOpportunity,
        leading_floats: &LeadingFloats,
        break_token: *const InlineBreakToken,
        exclusion_space: &mut ExclusionSpace,
    ) {
        debug_assert!(context.BalancedAvailableWidth().is_none());
        debug_assert!(!opportunities.is_empty());
        debug_assert!(!opportunities.last().unwrap().HasShapeExclusions());
        if self.use_score_line_break_ && self.score_context().is_some_and(|score| score.IsActive())
        {
            let score = self.score_context_mut().unwrap();
            debug_assert!(score.GetLineBreakPoints().is_empty());
            debug_assert_eq!(score.LineBreakPointsIndex(), 0);
            let mut line_widths = LineWidths::new();
            if line_widths.Set(node, opportunities, std::ptr::null()) {
                let mut optimizer = crate::score_line_breaker::ScoreLineBreaker::new(
                    node,
                    space,
                    &line_widths,
                    break_token,
                    exclusion_space,
                );
                optimizer.BalanceBreakPoints(leading_floats, score);
                if !score.GetLineBreakPoints().is_empty() {
                    return;
                }
            }
        }
        if opportunities.len() == 1
            && line_opportunity.AvailableInlineSize() > LayoutUnit::default()
        {
            if let Some(width) =
                crate::paragraph_line_breaker::ParagraphLineBreaker::AttemptParagraphBalancing(
                    node,
                    space,
                    line_opportunity,
                )
            {
                context.SetBalancedAvailableWidth(Some(width));
                if let Some(score) = self.score_context_mut() {
                    score.GetLineInfoList().Clear();
                }
            }
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:190-211
    fn Optimize(
        &mut self,
        node: &InlineNode,
        space: &ConstraintSpace,
        opportunities: &[LayoutOpportunity],
        leading_floats: &LeadingFloats,
        break_token: *const InlineBreakToken,
        exclusion_space: &mut ExclusionSpace,
    ) {
        let score = self.score_context_mut().unwrap();
        debug_assert!(score.GetLineBreakPoints().is_empty());
        debug_assert_eq!(score.LineBreakPointsIndex(), 0);
        if !score.IsActive() {
            return;
        }
        let mut line_widths = LineWidths::new();
        if !line_widths.Set(node, opportunities, break_token) {
            return;
        }
        let mut optimizer = crate::score_line_breaker::ScoreLineBreaker::new(
            node,
            space,
            &line_widths,
            break_token,
            exclusion_space,
        );
        optimizer.OptimalBreakPoints(leading_floats, score);
        if score.IsActive() {
            return;
        }
    }
}

// cpp: layoutng_inline/inline_layout_algorithm.cc:217-227
fn PlaceRelativePositionedItems(space: &ConstraintSpace, line_box: &mut LogicalLineItems) {
    for child in line_box.iter_mut() {
        let physical_fragment = child.GetPhysicalFragment();
        if physical_fragment.is_null() {
            continue;
        }
        child.rect.offset += ComputeRelativeOffsetForInline(space, unsafe { &*child.Style() });
    }
}

// cpp: layoutng_inline/inline_layout_algorithm.cc:295-303
fn AdjustLineOffsetForHanging(line_info: &LineInfo) -> LayoutUnit {
    if IsLtr(line_info.BaseDirection()) {
        return LayoutUnit::default();
    }
    -line_info.HangWidth()
}

// cpp: layoutng_inline/inline_layout_algorithm.h:73-80
pub struct LineClampEllipsis {
    pub text: String,
    pub shape_result: *const ShapeResult,
    pub text_metrics: FontHeight,
}

// cpp: layoutng_inline/inline_layout_algorithm.h:126-131
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineClampState {
    kShow,
    kLineClampEllipsis,
    kTextOverflowEllipsis,
    kHide,
}

// cpp: layoutng_inline/inline_layout_algorithm.h:43-170
#[repr(C)]
pub struct InlineLayoutAlgorithm {
    base_: LayoutAlgorithm<InlineNode, LineBoxFragmentBuilder, InlineBreakToken>,
    box_states_: *mut InlineLayoutStateStack,
    context_: *mut InlineChildLayoutContext,
    column_spanner_path_: *const ColumnSpannerPath,
    end_margin_strut_: MarginStrut,
    lines_until_clamp_: Option<i32>,
    line_clamp_ellipsis_: Option<LineClampEllipsis>,
    baseline_type_: FontBaseline,
    quirks_mode_: bool,
    apply_text_fit_: bool,
    #[cfg(feature = "expensive_dchecks")]
    is_box_states_from_context_: bool,
}

impl InlineLayoutAlgorithm {
    // cpp: layoutng_inline/inline_layout_algorithm.cc:231-250
    pub fn new(
        inline_node: InlineNode,
        space: &ConstraintSpace,
        break_token: *const InlineBreakToken,
        column_spanner_path: *const ColumnSpannerPath,
        context: *mut InlineChildLayoutContext,
    ) -> Self {
        let style = inline_node.Style() as *const _;
        let baseline_type = inline_node.Style().GetFontBaseline();
        let quirks_mode = unsafe { &*inline_node.GetLayoutBox() }.InLineHeightQuirksModeForLayout();
        let builder = LineBoxFragmentBuilder::new(
            inline_node.clone(),
            style,
            space,
            WritingDirectionMode::new(space.GetWritingMode(), TextDirection::kLtr),
            break_token,
        );
        let result = Self {
            base_: LayoutAlgorithm::new_with_builder(inline_node, builder),
            box_states_: std::ptr::null_mut(),
            context_: context,
            column_spanner_path_: column_spanner_path,
            end_margin_strut_: MarginStrut::default(),
            lines_until_clamp_: None,
            line_clamp_ellipsis_: None,
            baseline_type_: baseline_type,
            quirks_mode_: quirks_mode,
            apply_text_fit_: false,
            #[cfg(feature = "expensive_dchecks")]
            is_box_states_from_context_: false,
        };
        debug_assert!(!context.is_null());
        result
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:62-64
    pub fn ComputeMinMaxSizes(&self, _input: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        unreachable!("InlineLayoutAlgorithm::ComputeMinMaxSizes is NOTREACHED in the source")
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:81-83
    pub fn GetLineClampEllipsis(&self) -> &Option<LineClampEllipsis> {
        &self.line_clamp_ellipsis_
    }

    // cpp: layoutng_inline/inline_layout_algorithm.cc:328-351
    fn GetLineClampState(&self, line_info: Option<&LineInfo>) -> LineClampState {
        let space = self.base_.container_builder_.GetConstraintSpace();
        let line_clamp_data = space.GetLineClampData();
        if line_clamp_data.ShouldHideForPaint() {
            return LineClampState::kHide;
        }
        if !line_info.is_some_and(LineInfo::IsBlockInInline) && line_clamp_data.IsAtClampPoint() {
            let block_ellipsis = if RuntimeEnabledFeatures::CSSLineClampAsShorthandEnabled() {
                self.base_.node_.Style().BlockEllipsis()
            } else {
                line_clamp_data.block_ellipsis
            };
            if block_ellipsis == EBlockEllipsis::kEllipsis {
                return LineClampState::kLineClampEllipsis;
            }
        }
        if let Some(line_info) = line_info {
            if !line_info.IsBlockInInline()
                && line_info.HasOverflow()
                && unsafe { &*self.base_.node_.GetLayoutBlockFlow() }
                    .ShouldTruncateOverflowingText()
            {
                return LineClampState::kTextOverflowEllipsis;
            }
        }
        LineClampState::kShow
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:95-97
    // cpp: layoutng_inline/inline_layout_algorithm.cc:257-292
    fn PrepareBoxStates(
        &mut self,
        line_info: &LineInfo,
        should_scale_line_height: bool,
        break_token: *const InlineBreakToken,
    ) {
        #[cfg(feature = "expensive_dchecks")]
        {
            self.is_box_states_from_context_ = false;
        }
        let context = unsafe { &mut *self.context_ };
        if break_token.is_null() {
            self.box_states_ = context.ResetBoxStates();
            return;
        }

        let items = &line_info.ItemsData().items;
        let break_token_ref = unsafe { &*break_token };
        if !break_token_ref.UseFirstLineStyle()
            && !break_token_ref.IsLineClampDisplacedLine()
            && !self.apply_text_fit_
        {
            self.box_states_ =
                context.BoxStatesIfValidForItemIndex(items, break_token_ref.StartItemIndex());
            if !self.box_states_.is_null() {
                #[cfg(feature = "expensive_dchecks")]
                {
                    self.is_box_states_from_context_ = true;
                }
                return;
            }
        }

        self.box_states_ = context.ResetBoxStates();
        LogicalLineBuilder::new(
            self.base_.node_.clone(),
            self.base_.container_builder_.GetConstraintSpace(),
            std::ptr::null(),
            self.box_states_,
            self.context_,
            should_scale_line_height,
        )
        .RebuildBoxStates(line_info, 0, break_token_ref.StartItemIndex());
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:66-68
    // cpp: layoutng_inline/inline_layout_algorithm.cc:306-326
    #[cfg(feature = "expensive_dchecks")]
    pub fn CheckBoxStates(&self, line_info: &LineInfo, should_scale_line_height: bool) {
        if !self.is_box_states_from_context_ {
            return;
        }
        let mut rebuilt = InlineLayoutStateStack::default();
        LogicalLineBuilder::new(
            self.base_.node_.clone(),
            self.base_.container_builder_.GetConstraintSpace(),
            std::ptr::null(),
            &mut rebuilt,
            self.context_,
            should_scale_line_height,
        )
        .RebuildBoxStates(
            line_info,
            0,
            unsafe { &*self.base_.GetBreakToken() }.StartItemIndex(),
        );
        let line_box = unsafe { &mut *self.context_ }.AcquireTempLogicalLineItems();
        let is_only_line_clamp_ellipsis =
            self.line_clamp_ellipsis_.is_some() && line_info.Results().is_empty();
        rebuilt.OnBeginPlaceItems(
            &self.base_.node_,
            line_info,
            self.baseline_type_,
            self.quirks_mode_ || is_only_line_clamp_ellipsis,
            should_scale_line_height,
            line_box,
        );
        debug_assert!(!self.box_states_.is_null());
        unsafe { &*self.box_states_ }.CheckSame(&rebuilt);
        unsafe { &mut *self.context_ }.ReleaseTempLogicalLineItems(line_box);
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:108
    // cpp: layoutng_inline/inline_layout_algorithm.cc:942-969
    fn ApplyTextAlign(&mut self, line_info: &mut LineInfo) -> LayoutUnit {
        let space = line_info.AvailableWidth() - line_info.WidthForAlignment();
        let mut text_align = line_info.TextAlign();
        if text_align == ETextAlign::kJustify {
            let target = if self.base_.node_.IsSvgText() {
                JustificationTarget::kSvgText
            } else if line_info.IsRubyBase() {
                JustificationTarget::kRubyBase
            } else if line_info.IsRubyText() {
                JustificationTarget::kRubyText
            } else {
                JustificationTarget::kNormal
            };
            if let Some(offset) = ApplyJustification(space, target, line_info) {
                return offset;
            }
            text_align = ETextAlign::kStart;
        }
        LineOffsetForTextAlign(text_align, line_info.BaseDirection(), space)
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:55-58
    // cpp: layoutng_inline/inline_layout_algorithm.cc:353-627
    pub fn CreateLine(
        &mut self,
        opportunity: &LineLayoutOpportunity,
        line_info: &mut LineInfo,
        should_scale_line_height: bool,
        line_container: &mut LogicalLineContainer,
    ) {
        let line_box = unsafe { &mut *line_container.BaseLine() };
        let line_offset_for_text_align = self.ApplyTextAlign(line_info);
        line_container.Shrink();

        let mut line_builder = LogicalLineBuilder::new(
            self.base_.node_.clone(),
            self.base_.container_builder_.GetConstraintSpace(),
            self.base_.GetBreakToken(),
            self.box_states_,
            self.context_,
            should_scale_line_height,
        );
        line_builder.CreateLine(line_info, line_box, self);

        let hang_width = line_info.HangWidth();
        let position = AdjustLineOffsetForHanging(line_info);
        let mut inline_size = unsafe { &mut *self.box_states_ }.ComputeInlinePositions(
            line_box,
            position,
            line_info.IsBlockInInline(),
        );
        if hang_width != LayoutUnit::default() {
            if position == LayoutUnit::default() {
                inline_size -= hang_width;
            }
            self.base_.container_builder_.SetHangInlineSize(hang_width);
        }

        let box_states = unsafe { &mut *self.box_states_ };
        if line_info.HasLineEvenIfEmpty() || !box_states.RubyColumnList().is_empty() {
            let line_box_state = box_states.LineBoxState();
            line_box_state.EnsureTextMetrics(
                line_info.LineStyle(),
                unsafe { &*line_box_state.font.Get() },
                self.baseline_type_,
                std::ptr::null(),
            );
        } else if !line_builder.InitialLetterItemResult().is_null()
            && box_states.LineBoxState().metrics.IsEmpty()
        {
            box_states.LineBoxState().metrics = FontHeight::default();
        }
        let line_box_metrics = box_states.LineBoxState().metrics;

        let has_text_emphasis = RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled()
            && self.base_.node_.HasTextEmphasis();
        if (self.base_.node_.HasRuby() || has_text_emphasis) && !line_info.IsEmptyLine() {
            let mut annotation_metrics = None;
            let column_list = unsafe { &mut *self.box_states_ }.RubyColumnList();
            if !column_list.is_empty() {
                UpdateRubyColumnInlinePositions(line_box, inline_size, column_list);
                let mut calculator = RubyBlockPositionCalculator::new();
                calculator
                    .GroupLines(column_list)
                    .PlaceLines(line_box, &line_box_metrics)
                    .AddLinesTo(line_container);
                annotation_metrics = Some(calculator.AnnotationMetrics());
                if RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled() {
                    calculator.UpdateColumnLayoutAnnotationMetrics(column_list);
                } else if RuntimeEnabledFeatures::TextEmphasisWithRubyEnabled() {
                    for column in column_list.iter() {
                        let column = unsafe { &*column.Get() };
                        for index in 0..column.size {
                            line_box[(column.start_index + index) as usize].annotation_metrics =
                                column.annotation_metrics;
                        }
                    }
                }
            }
            if RuntimeEnabledFeatures::TextEmphasisAsRubyEnabled() {
                SetTextEmphasisAnnotationMetrics(column_list, line_box);
            }
            let adjustment = self.SetAnnotationOverflow(
                line_info,
                line_box,
                &line_box_metrics,
                annotation_metrics,
            );
            line_info.SetAnnotationBlockStartAdjustment(adjustment);
        } else if RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
            && self
                .base_
                .container_builder_
                .GetConstraintSpace()
                .ContainsAnnotations()
            && line_info.IsLastLine()
            && !line_info.IsEmptyLine()
        {
            let adjustment =
                self.SetAnnotationOverflow(line_info, line_box, &line_box_metrics, None);
            line_info.SetAnnotationBlockStartAdjustment(adjustment);
        }

        let line_clamp_state = self.GetLineClampState(Some(line_info));
        if line_clamp_state == LineClampState::kTextOverflowEllipsis
            || (line_clamp_state == LineClampState::kLineClampEllipsis
                && !RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled())
        {
            debug_assert!(!line_info.IsBlockInInline());
            let mut truncator = crate::line_truncator::LineTruncator::new(
                line_info,
                line_clamp_state == LineClampState::kLineClampEllipsis,
            );
            let node = unsafe { &*self.base_.node_.GetLayoutBlockFlow() }.GetNode();
            let element = DynamicTo::<Element>(node);
            let element_data = if element.is_null() {
                None
            } else {
                unsafe { &*element }.InputElementData().as_ref()
            };
            if element_data.is_some_and(|data| {
                data.form_control_type == Some(FormControlType::kInputFile)
                    && data.selected_file_count <= 1
            }) {
                inline_size = truncator.TruncateLineInTheMiddle(inline_size, line_box, unsafe {
                    &mut *self.box_states_
                });
            } else {
                inline_size = truncator
                    .TruncateLine(inline_size, line_box, unsafe { &mut *self.box_states_ });
            }
        }

        if line_clamp_state == LineClampState::kHide {
            self.base_.container_builder_.SetIsHiddenForPaint(true);
            for child in line_box.iter_mut() {
                child.is_hidden_for_paint = true;
            }
        }
        inline_size = inline_size.ClampNegativeToZero();

        if line_info.IsBlockInInline() {
            let bfc_line_offset = self
                .base_
                .container_builder_
                .GetConstraintSpace()
                .GetBfcOffset()
                .line_offset;
            self.base_
                .container_builder_
                .SetBfcLineOffset(bfc_line_offset);
        } else {
            let mut bfc_line_offset =
                line_info.GetBfcOffset().line_offset + line_offset_for_text_align;
            if IsLtr(line_info.BaseDirection()) {
                bfc_line_offset += line_info.TextIndent();
            }
            self.base_
                .container_builder_
                .SetBfcLineOffset(bfc_line_offset);
        }

        let initial_letter_item = line_builder.InitialLetterItemResult();
        if !initial_letter_item.is_null() {
            debug_assert!(!line_info.IsEmptyLine());
            let margins = BoxStrut::from_line(
                &unsafe { &*initial_letter_item }.margins,
                line_info.LineStyle().IsFlippedLinesWritingMode(),
            );
            let line_origin = BfcOffset::new(
                self.base_.container_builder_.BfcLineOffset(),
                line_info.GetBfcOffset().block_offset,
            );
            let exclusion = crate::initial_letter_utils::PostPlaceInitialLetterBox(
                &line_box_metrics,
                &margins,
                line_box,
                &line_origin,
                line_info,
            );
            self.base_.GetExclusionSpace().Add(exclusion);
        }

        if line_builder.HasOutOfFlowPositionedItems() {
            debug_assert!(!line_info.IsBlockInInline());
            self.PlaceOutOfFlowObjects(line_info, &line_box_metrics, line_box);
        }
        if line_builder.HasFloatingItems() {
            debug_assert!(!line_info.IsBlockInInline());
            self.PlaceFloatingObjects(
                &line_box_metrics,
                opportunity,
                line_info.ComputeBlockStartAdjustment(),
                line_info,
                line_box,
            );
        }
        if line_builder.HasRelativePositionedItems() {
            PlaceRelativePositionedItems(
                self.base_.container_builder_.GetConstraintSpace(),
                line_box,
            );
        }
        for annotation_line in line_container.AnnotationLineList() {
            PlaceRelativePositionedItems(
                self.base_.container_builder_.GetConstraintSpace(),
                unsafe { &mut *annotation_line.get() },
            );
        }

        unsafe { &mut *self.box_states_ }.ApplyRelativePositioning(
            self.base_.container_builder_.GetConstraintSpace(),
            line_box,
            std::ptr::null(),
        );
        unsafe { &mut *self.box_states_ }.CreateBoxFragments(
            self.base_.container_builder_.GetConstraintSpace(),
            line_box,
            line_box_metrics.LineHeight(),
            line_info.IsBlockInInline(),
        );
        unsafe { &mut *self.box_states_ }.ClearRubyColumnList();
        unsafe { &mut *self.context_ }
            .SetItemIndex(&line_info.ItemsData().items, line_info.EndItemIndex());

        if line_info.UseFirstLineStyle() {
            self.base_
                .container_builder_
                .SetStyleVariant(layoutng::internal::style_variant::StyleVariant::kFirstLine);
        }
        if line_info.IsEmptyLine() {
            return;
        }
        if !line_box_metrics.IsEmpty() {
            self.base_.container_builder_.SetMetrics(&line_box_metrics);
        }

        let space = self.base_.container_builder_.GetConstraintSpace();
        if space.ShouldTextBoxTrimNodeStart()
            || space.ShouldTextBoxTrimNodeEnd()
            || space.ShouldTextBoxTrimFragmentainerStart()
            || space.ShouldTextBoxTrimFragmentainerEnd()
            || space.ShouldTextBoxTrimInsideWhenLineClamp()
        {
            let line_clamp_data = space.GetLineClampData();
            let is_truncated =
                line_clamp_data.IsAtClampPoint() || line_clamp_data.IsMeasureUntilBfcOffset();
            self.ApplyTextBoxTrim(line_info, is_truncated);
        }
        if line_info.IsBlockInInline() {
            return;
        }

        if self.base_.node_.IsTextCombine() {
            let one_em = self.base_.node_.Style().ComputedFontSizeAsFixedValue();
            inline_size = inline_size.min(one_em);
        } else if self.base_.node_.IsInitialLetterBox() {
            let adjusted_metrics = crate::initial_letter_utils::AdjustInitialLetterInTextPosition(
                &line_box_metrics,
                line_box,
            );
            if !adjusted_metrics.IsEmpty() {
                self.base_.container_builder_.SetMetrics(&adjusted_metrics);
                line_container.MoveInBlockDirection(adjusted_metrics.ascent);
            }
        } else if !self.base_.node_.IsSvgText() {
            line_container.MoveInBlockDirection(line_box_metrics.ascent);
        }
        line_container.SetTextFitScale(line_info.TextFitScale());
        self.base_.container_builder_.SetInlineSize(inline_size);
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:118-122
    // cpp: layoutng_inline/inline_layout_algorithm.cc:971-1023
    fn SetAnnotationOverflow(
        &mut self,
        line_info: &LineInfo,
        line_box: &LogicalLineItems,
        line_box_metrics: &FontHeight,
        annotation_font_height: Option<FontHeight>,
    ) -> LayoutUnit {
        let annotation_metrics = ComputeAnnotationOverflow(
            line_box,
            line_box_metrics,
            LayoutUnit::FromFloatRound(
                line_info.LineStyle().ComputedFontSize() * line_info.TextFitScale(),
            ),
            annotation_font_height,
        );
        let (
            annotation_overflow_block_start,
            annotation_overflow_block_end,
            annotation_space_block_start,
            annotation_space_block_end,
        ) = if !IsFlippedLinesWritingMode(line_info.LineStyle().GetWritingMode()) {
            (
                annotation_metrics.overflow_over,
                annotation_metrics.overflow_under,
                annotation_metrics.space_over,
                annotation_metrics.space_under,
            )
        } else {
            (
                annotation_metrics.overflow_under,
                annotation_metrics.overflow_over,
                annotation_metrics.space_under,
                annotation_metrics.space_over,
            )
        };

        let block_start_annotation_space = self
            .base_
            .container_builder_
            .GetConstraintSpace()
            .BlockStartAnnotationSpace();
        let mut block_offset_shift = annotation_overflow_block_start;
        if block_start_annotation_space < LayoutUnit::default()
            && annotation_space_block_start != LayoutUnit::default()
        {
            let overflow = -block_start_annotation_space;
            block_offset_shift = -std::cmp::min(annotation_space_block_start, overflow);
        }
        if annotation_overflow_block_start != LayoutUnit::default()
            && block_start_annotation_space > LayoutUnit::default()
        {
            block_offset_shift = (annotation_overflow_block_start - block_start_annotation_space)
                .ClampNegativeToZero();
        }

        if annotation_overflow_block_end != LayoutUnit::default() {
            self.base_
                .container_builder_
                .SetAnnotationOverflow(annotation_overflow_block_end);
        } else if annotation_space_block_end != LayoutUnit::default() {
            self.base_
                .container_builder_
                .SetBlockEndAnnotationSpace(annotation_space_block_end);
        }
        block_offset_shift
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:110
    // cpp: layoutng_inline/inline_layout_algorithm.cc:629-718
    fn ApplyTextBoxTrim(&mut self, line_info: &mut LineInfo, is_truncated: bool) {
        let space = self.base_.container_builder_.GetConstraintSpace();
        if !line_info.BlockInInlineLayoutResult().is_null() {
            return;
        }
        let should_apply_start = (space.ShouldTextBoxTrimNodeStart()
            && line_info.IsFirstFormattedLine())
            || space.ShouldTextBoxTrimFragmentainerStart();
        let should_apply_end = (space.ShouldTextBoxTrimNodeEnd()
            && line_info.GetBreakToken().is_null())
            || (space.ShouldTextBoxTrimInsideWhenLineClamp() && is_truncated)
            || space.ShouldForceTextBoxTrimEnd();
        if !should_apply_start && !should_apply_end {
            return;
        }

        let line_style = line_info.LineStyle();
        let is_flipped_line = line_style.IsFlippedLinesWritingMode();
        let mut should_apply_over = should_apply_start;
        let mut should_apply_under = should_apply_end;
        if is_flipped_line {
            should_apply_over = should_apply_end;
            should_apply_under = should_apply_start;
        }

        let line_box_metrics = *self.base_.container_builder_.Metrics();
        let mut intrinsic_metrics = line_box_metrics;
        InlineBoxState::AdjustEdges(
            line_style,
            unsafe { &*line_style.GetFont() },
            self.baseline_type_,
            should_apply_over,
            should_apply_under,
            &mut intrinsic_metrics,
        );
        if RuntimeEnabledFeatures::CssTextFitEnabled() && self.apply_text_fit_ {
            let mut scale = line_info.TextFitScale();
            if scale < 1.0 {
                if let Some(min_size) = self.base_.node_.MinimumFontPhysicalSize() {
                    let original_size = unsafe { &*line_style.GetFont() }
                        .GetFontDescription()
                        .ComputedSize();
                    if original_size * scale < min_size {
                        scale = min_size / original_size;
                    }
                }
            }
            if scale != 1.0 {
                intrinsic_metrics.ascent *= scale;
                intrinsic_metrics.descent *= scale;
            }
        }

        if should_apply_start {
            let offset_for_trimming_box = if is_flipped_line {
                intrinsic_metrics.descent - line_box_metrics.descent
            } else {
                intrinsic_metrics.ascent - line_box_metrics.ascent
            };
            let offset = self
                .base_
                .container_builder_
                .LineBoxBfcBlockOffset()
                .map_or(offset_for_trimming_box, |previous| {
                    offset_for_trimming_box + previous
                });
            self.base_
                .container_builder_
                .SetLineBoxBfcBlockOffset(offset);
            line_info.SetAnnotationBlockStartAdjustment(LayoutUnit::default());
            line_info.SetInitialLetterBlockStartAdjustment(LayoutUnit::default());
        }
        if should_apply_end {
            self.base_.container_builder_.SetIsBlockEndTrimmableLine();
            let block_end_to_be_trimmed = if is_flipped_line {
                line_box_metrics.ascent - intrinsic_metrics.ascent
            } else {
                line_box_metrics.descent - intrinsic_metrics.descent
            };
            self.base_
                .container_builder_
                .SetTrimBlockEndBy(block_end_to_be_trimmed);
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:69-71
    // cpp: layoutng_inline/inline_layout_algorithm.cc:720-761
    pub fn PlaceBlockInInline(
        &mut self,
        item: &InlineItem,
        item_result: &mut InlineItemResult,
        line_box: &mut LogicalLineItems,
    ) {
        debug_assert_eq!(item.Type(), InlineItemType::kBlockInInline);
        let layout_object = item.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        debug_assert!(unsafe { &*layout_object }.IsAnonymous());
        debug_assert!(!unsafe { &*layout_object }.IsInline());
        debug_assert!(!item_result.layout_result.Get().is_null());
        let result = unsafe { &*item_result.layout_result.Get() };
        let box_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
        let fragment = LogicalBoxFragment::new(
            self.base_
                .container_builder_
                .GetConstraintSpace()
                .GetWritingDirection(),
            box_fragment,
        );

        self.base_.container_builder_.SetIsBlockInInline();
        self.base_
            .container_builder_
            .SetInlineSize(fragment.InlineSize());
        self.base_
            .container_builder_
            .ClampBreakAppeal(result.GetBreakAppeal());
        if !result.IsSelfCollapsing() {
            let metrics = fragment.BaselineMetrics(&LineBoxStrut::default(), self.baseline_type_);
            unsafe { &mut *self.box_states_ }.OnBlockInInline(&metrics, line_box);
        }

        self.end_margin_strut_ = result.EndMarginStrut();
        self.base_
            .container_builder_
            .SetExclusionSpace(result.GetExclusionSpace());
        self.base_
            .container_builder_
            .SetAdjoiningObjectTypes(result.GetAdjoiningObjectTypes());
        self.lines_until_clamp_ = Some(result.LinesUntilClamp());
        if box_fragment.MayHaveDescendantAboveBlockStart() {
            self.base_
                .container_builder_
                .SetMayHaveDescendantAboveBlockStart(true);
        }

        let layout_result = std::mem::take(&mut item_result.layout_result);
        line_box.AddChild(LogicalLineItem::from_layout_result_offset(
            layout_result.Get(),
            LogicalOffset::default(),
            item_result.inline_size,
            0,
            item.BidiLevel(),
        ));
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:99-101
    // cpp: layoutng_inline/inline_layout_algorithm.cc:764-854
    fn PlaceOutOfFlowObjects(
        &mut self,
        line_info: &LineInfo,
        line_box_metrics: &FontHeight,
        line_box: &mut LogicalLineItems,
    ) {
        debug_assert!(line_info.IsEmptyLine() || !line_box_metrics.IsEmpty());
        let metrics = if line_info.IsEmptyLine() {
            FontHeight::default()
        } else {
            *line_box_metrics
        };
        let space = self.base_.container_builder_.GetConstraintSpace();
        let block_level_line_location = if IsLtr(space.Direction()) {
            LayoutUnit::default()
        } else {
            space.AvailableSize().inline_size
        };
        let block_level_inline_offset = block_level_line_location
            - (self.base_.container_builder_.BfcLineOffset() - space.GetBfcOffset().line_offset);

        let mut has_preceding_inline_level_content = false;
        let mut has_rtl_block_level_out_of_flow_objects = false;
        let is_ltr = IsLtr(line_info.BaseDirection());
        for child in line_box.iter_mut() {
            has_preceding_inline_level_content |= child.HasInFlowFragment();
            let box_ = child.out_of_flow_positioned_box.Get();
            if box_.is_null() {
                continue;
            }
            let mut static_offset = LogicalOffset::new(LayoutUnit::default(), -metrics.ascent);
            if unsafe { &*box_ }.StyleRef().IsOriginalDisplayInlineType() {
                static_offset.inline_offset = child.rect.offset.inline_offset;
                self.base_.container_builder_.AddAdjoiningObjectTypes(
                    AdjoiningObjectTypeValue::kAdjoiningInlineOutOfFlow as i32,
                );
            } else {
                static_offset.inline_offset = block_level_inline_offset;
                if is_ltr {
                    if has_preceding_inline_level_content {
                        static_offset.block_offset += metrics.LineHeight();
                    }
                } else {
                    has_rtl_block_level_out_of_flow_objects = true;
                }
            }
            child.rect.offset = static_offset;
        }

        if has_rtl_block_level_out_of_flow_objects {
            has_preceding_inline_level_content = false;
            for child in line_box.iter_mut().rev() {
                let box_ = child.out_of_flow_positioned_box.Get();
                if box_.is_null() {
                    has_preceding_inline_level_content |= child.HasInFlowFragment();
                    continue;
                }
                if has_preceding_inline_level_content
                    && !unsafe { &*box_ }.StyleRef().IsOriginalDisplayInlineType()
                {
                    child.rect.offset.block_offset += metrics.LineHeight();
                }
            }
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:102-106
    // cpp: layoutng_inline/inline_layout_algorithm.cc:856-938
    fn PlaceFloatingObjects(
        &mut self,
        line_box_metrics: &FontHeight,
        opportunity: &LineLayoutOpportunity,
        ruby_block_start_adjust: LayoutUnit,
        line_info: &mut LineInfo,
        line_box: &mut LogicalLineItems,
    ) {
        debug_assert!(line_info.IsEmptyLine() || !line_box_metrics.IsEmpty());
        let metrics = if line_info.IsEmptyLine() {
            FontHeight::default()
        } else {
            *line_box_metrics
        };
        let origin_bfc_block_offset = opportunity.bfc_block_offset + metrics.LineHeight();
        let bfc_line_offset = self.base_.container_builder_.BfcLineOffset();
        let bfc_block_offset = if line_info.IsEmptyLine() {
            self.base_
                .container_builder_
                .GetConstraintSpace()
                .ExpectedBfcBlockOffset()
        } else {
            line_info.GetBfcOffset().block_offset + ruby_block_start_adjust
        };

        for child in line_box.iter_mut() {
            let unpositioned_float = child.unpositioned_float.Get();
            if !unpositioned_float.is_null() {
                let incoming = self.base_.container_builder_.PreviousBreakToken();
                debug_assert!(incoming.is_null() || !unsafe { &*incoming }.IsInParallelBlockFlow());
                let exclusion_space =
                    self.base_.container_builder_.GetExclusionSpace() as *mut ExclusionSpace;
                let mut positioned_float = self.PositionFloat(
                    origin_bfc_block_offset,
                    unpositioned_float,
                    exclusion_space,
                );
                let break_token = positioned_float.BreakToken();
                if !break_token.is_null() {
                    let parallel_token = InlineBreakToken::CreateForParallelBlockFlow(
                        self.base_.node_.clone(),
                        &child.item_index,
                        unsafe { &*break_token },
                    );
                    line_info.PropagateParallelFlowBreakToken(parallel_token);
                    if positioned_float.minimum_space_shortage != LayoutUnit::default() {
                        line_info
                            .PropagateMinimumSpaceShortage(positioned_float.minimum_space_shortage);
                        debug_assert_eq!(
                            positioned_float.tallest_unbreakable_block_size,
                            LayoutUnit::default()
                        );
                    } else if positioned_float.tallest_unbreakable_block_size
                        != LayoutUnit::default()
                    {
                        line_info.PropagateTallestUnbreakableBlockSize(
                            positioned_float.tallest_unbreakable_block_size,
                        );
                    }
                }
                if break_token.is_null() || !unsafe { &*break_token }.IsBreakBefore() {
                    child.layout_result = std::mem::take(&mut positioned_float.layout_result);
                    child.bfc_offset = positioned_float.bfc_offset;
                    child.unpositioned_float = Member::default();
                }
            }

            let child_result = child.layout_result.Get();
            if child_result.is_null()
                || !unsafe { &*child_result }.GetPhysicalFragment().IsFloating()
            {
                continue;
            }
            let space = self.base_.container_builder_.GetConstraintSpace();
            let block_offset = if IsFlippedLinesWritingMode(space.GetWritingMode()) {
                let fragment = LogicalFragment::new(
                    space.GetWritingDirection(),
                    unsafe { &*child_result }.GetPhysicalFragment(),
                );
                -fragment.BlockSize() - child.bfc_offset.block_offset
                    + bfc_block_offset
                    + metrics.descent
            } else {
                child.bfc_offset.block_offset - bfc_block_offset - metrics.ascent
            };
            child.rect.offset =
                LogicalOffset::new(child.bfc_offset.line_offset - bfc_line_offset, block_offset);
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:112-116
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1025-1069
    fn AddAnyClearanceAfterLine(&mut self, line_info: &LineInfo) -> bool {
        let line_items = line_info.Results();
        if line_items.empty() {
            return true;
        }
        let item_result = line_items.last().unwrap();
        debug_assert!(!item_result.item.Get().is_null());
        let item = unsafe { &*item_result.item.Get() };
        let layout_object = item.GetLayoutObject();
        let content_size = self.base_.container_builder_.LineHeight()
            - self
                .base_
                .container_builder_
                .TrimBlockEndBy()
                .unwrap_or_default();
        if !layout_object.is_null() && unsafe { &*layout_object }.IsBR() {
            let line_box_bfc_block_offset = self
                .base_
                .container_builder_
                .LineBoxBfcBlockOffset()
                .expect("line box BFC block offset");
            let mut bfc_offset = BfcOffset::new(
                LayoutUnit::default(),
                line_box_bfc_block_offset + content_size,
            );
            let block_end_offset_without_clearance = bfc_offset.block_offset;
            let clear_type =
                unsafe { &*item.Style() }.ClearWithContainingStyle(self.base_.node_.Style());
            if clear_type != foundation::EClear::kNone {
                let clearance = self
                    .base_
                    .container_builder_
                    .GetExclusionSpace()
                    .ClearanceOffset(clear_type);
                AdjustToClearance(clearance, &mut bfc_offset);
                self.base_.container_builder_.SetClearanceAfterLine(
                    bfc_offset.block_offset - block_end_offset_without_clearance,
                );
            }
            if self
                .base_
                .container_builder_
                .GetConstraintSpace()
                .HasBlockFragmentation()
                && self
                    .base_
                    .container_builder_
                    .GetExclusionSpace()
                    .NeedsClearancePastFragmentainer(clear_type)
            {
                return false;
            }
        }
        true
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:85-85
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1072-1089
    pub fn ShapeLineClampEllipsis(node: &InlineNode) -> LineClampEllipsis {
        debug_assert!(RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled());
        let font = node.Style().GetFont();
        let font_data = unsafe { &*font }.PrimaryFont();
        debug_assert!(!font_data.is_null());
        let ellipsis_text =
            if !font_data.is_null() && unsafe { &*font_data }.GlyphForCharacter(0x2026) != 0 {
                String::from_utf16(&[0x2026])
            } else {
                String::from("...")
            };
        let shaper = HarfBuzzShaper::new(ellipsis_text.clone());
        let shape_result = shaper.Shape(font, node.BaseDirection());
        debug_assert!(!shape_result.is_null());
        let text_metrics = unsafe { &*font_data }
            .GetFontMetrics()
            .GetFontHeight(node.Style().GetFontBaseline());
        LineClampEllipsis {
            text: ellipsis_text,
            shape_result,
            text_metrics,
        }
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:124
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1091-1094
    fn SetupLineClampEllipsis(&mut self) -> LayoutUnit {
        self.line_clamp_ellipsis_ = Some(Self::ShapeLineClampEllipsis(&self.base_.node_));
        unsafe { &*self.line_clamp_ellipsis_.as_ref().unwrap().shape_result }.SnappedWidth()
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:59-59
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1096-1547
    pub fn Layout(&mut self) -> *const LayoutResult {
        let constraint_space_ptr =
            self.base_.container_builder_.GetConstraintSpace() as *const ConstraintSpace;
        let constraint_space = unsafe { &*constraint_space_ptr };
        let mut initial_exclusion_space = constraint_space.GetExclusionSpace().clone();
        let context = unsafe { &mut *self.context_ };
        context.ClearParallelFlowBreakTokens();
        self.end_margin_strut_ = constraint_space.GetMarginStrut();
        self.base_
            .container_builder_
            .SetAdjoiningObjectTypes(constraint_space.GetAdjoiningObjectTypes());
        self.lines_until_clamp_ = constraint_space.GetLineClampData().LinesUntilClamp(true);

        let mut leading_floats = LeadingFloats::default();
        self.PositionLeadingFloats(&mut initial_exclusion_space, &mut leading_floats);

        let mut is_pushed_by_floats = false;
        let mut bfc_block_offset = constraint_space.ForcedBfcBlockOffset().unwrap_or(
            constraint_space.GetBfcOffset().block_offset + constraint_space.GetMarginStrut().Sum(),
        );
        if constraint_space.HasClearanceOffset()
            && bfc_block_offset < constraint_space.ClearanceOffset()
        {
            bfc_block_offset = constraint_space.ClearanceOffset();
            is_pushed_by_floats = true;
        }

        if context.ItemsBuilder().Size() == 0 {
            let clear_type = if self.base_.node_.HasInitialLetterBox() {
                foundation::EClear::kBoth
            } else {
                self.base_
                    .node_
                    .Style()
                    .ClearWithDirection(constraint_space.Direction())
            };
            let initial_letter_clearance = constraint_space
                .GetExclusionSpace()
                .InitialLetterClearanceOffset(clear_type);
            if initial_letter_clearance > bfc_block_offset {
                bfc_block_offset = initial_letter_clearance;
                is_pushed_by_floats = true;
            }
        }

        let mut opportunities = initial_exclusion_space.AllLayoutOpportunities(
            &BfcOffset::new(
                constraint_space.GetBfcOffset().line_offset,
                bfc_block_offset,
            ),
            constraint_space.AvailableSize().inline_size,
            constraint_space.Direction(),
        );
        let _scope = unsafe { ClearCollectionScope::new(&mut opportunities) };

        let break_token = self.base_.GetBreakToken();
        if !break_token.is_null() && unsafe { &*break_token }.IsInParallelBlockFlow() {
            self.base_.container_builder_.SetIsLineForParallelFlow();
        }
        self.apply_text_fit_ = ShouldApplyTextFit(&self.base_.node_);

        let items_builder = context.ItemsBuilder()
            as *mut layoutng_fragment_tree::fragment_items_builder::FragmentItemsBuilder;
        let line_container = unsafe { &mut *items_builder }.AcquireLogicalLineContainer();
        debug_assert!(!line_container.is_null());
        let mut line_break_strategy = LineBreakStrategy::new(
            context,
            &self.base_.node_,
            self.base_.node_.Style(),
            break_token,
            self.column_spanner_path_,
        );
        let mut is_line_created = false;
        let mut is_end_paragraph = false;
        let mut previous_line_block_size = kIndefiniteSize;
        let mut line_block_size = LayoutUnit::default();
        let mut block_delta = LayoutUnit::default();
        let mut opportunities_index = 0usize;
        while opportunities_index < opportunities.len() {
            let opportunity = &opportunities[opportunities_index];
            #[cfg(debug_assertions)]
            if opportunities_index + 1 == opportunities.len() {
                debug_assert!(!opportunity.HasShapeExclusions());
                debug_assert_eq!(previous_line_block_size, kIndefiniteSize);
                debug_assert_eq!(line_block_size, LayoutUnit::default());
                debug_assert_eq!(block_delta, LayoutUnit::default());
                if opportunity.rect.InlineSize() != LayoutUnit::Max() {
                    debug_assert_eq!(
                        opportunity.rect.InlineSize(),
                        constraint_space.AvailableSize().inline_size
                    );
                }
                debug_assert_eq!(opportunity.rect.BlockSize(), LayoutUnit::Max());
            }

            self.base_.container_builder_.Reset();
            self.base_
                .container_builder_
                .SetExclusionSpace(&initial_exclusion_space);
            is_line_created = false;
            let line_opportunity = opportunity.ComputeLineLayoutOpportunity(
                constraint_space,
                line_block_size,
                block_delta,
            );
            if line_break_strategy.NeedsToPrepare() {
                let node = self.base_.node_.clone();
                line_break_strategy.Prepare(
                    context,
                    &node,
                    constraint_space,
                    &opportunities[opportunities_index..],
                    &line_opportunity,
                    &leading_floats,
                    break_token,
                    self.base_.GetExclusionSpace(),
                );
            }

            let mut is_line_info_cached = false;
            let line_info_ptr =
                context.GetLineInfo(break_token, &mut is_line_info_cached) as *mut LineInfo;
            let line_info = unsafe { &mut *line_info_ptr };
            if is_line_info_cached {
                line_info.SetBfcOffset(&BfcOffset::new(
                    line_opportunity.line_left_offset,
                    line_opportunity.bfc_block_offset,
                ));
            } else {
                let mut line_breaker = LineBreaker::new(
                    self.base_.node_.clone(),
                    crate::line_breaker::LineBreakerMode::kContent,
                    constraint_space,
                    &line_opportunity,
                    &leading_floats,
                    break_token,
                    self.column_spanner_path_,
                    self.base_.GetExclusionSpace(),
                );
                line_break_strategy.SetupLineBreaker(context, &mut line_breaker);
                if RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled()
                    && self.GetLineClampState(None) == LineClampState::kLineClampEllipsis
                {
                    let ellipsis_width = self.SetupLineClampEllipsis();
                    line_breaker.SetLineClampEllipsisWidth(ellipsis_width);
                }
                line_breaker.NextLine(line_info);

                if self.line_clamp_ellipsis_.is_some() {
                    if line_info.IsEmptyLine() || line_info.IsBlockInInline() {
                        self.line_clamp_ellipsis_ = None;
                    } else if !line_info.IsLastLine() {
                        match self.DoesRemainderFitInLineWithoutEllipsis(line_info) {
                            Some(true) => self
                                .base_
                                .container_builder_
                                .SetWouldBeLastLineIfNotForEllipsis(),
                            None => {
                                let mut without_ellipsis = LineBreaker::new(
                                    self.base_.node_.clone(),
                                    crate::line_breaker::LineBreakerMode::kContent,
                                    constraint_space,
                                    &line_opportunity,
                                    &leading_floats,
                                    break_token,
                                    self.column_spanner_path_,
                                    self.base_.GetExclusionSpace(),
                                );
                                line_break_strategy
                                    .SetupLineBreaker(context, &mut without_ellipsis);
                                let mut line_info_without_ellipsis = LineInfo::default();
                                without_ellipsis.NextLine(&mut line_info_without_ellipsis);
                                if line_info_without_ellipsis.GetBreakToken().is_null() {
                                    self.base_
                                        .container_builder_
                                        .SetWouldBeLastLineIfNotForEllipsis();
                                }
                            }
                            Some(false) => {}
                        }
                    }
                }
            }

            if self.base_.node_.IsInitialLetterBox() {
                line_info.SetWidth(
                    line_info.AvailableWidth(),
                    crate::initial_letter_utils::CalculateInitialLetterBoxInlineSize(line_info),
                );
            }

            let block_in_inline_result = line_info.BlockInInlineLayoutResult();
            if !block_in_inline_result.is_null() {
                let result = unsafe { &*block_in_inline_result };
                if result.Status() != EStatus::kSuccess {
                    unsafe { &mut *items_builder }.ReleaseCurrentLogicalLineContainer();
                    return block_in_inline_result;
                }
                if result.IsPushedByFloats() {
                    self.base_.container_builder_.SetIsPushedByFloats();
                } else if result.SubtreeModifiedMarginStrut() {
                    self.base_
                        .container_builder_
                        .SetSubtreeModifiedMarginStrut();
                }
            }

            if !line_info.IsEmptyLine() {
                let result_bfc_block_offset = if block_in_inline_result.is_null() {
                    None
                } else {
                    unsafe { &*block_in_inline_result }.BfcBlockOffset()
                };
                if let Some(offset) = result_bfc_block_offset {
                    self.base_.container_builder_.SetBfcBlockOffset(offset);
                    self.base_
                        .container_builder_
                        .SetLineBoxBfcBlockOffset(offset);
                } else {
                    self.base_
                        .container_builder_
                        .SetBfcBlockOffset(bfc_block_offset);
                    self.base_
                        .container_builder_
                        .SetLineBoxBfcBlockOffset(line_info.GetBfcOffset().block_offset);
                    if is_pushed_by_floats {
                        self.base_.container_builder_.SetIsPushedByFloats();
                    }
                }
                if self.base_.container_builder_.GetAdjoiningObjectTypes() != 0
                    && bfc_block_offset != constraint_space.ExpectedBfcBlockOffset()
                {
                    unsafe { &mut *items_builder }.ReleaseCurrentLogicalLineContainer();
                    return self
                        .base_
                        .container_builder_
                        .Abort(EStatus::kBfcBlockOffsetResolved);
                }
            }

            if line_info.HasOverflow()
                && !line_opportunity
                    .IsEqualToAvailableFloatInlineSize(constraint_space.AvailableSize().inline_size)
                && self.base_.node_.Style().ShouldWrapLine()
            {
                debug_assert!(!line_info.IsBlockInInline());
                if opportunity.HasShapeExclusions()
                    && block_delta < opportunity.rect.BlockSize()
                    && !opportunity.IsBlockDeltaBelowShapes(block_delta)
                {
                    block_delta += LayoutUnit::from_signed(1);
                    previous_line_block_size = kIndefiniteSize;
                    line_block_size = LayoutUnit::default();
                    continue;
                }
                if opportunities_index + 1 != opportunities.len() {
                    block_delta = LayoutUnit::default();
                    previous_line_block_size = kIndefiniteSize;
                    line_block_size = LayoutUnit::default();
                    opportunities_index += 1;
                    continue;
                }
            }

            let mut should_scale_line_height = false;
            if self.apply_text_fit_ {
                if context.IsMeasuringScale() {
                    // The measured paragraph scale is applied after this line.
                } else {
                    let scale = context.MeasuredScale();
                    if scale.scale != 1.0 {
                        LineFitter::new(&self.base_.node_, line_info_ptr)
                            .FitLine(scale.scale, Some(scale.additional_paint_time_scale));
                        should_scale_line_height = true;
                    } else {
                        should_scale_line_height =
                            LineFitter::new(&self.base_.node_, line_info_ptr).MeasureAndFitLine();
                    }
                }
            }

            self.PrepareBoxStates(line_info, should_scale_line_height, break_token);
            self.CreateLine(
                &line_opportunity,
                line_info,
                should_scale_line_height,
                unsafe { &mut *line_container },
            );
            is_line_created = true;
            is_end_paragraph = line_info.IsEndParagraph();

            let block_start_adjust = line_info.ComputeBlockStartAdjustment();
            if block_start_adjust != LayoutUnit::default() {
                debug_assert!(self.base_.container_builder_.BfcBlockOffset().is_some());
                debug_assert!(self
                    .base_
                    .container_builder_
                    .LineBoxBfcBlockOffset()
                    .is_some());
                debug_assert!(!line_info.IsEmptyLine());
                self.base_.container_builder_.SetLineBoxBfcBlockOffset(
                    line_info.GetBfcOffset().block_offset + block_start_adjust,
                );
                self.base_
                    .container_builder_
                    .SetAnnotationBlockOffsetAdjustment(
                        line_info.ComputeAnnotationBlockOffsetAdjustment(),
                    );
            }
            let total_block_size = line_info.ComputeTotalBlockSize(
                self.base_.container_builder_.LineHeight(),
                self.base_
                    .container_builder_
                    .AnnotationOverflow()
                    .ClampNegativeToZero(),
            );
            if opportunity.HasShapeExclusions() && !line_info.IsEmptyLine() {
                let new_inline_size = opportunity
                    .ComputeLineLayoutOpportunity(constraint_space, total_block_size, block_delta)
                    .AvailableInlineSize();
                let old_inline_size = line_opportunity.AvailableInlineSize();
                let use_new_size = if previous_line_block_size == total_block_size {
                    new_inline_size < old_inline_size
                } else {
                    new_inline_size != old_inline_size
                };
                if use_new_size {
                    previous_line_block_size = line_block_size;
                    line_block_size = total_block_size;
                    continue;
                }
            }
            if total_block_size + block_delta > opportunity.rect.BlockSize() {
                block_delta = LayoutUnit::default();
                previous_line_block_size = kIndefiniteSize;
                line_block_size = LayoutUnit::default();
                opportunities_index += 1;
                continue;
            }

            self.base_
                .container_builder_
                .SetBreakToken(line_info.GetBreakToken());
            self.base_
                .container_builder_
                .SetBaseDirection(line_info.BaseDirection());
            for token in line_info.ParallelFlowBreakTokens().iter() {
                let parallel_token = token.Get();
                debug_assert!(unsafe { &*parallel_token }.IsInParallelBlockFlow());
                context.PropagateParallelFlowBreakToken(parallel_token.cast());
            }
            if let Some(minimum_space_shortage) = line_info.MinimumSpaceShortage() {
                self.base_
                    .container_builder_
                    .PropagateSpaceShortage(Some(minimum_space_shortage));
                debug_assert_eq!(
                    line_info.TallestUnbreakableBlockSize(),
                    LayoutUnit::default()
                );
            } else if line_info.TallestUnbreakableBlockSize() != LayoutUnit::default() {
                self.base_
                    .container_builder_
                    .PropagateTallestUnbreakableBlockSize(line_info.TallestUnbreakableBlockSize());
            }

            if line_info.IsEmptyLine() {
                debug_assert_eq!(
                    self.base_.container_builder_.BlockSize(),
                    LayoutUnit::default()
                );
                debug_assert!(self.base_.container_builder_.BfcBlockOffset().is_none());
                self.base_.container_builder_.SetIsSelfCollapsing();
                self.base_.container_builder_.SetIsEmptyLineBox();
                if let Some(forced) = constraint_space.ForcedBfcBlockOffset() {
                    self.base_.container_builder_.SetBfcBlockOffset(forced);
                    self.base_
                        .container_builder_
                        .SetLineBoxBfcBlockOffset(forced);
                }
            } else {
                if !self.AddAnyClearanceAfterLine(line_info) {
                    return self
                        .base_
                        .container_builder_
                        .Abort(EStatus::kOutOfFragmentainerSpace);
                }
                let line_height = self.base_.container_builder_.LineHeight();
                self.base_.container_builder_.SetBlockSize(line_height);
                if !line_info.IsBlockInInline() {
                    self.end_margin_strut_ = MarginStrut::default();
                    if let Some(lines_until_clamp) = &mut self.lines_until_clamp_ {
                        if constraint_space.GetLineClampData().IsClampByLines() {
                            *lines_until_clamp -= 1;
                        } else {
                            debug_assert!(constraint_space.GetLineClampData().IsCountLines());
                            *lines_until_clamp += 1;
                        }
                    }
                }
                self.base_.container_builder_.ResetAdjoiningObjectTypes();
            }
            break;
        }

        assert!(is_line_created);
        self.base_
            .container_builder_
            .SetEndMarginStrut(&self.end_margin_strut_);
        if self.lines_until_clamp_.is_some() {
            self.base_
                .container_builder_
                .SetLinesUntilClamp(self.lines_until_clamp_);
        }
        self.base_
            .container_builder_
            .PropagateChildrenData(unsafe { &mut *line_container });
        let layout_result = self.base_.container_builder_.ToLineBoxFragment();
        unsafe { &mut *items_builder }.AssociateLogicalLineContainer(
            line_container,
            unsafe { &*layout_result }.GetPhysicalFragment(),
        );
        line_break_strategy.DidCreateLine(is_end_paragraph);
        layout_result
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:102-104
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1549-1733
    fn DoesRemainderFitInLineWithoutEllipsis(&self, line_info: &LineInfo) -> Option<bool> {
        debug_assert!(!line_info.IsLastLine());
        debug_assert!(!line_info.IsEmptyLine() && !line_info.IsBlockInInline());
        debug_assert!(!line_info.IsRubyBase() && !line_info.IsRubyText());
        if line_info.GetBreakToken().is_null() {
            return Some(true);
        }
        if line_info.HasForcedBreak() {
            return Some(false);
        }

        let ellipsis = self
            .line_clamp_ellipsis_
            .as_ref()
            .expect("line-clamp ellipsis must be shaped");
        let mut remaining_width = line_info.AvailableWidth() - line_info.Width()
            + unsafe { &*ellipsis.shape_result }.SnappedWidth();
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum BreakpointStatus {
            NoBreakpoints,
            HasBreakpoints,
            MightHaveBreakpoints,
        }
        let mut breakpoint_status = BreakpointStatus::NoBreakpoints;

        let items_data = self.base_.node_.ItemsData(line_info.UseFirstLineStyle());
        let items = &items_data.items;
        let text = items_data.text_content.clone();
        let mut current = InlineItemTextIndex::default();
        if line_info.HasTrailingSpaces() {
            current = line_info.Results()[0].Start();
            for item_result in line_info.Results().iter().rev() {
                let item = unsafe { &*item_result.item.Get() };
                if item.EndCollapseType() != CollapseType::kOpaqueToCollapsing
                    && !item_result.has_only_pre_wrap_trailing_spaces
                {
                    current = item_result.End();
                    breakpoint_status = BreakpointStatus::HasBreakpoints;
                    break;
                }
            }
        } else if let Some(last) = line_info.Results().last() {
            breakpoint_status = BreakpointStatus::HasBreakpoints;
            current = last.End();
        } else {
            current = *unsafe { &*line_info.GetBreakToken() }.Start();
        }

        let mut can_hang_or_collapse = LayoutUnit::default();
        while remaining_width + can_hang_or_collapse >= LayoutUnit::default()
            && current.item_index < items.size()
        {
            let item = unsafe { &*items[current.item_index as usize].Get() };
            debug_assert!(item.StartOffset() <= current.text_offset);
            debug_assert!(item.EndOffset() >= current.text_offset);
            if item.IsForcedLineBreak() || item.Type() == InlineItemType::kBlockInInline {
                return Some(false);
            } else if matches!(
                item.Type(),
                InlineItemType::kText | InlineItemType::kControl
            ) {
                let units = text.Span16().unwrap_or_default();
                if breakpoint_status != BreakpointStatus::HasBreakpoints
                    && item.Type() == InlineItemType::kControl
                    && units[item.StartOffset() as usize] == 0x200b
                {
                    breakpoint_status = BreakpointStatus::HasBreakpoints;
                }
                if current.text_offset == item.EndOffset() {
                    current.item_index += 1;
                    continue;
                }

                let shape_result = item.TextShapeResult();
                if shape_result.is_null() {
                    return None;
                }
                let mut width = unsafe { &*shape_result }
                    .SnappedWidth()
                    .ClampNegativeToZero();
                if current.text_offset != item.StartOffset() {
                    let view = ShapeResultView::CreateFromResultRange(
                        shape_result,
                        item.StartOffset(),
                        current.text_offset,
                    );
                    width = (width - unsafe { &*view }.SnappedWidth()).ClampNegativeToZero();
                }
                remaining_width -= width;
                match item.EndCollapseType() {
                    CollapseType::kNotCollapsible | CollapseType::kCollapsed => {
                        let mut has_hanging_whitespace = false;
                        let style = unsafe { &*item.Style() };
                        if !style.ShouldBreakSpaces() {
                            let last_char = units[(item.EndOffset() - 1) as usize];
                            has_hanging_whitespace =
                                Character::IsOtherSpaceSeparator(i32::from(last_char))
                                    || (style.ShouldPreserveWhiteSpaces()
                                        && style.ShouldWrapLine()
                                        && (last_char == 0x20 || last_char == 0x09));
                        }
                        if has_hanging_whitespace {
                            can_hang_or_collapse += width;
                        } else {
                            can_hang_or_collapse = LayoutUnit::default();
                        }
                    }
                    CollapseType::kCollapsible => can_hang_or_collapse += width,
                    CollapseType::kOpaqueToCollapsing => {}
                }
                if breakpoint_status == BreakpointStatus::NoBreakpoints
                    && unsafe { &*item.Style() }.ShouldWrapLine()
                {
                    breakpoint_status = BreakpointStatus::MightHaveBreakpoints;
                }
            } else if item.Type() == InlineItemType::kOpenTag {
                let style = unsafe { &*item.Style() };
                let space = self.base_.container_builder_.GetConstraintSpace();
                let bmp_width = ComputeLineBorders(style).inline_start
                    + ComputeLinePadding(space, style).inline_start
                    + ComputeLineMarginsForSelf(space, style).inline_start;
                remaining_width -= bmp_width;
                if bmp_width != LayoutUnit::default() {
                    can_hang_or_collapse = LayoutUnit::default();
                }
            } else if item.Type() == InlineItemType::kCloseTag {
                let style = unsafe { &*item.Style() };
                let space = self.base_.container_builder_.GetConstraintSpace();
                let bmp_width = ComputeLineBorders(style).inline_end
                    + ComputeLinePadding(space, style).inline_end
                    + ComputeLineMarginsForSelf(space, style).inline_end;
                remaining_width -= bmp_width;
                if bmp_width != LayoutUnit::default() {
                    can_hang_or_collapse = LayoutUnit::default();
                }
            } else if matches!(
                item.Type(),
                InlineItemType::kBidiControl | InlineItemType::kOutOfFlowPositioned
            ) {
                // Neither item contributes width or changes whitespace behavior.
            } else {
                debug_assert!(matches!(
                    item.Type(),
                    InlineItemType::kAtomicInline
                        | InlineItemType::kFloating
                        | InlineItemType::kInitialLetterBox
                        | InlineItemType::kListMarker
                        | InlineItemType::kOpenRubyColumn
                        | InlineItemType::kCloseRubyColumn
                        | InlineItemType::kRubyLinePlaceholder
                ));
                return None;
            }
            current.item_index += 1;
            current.text_offset = item.EndOffset();
        }

        if remaining_width >= LayoutUnit::default()
            || breakpoint_status == BreakpointStatus::NoBreakpoints
        {
            return Some(true);
        }
        if remaining_width + can_hang_or_collapse <= LayoutUnit::default()
            && breakpoint_status == BreakpointStatus::HasBreakpoints
        {
            return Some(false);
        }
        None
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:91-93
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1787-1825
    fn PositionFloat(
        &mut self,
        origin_bfc_block_offset: LayoutUnit,
        floating_object: *mut LayoutObject,
        exclusion_space: *mut ExclusionSpace,
    ) -> PositionedFloat {
        let space_ptr =
            self.base_.container_builder_.GetConstraintSpace() as *const ConstraintSpace;
        let space = unsafe { &*space_ptr };
        let origin_bfc_offset =
            BfcOffset::new(space.GetBfcOffset().line_offset, origin_bfc_block_offset);
        let is_hidden_for_paint = space.GetLineClampData().ShouldHideForPaint();
        let child = BlockNode::new(To::<LayoutBox>(floating_object));
        let percentage_size = if child.IsReplaced() {
            space.ReplacedChildPercentageResolutionSize()
        } else {
            space.PercentageResolutionSize()
        };
        let style = self.base_.node_.Style() as *const _;
        let mut unpositioned_float = UnpositionedFloat::new(
            child,
            std::ptr::null(),
            space.AvailableSize(),
            percentage_size,
            &origin_bfc_offset,
            space,
            unsafe { &*style },
            space.FragmentainerBlockSize(),
            space.FragmentainerOffset(),
            is_hidden_for_paint,
        );
        let support = unsafe { &*LayoutPassScope::RequireFloatSupport() };
        let positioned_float = support
            .position
            .expect("float layout support must be installed")(
            &mut unpositioned_float,
            exclusion_space,
        );
        if positioned_float.minimum_space_shortage != LayoutUnit::default() {
            self.base_
                .container_builder_
                .PropagateSpaceShortage(Some(positioned_float.minimum_space_shortage));
            debug_assert_eq!(
                positioned_float.tallest_unbreakable_block_size,
                LayoutUnit::default()
            );
        } else if positioned_float.tallest_unbreakable_block_size != LayoutUnit::default() {
            self.base_
                .container_builder_
                .PropagateTallestUnbreakableBlockSize(
                    positioned_float.tallest_unbreakable_block_size,
                );
        }
        positioned_float
    }

    // cpp: layoutng_inline/inline_layout_algorithm.h:90-90
    // cpp: layoutng_inline/inline_layout_algorithm.cc:1737-1785
    fn PositionLeadingFloats(
        &mut self,
        exclusion_space: &mut ExclusionSpace,
        leading_floats: &mut LeadingFloats,
    ) {
        let incoming = self.base_.container_builder_.PreviousBreakToken();
        if !incoming.is_null() && unsafe { &*incoming }.IsInParallelBlockFlow() {
            return;
        }
        let items = &self.base_.node_.ItemsData(false).items as *const InlineItems;
        let mut index = if incoming.is_null() {
            0
        } else {
            unsafe { &*incoming }.StartItemIndex()
        };
        while index < unsafe { &*items }.size() {
            let item = unsafe { &*(&*items)[index as usize].Get() };
            if !item.IsEmptyItem() {
                break;
            }
            if item.Type() != InlineItemType::kFloating {
                index += 1;
                continue;
            }

            let space = self.base_.container_builder_.GetConstraintSpace();
            let is_left = unsafe { &*item.GetLayoutObject() }
                .StyleRef()
                .FloatingWithDirection(space.Direction())
                == EFloat::kLeft;
            let origin_bfc_block_offset = space.ExpectedBfcBlockOffset();
            self.base_
                .container_builder_
                .AddAdjoiningObjectTypes(if is_left {
                    AdjoiningObjectTypeValue::kAdjoiningFloatLeft as i32
                } else {
                    AdjoiningObjectTypeValue::kAdjoiningFloatRight as i32
                });
            let positioned_float = self.PositionFloat(
                origin_bfc_block_offset,
                item.GetLayoutObject(),
                exclusion_space,
            );

            let mut parallel_break_token = std::ptr::null();
            if self
                .base_
                .container_builder_
                .GetConstraintSpace()
                .HasBlockFragmentation()
            {
                let float_break_token = positioned_float.BreakToken();
                if !float_break_token.is_null() {
                    let start = InlineItemTextIndex {
                        item_index: index,
                        text_offset: item.StartOffset(),
                    };
                    parallel_break_token = InlineBreakToken::CreateForParallelBlockFlow(
                        self.base_.node_.clone(),
                        &start,
                        unsafe { &*float_break_token },
                    );
                }
            }
            leading_floats.Add(&positioned_float, parallel_break_token);
            index += 1;
        }
        leading_floats.SetHandledIndex(index);
    }
}

// The C++ destructor is defaulted at inline_layout_algorithm.cc:254; Rust
// field Drop preserves destruction order without a custom implementation.
