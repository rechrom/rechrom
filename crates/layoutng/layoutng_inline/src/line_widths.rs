// C++: layoutng_inline/line_widths.h/.cc.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use foundation::{LayoutUnit, Member, WtfSizeT};
use layoutng::internal::exclusions::layout_opportunity::LayoutOpportunity;
use layoutng::internal::inline_item::InlineItemType;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::text_fit_scale::TextFitBlockScale;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_style::style::computed_style_constants::EVerticalAlign;

use crate::inline_box_state::InlineBoxState;

// cpp: layoutng_inline/line_widths.h:21-44
#[derive(Default)]
pub struct LineWidths {
    default_width_: LayoutUnit,
    excluded_width_: LayoutUnit,
    num_excluded_lines_: WtfSizeT,
}

impl LineWidths {
    // cpp: layoutng_inline/line_widths.h:25-27
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(width: LayoutUnit) -> Self {
        Self {
            default_width_: width,
            ..Self::default()
        }
    }

    // cpp: layoutng_inline/line_widths.h:29-30
    pub fn Default(&self) -> LayoutUnit {
        self.default_width_
    }

    pub fn HasExclusions(&self) -> bool {
        self.num_excluded_lines_ != 0
    }

    // cpp: layoutng_inline/line_widths.h:47-52
    pub fn At(&self, index: WtfSizeT) -> LayoutUnit {
        if index < self.num_excluded_lines_ {
            return self.excluded_width_;
        }
        self.default_width_
    }

    // cpp: layoutng_inline/line_widths.cc:17-156
    pub fn Set(
        &mut self,
        node: &InlineNode,
        opportunities: &[LayoutOpportunity],
        break_token: *const InlineBreakToken,
    ) -> bool {
        // cpp: layoutng_inline/line_widths.cc:21-29
        debug_assert!(!opportunities.is_empty());
        let first_opportunity = &opportunities[0];
        if opportunities.len() == 1 && !node.HasFloats() {
            debug_assert!(!first_opportunity.HasShapeExclusions());
            self.default_width_ = first_opportunity.rect.InlineSize();
            debug_assert_eq!(self.num_excluded_lines_, 0);
            return true;
        }

        // cpp: layoutng_inline/line_widths.cc:31-34
        if opportunities.len() > 2 || first_opportunity.HasShapeExclusions() {
            return false;
        }

        // cpp: layoutng_inline/line_widths.cc:36-45
        let block_style = node.Style();
        let block_font = block_style.GetFont();
        let baseline_type = block_style.GetFontBaseline();
        let mut line_box = InlineBoxState::default();
        line_box.ComputeTextMetrics(
            block_style,
            unsafe { &*block_font },
            baseline_type,
            TextFitBlockScale::kFixed,
        );

        // cpp: layoutng_inline/line_widths.cc:47-60
        let primary_font = unsafe { &*block_font }.PrimaryFont();
        debug_assert!(!primary_font.is_null());
        let items_data = node.ItemsData(false);
        debug_assert!(std::ptr::eq(items_data, node.ItemsData(true)));
        let mut items = &items_data.items[..];
        let mut is_empty_so_far = true;
        if !break_token.is_null() {
            let token = unsafe { &*break_token };
            debug_assert!(!token.Start().IsZero());
            items = &items[token.StartItemIndex() as usize..];
            is_empty_so_far = false;
        }

        // cpp: layoutng_inline/line_widths.cc:61-127
        for item_ptr in items {
            let item = unsafe { &*item_ptr.Get() };
            match item.Type() {
                InlineItemType::kText => {
                    if item.Length() != 0 {
                        let shape_result = item.TextShapeResult();
                        debug_assert!(!shape_result.is_null());
                        if unsafe { &*shape_result }.HasFallbackFonts(primary_font) {
                            let item_style = item.Style();
                            debug_assert!(!item_style.is_null());
                            let item_style = unsafe { &*item_style };
                            let mut text_box = InlineBoxState::default();
                            text_box.ComputeTextMetrics(
                                item_style,
                                unsafe { &*item_style.GetFont() },
                                baseline_type,
                                TextFitBlockScale::kFixed,
                            );
                            if text_box.include_used_fonts {
                                text_box.style = Member::from_ptr(item_style as *const _ as *mut _);
                                let shape_result_view =
                                    ShapeResultView::CreateFromResult(shape_result);
                                text_box.AccumulateUsedFonts(shape_result_view);
                            }
                            if !line_box.metrics.Contains(&text_box.metrics) {
                                return false;
                            }
                        }
                    }
                }
                InlineItemType::kOpenTag => {
                    let style = item.Style();
                    debug_assert!(!style.is_null());
                    if unsafe { &*style }.VerticalAlign() != EVerticalAlign::kBaseline {
                        return false;
                    }
                }
                InlineItemType::kCloseTag
                | InlineItemType::kControl
                | InlineItemType::kOutOfFlowPositioned
                | InlineItemType::kBidiControl
                | InlineItemType::kOpenRubyColumn
                | InlineItemType::kCloseRubyColumn
                | InlineItemType::kRubyLinePlaceholder => {}
                InlineItemType::kFloating => {
                    if !is_empty_so_far {
                        return false;
                    }
                }
                InlineItemType::kAtomicInline
                | InlineItemType::kBlockInInline
                | InlineItemType::kInitialLetterBox
                | InlineItemType::kListMarker => return false,
            }
            if is_empty_so_far && !item.IsEmptyItem() {
                is_empty_so_far = false;
            }
        }

        // cpp: layoutng_inline/line_widths.cc:129-139
        if opportunities.len() == 1 {
            self.default_width_ = first_opportunity.rect.InlineSize();
            return true;
        }

        // cpp: layoutng_inline/line_widths.cc:141-155
        let line_height = line_box.metrics.LineHeight();
        if line_height <= LayoutUnit::default() {
            return false;
        }
        debug_assert!(opportunities.len() >= 2);
        let last_opportunity = opportunities.last().unwrap();
        debug_assert!(!last_opportunity.HasShapeExclusions());
        self.default_width_ = last_opportunity.rect.InlineSize();
        let exclusion_block_size =
            last_opportunity.rect.BlockStartOffset() - first_opportunity.rect.BlockStartOffset();
        debug_assert!(exclusion_block_size > LayoutUnit::default());
        let num_excluded_lines = (exclusion_block_size.ToFloat() / line_height.ToFloat()).ceil();
        debug_assert!(num_excluded_lines >= 1.0);
        // Rust's float-to-integer cast saturates, matching saturated_cast here.
        self.num_excluded_lines_ = num_excluded_lines as WtfSizeT;
        self.excluded_width_ = first_opportunity.rect.InlineSize();
        true
    }
}
