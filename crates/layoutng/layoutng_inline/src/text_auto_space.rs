// C++: layoutng_inline/text_auto_space.h/.cc.
#![allow(non_snake_case)]

use std::ptr::NonNull;

use font_engine::fonts::shaping::shape_result::OffsetWithSpacing;
use font_engine::text::native::character::Character;
use font_engine::FontOrientation;
use foundation::{
    ETextAutospace, EastAsianSpacingType, IsLtr, Member, RuntimeEnabledFeatures, String,
};
use layoutng::internal::inline_item::{InlineItem, InlineItemType};
use layoutng::internal::inline_node::InlineNode;
use layoutng_fragment_tree::inline_items_data::InlineItemsData;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_inline/text_auto_space.cc:23-27
fn is_macrolanguage_chinese(node: &InlineNode) -> bool {
    let locale = node.Style().GetFontDescription().Locale();
    !locale.is_null() && unsafe { &*locale }.IsMacrolanguageChinese()
}

// cpp: layoutng_inline/text_auto_space.cc:29-38
fn resolve_conditional(value: EastAsianSpacingType, is_chinese: bool) -> EastAsianSpacingType {
    if value != EastAsianSpacingType::kConditional {
        return value;
    }
    if is_chinese {
        EastAsianSpacingType::kNarrow
    } else {
        EastAsianSpacingType::kOther
    }
}

// cpp: layoutng_inline/text_auto_space.cc:40-76
const RUBY_ANNOTATION_MASK_CHARACTER: u16 = 0x0300;

fn mask_ruby_annotations(text: &String, items: &[Member<InlineItem>]) -> String {
    let mut buffer = text.Span16().expect("non-null UTF-16 text").to_vec();
    let mut ruby_text_nesting_level = 0u32;
    for item_ptr in items {
        let item = unsafe { &*item_ptr.Get() };
        let is_ruby_annotation = ruby_text_nesting_level != 0
            || matches!(
                item.Type(),
                InlineItemType::kOpenRubyColumn
                    | InlineItemType::kCloseRubyColumn
                    | InlineItemType::kRubyLinePlaceholder
            );
        if is_ruby_annotation {
            for offset in item.StartOffset()..item.EndOffset() {
                buffer[offset as usize] = RUBY_ANNOTATION_MASK_CHARACTER;
            }
        }
        if matches!(
            item.Type(),
            InlineItemType::kOpenTag | InlineItemType::kCloseTag
        ) {
            let object = item.GetLayoutObject();
            if !object.is_null() && unsafe { &*object }.IsInlineRubyText() {
                if item.Type() == InlineItemType::kOpenTag {
                    ruby_text_nesting_level += 1;
                } else {
                    debug_assert!(ruby_text_nesting_level > 0);
                    ruby_text_nesting_level -= 1;
                }
            }
        }
    }
    String::from_utf16(&buffer)
}

// cpp: layoutng_inline/text_auto_space.h:28-32
pub trait TextAutoSpaceCallback {
    fn DidApply(&mut self, offsets: &[OffsetWithSpacing]);
}

// cpp: layoutng_inline/text_auto_space.h:23-58
pub struct TextAutoSpace<'callback> {
    may_apply_: bool,
    callback_for_testing_: Option<NonNull<dyn TextAutoSpaceCallback + 'callback>>,
}

impl<'callback> TextAutoSpace<'callback> {
    // cpp: layoutng_inline/text_auto_space.h:60-75
    pub fn new(data: &InlineItemsData) -> Self {
        let units = data.text_content.Span16().unwrap_or_default();
        let is_latin1 = units.iter().all(|unit| *unit <= 0xff);
        let has_autospace_text = data.items.iter().any(|item_ptr| {
            let item = unsafe { &*item_ptr.Get() };
            item.Type() == InlineItemType::kText
                && unsafe { &*item.Style() }.TextAutospace() != ETextAutospace::kNoAutospace
        });
        let has_candidate = units
            .iter()
            .any(|unit| Character::MayNeedEastAsianSpacing(i32::from(*unit)));
        Self {
            may_apply_: !is_latin1 && has_autospace_text && has_candidate,
            callback_for_testing_: None,
        }
    }

    // cpp: layoutng_inline/text_auto_space.h:36-38
    pub fn MayApply(&self) -> bool {
        self.may_apply_
    }

    // cpp: layoutng_inline/text_auto_space.h:45-49
    pub fn ApplyIfNeeded(&mut self, node: &InlineNode, data: &mut InlineItemsData) {
        if self.MayApply() {
            self.Apply(node, data);
        }
    }

    // cpp: layoutng_inline/text_auto_space.h:51-53
    pub fn SetCallbackForTesting(
        &mut self,
        callback: Option<&'callback mut dyn TextAutoSpaceCallback>,
    ) {
        self.callback_for_testing_ = callback.map(NonNull::from);
    }
}

// cpp: layoutng_inline/text_auto_space.cc:78-239
struct SpacingApplier<'items, 'callback> {
    item_end_offset_: u32,
    is_disabled_: bool,
    is_disabled_by_style_: bool,
    is_last_disabled_: bool,
    ignore_ruby_annotation_: bool,
    item_: *mut InlineItem,
    style_: *const ComputedStyle,
    items_: &'items [Member<InlineItem>],
    item_index_: usize,
    offsets_with_spacing_: Vec<OffsetWithSpacing>,
    callback_for_testing_: Option<NonNull<dyn TextAutoSpaceCallback + 'callback>>,
}

impl<'items, 'callback> SpacingApplier<'items, 'callback> {
    // cpp: layoutng_inline/text_auto_space.cc:95-122
    fn new(
        offset: u32,
        items: &'items [Member<InlineItem>],
        callback: Option<NonNull<dyn TextAutoSpaceCallback + 'callback>>,
        ignore_ruby_annotation: bool,
    ) -> Self {
        let mut result = Self {
            item_end_offset_: 0,
            is_disabled_: false,
            is_disabled_by_style_: false,
            is_last_disabled_: false,
            ignore_ruby_annotation_: ignore_ruby_annotation,
            item_: items[0].Get(),
            style_: std::ptr::null(),
            items_: items,
            item_index_: 0,
            offsets_with_spacing_: Vec::new(),
            callback_for_testing_: callback,
        };
        result.DidChangeItem();
        if result.ignore_ruby_annotation_ {
            while (offset > result.item_end_offset_
                || unsafe { &*result.item_ }.Length() == 0
                || (offset == result.item_end_offset_
                    && unsafe { &*result.item_ }.Type() != InlineItemType::kText))
                && result.item_index_ + 1 != result.items_.len()
            {
                result.AdvanceItem();
            }
        } else {
            while offset > result.item_end_offset_ {
                result.AdvanceItem();
            }
        }
        if !result.is_disabled_ && !result.IsOffsetDisabled(offset) {
            result.InsertSpaceBefore(offset);
        }
        result
    }

    // cpp: layoutng_inline/text_auto_space.cc:124-128
    fn ItemEndOffset(&self) -> u32 {
        self.item_end_offset_
    }
    fn IsDisabled(&self) -> bool {
        self.is_disabled_
    }
    fn IsOffsetDisabled(&self, offset: u32) -> bool {
        self.is_last_disabled_ && offset == unsafe { &*self.item_ }.StartOffset()
    }

    // cpp: layoutng_inline/text_auto_space.cc:130-162
    fn InsertSpaceBefore(&mut self, offset: u32) {
        debug_assert!(!self.item_.is_null());
        debug_assert!(offset >= unsafe { &*self.item_ }.StartOffset());
        debug_assert!(!(self.is_disabled_ && offset < self.item_end_offset_));
        debug_assert!(!self.IsOffsetDisabled(offset));
        if offset < self.item_end_offset_ {
            self.offsets_with_spacing_.push(OffsetWithSpacing {
                offset,
                spacing: 0.0,
            });
            return;
        }

        let is_offset_for_last_item =
            offset == self.item_end_offset_ && IsLtr(unsafe { &*self.item_ }.Direction());
        let last_item = self.item_;
        let last_style = self.style_;
        while offset >= self.item_end_offset_ {
            self.AdvanceItem();
        }
        if self.is_disabled_ || self.IsOffsetDisabled(offset) {
            self.ApplyIfNeededFor(last_style, last_item);
        } else if is_offset_for_last_item {
            self.offsets_with_spacing_.push(OffsetWithSpacing {
                offset,
                spacing: 0.0,
            });
            self.Apply(unsafe { &*last_style }, unsafe { &mut *last_item });
        } else {
            self.ApplyIfNeededFor(last_style, last_item);
            self.offsets_with_spacing_.push(OffsetWithSpacing {
                offset,
                spacing: 0.0,
            });
        }
    }

    // cpp: layoutng_inline/text_auto_space.cc:164-170
    fn ApplyIfNeeded(&mut self) {
        self.ApplyIfNeededFor(self.style_, self.item_);
    }
    fn ApplyIfNeededFor(&mut self, style: *const ComputedStyle, item: *mut InlineItem) {
        if !self.offsets_with_spacing_.is_empty() {
            self.Apply(unsafe { &*style }, unsafe { &mut *item });
        }
    }

    // cpp: layoutng_inline/text_auto_space.cc:172-186
    fn Apply(&mut self, style: &ComputedStyle, item: &mut InlineItem) {
        let spacing = unsafe { &*style.GetFont() }.TextAutoSpaceInlineSize();
        for offset_with_spacing in &mut self.offsets_with_spacing_ {
            offset_with_spacing.spacing = spacing;
        }
        let shape_result = item.CloneTextShapeResult();
        debug_assert!(!shape_result.is_null());
        unsafe { &mut *shape_result }.ApplyTextAutoSpacing(&self.offsets_with_spacing_);
        item.SetUnsafeToReuseShapeResult();
        if let Some(mut callback) = self.callback_for_testing_ {
            unsafe { callback.as_mut() }.DidApply(&self.offsets_with_spacing_);
        }
        self.offsets_with_spacing_.clear();
    }

    // cpp: layoutng_inline/text_auto_space.cc:189-193
    fn AdvanceItem(&mut self) {
        self.is_last_disabled_ = self.is_disabled_;
        self.item_index_ += 1;
        self.item_ = self.items_[self.item_index_].Get();
        self.DidChangeItem();
    }

    // cpp: layoutng_inline/text_auto_space.cc:195-226
    fn DidChangeItem(&mut self) {
        let item = unsafe { &*self.item_ };
        self.item_end_offset_ = item.EndOffset();
        if self.ignore_ruby_annotation_
            && matches!(
                item.Type(),
                InlineItemType::kOpenRubyColumn
                    | InlineItemType::kCloseRubyColumn
                    | InlineItemType::kRubyLinePlaceholder
            )
        {
            self.is_disabled_ = false;
            return;
        }
        if item.Length() == 0 {
            if self.ignore_ruby_annotation_ {
                self.is_disabled_ = false;
            }
            return;
        }
        if item.GetLayoutObject().is_null() {
            self.is_disabled_ = true;
            return;
        }
        let style = item.Style();
        debug_assert!(!style.is_null());
        if style != self.style_ {
            self.style_ = style;
            let style = unsafe { &*style };
            self.is_disabled_by_style_ = style.TextAutospace() != ETextAutospace::kNormal
                || style.GetFontDescription().Orientation() == FontOrientation::kVerticalUpright;
        }
        self.is_disabled_ = self.is_disabled_by_style_ || item.TextShapeResult().is_null();
    }
}

impl<'callback> TextAutoSpace<'callback> {
    // cpp: layoutng_inline/text_auto_space.cc:243-312
    pub fn Apply(&mut self, node: &InlineNode, data: &mut InlineItemsData) {
        let text = &data.text_content;
        debug_assert!(!text
            .Span16()
            .unwrap_or_default()
            .iter()
            .all(|unit| *unit <= 0xff));
        debug_assert_eq!(
            text.length(),
            unsafe { &*data.items.last().expect("inline items are nonempty").Get() }.EndOffset()
        );
        debug_assert!(self.MayApply());

        let mut last_type = EastAsianSpacingType::kOther;
        let mut is_last_wide = false;
        let mut applier: Option<SpacingApplier<'_, 'callback>> = None;
        let mut is_chinese: Option<bool> = None;

        // MutableData is used while ShapeText is preparing the node, before
        // InlineNode::Data is valid, as in the source implementation.
        let ignore_ruby_annotation =
            RuntimeEnabledFeatures::TextAutoSpaceIgnoreRubyAnnotationEnabled()
                && unsafe { &*node.MutableData() }.HasRuby();
        let masked_text;
        let iter_text = if ignore_ruby_annotation {
            masked_text = mask_ruby_annotations(text, &data.items);
            &masked_text
        } else {
            text
        };
        let units = iter_text.Span16().expect("non-null UTF-16 text");
        let mut offset = 0usize;
        while offset < units.len() {
            let first = units[offset];
            let (ch, code_units): (i32, usize) = if (0xd800..=0xdbff).contains(&first)
                && offset + 1 < units.len()
                && (0xdc00..=0xdfff).contains(&units[offset + 1])
            {
                (
                    (0x10000
                        + ((u32::from(first) - 0xd800) << 10)
                        + (u32::from(units[offset + 1]) - 0xdc00)) as i32,
                    2,
                )
            } else if (0xd800..=0xdfff).contains(&first) {
                // U16_NEXT reports an unpaired surrogate as U_SENTINEL.
                (-1, 1)
            } else {
                (i32::from(first), 1)
            };
            if Character::IsGcMark(ch) {
                offset += code_units;
                continue;
            }
            let mut spacing_type = Character::GetEastAsianSpacingType(ch);
            let is_wide = spacing_type == EastAsianSpacingType::kWide;
            if is_wide || is_last_wide {
                let chinese = *is_chinese.get_or_insert_with(|| is_macrolanguage_chinese(node));
                spacing_type = resolve_conditional(spacing_type, chinese);
                last_type = resolve_conditional(last_type, chinese);

                let needs_space = (is_last_wide && spacing_type == EastAsianSpacingType::kNarrow)
                    || (is_wide && last_type == EastAsianSpacingType::kNarrow);
                if needs_space {
                    if let Some(ref mut applier) = applier {
                        applier.InsertSpaceBefore(offset as u32);
                    } else {
                        applier = Some(SpacingApplier::new(
                            offset as u32,
                            &data.items,
                            self.callback_for_testing_,
                            ignore_ruby_annotation,
                        ));
                    }

                    let applier_ref = applier.as_ref().expect("spacing applier exists");
                    if applier_ref.IsDisabled() {
                        let item_end_offset = applier_ref.ItemEndOffset() as usize;
                        debug_assert!(item_end_offset >= offset);
                        offset = item_end_offset;
                        last_type = EastAsianSpacingType::kOther;
                        is_last_wide = false;
                        continue;
                    }
                }
            }
            last_type = spacing_type;
            is_last_wide = is_wide;
            offset += code_units;
        }

        if let Some(ref mut applier) = applier {
            applier.ApplyIfNeeded();
        }
    }
}
