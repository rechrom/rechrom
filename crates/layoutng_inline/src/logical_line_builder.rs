// C++: layoutng_inline/logical_line_builder.h/.cc. Source behavior mapped;
// integration awaits completion of the layoutng_inline package.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::text::native::bidi_paragraph::BidiParagraph;
use font_engine::FontBaseline;
use foundation::{
    EPosition, ETextAlign, HeapVector, IsA, IsLtr, LayoutUnit, MakeGarbageCollected, Member,
    RubyPosition, RuntimeEnabledFeatures, String, TextDirection, To, WritingDirectionMode,
};
use icu_bidi::UBiDiLevel;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::inline_item_result::{FindTextScale, InlineItemResult, InlineItemResults};
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::text_fit_scale::TextFitBlockScale;
use layoutng::internal::text_item_type::TextItemType;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::inline_items_data::OpenTagItems;
use layoutng_fragment_tree::logical_box_fragment::LogicalBoxFragment;
use layoutng_fragment_tree::logical_line_item::{LogicalLineItem, LogicalLineItems};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;

use crate::inline_box_state::{InlineBoxState, InlineLayoutStateStack, LogicalRubyColumn};
use crate::inline_child_layout_context::InlineChildLayoutContext;
use crate::inline_layout_algorithm::InlineLayoutAlgorithm;
use crate::justification_utils::ApplyLeftAndRightExpansion;
use crate::line_breaker::LineBreaker;
use crate::line_info::LineInfo;
use crate::ruby_utils::ApplyRubyAlign;

// cpp: layoutng_inline/logical_line_builder.cc:29-54
fn CanUseItemForNeedsPaint(item: &InlineItem) -> bool {
    match item.Type() {
        InlineItemType::kBlockInInline
        | InlineItemType::kCloseTag
        | InlineItemType::kFloating
        | InlineItemType::kOutOfFlowPositioned
        | InlineItemType::kListMarker
        | InlineItemType::kOpenTag
        | InlineItemType::kBidiControl
        | InlineItemType::kOpenRubyColumn
        | InlineItemType::kCloseRubyColumn
        | InlineItemType::kRubyLinePlaceholder => return false,
        InlineItemType::kControl | InlineItemType::kText => {
            if item.Length() == 0 {
                return false;
            }
        }
        InlineItemType::kAtomicInline | InlineItemType::kInitialLetterBox => {}
    }
    !item.GetLayoutObject().is_null()
}

// cpp: layoutng_inline/logical_line_builder.cc:56-91
fn LayoutObjectForLineClampEllipsis(
    node: &InlineNode,
    line_items: &InlineItemResults,
    line_start: &InlineItemTextIndex,
) -> *mut LayoutObject {
    for item_result in line_items.iter().rev() {
        let item = unsafe { &*item_result.item.Get() };
        if !CanUseItemForNeedsPaint(item) {
            continue;
        }
        if matches!(
            item.Type(),
            InlineItemType::kText | InlineItemType::kControl
        ) && item_result.Length() == 0
        {
            continue;
        }
        return item.GetLayoutObject();
    }

    let items = &node.ItemsData(false).items;
    for item in items.iter().take(line_start.item_index as usize).rev() {
        let item = unsafe { &*item.Get() };
        if CanUseItemForNeedsPaint(item) {
            return item.GetLayoutObject();
        }
    }
    node.GetLayoutBlockFlow() as *mut LayoutObject
}

// cpp: layoutng_inline/logical_line_builder.h:27-126
// C++ reference and stack-only links are non-owning pointers here; the
// InlineLayoutAlgorithm keeps the builder within one line-layout call.
pub struct LogicalLineBuilder {
    node_: InlineNode,
    constraint_space_: *const ConstraintSpace,
    break_token_: *const InlineBreakToken,
    box_states_: *mut InlineLayoutStateStack,
    context_: *mut InlineChildLayoutContext,
    baseline_type_: FontBaseline,
    quirks_mode_: bool,
    should_scale_line_height_: bool,
    has_out_of_flow_positioned_items_: bool,
    has_floating_items_: bool,
    has_relative_positioned_items_: bool,
    initial_letter_item_result_: *const InlineItemResult,
}

impl LogicalLineBuilder {
    // cpp: layoutng_inline/logical_line_builder.h:31-36
    // cpp: layoutng_inline/logical_line_builder.cc:96-107
    pub fn new(
        node: InlineNode,
        constraint_space: &ConstraintSpace,
        break_token: *const InlineBreakToken,
        state_stack: *mut InlineLayoutStateStack,
        context: *mut InlineChildLayoutContext,
        should_scale_line_height: bool,
    ) -> Self {
        let baseline_type = node.Style().GetFontBaseline();
        let quirks_mode = unsafe { &*node.GetLayoutBox() }.InLineHeightQuirksModeForLayout();
        Self {
            node_: node,
            constraint_space_: constraint_space,
            break_token_: break_token,
            box_states_: state_stack,
            context_: context,
            baseline_type_: baseline_type,
            quirks_mode_: quirks_mode,
            should_scale_line_height_: should_scale_line_height,
            has_out_of_flow_positioned_items_: false,
            has_floating_items_: false,
            has_relative_positioned_items_: false,
            initial_letter_item_result_: std::ptr::null(),
        }
    }

    // cpp: layoutng_inline/logical_line_builder.h:46-48
    pub fn InitialLetterItemResult(&self) -> *const InlineItemResult {
        self.initial_letter_item_result_
    }

    // cpp: layoutng_inline/logical_line_builder.h:49-51
    pub fn HasOutOfFlowPositionedItems(&self) -> bool {
        self.has_out_of_flow_positioned_items_
    }

    // cpp: layoutng_inline/logical_line_builder.h:52
    pub fn HasFloatingItems(&self) -> bool {
        self.has_floating_items_
    }

    // cpp: layoutng_inline/logical_line_builder.h:53-55
    pub fn HasRelativePositionedItems(&self) -> bool {
        self.has_relative_positioned_items_
    }

    // cpp: layoutng_inline/logical_line_builder.h:42-44
    // cpp: layoutng_inline/logical_line_builder.cc:109-198
    pub fn CreateLine(
        &mut self,
        line_info: &mut LineInfo,
        line_box: &mut LogicalLineItems,
        main_line_helper: *mut InlineLayoutAlgorithm,
    ) {
        let line_items = line_info.MutableResults() as *mut InlineItemResults;
        let ellipsis = if main_line_helper.is_null() {
            None
        } else {
            unsafe { &*main_line_helper }
                .GetLineClampEllipsis()
                .as_ref()
        };
        let is_line_clamp_displaced_line = ellipsis.is_some() && unsafe { &*line_items }.is_empty();
        let line_style = line_info.LineStyle();
        let box_states = unsafe { &mut *self.box_states_ };
        box_states.SetIsEmptyLine(line_info.IsEmptyLine());
        let box_state = box_states.OnBeginPlaceItems(
            &self.node_,
            line_info,
            self.baseline_type_,
            self.quirks_mode_ || is_line_clamp_displaced_line,
            self.should_scale_line_height_,
            line_box,
        );
        #[cfg(feature = "expensive_dchecks")]
        if !main_line_helper.is_null() {
            unsafe { &*main_line_helper }.CheckBoxStates(line_info, self.should_scale_line_height_);
        }

        if (self.quirks_mode_ && line_style.IsDisplayListItem()) || is_line_clamp_displaced_line {
            let text_scale = FindTextScale(
                self.should_scale_line_height_,
                unsafe { &*line_items },
                0,
                0,
            );
            let box_ref = unsafe { &mut *box_state };
            box_ref.ComputeTextMetrics(
                line_style,
                unsafe { &*box_ref.font.Get() },
                self.baseline_type_,
                &text_scale,
            );
        }
        if line_info.IsBlockInInline() {
            debug_assert_eq!(unsafe { &*line_items }.len(), 1);
            debug_assert_eq!(
                unsafe { &*(&*line_items)[0].item.Get() }.Type(),
                InlineItemType::kBlockInInline
            );
        }
        self.HandleItemResults(
            line_info,
            unsafe { &mut *line_items },
            line_box,
            main_line_helper,
            box_state,
        );
        unsafe { &mut *self.box_states_ }.OnEndPlaceItems(
            unsafe { &*self.constraint_space_ },
            line_box,
            self.baseline_type_,
        );

        if let Some(ellipsis_data) = ellipsis {
            debug_assert!(RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled());
            let shape = ShapeResultView::CreateFromResult(ellipsis_data.shape_result);
            let text_metrics = ellipsis_data.text_metrics;
            let object = LayoutObjectForLineClampEllipsis(
                &self.node_,
                unsafe { &*line_items },
                line_info.Start(),
            );
            line_box.AddChild(LogicalLineItem::ellipsis(
                unsafe { &*object },
                StyleVariant::kStandardEllipsis,
                unsafe { &*shape },
                &ellipsis_data.text,
                &LogicalRect::from_units(
                    LayoutUnit::default(),
                    -text_metrics.ascent,
                    unsafe { &*shape }.SnappedWidth(),
                    text_metrics.LineHeight(),
                ),
                if IsLtr(line_info.BaseDirection()) {
                    0
                } else {
                    1
                },
            ));
        }

        if self.node_.IsBidiEnabled() {
            let box_states = unsafe { &mut *self.box_states_ };
            box_states.PrepareForReorder(line_box);
            // BidiReorder mutates this list and self.context_, but does not
            // touch box_states_. Keep the list's owner stable across the call.
            let ruby_columns =
                box_states.RubyColumnList() as *mut HeapVector<Member<LogicalRubyColumn>>;
            self.BidiReorder(line_info.BaseDirection(), line_box, unsafe {
                &mut *ruby_columns
            });
            unsafe { &mut *self.box_states_ }.UpdateAfterReorder(line_box);
        } else {
            debug_assert!(IsLtr(line_info.BaseDirection()));
        }

        for logical_column in unsafe { &mut *self.box_states_ }
            .RubyColumnList()
            .iter_mut()
        {
            let column = unsafe { &mut *logical_column.Get() };
            let (left, right) = column.base_insets;
            let start = column.start_index as usize;
            let end = start + column.size as usize;
            ApplyLeftAndRightExpansion(left, right, &mut line_box.AsSpan()[start..end]);
        }
    }

    // cpp: layoutng_inline/logical_line_builder.h:69-72
    // cpp: layoutng_inline/logical_line_builder.cc:355-380
    fn HandleOpenTag(
        &mut self,
        item: &InlineItem,
        item_result: &InlineItemResult,
        text_scale: &TextFitBlockScale,
        line_box: &mut LogicalLineItems,
    ) -> *mut InlineBoxState {
        let box_state = unsafe { &mut *self.box_states_ }.OnOpenTagWithPlaceholder(
            unsafe { &*self.constraint_space_ },
            item,
            item_result,
            self.baseline_type_,
            text_scale,
            line_box,
        );
        if !self.quirks_mode_ || !item.IsEmptyItem() {
            let box_ref = unsafe { &mut *box_state };
            box_ref.ComputeTextMetrics(
                unsafe { &*item.Style() },
                unsafe { &*box_ref.font.Get() },
                self.baseline_type_,
                text_scale,
            );
        }
        if unsafe { &*item.Style() }.HasMask() {
            let object = item.GetLayoutObject();
            if !object.is_null() {
                unsafe { &mut *object }.SetNeedsPaintPropertyUpdate();
            }
        }
        box_state
    }

    // cpp: layoutng_inline/logical_line_builder.h:73-77
    // cpp: layoutng_inline/logical_line_builder.cc:382-402
    fn HandleCloseTag(
        &mut self,
        item: &InlineItem,
        _item_result: &InlineItemResult,
        line_box: &mut LogicalLineItems,
        box_state: *mut InlineBoxState,
    ) -> *mut InlineBoxState {
        if self.quirks_mode_ && !item.IsEmptyItem() {
            let box_ref = unsafe { &mut *box_state };
            box_ref.EnsureTextMetrics(
                unsafe { &*item.Style() },
                unsafe { &*box_ref.font.Get() },
                self.baseline_type_,
                TextFitBlockScale::kFixed,
            );
        }
        let box_state = unsafe { &mut *self.box_states_ }.OnCloseTag(
            unsafe { &*self.constraint_space_ },
            line_box,
            box_state,
            self.baseline_type_,
        );
        if !DisableLayoutSideEffectsScope::IsDisabled() {
            unsafe { &mut *item.GetLayoutObject() }.ClearNeedsLayoutWithoutPaintInvalidation();
        }
        box_state
    }

    // cpp: layoutng_inline/logical_line_builder.h:79-83
    // cpp: layoutng_inline/logical_line_builder.cc:404-445
    fn PlaceControlItem(
        &mut self,
        item: &InlineItem,
        text_content: &String,
        item_result: &mut InlineItemResult,
        line_box: &mut LogicalLineItems,
        box_state: *mut InlineBoxState,
    ) {
        debug_assert_eq!(item.Type(), InlineItemType::kControl);
        debug_assert!(item.Length() >= 1);
        debug_assert!(item.TextShapeResult().is_null());
        debug_assert_ne!(item.TextType(), TextItemType::kNormal);
        #[cfg(debug_assertions)]
        item.CheckTextType(text_content);
        if item.IsGeneratedForLineBreak() {
            return;
        }

        let layout_object = item.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        debug_assert!(unsafe { &*layout_object }.IsText());
        if !DisableLayoutSideEffectsScope::IsDisabled() {
            unsafe { &mut *layout_object }.ClearNeedsLayoutWithFullPaintInvalidation();
        }
        if item_result.Length() == 0 {
            return;
        }

        let box_ref = unsafe { &mut *box_state };
        if self.quirks_mode_ && !box_ref.HasMetrics() {
            box_ref.EnsureTextMetrics(
                unsafe { &*item.Style() },
                unsafe { &*box_ref.font.Get() },
                self.baseline_type_,
                TextFitBlockScale::kFixed,
            );
        }
        let shape_result = std::mem::take(&mut item_result.shape_result);
        let text_fit_scale = item_result.text_fit_scale.Get();
        line_box.AddChild(LogicalLineItem::from_text_fragment(
            item,
            shape_result.Get(),
            item_result.TextOffset(),
            box_ref.text_top,
            item_result.inline_size,
            box_ref.text_height,
            item.BidiLevel(),
            if text_fit_scale.is_null() {
                None
            } else {
                Some(unsafe { &*text_fit_scale })
            },
        ));
    }

    // cpp: layoutng_inline/logical_line_builder.h:84-87
    // cpp: layoutng_inline/logical_line_builder.cc:447-464
    fn PlaceHyphen(
        &mut self,
        item_result: &InlineItemResult,
        mut hyphen_inline_size: LayoutUnit,
        line_box: &mut LogicalLineItems,
        box_state: *mut InlineBoxState,
    ) {
        debug_assert!(!item_result.item.Get().is_null());
        debug_assert!(item_result.is_hyphenated);
        debug_assert!(item_result.hyphen.IsPresent());
        debug_assert_eq!(hyphen_inline_size, item_result.hyphen.InlineSize());
        let item = unsafe { &*item_result.item.Get() };
        let text_fit_scale = item_result.text_fit_scale.Get();
        if !text_fit_scale.is_null() {
            hyphen_inline_size *= unsafe { &*text_fit_scale }.scale;
        }
        let shape = ShapeResultView::CreateFromResult(item_result.hyphen.GetShapeResult());
        let box_ref = unsafe { &*box_state };
        line_box.AddChild(LogicalLineItem::generated_text(
            item,
            unsafe { &*shape },
            item_result.hyphen.Text(),
            text_fit_scale,
            box_ref.text_top,
            hyphen_inline_size,
            box_ref.text_height,
            item.BidiLevel(),
        ));
    }

    // cpp: layoutng_inline/logical_line_builder.h:88-92
    // cpp: layoutng_inline/logical_line_builder.cc:466-504
    fn PlaceAtomicInline(
        &mut self,
        item: &InlineItem,
        item_result: &mut InlineItemResult,
        line_box: &mut LogicalLineItems,
    ) -> *mut InlineBoxState {
        debug_assert!(!item_result.layout_result.Get().is_null());
        let layout_object = item.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        debug_assert!(unsafe { &*layout_object }.IsAtomicInline());
        debug_assert!(unsafe { &*To::<LayoutBox>(layout_object) }.IsMonolithic());
        unsafe { &mut *layout_object }.SetIsTruncated(false);

        let box_state = unsafe { &mut *self.box_states_ }.OnOpenTag(
            unsafe { &*self.constraint_space_ },
            item,
            item_result,
            self.baseline_type_,
            line_box,
        );
        if !IsA::<LayoutTextCombine>(layout_object) {
            let box_ref = unsafe { &*box_state };
            let inline_offset = box_ref.margins.inline_start + item_result.spacing_before;
            self.PlaceLayoutResult(item_result, line_box, box_state, inline_offset);
        } else {
            let parent = unsafe { &*layout_object }.Parent();
            let style = unsafe { &*parent }.StyleRef();
            let box_ref = unsafe { &mut *box_state };
            box_ref.ComputeTextMetrics(
                style,
                unsafe { &*style.GetFont() },
                self.baseline_type_,
                TextFitBlockScale::kFixed,
            );
            let inline_offset = box_ref.margins.inline_start + item_result.spacing_before;
            let layout_result = std::mem::take(&mut item_result.layout_result);
            line_box.AddChild(LogicalLineItem::from_layout_result_offset(
                layout_result.Get(),
                LogicalOffset::new(inline_offset, box_ref.text_top),
                item_result.inline_size,
                0,
                item.BidiLevel(),
            ));
        }
        unsafe { &mut *self.box_states_ }.OnCloseTag(
            unsafe { &*self.constraint_space_ },
            line_box,
            box_state,
            self.baseline_type_,
        )
    }

    // cpp: layoutng_inline/logical_line_builder.h:97-100
    // cpp: layoutng_inline/logical_line_builder.cc:506-528
    fn PlaceLayoutResult(
        &mut self,
        item_result: &mut InlineItemResult,
        line_box: &mut LogicalLineItems,
        box_state: *mut InlineBoxState,
        inline_offset: LayoutUnit,
    ) {
        debug_assert!(!item_result.layout_result.Get().is_null());
        let item = unsafe { &*item_result.item.Get() };
        debug_assert!(!item.Style().is_null());
        let fragment = unsafe { &*item_result.layout_result.Get() }.GetPhysicalFragment();
        let physical_box = unsafe { &*To::<PhysicalBoxFragment>(fragment as *const _) };
        let metrics = LogicalBoxFragment::new(
            unsafe { &*self.constraint_space_ }.GetWritingDirection(),
            physical_box,
        )
        .BaselineMetrics(&item_result.margins, self.baseline_type_);
        if !box_state.is_null() {
            unsafe { &mut *box_state }.metrics.Unite(&metrics);
        }

        let line_top = item_result.margins.line_over - metrics.ascent;
        let layout_result = std::mem::take(&mut item_result.layout_result);
        line_box.AddChild(LogicalLineItem::from_layout_result_offset(
            layout_result.Get(),
            LogicalOffset::new(inline_offset, line_top),
            item_result.inline_size,
            0,
            item.BidiLevel(),
        ));
    }

    // cpp: layoutng_inline/logical_line_builder.h:93-96
    // cpp: layoutng_inline/logical_line_builder.cc:530-548
    fn PlaceInitialLetterBox(
        &mut self,
        item: &InlineItem,
        item_result: &mut InlineItemResult,
        line_box: &mut LogicalLineItems,
    ) {
        debug_assert!(!item_result.layout_result.Get().is_null());
        debug_assert!(!IsA::<LayoutTextCombine>(item.GetLayoutObject()));
        debug_assert_eq!(item_result.spacing_before, LayoutUnit::default());
        let layout_result = std::mem::take(&mut item_result.layout_result);
        line_box.AddChild(LogicalLineItem::from_layout_result_offset(
            layout_result.Get(),
            LogicalOffset::new(item_result.margins.inline_start, LayoutUnit::default()),
            item_result.inline_size,
            0,
            item.BidiLevel(),
        ));
    }

    // cpp: layoutng_inline/logical_line_builder.h:101-102
    // cpp: layoutng_inline/logical_line_builder.cc:550-635
    fn PlaceRubyColumn(
        &mut self,
        line_info: &LineInfo,
        item_result: &mut InlineItemResult,
        line_box: &mut LogicalLineItems,
        mut box_state: *mut InlineBoxState,
    ) -> *mut InlineBoxState {
        let ruby_column_ptr = item_result.ruby_column.Get();
        let ruby_column = unsafe { &mut *ruby_column_ptr };
        let mut on_start_edge = false;
        let mut on_end_edge = false;
        let mut line_available_size = None;
        if !self.node_.IsBidiEnabled()
            && !line_info.IsRubyBase()
            && !line_info.IsRubyText()
            && (line_info.TextAlign() == ETextAlign::kJustify
                || (line_info.IsLastLine()
                    && line_info.LineStyle().GetTextAlignForLine(false) == ETextAlign::kJustify))
        {
            on_start_edge =
                ruby_column.base_line.InflowStartOffset() == line_info.InflowStartOffset();
            if line_info.TextAlign() == ETextAlign::kJustify {
                let end_text_offset = ruby_column.base_line.EndTextOffset();
                let inflow_end = line_info.InflowEndOffsetWithoutForcedBreak();
                on_end_edge = end_text_offset == inflow_end;
                if on_start_edge
                    && on_end_edge
                    && item_result.inline_size > ruby_column.base_line.Width()
                {
                    line_available_size = Some(line_info.AvailableWidth());
                }
            }
        }
        let base_insets = ApplyRubyAlign(
            line_available_size.unwrap_or(item_result.inline_size - item_result.spacing_before),
            on_start_edge,
            on_end_edge,
            &mut ruby_column.base_line,
        );

        let start_index = line_box.size();
        let ruby_column_start_index = unsafe { &mut *self.box_states_ }.RubyColumnList().size();
        for &position in &ruby_column.position_list {
            let logical_column = unsafe { &mut *self.box_states_ }.CreateRubyColumn();
            logical_column.start_index = start_index;
            logical_column.ruby_position = position;
        }

        box_state = self.HandleItemResults(
            line_info,
            ruby_column.base_line.MutableResults(),
            line_box,
            std::ptr::null_mut(),
            box_state,
        );
        if start_index == line_box.size() && self.node_.IsBidiEnabled() {
            let item = unsafe { &*item_result.item.Get() };
            line_box.AddChild(LogicalLineItem::bidi_control(item.BidiLevel()));
        }
        let column_base_size = line_box.size() - start_index;

        for index in 0..ruby_column.annotation_line_list.size() {
            let logical_column = unsafe {
                &mut *(&mut *self.box_states_).RubyColumnList()
                    [(ruby_column_start_index + index) as usize]
                    .Get()
            };
            let annotation_line = &mut ruby_column.annotation_line_list[index as usize];
            if !annotation_line.IsEmptyLine() {
                if !line_box[start_index as usize].has_over_annotation
                    && logical_column.ruby_position == RubyPosition::kOver
                {
                    for child_index in start_index..line_box.size() {
                        line_box[child_index as usize].has_over_annotation = true;
                    }
                }
                if !line_box[start_index as usize].has_under_annotation
                    && logical_column.ruby_position == RubyPosition::kUnder
                {
                    for child_index in start_index..line_box.size() {
                        line_box[child_index as usize].has_under_annotation = true;
                    }
                }
            }
            if index == 0 {
                logical_column.base_insets = base_insets;
                logical_column.base_insets.0 += item_result.spacing_before;
            }
            logical_column.size = column_base_size;
            self.PlaceRubyAnnotation(
                item_result,
                line_available_size,
                index,
                annotation_line,
                logical_column,
            );
        }
        box_state
    }

    // cpp: layoutng_inline/logical_line_builder.h:103
    // cpp: layoutng_inline/logical_line_builder.cc:637-675
    fn PlaceRubyAnnotation(
        &mut self,
        item_result: &mut InlineItemResult,
        line_available_size: Option<LayoutUnit>,
        index: u32,
        annotation_line: &mut LineInfo,
        logical_column: &mut LogicalRubyColumn,
    ) {
        let ruby_column = unsafe { &*item_result.ruby_column.Get() };
        let insets = ApplyRubyAlign(
            line_available_size.unwrap_or(
                item_result.inline_size
                    - ruby_column.last_base_glyph_spacing
                    - item_result.spacing_before,
            ),
            false,
            false,
            annotation_line,
        );

        let line_items = MakeGarbageCollected(LogicalLineItems::default());
        let mut annotation_builder = LogicalLineBuilder::new(
            self.node_.clone(),
            unsafe { &*self.constraint_space_ },
            std::ptr::null(),
            &mut logical_column.state_stack,
            self.context_,
            false,
        );
        if ruby_column.is_continuation && !annotation_line.Results().is_empty() {
            let ruby_data = unsafe { &*self.break_token_ }.RubyData();
            assert!(!ruby_data.is_null());
            annotation_builder.RebuildBoxStates(
                annotation_line,
                unsafe { &*ruby_data }.annotation_data[index as usize].start_item_index,
                annotation_line.Results()[0].item_index,
            );
        }
        annotation_builder.CreateLine(
            annotation_line,
            unsafe { &mut *line_items },
            std::ptr::null_mut(),
        );
        ApplyLeftAndRightExpansion(insets.0, insets.1, unsafe { &mut *line_items }.AsSpan());
        logical_column.state_stack.ComputeInlinePositions(
            line_items,
            item_result.spacing_before,
            false,
        );
        logical_column.annotation_items = Member::from_ptr(line_items);
    }

    // cpp: layoutng_inline/logical_line_builder.h:108-110
    // cpp: layoutng_inline/logical_line_builder.cc:688-788
    fn BidiReorder(
        &mut self,
        base_direction: TextDirection,
        line_box: &mut LogicalLineItems,
        column_list: &mut HeapVector<Member<LogicalRubyColumn>>,
    ) {
        if line_box.IsEmpty() {
            return;
        }
        const OPAQUE_BIDI_LEVEL: UBiDiLevel = 0xff;
        let base_direction_level: UBiDiLevel = if IsLtr(base_direction) { 0 } else { 1 };
        let mut levels = Vec::with_capacity(line_box.size() as usize);
        let mut has_opaque_items = false;
        for item in line_box.iter() {
            if item.IsOpaqueToBidiReordering() {
                levels.push(OPAQUE_BIDI_LEVEL);
                has_opaque_items = true;
                continue;
            }
            debug_assert_ne!(item.bidi_level, OPAQUE_BIDI_LEVEL);
            if item.has_only_bidi_trailing_spaces {
                levels.push(base_direction_level);
            } else {
                levels.push(item.bidi_level);
            }
        }
        debug_assert_eq!(line_box.size() as usize, levels.len());

        if has_opaque_items {
            let mut last_level = base_direction_level;
            for level in levels.iter_mut().rev() {
                if *level == OPAQUE_BIDI_LEVEL {
                    *level = last_level;
                } else {
                    last_level = *level;
                }
            }
        }

        let mut indices_in_visual_order = vec![0i32; levels.len()];
        BidiParagraph::IndicesInVisualOrder(&levels, &mut indices_in_visual_order);
        let visual_items = unsafe { &mut *self.context_ }.AcquireTempLogicalLineItems();
        visual_items.ReserveInitialCapacity(line_box.size());
        for logical_index in &indices_in_visual_order {
            visual_items.AddChild(std::mem::take(&mut line_box[*logical_index as usize]));
        }
        debug_assert_eq!(line_box.size(), visual_items.size());
        line_box.swap(visual_items);
        unsafe { &mut *self.context_ }.ReleaseTempLogicalLineItems(visual_items);

        if !column_list.is_empty() {
            let mut logical_to_visual = vec![0u32; line_box.size() as usize];
            for (visual_index, logical_index) in indices_in_visual_order.iter().enumerate() {
                logical_to_visual[*logical_index as usize] = visual_index as u32;
            }
            for column in column_list.iter_mut() {
                let column = unsafe { &mut *column.Get() };
                let start = column.start_index as usize;
                let end = column.EndIndex() as usize;
                column.start_index = *logical_to_visual[start..end].iter().min().unwrap();
            }
            column_list.sort_by(|left, right| {
                let left = unsafe { &*left.Get() };
                let right = unsafe { &*right.Get() };
                left.start_index
                    .cmp(&right.start_index)
                    .then_with(|| right.size.cmp(&left.size))
            });
        }
    }

    // cpp: layoutng_inline/logical_line_builder.h:104
    // cpp: layoutng_inline/logical_line_builder.cc:678-686
    fn PlaceListMarker(&mut self, item: &InlineItem, _item_result: &mut InlineItemResult) {
        if self.quirks_mode_ {
            let style = unsafe { &*item.Style() };
            unsafe { &mut *self.box_states_ }
                .LineBoxState()
                .EnsureTextMetrics(
                    style,
                    unsafe { &*style.GetFont() },
                    self.baseline_type_,
                    TextFitBlockScale::kFixed,
                );
        }
    }

    // cpp: layoutng_inline/logical_line_builder.h:58-63
    // cpp: layoutng_inline/logical_line_builder.cc:200-353
    fn HandleItemResults(
        &mut self,
        line_info: &LineInfo,
        line_items: &mut InlineItemResults,
        line_box: &mut LogicalLineItems,
        main_line_helper: *mut InlineLayoutAlgorithm,
        mut box_state: *mut InlineBoxState,
    ) -> *mut InlineBoxState {
        let line_items_ptr = line_items as *mut InlineItemResults;
        for index in 0..line_items.len() {
            let item_result = unsafe { &mut (&mut *line_items_ptr)[index] };
            debug_assert!(!item_result.item.Get().is_null());
            let item = unsafe { &*item_result.item.Get() };
            if item.Type() == InlineItemType::kText {
                let object = item.GetLayoutObject();
                debug_assert!(!object.is_null());
                debug_assert!(
                    unsafe { &*object }.IsText() || unsafe { &*object }.IsLayoutListItem()
                );
                if item_result.Length() == 0 {
                    if unsafe { &*object }.NeedsLayout() {
                        unsafe { &mut *object }.ClearNeedsLayout();
                    }
                    continue;
                }
                debug_assert!(!item_result.shape_result.Get().is_null());

                let mut scale = 1.0f32;
                let mut scaled_font = std::ptr::null();
                let text_fit_scale = item_result.text_fit_scale.Get();
                if !text_fit_scale.is_null() {
                    scale = unsafe { &*text_fit_scale }.scale;
                    scaled_font = unsafe { &*text_fit_scale }.font.Get();
                }
                let box_ref = unsafe { &mut *box_state };
                if self.quirks_mode_ {
                    let text_fit_block_scale = TextFitBlockScale {
                        paint_scale: scale,
                        scaled_font,
                    };
                    box_ref.EnsureTextMetrics(
                        unsafe { &*item.Style() },
                        unsafe { &*box_ref.font.Get() },
                        self.baseline_type_,
                        &text_fit_block_scale,
                    );
                }
                if box_ref.include_used_fonts {
                    box_ref.AccumulateUsedFontsAtScale(item_result.shape_result.Get(), scale);
                }

                debug_assert!(matches!(
                    item.TextType(),
                    TextItemType::kNormal | TextItemType::kSymbolMarker
                ));
                if item_result.is_hyphenated {
                    debug_assert!(item_result.hyphen.IsPresent());
                    let hyphen_inline_size = item_result.hyphen.InlineSize();
                    let text_offset = item_result.text_offset;
                    let inline_size = LayoutUnit::FromFloatRound(
                        (item_result.inline_size - hyphen_inline_size).ToFloat() * scale,
                    );
                    line_box.AddChild(LogicalLineItem::from_item_result(
                        item,
                        item_result,
                        &text_offset,
                        box_ref.text_top,
                        inline_size,
                        box_ref.text_height,
                        item.BidiLevel(),
                    ));
                    self.PlaceHyphen(item_result, hyphen_inline_size, line_box, box_state);
                } else if self.node_.IsTextCombine() {
                    let one_em = unsafe { &*item.Style() }.ComputedFontSizeAsFixedValue();
                    let text_offset = item_result.text_offset;
                    let inline_size = item_result.inline_size;
                    line_box.AddChild(LogicalLineItem::from_item_result(
                        item,
                        item_result,
                        &text_offset,
                        LayoutUnit::default(),
                        inline_size,
                        one_em,
                        item.BidiLevel(),
                    ));
                } else {
                    let text_offset = item_result.text_offset;
                    let inline_size =
                        LayoutUnit::FromFloatRound(item_result.inline_size.ToFloat() * scale);
                    line_box.AddChild(LogicalLineItem::from_item_result(
                        item,
                        item_result,
                        &text_offset,
                        box_ref.text_top,
                        inline_size,
                        box_ref.text_height,
                        item.BidiLevel(),
                    ));
                }
                unsafe { &mut *object }.ClearNeedsLayoutWithFullPaintInvalidation();
            } else if item.Type() == InlineItemType::kControl {
                self.PlaceControlItem(
                    item,
                    &line_info.ItemsData().text_content,
                    item_result,
                    line_box,
                    box_state,
                );
            } else if item.Type() == InlineItemType::kOpenTag {
                let text_scale = FindTextScale(
                    self.should_scale_line_height_,
                    unsafe { &*line_items_ptr },
                    index as u32 + 1,
                    0,
                );
                box_state = self.HandleOpenTag(item, item_result, &text_scale, line_box);
            } else if item.Type() == InlineItemType::kCloseTag {
                box_state = self.HandleCloseTag(item, item_result, line_box, box_state);
            } else if item.Type() == InlineItemType::kAtomicInline {
                box_state = self.PlaceAtomicInline(item, item_result, line_box);
                self.has_relative_positioned_items_ |=
                    unsafe { &*item.Style() }.GetPosition() == EPosition::kRelative;
            } else if item.Type() == InlineItemType::kBlockInInline {
                debug_assert!(line_info.IsBlockInInline());
                debug_assert!(!main_line_helper.is_null());
                unsafe { &mut *main_line_helper }.PlaceBlockInInline(item, item_result, line_box);
            } else if item.Type() == InlineItemType::kOpenRubyColumn {
                if !item_result.ruby_column.Get().is_null() {
                    box_state = self.PlaceRubyColumn(line_info, item_result, line_box, box_state);
                } else {
                    line_box.AddChild(LogicalLineItem::bidi_control(item.BidiLevel()));
                }
            } else if item.Type() == InlineItemType::kCloseRubyColumn {
                line_box.AddChild(LogicalLineItem::bidi_control(item.BidiLevel()));
            } else if item.Type() == InlineItemType::kRubyLinePlaceholder {
                let start_overhang = item_result.margins.inline_start;
                let end_overhang = item_result.margins.inline_end;
                let text_offset = item_result.text_offset;
                let inline_size = item_result.inline_size + start_overhang + end_overhang;
                line_box.AddChild(LogicalLineItem::from_item_result(
                    item,
                    item_result,
                    &text_offset,
                    LayoutUnit::default(),
                    inline_size,
                    LayoutUnit::default(),
                    item.BidiLevel(),
                ));
                let last_index = line_box.size() as usize - 1;
                line_box[last_index].rect.offset.inline_offset = start_overhang;
            } else if item.Type() == InlineItemType::kListMarker {
                self.PlaceListMarker(item, item_result);
            } else if item.Type() == InlineItemType::kOutOfFlowPositioned {
                let object = item.GetLayoutObject();
                let space = unsafe { &*self.constraint_space_ };
                let writing_direction =
                    if unsafe { &*object }.StyleRef().IsOriginalDisplayInlineType() {
                        WritingDirectionMode::new(space.GetWritingMode(), item.Direction())
                    } else {
                        space.GetWritingDirection()
                    };
                line_box.AddChild(LogicalLineItem::OutOfFlowPositioned(
                    item,
                    writing_direction,
                ));
                self.has_out_of_flow_positioned_items_ = true;
            } else if item.Type() == InlineItemType::kFloating {
                if item_result.positioned_float.HasValue() {
                    let positioned = unsafe { &*item_result.positioned_float.Get() };
                    if positioned.break_before_token.Get().is_null() {
                        line_box.AddChild(LogicalLineItem::PositionedFloat(
                            item,
                            &item_result.positioned_float,
                        ));
                    }
                } else {
                    line_box.AddChild(LogicalLineItem::UnpositionedFloat(
                        item,
                        item_result.Start(),
                    ));
                }
                self.has_floating_items_ = true;
                self.has_relative_positioned_items_ |=
                    unsafe { &*item.Style() }.GetPosition() == EPosition::kRelative;
            } else if item.Type() == InlineItemType::kBidiControl {
                line_box.AddChild(LogicalLineItem::bidi_control(item.BidiLevel()));
            } else if item.Type() == InlineItemType::kInitialLetterBox {
                debug_assert!(self.initial_letter_item_result_.is_null());
                self.initial_letter_item_result_ = item_result;
                self.PlaceInitialLetterBox(item, item_result, line_box);
            }
        }
        box_state
    }

    // cpp: layoutng_inline/logical_line_builder.h:38-40
    // cpp: layoutng_inline/logical_line_builder.cc:790-813
    pub fn RebuildBoxStates(
        &mut self,
        line_info: &LineInfo,
        start_item_index: u32,
        end_item_index: u32,
    ) {
        let mut open_items = OpenTagItems::default();
        line_info.ItemsData().GetOpenTagItems(
            start_item_index,
            end_item_index - start_item_index,
            &mut open_items,
        );

        let line_box = unsafe { &mut *self.context_ }.AcquireTempLogicalLineItems();
        unsafe { &mut *self.box_states_ }.OnBeginPlaceItems(
            &self.node_,
            line_info,
            self.baseline_type_,
            self.quirks_mode_,
            self.should_scale_line_height_,
            line_box,
        );
        for (index, item) in open_items.iter().enumerate() {
            let item = unsafe { &*item.Get() };
            let mut item_result = InlineItemResult::default();
            LineBreaker::ComputeOpenTagResult(
                item,
                unsafe { &*self.constraint_space_ },
                self.node_.IsSvgText(),
                &mut item_result,
            );
            let text_scale = FindTextScale(
                self.should_scale_line_height_,
                line_info.Results(),
                0,
                open_items.size() - index as u32 - 1,
            );
            self.HandleOpenTag(item, &item_result, &text_scale, line_box);
        }
        unsafe { &mut *self.context_ }.ReleaseTempLogicalLineItems(line_box);
    }
}
