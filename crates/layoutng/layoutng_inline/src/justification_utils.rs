// C++: layoutng_inline/justification_utils.h/.cc. Source behavior mapped;
// integration awaits completion of the layoutng_inline package.
#![allow(non_snake_case)]

use font_engine::fonts::shaping::shape_result_spacing::{ExpansionSetup, ShapeResultSpacing};
use font_engine::fonts::shaping::shape_result_view::ShapeResultView;
use foundation::blink_geometry::geometry::TextRunLayoutUnit;
use foundation::{IsRtl, LayoutUnit, StringView, TextDirection, TextJustify};
use layoutng::internal::inline_item::InlineItemType;
use layoutng::internal::inline_item_result::{InlineItemResult, InlineItemResults};
use layoutng_fragment_tree::logical_line_item::LogicalLineItem;

use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;
use crate::line_info::LineInfo;

// cpp: layoutng_inline/justification_utils.cc:18-19
const TEXT_COMBINE_ITEM_MARKER: u16 = 0x3042;
const BASE_SHORTER_RUBY_MARKER: u16 = 0xfffc;

// cpp: layoutng_inline/justification_utils.h:16-21
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JustificationTarget {
    kNormal,
    kRubyBase,
    kRubyText,
    kSvgText,
}

// cpp: layoutng_inline/justification_utils.cc:32-39
fn GetTextJustify(item_result: &InlineItemResult) -> TextJustify {
    let item = unsafe { &*item_result.item.Get() };
    if item.GetLayoutObject().is_null() {
        return TextJustify::kNone;
    }
    unsafe { &*item.Style() }.GetTextJustify()
}

// cpp: layoutng_inline/justification_utils.cc:41-83
#[inline(never)]
fn SetupItemJustificationOpportunity(
    item_result: &InlineItemResult,
    end_offset: u32,
    base_direction: TextDirection,
    spacing_setup: &mut ExpansionSetup<'_>,
) {
    if item_result.StartOffset() >= end_offset || item_result.has_only_pre_wrap_trailing_spaces {
        return;
    }
    let method = GetTextJustify(item_result);
    let shape = item_result.shape_result.Get();
    if !shape.is_null() {
        let start_index = unsafe { &*shape }.StartIndex();
        let spacing_text = spacing_setup.Spacing().Text().clone();
        let view = StringView::from(&spacing_text).Substring(
            start_index,
            unsafe { &*shape }.EndIndex().min(end_offset) - start_index,
        );
        spacing_setup.CountOpportunities(method, view, base_direction);
    } else if item_result.IsRubyColumn() {
        let ruby_column =
            unsafe { &*(item_result.ruby_column.Get() as *const InlineItemResultRubyColumn) };
        let base_line = &ruby_column.base_line;
        if item_result.inline_size > base_line.Width() {
            spacing_setup.CountOpportunityForCharacter(method, BASE_SHORTER_RUBY_MARKER);
            return;
        }
        let base_results = base_line.Results();
        if base_results.is_empty() {
            return;
        }
        SetupJustificationOpportunity(
            base_results,
            base_results.last().unwrap().EndOffset().min(end_offset),
            base_line.BaseDirection(),
            spacing_setup,
        );
    } else if unsafe { &*item_result.item.Get() }.Type() == InlineItemType::kAtomicInline {
        let item = unsafe { &*item_result.item.Get() };
        spacing_setup.CountOpportunityForCharacter(
            method,
            if item.IsTextCombine() {
                TEXT_COMBINE_ITEM_MARKER
            } else {
                BASE_SHORTER_RUBY_MARKER
            },
        );
    }
}

// cpp: layoutng_inline/justification_utils.cc:85-110
fn SetupJustificationOpportunity(
    results: &InlineItemResults,
    end_offset: u32,
    base_direction: TextDirection,
    spacing_setup: &mut ExpansionSetup<'_>,
) {
    if results.is_empty() {
        return;
    }
    if IsRtl(base_direction) {
        let last = results.last().unwrap();
        if last.hyphen.IsPresent() {
            spacing_setup.CountOpportunities(
                GetTextJustify(last),
                StringView::from(last.hyphen.Text()),
                base_direction,
            );
        }
        for item_result in results.iter().rev() {
            SetupItemJustificationOpportunity(
                item_result,
                end_offset,
                base_direction,
                spacing_setup,
            );
        }
    } else {
        for item_result in results.iter() {
            SetupItemJustificationOpportunity(
                item_result,
                end_offset,
                base_direction,
                spacing_setup,
            );
        }
        let last = results.last().unwrap();
        if last.hyphen.IsPresent() {
            spacing_setup.CountOpportunities(
                GetTextJustify(last),
                StringView::from(last.hyphen.Text()),
                base_direction,
            );
        }
    }
}

// cpp: layoutng_inline/justification_utils.cc:114-177
fn JustifyResults(spacing: &mut ShapeResultSpacing, results: &mut InlineItemResults) -> f32 {
    let mut last_glyph_spacing = 0.0f32;
    for item_result in results.iter_mut() {
        if item_result.has_only_pre_wrap_trailing_spaces {
            break;
        }
        let method = GetTextJustify(item_result);
        let shape_view = item_result.shape_result.Get();
        if !shape_view.is_null() {
            let shape = unsafe { &*shape_view }.CreateShapeResult();
            debug_assert_eq!(unsafe { &*shape }.NumCharacters(), item_result.Length());
            last_glyph_spacing = unsafe { &mut *shape }
                .ApplyExpansion(
                    method,
                    spacing,
                    (item_result.StartOffset() - unsafe { &*shape }.StartIndex()) as i32,
                )
                .ToFloat();
            item_result.inline_size = unsafe { &*shape }.SnappedWidth();
            if item_result.is_hyphenated {
                item_result.inline_size += item_result.hyphen.InlineSize();
            }
            item_result.shape_result =
                foundation::Member::from_ptr(ShapeResultView::CreateFromResult(shape));
        } else if unsafe { &*item_result.item.Get() }.Type() == InlineItemType::kAtomicInline {
            last_glyph_spacing = 0.0;
            let item = unsafe { &*item_result.item.Get() };
            let (spacing_before, spacing_after) = spacing.ComputeExpansionForCharacter(
                method,
                if item.IsTextCombine() {
                    TEXT_COMBINE_ITEM_MARKER
                } else {
                    BASE_SHORTER_RUBY_MARKER
                },
            );
            item_result.inline_size += (spacing_before + spacing_after).To::<6, i32>();
            item_result.spacing_before = spacing_before.To::<6, i32>();
        } else if item_result.IsRubyColumn() {
            let ruby_column =
                unsafe { &mut *(item_result.ruby_column.Get() as *mut InlineItemResultRubyColumn) };
            let base_line = &mut ruby_column.base_line;
            if item_result.inline_size == base_line.Width() {
                last_glyph_spacing = JustifyResults(spacing, base_line.MutableResults());
                let width = base_line.ComputeWidth();
                base_line.SetWidth(base_line.AvailableWidth(), width);
                item_result.inline_size = item_result.inline_size.max(base_line.Width());
                ruby_column.last_base_glyph_spacing = LayoutUnit::from_f32(last_glyph_spacing);
            } else {
                last_glyph_spacing = 0.0;
                let (spacing_before, spacing_after) =
                    spacing.ComputeExpansionForCharacter(method, BASE_SHORTER_RUBY_MARKER);
                let spacing_before_layout = spacing_before.To::<6, i32>();
                item_result.inline_size += spacing_before_layout;
                item_result.spacing_before = spacing_before_layout;
                debug_assert_eq!(spacing_after, TextRunLayoutUnit::new());
            }
        }
    }
    last_glyph_spacing
}

// cpp: layoutng_inline/justification_utils.cc:238-311
fn ApplyJustificationInternal(
    space: LayoutUnit,
    target: JustificationTarget,
    line_info: &LineInfo,
    results: *mut InlineItemResults,
) -> Option<LayoutUnit> {
    if line_info.IsEmptyLine() {
        return None;
    }
    let end_offset = line_info.EndOffsetForJustify();
    if space <= LayoutUnit::default() || end_offset == line_info.StartOffset() {
        return None;
    }

    let text_content = line_info.ItemsData().text_content.clone();
    let line_text = if end_offset == line_info.EndTextOffset() {
        text_content
    } else {
        StringView::from(&text_content)
            .Substring(0, end_offset)
            .ToString()
    };
    if line_text.length() == 0 {
        return None;
    }

    let mut spacing = ShapeResultSpacing::new(&line_text, target == JustificationTarget::kSvgText);
    {
        let mut setup = ExpansionSetup::new(space.To::<16, i64>(), &mut spacing, false, false);
        SetupJustificationOpportunity(
            line_info.Results(),
            end_offset,
            line_info.BaseDirection(),
            &mut setup,
        );
    }
    let is_ruby = matches!(
        target,
        JustificationTarget::kRubyText | JustificationTarget::kRubyBase
    );
    if !spacing.HasExpansion() {
        return if is_ruby { Some(space / 2) } else { None };
    }

    let mut inset = LayoutUnit::default();
    if is_ruby {
        let count = spacing
            .ExpansionOppotunityCount()
            .min(LayoutUnit::Max().Floor() as u32);
        inset = space / (count as i32 + 1);
        if target == JustificationTarget::kRubyText {
            inset = inset.min(LayoutUnit::from_signed(
                2 * line_info.LineStyle().FontSize(),
            ));
        }
        let mut setup =
            ExpansionSetup::new((space - inset).To::<16, i64>(), &mut spacing, false, false);
        SetupJustificationOpportunity(
            line_info.Results(),
            end_offset,
            line_info.BaseDirection(),
            &mut setup,
        );
    }

    if !results.is_null() {
        debug_assert_eq!(line_info.Results() as *const _, results as *const _);
        JustifyResults(&mut spacing, unsafe { &mut *results });
    }
    Some(inset / 2)
}

// cpp: layoutng_inline/justification_utils.h:23-28
// cpp: layoutng_inline/justification_utils.cc:315-320
pub fn ApplyJustification(
    space: LayoutUnit,
    target: JustificationTarget,
    line_info: &mut LineInfo,
) -> Option<LayoutUnit> {
    let results = line_info.MutableResults() as *mut InlineItemResults;
    ApplyJustificationInternal(space, target, line_info, results)
}

// cpp: layoutng_inline/justification_utils.h:30-33
// cpp: layoutng_inline/justification_utils.cc:322-327
pub fn ComputeRubyBaseInset(space: LayoutUnit, line_info: &LineInfo) -> Option<LayoutUnit> {
    debug_assert!(line_info.IsRubyBase());
    ApplyJustificationInternal(
        space,
        JustificationTarget::kRubyBase,
        line_info,
        std::ptr::null_mut(),
    )
}

// cpp: layoutng_inline/justification_utils.cc:179-217
struct ExpandableItemsFinder {
    first_item_: *mut LogicalLineItem,
    last_item_: *mut LogicalLineItem,
    first_placeholder_item_: *mut LogicalLineItem,
    last_placeholder_item_: *mut LogicalLineItem,
}

impl Default for ExpandableItemsFinder {
    fn default() -> Self {
        Self {
            first_item_: std::ptr::null_mut(),
            last_item_: std::ptr::null_mut(),
            first_placeholder_item_: std::ptr::null_mut(),
            last_placeholder_item_: std::ptr::null_mut(),
        }
    }
}

impl ExpandableItemsFinder {
    fn Find(&mut self, items: &mut [LogicalLineItem]) {
        for item in items {
            if (!item.shape_result.Get().is_null()
                && unsafe { &*item.shape_result.Get() }.NumGlyphs() > 0)
                || !item.layout_result.Get().is_null()
            {
                self.last_item_ = item;
                if self.first_item_.is_null() {
                    self.first_item_ = item;
                }
            } else if item.IsRubyLinePlaceholder() {
                self.last_placeholder_item_ = item;
                if self.first_placeholder_item_.is_null() {
                    self.first_placeholder_item_ = item;
                }
            }
        }
    }

    fn FirstExpandable(&self) -> *mut LogicalLineItem {
        if self.first_item_.is_null() {
            self.first_placeholder_item_
        } else {
            self.first_item_
        }
    }

    fn LastExpandable(&self) -> *mut LogicalLineItem {
        if self.last_item_.is_null() {
            self.last_placeholder_item_
        } else {
            self.last_item_
        }
    }
}

// cpp: layoutng_inline/justification_utils.cc:219-236
fn ApplyExpansionToItem(
    left_expansion: LayoutUnit,
    right_expansion: LayoutUnit,
    item: &mut LogicalLineItem,
) {
    let shape_view = item.shape_result.Get();
    if !shape_view.is_null() {
        let shape = unsafe { &*shape_view }.CreateShapeResult();
        unsafe { &mut *shape }.ApplyLeadingExpansion(left_expansion);
        unsafe { &mut *shape }.ApplyTrailingExpansion(right_expansion);
        item.inline_size += left_expansion + right_expansion;
        item.shape_result = foundation::Member::from_ptr(ShapeResultView::CreateFromResult(shape));
    } else if !item.layout_result.Get().is_null() {
        item.inline_size += left_expansion + right_expansion;
        item.rect.offset.inline_offset += left_expansion;
    } else {
        debug_assert!(item.IsRubyLinePlaceholder());
        item.inline_size += left_expansion + right_expansion;
        item.margin_line_left += left_expansion;
    }
}

// cpp: layoutng_inline/justification_utils.h:35-38
// cpp: layoutng_inline/justification_utils.cc:329-345
pub fn ApplyLeftAndRightExpansion(
    left_expansion: LayoutUnit,
    right_expansion: LayoutUnit,
    items: &mut [LogicalLineItem],
) -> bool {
    if left_expansion == LayoutUnit::default() && right_expansion == LayoutUnit::default() {
        return true;
    }
    let mut finder = ExpandableItemsFinder::default();
    finder.Find(items);
    let first = finder.FirstExpandable();
    let last = finder.LastExpandable();
    if !first.is_null() && !last.is_null() {
        ApplyExpansionToItem(left_expansion, LayoutUnit::default(), unsafe {
            &mut *first
        });
        ApplyExpansionToItem(LayoutUnit::default(), right_expansion, unsafe {
            &mut *last
        });
        return true;
    }
    false
}
