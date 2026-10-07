// C++: layoutng_inline/line_truncator.h/.cc. Ellipsis preparation and the
// child truncation algorithms remain in this source-owned module.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::shape_result::ShapeResult;
use font_engine::fonts::shaping::shape_result_types::AdjustMidCluster;
use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use font_engine::fonts::simple_font_data::SimpleFontData;
use font_engine::text::native::bidi_paragraph::BidiParagraph;
use font_engine::text::native::character::Character;
use font_engine::{FontHeight, HarfBuzzShaper};
use foundation::{
    DirectionFromLevel, IsLtr, IsRtl, LayoutUnit, Member, RuntimeEnabledFeatures, String,
    TextDirection, To, Visitor,
};
use icu_bidi::UBiDiLevel;
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::text_offset_range::TextOffsetRange;
use layoutng_fragment_tree::logical_line_item::{LogicalLineItem, LogicalLineItems};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::inline_box_state::InlineLayoutStateStack;
use crate::line_info::LineInfo;

// cpp: layoutng_inline/line_truncator.cc:23-27
fn is_left_most_offset(shape_result: &ShapeResult, offset: u32) -> bool {
    if shape_result.IsRtl() {
        offset == shape_result.NumCharacters()
    } else {
        offset == 0
    }
}

// cpp: layoutng_inline/line_truncator.cc:29-33
fn is_right_most_offset(shape_result: &ShapeResult, offset: u32) -> bool {
    if shape_result.IsRtl() {
        offset == 0
    } else {
        offset == shape_result.NumCharacters()
    }
}

// cpp: layoutng_inline/line_truncator.cc:38-42
fn is_forced_line_break(character: u16) -> bool {
    matches!(
        character,
        0x000a | 0x000b | 0x000c | 0x0085 | 0x2028 | 0x2029
    )
}

// A UTF-16 vector maps the C++ StringBuilder append order without passing
// through UTF-8, which could alter offset units for surrogate pairs.
// cpp: layoutng_inline/line_truncator.cc:44-78
fn suppress_line_breaks(text: &String) -> String {
    let units = text.Span16().unwrap_or_default();
    let Some(first_break) = units.iter().position(|&unit| is_forced_line_break(unit)) else {
        return text.clone();
    };
    let mut result = Vec::with_capacity(units.len());
    result.extend_from_slice(&units[..first_break]);
    let mut index = first_break;
    while index < units.len() {
        let character = units[index];
        index += 1;
        if !is_forced_line_break(character) {
            result.push(character);
            continue;
        }
        while index < units.len() && is_forced_line_break(units[index]) {
            index += 1;
        }
        result.push(u16::from(b' '));
    }
    String::from_utf16(&result)
}

// C++ allocates each result in the GC heap. Rust's stack-scoped vector owns
// the records while Member preserves the referenced shaped result identity.
// cpp: layoutng_inline/line_truncator.h:105-119
struct EllipsisShapeResult {
    shape_result: Member<ShapeResultView>,
    text: String,
    bidi_level: UBiDiLevel,
}

impl EllipsisShapeResult {
    fn new(shape_result: *mut ShapeResultView, text: String, bidi_level: UBiDiLevel) -> Self {
        Self {
            shape_result: Member::from_ptr(shape_result),
            text,
            bidi_level,
        }
    }

    // cpp: layoutng_inline/line_truncator.h:118-118
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.shape_result);
    }
}

// cpp: layoutng_inline/line_truncator.h:30-126
pub struct LineTruncator {
    line_style_: *const ComputedStyle,
    available_width_: LayoutUnit,
    line_direction_: TextDirection,
    ellipsis_font_data_: *const SimpleFontData,
    ellipsis_width_: LayoutUnit,
    ellipsis_shape_results_: Vec<EllipsisShapeResult>,
    use_first_line_style_: bool,
    is_ellipsis_caused_by_line_clamp_: bool,
}

impl LineTruncator {
    // cpp: layoutng_inline/line_truncator.cc:82-88
    pub fn new(line_info: &LineInfo, is_ellipsis_caused_by_line_clamp: bool) -> Self {
        Self {
            line_style_: line_info.LineStyle(),
            available_width_: line_info.AvailableWidth() - line_info.TextIndent(),
            line_direction_: line_info.BaseDirection(),
            ellipsis_font_data_: std::ptr::null(),
            ellipsis_width_: LayoutUnit::default(),
            ellipsis_shape_results_: Vec::new(),
            use_first_line_style_: line_info.UseFirstLineStyle(),
            is_ellipsis_caused_by_line_clamp_: is_ellipsis_caused_by_line_clamp,
        }
    }

    // cpp: layoutng_inline/line_truncator.h:34-35
    pub fn new_default(line_info: &LineInfo) -> Self {
        Self::new(line_info, false)
    }

    // cpp: layoutng_inline/line_truncator.cc:90-95
    fn ellipsis_style(&self) -> &ComputedStyle {
        debug_assert!(!self.line_style_.is_null());
        unsafe { &*self.line_style_ }
    }

    // cpp: layoutng_inline/line_truncator.cc:97-107
    fn compute_ellipsis_text(&self) -> String {
        let text_overflow = self.ellipsis_style().TextOverflow();
        if text_overflow.IsString() && !self.is_ellipsis_caused_by_line_clamp_ {
            return suppress_line_breaks(text_overflow.StringValue());
        }
        if !self.ellipsis_font_data_.is_null()
            && unsafe { &*self.ellipsis_font_data_ }.GlyphForCharacter(0x2026) != 0
        {
            String::from_utf16(&[0x2026])
        } else {
            String::from("...")
        }
    }

    // cpp: layoutng_inline/line_truncator.cc:109-142
    fn setup_ellipsis(&mut self) {
        let font = self.ellipsis_style().GetFont();
        self.ellipsis_font_data_ = unsafe { &*font }.PrimaryFont();
        debug_assert!(!self.ellipsis_font_data_.is_null());
        let ellipsis_text = self.compute_ellipsis_text();
        let mut bidi = BidiParagraph::default();
        if RuntimeEnabledFeatures::TextOverflowStringEnabled()
            && Character::MaybeBidiRtlString(&ellipsis_text)
            && bidi.SetParagraph(&ellipsis_text, Some(self.line_direction_))
            && (!bidi.IsUnidirectional() || IsRtl(bidi.BaseDirection()))
        {
            let mut runs = Vec::new();
            bidi.GetVisualRuns(&ellipsis_text, &mut runs);
            for run in runs {
                let units = ellipsis_text.Span16().unwrap();
                let run_text = String::from_utf16(&units[run.start as usize..run.end as usize]);
                let shaper = HarfBuzzShaper::new(run_text.clone());
                let direction = DirectionFromLevel(u32::from(run.level));
                let shape = ShapeResultView::CreateFromResult(shaper.Shape(font, direction));
                self.ellipsis_shape_results_
                    .push(EllipsisShapeResult::new(shape, run_text, run.level));
                self.ellipsis_width_ += unsafe { &*shape }.SnappedWidth();
            }
        } else {
            let shaper = HarfBuzzShaper::new(ellipsis_text.clone());
            let shape = ShapeResultView::CreateFromResult(shaper.Shape(font, self.line_direction_));
            self.ellipsis_shape_results_
                .push(EllipsisShapeResult::new(shape, ellipsis_text, 0));
            self.ellipsis_width_ = unsafe { &*shape }.SnappedWidth();
        }
    }

    // cpp: layoutng_inline/line_truncator.cc:144-186
    fn place_ellipsis_next_to(
        &self,
        line_box: &mut LogicalLineItems,
        ellipsized_child: *const LogicalLineItem,
    ) -> LayoutUnit {
        let ellipsized_child = unsafe { &*ellipsized_child };
        debug_assert!(ellipsized_child.HasInFlowFragment());
        let layout_object = ellipsized_child.GetMutableLayoutObject();
        debug_assert!(!layout_object.is_null());
        let layout_object = unsafe { &*layout_object };
        debug_assert!(layout_object.IsInline());
        debug_assert!(layout_object.IsText() || layout_object.IsAtomicInline());

        let ellipsis_inline_offset = if IsLtr(self.line_direction_) {
            ellipsized_child.InlineOffset() + ellipsized_child.inline_size
        } else {
            ellipsized_child.InlineOffset() - self.ellipsis_width_
        };
        let mut metrics = FontHeight::default();
        debug_assert!(!self.ellipsis_font_data_.is_null());
        if !self.ellipsis_font_data_.is_null() {
            metrics = unsafe { &*self.ellipsis_font_data_ }
                .GetFontMetrics()
                .GetFontHeight(self.ellipsis_style().GetFontBaseline());
        }

        debug_assert!(!self.ellipsis_shape_results_.is_empty());
        let style_variant = if self.use_first_line_style_ {
            StyleVariant::kFirstLineEllipsis
        } else {
            StyleVariant::kStandardEllipsis
        };
        let mut current_offset = ellipsis_inline_offset;
        for ellipsis in &self.ellipsis_shape_results_ {
            let shape_result = unsafe { &*ellipsis.shape_result.Get() };
            let inline_size = shape_result.SnappedWidth();
            line_box.AddChild(LogicalLineItem::ellipsis(
                layout_object,
                style_variant,
                shape_result,
                &ellipsis.text,
                &LogicalRect::from_units(
                    current_offset,
                    -metrics.ascent,
                    inline_size,
                    metrics.LineHeight(),
                ),
                ellipsis.bidi_level,
            ));
            current_offset += inline_size;
        }
        ellipsis_inline_offset
    }

    // cpp: layoutng_inline/line_truncator.cc:188-217
    fn add_truncated_child(
        &self,
        source_index: usize,
        leave_one_character: bool,
        position: LayoutUnit,
        edge: TextDirection,
        line_box: &mut LogicalLineItems,
        box_states: &mut InlineLayoutStateStack,
    ) -> Option<usize> {
        let source_item = &line_box[source_index];
        debug_assert!(!source_item.shape_result.Get().is_null());
        let shape_result = unsafe { &*source_item.shape_result.Get() }.CreateShapeResult();
        let shape_result = unsafe { &*shape_result };
        let mut text_offset = shape_result.OffsetToFit(position.ToFloat(), edge);
        if if IsLtr(edge) {
            is_left_most_offset(shape_result, text_offset)
        } else {
            is_right_most_offset(shape_result, text_offset)
        } {
            if !leave_one_character {
                return None;
            }
            let keep_offset = if IsRtl(edge) == shape_result.IsRtl() {
                1
            } else {
                shape_result.NumCharacters() - 1
            };
            text_offset = shape_result.OffsetToFit(
                shape_result.PositionForOffset(keep_offset, AdjustMidCluster::kToEnd),
                edge,
            );
        }
        let new_index = line_box.size() as usize;
        let truncated = self.truncate_text(source_item, shape_result, text_offset, edge);
        line_box.AddChild(truncated);
        box_states.ChildInserted(new_index as u32);
        Some(new_index)
    }

    // cpp: layoutng_inline/line_truncator.cc:618-632
    fn truncate_text(
        &self,
        item: &LogicalLineItem,
        shape_result: &ShapeResult,
        offset_to_fit: u32,
        direction: TextDirection,
    ) -> LogicalLineItem {
        let text_offset = if direction == shape_result.Direction() {
            TextOffsetRange::new(item.StartOffset(), item.StartOffset() + offset_to_fit)
        } else {
            TextOffsetRange::new(item.StartOffset() + offset_to_fit, item.EndOffset())
        };
        let shape_view = ShapeResultView::CreateFromResultRange(
            shape_result,
            text_offset.start,
            text_offset.end,
        );
        debug_assert!(!item.inline_item.Get().is_null());
        LogicalLineItem::reshaped_text(item, unsafe { &*shape_view }, &text_offset)
    }

    // cpp: layoutng_inline/line_truncator.cc:496-520
    fn hide_child(&self, child: &mut LogicalLineItem) {
        debug_assert!(child.HasInFlowFragment());
        let layout_result = child.layout_result.Get();
        if !layout_result.is_null() {
            let fragment =
                To::<PhysicalBoxFragment>(unsafe { &*layout_result }.GetPhysicalFragment());
            let fragment = unsafe { &*fragment };
            if fragment.HasOutOfFlowPositionedDescendants() {
                return;
            }
            debug_assert!(fragment.IsMonolithic());
            let layout_object = fragment.GetMutableLayoutObject();
            debug_assert!(!layout_object.is_null());
            debug_assert!(unsafe { &*layout_object }.IsAtomicInline());
            unsafe { &mut *layout_object }.SetIsTruncated(true);
            return;
        }
        if !child.inline_item.Get().is_null() {
            child.is_hidden_for_paint = true;
            return;
        }
        unreachable!("in-flow item without a result or inline item");
    }

    // cpp: layoutng_inline/line_truncator.cc:526-573
    fn ellipsize_child(
        &self,
        line_width: LayoutUnit,
        ellipsis_width: LayoutUnit,
        is_first_child: bool,
        child: &mut LogicalLineItem,
        truncated_child: &mut Option<LogicalLineItem>,
    ) -> bool {
        debug_assert!(truncated_child.is_none());
        if !child.HasInFlowFragment() || child.IsInlineBox() {
            return false;
        }
        let child_inline_offset = if IsLtr(self.line_direction_) {
            child.InlineOffset()
        } else {
            line_width - (child.InlineOffset() + child.inline_size)
        };
        let mut space_for_child = self.available_width_ - child_inline_offset;
        if space_for_child <= LayoutUnit::default() {
            if !is_first_child {
                self.hide_child(child);
            }
            return false;
        }
        space_for_child -= ellipsis_width;
        if space_for_child >= child.inline_size {
            return true;
        }
        if self.truncate_child(space_for_child, is_first_child, child, truncated_child) {
            return true;
        }
        if !is_first_child {
            self.hide_child(child);
        }
        false
    }

    // cpp: layoutng_inline/line_truncator.cc:582-616
    fn truncate_child(
        &self,
        space_for_child: LayoutUnit,
        is_first_child: bool,
        child: &LogicalLineItem,
        truncated_child: &mut Option<LogicalLineItem>,
    ) -> bool {
        debug_assert!(truncated_child.is_none());
        if space_for_child <= LayoutUnit::default() && !is_first_child {
            return false;
        }
        let view = child.shape_result.Get();
        if view.is_null() {
            return is_first_child;
        }
        let shape_result = unsafe { &*view }.CreateShapeResult();
        debug_assert!(!shape_result.is_null());
        let shape_result = unsafe { &*shape_result };
        let original_offset = child.text_offset;
        let position = if IsLtr(self.line_direction_) {
            space_for_child.ToFloat()
        } else {
            shape_result.Width() - space_for_child.ToFloat()
        };
        let mut offset_to_fit = shape_result.OffsetToFit(position, self.line_direction_);
        debug_assert!(offset_to_fit <= original_offset.Length());
        if offset_to_fit == 0 || offset_to_fit == original_offset.Length() {
            if !is_first_child {
                return false;
            }
            offset_to_fit = if offset_to_fit == 0 {
                1
            } else {
                offset_to_fit - 1
            };
        }
        *truncated_child =
            Some(self.truncate_text(child, shape_result, offset_to_fit, self.line_direction_));
        true
    }

    // cpp: layoutng_inline/line_truncator.cc:219-286
    pub fn TruncateLine(
        &mut self,
        line_width: LayoutUnit,
        line_box: &mut LogicalLineItems,
        box_states: &mut InlineLayoutStateStack,
    ) -> LayoutUnit {
        self.setup_ellipsis();

        let mut ellipsized_child: *mut LogicalLineItem = std::ptr::null_mut();
        let mut truncated_child = None;
        if IsLtr(self.line_direction_) {
            let first_child = line_box.FirstInFlowChild();
            for child in line_box.iter_mut().rev() {
                let child_ptr = child as *mut LogicalLineItem;
                if self.ellipsize_child(
                    line_width,
                    self.ellipsis_width_,
                    child_ptr == first_child,
                    child,
                    &mut truncated_child,
                ) {
                    ellipsized_child = child_ptr;
                    break;
                }
            }
        } else {
            let first_child = line_box.LastInFlowChild();
            for child in line_box.iter_mut() {
                let child_ptr = child as *mut LogicalLineItem;
                if self.ellipsize_child(
                    line_width,
                    self.ellipsis_width_,
                    child_ptr == first_child,
                    child,
                    &mut truncated_child,
                ) {
                    ellipsized_child = child_ptr;
                    break;
                }
            }
        }
        if ellipsized_child.is_null() {
            return line_width;
        }

        if let Some(truncated_child) = truncated_child {
            let index = line_box
                .iter()
                .position(|child| std::ptr::eq(child, ellipsized_child))
                .expect("ellipsized child must be in line");
            line_box.InsertChild(index as u32 + 1, truncated_child);
            box_states.ChildInserted(index as u32 + 1);
            self.hide_child(&mut line_box[index]);
            debug_assert!(line_box[index + 1].inline_size <= line_box[index].inline_size);
            if IsRtl(self.line_direction_) {
                let size_difference = line_box[index].inline_size - line_box[index + 1].inline_size;
                line_box[index + 1].rect.offset.inline_offset += size_difference;
            }
            ellipsized_child = &mut line_box[index + 1];
        }

        let ellipsis_inline_offset = self.place_ellipsis_next_to(line_box, ellipsized_child);
        (ellipsis_inline_offset + self.ellipsis_width_).max(line_width)
    }

    // cpp: layoutng_inline/line_truncator.cc:288-492
    pub fn TruncateLineInTheMiddle(
        &mut self,
        mut line_width: LayoutUnit,
        line: &mut LogicalLineItems,
        box_states: &mut InlineLayoutStateStack,
    ) -> LayoutUnit {
        self.setup_ellipsis();

        let mut initial_index_left = usize::MAX;
        let mut initial_index_right = usize::MAX;
        for index in 0..line.size() as usize {
            let child = &line[index];
            if child.IsPlaceholder() {
                continue;
            }
            if child.shape_result.Get().is_null() {
                if initial_index_right != usize::MAX {
                    break;
                }
                continue;
            }
            if child.GetNode().is_null() {
                continue;
            }
            if initial_index_left == usize::MAX {
                initial_index_left = index;
            }
            initial_index_right = index;
        }
        if initial_index_left == usize::MAX {
            return line_width;
        }
        debug_assert_ne!(initial_index_right, usize::MAX);
        debug_assert!(line[initial_index_left].HasInFlowFragment());
        debug_assert!(line[initial_index_right].HasInFlowFragment());

        let static_width_left = line[initial_index_left].InlineOffset();
        let mut static_width_right = LayoutUnit::default();
        if initial_index_right + 1 < line.size() as usize {
            let item = &line[initial_index_right];
            let truncatable_right = item.InlineOffset() + item.inline_size;
            if line_width <= truncatable_right || truncatable_right < LayoutUnit::default() {
                return line_width;
            }
            static_width_right = line_width - truncatable_right;
        }
        let available_width = self.available_width_ - static_width_left - static_width_right;
        if available_width <= self.ellipsis_width_ {
            return line_width;
        }
        let mut available_width_left = (available_width - self.ellipsis_width_) / 2;
        let mut available_width_right = available_width_left;
        let new_child_start = line.size() as usize;
        let mut index_left = initial_index_left;
        let mut index_right = initial_index_right;

        if IsLtr(self.line_direction_) {
            while available_width_left >= line[index_left].inline_size {
                available_width_left -= line[index_left].inline_size;
                index_left += 1;
                if index_left >= line.size() as usize {
                    return line_width;
                }
            }
            debug_assert!(index_left <= index_right);
            debug_assert!(!line[index_left].IsPlaceholder());
            let new_index = self.add_truncated_child(
                index_left,
                index_left == initial_index_left,
                available_width_left,
                TextDirection::kLtr,
                line,
                box_states,
            );
            if let Some(new_index) = new_index {
                let child = &line[new_index] as *const LogicalLineItem;
                self.place_ellipsis_next_to(line, child);
                available_width_right +=
                    available_width_left - line[new_index].inline_size.ClampNegativeToZero();
            } else {
                debug_assert!(index_left > initial_index_left && index_left > 0);
                let mut index = index_left;
                loop {
                    index -= 1;
                    if line[index].HasInFlowFragment() {
                        break;
                    }
                    debug_assert!(line[index].IsPlaceholder());
                }
                let child = &line[index] as *const LogicalLineItem;
                self.place_ellipsis_next_to(line, child);
                available_width_right += available_width_left;
            }

            while available_width_right >= line[index_right].inline_size {
                available_width_right -= line[index_right].inline_size;
                if index_right == 0 {
                    break;
                }
                index_right -= 1;
            }
            let mut new_modified_right_offset =
                line[line.size() as usize - 1].InlineOffset() + self.ellipsis_width_;
            debug_assert!(index_left <= index_right);
            debug_assert!(!line[index_right].IsPlaceholder());
            if available_width_right > LayoutUnit::default() {
                if let Some(new_index) = self.add_truncated_child(
                    index_right,
                    false,
                    line[index_right].inline_size - available_width_right,
                    TextDirection::kRtl,
                    line,
                    box_states,
                ) {
                    line[new_index].rect.offset.inline_offset = new_modified_right_offset;
                    new_modified_right_offset += line[new_index].inline_size;
                }
            }
            let offset_diff = line[index_right].InlineOffset() + line[index_right].inline_size
                - new_modified_right_offset;
            for index in index_right + 1..new_child_start {
                line[index].rect.offset.inline_offset -= offset_diff;
            }
            line_width -= offset_diff;
        } else {
            while available_width_right >= line[index_right].inline_size {
                available_width_right -= line[index_right].inline_size;
                if index_right == 0 {
                    return line_width;
                }
                index_right -= 1;
            }
            debug_assert!(index_left <= index_right);
            debug_assert!(!line[index_right].IsPlaceholder());
            let new_index = self.add_truncated_child(
                index_right,
                index_right == initial_index_right,
                line[index_right].inline_size - available_width_right,
                TextDirection::kRtl,
                line,
                box_states,
            );
            if let Some(new_index) = new_index {
                let size_difference = line[index_right].inline_size - line[new_index].inline_size;
                line[new_index].rect.offset.inline_offset += size_difference;
                let child = &line[new_index] as *const LogicalLineItem;
                self.place_ellipsis_next_to(line, child);
                available_width_left +=
                    available_width_right - line[new_index].inline_size.ClampNegativeToZero();
            } else {
                debug_assert!(index_right < initial_index_right);
                let mut index = index_right;
                loop {
                    index += 1;
                    if line[index].HasInFlowFragment() {
                        break;
                    }
                    debug_assert!(line[index].IsPlaceholder());
                }
                let child = &line[index] as *const LogicalLineItem;
                self.place_ellipsis_next_to(line, child);
                available_width_left += available_width_right;
            }
            let ellipsis_offset = line[line.size() as usize - 1].InlineOffset();

            while available_width_left >= line[index_left].inline_size {
                available_width_left -= line[index_left].inline_size;
                index_left += 1;
                if index_left >= line.size() as usize {
                    break;
                }
            }
            debug_assert!(index_left <= index_right);
            debug_assert!(!line[index_left].IsPlaceholder());
            if available_width_left > LayoutUnit::default() {
                if let Some(new_index) = self.add_truncated_child(
                    index_left,
                    false,
                    available_width_left,
                    TextDirection::kLtr,
                    line,
                    box_states,
                ) {
                    line[new_index].rect.offset.inline_offset =
                        ellipsis_offset - line[new_index].inline_size;
                }
            }
            let offset_diff =
                line[line.size() as usize - 1].InlineOffset() - line[index_left].InlineOffset();
            for index in (0..index_left).rev() {
                line[index].rect.offset.inline_offset += offset_diff;
            }
            line_width -= offset_diff;
        }
        for index in index_left..=index_right {
            if line[index].HasInFlowFragment() {
                self.hide_child(&mut line[index]);
            }
        }
        line_width
    }
}
