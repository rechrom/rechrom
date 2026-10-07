// C++: layoutng_inline/line_breaker.h/.cc. Source-owned line breaking,
// candidate collection, Ruby, SVG segmentation, and float handling are mapped
// here. Shared inline assembly and external owner interfaces remain pending.
#![allow(non_snake_case)]

use font_engine::fonts::canvas_rotation_in_vertical::CanvasRotationInVertical;
use font_engine::fonts::shaping::harfbuzz_shaper::HarfBuzzShaper;
use font_engine::fonts::shaping::shape_options::ShapeOptions;
use font_engine::fonts::shaping::shape_result::ShapeResult;
use font_engine::fonts::shaping::shape_result_spacing::ShapeResultSpacing;
use font_engine::fonts::shaping::shape_result_view::{Segment, ShapeResultView};
use font_engine::fonts::shaping::shaping_line_breaker::{
    Result as ShapingBreakResult, ShapeLineBackend, ShapingLineBreaker,
};
use font_engine::fonts::simple_font_data::SimpleFontData;
use font_engine::text::native::bidi_paragraph::BidiParagraph;
use font_engine::text::native::character::Character;
use font_engine::text::native::hyphenation::Hyphenation;
use font_engine::text::native::layout_locale::LineBreakStrictness;
use font_engine::text::native::text_break_iterator::{
    BreakSpaceType, LazyLineBreakIterator, LineBreakType,
};
use foundation::{
    kIndefiniteSize, kNotFound, DynamicTo, EBaselineSource, EBoxDecorationBreak, EOverflowWrap,
    ETextOrientation, EWordBreak, HeapVector, Hyphens, IsA, IsLtr, LayoutUnit, LineBreak,
    MakeGarbageCollected, Member, MinimumValueForLength, RubyPosition, RuntimeEnabledFeatures,
    String, StringBuilder, StringView, TextDirection, To, UnicodeBidi,
};
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::{BaselineAlgorithmType, ConstraintSpace};
use layoutng::internal::constraint_space_builder_style::MinMaxConstraintSpaceBuilder;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::exclusions::line_layout_opportunity::LineLayoutOpportunity;
use layoutng::internal::fragmentation_utils::FollowColumnSpannerPath;
use layoutng::internal::hyphen_result::HyphenResult;
use layoutng::internal::inline_item::{CollapseType, InlineItem, InlineItemType, InlineItems};
use layoutng::internal::inline_item_result::{InlineItemResult, InlineItemResults};
use layoutng::internal::inline_item_segment::InlineItemSegment;
use layoutng::internal::inline_item_text_index::InlineItemTextIndex;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng::internal::layout_node_metadata::Element;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng::internal::length_utils::{
    ComputeBordersForInline, ComputeLineBorders, ComputeLineMarginsForSelf,
    ComputeLineMarginsForVisualContainer, ComputeLinePadding, ComputeMarginsForSelf,
    ComputeMinAndMaxContentContribution, ComputePadding,
};
use layoutng::internal::resolved_text_layout_attributes_iterator::ResolvedTextLayoutAttributesIterator;
use layoutng::internal::svg_length_adjust_type::SVGLengthAdjustType;
use layoutng::internal::text_item_type::TextItemType;
use layoutng::internal::text_offset_range::TextOffsetRange;
use layoutng::internal::unpositioned_float::UnpositionedFloat;
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::fragment_item::ItemType as FragmentItemType;
use layoutng_fragment_tree::inline_break_token::{
    AnnotationBreakTokenData, InlineBreakToken, InlineBreakTokenFlag, RubyBreakTokenData,
};
use layoutng_fragment_tree::inline_cursor::InlineCursor;
use layoutng_fragment_tree::inline_items_data::{InlineItemsData, OpenTagItems};
use layoutng_fragment_tree::layout_result::EStatus;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::box_strut::LineBoxStrut;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;
use crate::leading_floats::LeadingFloats;
use crate::line_break_candidate::{
    CandidateLineBreaker, LineBreakCandidateContext, State as CandidateState,
};
use crate::line_break_point::LineBreakPoint;
use crate::line_info::LineInfo;
use crate::ruby_utils::ParseRubyInInlineItems;

// StringImpl storage width is owned by //src/foundation. Keep the source
// Is8Bit branch as a typed dependency until that owner exposes the flag.
unsafe extern "Rust" {
    fn StringIs8BitForInline(text: &String) -> bool;
}

impl CandidateLineBreaker for LineBreaker {
    fn AppendCandidates(
        &mut self,
        item_result: &InlineItemResult,
        line_info: &LineInfo,
        context: &mut LineBreakCandidateContext<'_>,
    ) {
        LineBreaker::AppendCandidates(self, item_result, line_info, context);
    }
}

// cpp: layoutng_inline/line_breaker.h:39-39
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineBreakerMode {
    kContent,
    kMinContent,
    kMaxContent,
}

// cpp: layoutng_inline/line_breaker.cc:60-75
fn StrictnessFromLineBreak(line_break: LineBreak) -> LineBreakStrictness {
    match line_break {
        LineBreak::kAuto | LineBreak::kAfterWhiteSpace | LineBreak::kAnywhere => {
            LineBreakStrictness::kDefault
        }
        LineBreak::kNormal => LineBreakStrictness::kNormal,
        LineBreak::kStrict => LineBreakStrictness::kStrict,
        LineBreak::kLoose => LineBreakStrictness::kLoose,
    }
}

// cpp: layoutng_inline/line_breaker.cc:47-58
fn ShouldApplyTextIndent(
    style: &ComputedStyle,
    is_first_formatted_line: bool,
    is_after_forced_break: bool,
) -> bool {
    if style.TextIndent().IsZero() {
        return false;
    }
    let is_first_line =
        is_first_formatted_line || (is_after_forced_break && style.IsTextIndentEachLine());
    if !style.IsTextIndentHanging() {
        is_first_line
    } else {
        !is_first_line
    }
}

// cpp: layoutng_inline/line_breaker.cc:236-247
fn CanBreakAfterLast(item_results: &InlineItemResults) -> bool {
    !item_results.empty() && item_results.last().unwrap().can_break_after
}

// cpp: layoutng_inline/line_breaker.cc:318-330
fn CollectCharIndex(
    context: *mut std::ffi::c_void,
    char_index: u32,
    _glyph: u16,
    _offset: foundation::gfx::Vector2dF,
    _advance: f32,
    _is_horizontal: bool,
    _rotation: CanvasRotationInVertical,
    _font: *const SimpleFontData,
) {
    let index_list = unsafe { &mut *(context as *mut Vec<u32>) };
    if index_list.last().copied() == Some(char_index) {
        return;
    }
    index_list.push(char_index);
}

// cpp: layoutng_inline/line_breaker.cc:214-227
fn HyphenAdvance(
    style: &ComputedStyle,
    is_ltr: bool,
    hyphen_result: &HyphenResult,
    cache: &mut Option<LayoutUnit>,
) -> LayoutUnit {
    if let Some(advance) = cache {
        return *advance;
    }
    let size = if hyphen_result.IsPresent() {
        hyphen_result.InlineSize()
    } else {
        HyphenResult::new(style).InlineSize()
    };
    let advance = if is_ltr { size } else { -size };
    *cache = Some(advance);
    advance
}

// C++ nests this stack-only adapter inside AppendCandidates. Rust lifts it to
// the module, retaining the external position coordinate system.
// cpp: layoutng_inline/line_breaker.cc:2099-2141
struct CandidateShapeResult<'a> {
    shape_result: &'a ShapeResult,
    shape_result_start_index: u32,
    base_position: f32,
    is_ltr: bool,
}

impl<'a> CandidateShapeResult<'a> {
    fn new(shape_result: &'a ShapeResult) -> Self {
        shape_result.EnsurePositionData(true);
        Self {
            shape_result,
            shape_result_start_index: shape_result.StartIndex(),
            base_position: 0.0,
            is_ltr: shape_result.IsLtr(),
        }
    }
    fn IsLtr(&self) -> bool {
        self.is_ltr
    }
    fn PositionForOffset(&self, offset: u32) -> f32 {
        debug_assert!(offset >= self.shape_result_start_index);
        let position = self
            .shape_result
            .CachedPositionForOffset(offset - self.shape_result_start_index)
            .ToFloat();
        if self.IsLtr() {
            self.base_position + position
        } else {
            self.base_position - position
        }
    }
    fn SetBasePosition(&mut self, offset: u32, adjusted: f32) {
        debug_assert!(offset >= self.shape_result_start_index);
        let position = self
            .shape_result
            .CachedPositionForOffset(offset - self.shape_result_start_index)
            .ToFloat();
        self.base_position = if self.IsLtr() {
            adjusted - position
        } else {
            adjusted + position
        };
        debug_assert_eq!(adjusted, self.PositionForOffset(offset));
    }
    fn PreviousSafeToBreakOffset(&self, offset: u32) -> u32 {
        self.shape_result.CachedPreviousSafeToBreakOffset(offset)
    }
}

// cpp: layoutng_inline/line_breaker.cc:228-234
fn IsTrailableItemType(item_type: InlineItemType) -> bool {
    item_type != InlineItemType::kAtomicInline
        && item_type != InlineItemType::kOutOfFlowPositioned
        && item_type != InlineItemType::kInitialLetterBox
        && item_type != InlineItemType::kListMarker
        && item_type != InlineItemType::kOpenRubyColumn
}

// cpp: layoutng_inline/line_breaker.cc:78-171
fn ComputeNegativeSideBearings(box_fragment: &PhysicalBoxFragment) -> LineBoxStrut {
    let get_shape_result = |cursor: &InlineCursor| -> *const ShapeResultView {
        if !cursor.IsNotNull() {
            return std::ptr::null();
        }
        let item = unsafe { &*cursor.CurrentItem() };
        if item.Type() != FragmentItemType::kText && item.Type() != FragmentItemType::kGeneratedText
        {
            return std::ptr::null();
        }
        if item.IsFlowControl() {
            return std::ptr::null();
        }
        item.TextShapeResult()
    };
    let mut side_bearing = LineBoxStrut::default();
    let mut cursor = InlineCursor::from_fragment(box_fragment);
    while cursor.IsNotNull() {
        debug_assert!(cursor.Current().IsLineBox());
        let mut left_child = cursor.clone();
        left_child.MoveToFirstChild();
        let left_shape = get_shape_result(&left_child);
        if !left_shape.is_null() {
            let ink_bounds = unsafe { &*left_shape }.ComputeInkBounds();
            let left_bearing = LogicalRect::EnclosingRect(&ink_bounds).offset.inline_offset;
            side_bearing.inline_start = side_bearing.inline_start.min(left_bearing);
        }

        let mut right_child = cursor.clone();
        right_child.MoveToLastChild();
        let right_shape = get_shape_result(&right_child);
        if !right_shape.is_null() {
            let shape_result = unsafe { &*right_shape };
            let width = shape_result.SnappedWidth();
            let ink_bounds = LogicalRect::EnclosingRect(&shape_result.ComputeInkBounds());
            let right_bearing = width - ink_bounds.InlineEndOffset();
            side_bearing.inline_end = side_bearing.inline_end.min(right_bearing);
        }
        cursor.MoveToNextLine();
    }
    side_bearing
}

// cpp: layoutng_inline/line_breaker.cc:177-183
fn ShouldApplyInlineKerning(box_fragment: &PhysicalBoxFragment) -> bool {
    if !box_fragment.Borders().IsZero() || !box_fragment.Padding().IsZero() {
        return false;
    }
    let style = box_fragment.Style();
    style.IsHorizontalWritingMode() || style.GetTextOrientation() == ETextOrientation::kSideways
}

// cpp: layoutng_inline/line_breaker.cc:185-203
fn IsBreakableSpace(character: u16) -> bool {
    character == 0x20 || character == 0x09
}

// cpp: layoutng_inline/line_breaker.cc:193-195
fn IsBreakableSpaceOrOtherSeparator(character: u16) -> bool {
    IsBreakableSpace(character) || Character::IsOtherSpaceSeparator(i32::from(character))
}

// unicode::Direction delegates to unicode_data::BidiClass in the source;
// ICU's U_WHITE_SPACE_NEUTRAL value is 9.
// cpp: layoutng_inline/line_breaker.cc:205-207
fn IsBidiTrailingSpace(character: u16) -> bool {
    foundation::unicode_data::BidiClass(u32::from(character)) == 9
}

fn IsAllBreakableSpaces(text: &String, start: u32, end: u32) -> bool {
    debug_assert!(end >= start);
    text.Span16().unwrap_or_default()[start as usize..end as usize]
        .iter()
        .all(|&character| IsBreakableSpace(character))
}

fn ShouldCreateLineBox(item_results: &InlineItemResults) -> bool {
    !item_results.empty() && item_results.last().unwrap().should_create_line_box
}

fn HasUnpositionedFloats(item_results: &InlineItemResults) -> bool {
    !item_results.empty() && item_results.last().unwrap().has_unpositioned_floats
}

// cpp: layoutng_inline/line_breaker.cc:347-418
#[derive(Default)]
struct FastMinTextContext {
    min_inline_size: LayoutUnit,
    hyphen_inline_size: Option<LayoutUnit>,
}

impl FastMinTextContext {
    fn MinInlineSize(&self) -> LayoutUnit {
        self.min_inline_size
    }

    fn HyphenInlineSize(&mut self, item_result: &mut InlineItemResult) -> LayoutUnit {
        if self.hyphen_inline_size.is_none() {
            if !item_result.hyphen.IsPresent() {
                item_result.ShapeHyphen();
            }
            self.hyphen_inline_size = Some(item_result.hyphen.InlineSize());
        }
        self.hyphen_inline_size.unwrap()
    }

    fn AddWidth(&mut self, width: LayoutUnit) {
        self.min_inline_size = self.min_inline_size.max(width);
    }

    fn Add(
        &mut self,
        shape_result: &ShapeResult,
        start_offset: u32,
        end_offset: u32,
        has_hyphen: bool,
        item_result: &mut InlineItemResult,
    ) {
        let mut width = shape_result.CachedWidth(start_offset, end_offset);
        if has_hyphen {
            width += self.HyphenInlineSize(item_result);
        }
        self.AddWidth(width);
    }

    fn AddHyphenated(
        &mut self,
        shape_result: &ShapeResult,
        start_offset: u32,
        mut end_offset: u32,
        mut has_hyphen: bool,
        item_result: &mut InlineItemResult,
        hyphenation: &Hyphenation,
        word: &String,
    ) {
        let mut locations = hyphenation.HyphenLocations(word);
        debug_assert_eq!(word.length(), end_offset - start_offset);
        debug_assert!(locations.windows(2).all(|pair| pair[0] > pair[1]));
        debug_assert!(!locations.contains(&0));
        debug_assert!(!locations.contains(&word.length()));
        locations.push(0);
        let hyphen_inline_size = self.HyphenInlineSize(item_result);
        let mut max_part_width = LayoutUnit::default();
        for location in locations {
            let part_start_offset = start_offset + location;
            let mut part_width = shape_result.CachedWidth(part_start_offset, end_offset);
            if has_hyphen {
                part_width += hyphen_inline_size;
            }
            max_part_width = max_part_width.max(part_width);
            end_offset = part_start_offset;
            has_hyphen = true;
        }
        self.AddWidth(max_part_width);
    }
}

// cpp: layoutng_inline/line_breaker.cc:249-259
fn ComputeInlineEndSize(space: &ConstraintSpace, style: *const ComputedStyle) -> LayoutUnit {
    debug_assert!(!style.is_null());
    let style = unsafe { &*style };
    let margins = ComputeMarginsForSelf(space, style);
    let borders = ComputeBordersForInline(style);
    let paddings = ComputePadding(space, style);
    margins.inline_end + borders.inline_end + paddings.inline_end
}

// C++ overloads retain separate names in Rust.
// cpp: layoutng_inline/line_breaker.cc:261-268
fn NeedsAccurateEndPositionForItem(line_end_item: &InlineItem) -> bool {
    debug_assert!(
        line_end_item.Type() == InlineItemType::kText
            || line_end_item.Type() == InlineItemType::kControl
    );
    debug_assert!(!line_end_item.Style().is_null());
    let style = unsafe { &*line_end_item.Style() };
    style.HasBoxDecorationBackground() || style.HasAppliedTextDecorations()
}

// cpp: layoutng_inline/line_breaker.cc:270-274
fn NeedsAccurateEndPositionForLine(line_info: &LineInfo, line_end_item: &InlineItem) -> bool {
    line_info.NeedsAccurateEndPosition() || NeedsAccurateEndPositionForItem(line_end_item)
}

// cpp: layoutng_inline/line_breaker.cc:276-280
fn ComputeCanBreakAfter(
    item_result: &mut InlineItemResult,
    auto_wrap: bool,
    break_iterator: &LazyLineBreakIterator,
) {
    item_result.can_break_after = auto_wrap && break_iterator.IsBreakable(item_result.EndOffset());
}

// cpp: layoutng_inline/line_breaker.cc:331-335
fn MayBeTextCombine(item: *const InlineItem) -> *mut LayoutTextCombine {
    if item.is_null() {
        return std::ptr::null_mut();
    }
    DynamicTo::<LayoutTextCombine>(unsafe { &*item }.GetLayoutObject())
}

// cpp: layoutng_inline/line_breaker.cc:337-345
fn MaxLineWidth(base_line: &LineInfo, annotation_lines: &[LineInfo]) -> LayoutUnit {
    let mut max_width = base_line.Width();
    for line in annotation_lines {
        max_width = max_width.max(line.Width());
    }
    max_width
}

// cpp: layoutng_inline/line_breaker.cc:1632-1655
struct ShapingLineBreakerImpl<'a> {
    shaper: *const HarfBuzzShaper,
    spacing: *mut ShapeResultSpacing,
    items_data: *const InlineItemsData,
    item: &'a InlineItem,
}

impl ShapeLineBackend for ShapingLineBreakerImpl<'_> {
    fn Shape(&self, start: u32, end: u32, options: ShapeOptions) -> *const ShapeResult {
        // ShapeLineBackend mirrors C++'s const virtual callback. Its calls are
        // sequential; spacing is the only mutable owner field used here.
        LineBreaker::ShapeTextFromParts(
            unsafe { &*self.shaper },
            unsafe { &mut *self.spacing },
            unsafe { &*self.items_data },
            self.item,
            start,
            end,
            options,
        )
    }
}

// cpp: layoutng_inline/line_breaker.cc:282-286
fn RemoveLastItem(line_info: &mut LineInfo) {
    let item_results = line_info.MutableResults();
    debug_assert!(item_results.size() > 0);
    item_results.Shrink(item_results.size() - 1);
}

// cpp: layoutng_inline/line_breaker.cc:290-314
fn ComputeFloatAncestorInlineEndSize(
    space: &ConstraintSpace,
    items: &InlineItems,
    item_index: u32,
) -> LayoutUnit {
    let mut inline_end_size = LayoutUnit::default();
    for item_ptr in &items[item_index as usize..] {
        let item = unsafe { &*item_ptr.Get() };
        if item.Type() == InlineItemType::kCloseTag {
            inline_end_size += ComputeInlineEndSize(space, item.Style());
            continue;
        }
        if item.Type() == InlineItemType::kOpenTag || !item.IsEmptyItem() {
            break;
        }
    }
    inline_end_size
}

// cpp: layoutng_inline/line_breaker.h:97-97
// The inline capacity is a storage optimization; the indexed value contract
// is preserved by Vec, and callers can reserve 64 elements if needed.
pub type MaxSizeCache = Vec<LayoutUnit>;

// cpp: layoutng_inline/line_breaker.h:106-114
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhitespaceState {
    kLeading,
    kNone,
    kUnknown,
    kCollapsible,
    kCollapsed,
    kPreserved,
}

// cpp: layoutng_inline/line_breaker.h:169-183
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LineBreakState {
    kDone,
    kOverflow,
    kTrailing,
    kContinue,
}

// cpp: layoutng_inline/line_breaker.h:194-194
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BreakResult {
    kSuccess,
    kOverflow,
    kBreakAt,
}

// cpp: layoutng_inline/line_breaker.h:441-461
struct TrailingCollapsibleSpace {
    item_results: *mut InlineItemResults,
    item_result_index: u32,
    collapsed_shape_result: *const ShapeResultView,
    ancestor_ruby_columns: Vec<(*mut InlineItemResults, u32)>,
}

impl TrailingCollapsibleSpace {
    // cpp: layoutng_inline/line_breaker.h:483-485
    fn ItemResult(&mut self) -> &mut InlineItemResult {
        unsafe { &mut (&mut *self.item_results)[self.item_result_index as usize] }
    }
}

// cpp: layoutng_inline/line_breaker.h:499-502
#[derive(Clone, Copy)]
struct RewindIndex {
    from_item_index: u32,
    to_index: u32,
}

// cpp: layoutng_inline/line_breaker.h:44-49,358-526
pub struct LineBreaker {
    state_: Option<LineBreakState>,
    current_: InlineItemTextIndex,
    svg_addressable_offset_: u32,
    break_at_: LineBreakPoint,
    trailing_whitespace_: WhitespaceState,
    initial_whitespace_: WhitespaceState,
    position_: LayoutUnit,
    tab_stop_offset_: LayoutUnit,
    applied_text_indent_: LayoutUnit,
    available_width_: LayoutUnit,
    base_available_width_: LayoutUnit,
    line_opportunity_: LineLayoutOpportunity,
    node_: InlineNode,
    mode_: LineBreakerMode,
    is_initial_letter_box_: bool,
    is_svg_text_: bool,
    is_text_combine_: bool,
    is_first_formatted_line_: bool,
    use_first_line_style_: bool,
    auto_wrap_: bool,
    disallow_auto_wrap_: bool,
    break_anywhere_if_overflow_: bool,
    override_break_anywhere_: bool,
    disable_phrase_: bool,
    disable_score_line_break_: bool,
    disable_bisect_line_break_: bool,
    disable_trailing_whitespace_collapsing_: bool,
    force_non_empty_if_last_line_: bool,
    is_forced_break_: bool,
    previous_line_had_forced_break_: bool,
    sticky_images_quirk_: bool,
    maybe_have_end_overhang_: bool,
    needs_svg_segmentation_: bool,
    resume_block_in_inline_in_same_flow_: bool,
    #[cfg(debug_assertions)]
    has_considered_creating_break_token_: bool,
    items_data_: *const InlineItemsData,
    end_item_index_: u32,
    text_content_: String,
    constraint_space_: *const ConstraintSpace,
    exclusion_space_: *mut ExclusionSpace,
    break_token_: *const InlineBreakToken,
    ruby_break_token_: *const RubyBreakTokenData,
    column_spanner_path_: *const ColumnSpannerPath,
    current_style_: *const ComputedStyle,
    break_iterator_: LazyLineBreakIterator,
    shaper_: HarfBuzzShaper,
    spacing_: ShapeResultSpacing,
    hyphenation_: *const Hyphenation,
    hyphen_index_: Option<u32>,
    has_any_hyphens_: bool,
    trailing_collapsible_space_: Option<TrailingCollapsibleSpace>,
    override_available_width_: LayoutUnit,
    leading_floats_: *const LeadingFloats,
    leading_floats_index_: u32,
    max_size_cache_: *mut MaxSizeCache,
    depends_on_block_constraints_out_: *mut bool,
    base_direction_: TextDirection,
    cloned_box_decorations_count_: u32,
    cloned_box_decorations_initial_size_: LayoutUnit,
    cloned_box_decorations_end_size_: LayoutUnit,
    has_cloned_box_decorations_: bool,
    last_rewind_: Option<RewindIndex>,
    // The iterator borrows GC-owned SVG character data, which outlives the
    // stack-allocated LineBreaker. It is released with this field on Drop.
    svg_resolved_iterator_: Option<Box<ResolvedTextLayoutAttributesIterator<'static>>>,
    line_clamp_ellipsis_width_: LayoutUnit,
    parent_breaker_: *const LineBreaker,
}

impl LineBreaker {
    // cpp: layoutng_inline/line_breaker.cc:3950-3969
    pub fn ComputeOpenTagResult(
        item: &InlineItem,
        constraint_space: &ConstraintSpace,
        is_in_svg_text: bool,
        item_result: &mut InlineItemResult,
    ) -> bool {
        debug_assert_eq!(item.Type(), InlineItemType::kOpenTag);
        debug_assert!(!item.Style().is_null());
        let style = unsafe { &*item.Style() };
        if !is_in_svg_text
            && item.ShouldCreateBoxFragment()
            && (style.HasBorder() || style.MayHavePadding() || style.MayHaveMargin())
        {
            item_result.borders = ComputeLineBorders(style);
            item_result.padding = ComputeLinePadding(constraint_space, style);
            item_result.margins = ComputeLineMarginsForSelf(constraint_space, style);
            item_result.inline_size = item_result.margins.inline_start
                + item_result.borders.inline_start
                + item_result.padding.inline_start;
            return true;
        }
        false
    }

    // C++ overloads are named separately; raw result pointers retain the
    // source's vector-owned result identity while the line grows.
    // cpp: layoutng_inline/line_breaker.cc:579-597
    fn AddItemWithEnd(
        &mut self,
        item: &InlineItem,
        end_offset: u32,
        line_info: &mut LineInfo,
    ) -> *mut InlineItemResult {
        if item.Type() != InlineItemType::kOpenRubyColumn {
            debug_assert!(std::ptr::eq(item, unsafe {
                &*self.ItemsData().items[self.current_.item_index as usize].Get()
            }));
            debug_assert!(self.current_.text_offset >= item.StartOffset());
            debug_assert!(end_offset >= self.current_.text_offset);
            debug_assert!(end_offset <= item.EndOffset());
        }
        if item.IsTextCombine() {
            line_info.SetHaveTextCombineOrRubyItem();
        }
        let item_results = line_info.MutableResults();
        let result = InlineItemResult::new(
            item,
            self.current_.item_index,
            &TextOffsetRange::new(self.current_.text_offset, end_offset),
            self.break_anywhere_if_overflow_,
            ShouldCreateLineBox(item_results),
            HasUnpositionedFloats(item_results),
        );
        item_results.emplace_back(result) as *mut InlineItemResult
    }

    // cpp: layoutng_inline/line_breaker.cc:599-603
    fn AddItem(&mut self, item: &InlineItem, line_info: &mut LineInfo) -> *mut InlineItemResult {
        self.AddItemWithEnd(item, item.EndOffset(), line_info)
    }

    // cpp: layoutng_inline/line_breaker.cc:605-621
    fn AddEmptyItem(
        &mut self,
        item: &InlineItem,
        line_info: &mut LineInfo,
    ) -> *mut InlineItemResult {
        let item_result = self.AddItemWithEnd(item, self.current_.text_offset, line_info);
        debug_assert!(!unsafe { &*item_result }.can_break_after);
        let item_results = line_info.MutableResults();
        if item_results.size() >= 2 {
            let last_index = item_results.size() as usize - 2;
            let last_item_result = &mut item_results[last_index];
            if last_item_result.can_break_after {
                last_item_result.can_break_after = false;
                unsafe { &mut *item_result }.can_break_after = true;
            }
        }
        item_result
    }

    // cpp: layoutng_inline/line_breaker.cc:1171-1211
    fn CanBreakAfterAtomicInline(&self, item: &InlineItem) -> bool {
        debug_assert!(
            item.Type() == InlineItemType::kAtomicInline
                || item.Type() == InlineItemType::kInitialLetterBox
        );
        if !self.auto_wrap_ {
            return false;
        }
        if item.EndOffset() == self.Text().length() {
            return true;
        }
        if item.IsImage() {
            return !self.sticky_images_quirk_;
        }

        let text_combine = MayBeTextCombine(item);
        if text_combine.is_null() {
            return true;
        }

        let mut text_content = StringBuilder::new();
        InlineNode::new(text_combine.cast::<LayoutBlockFlow>()).PrepareLayoutIfNeeded();
        text_content.AppendString(&unsafe { &*text_combine }.GetTextContent());
        let text_combine_end_offset = text_content.ToString().length();
        let atomic_inline_item = self.TryGetAtomicInlineItemAfter(item);
        let next_text_combine = MayBeTextCombine(atomic_inline_item);
        if !next_text_combine.is_null() {
            InlineNode::new(next_text_combine.cast::<LayoutBlockFlow>()).PrepareLayoutIfNeeded();
            text_content.AppendString(&unsafe { &*next_text_combine }.GetTextContent());
        } else {
            let tail = StringView::from(self.Text())
                .Substring(item.EndOffset(), self.Text().length() - item.EndOffset());
            text_content.AppendString(&tail.ToString());
        }

        debug_assert_eq!(self.Text(), self.break_iterator_.GetString());
        let break_iterator = LazyLineBreakIterator::with_replacement_string(
            &self.break_iterator_,
            text_content.ReleaseString(),
        );
        break_iterator.IsBreakable(text_combine_end_offset)
    }

    // cpp: layoutng_inline/line_breaker.cc:1213-1270
    fn CanBreakAfter(&self, item: &InlineItem) -> bool {
        debug_assert_ne!(item.Type(), InlineItemType::kAtomicInline);
        debug_assert!(self.auto_wrap_);
        let can_break_after = self.break_iterator_.IsBreakable(item.EndOffset());
        if item.Type() != InlineItemType::kText {
            debug_assert_eq!(item.Type(), InlineItemType::kControl);
            return can_break_after;
        }
        let ignorable_bidi_length = self.IgnorableBidiControlLength(item);
        if ignorable_bidi_length > 0 {
            return self
                .break_iterator_
                .IsBreakable(item.EndOffset() + ignorable_bidi_length);
        }
        let atomic_inline_item = self.TryGetAtomicInlineItemAfter(item);
        if atomic_inline_item.is_null() {
            return can_break_after;
        }
        if self.Text().Span16().unwrap_or_default()
            [unsafe { &*atomic_inline_item }.StartOffset() as usize]
            == 0x00a0
        {
            debug_assert!(unsafe { &*atomic_inline_item }.IsImage() && self.sticky_images_quirk_);
            return can_break_after;
        }
        let text_combine = MayBeTextCombine(atomic_inline_item);
        if text_combine.is_null() {
            return true;
        }

        let mut text_content = StringBuilder::new();
        let item_text = StringView::from(self.Text()).Substring(item.StartOffset(), item.Length());
        text_content.AppendString(&item_text.ToString());
        let item_end_offset = text_content.ToString().length();
        InlineNode::new(text_combine.cast::<LayoutBlockFlow>()).PrepareLayoutIfNeeded();
        text_content.AppendString(&unsafe { &*text_combine }.GetTextContent());
        debug_assert_eq!(self.Text(), self.break_iterator_.GetString());
        let break_iterator = LazyLineBreakIterator::with_replacement_string(
            &self.break_iterator_,
            text_content.ReleaseString(),
        );
        break_iterator.IsBreakable(item_end_offset)
    }

    // cpp: layoutng_inline/line_breaker.cc:1272-1279
    fn MayBeAtomicInline(&self, offset: u32) -> bool {
        debug_assert!(offset < self.Text().length());
        let char_code = self.Text().Span16().unwrap_or_default()[offset as usize];
        char_code == 0xfffc || (self.sticky_images_quirk_ && char_code == 0x00a0)
    }

    // cpp: layoutng_inline/line_breaker.cc:1281-1304
    fn TryGetAtomicInlineItemAfter(&self, item: &InlineItem) -> *const InlineItem {
        debug_assert!(self.auto_wrap_);
        if item.EndOffset() == self.Text().length() || !self.MayBeAtomicInline(item.EndOffset()) {
            return std::ptr::null();
        }
        for item_ptr in self.Items().iter().skip(item.Index() as usize + 1) {
            let next_item = unsafe { &*item_ptr.Get() };
            debug_assert_eq!(next_item.StartOffset(), item.EndOffset());
            if next_item.Type() == InlineItemType::kAtomicInline {
                return next_item;
            }
            if next_item.EndOffset() > item.EndOffset() {
                return std::ptr::null();
            }
        }
        std::ptr::null()
    }

    // cpp: layoutng_inline/line_breaker.cc:1306-1323
    fn IgnorableBidiControlLength(&self, item: &InlineItem) -> u32 {
        let start_item_index = item.Index() + 1;
        for item_ptr in self
            .Items()
            .iter()
            .skip(start_item_index as usize)
            .take((self.end_item_index_ - start_item_index) as usize)
        {
            let item_i = unsafe { &*item_ptr.Get() };
            if item_i.Length() == 0 {
                continue;
            }
            if item_i.Type() != InlineItemType::kOpenRubyColumn
                && item_i.Type() != InlineItemType::kCloseRubyColumn
            {
                return item_i.StartOffset() - item.EndOffset();
            }
        }
        let end_offset = if self.end_item_index_ >= self.Items().size() {
            self.Text().length()
        } else {
            unsafe { &*self.Items()[self.end_item_index_ as usize].Get() }.StartOffset()
        };
        end_offset - item.EndOffset()
    }

    // cpp: layoutng_inline/line_breaker.cc:1325-1527
    // cpp: layoutng_inline/line_breaker.cc:1855-2044
    fn HandleTextForFastMinContent(
        &mut self,
        item_result: *mut InlineItemResult,
        item: &InlineItem,
        shape_result: &ShapeResult,
        line_info: &mut LineInfo,
    ) -> bool {
        debug_assert_eq!(self.mode_, LineBreakerMode::kMinContent);
        debug_assert!(self.auto_wrap_);
        let text = self.Text().clone();
        let units = text.Span16().unwrap_or_default();
        debug_assert!(
            item.Type() == InlineItemType::kText
                || (item.Type() == InlineItemType::kControl
                    && units[item.StartOffset() as usize] == 0x09)
        );

        let mut start_offset = unsafe { &*item_result }.StartOffset();
        debug_assert!(start_offset < item.EndOffset());
        debug_assert_eq!(shape_result.StartIndex(), item.StartOffset());
        debug_assert!(start_offset >= shape_result.StartIndex());
        let item_end_offset = item.EndOffset();
        let mut end_offset = item_end_offset;
        let mut should_break_at_first_opportunity = false;
        let indent = line_info.TextIndent();
        if indent != LayoutUnit::default() {
            if indent < LayoutUnit::default() {
                return false;
            }
            should_break_at_first_opportunity = true;
            end_offset = start_offset + 1;
        } else if self.position_ < indent {
            return false;
        } else if self.position_ != indent {
            should_break_at_first_opportunity = true;
            end_offset = start_offset + 1;
        }

        shape_result.EnsurePositionData(true);
        let saved_start_offset = self.break_iterator_.StartOffset();
        let mut context = FastMinTextContext::default();
        let item_style = unsafe { &*item.Style() };
        let should_break_spaces = item_style.ShouldBreakSpaces();
        let mut next_break = 0;
        let mut non_hangable_run_end = 0;
        let mut can_break_after = false;
        while start_offset < end_offset {
            self.break_iterator_.SetStartOffset(start_offset);
            next_break = self
                .break_iterator_
                .NextBreakOpportunityTo(start_offset + 1, (item_end_offset + 1).min(text.length()));
            if next_break > item_end_offset {
                debug_assert_eq!(next_break, item_end_offset + 1);
                if start_offset == unsafe { &*item_result }.StartOffset() {
                    next_break = item_end_offset;
                    can_break_after = false;
                } else if units[(next_break - 1) as usize] == 0x0A {
                    next_break = item_end_offset;
                    can_break_after = false;
                } else {
                    next_break = start_offset;
                    debug_assert!(can_break_after);
                    break;
                }
            } else {
                can_break_after = true;
            }
            debug_assert!(next_break <= item_end_offset);

            non_hangable_run_end = next_break;
            if !should_break_spaces {
                while non_hangable_run_end > start_offset
                    && IsBreakableSpace(units[(non_hangable_run_end - 1) as usize])
                {
                    non_hangable_run_end -= 1;
                }
            }
            debug_assert!(non_hangable_run_end >= start_offset);
            let word_len = non_hangable_run_end - start_offset;
            if word_len != 0 {
                let mut has_hyphen =
                    can_break_after && units[(non_hangable_run_end - 1) as usize] == 0x00AD;
                let result = unsafe { &mut *item_result };
                if !self.hyphenation_.is_null() {
                    let word = StringView::from(&text)
                        .Substring(start_offset, word_len)
                        .ToString();
                    let hyphenation = unsafe { &*self.hyphenation_ };
                    if should_break_at_first_opportunity {
                        let location = hyphenation.FirstHyphenLocation(&word, 0);
                        if location != 0 {
                            next_break = start_offset + location;
                            non_hangable_run_end = next_break;
                            has_hyphen = true;
                            can_break_after = true;
                        }
                        context.Add(
                            shape_result,
                            start_offset,
                            non_hangable_run_end,
                            has_hyphen,
                            result,
                        );
                    } else {
                        context.AddHyphenated(
                            shape_result,
                            start_offset,
                            non_hangable_run_end,
                            has_hyphen,
                            result,
                            hyphenation,
                            &word,
                        );
                    }
                } else {
                    context.Add(
                        shape_result,
                        start_offset,
                        non_hangable_run_end,
                        has_hyphen,
                        result,
                    );
                }
            }
            debug_assert!(next_break > start_offset);
            start_offset = next_break;
        }

        self.break_iterator_.SetStartOffset(saved_start_offset);
        debug_assert!(non_hangable_run_end >= unsafe { &*item_result }.StartOffset());
        debug_assert!(non_hangable_run_end <= item_end_offset);
        if item_style.ShouldCollapseWhiteSpaces() {
            unsafe { &mut *item_result }.text_offset.end = non_hangable_run_end;
            self.trailing_whitespace_ = if non_hangable_run_end != next_break {
                WhitespaceState::kCollapsed
            } else {
                WhitespaceState::kNone
            };
        } else {
            unsafe { &mut *item_result }.text_offset.end = next_break;
            self.trailing_whitespace_ = if non_hangable_run_end != next_break {
                WhitespaceState::kPreserved
            } else {
                WhitespaceState::kNone
            };
        }
        unsafe { &*item_result }.text_offset.AssertValid();
        unsafe { &mut *item_result }.inline_size = context.MinInlineSize();
        self.position_ += unsafe { &*item_result }.inline_size;
        unsafe { &mut *item_result }.can_break_after = can_break_after;
        self.state_ = Some(if can_break_after {
            LineBreakState::kTrailing
        } else {
            LineBreakState::kOverflow
        });

        debug_assert!(next_break >= non_hangable_run_end);
        debug_assert!(next_break <= item_end_offset);
        if next_break >= item_end_offset {
            self.MoveToNextOfItem(item);
        } else {
            debug_assert_eq!(
                self.current_.text_offset,
                unsafe { &*item_result }.StartOffset()
            );
            assert!(next_break > self.current_.text_offset);
            self.current_.text_offset = next_break;
        }
        true
    }

    fn HandleText(
        &mut self,
        item: &InlineItem,
        shape_result: &ShapeResult,
        line_info: &mut LineInfo,
    ) {
        debug_assert!(
            item.Type() == InlineItemType::kText
                || (item.Type() == InlineItemType::kControl
                    && self.Text().Span16().unwrap_or_default()[item.StartOffset() as usize]
                        == 0x09)
        );
        let style = unsafe { &*item.Style() };
        if self.auto_wrap_ != self.ShouldAutoWrap(style) {
            let mut detail = std::string::String::new();
            let object_ptr = item.GetLayoutObject();
            if !object_ptr.is_null() {
                let object = unsafe { &*object_ptr };
                detail.push_str(": ");
                detail.push_str(object.GetName());
                let input = object.GetNode();
                if !input.is_null() {
                    detail.push_str(&format!(" ({})", unsafe { &*input }.InputDebugName()));
                }
            }
            eprintln!(
                "inline wrap mismatch{} line={} item={}",
                detail,
                self.auto_wrap_,
                self.ShouldAutoWrap(style)
            );
        }
        debug_assert_eq!(self.auto_wrap_, self.ShouldAutoWrap(style));

        if self.state_ == Some(LineBreakState::kTrailing) {
            self.HandleTrailingSpacesWithShape(item, shape_result, line_info);
            return;
        }

        if self.trailing_whitespace_ == WhitespaceState::kLeading {
            if style.ShouldCollapseWhiteSpaces()
                && self.Text().Span16().unwrap_or_default()[self.current_.text_offset as usize]
                    == 0x20
            {
                self.current_.text_offset += 1;
                if self.current_.text_offset == item.EndOffset() {
                    self.HandleEmptyText(item, line_info);
                    return;
                }
            }
        }

        if self.state_ == Some(LineBreakState::kContinue) && !self.CanFitOnLine() {
            if self.auto_wrap_
                && IsBreakableSpace(
                    self.Text().Span16().unwrap_or_default()[self.current_.text_offset as usize],
                )
            {
                self.HandleTrailingSpacesWithShape(item, shape_result, line_info);
                if self.state_ != Some(LineBreakState::kDone) {
                    self.state_ = Some(LineBreakState::kContinue);
                    return;
                }
            }
            self.HandleOverflow(line_info);
            return;
        }

        if self.HasHyphen() {
            let hyphen_width = self.RemoveHyphen(line_info.MutableResults());
            self.position_ -= hyphen_width;
        }

        if self.maybe_have_end_overhang_ {
            let committed =
                crate::ruby_utils::CommitPendingEndOverhang(item, shape_result, line_info);
            self.position_ -= committed;
        }

        let mut item_result = std::ptr::null_mut();
        if !self.is_svg_text_ {
            item_result = self.AddItem(item, line_info);
            unsafe { &mut *item_result }.should_create_line_box = true;
        }

        if self.auto_wrap_ {
            if self.mode_ == LineBreakerMode::kMinContent
                && self.parent_breaker_.is_null()
                && self.HandleTextForFastMinContent(item_result, item, shape_result, line_info)
            {
                return;
            }

            let available_width = self.RemainingAvailableWidth();
            let break_result = self.BreakText(
                item_result,
                item,
                shape_result,
                available_width,
                available_width,
                line_info,
            );
            let result = unsafe { &*item_result };
            debug_assert!(
                !result.shape_result.Get().is_null()
                    || result.TextOffset().Length() == 0
                    || (break_result == BreakResult::kOverflow
                        && self.break_anywhere_if_overflow_
                        && !self.override_break_anywhere_)
            );
            self.position_ += result.inline_size;
            self.MoveToNextOfResult(result);

            if break_result == BreakResult::kSuccess {
                debug_assert!(
                    !result.shape_result.Get().is_null() || result.TextOffset().Length() == 0
                );
                if result.EndOffset() < item.EndOffset() {
                    self.HandleTrailingSpacesWithShape(item, shape_result, line_info);
                }
                return;
            }
            if break_result == BreakResult::kBreakAt {
                if result.EndOffset() < item.EndOffset() {
                    self.HandleTrailingSpacesWithShape(item, shape_result, line_info);
                    return;
                }
                self.state_ = Some(LineBreakState::kTrailing);
                return;
            }
            debug_assert_eq!(break_result, BreakResult::kOverflow);

            if result.shape_result.Get().is_null() {
                debug_assert!(self.break_anywhere_if_overflow_ && !self.override_break_anywhere_);
                self.HandleOverflow(line_info);
                return;
            }

            if result.has_only_pre_wrap_trailing_spaces {
                self.state_ = Some(LineBreakState::kTrailing);
                let result_style = unsafe { &*(*result.item.Get()).Style() };
                if result_style.ShouldPreserveWhiteSpaces()
                    && IsBreakableSpace(
                        self.Text().Span16().unwrap_or_default()[result.EndOffset() as usize - 1],
                    )
                {
                    let end_index =
                        unsafe { item_result.offset_from(line_info.Results().as_ptr()) } as u32;
                    if self.parent_breaker_.is_null() || end_index > 0 {
                        self.Rewind(end_index, line_info);
                    }
                }
                return;
            }

            if self.state_ == Some(LineBreakState::kOverflow) {
                if result.can_break_after {
                    self.state_ = Some(LineBreakState::kTrailing);
                }
                return;
            }

            if IsAllBreakableSpaces(self.Text(), result.StartOffset(), result.EndOffset()) {
                return;
            }

            self.HandleOverflow(line_info);
            return;
        }

        if self.is_svg_text_ {
            self.SplitTextIntoSegments(item, line_info);
            return;
        }

        let result = unsafe { &mut *item_result };
        debug_assert_eq!(result.EndOffset(), item.EndOffset());
        if result.StartOffset() == item.StartOffset() {
            result.inline_size = shape_result.SnappedWidth().ClampNegativeToZero();
            result.shape_result = Member::from_ptr(ShapeResultView::CreateFromResult(shape_result));
        } else {
            debug_assert!(
                self.trailing_whitespace_ == WhitespaceState::kLeading
                    && result.StartOffset() >= item.StartOffset()
            );
            result.shape_result = Member::from_ptr(ShapeResultView::CreateFromResultRange(
                shape_result,
                result.StartOffset(),
                result.EndOffset(),
            ));
            result.inline_size = unsafe { &*result.shape_result.Get() }
                .SnappedWidth()
                .ClampNegativeToZero();
        }

        debug_assert!(!result.may_break_inside);
        debug_assert!(!result.can_break_after);
        self.trailing_whitespace_ = WhitespaceState::kUnknown;
        self.position_ += result.inline_size;
        self.MoveToNextOfItem(item);
    }

    // cpp: layoutng_inline/line_breaker.cc:1526-1593
    fn SplitTextIntoSegments(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        debug_assert!(self.is_svg_text_);
        debug_assert_eq!(self.current_.text_offset, item.StartOffset());

        let shape = unsafe { &*item.TextShapeResult() };
        let num_glyphs = shape.NumGlyphs();
        if num_glyphs == 0 || !self.needs_svg_segmentation_ {
            let result = unsafe { &mut *self.AddItem(item, line_info) };
            result.should_create_line_box = true;
            result.shape_result = Member::from_ptr(ShapeResultView::CreateFromResult(shape));
            result.inline_size = shape.SnappedWidth();
            self.current_.text_offset = item.EndOffset();
            self.position_ += result.inline_size;
            self.trailing_whitespace_ = WhitespaceState::kUnknown;
            self.MoveToNextOfItem(item);
            return;
        }

        let mut index_list = Vec::<u32>::with_capacity(num_glyphs as usize);
        shape.ForEachGlyph(
            0.0,
            CollectCharIndex,
            &mut index_list as *mut Vec<u32> as *mut std::ffi::c_void,
        );
        if shape.IsRtl() {
            index_list.reverse();
        }
        let size = index_list.len();
        let mut glyph_start = self.current_.text_offset;
        let text_view = StringView::from(self.Text());
        for index in 0..size {
            if index == 0 {
                debug_assert!(glyph_start <= index_list[0]);
            } else {
                debug_assert_eq!(glyph_start, index_list[index]);
            }
            let glyph_end = if index + 1 < size {
                index_list[index + 1]
            } else {
                shape.EndIndex()
            };
            let mut should_split = index == size - 1;
            while glyph_start < glyph_end {
                self.svg_addressable_offset_ += 1;
                should_split = should_split || self.ShouldCreateNewSvgSegment();
                // StringView::NextCodePointOffset is not exposed by foundation;
                // its UTF-16 surrogate step is local to this source loop.
                glyph_start += if text_view.CodePointAt(glyph_start) > 0xffff {
                    2
                } else {
                    1
                };
            }
            if !should_split {
                continue;
            }
            let result = unsafe { &mut *self.AddItemWithEnd(item, glyph_end, line_info) };
            result.should_create_line_box = true;
            let shape_result_view =
                ShapeResultView::CreateFromResultRange(shape, self.current_.text_offset, glyph_end);
            // SVG text keeps a negative snapped width from word spacing.
            result.inline_size = unsafe { &*shape_result_view }.SnappedWidth();
            result.shape_result = Member::from_ptr(shape_result_view);
            self.current_.text_offset = glyph_end;
            self.position_ += result.inline_size;
        }
        self.trailing_whitespace_ = WhitespaceState::kUnknown;
        self.MoveToNextOfItem(item);
    }

    // C++ declares this const while mutating the iterator behind unique_ptr.
    // Rust reflects that hidden mutation with &mut self.
    // cpp: layoutng_inline/line_breaker.cc:1595-1613
    fn ShouldCreateNewSvgSegment(&mut self) -> bool {
        debug_assert!(self.is_svg_text_);
        for range in self.node_.SvgTextPathRangeList() {
            if range.start_index <= self.svg_addressable_offset_
                && self.svg_addressable_offset_ <= range.end_index
            {
                return true;
            }
        }
        for range in self.node_.SvgTextLengthRangeList() {
            let layout_object = unsafe { &*range.layout_object.Get() };
            let element = unsafe { &*To::<Element>(layout_object.GetNode()) };
            if element.InputSvgLengthAdjust()
                == SVGLengthAdjustType::kSVGLengthAdjustSpacingAndGlyphs
            {
                continue;
            }
            if range.start_index <= self.svg_addressable_offset_
                && self.svg_addressable_offset_ <= range.end_index
            {
                return true;
            }
        }
        let char_data = self
            .svg_resolved_iterator_
            .as_mut()
            .expect("SVG resolved attributes iterator must be installed")
            .AdvanceTo(self.svg_addressable_offset_);
        char_data.HasRotate()
            || char_data.HasX()
            || char_data.HasY()
            || char_data.HasDx()
            || char_data.HasDy()
    }

    // cpp: layoutng_inline/line_breaker.cc:1615-1771
    fn BreakText(
        &mut self,
        item_result: *mut InlineItemResult,
        item: &InlineItem,
        item_shape_result: &ShapeResult,
        mut available_width: LayoutUnit,
        available_width_with_hyphens: LayoutUnit,
        line_info: &mut LineInfo,
    ) -> BreakResult {
        debug_assert!(
            item.Type() == InlineItemType::kText
                || (item.Type() == InlineItemType::kControl
                    && self.Text().Span16().unwrap_or_default()[item.StartOffset() as usize]
                        == 0x09)
        );
        item.AssertOffset(unsafe { &*item_result }.StartOffset());
        debug_assert!(!self.HasHyphen());
        debug_assert_eq!(item_shape_result.StartIndex(), item.StartOffset());
        debug_assert_eq!(item_shape_result.EndIndex(), item.EndOffset());

        // The callback reads these stable fields while hyphen state is
        // updated below. Raw pointers keep those disjoint borrows explicit.
        let backend = ShapingLineBreakerImpl {
            shaper: std::ptr::addr_of!(self.shaper_),
            spacing: std::ptr::addr_of_mut!(self.spacing_),
            items_data: self.items_data_,
            item,
        };
        let style = unsafe { &*item.Style() };
        let font = unsafe { &*style.GetFont() };
        let mut breaker = ShapingLineBreaker::new(
            item_shape_result,
            unsafe { &*std::ptr::addr_of!(self.break_iterator_) },
            unsafe { self.hyphenation_.as_ref() },
            font,
            &backend,
        );
        breaker.SetTextSpacingTrim(style.GetFontDescription().GetTextSpacingTrim());
        breaker.SetLineStart(line_info.StartOffset());
        breaker.SetIsAfterForcedBreak(self.previous_line_had_forced_break_);
        if !NeedsAccurateEndPositionForLine(line_info, item) {
            breaker.SetDontReshapeEndIfAtSpace();
        }

        if self.break_at_.IsNotZero() {
            return if self.BreakTextAt(item_result, item, &breaker, line_info) {
                BreakResult::kBreakAt
            } else {
                BreakResult::kSuccess
            };
        }
        if self.break_anywhere_if_overflow_ && !self.override_break_anywhere_ {
            breaker.SetNoResultIfOverflow();
        }

        let mut result = ShapingBreakResult::default();
        let mut try_count = 0;
        let inline_size = loop {
            try_count += 1;
            debug_assert!(try_count <= 2);
            let shape_result = breaker.ShapeLine(
                unsafe { &*item_result }.StartOffset(),
                available_width.ClampNegativeToZero(),
                &mut result,
            );
            if shape_result.is_null() {
                debug_assert!(breaker.NoResultIfOverflow());
                let item_result = unsafe { &mut *item_result };
                item_result.inline_size = available_width_with_hyphens + LayoutUnit::from_signed(1);
                item_result.text_offset.end = item.EndOffset();
                item_result.text_offset.AssertNotEmpty();
                return BreakResult::kOverflow;
            }
            let shape_result_ref = unsafe { &*shape_result };
            debug_assert_eq!(
                shape_result_ref.NumCharacters(),
                result.break_offset - unsafe { &*item_result }.StartOffset()
            );
            assert!(result.break_offset > unsafe { &*item_result }.StartOffset());

            let mut inline_size = shape_result_ref.SnappedWidth().ClampNegativeToZero();
            unsafe { &mut *item_result }.inline_size = inline_size;
            if result.is_hyphenated {
                let item_results = line_info.MutableResults();
                let hyphen_inline_size = self.AddHyphenForResult(item_results, item_result);
                if !result.is_overflow && inline_size <= available_width {
                    let space_for_hyphen = available_width_with_hyphens - inline_size;
                    if space_for_hyphen >= LayoutUnit::default()
                        && hyphen_inline_size > space_for_hyphen
                    {
                        available_width -= hyphen_inline_size;
                        self.RemoveHyphen(item_results);
                        continue;
                    }
                }
                inline_size = unsafe { &*item_result }.inline_size;
            }
            let item_result_ref = unsafe { &mut *item_result };
            item_result_ref.text_offset.end = result.break_offset;
            item_result_ref.text_offset.AssertNotEmpty();
            item_result_ref.has_only_pre_wrap_trailing_spaces = result.has_trailing_spaces;
            item_result_ref.has_only_bidi_trailing_spaces = result.has_trailing_spaces;
            item_result_ref.shape_result = Member::from_ptr(shape_result as *mut ShapeResultView);
            break inline_size;
        };

        let item_result_ref = unsafe { &mut *item_result };
        if item_result_ref.EndOffset() < item.EndOffset() {
            item_result_ref.can_break_after = true;
            self.trailing_whitespace_ =
                if self.break_iterator_.BreakType() == LineBreakType::kBreakCharacter {
                    WhitespaceState::kUnknown
                } else {
                    WhitespaceState::kNone
                };
        } else {
            debug_assert_eq!(item_result_ref.EndOffset(), item.EndOffset());
            item_result_ref.can_break_after = self.CanBreakAfter(item);
            self.trailing_whitespace_ = WhitespaceState::kUnknown;
        }
        item_result_ref.may_break_inside = !result.is_overflow;
        if inline_size <= available_width_with_hyphens {
            BreakResult::kSuccess
        } else {
            BreakResult::kOverflow
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:1773-1807
    fn BreakTextAt(
        &mut self,
        item_result: *mut InlineItemResult,
        _item: &InlineItem,
        breaker: &ShapingLineBreaker<'_>,
        line_info: &mut LineInfo,
    ) -> bool {
        debug_assert!(self.break_at_.IsNotZero());
        debug_assert!(self.current_.text_offset <= self.break_at_.end.text_offset);
        debug_assert!(self.current_.item_index <= self.break_at_.offset.item_index);
        let should_break = self.current_.item_index >= self.break_at_.end.item_index;
        let has_length = {
            let item_result_ref = unsafe { &mut *item_result };
            if should_break {
                debug_assert!(self.break_at_.end.text_offset <= item_result_ref.text_offset.end);
                item_result_ref.text_offset.end = self.break_at_.end.text_offset;
                item_result_ref.text_offset.AssertValid();
            } else {
                debug_assert!(self.break_at_.end.text_offset >= item_result_ref.text_offset.end);
            }
            if item_result_ref.Length() > 0 {
                let shape_result =
                    breaker.ShapeLineAt(item_result_ref.StartOffset(), item_result_ref.EndOffset());
                item_result_ref.inline_size = unsafe { &*shape_result }
                    .SnappedWidth()
                    .ClampNegativeToZero();
                item_result_ref.shape_result =
                    Member::from_ptr(shape_result as *mut ShapeResultView);
                true
            } else {
                debug_assert_eq!(item_result_ref.inline_size, LayoutUnit::default());
                debug_assert!(!self.break_at_.is_hyphenated);
                false
            }
        };
        if has_length && self.break_at_.is_hyphenated {
            self.AddHyphenForResult(line_info.MutableResults(), item_result);
        }
        unsafe { &mut *item_result }.can_break_after = true;
        self.trailing_whitespace_ = WhitespaceState::kNone;
        should_break
    }

    // cpp: layoutng_inline/line_breaker.cc:1809-1845
    fn BreakTextAtPreviousBreakOpportunity(
        &mut self,
        results: &mut InlineItemResults,
        item_result_index: u32,
    ) -> bool {
        let item_result = &mut results[item_result_index as usize];
        debug_assert!(!item_result.item.Get().is_null());
        debug_assert!(item_result.may_break_inside);
        let item = unsafe { &*item_result.item.Get() };
        debug_assert_eq!(item.Type(), InlineItemType::kText);
        debug_assert!(!item.Style().is_null() && unsafe { &*item.Style() }.ShouldWrapLine());
        debug_assert!(!self.is_text_combine_);

        let break_opportunity = self
            .break_iterator_
            .PreviousBreakOpportunity(item_result.EndOffset() - 1, item_result.StartOffset());
        if break_opportunity <= item_result.StartOffset() {
            return false;
        }
        item_result.text_offset.end = break_opportunity;
        item_result.text_offset.AssertNotEmpty();
        item_result.shape_result = Member::from_ptr(ShapeResultView::CreateFromResultRange(
            item.TextShapeResult(),
            item_result.StartOffset(),
            item_result.EndOffset(),
        ));
        item_result.inline_size = unsafe { &*item_result.shape_result.Get() }
            .SnappedWidth()
            .ClampNegativeToZero();
        item_result.can_break_after = true;

        if self
            .trailing_collapsible_space_
            .as_ref()
            .is_some_and(|space| {
                std::ptr::eq(space.item_results, results)
                    && space.item_result_index == item_result_index
            })
        {
            self.trailing_collapsible_space_ = None;
        }
        true
    }

    // cpp: layoutng_inline/line_breaker.cc:2057-2076
    fn ShapeTextFromParts(
        shaper: &HarfBuzzShaper,
        spacing: &mut ShapeResultSpacing,
        items_data: &InlineItemsData,
        item: &InlineItem,
        start: u32,
        end: u32,
        options: ShapeOptions,
    ) -> *const ShapeResult {
        let style = unsafe { &*item.Style() };
        let shape_result = if items_data.segments.Get().is_null() {
            let segment_range =
                InlineItemSegment::UnpackSegmentData(start, end, item.SegmentData());
            shaper.ShapeSingleRange(
                style.GetFont(),
                item.Direction(),
                start,
                end,
                segment_range,
                options,
            )
        } else {
            unsafe { &*items_data.segments.Get() }.ShapeText(
                shaper,
                style.GetFont(),
                item.Direction(),
                start,
                end,
                item.Index(),
                options,
            )
        };
        if spacing.HasSpacing() {
            unsafe { &mut *shape_result }.ApplySpacing(spacing, 0);
        }
        shape_result
    }

    // cpp: layoutng_inline/line_breaker.cc:2057-2076
    fn ShapeText(
        &mut self,
        item: &InlineItem,
        start: u32,
        end: u32,
        options: ShapeOptions,
    ) -> *const ShapeResult {
        Self::ShapeTextFromParts(
            &self.shaper_,
            &mut self.spacing_,
            unsafe { &*self.items_data_ },
            item,
            start,
            end,
            options,
        )
    }

    // cpp: layoutng_inline/line_breaker.cc:2078-2331
    pub fn AppendCandidates(
        &mut self,
        item_result: &InlineItemResult,
        line_info: &LineInfo,
        context: &mut LineBreakCandidateContext<'_>,
    ) {
        debug_assert!(!item_result.item.Get().is_null());
        let item = unsafe { &*item_result.item.Get() };
        let item_index = item_result.item_index;
        debug_assert!(
            context.GetState() == CandidateState::kBreak || !context.Candidates().is_empty()
        );
        debug_assert_eq!(item.Type(), InlineItemType::kText);
        if item.Length() == 0 {
            context.AppendTrailingSpaces(
                if item_result.can_break_after {
                    CandidateState::kBreak
                } else {
                    context.GetState()
                },
                InlineItemTextIndex {
                    item_index,
                    text_offset: item.EndOffset(),
                },
                context.Position(),
            );
            context.SetLast(item, item.EndOffset());
            return;
        }

        debug_assert!(!item.TextShapeResult().is_null());
        let mut shape_result = CandidateShapeResult::new(unsafe { &*item.TextShapeResult() });
        let text_content = self.Text().clone();
        let text_units = text_content.Span16().unwrap_or_default();
        let mut offset = *item_result.TextOffset();
        offset.end = offset
            .end
            .max(item.EndOffset().min(line_info.EndTextOffset()));

        if !context.LastItem().is_null() {
            debug_assert!(context.LastEndOffset() >= item.StartOffset());
            if context.LastEndOffset() >= offset.end {
                return;
            }
            offset.start = context.LastEndOffset();
            offset.AssertNotEmpty();
            shape_result.SetBasePosition(offset.start, context.Position());
            if IsBreakableSpace(text_units[offset.start as usize]) {
                debug_assert!(offset.start >= item.StartOffset());
                loop {
                    offset.start += 1;
                    if offset.start >= offset.end
                        || !IsBreakableSpace(text_units[offset.start as usize])
                    {
                        break;
                    }
                }
                let end_position = shape_result.PositionForOffset(offset.start);
                if offset.Length() == 0 {
                    context.AppendTrailingSpaces(
                        if item_result.can_break_after {
                            CandidateState::kBreak
                        } else {
                            CandidateState::kMidWord
                        },
                        InlineItemTextIndex {
                            item_index,
                            text_offset: offset.start,
                        },
                        end_position,
                    );
                    context.SetLast(item, offset.end);
                    return;
                }
                context.AppendTrailingSpaces(
                    if self.auto_wrap_ {
                        CandidateState::kBreak
                    } else {
                        CandidateState::kMidWord
                    },
                    InlineItemTextIndex {
                        item_index,
                        text_offset: offset.start,
                    },
                    end_position,
                );
            }
        } else {
            shape_result.SetBasePosition(offset.start, context.Position());
        }
        offset.AssertNotEmpty();
        debug_assert!(offset.start >= item.StartOffset());
        debug_assert!(offset.start >= context.LastEndOffset());
        debug_assert!(offset.end <= item.EndOffset());
        context.SetLast(item, offset.end);

        if offset.start < self.break_iterator_.StartOffset() {
            self.break_iterator_.SetStartOffset(offset.start);
        }
        debug_assert!(!item.Style().is_null());
        self.SetCurrentStyle(unsafe { &*item.Style() });

        let mut hyphen_advance_cache: Option<LayoutUnit> = None;
        loop {
            let mut next_offset = if self.auto_wrap_ {
                let length = (offset.end + 1).min(text_content.length());
                self.break_iterator_
                    .NextBreakOpportunityTo(offset.start + 1, length)
            } else {
                offset.end + 1
            };
            if next_offset > offset.end && item_result.can_break_after {
                next_offset = offset.end;
            }

            let end_offset;
            let next_position;
            let mut end_position;
            let mut next_state = CandidateState::kBreak;
            let mut penalty = 0.0;
            let mut is_hyphenated = false;
            if next_offset > offset.end {
                next_offset = offset.end;
                end_offset = next_offset;
                next_position = shape_result.PositionForOffset(next_offset);
                end_position = next_position;
                next_state = CandidateState::kMidWord;
            } else {
                if next_offset == offset.end && !item_result.can_break_after {
                    next_state = CandidateState::kMidWord;
                }
                next_position = shape_result.PositionForOffset(next_offset);
                let mut word_end_offset = next_offset;
                debug_assert!(word_end_offset > offset.start);
                let mut last_ch = text_units[(word_end_offset - 1) as usize];
                while IsBreakableSpace(last_ch) {
                    word_end_offset -= 1;
                    if word_end_offset == offset.start {
                        last_ch = 0;
                        break;
                    }
                    last_ch = text_units[(word_end_offset - 1) as usize];
                }
                end_offset = word_end_offset;
                debug_assert!(end_offset <= offset.end);

                if !self.hyphenation_.is_null() {
                    let hyphen_advance = HyphenAdvance(
                        unsafe { &*self.current_style_ },
                        shape_result.IsLtr(),
                        &item_result.hyphen,
                        &mut hyphen_advance_cache,
                    );
                    debug_assert!(end_offset > offset.start);
                    #[cfg(feature = "expensive_dchecks")]
                    let word_len = end_offset - offset.start;
                    let word =
                        String::from_utf16(&text_units[offset.start as usize..end_offset as usize]);
                    let locations = unsafe { &*self.hyphenation_ }.HyphenLocations(&word);
                    #[cfg(feature = "expensive_dchecks")]
                    {
                        debug_assert!(!locations.contains(&0));
                        debug_assert!(!locations.contains(&word_len));
                        debug_assert!(locations.windows(2).all(|window| window[0] >= window[1]));
                    }
                    let hyphen_penalty = context.HyphenPenalty();
                    for &location in locations.iter().rev() {
                        let hyphen_offset = InlineItemTextIndex {
                            item_index,
                            text_offset: offset.start + location,
                        };
                        let position = shape_result.PositionForOffset(hyphen_offset.text_offset);
                        context.Append(
                            CandidateState::kBreak,
                            hyphen_offset,
                            hyphen_offset,
                            position,
                            position + hyphen_advance.ToFloat(),
                            hyphen_penalty,
                            true,
                        );
                    }
                }

                let end_safe_offset = match next_state {
                    CandidateState::kBreak => {
                        let mut safe = shape_result.PreviousSafeToBreakOffset(end_offset);
                        if safe < offset.start {
                            debug_assert_eq!(
                                context.Candidates().last().unwrap().offset.text_offset,
                                offset.start
                            );
                            safe = offset.start;
                        }
                        safe
                    }
                    CandidateState::kMidWord => end_offset,
                };
                if end_safe_offset == end_offset {
                    end_position = if end_offset == next_offset {
                        next_position
                    } else {
                        shape_result.PositionForOffset(end_offset)
                    };
                } else {
                    debug_assert!(end_safe_offset < end_offset);
                    end_position = shape_result.PositionForOffset(end_safe_offset);
                    let end_shape_result =
                        self.ShapeText(item, end_safe_offset, end_offset, ShapeOptions::default());
                    end_position += unsafe { &*end_shape_result }.Width();
                }

                debug_assert!(!is_hyphenated);
                if end_offset == item_result.EndOffset() {
                    is_hyphenated = item_result.is_hyphenated;
                } else if last_ch == 0x00ad && next_state == CandidateState::kBreak {
                    is_hyphenated = true;
                }
                if is_hyphenated {
                    end_position += HyphenAdvance(
                        unsafe { &*self.current_style_ },
                        shape_result.IsLtr(),
                        &item_result.hyphen,
                        &mut hyphen_advance_cache,
                    )
                    .ToFloat();
                    penalty = context.HyphenPenalty();
                }
            }

            context.Append(
                next_state,
                InlineItemTextIndex {
                    item_index,
                    text_offset: next_offset,
                },
                InlineItemTextIndex {
                    item_index,
                    text_offset: end_offset,
                },
                next_position,
                end_position,
                penalty,
                is_hyphenated,
            );
            if next_offset >= offset.end {
                break;
            }
            offset.start = next_offset;
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:2383-2417
    fn TruncateLineEndResult(
        &mut self,
        line_info: &LineInfo,
        item_result: &InlineItemResult,
        end_offset: u32,
    ) -> *const ShapeResultView {
        debug_assert!(!item_result.item.Get().is_null());
        let item = unsafe { &*item_result.item.Get() };
        let start_offset = item_result.StartOffset();
        let source_result = item_result.shape_result.Get();
        debug_assert!(!source_result.is_null());
        let source_result_ref = unsafe { &*source_result };
        debug_assert!(start_offset >= source_result_ref.StartIndex());
        debug_assert!(end_offset <= source_result_ref.EndIndex());
        debug_assert!(
            start_offset > source_result_ref.StartIndex()
                || end_offset < source_result_ref.EndIndex()
        );

        if !NeedsAccurateEndPositionForLine(line_info, item) {
            return ShapeResultView::CreateFromViewRange(source_result, start_offset, end_offset);
        }
        let last_safe = source_result_ref.PreviousSafeToBreakOffset(end_offset);
        debug_assert!(last_safe <= end_offset);
        if last_safe == end_offset || last_safe <= start_offset {
            return ShapeResultView::CreateFromViewRange(source_result, start_offset, end_offset);
        }

        let end_result = self.ShapeText(
            item,
            last_safe.max(start_offset),
            end_offset,
            ShapeOptions::default(),
        );
        debug_assert_eq!(
            unsafe { &*end_result }.Direction(),
            source_result_ref.Direction()
        );
        ShapeResultView::Create(&[
            Segment::from_view(source_result, start_offset, last_safe),
            Segment::from_result(end_result, 0, end_offset),
        ])
    }

    // cpp: layoutng_inline/line_breaker.cc:2421-2428
    fn UpdateShapeResult(&mut self, line_info: &LineInfo, item_result: *mut InlineItemResult) {
        debug_assert!(!item_result.is_null());
        let result = self.TruncateLineEndResult(
            line_info,
            unsafe { &*item_result },
            unsafe { &*item_result }.EndOffset(),
        );
        debug_assert!(!result.is_null());
        let item_result = unsafe { &mut *item_result };
        item_result.shape_result = Member::from_ptr(result as *mut ShapeResultView);
        item_result.inline_size = unsafe { &*result }.SnappedWidth();
    }

    // cpp: layoutng_inline/line_breaker.cc:2430-2436
    fn HandleTrailingSpaces(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        let shape_result = item.TextShapeResult();
        self.HandleTrailingSpacesWithShape(item, shape_result, line_info);
    }

    // cpp: layoutng_inline/line_breaker.cc:2438-2546
    fn HandleTrailingSpacesWithShape(
        &mut self,
        item: &InlineItem,
        shape_result: *const ShapeResult,
        line_info: &mut LineInfo,
    ) {
        debug_assert!(
            item.Type() == InlineItemType::kText
                || (item.Type() == InlineItemType::kControl
                    && self.Text().Span16().unwrap_or_default()[item.StartOffset() as usize]
                        == 0x09)
        );
        debug_assert!(self.current_.text_offset >= item.StartOffset());
        debug_assert!(self.current_.text_offset < item.EndOffset());
        let text = self.Text().clone();
        let units = text.Span16().unwrap_or_default();
        debug_assert!(!item.Style().is_null());
        let style = unsafe { &*item.Style() };

        if !self.auto_wrap_ {
            self.state_ = Some(LineBreakState::kDone);
            return;
        }
        debug_assert!(!self.is_text_combine_);

        if style.ShouldCollapseWhiteSpaces()
            && !Character::IsOtherSpaceSeparator(i32::from(
                units[self.current_.text_offset as usize],
            ))
        {
            if units[self.current_.text_offset as usize] != 0x20 {
                if self.current_.text_offset > 0
                    && IsBreakableSpace(units[self.current_.text_offset as usize - 1])
                {
                    self.trailing_whitespace_ = WhitespaceState::kCollapsible;
                }
                self.state_ = Some(LineBreakState::kDone);
                return;
            }

            self.current_.text_offset += 1;
            if self.trailing_whitespace_ != WhitespaceState::kPreserved {
                self.trailing_whitespace_ = WhitespaceState::kCollapsed;
            }
            let item_results = line_info.MutableResults();
            debug_assert!(!item_results.empty());
            item_results.last_mut().unwrap().can_break_after = true;
        } else if !style.ShouldBreakSpaces() {
            debug_assert!(
                style.ShouldBreakOnlyAfterWhiteSpace()
                    || Character::IsOtherSpaceSeparator(i32::from(
                        units[self.current_.text_offset as usize]
                    ))
            );
            let mut end = self.current_.text_offset;
            while end < item.EndOffset() && IsBreakableSpaceOrOtherSeparator(units[end as usize]) {
                end += 1;
            }
            if end == self.current_.text_offset {
                if IsBreakableSpaceOrOtherSeparator(units[end as usize - 1]) {
                    self.trailing_whitespace_ = WhitespaceState::kPreserved;
                }
                self.state_ = Some(LineBreakState::kDone);
                return;
            }

            debug_assert!(!shape_result.is_null());
            let item_result = self.AddItemWithEnd(item, end, line_info);
            let result = unsafe { &mut *item_result };
            result.should_create_line_box = true;
            result.has_only_pre_wrap_trailing_spaces = true;
            result.has_only_bidi_trailing_spaces = true;
            result.shape_result = Member::from_ptr(ShapeResultView::CreateFromResult(shape_result));
            if result.StartOffset() == item.StartOffset() && result.EndOffset() == item.EndOffset()
            {
                result.inline_size = if !result.shape_result.Get().is_null()
                    && self.mode_ != LineBreakerMode::kMinContent
                    && self.line_clamp_ellipsis_width_ == LayoutUnit::default()
                {
                    unsafe { &*result.shape_result.Get() }.SnappedWidth()
                } else {
                    LayoutUnit::default()
                };
            } else {
                self.UpdateShapeResult(line_info, item_result);
                if self.mode_ == LineBreakerMode::kMinContent
                    || self.line_clamp_ellipsis_width_ != LayoutUnit::default()
                {
                    unsafe { &mut *item_result }.inline_size = LayoutUnit::default();
                }
            }
            self.position_ += unsafe { &*item_result }.inline_size;
            unsafe { &mut *item_result }.can_break_after =
                end < text.length() && !IsBreakableSpaceOrOtherSeparator(units[end as usize]);
            self.current_.text_offset = end;
            self.trailing_whitespace_ = WhitespaceState::kPreserved;
        }

        debug_assert!(self.current_.text_offset <= item.EndOffset());
        if self.current_.text_offset < item.EndOffset() {
            self.state_ = Some(LineBreakState::kDone);
            return;
        }
        debug_assert_eq!(self.current_.text_offset, item.EndOffset());
        let item_results = line_info.Results();
        if item_results.empty()
            || item_results.last().unwrap().item.Get()
                != item as *const InlineItem as *mut InlineItem
        {
            self.AddEmptyItem(item, line_info);
        }
        self.current_.item_index += 1;
        self.state_ = Some(LineBreakState::kTrailing);
    }

    // cpp: layoutng_inline/line_breaker.cc:2663-2679
    fn ComputeTrailingCollapsibleSpace(&mut self, line_info: &mut LineInfo) {
        if matches!(
            self.trailing_whitespace_,
            WhitespaceState::kLeading
                | WhitespaceState::kNone
                | WhitespaceState::kCollapsed
                | WhitespaceState::kPreserved
        ) {
            self.trailing_collapsible_space_ = None;
            return;
        }
        debug_assert!(matches!(
            self.trailing_whitespace_,
            WhitespaceState::kUnknown | WhitespaceState::kCollapsible
        ));
        self.trailing_whitespace_ = WhitespaceState::kNone;
        if !self.ComputeTrailingCollapsibleSpaceHelper(line_info) {
            self.trailing_collapsible_space_ = None;
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:2681-2758
    fn ComputeTrailingCollapsibleSpaceHelper(&mut self, line_info: &mut LineInfo) -> bool {
        let text = self.Text().clone();
        let units = text.Span16().unwrap_or_default();
        let results = line_info.MutableResults() as *mut InlineItemResults;
        let count = unsafe { &*results }.size();
        for index in (0..count).rev() {
            let item_result = unsafe { &(&*results)[index as usize] };
            debug_assert!(!item_result.item.Get().is_null());
            let item = unsafe { &*item_result.item.Get() };
            if item_result.IsRubyColumn() {
                let ruby_column = unsafe { &mut *item_result.ruby_column.Get() };
                if self.ComputeTrailingCollapsibleSpaceHelper(&mut ruby_column.base_line) {
                    if let Some(space) = self.trailing_collapsible_space_.as_mut() {
                        if space.item_result_index != kNotFound {
                            space.ancestor_ruby_columns.push((results, index));
                        }
                    }
                    return true;
                }
                continue;
            } else if item.EndCollapseType() == CollapseType::kOpaqueToCollapsing {
                continue;
            }
            if item.Type() == InlineItemType::kText {
                if item_result.Length() == 0 {
                    continue;
                }
                debug_assert!(item_result.EndOffset() > 0);
                debug_assert!(!item.Style().is_null());
                let last_character = units[item_result.EndOffset() as usize - 1];
                if Character::IsOtherSpaceSeparator(i32::from(last_character)) {
                    self.trailing_whitespace_ = WhitespaceState::kPreserved;
                    self.trailing_collapsible_space_ = None;
                    return true;
                }
                if !IsBreakableSpace(last_character) {
                    self.trailing_collapsible_space_ = None;
                    return true;
                }
                if unsafe { &*item.Style() }.ShouldPreserveWhiteSpaces() {
                    self.trailing_whitespace_ = WhitespaceState::kPreserved;
                    self.trailing_collapsible_space_ = None;
                    return true;
                }
                if item_result.shape_result.Get().is_null() {
                    self.trailing_collapsible_space_ = None;
                    return true;
                }

                let needs_new = self
                    .trailing_collapsible_space_
                    .as_ref()
                    .map_or(true, |space| {
                        !std::ptr::eq(space.item_results, results)
                            || space.item_result_index != index
                    });
                if needs_new {
                    let collapsed_shape_result =
                        if item_result.EndOffset() - 1 > item_result.StartOffset() {
                            self.TruncateLineEndResult(
                                line_info,
                                item_result,
                                item_result.EndOffset() - 1,
                            )
                        } else {
                            std::ptr::null()
                        };
                    self.trailing_collapsible_space_ = Some(TrailingCollapsibleSpace {
                        item_results: results,
                        item_result_index: index,
                        collapsed_shape_result,
                        ancestor_ruby_columns: Vec::new(),
                    });
                }
                self.trailing_whitespace_ = WhitespaceState::kCollapsible;
                return true;
            }
            if item.Type() == InlineItemType::kControl {
                if item.TextType() == TextItemType::kForcedLineBreak {
                    debug_assert_eq!(units[item.StartOffset() as usize], 0x0a);
                    continue;
                }
                self.trailing_whitespace_ = WhitespaceState::kPreserved;
                self.trailing_collapsible_space_ = None;
                return true;
            }
            self.trailing_collapsible_space_ = None;
            return true;
        }
        false
    }

    // cpp: layoutng_inline/line_breaker.cc:2548-2574
    fn RewindTrailingOpenTags(&mut self, line_info: &mut LineInfo) {
        let rewind_to = {
            let item_results = line_info.Results();
            let mut rewind_to = None;
            for index in (0..item_results.size()).rev() {
                let item_result = &item_results[index as usize];
                debug_assert!(!item_result.item.Get().is_null());
                if unsafe { &*item_result.item.Get() }.Type() != InlineItemType::kOpenTag {
                    let end_index = index + 1;
                    if end_index < item_results.size() {
                        rewind_to = Some((end_index, item_results[end_index as usize].Start()));
                    }
                    break;
                }
            }
            rewind_to
        };
        if let Some((end_index, end)) = rewind_to {
            self.ResetRewindLoopDetector();
            self.Rewind(end_index, line_info);
            self.current_ = end;
            self.ItemsData()
                .AssertOffset(self.current_.item_index, self.current_.text_offset);
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:2576-2626
    fn RemoveTrailingCollapsibleSpace(&mut self, line_info: &mut LineInfo) {
        if !self.is_forced_break_ {
            self.RewindTrailingOpenTags(line_info);
        }
        self.ComputeTrailingCollapsibleSpace(line_info);
        let Some(mut space) = self.trailing_collapsible_space_.take() else {
            return;
        };

        let position_was_saturated = self.position_ == LayoutUnit::Max();
        {
            let collapsed_shape_result = space.collapsed_shape_result;
            let item_result = space.ItemResult();
            self.position_ -= item_result.inline_size;
            if !collapsed_shape_result.is_null() {
                item_result.text_offset.end -= 1;
                item_result.text_offset.AssertNotEmpty();
                item_result.shape_result =
                    Member::from_ptr(collapsed_shape_result as *mut ShapeResultView);
                item_result.inline_size = unsafe { &*collapsed_shape_result }.SnappedWidth();
                self.position_ += item_result.inline_size;
            } else {
                item_result.text_offset.end = item_result.text_offset.start;
                item_result.shape_result = Member::default();
                item_result.inline_size = LayoutUnit::default();
            }
        }
        for &(results, index) in &space.ancestor_ruby_columns {
            let ruby_column = unsafe { &mut (&mut *results)[index as usize] };
            assert!(ruby_column.IsRubyColumn());
            let ruby = unsafe { &mut *ruby_column.ruby_column.Get() };
            let base_line = &mut ruby.base_line;
            let mut new_width = base_line.ComputeWidth();
            base_line.SetWidth(base_line.AvailableWidth(), new_width);
            base_line.UpdateTextAlign();
            for line in &ruby.annotation_line_list {
                new_width = new_width.max(line.Width());
            }
            ruby_column.inline_size = new_width;
        }
        if position_was_saturated || !space.ancestor_ruby_columns.is_empty() {
            self.position_ = line_info.ComputeWidth();
        }
        self.trailing_whitespace_ = WhitespaceState::kCollapsed;
    }

    // cpp: layoutng_inline/line_breaker.cc:2628-2661
    fn TrailingCollapsibleSpaceWidth(&mut self, line_info: &mut LineInfo) -> LayoutUnit {
        self.ComputeTrailingCollapsibleSpace(line_info);
        let Some(space) = self.trailing_collapsible_space_.as_mut() else {
            return LayoutUnit::default();
        };
        let mut width_diff = space.ItemResult().inline_size;
        if !space.collapsed_shape_result.is_null() {
            width_diff -= unsafe { &*space.collapsed_shape_result }.SnappedWidth();
        }
        if space.ancestor_ruby_columns.is_empty() {
            return width_diff;
        }
        for &(results, index) in &space.ancestor_ruby_columns {
            let ruby_column = unsafe { &mut (&mut *results)[index as usize] };
            assert!(ruby_column.IsRubyColumn());
            let ruby = unsafe { &mut *ruby_column.ruby_column.Get() };
            let mut new_width = ruby.base_line.Width() - width_diff;
            for line in &ruby.annotation_line_list {
                new_width = new_width.max(line.Width());
            }
            width_diff = ruby_column.inline_size - new_width;
            if width_diff == LayoutUnit::default() {
                break;
            }
        }
        width_diff
    }

    // cpp: layoutng_inline/line_breaker.cc:2770-2865
    fn SplitTrailingBidiPreservedSpace(&mut self, line_info: &mut LineInfo) {
        debug_assert!(matches!(
            self.trailing_whitespace_,
            WhitespaceState::kLeading
                | WhitespaceState::kNone
                | WhitespaceState::kCollapsed
                | WhitespaceState::kPreserved
        ));
        if matches!(
            self.trailing_whitespace_,
            WhitespaceState::kLeading | WhitespaceState::kNone
        ) {
            return;
        }
        if !self.node_.IsBidiEnabled() {
            return;
        }
        if self.mode_ == LineBreakerMode::kMinContent {
            return;
        }

        let text = self.Text().clone();
        let units = text.Span16().unwrap_or_default();
        let results = line_info.MutableResults();
        for result_index in (0..results.size()).rev() {
            let mut spaces_result = None;
            {
                let item_result = &mut results[result_index as usize];
                debug_assert!(!item_result.item.Get().is_null());
                let item = unsafe { &*item_result.item.Get() };
                if item_result.has_only_bidi_trailing_spaces
                    || item.EndCollapseType() == CollapseType::kOpaqueToCollapsing
                    || item.TextType() == TextItemType::kForcedLineBreak
                {
                    continue;
                }
                if item.Type() != InlineItemType::kText && item.Type() != InlineItemType::kControl {
                    return;
                }
                if item_result.Length() == 0 {
                    item_result.has_only_bidi_trailing_spaces = true;
                    continue;
                }
                debug_assert!(item_result.EndOffset() > 0);
                let mut split_offset = item_result.EndOffset();
                while split_offset > item_result.StartOffset()
                    && (IsBreakableSpace(units[split_offset as usize - 1])
                        || IsBidiTrailingSpace(units[split_offset as usize - 1]))
                {
                    split_offset -= 1;
                }
                if split_offset == item_result.StartOffset() {
                    item_result.has_only_bidi_trailing_spaces = true;
                    continue;
                }
                if split_offset != item_result.EndOffset()
                    && item.BidiLevel() != self.base_direction_ as u8
                {
                    let source_shape_result = item_result.shape_result.Get();
                    let previous_inline_size = item_result.inline_size;
                    let start = item_result.StartOffset();
                    let end = item_result.EndOffset();
                    item_result.text_offset.end = split_offset;
                    item_result.shape_result =
                        Member::from_ptr(ShapeResultView::CreateFromViewRange(
                            source_shape_result,
                            start,
                            split_offset,
                        ));
                    item_result.inline_size =
                        unsafe { &*item_result.shape_result.Get() }.SnappedWidth();
                    debug_assert!(item_result.inline_size <= previous_inline_size);

                    let mut trailing = InlineItemResult::new(
                        item,
                        item_result.item_index,
                        &TextOffsetRange::new(split_offset, end),
                        item_result.break_anywhere_if_overflow,
                        item_result.should_create_line_box,
                        item_result.has_unpositioned_floats,
                    );
                    trailing.has_only_bidi_trailing_spaces = true;
                    trailing.shape_result = Member::from_ptr(ShapeResultView::CreateFromViewRange(
                        source_shape_result,
                        split_offset,
                        end,
                    ));
                    trailing.inline_size = previous_inline_size - item_result.inline_size;
                    spaces_result = Some(trailing);
                }
            }
            if let Some(spaces_result) = spaces_result {
                results.insert(result_index as usize + 1, spaces_result);
            }
            break;
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:2046-2054
    fn HandleEmptyText(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        self.AddEmptyItem(item, line_info);
        self.MoveToNextOfItem(item);
    }

    // cpp: layoutng_inline/line_breaker.cc:3970-4021
    fn HandleOpenTag(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        debug_assert_eq!(item.Type(), InlineItemType::kOpenTag);

        let item_result = self.AddItem(item, line_info);
        debug_assert!(!item.Style().is_null());
        let style = unsafe { &*item.Style() };
        if Self::ComputeOpenTagResult(
            item,
            unsafe { &*self.constraint_space_ },
            self.is_svg_text_,
            unsafe { &mut *item_result },
        ) {
            let inline_size = unsafe { &*item_result }.inline_size;
            if inline_size < LayoutUnit::default() && self.state_ == Some(LineBreakState::kTrailing)
            {
                let available_width = self.AvailableWidthToFit();
                if self.position_ > available_width
                    && self.position_ + inline_size <= available_width
                {
                    self.state_ = Some(LineBreakState::kContinue);
                }
            }

            self.position_ += inline_size;
            if !unsafe { &*item_result }.should_create_line_box && !item.IsEmptyItem() {
                unsafe { &mut *item_result }.should_create_line_box = true;
            }
        }

        if style.BoxDecorationBreak() == EBoxDecorationBreak::kClone {
            self.has_cloned_box_decorations_ = true;
            self.disable_score_line_break_ = true;
            self.cloned_box_decorations_count_ += 1;
            let result = unsafe { &*item_result };
            self.cloned_box_decorations_end_size_ +=
                result.margins.inline_end + result.borders.inline_end + result.padding.inline_end;
            self.UpdateAvailableWidthFromBaseAvailableWidth();
        }

        let was_auto_wrap = self.auto_wrap_;
        self.SetCurrentStyle(style);
        self.MoveToNextOfItem(item);

        debug_assert!(!unsafe { &*item_result }.can_break_after);
        let item_results = line_info.Results();
        if !was_auto_wrap && self.auto_wrap_ && item_results.size() >= 2 {
            if self.IsPreviousItemOfType(InlineItemType::kText) {
                let last_index = item_results.size() as usize - 2;
                ComputeCanBreakAfter(
                    &mut line_info.MutableResults()[last_index],
                    self.auto_wrap_,
                    &self.break_iterator_,
                );
            }
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:4023-4087
    fn HandleCloseTag(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        let item_result = self.AddItem(item, line_info);

        if !self.is_svg_text_ {
            debug_assert!(!item.Style().is_null());
            let style = unsafe { &*item.Style() };
            let inline_size =
                ComputeInlineEndSize(unsafe { &*self.constraint_space_ }, item.Style());
            unsafe { &mut *item_result }.inline_size = inline_size;
            self.position_ += inline_size;

            if !unsafe { &*item_result }.should_create_line_box && !item.IsEmptyItem() {
                unsafe { &mut *item_result }.should_create_line_box = true;
            }

            if style.BoxDecorationBreak() == EBoxDecorationBreak::kClone {
                debug_assert!(self.cloned_box_decorations_count_ > 0);
                self.cloned_box_decorations_count_ -= 1;
                debug_assert!(self.cloned_box_decorations_end_size_ >= inline_size);
                self.cloned_box_decorations_end_size_ -= inline_size;
                self.UpdateAvailableWidthFromBaseAvailableWidth();
            }
        }
        let layout_object = item.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        let parent = unsafe { &*layout_object }.Parent();
        debug_assert!(!parent.is_null());
        let was_auto_wrap = self.auto_wrap_;
        self.SetCurrentStyle(unsafe { &*parent }.StyleRef());
        self.MoveToNextOfItem(item);

        let item_results = line_info.Results();
        if item_results.size() >= 2 {
            let last_index = item_results.size() as usize - 2;
            let last = &item_results[last_index];
            let last_item = unsafe { &*last.item.Get() };
            if IsA::<LayoutTextCombine>(last_item.GetLayoutObject()) {
                unsafe { &mut *item_result }.can_break_after = last.can_break_after;
                return;
            }
            if last.can_break_after {
                unsafe { &mut *item_result }.can_break_after = true;
                line_info.MutableResults()[last_index].can_break_after = false;
                return;
            }
            let end_offset = unsafe { &*item_result }.EndOffset() as usize;
            let units = self.Text().Span16().unwrap_or_default();
            if was_auto_wrap {
                let preceded_by_breakable_space =
                    end_offset > 0 && IsBreakableSpace(units[end_offset - 1]);
                unsafe { &mut *item_result }.can_break_after =
                    units.get(end_offset).copied().is_some_and(IsBreakableSpace)
                        && (!unsafe { &*self.current_style_ }.ShouldBreakOnlyAfterWhiteSpace()
                            || preceded_by_breakable_space)
                        && (!RuntimeEnabledFeatures::LineBreakAfterSpaceBeforeOpenTagEnabled()
                            || !self.IsNextNonBidiControlItemOpenTag());
                return;
            }
            debug_assert!(end_offset > 0);
            if self.auto_wrap_ && !IsBreakableSpace(units[end_offset - 1]) {
                ComputeCanBreakAfter(
                    unsafe { &mut *item_result },
                    self.auto_wrap_,
                    &self.break_iterator_,
                );
            }
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:3915-3920
    fn HandleInitialLetter(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        self.HandleAtomicInline(item, line_info);
    }

    // cpp: layoutng_inline/line_breaker.cc:3922-3948
    fn HandleOutOfFlowPositioned(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        debug_assert_eq!(item.Type(), InlineItemType::kOutOfFlowPositioned);
        let item_result = self.AddItem(item, line_info);

        debug_assert!(!unsafe { &*item_result }.can_break_after);
        let item_results = line_info.MutableResults();
        if item_results.size() >= 2 {
            let last_index = item_results.size() as usize - 2;
            let last = &item_results[last_index];
            if last.IsEmptyText() && !last.can_break_after {
                ComputeCanBreakAfter(
                    unsafe { &mut *item_result },
                    self.auto_wrap_,
                    &self.break_iterator_,
                );
            } else {
                unsafe { &mut *item_result }.can_break_after = last.can_break_after;
            }
        }

        self.MoveToNextOfItem(item);
    }

    // cpp: layoutng_inline/line_breaker.cc:3013-3057
    fn HandleBidiControlItem(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        debug_assert_eq!(item.Length(), 1);
        let character = self.Text().Span16().unwrap_or_default()[item.StartOffset() as usize];
        // uchar::kPopDirectionalIsolate and kPopDirectionalFormatting.
        let is_pop = character == 0x2069 || character == 0x202C;
        if is_pop {
            if !line_info.Results().empty() {
                let item_result = self.AddItem(item, line_info);
                let item_results = line_info.MutableResults();
                let last_index = item_results.size() as usize - 2;
                let last = &mut item_results[last_index];
                if last.can_break_after {
                    unsafe { &mut *item_result }.can_break_after = last.can_break_after;
                    last.can_break_after = false;
                } else {
                    ComputeCanBreakAfter(
                        unsafe { &mut *item_result },
                        self.auto_wrap_,
                        &self.break_iterator_,
                    );
                }
            } else {
                self.AddItem(item, line_info);
            }
        } else {
            if self.state_ == Some(LineBreakState::kTrailing)
                && CanBreakAfterLast(line_info.Results())
            {
                debug_assert!(!line_info.IsLastLine());
                self.MoveToNextOfItem(item);
                self.state_ = Some(LineBreakState::kDone);
                return;
            }
            let item_result = self.AddItem(item, line_info);
            debug_assert!(!unsafe { &*item_result }.can_break_after);
        }
        self.MoveToNextOfItem(item);
    }

    // cpp: layoutng_inline/line_breaker.cc:2868-2953
    fn HandleForcedLineBreak(&mut self, item: Option<&InlineItem>, line_info: &mut LineInfo) {
        if self.HandleOverflowIfNeeded(line_info) {
            return;
        }

        if let Some(item) = item {
            debug_assert_eq!(item.TextType(), TextItemType::kForcedLineBreak);
            debug_assert_eq!(
                self.Text().Span16().unwrap_or_default()[item.StartOffset() as usize],
                0x0A
            );

            let constraint_space = unsafe { &*self.constraint_space_ };
            let layout_object = item.GetLayoutObject();
            if constraint_space.HasBlockFragmentation()
                && !layout_object.is_null()
                && unsafe { &*layout_object }.IsBR()
                && unsafe { &*self.exclusion_space_ }.NeedsClearancePastFragmentainer(
                    unsafe { &*item.Style() }
                        .ClearWithContainingStyle(unsafe { &*self.current_style_ }),
                )
                && !line_info.Results().empty()
            {
                self.state_ = Some(LineBreakState::kDone);
                return;
            }

            let item_result = self.AddItem(item, line_info);
            let result = unsafe { &mut *item_result };
            result.should_create_line_box = true;
            result.has_only_pre_wrap_trailing_spaces = true;
            result.has_only_bidi_trailing_spaces = true;
            result.can_break_after = true;
            self.MoveToNextOfItem(item);

            let items = self.Items() as *const InlineItems;
            while !self.IsAtEnd() {
                let next_item = unsafe { &*(&*items).at(self.current_.item_index).Get() };
                if next_item.Type() == InlineItemType::kCloseTag {
                    self.HandleCloseTag(next_item, line_info);
                    continue;
                }
                if next_item.Type() == InlineItemType::kText && next_item.Length() == 0 {
                    self.HandleEmptyText(next_item, line_info);
                    continue;
                }
                break;
            }
        }

        if self.HasHyphen() {
            let hyphen_width = self.RemoveHyphen(line_info.MutableResults());
            self.position_ -= hyphen_width;
        }
        self.is_forced_break_ = true;
        line_info.SetHasForcedBreak();
        line_info.SetIsLastLine(true);
        self.state_ = Some(LineBreakState::kDone);
    }

    // cpp: layoutng_inline/line_breaker.cc:2956-3011
    fn HandleControlItem(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        debug_assert!(item.Length() >= 1);
        if item.TextType() == TextItemType::kForcedLineBreak {
            self.HandleForcedLineBreak(Some(item), line_info);
            return;
        }

        debug_assert_eq!(item.TextType(), TextItemType::kFlowControl);
        let character = self.Text().Span16().unwrap_or_default()[item.StartOffset() as usize];
        match character {
            0x09 => {
                debug_assert!(!item.Style().is_null());
                let style = unsafe { &*item.Style() };
                if unsafe { &*style.GetFont() }.PrimaryFont().is_null() {
                    self.HandleEmptyText(item, line_info);
                    return;
                }
                let font = if RuntimeEnabledFeatures::TabSizeAncestorEnabled() {
                    self.node_.FontForTab()
                } else {
                    unsafe { &*style.GetFont() }
                };
                let position = if RuntimeEnabledFeatures::TabAlignmentWithFloatsEnabled() {
                    self.position_ + self.ComputeFloatOffset()
                } else {
                    self.position_
                } + self.tab_stop_offset_;
                let shape_result = ShapeResult::CreateForTabulationCharacters(
                    font,
                    item.Direction(),
                    style.GetTabSize(),
                    position.ToFloat(),
                    item.StartOffset(),
                    item.Length(),
                );
                self.HandleText(item, unsafe { &*shape_result }, line_info);
                return;
            }
            0x200B => {
                let item_result = self.AddItem(item, line_info);
                if !item.IsGeneratedForLineBreak() {
                    unsafe { &mut *item_result }.should_create_line_box = true;
                }
                unsafe { &mut *item_result }.can_break_after = true;
            }
            0x0D | 0x0C => {
                self.HandleEmptyText(item, line_info);
                return;
            }
            _ => unreachable!("unexpected inline flow-control character"),
        }
        self.MoveToNextOfItem(item);
    }

    // cpp: layoutng_inline/line_breaker.cc:895-1012
    pub fn NextLine(&mut self, line_info: &mut LineInfo) {
        self.PrepareNextLine(line_info);

        if !self.break_token_.is_null() && unsafe { &*self.break_token_ }.IsInParallelBlockFlow() {
            let break_token = unsafe { &*self.break_token_ };
            let block_break_token = break_token.GetBlockBreakToken();
            debug_assert!(!block_break_token.is_null());
            let item = unsafe { &*self.Items()[break_token.StartItemIndex() as usize].Get() };
            let input_node = unsafe { &*block_break_token }.InputNode();
            debug_assert_eq!(item.GetLayoutObject(), input_node.GetLayoutBox() as *mut _);
            if unsafe { &*input_node.GetLayoutBox() }.IsFloating() {
                self.HandleFloat(item, block_break_token, line_info);
            } else {
                debug_assert_eq!(item.Type(), InlineItemType::kBlockInInline);
                self.HandleBlockInInline(item, block_break_token, line_info);
            }
            self.state_ = Some(LineBreakState::kDone);
            line_info.SetIsEmptyLine();
            return;
        }

        self.BreakLine(line_info);

        if self.HasHyphen() {
            self.FinalizeHyphen(line_info.MutableResults());
        }
        if !self.disable_trailing_whitespace_collapsing_ {
            self.RemoveTrailingCollapsibleSpace(line_info);
            self.SplitTrailingBidiPreservedSpace(line_info);
        }

        let item_results = line_info.Results();
        #[cfg(debug_assertions)]
        for result in item_results {
            result.CheckConsistency(self.mode_ == LineBreakerMode::kMinContent);
        }

        let should_create_line_box = ShouldCreateLineBox(item_results)
            || (self.force_non_empty_if_last_line_ && line_info.IsLastLine())
            || self.mode_ != LineBreakerMode::kContent;

        if self.line_clamp_ellipsis_width_ != LayoutUnit::default()
            && (!should_create_line_box || line_info.IsBlockInInline())
        {
            self.line_clamp_ellipsis_width_ = LayoutUnit::default();
        }

        if self.line_clamp_ellipsis_width_ != LayoutUnit::default() && !self.CanFitOnLine() {
            self.Rewind(0, line_info);
            line_info.SetIsLastLine(false);
            self.disable_bisect_line_break_ = false;
        }

        if !should_create_line_box {
            let block = To::<LayoutBlockFlow>(self.node_.GetLayoutBox());
            if unsafe { &*block }.HasLineIfEmpty() {
                line_info.SetHasLineEvenIfEmpty();
            } else {
                line_info.SetIsEmptyLine();
            }
        }

        line_info.SetEndItemIndex(self.current_.item_index);
        if !self.disable_trailing_whitespace_collapsing_ {
            debug_assert_ne!(self.trailing_whitespace_, WhitespaceState::kUnknown);
            if self.trailing_whitespace_ == WhitespaceState::kPreserved {
                line_info.SetHasTrailingSpaces();
            }
        }

        if self.override_available_width_ != LayoutUnit::default() {
            self.override_available_width_ = LayoutUnit::default();
            self.UpdateAvailableWidth();
        }
        self.ComputeLineLocation(line_info);
        debug_assert!(self.ruby_break_token_.is_null());
        let results = line_info.Results();
        if !results.empty() && results.last().unwrap().IsRubyColumn() {
            let ruby_column =
                results.last().unwrap().ruby_column.Get() as *const InlineItemResultRubyColumn;
            self.ruby_break_token_ = unsafe { &*ruby_column }.end_ruby_break_token.Get();
        }
        if self.mode_ == LineBreakerMode::kContent {
            let break_token = self.CreateBreakToken(line_info);
            line_info.SetBreakToken(break_token);
        }

        #[cfg(feature = "expensive_dchecks")]
        if self.break_at_.IsNotZero() {
            debug_assert!(line_info.End().GreaterOrEqual(&self.break_at_.end));
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:623-630
    fn HandleOverflowIfNeeded(&mut self, line_info: &mut LineInfo) -> bool {
        if self.state_ == Some(LineBreakState::kContinue) && !self.CanFitOnLine() {
            self.HandleOverflow(line_info);
            return true;
        }
        false
    }

    // cpp: layoutng_inline/line_breaker.cc:1013-1152
    fn BreakLine(&mut self, line_info: &mut LineInfo) {
        debug_assert!(!line_info.IsLastLine());
        let items = self.Items() as *const InlineItems;
        self.state_ = Some(if self.mode_ == LineBreakerMode::kMinContent {
            LineBreakState::kOverflow
        } else {
            LineBreakState::kContinue
        });
        self.trailing_whitespace_ = self.initial_whitespace_;

        while self.state_ != Some(LineBreakState::kDone) {
            if !self.ruby_break_token_.is_null() {
                self.HandleRuby(line_info, kIndefiniteSize);
                self.HandleOverflowIfNeeded(line_info);
                continue;
            }

            debug_assert!(self.current_.item_index <= unsafe { &*items }.size());
            if self.IsAtEnd() {
                if self.HandleOverflowIfNeeded(line_info) && !self.IsAtEnd() {
                    continue;
                }
                if self.HasHyphen() {
                    let hyphen_width = self.RemoveHyphen(line_info.MutableResults());
                    self.position_ -= hyphen_width;
                }
                line_info.SetIsLastLine(true);
                return;
            }
            if self.break_at_.IsNotZero() && self.current_.GreaterOrEqual(&self.break_at_.offset) {
                return;
            }

            if self.state_ == Some(LineBreakState::kOverflow)
                && CanBreakAfterLast(line_info.Results())
            {
                self.state_ = Some(LineBreakState::kTrailing);
            }

            let item = unsafe { &*(&*items).at(self.current_.item_index).Get() };
            if item.Type() == InlineItemType::kText {
                if item.Length() != 0 {
                    self.HandleText(item, unsafe { &*item.TextShapeResult() }, line_info);
                } else {
                    self.HandleEmptyText(item, line_info);
                }
                #[cfg(debug_assertions)]
                if let Some(result) = line_info.Results().last() {
                    result.CheckConsistency(true);
                }
                continue;
            }
            if item.Type() == InlineItemType::kOpenTag {
                self.HandleOpenTag(item, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kCloseTag {
                self.HandleCloseTag(item, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kControl {
                self.HandleControlItem(item, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kFloating {
                self.HandleFloat(item, std::ptr::null(), line_info);
                continue;
            }
            if item.Type() == InlineItemType::kBidiControl {
                self.HandleBidiControlItem(item, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kBlockInInline {
                let block_break_token = if self.break_token_.is_null() {
                    std::ptr::null()
                } else {
                    unsafe { &*self.break_token_ }.GetBlockBreakToken()
                };
                self.HandleBlockInInline(item, block_break_token, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kCloseRubyColumn
                || item.Type() == InlineItemType::kRubyLinePlaceholder
            {
                self.AddItem(item, line_info);
                self.MoveToNextOfItem(item);
                continue;
            }

            debug_assert!(!IsTrailableItemType(item.Type()));
            if self.state_ == Some(LineBreakState::kTrailing) {
                debug_assert!(!line_info.IsLastLine());
                return;
            }

            if item.Type() == InlineItemType::kAtomicInline {
                self.HandleAtomicInline(item, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kInitialLetterBox {
                self.HandleInitialLetter(item, line_info);
                continue;
            }
            if item.Type() == InlineItemType::kOpenRubyColumn {
                let index = self.current_.item_index as usize;
                let items_ref = unsafe { &*items };
                let next_type = unsafe { &*items_ref[index + 1].Get() }.Type();
                let next_next_type = unsafe { &*items_ref[index + 2].Get() }.Type();
                if next_type == InlineItemType::kRubyLinePlaceholder
                    && (next_next_type == InlineItemType::kCloseRubyColumn
                        || (next_next_type == InlineItemType::kRubyLinePlaceholder
                            && unsafe { &*items_ref[index + 3].Get() }.Type()
                                == InlineItemType::kCloseRubyColumn))
                {
                    self.AddItem(item, line_info);
                    self.MoveToNextOfItem(item);
                    continue;
                }
                if self.HandleRuby(line_info, kIndefiniteSize) {
                    self.HandleOverflowIfNeeded(line_info);
                } else {
                    self.AddItem(item, line_info);
                    self.MoveToNextOfItem(item);
                }
                continue;
            }
            if item.Type() == InlineItemType::kOutOfFlowPositioned {
                self.HandleOutOfFlowPositioned(item, line_info);
            } else if item.Length() != 0 {
                unreachable!("unexpected non-empty inline item");
            } else if item.Type() == InlineItemType::kListMarker {
                let item_result = self.AddItem(item, line_info);
                self.force_non_empty_if_last_line_ = true;
                debug_assert!(!unsafe { &*item_result }.can_break_after);
                self.MoveToNextOfItem(item);
            } else {
                unreachable!("unexpected inline item type");
            }
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:1154-1168
    fn ComputeLineLocation(&self, line_info: &mut LineInfo) {
        let available_width = self.base_available_width_;
        line_info.SetWidth(
            available_width + self.line_clamp_ellipsis_width_,
            self.position_
                + self.cloned_box_decorations_end_size_
                + self.line_clamp_ellipsis_width_,
        );
        line_info.SetBfcOffset(&BfcOffset::new(
            self.line_opportunity_.line_left_offset,
            self.line_opportunity_.bfc_block_offset,
        ));
        if self.mode_ == LineBreakerMode::kContent {
            line_info.UpdateTextAlign();
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:467-557
    pub fn new(
        node: InlineNode,
        mode: LineBreakerMode,
        space: &ConstraintSpace,
        line_opportunity: &LineLayoutOpportunity,
        leading_floats: &LeadingFloats,
        break_token: *const InlineBreakToken,
        column_spanner_path: *const ColumnSpannerPath,
        exclusion_space: *mut ExclusionSpace,
    ) -> Self {
        let is_initial_letter_box = node.IsInitialLetterBox();
        let is_svg_text = node.IsSvgText();
        let is_text_combine = node.IsTextCombine();
        let is_first_formatted_line = (break_token.is_null()
            || !unsafe { &*break_token }.IsPastFirstFormattedLine())
            && node.CanContainFirstFormattedLine();
        let use_first_line_style = is_first_formatted_line && node.UseFirstLineStyleItemsData();
        let sticky_images_quirk =
            mode != LineBreakerMode::kContent && node.IsStickyImagesQuirkForContentSize();
        let items_data = node.ItemsData(use_first_line_style) as *const InlineItemsData;
        let text_content = if sticky_images_quirk {
            InlineNode::TextContentForStickyImagesQuirk(unsafe { &*items_data })
        } else {
            unsafe { &*items_data }.text_content.clone()
        };
        let mut breaker = Self {
            state_: None,
            current_: InlineItemTextIndex::default(),
            svg_addressable_offset_: 0,
            break_at_: LineBreakPoint::default(),
            trailing_whitespace_: WhitespaceState::kUnknown,
            initial_whitespace_: WhitespaceState::kLeading,
            position_: LayoutUnit::default(),
            tab_stop_offset_: LayoutUnit::default(),
            applied_text_indent_: LayoutUnit::default(),
            available_width_: LayoutUnit::default(),
            base_available_width_: LayoutUnit::default(),
            line_opportunity_: *line_opportunity,
            node_: node.clone(),
            mode_: mode,
            is_initial_letter_box_: is_initial_letter_box,
            is_svg_text_: is_svg_text,
            is_text_combine_: is_text_combine,
            is_first_formatted_line_: is_first_formatted_line,
            use_first_line_style_: use_first_line_style,
            auto_wrap_: false,
            disallow_auto_wrap_: false,
            break_anywhere_if_overflow_: false,
            override_break_anywhere_: false,
            disable_phrase_: false,
            disable_score_line_break_: false,
            disable_bisect_line_break_: false,
            disable_trailing_whitespace_collapsing_: false,
            force_non_empty_if_last_line_: false,
            is_forced_break_: false,
            previous_line_had_forced_break_: false,
            sticky_images_quirk_: sticky_images_quirk,
            maybe_have_end_overhang_: false,
            needs_svg_segmentation_: false,
            resume_block_in_inline_in_same_flow_: false,
            #[cfg(debug_assertions)]
            has_considered_creating_break_token_: false,
            items_data_: items_data,
            end_item_index_: unsafe { &*items_data }.items.size(),
            text_content_: text_content.clone(),
            constraint_space_: space,
            exclusion_space_: exclusion_space,
            break_token_: break_token,
            ruby_break_token_: std::ptr::null(),
            column_spanner_path_: column_spanner_path,
            current_style_: std::ptr::null(),
            break_iterator_: LazyLineBreakIterator::new(
                &text_content,
                std::ptr::null(),
                LineBreakType::kNormal,
            ),
            shaper_: HarfBuzzShaper::new(text_content.clone()),
            spacing_: ShapeResultSpacing::new(&text_content, is_svg_text),
            hyphenation_: std::ptr::null(),
            hyphen_index_: None,
            has_any_hyphens_: false,
            trailing_collapsible_space_: None,
            override_available_width_: LayoutUnit::default(),
            leading_floats_: leading_floats,
            leading_floats_index_: 0,
            max_size_cache_: std::ptr::null_mut(),
            depends_on_block_constraints_out_: std::ptr::null_mut(),
            base_direction_: node.BaseDirection(),
            cloned_box_decorations_count_: 0,
            cloned_box_decorations_initial_size_: LayoutUnit::default(),
            cloned_box_decorations_end_size_: LayoutUnit::default(),
            has_cloned_box_decorations_: false,
            last_rewind_: None,
            svg_resolved_iterator_: None,
            line_clamp_ellipsis_width_: LayoutUnit::default(),
            parent_breaker_: std::ptr::null(),
        };
        breaker.UpdateAvailableWidth();
        if is_svg_text {
            let char_data = node.SvgCharacterDataList();
            if node.SvgTextPathRangeList().is_empty()
                && node.SvgTextLengthRangeList().is_empty()
                && (char_data.is_empty() || (char_data.len() == 1 && char_data[0].0 == 0))
            {
                breaker.needs_svg_segmentation_ = false;
            } else {
                breaker.needs_svg_segmentation_ = true;
                let iterator = ResolvedTextLayoutAttributesIterator::new(char_data);
                // C++ stores a unique_ptr borrowing GC-owned data. The data
                // is stable for the lifetime of this stack-only breaker.
                breaker.svg_resolved_iterator_ = Some(Box::new(unsafe {
                    std::mem::transmute::<
                        ResolvedTextLayoutAttributesIterator<'_>,
                        ResolvedTextLayoutAttributesIterator<'static>,
                    >(iterator)
                }));
            }
        }
        breaker.disallow_auto_wrap_ = is_svg_text || is_text_combine || is_initial_letter_box;
        if break_token.is_null() {
            return breaker;
        }
        let token = unsafe { &*break_token };
        let initial_style = token.Style();
        if initial_style.is_null() {
            debug_assert_eq!(token.StartItemIndex(), 0);
            debug_assert_eq!(token.StartTextOffset(), 0);
            debug_assert!(!token.IsForcedBreak());
            debug_assert_eq!(breaker.current_, *token.Start());
            debug_assert_eq!(breaker.is_forced_break_, token.IsForcedBreak());
            return breaker;
        }
        breaker.current_ = *token.Start();
        breaker.ruby_break_token_ = token.RubyData();
        breaker
            .break_iterator_
            .SetStartOffset(breaker.current_.text_offset);
        breaker.is_forced_break_ = token.IsForcedBreak();
        unsafe { &*items_data }.AssertOffsetAt(&breaker.current_);
        breaker.SetCurrentStyle(unsafe { &*initial_style });
        breaker
    }

    // cpp: layoutng_inline/line_breaker.h:52-52
    pub fn ItemsData(&self) -> &InlineItemsData {
        unsafe { &*self.items_data_ }
    }

    // cpp: layoutng_inline/line_breaker.h:56-56
    pub fn HasClonedBoxDecorations(&self) -> bool {
        self.has_cloned_box_decorations_
    }

    // cpp: layoutng_inline/line_breaker.h:62-62
    pub fn IsFinished(&self) -> bool {
        self.current_.item_index >= self.ItemsData().items.size()
    }

    // cpp: layoutng_inline/line_breaker.h:68-73
    pub fn ShouldDisableScoreLineBreak(&self) -> bool {
        self.disable_score_line_break_
    }
    pub fn ShouldDisableBisectLineBreak(&self) -> bool {
        self.disable_bisect_line_break_
    }

    // cpp: layoutng_inline/line_breaker.cc:561-565
    pub fn SetLineOpportunity(&mut self, line_opportunity: &LineLayoutOpportunity) {
        self.line_opportunity_ = *line_opportunity;
        self.UpdateAvailableWidth();
    }

    // cpp: layoutng_inline/line_breaker.cc:567-571
    pub fn OverrideAvailableWidth(&mut self, available_width: LayoutUnit) {
        debug_assert!(available_width >= LayoutUnit::default());
        self.override_available_width_ = available_width;
        self.UpdateAvailableWidth();
    }

    // cpp: layoutng_inline/line_breaker.cc:573-576
    pub fn SetBreakAt(&mut self, offset: &LineBreakPoint) {
        self.break_at_ = *offset;
        self.OverrideAvailableWidth(LayoutUnit::NearlyMax());
    }

    // cpp: layoutng_inline/line_breaker.h:87-90
    pub fn SetLineClampEllipsisWidth(&mut self, width: LayoutUnit) {
        debug_assert!(RuntimeEnabledFeatures::CSSLineClampLineBreakingEllipsisEnabled());
        self.line_clamp_ellipsis_width_ = width;
        self.UpdateAvailableWidth();
    }

    // cpp: layoutng_inline/line_breaker.cc:630-637
    pub fn SetIntrinsicSizeOutputs(
        &mut self,
        max_size_cache: *mut MaxSizeCache,
        depends_on_block_constraints: *mut bool,
    ) {
        debug_assert_ne!(self.mode_, LineBreakerMode::kContent);
        debug_assert!(!max_size_cache.is_null());
        self.max_size_cache_ = max_size_cache;
        self.depends_on_block_constraints_out_ = depends_on_block_constraints;
    }

    // cpp: layoutng_inline/line_breaker.h:117-119
    pub fn TrailingWhitespaceForTesting(&self) -> WhitespaceState {
        self.trailing_whitespace_
    }

    // cpp: layoutng_inline/line_breaker.cc:4698-4707
    pub fn SetInputRange(
        &mut self,
        start: InlineItemTextIndex,
        end_item_index: u32,
        initial_whitespace_state: WhitespaceState,
        parent: *const LineBreaker,
    ) {
        self.current_ = start;
        self.end_item_index_ = end_item_index;
        self.initial_whitespace_ = initial_whitespace_state;
        self.parent_breaker_ = parent;
    }

    // cpp: layoutng_inline/line_breaker.h:148-148
    fn IsSubLineBreaker(&self) -> bool {
        self.end_item_index_ != self.ItemsData().items.size()
    }

    // cpp: layoutng_inline/line_breaker.h:150-151
    fn Text(&self) -> &String {
        &self.text_content_
    }
    fn Items(&self) -> &InlineItems {
        &self.ItemsData().items
    }

    // cpp: layoutng_inline/line_breaker.h:302-302
    fn IsAtEnd(&self) -> bool {
        self.current_.item_index >= self.end_item_index_
    }

    // cpp: layoutng_inline/line_breaker.h:305-318
    fn AvailableWidth(&self) -> LayoutUnit {
        self.available_width_
    }
    fn AvailableWidthToFit(&self) -> LayoutUnit {
        self.AvailableWidth().AddEpsilon()
    }
    fn RemainingAvailableWidth(&self) -> LayoutUnit {
        self.AvailableWidthToFit() - self.position_
    }
    fn CanFitOnLine(&self) -> bool {
        self.position_ <= self.AvailableWidthToFit()
            || (!self.parent_breaker_.is_null() && !self.auto_wrap_)
    }

    // cpp: layoutng_inline/line_breaker.h:323-323
    fn HasHyphen(&self) -> bool {
        self.hyphen_index_.is_some()
    }

    // The three C++ AddHyphen overloads use distinct Rust names. The pointer
    // overload retains its vector-membership contract with an index check.
    // cpp: layoutng_inline/line_breaker.cc:735-755
    fn AddHyphenAt(&mut self, item_results: &mut InlineItemResults, index: u32) -> LayoutUnit {
        debug_assert!(!self.HasHyphen());
        debug_assert!(index < item_results.size());
        self.hyphen_index_ = Some(index);

        let item_result = &mut item_results[index as usize];
        if !item_result.hyphen.IsPresent() {
            item_result.ShapeHyphen();
            self.has_any_hyphens_ = true;
        }
        debug_assert!(item_result.hyphen.IsPresent());
        debug_assert!(self.has_any_hyphens_);

        let hyphen_inline_size = item_result.hyphen.InlineSize();
        item_result.inline_size += hyphen_inline_size;
        hyphen_inline_size
    }

    // cpp: layoutng_inline/line_breaker.cc:757-763
    fn AddHyphenAtCheckedIndex(
        &mut self,
        item_results: &mut InlineItemResults,
        index: u32,
    ) -> LayoutUnit {
        debug_assert!(!item_results[index as usize].item.Get().is_null());
        self.AddHyphenAt(item_results, index)
    }

    // cpp: layoutng_inline/line_breaker.cc:765-771
    fn AddHyphenForResult(
        &mut self,
        item_results: &mut InlineItemResults,
        item_result: *mut InlineItemResult,
    ) -> LayoutUnit {
        let offset = unsafe { item_result.offset_from(item_results.as_ptr()) };
        let index = u32::try_from(offset).expect("hyphen result is outside its item vector");
        debug_assert!(index < item_results.size());
        debug_assert!(std::ptr::eq(item_result, &item_results[index as usize]));
        self.AddHyphenAt(item_results, index)
    }

    // cpp: layoutng_inline/line_breaker.cc:777-788
    fn RemoveHyphen(&mut self, item_results: &mut InlineItemResults) -> LayoutUnit {
        debug_assert!(self.HasHyphen());
        let index = self.hyphen_index_.expect("hyphen index required");
        let item_result = &mut item_results[index as usize];
        debug_assert!(item_result.hyphen.IsPresent());
        let hyphen_inline_size = item_result.hyphen.InlineSize();
        item_result.inline_size -= hyphen_inline_size;
        // Keep the shaped hyphen for a possible rewind, as the source does.
        self.hyphen_index_ = None;
        hyphen_inline_size
    }

    // cpp: layoutng_inline/line_breaker.cc:792-808
    fn RestoreLastHyphen(&mut self, item_results: &mut InlineItemResults) {
        debug_assert!(!self.HasHyphen());
        debug_assert!(self.has_any_hyphens_);
        for index in (0..item_results.size()).rev() {
            let item_result = &item_results[index as usize];
            debug_assert!(!item_result.item.Get().is_null());
            if item_result.hyphen.IsPresent() {
                self.AddHyphenAt(item_results, index);
                return;
            }
            let item_type = unsafe { &*item_result.item.Get() }.Type();
            if item_type == InlineItemType::kText || item_type == InlineItemType::kAtomicInline {
                return;
            }
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:811-818
    fn FinalizeHyphen(&mut self, item_results: &mut InlineItemResults) {
        debug_assert!(self.HasHyphen());
        let index = self.hyphen_index_.expect("hyphen index required");
        let item_result = &mut item_results[index as usize];
        debug_assert!(item_result.hyphen.IsPresent());
        item_result.is_hyphenated = true;
    }

    // cpp: layoutng_inline/line_breaker.h:291-291
    fn ResetRewindLoopDetector(&mut self) {
        self.last_rewind_ = None;
    }

    // cpp: layoutng_inline/line_breaker.cc:811-893
    fn PrepareNextLine(&mut self, line_info: &mut LineInfo) {
        line_info.Reset();
        debug_assert!(line_info.Results().empty());

        if !self.parent_breaker_.is_null() {
            let parent = unsafe { &*self.parent_breaker_ };
            self.previous_line_had_forced_break_ = parent.previous_line_had_forced_break_;
            self.is_forced_break_ = parent.is_forced_break_;
            self.is_first_formatted_line_ = parent.is_first_formatted_line_;
            self.use_first_line_style_ = parent.use_first_line_style_;
            self.items_data_ = parent.items_data_;
        } else if !self.current_.IsZero() {
            self.previous_line_had_forced_break_ = self.is_forced_break_;
            self.is_forced_break_ = false;
            if self.break_token_.is_null()
                || self.current_ != *unsafe { &*self.break_token_ }.Start()
            {
                self.is_first_formatted_line_ = false;
                self.use_first_line_style_ = false;
            }
        }

        line_info.SetStart(&self.current_);
        line_info.SetIsFirstFormattedLine(self.is_first_formatted_line_);
        line_info
            .SetIsStartOfParagraph(self.current_.IsZero() || self.previous_line_had_forced_break_);
        line_info.SetLineStyle(
            &self.node_,
            unsafe { &*self.items_data_ },
            self.use_first_line_style_,
        );

        debug_assert_eq!(line_info.TextIndent(), LayoutUnit::default());
        let style = line_info.LineStyle();
        if ShouldApplyTextIndent(
            style,
            self.is_first_formatted_line_,
            self.previous_line_had_forced_break_,
        ) && !self.IsSubLineBreaker()
        {
            let length = style.TextIndent();
            let mut maximum_value = LayoutUnit::default();
            if length.HasPercent() && self.mode_ == LineBreakerMode::kContent {
                maximum_value = unsafe { &*self.constraint_space_ }
                    .AvailableSize()
                    .inline_size;
            }
            line_info.SetTextIndent(MinimumValueForLength(length, maximum_value));
        }

        self.override_break_anywhere_ = false;
        self.disable_phrase_ = false;
        self.disable_score_line_break_ = false;
        self.disable_bisect_line_break_ = false;
        if self.current_style_.is_null() {
            self.SetCurrentStyle(line_info.LineStyle());
        }
        self.ComputeBaseDirection();
        line_info.SetBaseDirection(self.base_direction_);
        self.hyphen_index_ = None;
        self.has_any_hyphens_ = false;
        self.resume_block_in_inline_in_same_flow_ = false;

        self.applied_text_indent_ = line_info.TextIndent();
        self.position_ = self.applied_text_indent_;

        self.has_cloned_box_decorations_ = false;
        if (!self.break_token_.is_null()
            && unsafe { &*self.break_token_ }.HasClonedBoxDecorations())
            || self.cloned_box_decorations_count_ != 0
        {
            self.RecalcClonedBoxDecorations();
        }

        self.ResetRewindLoopDetector();
        #[cfg(debug_assertions)]
        {
            self.has_considered_creating_break_token_ = false;
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:681-700
    fn ComputeFloatOffset(&self) -> LayoutUnit {
        let constraint_space = unsafe { &*self.constraint_space_ };
        if constraint_space.AvailableSize().inline_size == kIndefiniteSize {
            return LayoutUnit::default();
        }
        let left = self.line_opportunity_.line_left_offset;
        let bfc_left = constraint_space.GetBfcOffset().line_offset;
        let right = self.line_opportunity_.line_right_offset;
        let bfc_right = constraint_space.GetBfcOffset().line_offset
            + constraint_space.AvailableSize().inline_size;
        if IsLtr(self.base_direction_) {
            if left <= bfc_left {
                return LayoutUnit::default();
            }
            left - bfc_left
        } else {
            if right >= bfc_right {
                return LayoutUnit::default();
            }
            bfc_right - right
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:699-731
    fn RecalcClonedBoxDecorations(&mut self) {
        self.cloned_box_decorations_count_ = 0;
        self.cloned_box_decorations_initial_size_ = LayoutUnit::default();
        self.cloned_box_decorations_end_size_ = LayoutUnit::default();
        self.has_cloned_box_decorations_ = false;

        let mut open_items = OpenTagItems::default();
        self.ItemsData()
            .GetOpenTagItems(0, self.current_.item_index, &mut open_items);

        for member in &open_items {
            let item = unsafe { &*member.Get() };
            if unsafe { &*item.Style() }.BoxDecorationBreak() == EBoxDecorationBreak::kClone {
                self.has_cloned_box_decorations_ = true;
                self.disable_score_line_break_ = true;
                self.cloned_box_decorations_count_ += 1;
                let mut item_result = InlineItemResult::default();
                Self::ComputeOpenTagResult(
                    item,
                    unsafe { &*self.constraint_space_ },
                    self.is_svg_text_,
                    &mut item_result,
                );
                self.cloned_box_decorations_initial_size_ += item_result.inline_size;
                self.cloned_box_decorations_end_size_ += item_result.margins.inline_end
                    + item_result.borders.inline_end
                    + item_result.padding.inline_end;
            }
        }
        self.position_ += self.cloned_box_decorations_initial_size_;
        self.UpdateAvailableWidth();
        debug_assert!(self.base_available_width_ >= self.cloned_box_decorations_initial_size_);
    }

    // cpp: layoutng_inline/line_breaker.cc:641-679
    fn ComputeBaseDirection(&mut self) {
        let style = unsafe { &*self.node_.GetLayoutBox() }.StyleRef();
        if style.GetUnicodeBidi() != UnicodeBidi::kPlaintext {
            return;
        }

        let text = self.Text();
        if unsafe { StringIs8BitForInline(text) } {
            return;
        }
        let units = text.Span16().unwrap_or_default();

        let start_offset = if self.previous_line_had_forced_break_ {
            self.current_.text_offset
        } else {
            if self.current_.text_offset == 0 {
                return;
            }
            let prior = &units[..self.current_.text_offset as usize];
            match prior.iter().rposition(|&unit| Character::IsLineFeed(unit)) {
                Some(index) => index as u32 + 1,
                None => return,
            }
        };

        let view = StringView::from(text).Substring(start_offset, text.length() - start_offset);
        self.base_direction_ = BidiParagraph::BaseDirectionForStringOrLtrWithStopAt(
            &view,
            Some(Character::IsLineFeed),
        );
    }

    // Rust has no overloads: these two methods retain the C++ operand types.
    // cpp: layoutng_inline/line_breaker.cc:4657-4660
    fn IsPreviousItemOfType(&self, item_type: InlineItemType) -> bool {
        self.current_.item_index > 0
            && unsafe { &*self.Items().at(self.current_.item_index - 1).Get() }.Type() == item_type
    }

    // cpp: layoutng_inline/line_breaker.cc:4662-4675
    fn IsNextNonBidiControlItemOpenTag(&self) -> bool {
        let items = self.Items();
        for index in self.current_.item_index..items.size() {
            let item_type = unsafe { &*items[index as usize].Get() }.Type();
            if item_type == InlineItemType::kOpenTag {
                return true;
            }
            if item_type == InlineItemType::kBidiControl {
                continue;
            }
            return false;
        }
        false
    }

    // cpp: layoutng_inline/line_breaker.cc:4677-4688
    fn MoveToNextOfItem(&mut self, item: &InlineItem) {
        self.current_.text_offset = item.EndOffset();
        self.current_.item_index += 1;
        #[cfg(debug_assertions)]
        {
            let items = self.Items();
            if self.current_.item_index < items.size() {
                unsafe { &*items[self.current_.item_index as usize].Get() }
                    .AssertOffset(self.current_.text_offset);
            } else {
                debug_assert_eq!(self.current_.text_offset, self.Text().length());
            }
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:4690-4696
    fn MoveToNextOfResult(&mut self, item_result: &InlineItemResult) {
        self.current_ = item_result.End();
        debug_assert!(!item_result.item.Get().is_null());
        if self.current_.text_offset == unsafe { &*item_result.item.Get() }.EndOffset() {
            self.current_.item_index += 1;
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:425-430
    fn ShouldAutoWrap(&self, style: &ComputedStyle) -> bool {
        !self.disallow_auto_wrap_ && style.ShouldWrapLine()
    }

    // cpp: layoutng_inline/line_breaker.cc:4548-4568
    fn SetCurrentStyle(&mut self, style: &ComputedStyle) {
        if std::ptr::eq(style, self.current_style_) {
            #[cfg(debug_assertions)]
            {
                debug_assert_eq!(self.auto_wrap_, self.ShouldAutoWrap(style));
                if self.auto_wrap_ {
                    debug_assert_eq!(
                        self.break_iterator_.IsSoftHyphenEnabled(),
                        style.GetHyphens() != Hyphens::kNone
                            && (self.disable_phrase_
                                || style.WordBreak() != EWordBreak::kAutoPhrase)
                    );
                    debug_assert_eq!(
                        self.break_iterator_.Locale(),
                        style.GetFontDescription().Locale()
                    );
                }
                let mut spacing = ShapeResultSpacing::new(self.spacing_.Text(), self.is_svg_text_);
                spacing
                    .SetSpacingFromDescription(unsafe { &*style.GetFont() }.GetFontDescription());
                debug_assert_eq!(spacing.LetterSpacing(), self.spacing_.LetterSpacing());
                debug_assert_eq!(spacing.WordSpacing(), self.spacing_.WordSpacing());
            }
            return;
        }
        self.SetCurrentStyleForce(style);
    }

    // cpp: layoutng_inline/line_breaker.cc:4570-4655
    fn SetCurrentStyleForce(&mut self, style: &ComputedStyle) {
        self.current_style_ = style;

        let font_description = style.GetFontDescription();
        self.spacing_.SetSpacingFromDescription(font_description);

        self.auto_wrap_ = self.ShouldAutoWrap(style);
        if self.auto_wrap_ {
            debug_assert!(!self.is_text_combine_);
            self.break_iterator_.SetLocale(font_description.Locale());
            let mut hyphens = style.GetHyphens();
            let line_break = style.GetLineBreak();
            if line_break == LineBreak::kAnywhere {
                self.break_iterator_
                    .SetStrictness(LineBreakStrictness::kDefault);
                self.break_iterator_
                    .SetBreakType(LineBreakType::kBreakCharacter);
                self.break_anywhere_if_overflow_ = false;
            } else {
                self.break_iterator_
                    .SetStrictness(StrictnessFromLineBreak(line_break));
                let mut line_break_type = match style.WordBreak() {
                    EWordBreak::kNormal => {
                        self.break_anywhere_if_overflow_ = false;
                        LineBreakType::kNormal
                    }
                    EWordBreak::kBreakAll => {
                        self.break_anywhere_if_overflow_ = false;
                        LineBreakType::kBreakAll
                    }
                    EWordBreak::kBreakWord => {
                        self.break_anywhere_if_overflow_ =
                            self.line_clamp_ellipsis_width_ == LayoutUnit::default();
                        LineBreakType::kNormal
                    }
                    EWordBreak::kKeepAll => {
                        self.break_anywhere_if_overflow_ = false;
                        LineBreakType::kKeepAll
                    }
                    EWordBreak::kAutoPhrase => {
                        self.break_anywhere_if_overflow_ = false;
                        if self.disable_phrase_ {
                            LineBreakType::kNormal
                        } else {
                            hyphens = Hyphens::kNone;
                            LineBreakType::kPhrase
                        }
                    }
                };
                if !self.break_anywhere_if_overflow_
                    && self.line_clamp_ellipsis_width_ == LayoutUnit::default()
                {
                    let overflow_wrap = style.OverflowWrap();
                    self.break_anywhere_if_overflow_ = overflow_wrap == EOverflowWrap::kAnywhere
                        || (overflow_wrap == EOverflowWrap::kBreakWord
                            && self.mode_ == LineBreakerMode::kContent);
                }
                if self.break_anywhere_if_overflow_ {
                    if self.override_break_anywhere_ {
                        line_break_type = LineBreakType::kBreakCharacter;
                    } else if self.mode_ == LineBreakerMode::kMinContent {
                        self.override_break_anywhere_ = true;
                        line_break_type = LineBreakType::kBreakCharacter;
                    }
                }
                self.break_iterator_.SetBreakType(line_break_type);
            }

            if hyphens == Hyphens::kNone {
                self.break_iterator_.EnableSoftHyphen(false);
                self.hyphenation_ = std::ptr::null();
            } else {
                self.break_iterator_.EnableSoftHyphen(true);
                self.hyphenation_ = style.GetHyphenationWithLimits();
            }

            if style.ShouldBreakSpaces() {
                self.break_iterator_
                    .SetBreakSpace(BreakSpaceType::kAfterEverySpace);
                self.disable_score_line_break_ = true;
            } else {
                self.break_iterator_
                    .SetBreakSpace(BreakSpaceType::kAfterSpaceRun);
            }
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:3866-3880
    fn UpdateLineOpportunity(&mut self) {
        let bfc_block_offset = self.line_opportunity_.bfc_block_offset;
        let constraint_space = unsafe { &*self.constraint_space_ };
        let opportunity = unsafe { &*self.exclusion_space_ }.FindLayoutOpportunityDefault(
            &BfcOffset::new(
                constraint_space.GetBfcOffset().line_offset,
                bfc_block_offset,
            ),
            constraint_space.AvailableSize().inline_size,
            constraint_space.Direction(),
        );
        debug_assert_eq!(bfc_block_offset, opportunity.rect.BlockStartOffset());
        self.line_opportunity_ = opportunity.ComputeLineLayoutOpportunity(
            constraint_space,
            self.line_opportunity_.line_block_size,
            LayoutUnit::default(),
        );
        self.UpdateAvailableWidth();
        debug_assert!(self.AvailableWidth() >= LayoutUnit::default());
    }

    // cpp: layoutng_inline/line_breaker.cc:3884-3913
    fn RewindFloats(&mut self, new_end: u32, line_info: &mut LineInfo) {
        let rewound_float = line_info.Results()[new_end as usize..]
            .iter()
            .find(|result| result.positioned_float.HasValue())
            .map(|result| {
                (
                    result.item_index,
                    unsafe { &*result.positioned_float.Get() }
                        .layout_result
                        .Get(),
                    result.exclusion_space_before_position_float.clone(),
                )
            });
        let Some((item_index, layout_result, exclusion_space_before_float)) = rewound_float else {
            return;
        };
        line_info.RemoveParallelFlowBreakToken(item_index);
        let leading_floats = unsafe { &*self.leading_floats_ };
        if item_index < leading_floats.HandledIndex() {
            for index in 0..leading_floats.Count() {
                if leading_floats
                    .At(index)
                    .positioned_float
                    .layout_result
                    .Get()
                    == layout_result
                {
                    self.leading_floats_index_ = index;
                    break;
                }
            }
        }
        *unsafe { &mut *self.exclusion_space_ } = exclusion_space_before_float;
        self.UpdateLineOpportunity();
    }

    // cpp: layoutng_inline/line_breaker.cc:4092-4318
    fn HandleOverflow(&mut self, line_info: &mut LineInfo) {
        let available_width = self.AvailableWidthToFit();
        debug_assert!(self.position_ > available_width);
        let item_results = line_info.MutableResults() as *mut InlineItemResults;
        let hyphen_index_before = self.hyphen_index_;
        if self.HasHyphen() {
            let hyphen_width = self.RemoveHyphen(unsafe { &mut *item_results });
            self.position_ -= hyphen_width;
        }
        let mut width_to_rewind = self.position_ - available_width;
        let mut break_before = 0;
        let mut has_break_anywhere_if_overflow = self.break_anywhere_if_overflow_;

        let mut index = unsafe { &*item_results }.size();
        while index != 0 {
            index -= 1;
            let item_result =
                unsafe { &mut (&mut *item_results)[index as usize] } as *mut InlineItemResult;
            has_break_anywhere_if_overflow |= unsafe { &*item_result }.break_anywhere_if_overflow;
            if index < unsafe { &*item_results }.size() - 1
                && unsafe { &*item_result }.can_break_after
            {
                if width_to_rewind <= LayoutUnit::default() {
                    self.position_ = available_width + width_to_rewind;
                    self.RewindOverflow(index + 1, line_info);
                    return;
                }
                break_before = index + 1;
            }
            width_to_rewind -= unsafe { &*item_result }.inline_size;
            if width_to_rewind > LayoutUnit::default() {
                continue;
            }

            debug_assert!(!unsafe { &*item_result }.item.Get().is_null());
            let item = unsafe { &*(*item_result).item.Get() };
            if item.Type() == InlineItemType::kText {
                if unsafe { &*item_result }.Length() == 0 {
                    continue;
                }
                let result = unsafe { &*item_result };
                debug_assert!(
                    !result.shape_result.Get().is_null()
                        || (result.break_anywhere_if_overflow && !self.override_break_anywhere_)
                        || (self.mode_ == LineBreakerMode::kMinContent && !result.may_break_inside)
                );
                if width_to_rewind < LayoutUnit::default() && result.may_break_inside {
                    let item_available_width = -width_to_rewind;
                    let min_available_width = result.inline_size - LayoutUnit::from_signed(1);
                    if min_available_width <= LayoutUnit::default() {
                        if self.BreakTextAtPreviousBreakOpportunity(
                            unsafe { &mut *item_results },
                            index,
                        ) {
                            self.RewindOverflow(index + 1, line_info);
                            return;
                        }
                        continue;
                    }
                    let was_current_style = self.current_style_;
                    self.SetCurrentStyle(unsafe { &*item.Style() });
                    let item_result_before = unsafe { &*item_result }.clone();
                    self.BreakText(
                        item_result,
                        item,
                        unsafe { &*item.TextShapeResult() },
                        item_available_width.min(min_available_width),
                        item_available_width,
                        line_info,
                    );
                    #[cfg(debug_assertions)]
                    unsafe { &*item_result }.CheckConsistency(true);

                    let result = unsafe { &*item_result };
                    if result.can_break_after
                        && result.inline_size <= item_available_width
                        && result.EndOffset() < item_result_before.EndOffset()
                    {
                        debug_assert!(result.EndOffset() < item.EndOffset());
                        let new_end = index + 1;
                        assert!(new_end <= unsafe { &*item_results }.size());
                        if new_end == unsafe { &*item_results }.size() {
                            self.position_ = available_width + width_to_rewind + result.inline_size;
                            debug_assert_eq!(self.position_, line_info.ComputeWidth());
                            self.current_ = result.End();
                            self.ItemsData().AssertOffsetAt(&self.current_);
                            self.HandleTrailingSpaces(item, line_info);
                            return;
                        }
                        self.state_ = Some(LineBreakState::kTrailing);
                        self.Rewind(new_end, line_info);
                        return;
                    }

                    if self.HasHyphen() {
                        self.RemoveHyphen(unsafe { &mut *item_results });
                    }
                    unsafe { *item_result = item_result_before };
                    self.SetCurrentStyle(unsafe { &*was_current_style });
                }
            } else if unsafe { &*item_result }.IsRubyColumn()
                && width_to_rewind < LayoutUnit::default()
                && unsafe { &*item_result }.may_break_inside
            {
                let ruby_column = unsafe { &*(*item_result).ruby_column.Get() };
                let base_line = &ruby_column.base_line as *const LineInfo;
                let base_width = unsafe { &*base_line }.Width();
                self.Rewind(index, line_info);
                self.HandleRuby(line_info, base_width);
                let new_ruby_column =
                    unsafe { &*line_info.Results().last().unwrap().ruby_column.Get() };
                let new_base_line = &new_ruby_column.base_line;
                let new_width = new_base_line.Width();
                if new_width > LayoutUnit::default() && new_width != unsafe { &*base_line }.Width()
                {
                    self.state_ = Some(LineBreakState::kDone);
                    return;
                } else if index == 0 && !new_base_line.GetBreakToken().is_null() {
                    self.state_ = Some(LineBreakState::kDone);
                    return;
                }
            }
        }

        if self.applied_text_indent_ != LayoutUnit::default()
            && width_to_rewind > LayoutUnit::default()
            && self.is_first_formatted_line_
            && !unsafe { &*self.leading_floats_ }.Empty()
        {
            self.position_ -= self.applied_text_indent_;
            width_to_rewind -= self.applied_text_indent_;
            self.applied_text_indent_ = LayoutUnit::default();
            if width_to_rewind <= LayoutUnit::default() {
                self.state_ = Some(LineBreakState::kDone);
                return;
            }
        }

        if self.break_iterator_.BreakType() == LineBreakType::kPhrase
            && !self.disable_phrase_
            && self.mode_ == LineBreakerMode::kContent
        {
            self.disable_phrase_ = true;
            self.RetryAfterOverflow(line_info, item_results);
            return;
        }
        if !self.override_break_anywhere_ && has_break_anywhere_if_overflow {
            self.override_break_anywhere_ = true;
            self.RetryAfterOverflow(line_info, item_results);
            return;
        }

        line_info.SetHasOverflowDefault();
        self.disable_score_line_break_ = true;
        self.disable_bisect_line_break_ = true;
        debug_assert!(!self.HasHyphen());
        if let Some(hyphen_index_before) = hyphen_index_before {
            if hyphen_index_before < unsafe { &*item_results }.size() {
                let hyphen_width =
                    self.AddHyphenAt(unsafe { &mut *item_results }, hyphen_index_before);
                self.position_ += hyphen_width;
            }
        }
        if break_before != 0 {
            self.RewindOverflow(break_before, line_info);
            return;
        }
        if CanBreakAfterLast(unsafe { &*item_results }) {
            self.state_ = Some(LineBreakState::kTrailing);
            return;
        }
        debug_assert!(unsafe { &*item_results }
            .iter()
            .all(|result| !result.can_break_after));
        self.state_ = Some(LineBreakState::kOverflow);
    }

    // cpp: layoutng_inline/line_breaker.cc:3055-3177
    fn HandleAtomicInline(&mut self, item: &InlineItem, line_info: &mut LineInfo) {
        debug_assert!(
            item.Type() == InlineItemType::kAtomicInline
                || item.Type() == InlineItemType::kInitialLetterBox
        );
        let style = unsafe { &*item.Style() };
        let remaining_width = self.RemainingAvailableWidth();
        let mut ignore_overflow_if_negative_margin = false;
        if self.state_ == Some(LineBreakState::kContinue)
            && remaining_width < LayoutUnit::default()
            && (self.parent_breaker_.is_null() || self.auto_wrap_)
        {
            let item_index = self.current_.item_index;
            debug_assert_eq!(item_index, item.Index());
            self.HandleOverflow(line_info);
            if !line_info.HasOverflow() || item_index != self.current_.item_index {
                return;
            }
            debug_assert_ne!(self.state_, Some(LineBreakState::kContinue));
            ignore_overflow_if_negative_margin = true;
        }

        let item_result = self.AddItem(item, line_info);
        unsafe { &mut *item_result }.margins =
            ComputeLineMarginsForVisualContainer(unsafe { &*self.constraint_space_ }, style);
        let mut inline_margins = unsafe { &*item_result }.margins.InlineSum();
        if ignore_overflow_if_negative_margin {
            debug_assert!(remaining_width < LayoutUnit::default());
            if inline_margins >= remaining_width {
                RemoveLastItem(line_info);
                return;
            }
            self.state_ = Some(LineBreakState::kContinue);
            line_info.SetHasOverflow(false);
        }

        if self.HasHyphen() {
            let hyphen_width = self.RemoveHyphen(line_info.MutableResults());
            self.position_ -= hyphen_width;
        }

        let mut root_breaker = self as *const LineBreaker;
        while !unsafe { &*root_breaker }.parent_breaker_.is_null() {
            root_breaker = unsafe { &*root_breaker }.parent_breaker_;
        }
        let mode = unsafe { &*root_breaker }.mode_;
        let is_initial_letter_box = item.Type() == InlineItemType::kInitialLetterBox;
        if mode == LineBreakerMode::kContent || is_initial_letter_box {
            let baseline_algorithm_type = if style.BaselineSource() == EBaselineSource::kAuto {
                BaselineAlgorithmType::kInlineBlock
            } else {
                BaselineAlgorithmType::kDefault
            };
            let child = BlockNode::new(To::<layoutng::internal::layout_box::LayoutBox>(
                item.GetLayoutObject(),
            ));
            let constraint_space = unsafe { &*self.constraint_space_ };
            let layout_result = child.LayoutAtomicInline(
                constraint_space,
                self.node_.Style(),
                false,
                baseline_algorithm_type,
            );
            unsafe { &mut *item_result }.layout_result = Member::from_ptr(layout_result.cast_mut());
            assert!(!unsafe { &*self.node_.GetLayoutBox() }.NeedsCollectInlines());

            let physical_box_fragment = unsafe {
                &*To::<PhysicalBoxFragment>((&*layout_result).GetPhysicalFragment() as *const _)
            };
            unsafe { &mut *item_result }.inline_size = LogicalFragment::new(
                constraint_space.GetWritingDirection(),
                physical_box_fragment,
            )
            .InlineSize();
            if is_initial_letter_box && ShouldApplyInlineKerning(physical_box_fragment) {
                let side_bearing = ComputeNegativeSideBearings(physical_box_fragment);
                if IsLtr(self.base_direction_) {
                    unsafe { &mut *item_result }.margins.inline_start += side_bearing.inline_start;
                    inline_margins += side_bearing.inline_start;
                } else {
                    unsafe { &mut *item_result }.margins.inline_end += side_bearing.inline_end;
                    inline_margins += side_bearing.inline_end;
                }
            }
            unsafe { &mut *item_result }.inline_size += inline_margins;
        } else {
            debug_assert!(
                mode == LineBreakerMode::kMaxContent || mode == LineBreakerMode::kMinContent
            );
            self.ComputeMinMaxContentSizeForBlockChild(item, item_result, root_breaker);
        }

        unsafe { &mut *item_result }.should_create_line_box = true;
        unsafe { &mut *item_result }.can_break_after = self.CanBreakAfterAtomicInline(item);
        self.position_ += unsafe { &*item_result }.inline_size;
        self.trailing_whitespace_ = WhitespaceState::kNone;
        self.MoveToNextOfItem(item);
    }

    // cpp: layoutng_inline/line_breaker.cc:3179-3230
    fn ComputeMinMaxContentSizeForBlockChild(
        &mut self,
        item: &InlineItem,
        item_result: *mut InlineItemResult,
        root_breaker: *const LineBreaker,
    ) {
        let root_breaker_ref = unsafe { &*root_breaker };
        let mode = root_breaker_ref.mode_;
        let mut size_cache = root_breaker_ref.max_size_cache_;
        debug_assert!(matches!(
            mode,
            LineBreakerMode::kMaxContent | LineBreakerMode::kMinContent
        ));
        if mode == LineBreakerMode::kMaxContent && !size_cache.is_null() {
            unsafe { &mut *item_result }.inline_size =
                unsafe { &*size_cache }[item.Index() as usize];
            return;
        }

        debug_assert!(mode == LineBreakerMode::kMinContent || size_cache.is_null());
        let child = BlockNode::new(To::<layoutng::internal::layout_box::LayoutBox>(
            item.GetLayoutObject(),
        ));
        let constraint_space = unsafe { &*self.constraint_space_ };
        let mut builder =
            MinMaxConstraintSpaceBuilder::new(constraint_space, self.node_.Style(), &child, true);
        builder.SetAvailableBlockSize(constraint_space.AvailableSize().block_size);
        builder.SetPercentageResolutionBlockSize(if child.IsReplaced() {
            constraint_space.ReplacedChildPercentageResolutionBlockSize()
        } else {
            constraint_space.PercentageResolutionBlockSize()
        });
        let space = builder.ToConstraintSpace();
        let result = ComputeMinAndMaxContentContribution(
            self.node_.Style(),
            &child,
            &space,
            MinMaxSizesFloatInput::default(),
        );
        assert!(!unsafe { &*self.node_.GetLayoutBox() }.NeedsCollectInlines());
        let inline_margins = unsafe { &*item_result }.margins.InlineSum();
        if mode == LineBreakerMode::kMinContent {
            unsafe { &mut *item_result }.inline_size = result.sizes.min_size + inline_margins;
            if !root_breaker_ref.depends_on_block_constraints_out_.is_null() {
                *unsafe { &mut *root_breaker_ref.depends_on_block_constraints_out_ } |=
                    result.depends_on_block_constraints;
            }
            size_cache = root_breaker_ref.max_size_cache_;
            if !size_cache.is_null() {
                let cache = unsafe { &mut *size_cache };
                if cache.is_empty() {
                    cache.resize(self.Items().size() as usize, LayoutUnit::default());
                }
                cache[item.Index() as usize] = result.sizes.max_size + inline_margins;
            }
            return;
        }
        debug_assert!(mode == LineBreakerMode::kMaxContent && size_cache.is_null());
        unsafe { &mut *item_result }.inline_size = result.sizes.max_size + inline_margins;
    }

    // cpp: layoutng_inline/line_breaker.cc:3232-3325
    fn HandleBlockInInline(
        &mut self,
        item: &InlineItem,
        block_break_token: *const BlockBreakToken,
        line_info: &mut LineInfo,
    ) {
        debug_assert_eq!(item.Type(), InlineItemType::kBlockInInline);
        debug_assert!(
            block_break_token.is_null()
                || unsafe { &*block_break_token }.InputNode().GetLayoutBox()
                    == To::<layoutng::internal::layout_box::LayoutBox>(item.GetLayoutObject())
        );

        if !line_info.Results().empty() {
            self.force_non_empty_if_last_line_ = false;
            self.HandleForcedLineBreak(None, line_info);
            return;
        }

        let item_result = self.AddItem(item, line_info);
        let mut move_past_block = true;
        if self.mode_ == LineBreakerMode::kContent {
            let constraint_space = unsafe { &*self.constraint_space_ };
            let exclusion_space = unsafe { &*self.exclusion_space_ };
            debug_assert!(*exclusion_space == *constraint_space.GetExclusionSpace());
            constraint_space
                .GetExclusionSpace()
                .MoveAndUpdateDerivedGeometry(exclusion_space);

            let block_node = BlockNode::new(To::<layoutng::internal::layout_box::LayoutBox>(
                item.GetLayoutObject(),
            ));
            let mut modified_space = None;
            let child_space = constraint_space.CloneForBlockInInlineIfNeeded(&mut modified_space);
            let spanner_path_for_child =
                FollowColumnSpannerPath(self.column_spanner_path_, &block_node);
            let layout_result = block_node.Layout(
                child_space,
                block_break_token,
                std::ptr::null(),
                spanner_path_for_child,
            );
            assert!(!unsafe { &*self.node_.GetLayoutBox() }.NeedsCollectInlines());
            line_info.SetBlockInInlineLayoutResult(layout_result);

            let layout_result_ref = unsafe { &*layout_result };
            if layout_result_ref.Status() != EStatus::kSuccess {
                self.state_ = Some(LineBreakState::kDone);
                return;
            }

            let fragment = layout_result_ref.GetPhysicalFragment();
            unsafe { &mut *item_result }.inline_size =
                LogicalFragment::new(constraint_space.GetWritingDirection(), fragment).InlineSize();
            unsafe { &mut *item_result }.should_create_line_box =
                !layout_result_ref.IsSelfCollapsing();
            unsafe { &mut *item_result }.layout_result = Member::from_ptr(layout_result.cast_mut());

            let outgoing_block_break_token = To::<BlockBreakToken>(fragment.GetBreakToken());
            if !outgoing_block_break_token.is_null() {
                let outgoing = unsafe { &*outgoing_block_break_token };
                if outgoing.IsAtBlockEnd()
                    || (!self.break_token_.is_null()
                        && unsafe { &*self.break_token_ }.IsInParallelFlow())
                {
                    let parallel_token = InlineBreakToken::CreateForParallelBlockFlow(
                        self.node_.clone(),
                        &self.current_,
                        outgoing,
                    );
                    line_info.PropagateParallelFlowBreakToken(parallel_token);
                } else {
                    self.resume_block_in_inline_in_same_flow_ = true;
                    move_past_block = false;
                }
            }
        } else {
            debug_assert!(
                self.mode_ == LineBreakerMode::kMaxContent
                    || self.mode_ == LineBreakerMode::kMinContent
            );
            let root_breaker = self as *const LineBreaker;
            self.ComputeMinMaxContentSizeForBlockChild(item, item_result, root_breaker);
        }

        self.position_ += unsafe { &*item_result }.inline_size;
        line_info.SetIsBlockInInline();
        line_info.SetHasForcedBreak();
        self.is_forced_break_ = true;
        self.trailing_whitespace_ = WhitespaceState::kNone;
        if move_past_block {
            self.MoveToNextOfItem(item);
        }
        self.state_ = Some(LineBreakState::kDone);
    }

    // cpp: layoutng_inline/line_breaker.cc:4708-4786
    fn CreateBreakToken(&mut self, line_info: &LineInfo) -> *const InlineBreakToken {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.has_considered_creating_break_token_);
            self.has_considered_creating_break_token_ = true;
        }

        debug_assert!(!self.current_style_.is_null());
        let items = self.Items();
        debug_assert!(self.current_.item_index <= items.size());
        if self.IsAtEnd() {
            return std::ptr::null();
        }

        let mut sub_break_token: *const BlockBreakToken = std::ptr::null();
        if self.resume_block_in_inline_in_same_flow_ {
            let block_in_inline = line_info.BlockInInlineLayoutResult();
            debug_assert!(!block_in_inline.is_null());
            let block_in_inline = unsafe { &*block_in_inline };
            if block_in_inline.Status() != EStatus::kSuccess {
                return std::ptr::null();
            }
            let box_fragment = unsafe {
                &*To::<PhysicalBoxFragment>(block_in_inline.GetPhysicalFragment() as *const _)
            };
            sub_break_token = To::<BlockBreakToken>(box_fragment.GetBreakToken());
        }

        let is_past_first_formatted_line =
            !self.is_first_formatted_line_ || !line_info.IsEmptyLine();
        let is_line_clamp_displaced_line =
            self.line_clamp_ellipsis_width_ != LayoutUnit::default() && line_info.Results().empty();
        debug_assert_eq!(line_info.HasForcedBreak(), self.is_forced_break_);
        let flags = (if self.is_forced_break_ {
            InlineBreakTokenFlag::kIsForcedBreak as u32
        } else {
            0
        }) | (if line_info.UseFirstLineStyle() {
            InlineBreakTokenFlag::kUseFirstLineStyle as u32
        } else {
            0
        }) | (if self.cloned_box_decorations_count_ != 0 {
            InlineBreakTokenFlag::kHasClonedBoxDecorations as u32
        } else {
            0
        }) | (if is_past_first_formatted_line {
            InlineBreakTokenFlag::kIsPastFirstFormattedLine as u32
        } else {
            0
        }) | (if is_line_clamp_displaced_line {
            InlineBreakTokenFlag::kIsLineClampDisplacedLine as u32
        } else {
            0
        });

        let mut next_start = self.current_.clone();
        if line_info.UseFirstLineStyle() {
            if let Some(offset_map) = self.node_.FirstLineOffsetMap() {
                debug_assert!(RuntimeEnabledFeatures::FirstLineTextTransformEnabled());
                next_start.text_offset = offset_map.InverseMapOffset(next_start.text_offset);
                let base_items = &self.node_.ItemsData(false).items;
                while next_start.item_index < base_items.size() {
                    let item = unsafe { &*base_items[next_start.item_index as usize].Get() };
                    if (item.Length() > 0 && next_start.text_offset >= item.EndOffset())
                        || (item.Length() == 0 && next_start.text_offset > item.EndOffset())
                    {
                        next_start.item_index += 1;
                    } else {
                        break;
                    }
                }
                if next_start.item_index >= base_items.size() {
                    return std::ptr::null();
                }
            }
        }

        InlineBreakToken::Create(
            self.node_.clone(),
            self.current_style_,
            &next_start,
            flags,
            sub_break_token,
            self.ruby_break_token_,
        )
    }

    // C++ overloads have distinct Rust names because Rust does not overload
    // methods by parameter type.
    // cpp: layoutng_inline/line_breaker.cc:2333-2360
    pub fn CanBreakInsideLine(&mut self, line_info: &LineInfo) -> bool {
        let item_results = line_info.Results();
        if RuntimeEnabledFeatures::SkipOofItemForBreakCandidateEnabled() {
            let mut index = 0;
            while index + 1 < item_results.size() {
                if item_results[index as usize].can_break_after {
                    index += 1;
                    while index < item_results.size() {
                        let item = unsafe { &*item_results[index as usize].item.Get() };
                        if !item.IsFloatingOrOutOfFlowPositioned() {
                            return true;
                        }
                        index += 1;
                    }
                }
                index += 1;
            }
        } else if item_results
            .iter()
            .take(item_results.size().saturating_sub(1) as usize)
            .any(|result| result.can_break_after)
        {
            return true;
        }
        for item_result in item_results.iter() {
            debug_assert!(!item_result.item.Get().is_null());
            let item = unsafe { &*item_result.item.Get() };
            if item.Type() == InlineItemType::kText
                && item_result.may_break_inside
                && self.CanBreakInsideResult(item_result)
            {
                return true;
            }
        }
        false
    }

    // cpp: layoutng_inline/line_breaker.cc:2362-2379
    fn CanBreakInsideResult(&mut self, item_result: &InlineItemResult) -> bool {
        debug_assert!(item_result.may_break_inside);
        debug_assert!(!item_result.item.Get().is_null());
        let item = unsafe { &*item_result.item.Get() };
        debug_assert_eq!(item.Type(), InlineItemType::kText);
        debug_assert!(!item.Style().is_null());
        self.SetCurrentStyle(unsafe { &*item.Style() });
        if !self.auto_wrap_ {
            return false;
        }
        let offset = item_result.TextOffset();
        if offset.start < self.break_iterator_.StartOffset() {
            self.break_iterator_.SetStartOffset(offset.start);
        }
        self.break_iterator_.NextBreakOpportunity(offset.start + 1) < offset.end
    }

    // cpp: layoutng_inline/line_breaker.cc:3693-3732
    fn ShouldPushFloatAfterLine(
        &mut self,
        unpositioned_float: &mut UnpositionedFloat<'_>,
        line_info: &mut LineInfo,
    ) -> bool {
        if !unpositioned_float.token.is_null() {
            return false;
        }

        let support = unsafe { &*LayoutPassScope::RequireFloatSupport() };
        let inline_margin_size = support
            .margin_box_inline_size
            .expect("float layout support must be installed")(
            unpositioned_float as *mut UnpositionedFloat,
        );
        let constraint_space = unsafe { &*self.constraint_space_ };
        let used_size = self.position_
            + inline_margin_size
            + ComputeFloatAncestorInlineEndSize(
                constraint_space,
                self.Items(),
                self.current_.item_index,
            );
        let float_width = self
            .line_opportunity_
            .AvailableFloatInlineSize()
            .AddEpsilon();
        let mut can_fit_float = used_size <= float_width;
        if !can_fit_float {
            can_fit_float =
                used_size - self.TrailingCollapsibleSpaceWidth(line_info) <= float_width;
        }
        let bfc_block_offset = unpositioned_float.origin_bfc_offset.block_offset;
        let exclusion_space = unsafe { &*self.exclusion_space_ };
        !can_fit_float
            || exclusion_space.LastFloatBlockStart() > bfc_block_offset
            || exclusion_space
                .ClearanceOffset(unpositioned_float.ClearType(constraint_space.Direction()))
                > bfc_block_offset
    }

    // cpp: layoutng_inline/line_breaker.cc:3748-3864
    fn HandleFloat(
        &mut self,
        item: &InlineItem,
        float_break_token: *const BlockBreakToken,
        line_info: &mut LineInfo,
    ) {
        let item_result = self.AddItem(item, line_info);
        let index_before_float = self.current_.clone();
        debug_assert!(!unsafe { &*item_result }.can_break_after);
        if !unsafe { &*item_result }.should_create_line_box {
            unsafe { &mut *item_result }.can_break_after = self.auto_wrap_;
        }
        self.MoveToNextOfItem(item);
        if self.mode_ != LineBreakerMode::kContent {
            return;
        }

        let leading_floats = unsafe { &*self.leading_floats_ };
        if self.current_.item_index <= leading_floats.HandledIndex() && !leading_floats.Empty() {
            debug_assert!(self.leading_floats_index_ < leading_floats.Count());
            let leading_float = leading_floats.At(self.leading_floats_index_);
            self.leading_floats_index_ += 1;
            unsafe { &mut *item_result }
                .positioned_float
                .Assign(leading_float.positioned_float.clone());
            let parallel_token = leading_float.parallel_flow_break_token.Get();
            if !parallel_token.is_null() {
                line_info.PropagateParallelFlowBreakToken(parallel_token);
            }
            debug_assert!(!self.exclusion_space_.is_null());
            unsafe { &mut *item_result }
                .exclusion_space_before_position_float
                .CopyFrom(unsafe { &*self.exclusion_space_ });
            return;
        }

        let bfc_block_offset = self.line_opportunity_.bfc_block_offset;
        let constraint_space = unsafe { &*self.constraint_space_ };
        let is_hidden_for_paint = constraint_space.GetLineClampData().ShouldHideForPaint();
        let float_node = BlockNode::new(To::<layoutng::internal::layout_box::LayoutBox>(
            item.GetLayoutObject(),
        ));
        let percentage_size = if float_node.IsReplaced() {
            constraint_space.ReplacedChildPercentageResolutionSize()
        } else {
            constraint_space.PercentageResolutionSize()
        };
        let origin = BfcOffset::new(
            constraint_space.GetBfcOffset().line_offset,
            bfc_block_offset,
        );
        let parent_style = self.node_.Style() as *const ComputedStyle;
        let mut unpositioned_float = UnpositionedFloat::new(
            float_node,
            float_break_token,
            constraint_space.AvailableSize(),
            percentage_size,
            &origin,
            constraint_space,
            unsafe { &*parent_style },
            constraint_space.FragmentainerBlockSize(),
            constraint_space.FragmentainerOffset(),
            is_hidden_for_paint,
        );
        let float_after_line = self.ShouldPushFloatAfterLine(&mut unpositioned_float, line_info);
        if HasUnpositionedFloats(line_info.Results()) || float_after_line {
            unsafe { &mut *item_result }.has_unpositioned_floats = true;
            return;
        }

        debug_assert!(!self.exclusion_space_.is_null());
        unsafe { &mut *item_result }
            .exclusion_space_before_position_float
            .CopyFrom(unsafe { &*self.exclusion_space_ });
        let support = unsafe { &*LayoutPassScope::RequireFloatSupport() };
        let positioned_float = support
            .position
            .expect("float layout support must be installed")(
            &mut unpositioned_float,
            self.exclusion_space_,
        );
        unsafe { &mut *item_result }
            .positioned_float
            .Assign(positioned_float);
        assert!(!unsafe { &*self.node_.GetLayoutBox() }.NeedsCollectInlines());

        if constraint_space.HasBlockFragmentation() {
            let positioned = unsafe { &*(*item_result).positioned_float.Get() };
            let break_token = positioned.BreakToken();
            if !break_token.is_null() {
                let parallel_token = InlineBreakToken::CreateForParallelBlockFlow(
                    self.node_.clone(),
                    &index_before_float,
                    unsafe { &*break_token },
                );
                line_info.PropagateParallelFlowBreakToken(parallel_token);
                if positioned.minimum_space_shortage != LayoutUnit::default() {
                    line_info.PropagateMinimumSpaceShortage(positioned.minimum_space_shortage);
                    debug_assert_eq!(
                        positioned.tallest_unbreakable_block_size,
                        LayoutUnit::default()
                    );
                } else if positioned.tallest_unbreakable_block_size != LayoutUnit::default() {
                    line_info.PropagateTallestUnbreakableBlockSize(
                        positioned.tallest_unbreakable_block_size,
                    );
                }
                if unsafe { &*break_token }.IsBreakBefore() {
                    return;
                }
            }
        }
        self.UpdateLineOpportunity();
    }

    // cpp: layoutng_inline/line_breaker.h:265-272
    // cpp: layoutng_inline/line_breaker.cc:3327-3498
    fn HandleRuby(&mut self, line_info: &mut LineInfo, retry_size: LayoutUnit) -> bool {
        let ruby_token = self.ruby_break_token_;
        self.ruby_break_token_ = std::ptr::null();
        let mut base_start = self.current_;
        let base_end_index;
        let mut annotation_data: Vec<AnnotationBreakTokenData> = Vec::new();
        let open_column_item_index;
        if ruby_token.is_null() {
            open_column_item_index = self.current_.item_index;
            let ruby_indexes = ParseRubyInInlineItems(self.Items(), open_column_item_index);
            base_end_index = ruby_indexes.base_end;
            let base_end_item = unsafe { &*self.Items()[base_end_index as usize].Get() };
            if base_end_item.Type() == InlineItemType::kCloseRubyColumn {
                return false;
            }
            debug_assert_eq!(base_end_item.Type(), InlineItemType::kOpenTag);
            debug_assert!(unsafe { &*base_end_item.GetLayoutObject() }.IsInlineRubyText());
            let open_item = unsafe { &*self.Items()[open_column_item_index as usize].Get() };
            base_start = InlineItemTextIndex {
                item_index: open_column_item_index + 1,
                text_offset: open_item.EndOffset(),
            };
            let start = ruby_indexes.annotation_start;
            annotation_data.push(AnnotationBreakTokenData {
                start: InlineItemTextIndex {
                    item_index: start,
                    text_offset: unsafe { &*self.Items()[start as usize].Get() }.StartOffset(),
                },
                start_item_index: start,
                end_item_index: ruby_indexes.column_end,
            });
        } else {
            let token = unsafe { &*ruby_token };
            open_column_item_index = token.open_column_item_index;
            base_end_index = token.ruby_base_end_item_index;
            annotation_data = token.annotation_data.clone();
        }
        let item_ptr = self.Items()[open_column_item_index as usize].Get();
        let item = unsafe { &*item_ptr };

        let mut base_line_info = self.CreateSubLineInfo(
            base_start,
            base_end_index,
            LineBreakerMode::kMaxContent,
            kIndefiniteSize,
            self.trailing_whitespace_,
            true,
        );
        base_line_info.OverrideLineStyle(unsafe { &*self.current_style_ });
        base_line_info.SetIsRubyBase();
        base_line_info.UpdateTextAlign();

        let number_of_annotations = annotation_data.len();
        let mut annotation_line_list: HeapVector<LineInfo, 1> = HeapVector::default();
        annotation_line_list.ReserveInitialCapacity(number_of_annotations as u32);
        for data in &annotation_data {
            let mut line = self.CreateSubLineInfo(
                data.start,
                data.end_item_index,
                LineBreakerMode::kMaxContent,
                kIndefiniteSize,
                WhitespaceState::kLeading,
                false,
            );
            let annotation_item = unsafe { &*self.Items()[data.start_item_index as usize].Get() };
            let annotation_object = unsafe { &*annotation_item.GetLayoutObject() };
            line.OverrideLineStyle(annotation_object.StyleRef());
            annotation_line_list.push(line);
        }

        let mut ruby_size = MaxLineWidth(&base_line_info, &annotation_line_list);
        let available = self.RemainingAvailableWidth().ClampNegativeToZero();
        let ruby_index = line_info.Results().size();
        let mut overhang = crate::ruby_utils::GetOverhangForLines(
            ruby_size,
            &base_line_info,
            &annotation_line_list,
            line_info,
            ruby_index,
        );
        if !crate::ruby_utils::CanApplyStartOverhang(
            line_info,
            ruby_index,
            unsafe { &*self.current_style_ },
            &mut overhang.start,
        ) {
            overhang.start = LayoutUnit::default();
        }
        let is_monolithic = self.IsMonolithicRuby(&base_line_info, &annotation_line_list);
        if (retry_size == kIndefiniteSize && ruby_size <= available + overhang.start)
            || is_monolithic
        {
            if self.mode_ == LineBreakerMode::kContent {
                base_line_info = self.CreateSubLineInfo(
                    base_start,
                    base_end_index,
                    LineBreakerMode::kContent,
                    kIndefiniteSize,
                    self.trailing_whitespace_,
                    true,
                );
                for (index, data) in annotation_data.iter().enumerate() {
                    annotation_line_list[index] = self.CreateSubLineInfo(
                        data.start,
                        data.end_item_index,
                        LineBreakerMode::kContent,
                        kIndefiniteSize,
                        WhitespaceState::kLeading,
                        false,
                    );
                }
            }
            let result = self.AddRubyColumnResult(
                item,
                &base_line_info,
                &annotation_line_list,
                &annotation_data,
                ruby_size,
                !ruby_token.is_null(),
                line_info,
            );
            let ruby_column =
                unsafe { &mut *((*result).ruby_column.Get() as *mut InlineItemResultRubyColumn) };
            ruby_column.start_ruby_break_token = Member::from_ptr(ruby_token.cast_mut());
            unsafe { &mut *result }.may_break_inside = !is_monolithic;
            self.position_ += ruby_size;
            self.current_ = annotation_line_list[0].End();
            return true;
        }

        let base_intrinsic_size = base_line_info.Width();
        let base_target = if retry_size == kIndefiniteSize {
            available * base_intrinsic_size / ruby_size
        } else {
            retry_size - LayoutUnit::from_signed(1)
        };
        base_line_info = self.CreateSubLineInfo(
            base_start,
            base_end_index,
            self.mode_,
            base_target,
            self.trailing_whitespace_,
            false,
        );
        assert!(base_line_info.Results().size() > 0);

        let mut annotation_is_broken = false;
        for (index, data) in annotation_data.iter().enumerate() {
            let line = &annotation_line_list[index];
            let mut limit = kIndefiniteSize;
            let mut mode = self.mode_;
            if !base_line_info.GetBreakToken().is_null() {
                limit = if retry_size != kIndefiniteSize {
                    line.Width() * base_line_info.Width() / base_intrinsic_size
                } else {
                    available * line.Width() / ruby_size
                };
            } else if mode == LineBreakerMode::kMinContent {
                mode = LineBreakerMode::kMaxContent;
            }
            annotation_line_list[index] = self.CreateSubLineInfo(
                data.start,
                data.end_item_index,
                mode,
                limit,
                WhitespaceState::kLeading,
                false,
            );
            annotation_is_broken |= !annotation_line_list[index].GetBreakToken().is_null();
        }

        ruby_size = MaxLineWidth(&base_line_info, &annotation_line_list);
        let result = self.AddRubyColumnResult(
            item,
            &base_line_info,
            &annotation_line_list,
            &annotation_data,
            ruby_size,
            !ruby_token.is_null(),
            line_info,
        );
        let ruby_column =
            unsafe { &mut *((*result).ruby_column.Get() as *mut InlineItemResultRubyColumn) };
        ruby_column.start_ruby_break_token = Member::from_ptr(ruby_token.cast_mut());
        unsafe { &mut *result }.may_break_inside = true;
        self.position_ += ruby_size;

        if base_line_info.GetBreakToken().is_null() && !annotation_is_broken {
            self.current_ = annotation_line_list[0].End();
            return true;
        }
        debug_assert!(!base_line_info.GetBreakToken().is_null());
        self.current_ = base_line_info.End();

        let mut breaks = Vec::with_capacity(number_of_annotations);
        for (index, data) in annotation_data.iter().enumerate() {
            breaks.push(AnnotationBreakTokenData {
                start: annotation_line_list[index].End(),
                start_item_index: data.start_item_index,
                end_item_index: data.end_item_index,
            });
        }
        ruby_column.end_ruby_break_token = Member::from_ptr(MakeGarbageCollected(
            RubyBreakTokenData::new(open_column_item_index, base_end_index, &breaks),
        ));
        if retry_size == kIndefiniteSize {
            self.HandleOverflowIfNeeded(line_info);
            if !line_info.Results().empty() {
                self.state_ = Some(LineBreakState::kDone);
            }
        }
        true
    }

    // cpp: layoutng_inline/line_breaker.cc:3500-3542
    fn IsMonolithicRuby(&self, base_line: &LineInfo, annotation_line_list: &[LineInfo]) -> bool {
        if self.end_item_index_ != self.Items().size() {
            return true;
        }
        if !self.auto_wrap_ {
            return true;
        }
        if base_line.Width() <= LayoutUnit::default() {
            return true;
        }
        if !self.node_.Style().ShouldWrapLineGreedy() {
            return true;
        }
        const BASE_LETTER_LIMIT: u32 = 4;
        const ANNOTATION_LETTER_LIMIT: u32 = 8;
        if !base_line.GlyphCountIsGreaterThan(BASE_LETTER_LIMIT)
            && !annotation_line_list
                .iter()
                .any(|line| line.GlyphCountIsGreaterThan(ANNOTATION_LETTER_LIMIT))
        {
            return true;
        }
        false
    }

    // cpp: layoutng_inline/line_breaker.cc:3544-3586
    fn CreateSubLineInfo(
        &mut self,
        start: InlineItemTextIndex,
        end_item_index: u32,
        mode: LineBreakerMode,
        mut limit: LayoutUnit,
        initial_whitespace_state: WhitespaceState,
        disable_trailing_whitespace_collapsing: bool,
    ) -> LineInfo {
        let mut disallow_auto_wrap = false;
        if limit == kIndefiniteSize {
            limit = LayoutUnit::Max();
            disallow_auto_wrap = true;
        }
        let mut empty_exclusion_space = ExclusionSpace::default();
        let empty_leading_floats = LeadingFloats::default();
        let mut sub_line_info = LineInfo::default();
        let line_opportunity = LineLayoutOpportunity::with_inline_size(limit);
        let mut sub_line_breaker = LineBreaker::new(
            self.node_.clone(),
            mode,
            unsafe { &*self.constraint_space_ },
            &line_opportunity,
            &empty_leading_floats,
            std::ptr::null(),
            std::ptr::null(),
            &mut empty_exclusion_space,
        );
        sub_line_breaker.disallow_auto_wrap_ = disallow_auto_wrap;
        sub_line_breaker.SetInputRange(
            start,
            end_item_index,
            initial_whitespace_state,
            self as *const LineBreaker,
        );
        if RuntimeEnabledFeatures::TabSizeInRubyBaseEnabled() {
            sub_line_breaker.tab_stop_offset_ = self.position_ + self.tab_stop_offset_;
        }
        sub_line_breaker.disable_trailing_whitespace_collapsing_ =
            disable_trailing_whitespace_collapsing;
        sub_line_breaker.OverrideAvailableWidth(limit);
        sub_line_breaker.NextLine(&mut sub_line_info);
        if disallow_auto_wrap {
            assert!(sub_line_breaker.IsAtEnd());
        }
        sub_line_info
    }

    // cpp: layoutng_inline/line_breaker.h:282-290
    // cpp: layoutng_inline/line_breaker.cc:3588-3658
    fn AddRubyColumnResult(
        &mut self,
        item: &InlineItem,
        base_line_info: &LineInfo,
        annotation_line_list: &HeapVector<LineInfo, 1>,
        annotation_data_list: &[AnnotationBreakTokenData],
        ruby_size: LayoutUnit,
        is_continuation: bool,
        line_info: &mut LineInfo,
    ) -> *mut InlineItemResult {
        assert_eq!(item.Type(), InlineItemType::kOpenRubyColumn);
        let column_result = self.AddEmptyItem(item, line_info);
        unsafe { &mut *column_result }.inline_size = ruby_size;
        let data_ptr = MakeGarbageCollected(InlineItemResultRubyColumn::default());
        unsafe { &mut *column_result }.ruby_column = Member::from_ptr(data_ptr.cast());
        let data = unsafe { &mut *data_ptr };
        data.base_line = base_line_info.clone();
        data.base_line
            .OverrideLineStyle(unsafe { &*self.current_style_ });
        data.base_line.SetIsRubyBase();
        data.base_line.UpdateTextAlign();
        if data.base_line.MayHaveRubyOverhang() {
            line_info.SetMayHaveRubyOverhang();
        }
        line_info.SetHaveTextCombineOrRubyItem();
        data.is_continuation = is_continuation;

        data.annotation_line_list = annotation_line_list.clone();
        for (index, annotation) in annotation_data_list.iter().enumerate() {
            let annotation_item =
                unsafe { &*self.Items()[annotation.start_item_index as usize].Get() };
            let annotation_object = unsafe { &*annotation_item.GetLayoutObject() };
            data.annotation_line_list[index].OverrideLineStyle(annotation_object.StyleRef());
            data.annotation_line_list[index].SetIsRubyText();
            data.annotation_line_list[index].UpdateTextAlign();
            let parent = unsafe { &*annotation_object.Parent() };
            data.position_list.push(if parent.IsInlineRuby() {
                let parent_style = if self.use_first_line_style_ {
                    parent.FirstLineStyleRef()
                } else {
                    parent.StyleRef()
                };
                parent_style.GetRubyPosition()
            } else {
                RubyPosition::kOver
            });
        }
        debug_assert_eq!(data.annotation_line_list.len(), data.position_list.len());

        unsafe { &mut *column_result }.text_offset.end = annotation_line_list[0].EndTextOffset();
        unsafe { &mut *column_result }.should_create_line_box = true;
        unsafe { &mut *column_result }.can_break_after = self.CanBreakAfterRubyColumn(
            unsafe { &*column_result },
            annotation_data_list[0].end_item_index,
        );

        if base_line_info.Width() < ruby_size {
            line_info.SetMayHaveRubyOverhang();
            let ruby_index = line_info.Results().size() - 1;
            let mut overhang = crate::ruby_utils::GetOverhangForColumn(
                unsafe { &*column_result },
                line_info,
                ruby_index,
            );
            if overhang.end > LayoutUnit::default() {
                unsafe { &mut *column_result }.pending_end_overhang = overhang.end;
                self.maybe_have_end_overhang_ = true;
            }
            let style = if item.GetLayoutObject().is_null() {
                unsafe { &*self.current_style_ }
            } else {
                unsafe { &*item.Style() }
            };
            if crate::ruby_utils::CanApplyStartOverhang(
                line_info,
                ruby_index,
                style,
                &mut overhang.start,
            ) {
                debug_assert_eq!(
                    unsafe { &*column_result }.margins.inline_start,
                    LayoutUnit::default()
                );
                let base_results = data.base_line.MutableResults();
                debug_assert_eq!(
                    unsafe { &*base_results[0].item.Get() }.Type(),
                    InlineItemType::kRubyLinePlaceholder
                );
                base_results[0].margins.inline_start = -overhang.start;
                self.position_ -= overhang.start;
            }
        }
        self.trailing_whitespace_ = WhitespaceState::kUnknown;
        column_result
    }

    // cpp: layoutng_inline/line_breaker.cc:3660-3687
    fn CanBreakAfterRubyColumn(
        &self,
        column_result: &InlineItemResult,
        column_end_item_index: u32,
    ) -> bool {
        debug_assert_eq!(
            unsafe { &*column_result.item.Get() }.Type(),
            InlineItemType::kOpenRubyColumn
        );
        debug_assert!(!column_result.ruby_column.Get().is_null());
        if !self.auto_wrap_ {
            return false;
        }
        let base_line = &unsafe { &*column_result.ruby_column.Get() }.base_line;
        if !base_line.GetBreakToken().is_null() {
            return true;
        }
        let mut text_content = StringBuilder::default();
        let base_text_length = base_line.EndTextOffset() - base_line.StartOffset();
        let text = self.Text();
        text_content.AppendString(
            &StringView::from(text)
                .Substring(base_line.StartOffset(), base_text_length)
                .ToString(),
        );
        let next_item = unsafe { &*self.Items()[column_end_item_index as usize].Get() };
        debug_assert_eq!(next_item.Type(), InlineItemType::kCloseRubyColumn);
        let ignorable_bidi_length = 1 + self.IgnorableBidiControlLength(next_item);
        let remaining_start = next_item.StartOffset() + ignorable_bidi_length;
        text_content.AppendString(
            &StringView::from(text)
                .Substring(remaining_start, text.length() - remaining_start)
                .ToString(),
        );
        let break_iterator = LazyLineBreakIterator::with_replacement_string(
            &self.break_iterator_,
            text_content.ReleaseString(),
        );
        break_iterator.IsBreakable(base_text_length)
    }

    // cpp: layoutng_inline/line_breaker.cc:4320-4343
    fn RetryAfterOverflow(
        &mut self,
        line_info: &mut LineInfo,
        item_results: *mut InlineItemResults,
    ) {
        self.disable_score_line_break_ = true;
        self.disable_bisect_line_break_ = true;
        self.state_ = Some(LineBreakState::kContinue);
        if !unsafe { &*item_results }.empty() {
            let style = self.ComputeCurrentStyle(0, line_info);
            self.SetCurrentStyleForce(unsafe { &*style });
            self.Rewind(0, line_info);
        } else {
            self.SetCurrentStyleForce(unsafe { &*self.current_style_ });
        }
        self.ResetRewindLoopDetector();
    }

    // cpp: layoutng_inline/line_breaker.cc:4347-4443
    fn RewindOverflow(&mut self, mut new_end: u32, line_info: &mut LineInfo) {
        let text = self.Text().clone();
        let units = text.Span16().unwrap_or_default();
        let rewind_action = {
            let item_results = line_info.Results();
            debug_assert!(new_end < item_results.size());
            let mut open_tag_count = 0;
            let mut action = None;
            for index in new_end..item_results.size() {
                let item_result = &item_results[index as usize];
                debug_assert!(!item_result.item.Get().is_null());
                let item = unsafe { &*item_result.item.Get() };
                if item.Type() == InlineItemType::kText {
                    if item_result.Length() == 0 {
                        continue;
                    }
                    if !item_result.shape_result.Get().is_null()
                        || (self.break_anywhere_if_overflow_ && !self.override_break_anywhere_)
                    {
                        debug_assert!(!item.Style().is_null());
                        let style = unsafe { &*item.Style() };
                        if style.ShouldWrapLine()
                            && !style.ShouldBreakSpaces()
                            && IsBreakableSpace(units[item_result.StartOffset() as usize])
                        {
                            if !item_result.shape_result.Get().is_null()
                                && IsAllBreakableSpaces(
                                    &text,
                                    item_result.StartOffset() + 1,
                                    item_result.EndOffset(),
                                )
                            {
                                continue;
                            }
                            action = Some((index, LineBreakState::kTrailing));
                            break;
                        }
                    }
                } else if item.Type() == InlineItemType::kControl {
                    debug_assert_ne!(units[item_result.StartOffset() as usize], 0x0a);
                    debug_assert!(!item.Style().is_null());
                    let style = unsafe { &*item.Style() };
                    if style.ShouldWrapLine() && !style.ShouldBreakSpaces() {
                        continue;
                    }
                } else if item.Type() == InlineItemType::kOpenTag {
                    if open_tag_count == 0 {
                        new_end = index;
                    }
                    open_tag_count += 1;
                    continue;
                } else if item.Type() == InlineItemType::kCloseTag {
                    if open_tag_count > 0 {
                        open_tag_count -= 1;
                    }
                    continue;
                } else if IsTrailableItemType(item.Type()) {
                    continue;
                }

                action = Some((
                    if open_tag_count > 0 { new_end } else { index },
                    LineBreakState::kDone,
                ));
                break;
            }
            if action.is_none() && open_tag_count > 0 {
                action = Some((new_end, LineBreakState::kDone));
            }
            action
        };
        if let Some((index, state)) = rewind_action {
            self.state_ = Some(state);
            if state == LineBreakState::kDone {
                debug_assert!(!line_info.IsLastLine());
            }
            self.Rewind(index, line_info);
            return;
        }
        self.trailing_whitespace_ = WhitespaceState::kUnknown;
        self.position_ = line_info.ComputeWidth();
        self.state_ = Some(LineBreakState::kDone);
        debug_assert!(!line_info.IsLastLine());
        if self.IsAtEnd() {
            line_info.SetIsLastLine(true);
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:4445-4510
    fn Rewind(&mut self, new_end: u32, line_info: &mut LineInfo) {
        debug_assert!(new_end < line_info.Results().size());
        if let Some(last_rewind) = self.last_rewind_ {
            assert!(
                self.current_.item_index != last_rewind.from_item_index
                    || new_end != last_rewind.to_index,
                "line-break rewind loop"
            );
            self.last_rewind_ = Some(RewindIndex {
                from_item_index: self.current_.item_index,
                to_index: new_end,
            });
        }

        self.RewindFloats(new_end, line_info);

        if new_end != 0 {
            let previous_result = &line_info.Results()[new_end as usize - 1];
            self.MoveToNextOfResult(previous_result);
            self.trailing_whitespace_ = WhitespaceState::kUnknown;
            let items = self.Items() as *const InlineItems;
            while !self.IsAtEnd() {
                let current_item = unsafe { &*(&*items)[self.current_.item_index as usize].Get() };
                if current_item.Type() != InlineItemType::kText || current_item.Length() != 0 {
                    break;
                }
                self.HandleEmptyText(current_item, line_info);
            }
        } else {
            self.current_ = *line_info.Start();
            let item_results = line_info.Results();
            if !item_results.empty() && item_results[0].IsRubyColumn() {
                let ruby_column = unsafe { &*item_results[0].ruby_column.Get() };
                self.ruby_break_token_ = ruby_column.start_ruby_break_token.Get();
            }
            self.trailing_whitespace_ = WhitespaceState::kLeading;
            self.maybe_have_end_overhang_ = false;
        }
        let current_style = self.ComputeCurrentStyle(new_end, line_info);
        self.SetCurrentStyle(unsafe { &*current_style });

        let item_results = line_info.MutableResults();
        item_results.Shrink(new_end);
        self.trailing_collapsible_space_ = None;
        if self.hyphen_index_.is_some_and(|index| index >= new_end) {
            self.hyphen_index_ = None;
        }
        if self.hyphen_index_.is_none() && self.has_any_hyphens_ {
            self.RestoreLastHyphen(item_results);
        }
        self.position_ = line_info.ComputeWidth();
        if self.has_cloned_box_decorations_ {
            self.RecalcClonedBoxDecorations();
        }
    }

    // cpp: layoutng_inline/line_breaker.cc:4515-4546
    fn ComputeCurrentStyle(
        &self,
        mut item_result_index: u32,
        line_info: &LineInfo,
    ) -> *const ComputedStyle {
        let item_results = line_info.Results();
        let mut item = unsafe { &*item_results[item_result_index as usize].item.Get() };
        if item.Type() == InlineItemType::kText || item.Type() == InlineItemType::kCloseTag {
            debug_assert!(!item.Style().is_null());
            return item.Style();
        }
        while item_result_index != 0 {
            item_result_index -= 1;
            item = unsafe { &*item_results[item_result_index as usize].item.Get() };
            if item.Type() == InlineItemType::kText || item.Type() == InlineItemType::kOpenTag {
                debug_assert!(!item.Style().is_null());
                return item.Style();
            }
            if item.Type() == InlineItemType::kCloseTag {
                let parent = unsafe { &*item.GetLayoutObject() }.Parent();
                return unsafe { &*parent }.StyleRef();
            }
        }
        if !self.break_token_.is_null() {
            let style = unsafe { &*self.break_token_ }.Style();
            if !style.is_null() {
                return style;
            }
        }
        line_info.LineStyle()
    }

    // cpp: layoutng_inline/line_breaker.cc:432-452
    fn UpdateAvailableWidth(&mut self) {
        let mut available_width = if self.override_available_width_ != LayoutUnit::default() {
            if self.line_clamp_ellipsis_width_ != LayoutUnit::default() {
                (self.line_opportunity_.AvailableInlineSize() - self.line_clamp_ellipsis_width_)
                    .min(self.override_available_width_)
            } else {
                self.override_available_width_
            }
        } else {
            self.line_opportunity_.AvailableInlineSize() - self.line_clamp_ellipsis_width_
        };
        available_width = available_width.max(self.cloned_box_decorations_initial_size_);
        available_width = available_width.min(LayoutUnit::NearlyMax());
        self.base_available_width_ = available_width;
        self.UpdateAvailableWidthFromBaseAvailableWidth();
    }

    // cpp: layoutng_inline/line_breaker.cc:454-463
    fn UpdateAvailableWidthFromBaseAvailableWidth(&mut self) {
        if RuntimeEnabledFeatures::BoxDecorationBreakCloneLineBreakingEnabled()
            && self.cloned_box_decorations_end_size_ != LayoutUnit::default()
        {
            self.available_width_ = (self.base_available_width_
                - self.cloned_box_decorations_end_size_)
                .max(LayoutUnit::default());
        } else {
            self.available_width_ = self.base_available_width_;
        }
    }
}
