// C++: layoutng_inline/inline_node.cc. Data access, collection, incremental
// editing, segmentation, shaping, layout dispatch, and intrinsic sizing are
// source-mapped. Shared assembly and external font interfaces remain pending.
#![allow(non_snake_case)]

use font_engine::fonts::font_description::FontDescription;
use font_engine::fonts::font_width_variant::FontWidthVariant;
use font_engine::fonts::shaping::ng_shape_cache::{ShapeCacheKey, ShaperResult};
use font_engine::fonts::shaping::shape_options::ShapeOptions;
use font_engine::fonts::shaping::shape_result::ShapeRange;
use font_engine::fonts::shaping::text_spacing_trim::ShouldTrimStartOfParagraph;
use font_engine::text::native::bidi_paragraph::BidiParagraph;
use font_engine::text::native::character::Character;
use font_engine::{
    Font, FontFallbackPriority, FontOrientation, HarfBuzzShaper, RenderOrientation, RunSegmenter,
    RunSegmenterRange, ShapeResult, ShapeResultSpacing,
};
use foundation::blink_geometry::geometry::{InlineLayoutUnit, TextRunLayoutUnit};
use foundation::style_constants::{EClear, EFloat};
use foundation::{
    g_null_atom, ClearCollectionScope, DynamicTo, EDisplay, ETextSecurity, HeapVector, IsLtr,
    LayoutUnit, MakeGarbageCollected, Member, RuntimeEnabledFeatures, String, TextDirection,
    TextOffsetMap, To, UnicodeBidi, WritingMode,
};
use layoutng::internal::algorithm_forward::InlineChildLayoutContext as ForwardInlineChildLayoutContext;
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::constraint_space_builder_style::MinMaxConstraintSpaceBuilder;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::exclusions::line_layout_opportunity::LineLayoutOpportunity;
use layoutng::internal::inline_item::{InlineItem, InlineItemType, InlineItems};
use layoutng::internal::inline_item_segment::{InlineItemSegment, InlineItemSegments};
use layoutng::internal::inline_node::{FloatingObject, InlineNode};
use layoutng::internal::inline_node_data::InlineNodeData;
use layoutng::internal::layout_algorithm_set::ListLayoutSupport;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_input::TextDirection as InputTextDirection;
use layoutng::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_pass_scope::LayoutPassScope;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng::internal::length_utils::{ComputeMarginsFor, ComputeMinAndMaxContentContribution};
use layoutng::internal::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::svg_character_data::SvgCharacterData;
use layoutng::internal::svg_inline_node_data::{SvgTextChunkOffsets, SvgTextContentRange};
use layoutng::internal::svg_text_attributes_request::{
    SvgTextAttributesBuildRequest, SvgTextAttributesBuildResult,
};
use layoutng::internal::text_item_type::TextItemType;
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::inline_items_data::{InlineItemsData, InlineItemsDataWithOffsetMap};
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::EVerticalAlign;
use std::cell::Cell;

// These source-owned modules are translated later in this same Bazel package.
use crate::initial_letter_utils::CalculateInitialLetterBoxInlineSize;
use crate::inline_child_layout_context::InlineChildLayoutContext;
use crate::inline_item_result_ruby_column::InlineItemResultRubyColumn;
use crate::inline_items_builder::{
    InlineItemsBuilder, InlineItemsBuilderForOffsetMapping, InlineItemsBuilderTemplate,
    InlineItemsMappingMode,
};
use crate::inline_layout_algorithm::InlineLayoutAlgorithm;
use crate::layout_counter::LayoutCounter;
use crate::leading_floats::LeadingFloats;
use crate::line_breaker::{LineBreaker, LineBreakerMode, MaxSizeCache};
use crate::line_info::LineInfo;
use crate::offset_mapping::OffsetMapping;
use crate::text_auto_space::TextAutoSpace;
use crate::text_diff_range::TextDiffRange;

unsafe extern "Rust" {
    // cpp: foundation/blink_base/wtf/text/wtf_string.h:321-321
    // StringImpl storage width affects incremental shape reuse context.
    fn StringIs8BitForInline(text: &String) -> bool;
    // cpp: font_engine/fonts/font_description.h:423-425
    fn FontDescriptionSetWidthVariantForInline(
        description: &mut FontDescription,
        variant: FontWidthVariant,
    );
}

// cpp: layoutng_inline/inline_node.cc:138-150
fn list_support() -> *const ListLayoutSupport {
    let algorithms = LayoutPassScope::Algorithms();
    if algorithms.is_null()
        || unsafe { &*algorithms }
            .list_support
            .update_marker_text
            .is_none()
        || unsafe { &*algorithms }
            .list_support
            .symbol_marker_text
            .is_none()
        || unsafe { &*algorithms }.list_support.symbol_width.is_none()
    {
        panic!("list layout module is not installed");
    }
    &unsafe { &*algorithms }.list_support
}

// cpp: layoutng_inline/inline_node.cc:226-398
struct ReusingTextShaper<'a> {
    data: *const InlineItemsData,
    reusable_items: Option<&'a InlineItems>,
    shaper: HarfBuzzShaper,
    options: Cell<ShapeOptions>,
    allow_shape_cache: bool,
}

impl<'a> ReusingTextShaper<'a> {
    // cpp: layoutng_inline/inline_node.cc:232-238
    fn new(
        data: &InlineItemsData,
        reusable_items: Option<&'a InlineItems>,
        allow_shape_cache: bool,
    ) -> Self {
        Self {
            data: data as *const InlineItemsData,
            reusable_items,
            shaper: HarfBuzzShaper::new(data.text_content.clone()),
            options: Cell::new(ShapeOptions::default()),
            allow_shape_cache,
        }
    }

    // cpp: layoutng_inline/inline_node.cc:240-240
    fn set_options(&self, options: ShapeOptions) {
        self.options.set(options);
    }

    // cpp: layoutng_inline/inline_node.cc:242-257
    fn shape(&self, start_item: &InlineItem, font: &Font, end_offset: u32) -> *const ShapeResult {
        let shape_func = || self.shape_without_cache(start_item, font, end_offset);
        if self.allow_shape_cache {
            let locale = font.GetFontDescription().Locale();
            let locale_string = if locale.is_null() {
                &*g_null_atom
            } else {
                unsafe { &*locale }.LocaleStringValue()
            };
            let key = ShapeCacheKey::new(
                self.shaper.GetText(),
                start_item.StartOffset(),
                end_offset,
                locale_string,
                font.GetFontFeatures(),
                start_item.Direction(),
            );
            let primary_font = font.PrimaryFont();
            let cache = unsafe { &mut *(&*primary_font).GetShapeCachePtrForInline() };
            return cache.GetOrCreate(&key, &shape_func);
        }
        shape_func().shape_result
    }

    // cpp: layoutng_inline/inline_node.cc:260-323
    fn shape_without_cache(
        &self,
        start_item: &InlineItem,
        font: &Font,
        end_offset: u32,
    ) -> ShaperResult {
        let start_offset = start_item.StartOffset();
        debug_assert!(start_offset < end_offset);
        if self.reusable_items.is_none() {
            return ShaperResult {
                shape_result: self.reshape(start_item, font, start_offset, end_offset),
                can_cache: true,
            };
        }
        if !unsafe { &*self.data }.segments.Get().is_null() {
            return ShaperResult {
                shape_result: self.reshape(start_item, font, start_offset, end_offset),
                can_cache: true,
            };
        }

        let mut reusable_shape_results = self.collect_reusable_shape_results(
            start_offset,
            end_offset,
            font,
            start_item.Direction(),
        );
        let _clear_scope = unsafe { ClearCollectionScope::new(&mut reusable_shape_results) };
        if reusable_shape_results.is_empty() {
            return ShaperResult {
                shape_result: self.reshape(start_item, font, start_offset, end_offset),
                can_cache: true,
            };
        }

        let shape_result = ShapeResult::CreateEmpty(unsafe { &*reusable_shape_results[0].Get() });
        let mut offset = start_offset;
        for reusable_ptr in &reusable_shape_results {
            let reusable = unsafe { &*reusable_ptr.Get() };
            if offset < reusable.StartIndex() {
                let gap = self.reshape(start_item, font, offset, reusable.StartIndex());
                self.append_shape_result(unsafe { &*gap }, unsafe { &mut *shape_result });
                offset = unsafe { &*shape_result }.EndIndex();
                let mut options = self.options.get();
                options.han_kerning_start = false;
                self.options.set(options);
            }
            debug_assert!(offset < reusable.EndIndex());
            debug_assert!(
                unsafe { &*shape_result }.NumCharacters() == 0
                    || unsafe { &*shape_result }.EndIndex() == offset
            );
            reusable.CopyRange(offset, reusable.EndIndex().min(end_offset), unsafe {
                &mut *shape_result
            });
            offset = unsafe { &*shape_result }.EndIndex();
            if offset == end_offset {
                break;
            }
        }
        if offset < end_offset {
            let tail = self.reshape(start_item, font, offset, end_offset);
            self.append_shape_result(unsafe { &*tail }, unsafe { &mut *shape_result });
        }
        ShaperResult {
            shape_result,
            can_cache: false,
        }
    }

    // cpp: layoutng_inline/inline_node.cc:325-330
    fn append_shape_result(&self, source: &ShapeResult, target: &mut ShapeResult) {
        debug_assert!(target.NumCharacters() == 0 || target.EndIndex() == source.StartIndex());
        source.CopyRange(source.StartIndex(), source.EndIndex(), target);
    }

    // cpp: layoutng_inline/inline_node.cc:332-373
    fn collect_reusable_shape_results(
        &self,
        start_offset: u32,
        end_offset: u32,
        font: &Font,
        direction: TextDirection,
    ) -> HeapVector<Member<ShapeResult>> {
        debug_assert!(start_offset < end_offset);
        let mut shape_results = HeapVector::new();
        let Some(reusable_items) = self.reusable_items else {
            return shape_results;
        };
        let start_item_index = reusable_items
            .as_slice()
            .partition_point(|item| unsafe { &*item.Get() }.EndOffset() <= start_offset);
        for item_ptr in &reusable_items[start_item_index..] {
            let item = unsafe { &*item_ptr.Get() };
            if end_offset <= item.StartOffset() {
                break;
            }
            if item.EndOffset() < start_offset {
                continue;
            }
            let shape_result = item.TextShapeResult();
            if shape_result.is_null() || item.Direction() != direction {
                continue;
            }
            if unsafe { &*(*item.Style()).GetFont() } != font {
                continue;
            }
            if item.IsUnsafeToReuseShapeResult() || unsafe { &*shape_result }.IsAppliedSpacing() {
                continue;
            }
            shape_results.push(Member::from_ptr(shape_result.cast_mut()));
        }
        shape_results
    }

    // cpp: layoutng_inline/inline_node.cc:375-391
    fn reshape(
        &self,
        start_item: &InlineItem,
        font: &Font,
        start_offset: u32,
        end_offset: u32,
    ) -> *const ShapeResult {
        debug_assert!(start_offset < end_offset);
        let direction = start_item.Direction();
        let segments = unsafe { &*self.data }.segments.Get();
        if !segments.is_null() {
            return unsafe { &*segments }.ShapeText(
                &self.shaper,
                font,
                direction,
                start_offset,
                end_offset,
                start_item.Index(),
                self.options.get(),
            );
        }
        let mut range = start_item.CreateRunSegmenterRange();
        range.end = end_offset;
        self.shaper.ShapeWithRanges(
            font,
            direction,
            start_offset,
            end_offset,
            &[range],
            self.options.get(),
        )
    }
}

// cpp: layoutng_inline/inline_node.cc:190-205
fn calculate_width_for_text_combine(data: &InlineItemsData) -> f32 {
    data.items.iter().fold(0.0, |sum, item_ptr| {
        let item = unsafe { &*item_ptr.Get() };
        debug_assert!(matches!(
            item.Type(),
            InlineItemType::kText | InlineItemType::kBidiControl | InlineItemType::kControl
        ));
        let shape_result = item.TextShapeResult();
        if !shape_result.is_null() {
            unsafe { &*shape_result }.Width() + sum
        } else {
            0.0
        }
    })
}

// cpp: layoutng_inline/inline_node.cc:207-215
fn estimate_inline_items_count(block: &LayoutBlockFlow) -> u32 {
    let mut count = 0u32;
    let mut child = block.FirstChild();
    while !child.is_null() {
        count = count.wrapping_add(1);
        child = unsafe { &*child }.NextSibling();
    }
    count.wrapping_mul(4)
}

// cpp: layoutng_inline/inline_node.cc:217-224
fn estimate_offset_mapping_items_count(block: &LayoutBlockFlow) -> u32 {
    estimate_inline_items_count(block) / 4
}

// cpp: layoutng_inline/inline_node.cc:400-404
fn scaled_font(layout_text: &LayoutText) -> &Font {
    if layout_text.IsSVGInlineText() {
        return layout_text.ScaledFont();
    }
    unsafe { &*layout_text.StyleRef().GetFont() }
}

// cpp: layoutng_inline/inline_node.cc:406-547
// Both C++ template instantiations use this traversal; MappingMode retains
// their distinct mutation and offset-mapping behavior.
fn collect_inlines_internal<MappingMode: InlineItemsMappingMode>(
    builder: &mut InlineItemsBuilderTemplate<MappingMode>,
    previous_data: *const InlineNodeData,
) {
    let block = builder.GetLayoutBlockFlow();
    builder.EnterBlock(unsafe { &*block }.StyleRef());
    let mut node = unsafe { &*block }.FirstChild();

    let symbol =
        if unsafe { &*block }.IsListItem() || unsafe { &*block }.IsLayoutOutsideListMarker() {
            unsafe { &*list_support() }.symbol_marker_text.unwrap()(block.cast())
        } else {
            std::ptr::null()
        };
    let mut inline_list_item_marker: *const LayoutObject = std::ptr::null();
    while !node.is_null() {
        let counter = DynamicTo::<LayoutCounter>(node);
        let layout_text = DynamicTo::<LayoutText>(node);
        if !counter.is_null() {
            let counter_ref = unsafe { &*counter };
            let font = counter_ref.StyleRef().GetFont();
            if !unsafe { &*font }.PrimaryFont().is_null() {
                if counter_ref.IsDirectionalSymbolMarker() {
                    let text = counter_ref.TransformedText();
                    let units = text.Span16().unwrap_or_default();
                    if units.len() <= 1 {
                        builder.AppendText(counter.cast(), previous_data);
                        builder.SetIsSymbolMarker();
                    } else {
                        builder.AppendTextString(&String::from_utf16(&units[..1]), counter.cast());
                        builder.SetIsSymbolMarker();
                        let separator = counter_ref.Separator();
                        let separator_units = separator.Span16();
                        let mut i = 1usize;
                        while i < units.len() {
                            if !separator_units.is_empty() {
                                debug_assert_eq!(
                                    separator_units,
                                    &units[i..i + separator_units.len()]
                                );
                                builder.AppendTextString(
                                    &String::from_utf16(separator_units),
                                    counter.cast(),
                                );
                                i += separator_units.len();
                                debug_assert!(i < units.len());
                            }
                            builder.AppendTextString(
                                &String::from_utf16(&units[i..i + 1]),
                                counter.cast(),
                            );
                            builder.SetIsSymbolMarker();
                            i += 1;
                        }
                    }
                } else {
                    builder.AppendText(counter.cast(), previous_data);
                }
            }
            builder.ClearNeedsLayout(node);
        } else if !layout_text.is_null() {
            if !scaled_font(unsafe { &*layout_text })
                .PrimaryFont()
                .is_null()
            {
                builder.AppendText(layout_text, previous_data);
                if symbol == node || inline_list_item_marker == node {
                    builder.SetIsSymbolMarker();
                }
            }
            builder.ClearNeedsLayout(node);
        } else if unsafe { &*node }.IsFloating() {
            builder.AppendFloating(node);
            if builder.ShouldAbort() {
                return;
            }
            builder.ClearInlineFragment(node);
        } else if unsafe { &*node }.IsOutOfFlowPositioned() {
            builder.AppendOutOfFlowPositioned(node);
            if builder.ShouldAbort() {
                return;
            }
            builder.ClearInlineFragment(node);
        } else if unsafe { &*node }.IsAtomicInline() {
            if unsafe { &*node }.IsLayoutOutsideListMarker() {
                builder.AppendOpaqueItem(InlineItemType::kListMarker, node);
            } else if unsafe { &*node }.IsInitialLetterBox() {
                builder.AppendOpaqueCharacter(InlineItemType::kInitialLetterBox, 0xfffc, node);
                builder.SetHasInititialLetterBox();
            } else {
                builder.AppendAtomicInline(node);
            }
            builder.ClearInlineFragment(node);
        } else {
            let layout_inline = DynamicTo::<LayoutInline>(node);
            if !layout_inline.is_null() {
                if unsafe { &*node }.IsInlineListItem() {
                    let support = unsafe { &*list_support() };
                    support.update_marker_text.unwrap()(unsafe { &mut *node });
                    inline_list_item_marker = support.symbol_marker_text.unwrap()(node);
                }
                builder.UpdateShouldCreateBoxFragment(layout_inline);
                builder.EnterInline(layout_inline);
                let child = unsafe { &*layout_inline }.FirstChild();
                if !child.is_null() {
                    node = child;
                    continue;
                }
                builder.ExitInline(node);
                builder.ClearNeedsLayout(node);
            } else {
                debug_assert!(!unsafe { &*node }.IsInline());
                builder.AppendBlockInInline(node);
                builder.ClearInlineFragment(node);
            }
        }

        loop {
            let next = unsafe { &*node }.NextSibling();
            if !next.is_null() {
                node = next;
                break;
            }
            node = unsafe { &*node }.Parent();
            if node == block.cast() || node.is_null() {
                node = std::ptr::null_mut();
                break;
            }
            debug_assert!(unsafe { &*node }.IsInline());
            builder.ExitInline(node);
            builder.ClearNeedsLayout(node);
        }
    }
    builder.ExitBlock();
}

// cpp: layoutng_inline/inline_node.cc:1148-1186
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeCollectInlines(
    this: &InlineNode,
    data: *mut InlineNodeData,
    mut previous_data: *mut InlineNodeData,
) {
    let data_ref = unsafe { &mut *data };
    debug_assert!(data_ref.text_content.IsNull());
    debug_assert!(data_ref.items.is_empty());
    let block = this.GetLayoutBlockFlow();
    unsafe { &mut *block }.WillCollectInlines();

    let mut chunk_offsets: *const SvgTextChunkOffsets = std::ptr::null();
    if unsafe { &*block }.IsSVGText() {
        previous_data = std::ptr::null_mut();
        data_ref.svg_node_data_ = Member::default();
        let first_child = unsafe { &*block }.FirstChild();
        let layout_text = DynamicTo::<LayoutText>(first_child);
        let empty_or_one_char = first_child.is_null()
            || (!layout_text.is_null()
                && unsafe { &*layout_text }.NextSibling().is_null()
                && unsafe { &*layout_text }.TransformedTextLength() <= 1);
        if !empty_or_one_char {
            chunk_offsets = InlineNodeFindSvgTextChunks(this, unsafe { &mut *block }, data_ref);
        }
    }

    data_ref
        .items
        .reserve(estimate_inline_items_count(unsafe { &*block }) as usize);
    let previous_text = if previous_data.is_null() {
        String::default()
    } else {
        unsafe { &*previous_data }.text_content.clone()
    };
    let mut builder =
        InlineItemsBuilder::new(block, &mut data_ref.items, &previous_text, chunk_offsets);
    collect_inlines_internal(&mut builder, previous_data);
    if unsafe { &*block }.IsSVGText() && data_ref.svg_node_data_.Get().is_null() {
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.svg_support.build_text_attributes
        };
        let callback = callback.expect("SVG text module is not installed");
        let text = builder.ToString();
        let request = SvgTextAttributesBuildRequest::new(this, &text, &data_ref.items);
        let mut result = SvgTextAttributesBuildResult::default();
        callback(&request, &mut result);
        data_ref.svg_node_data_ = Member::from_ptr(result.data);
    }
    builder.DidFinishCollectInlines(data_ref);
}

// cpp: layoutng_inline/inline_node.cc:1189-1236
pub fn InlineNodeFindSvgTextChunks(
    this: &InlineNode,
    block: &mut LayoutBlockFlow,
    data: &mut InlineNodeData,
) -> *const SvgTextChunkOffsets {
    let mut items = layoutng::internal::inline_item::InlineItems::default();
    let _clear_scope = unsafe { ClearCollectionScope::new(&mut items) };
    items.reserve(estimate_inline_items_count(block) as usize);
    let mut items_builder = InlineItemsBuilderForOffsetMapping::new(
        block,
        &mut items,
        &String::default(),
        std::ptr::null(),
    );
    items_builder
        .GetOffsetMappingBuilder()
        .ReserveCapacity(estimate_offset_mapping_items_count(block));
    collect_inlines_internal(&mut items_builder, std::ptr::null());
    let ifc_text_content = items_builder.ToString();

    let algorithms = LayoutPassScope::Algorithms();
    let callback = if algorithms.is_null() {
        None
    } else {
        unsafe { &*algorithms }.svg_support.build_text_attributes
    };
    let callback = callback.expect("SVG text module is not installed");
    let request = SvgTextAttributesBuildRequest {
        node: this,
        text: &ifc_text_content,
        items: &items,
        include_ifc_offsets: true,
    };
    let mut result = SvgTextAttributesBuildResult::default();
    callback(&request, &mut result);
    data.svg_node_data_ = Member::from_ptr(result.data);

    let mapping_builder = items_builder.GetOffsetMappingBuilder();
    assert!(mapping_builder.SetDestinationString(&ifc_text_content));
    let mapping = mapping_builder.Build();
    let svg_data = unsafe { &mut *data.svg_node_data_.Get() };
    for (index, char_data) in svg_data.character_data_list.iter().enumerate() {
        if !char_data.1.anchored_chunk || char_data.0 == 0 {
            continue;
        }
        let text_content_offset = result.ifc_text_content_offsets[index];
        let unit = unsafe { &*mapping }
            .GetLastMappingUnit(text_content_offset)
            .expect("SVG text chunk offset must map to a DOM unit");
        let layout_text = To::<LayoutText>(unit.GetLayoutObject() as *const LayoutObject);
        svg_data
            .chunk_offsets
            .entry(Member::from_ptr(layout_text))
            .or_default()
            .push(unit.ConvertTextContentToFirstDOMOffset(text_content_offset));
    }
    if svg_data.chunk_offsets.is_empty() {
        std::ptr::null()
    } else {
        &svg_data.chunk_offsets
    }
}

// cpp: layoutng_inline/inline_node.cc:1078-1089
pub fn InlineNodeComputeOffsetMappingIfNeeded(this: &InlineNode) -> *const OffsetMapping {
    debug_assert!(unsafe { &*this.GetLayoutBlockFlow() }.HasPreparedLayoutInput());
    let data = this.MutableData();
    if unsafe { &*data }.offset_mapping.Get().is_null() {
        debug_assert!(!unsafe { &*data }.text_content.IsNull());
        InlineNodeComputeOffsetMapping(this.GetLayoutBlockFlow(), data);
    }
    unsafe { &*data }.offset_mapping.Get()
}

// cpp: layoutng_inline/inline_node.cc:1092-1125
pub fn InlineNodeComputeOffsetMapping(block: *mut LayoutBlockFlow, data: *mut InlineNodeData) {
    let block_ref = unsafe { &*block };
    let data_ref = unsafe { &mut *data };
    debug_assert!(data_ref.offset_mapping.Get().is_null());
    debug_assert!(block_ref.HasPreparedLayoutInput());
    let svg_data = data_ref.svg_node_data_.Get();
    let chunk_offsets = if !svg_data.is_null() && !unsafe { &*svg_data }.chunk_offsets.is_empty() {
        &unsafe { &*svg_data }.chunk_offsets as *const SvgTextChunkOffsets
    } else {
        std::ptr::null()
    };

    let mut items = layoutng::internal::inline_item::InlineItems::default();
    let _clear_scope = unsafe { ClearCollectionScope::new(&mut items) };
    items.reserve(estimate_inline_items_count(block_ref) as usize);
    let mut builder = InlineItemsBuilderForOffsetMapping::new(
        block,
        &mut items,
        &data_ref.text_content,
        chunk_offsets,
    );
    builder
        .GetOffsetMappingBuilder()
        .ReserveCapacity(estimate_offset_mapping_items_count(block_ref));
    collect_inlines_internal(&mut builder, std::ptr::null());
    debug_assert!(!data_ref.text_content.IsNull());

    let mapping_builder = builder.GetOffsetMappingBuilder();
    data_ref.offset_mapping = Member::default();
    if mapping_builder.SetDestinationString(&data_ref.text_content) {
        data_ref.offset_mapping = Member::from_ptr(mapping_builder.Build());
        debug_assert!(!data_ref.offset_mapping.Get().is_null());
    }
}

// cpp: layoutng_inline/inline_node.cc:1128-1145
pub fn InlineNodeGetOffsetMapping(block: *mut LayoutBlockFlow) -> *const OffsetMapping {
    debug_assert!(unsafe { &*block }.HasPreparedLayoutInput());
    if unsafe { &*block }.NeedsLayout() {
        return std::ptr::null();
    }
    let node = InlineNode::new(block);
    assert!(node.IsPrepareLayoutFinished());
    node.ComputeOffsetMappingIfNeeded()
}

// cpp: layoutng_inline/inline_node.cc:722-1023
// The source keeps this editor on the stack. Raw owner pointers retain the
// address of the text and its old data while SetTextWithOffset mutates them.
struct InlineNodeDataEditor {
    data_: *mut InlineNodeData,
    block_flow_: *mut LayoutBlockFlow,
    layout_text_: *mut LayoutText,
}

impl InlineNodeDataEditor {
    // cpp: layoutng_inline/inline_node.cc:726-729
    fn new(layout_text: *mut LayoutText) -> Self {
        debug_assert!(unsafe { &*layout_text }.HasValidInlineItems());
        Self {
            data_: std::ptr::null_mut(),
            block_flow_: unsafe { &*layout_text }.FragmentItemsContainer(),
            layout_text_: layout_text,
        }
    }

    // cpp: layoutng_inline/inline_node.cc:733-733
    fn GetLayoutBlockFlow(&self) -> *mut LayoutBlockFlow {
        self.block_flow_
    }

    // cpp: layoutng_inline/inline_node.cc:737-781
    fn Prepare(&mut self) -> *mut InlineNodeData {
        let block = self.block_flow_;
        if block.is_null()
            || unsafe { &*block }.NeedsCollectInlines()
            || unsafe { &*block }.NeedsLayout()
            || !unsafe { &*block }.HasPreparedLayoutInput()
            || unsafe { &*block }.GetInlineNodeData().is_null()
        {
            return std::ptr::null_mut();
        }
        let current = unsafe { &*unsafe { &*block }.GetInlineNodeData() };
        if current.text_content.IsNull() || current.items.is_empty() {
            return std::ptr::null_mut();
        }
        if unsafe { &*block }.IsLayoutTextCombine()
            || unsafe { &*self.layout_text_ }.StyleRef().TextSecurity() != ETextSecurity::kNone
            || unsafe { &*self.layout_text_ }.HasBidiControlInlineItems()
        {
            return std::ptr::null_mut();
        }
        let mapping = InlineNodeGetOffsetMapping(block);
        debug_assert!(!mapping.is_null());
        if !self.data_.is_null() {
            let data = unsafe { &mut *self.data_ };
            data.items.clear();
            data.text_content = String::default();
        }
        self.data_ = unsafe { &mut *block }.TakeInlineNodeData();
        self.data_
    }

    // cpp: layoutng_inline/inline_node.cc:783-883
    fn Run(&mut self) {
        let new_data = unsafe { &*unsafe { &*self.block_flow_ }.GetInlineNodeData() };
        let old_data = unsafe { &*self.data_ };
        let old_text = &old_data.text_content;
        let new_text = &new_data.text_content;
        let (start_offset, end_match_length) = self.MatchedLengths(old_text, new_text);
        let old_length = old_text.length();
        let new_length = new_text.length();
        debug_assert!(start_offset <= old_length - end_match_length);
        debug_assert!(start_offset <= new_length - end_match_length);
        let end_offset = old_length - end_match_length;
        debug_assert!(start_offset <= end_offset);
        let mut items = InlineItems::new();
        let _clear_scope = unsafe { ClearCollectionScope::new(&mut items) };
        let old_items = &old_data.items;
        items.reserve(old_items.len() + 3);
        let mut index = 0usize;
        while index < old_items.len()
            && unsafe { &*old_items[index].Get() }.EndOffset() < start_offset
        {
            items.push_back(old_items[index]);
            index += 1;
        }

        while index < old_items.len() {
            let old_item = unsafe { &*old_items[index].Get() };
            if old_item.StartOffset() < start_offset {
                let copied = self.CopyItemBefore(old_item, start_offset);
                items.push_back(Member::from_ptr(copied));
                let copied_end = unsafe { &*copied }.EndOffset();
                if copied_end < start_offset {
                    let gap = MakeGarbageCollected(old_item.clone_adjusted(
                        copied_end,
                        start_offset,
                        std::ptr::null(),
                    ));
                    items.push_back(Member::from_ptr(gap));
                }
            }

            while index < old_items.len()
                && unsafe { &*old_items[index].Get() }.EndOffset() < end_offset
            {
                index += 1;
            }
            if index == old_items.len() {
                break;
            }

            let delta = new_length as i64 - old_length as i64;
            let delta = i32::try_from(delta).expect("inline text length delta exceeds i32");
            let inserted_end = Self::AdjustOffset(end_offset, delta);
            let old_item = unsafe { &*old_items[index].Get() };
            if start_offset < inserted_end {
                let inserted = MakeGarbageCollected(old_item.clone_adjusted(
                    start_offset,
                    inserted_end,
                    std::ptr::null(),
                ));
                items.push_back(Member::from_ptr(inserted));
            }

            if end_offset < old_item.EndOffset() {
                let copied = self.CopyItemAfter(old_item, end_offset);
                let copied_start = unsafe { &*copied }.StartOffset();
                if end_offset < copied_start {
                    let gap = MakeGarbageCollected(old_item.clone_adjusted(
                        end_offset,
                        copied_start,
                        std::ptr::null(),
                    ));
                    items.push_back(Member::from_ptr(gap));
                    Self::ShiftItem(gap, delta);
                }
                items.push_back(Member::from_ptr(copied));
                Self::ShiftItem(copied, delta);
            }

            index += 1;
            while index < old_items.len() {
                let old_item = unsafe { &*old_items[index].Get() };
                debug_assert!(end_offset <= old_item.StartOffset());
                let copy = MakeGarbageCollected(old_item.clone());
                items.push_back(Member::from_ptr(copy));
                Self::ShiftItem(copy, delta);
                index += 1;
            }
            break;
        }

        if items.is_empty() {
            let first = unsafe { &*old_items[0].Get() };
            let item =
                MakeGarbageCollected(first.clone_adjusted(0, new_text.length(), std::ptr::null()));
            items.push_back(Member::from_ptr(item));
        } else {
            let last_end = unsafe { &*items.last().unwrap().Get() }.EndOffset();
            if last_end < new_text.length() {
                let last = unsafe { &*old_items.last().unwrap().Get() };
                let item = MakeGarbageCollected(last.clone_adjusted(
                    last_end,
                    new_text.length(),
                    std::ptr::null(),
                ));
                items.push_back(Member::from_ptr(item));
            }
        }

        self.VerifyItems(&items);
        let old_data = unsafe { &mut *self.data_ };
        old_data.items.clear();
        // C++ moves from `items` while its ClearCollectionScope remains live.
        // `take` leaves a valid empty vector for that scope's Drop.
        old_data.items = std::mem::take(&mut items);
        old_data.text_content = new_text.clone();
    }

    // cpp: layoutng_inline/inline_node.cc:151-189,886-902
    // C++ branches on 8-bit/16-bit storage to compare code units; the Rust
    // string always retains UTF-16 units, so one comparison covers all cases.
    fn MatchedLengths(&self, old_text: &String, new_text: &String) -> (u32, u32) {
        let old = old_text.Span16().unwrap_or_default();
        let new = new_text.Span16().unwrap_or_default();
        let start = old
            .iter()
            .zip(new.iter())
            .take_while(|(a, b)| a == b)
            .count();
        let max_end = old.len().min(new.len()) - start;
        let end = old[old.len() - max_end..]
            .iter()
            .rev()
            .zip(new[new.len() - max_end..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        debug_assert!(start <= old.len() - end && start <= new.len() - end);
        (start as u32, end as u32)
    }

    // cpp: layoutng_inline/inline_node.cc:904-908
    fn AdjustOffset(offset: u32, delta: i32) -> u32 {
        if delta > 0 {
            offset + delta as u32
        } else {
            offset - (-delta) as u32
        }
    }

    // cpp: layoutng_inline/inline_node.cc:910-930
    fn CopyItemAfter(&self, item: &InlineItem, start_offset: u32) -> *mut InlineItem {
        debug_assert!(item.StartOffset() <= start_offset && start_offset < item.EndOffset());
        let safe_start = self.GetFirstSafeToReuse(item, start_offset);
        let end = item.EndOffset();
        if end == safe_start {
            return MakeGarbageCollected(item.clone_adjusted(start_offset, end, std::ptr::null()));
        }
        debug_assert!(safe_start < end);
        let shape = unsafe { &*item.TextShapeResult() }.SubRange(safe_start, end);
        MakeGarbageCollected(item.clone_adjusted(safe_start, end, shape))
    }

    // cpp: layoutng_inline/inline_node.cc:932-950
    fn CopyItemBefore(&self, item: &InlineItem, end_offset: u32) -> *mut InlineItem {
        debug_assert!(item.StartOffset() < end_offset && end_offset <= item.EndOffset());
        let safe_end = self.GetLastSafeToReuse(item, end_offset);
        let start = item.StartOffset();
        if safe_end <= start {
            return MakeGarbageCollected(item.clone_adjusted(start, end_offset, std::ptr::null()));
        }
        debug_assert!(safe_end < item.EndOffset());
        let shape = unsafe { &*item.TextShapeResult() }.SubRange(start, safe_end);
        MakeGarbageCollected(item.clone_adjusted(start, safe_end, shape))
    }

    // cpp: layoutng_inline/inline_node.cc:952-969
    fn GetFirstSafeToReuse(&self, item: &InlineItem, start_offset: u32) -> u32 {
        debug_assert!(item.StartOffset() <= start_offset && start_offset <= item.EndOffset());
        let end = item.EndOffset();
        let skip = 1;
        let shape = item.TextShapeResult();
        if shape.is_null() || unsafe { &*shape }.IsAppliedSpacing() || start_offset + skip >= end {
            return end;
        }
        unsafe { &*shape }.EnsurePositionData(true);
        unsafe { &*shape }.CachedNextSafeToBreakOffset(start_offset + skip)
    }

    // cpp: layoutng_inline/inline_node.cc:971-991
    fn GetLastSafeToReuse(&self, item: &InlineItem, end_offset: u32) -> u32 {
        debug_assert!(item.StartOffset() < end_offset && end_offset <= item.EndOffset());
        let start = item.StartOffset();
        let max_context = if unsafe { StringIs8BitForInline(&(&*self.data_).text_content) } {
            2
        } else {
            10
        };
        let skip = max_context - 1;
        let shape = item.TextShapeResult();
        if shape.is_null() || unsafe { &*shape }.IsAppliedSpacing() || end_offset <= start + skip {
            return start;
        }
        unsafe { &*shape }.EnsurePositionData(true);
        unsafe { &*shape }.CachedPreviousSafeToBreakOffset(end_offset - skip)
    }

    // cpp: layoutng_inline/inline_node.cc:993-1003
    fn ShiftItem(item: *mut InlineItem, delta: i32) {
        if delta == 0 {
            return;
        }
        let item_ref = unsafe { &mut *item };
        let shape = item_ref.TextShapeResult();
        let start = Self::AdjustOffset(item_ref.StartOffset(), delta);
        let end = Self::AdjustOffset(item_ref.EndOffset(), delta);
        item_ref.SetOffset(start, end);
        if !shape.is_null() {
            item_ref.SetTextShapeResultForInline(unsafe { &*shape }.CopyAdjustedOffset(start));
        }
    }

    // cpp: layoutng_inline/inline_node.cc:1005-1020
    fn VerifyItems(&self, items: &InlineItems) {
        #[cfg(debug_assertions)]
        {
            if items.is_empty() {
                return;
            }
            let mut last = unsafe { &*items[0].Get() }.StartOffset();
            for member in items {
                let item = unsafe { &*member.Get() };
                debug_assert!(item.StartOffset() <= item.EndOffset());
                debug_assert_eq!(last, item.StartOffset());
                last = item.EndOffset();
                let shape = item.TextShapeResult();
                if !shape.is_null() {
                    debug_assert!(item.StartOffset() < item.EndOffset());
                    debug_assert_eq!(unsafe { &*shape }.StartIndex(), item.StartOffset());
                    debug_assert_eq!(unsafe { &*shape }.EndIndex(), item.EndOffset());
                }
            }
            let current = unsafe { &*unsafe { &*self.block_flow_ }.GetInlineNodeData() };
            debug_assert_eq!(last, current.text_content.length());
        }
    }
}

// cpp: layoutng_inline/inline_node.cc:1026-1070
pub fn InlineNodeSetTextWithOffset(
    layout_text: *mut LayoutText,
    mut new_text: String,
    diff: &TextDiffRange,
) -> bool {
    let text = unsafe { &mut *layout_text };
    if !text.HasValidInlineItems() || !text.IsInLayoutNGInlineFormattingContext() {
        return false;
    }
    let old_text = text.TransformedText().clone();
    if diff.offset == 0 && diff.old_size == old_text.length() {
        return false;
    }

    let mut editor = InlineNodeDataEditor::new(layout_text);
    let previous_data = editor.Prepare();
    if previous_data.is_null() {
        return false;
    }
    let mut offset_map = TextOffsetMap::default();
    new_text = text.TransformAndSecureText(&new_text, &mut offset_map);
    if !offset_map.IsEmpty() {
        return false;
    }
    text.SetTextInternal(new_text);
    text.ClearHasNoControlItems();
    text.ClearHasVariableLengthTransform();

    let node = InlineNode::new(editor.GetLayoutBlockFlow());
    let data = node.MutableData();
    unsafe { &mut *data }
        .items
        .reserve(unsafe { &*previous_data }.items.len());
    let previous_text = unsafe { &*previous_data }.text_content.clone();
    let mut builder = InlineItemsBuilder::new(
        editor.GetLayoutBlockFlow(),
        &mut unsafe { &mut *data }.items,
        &previous_text,
        std::ptr::null(),
    );
    text.ClearInlineItems();
    collect_inlines_internal(&mut builder, previous_data);
    builder.DidFinishCollectInlines(unsafe { &mut *data });
    editor.Run();
    node.SegmentText(data, std::ptr::null_mut());
    node.ShapeTextIncludingFirstLine(
        data,
        &unsafe { &*previous_data }.text_content,
        &unsafe { &*previous_data }.items,
    );
    node.AssociateItemsWithInlines(data);
    true
}

// cpp: layoutng_inline/inline_node.cc:549-568
fn should_break_shaping_before_text(
    item: &InlineItem,
    start_item: &InlineItem,
    start_style: &ComputedStyle,
    start_font: &Font,
    start_direction: TextDirection,
) -> bool {
    debug_assert_eq!(item.Type(), InlineItemType::kText);
    debug_assert!(!item.Style().is_null());
    let style = unsafe { &*item.Style() };
    if !std::ptr::eq(style, start_style) {
        let font = style.GetFont();
        if !std::ptr::eq(font, start_font) && unsafe { &*font } != start_font {
            return true;
        }
    }
    item.Direction() != start_direction || !item.EqualsRunSegment(start_item)
}

// cpp: layoutng_inline/inline_node.cc:571-586
fn should_break_shaping_before_box(item: &InlineItem) -> bool {
    debug_assert_eq!(item.Type(), InlineItemType::kOpenTag);
    debug_assert!(!item.Style().is_null());
    let style = unsafe { &*item.Style() };
    if (style.MayHavePadding() && !style.PaddingInlineStart().IsZero())
        || (style.MayHaveMargin() && !style.MarginInlineStart().IsZero())
        || style.BorderInlineStartWidth() != 0
        || style.VerticalAlign() != EVerticalAlign::kBaseline
    {
        return true;
    }
    false
}

// cpp: layoutng_inline/inline_node.cc:589-604
fn should_break_shaping_after_box(item: &InlineItem) -> bool {
    debug_assert_eq!(item.Type(), InlineItemType::kCloseTag);
    debug_assert!(!item.Style().is_null());
    let style = unsafe { &*item.Style() };
    if (style.MayHavePadding() && !style.PaddingInlineEnd().IsZero())
        || (style.MayHaveMargin() && !style.MarginInlineEnd().IsZero())
        || style.BorderInlineEndWidth() != 0
        || style.VerticalAlign() != EVerticalAlign::kBaseline
    {
        return true;
    }
    false
}

// cpp: layoutng_inline/inline_node.cc:606-628
fn needs_shaping(item: &InlineItem) -> bool {
    if item.Type() != InlineItemType::kText {
        return false;
    }
    if item.Length() == 0 {
        return false;
    }
    if item.IsUnsafeToReuseShapeResult() {
        return true;
    }
    let shape_result = item.TextShapeResult();
    if shape_result.is_null() {
        return true;
    }
    let shape_result = unsafe { &*shape_result };
    debug_assert_eq!(item.StartOffset(), shape_result.StartIndex());
    if !shape_result.IsStartSafeToBreak() {
        return true;
    }
    false
}

// cpp: layoutng_inline/inline_node.cc:631-636
fn first_line_needs_reshape(first_line_style: &ComputedStyle, base_style: &ComputedStyle) -> bool {
    let base_font = base_style.GetFont();
    let first_line_font = first_line_style.GetFont();
    base_font != first_line_font && unsafe { &*base_font } != unsafe { &*first_line_font }
}

// cpp: layoutng_inline/inline_node.cc:640-651
fn truncate_or_pad_text(text: &mut String, length: u32) {
    if text.length() > length {
        *text =
            String::from_utf16(&text.Span16().expect("non-null UTF-16 text")[..length as usize]);
    } else if text.length() < length {
        let mut units = text.Span16().unwrap_or_default().to_vec();
        units.reserve(length as usize - units.len());
        while units.len() < length as usize {
            units.push(0x0020);
        }
        *text = String::from_utf16(&units);
    }
}

// cpp: layoutng_inline/inline_node.cc:66-77
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeIsStickyImagesQuirkForContentSize(this: &InlineNode) -> bool {
    if unsafe { &*this.GetLayoutBox() }.InQuirksModeForLayout() {
        let style = this.Style();
        if style.Display() == EDisplay::kTableCell && style.LogicalWidth().IsAuto() {
            return true;
        }
    }
    false
}

// cpp: layoutng_inline/inline_node.cc:80-83
pub fn InlineNodeInvalidatePrepareLayoutForTest(this: &mut InlineNode) {
    let block = this.GetLayoutBlockFlow();
    unsafe { &mut *block }.ResetInlineNodeData();
    debug_assert!(!this.IsPrepareLayoutFinished());
}

// cpp: layoutng_inline/inline_node.cc:85-87
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeItemsData(
    this: &InlineNode,
    first_line: bool,
) -> *const InlineItemsData {
    this.Data().ItemsData(first_line)
}

// cpp: layoutng_inline/inline_node.cc:89-95
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeFirstLineOffsetMap(
    this: &InlineNode,
) -> *const Option<TextOffsetMap> {
    static EMPTY: Option<TextOffsetMap> = None;
    if this.Data().HasFirstLineItems() {
        return this.Data().ItemsData(true).OffsetMap();
    }
    &EMPTY
}

// cpp: layoutng_inline/inline_node.cc:97-119
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeIsBidiEnabled(this: &InlineNode) -> bool {
    this.Data().IsBidiEnabled()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeBaseDirection(this: &InlineNode) -> foundation::TextDirection {
    match this.Data().BaseDirection() {
        InputTextDirection::kLtr => TextDirection::kLtr,
        InputTextDirection::kRtl => TextDirection::kRtl,
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeHasFloats(this: &InlineNode) -> bool {
    this.Data().HasFloats()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeHasInitialLetterBox(this: &InlineNode) -> bool {
    this.Data().HasInitialLetterBox()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeHasRuby(this: &InlineNode) -> bool {
    this.Data().HasRuby()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeHasTextEmphasis(this: &InlineNode) -> bool {
    this.Data().HasTextEmphasis()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeIsBisectLineBreakDisabled(this: &InlineNode) -> bool {
    this.Data().IsBisectLineBreakDisabled()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeIsScoreLineBreakDisabled(this: &InlineNode) -> bool {
    this.Data().IsScoreLineBreakDisabled()
}
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeCanContainFirstFormattedLine(this: &InlineNode) -> bool {
    let block = this.GetLayoutBlockFlow();
    debug_assert!(!block.is_null());
    unsafe { &*block }.CanContainFirstFormattedLine()
}

// cpp: layoutng_inline/inline_node.cc:121-124
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeMutableData(this: &InlineNode) -> *mut InlineNodeData {
    unsafe { &*this.GetLayoutBlockFlow() }.GetInlineNodeData()
}

// cpp: layoutng_inline/inline_node.cc:125-130
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeData(this: &InlineNode) -> *const InlineNodeData {
    debug_assert!(this.IsPrepareLayoutFinished());
    let block = this.GetLayoutBlockFlow();
    debug_assert!(!unsafe { &*block }.NeedsCollectInlines());
    unsafe { &*block }.GetInlineNodeData()
}

// cpp: layoutng_inline/inline_node.cc:131-135
pub fn InlineNodeMaybeDirtyData(this: &InlineNode) -> *const InlineNodeData {
    debug_assert!(this.IsPrepareLayoutFinished());
    unsafe { &*this.GetLayoutBlockFlow() }.GetInlineNodeData()
}

// cpp: layoutng_inline/inline_node.cc:658-660
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeIsPrepareLayoutFinished(this: &InlineNode) -> bool {
    let data = unsafe { &*this.GetLayoutBlockFlow() }.GetInlineNodeData();
    !data.is_null() && !unsafe { &*data }.text_content.IsNull()
}

// cpp: layoutng_inline/inline_node.cc:662-686
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodePrepareLayoutIfNeeded(this: &InlineNode) {
    let mut previous_data: *mut InlineNodeData = std::ptr::null_mut();
    let block_flow = unsafe { &mut *this.GetLayoutBlockFlow() };
    if this.IsPrepareLayoutFinished() {
        if !block_flow.NeedsCollectInlines() {
            return;
        }
        // Text-combine calculates its font from the current text width, so
        // the previous inline data cannot be reused for that path.
        if !this.IsTextCombine() {
            previous_data = block_flow.TakeInlineNodeData();
        }
        block_flow.ResetInlineNodeData();
    }

    this.PrepareLayout(previous_data);

    if !previous_data.is_null() {
        // The old GC allocation survives until collection; release its large
        // vectors and UTF-16 contents as soon as the new data is prepared.
        let previous_data = unsafe { &mut *previous_data };
        previous_data.items.clear();
        previous_data.text_content = String::default();
    }
}

// cpp: layoutng_inline/inline_node.cc:688-720
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodePrepareLayout(
    this: &InlineNode,
    previous_data: *mut InlineNodeData,
) {
    let data = this.MutableData();
    debug_assert!(!data.is_null());
    this.CollectInlines(data, previous_data);
    this.SegmentText(data, previous_data);
    let previous_text = if previous_data.is_null() {
        std::ptr::null()
    } else {
        unsafe { &(&*previous_data).text_content }
    };
    this.ShapeTextIncludingFirstLine(data, previous_text, std::ptr::null());
    this.AssociateItemsWithInlines(data);
    debug_assert_eq!(data, this.MutableData());

    let block_flow = unsafe { &mut *this.GetLayoutBlockFlow() };
    block_flow.ClearNeedsCollectInlines();
    if this.IsTextCombine() {
        this.AdjustFontForTextCombineUprightAll();
    }

    #[cfg(feature = "expensive_dchecks")]
    {
        let data = unsafe { &mut *data };
        debug_assert!(data.offset_mapping.Get().is_null());
        this.ComputeOffsetMappingIfNeeded();
        debug_assert!(!data.offset_mapping.Get().is_null());
        data.offset_mapping.Clear();
    }
}

// cpp: layoutng_inline/inline_node.cc:1239-1246
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSegmentText(
    this: &InlineNode,
    data: *mut InlineNodeData,
    previous_data: *mut InlineNodeData,
) {
    this.SegmentBidiRuns(data);
    this.SegmentScriptRuns(data, previous_data);
    this.SegmentFontOrientation(data);
    let data = unsafe { &mut *data };
    if !data.segments.Get().is_null() {
        unsafe { &mut *data.segments.Get() }.ComputeItemIndex(&data.items);
    }
}

// cpp: layoutng_inline/inline_node.cc:1249-1313
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSegmentScriptRuns(
    this: &InlineNode,
    data: *mut InlineNodeData,
    previous_data: *mut InlineNodeData,
) {
    let data = unsafe { &mut *data };
    let text_content = data.text_content.clone();
    if text_content.empty() {
        data.segments.Clear();
        return;
    }

    if !previous_data.is_null() {
        let previous_data = unsafe { &*previous_data };
        if text_content == previous_data.text_content {
            if previous_data.segments.Get().is_null() {
                if let Some(previous_item) = previous_data
                    .items
                    .iter()
                    .find(|item| unsafe { &*item.Get() }.Type() == InlineItemType::kText)
                {
                    let packed_segment = unsafe { &*previous_item.Get() }.SegmentData();
                    for item_ptr in &data.items {
                        let item = unsafe { &mut *item_ptr.Get() };
                        if item.Type() == InlineItemType::kText {
                            item.SetPackedSegmentDataForInline(packed_segment);
                        }
                    }
                    data.segments.Clear();
                    return;
                }
            } else if this.IsHorizontalTypographicMode() {
                data.segments = previous_data.segments;
                return;
            }
        }
    }

    // BlinkString stores UTF-16 regardless of whether C++ String chose its
    // Latin-1 buffer; Latin-1 representability preserves the fast-path test.
    let is_latin1 = text_content
        .Span16()
        .is_some_and(|units| units.iter().all(|unit| *unit <= 0xff));
    if (is_latin1 || !data.HasNonOrc16BitCharacters()) && !data.is_bidi_enabled_ {
        if !data.items.is_empty() {
            let range = RunSegmenterRange {
                start: 0,
                end: text_content.length(),
                script: icu_bidi::USCRIPT_LATIN,
                render_orientation: RenderOrientation::kOrientationKeep,
                font_fallback_priority: FontFallbackPriority::kText,
            };
            layoutng::internal::inline_item::InlineItem::SetSegmentData(&range, &mut data.items);
        }
        data.segments.Clear();
        return;
    }

    let mut segmenter = RunSegmenter::new(
        text_content.Span16().expect("non-null UTF-16 text"),
        FontOrientation::kHorizontal,
    );
    let mut range = RunSegmenterRange::default();
    let consumed = segmenter.Consume(&mut range);
    debug_assert!(consumed);
    if range.end == text_content.length() {
        layoutng::internal::inline_item::InlineItem::SetSegmentData(&range, &mut data.items);
        data.segments.Clear();
        return;
    }

    if data.segments.Get().is_null() {
        data.segments = Member::from_ptr(MakeGarbageCollected(InlineItemSegments::default()));
    }
    unsafe { &mut *data.segments.Get() }.ComputeSegments(&mut segmenter, &mut range);
    debug_assert_eq!(range.end, text_content.length());
}

// cpp: layoutng_inline/inline_node.cc:1315-1355
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSegmentFontOrientation(
    this: &InlineNode,
    data: *mut InlineNodeData,
) {
    if this.IsHorizontalTypographicMode() {
        return;
    }

    let data = unsafe { &mut *data };
    let items = &data.base_.items;
    if items.is_empty() {
        return;
    }
    // The Rust string stores UTF-16 units already; C++ Ensure16Bit() needs no
    // additional conversion before the orientation iterator reads it.
    let text_content = &data.base_.text_content;
    let capacity = items.len() as u32 + text_content.length() / 10;
    let segments_member = &mut data.base_.segments;
    let mut segments = segments_member.Get();
    if !segments.is_null() {
        let segments_ref = unsafe { &mut *segments };
        debug_assert!(!segments_ref.IsEmpty());
        segments_ref.ReserveCapacity(capacity);
        debug_assert_eq!(text_content.length(), segments_ref.EndOffset());
    }
    let mut segment_index = 0;

    for item_ptr in items {
        let item = unsafe { &*item_ptr.Get() };
        if item.Type() == InlineItemType::kText
            && item.Length() != 0
            && unsafe { &*(*item.Style()).GetFont() }
                .GetFontDescription()
                .Orientation()
                == FontOrientation::kVerticalMixed
        {
            if segments.is_null() {
                segments = MakeGarbageCollected(InlineItemSegments::default());
                *segments_member = Member::from_ptr(segments);
                let segments_ref = unsafe { &mut *segments };
                segments_ref.ReserveCapacity(capacity);
                segments_ref.Append(InlineItemSegment::new_from_item(
                    text_content.length(),
                    item,
                ));
                debug_assert_eq!(text_content.length(), segments_ref.EndOffset());
            }
            segment_index = unsafe { &mut *segments }.AppendMixedFontOrientation(
                text_content,
                item.StartOffset(),
                item.EndOffset(),
                segment_index,
            );
        }
    }
}

// cpp: layoutng_inline/inline_node.cc:1357-1487
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSegmentBidiRuns(this: &InlineNode, data: *mut InlineNodeData) {
    let data = unsafe { &mut *data };
    if !data.is_bidi_enabled_ {
        data.SetBaseDirection(InputTextDirection::kLtr);
        return;
    }

    let block_style = this.Style();
    let base_direction = if block_style.GetUnicodeBidi() != UnicodeBidi::kPlaintext {
        Some(block_style.Direction())
    } else {
        None
    };
    let mut bidi = BidiParagraph::default();
    // BlinkString is already backed by UTF-16; C++ Ensure16Bit() is implicit.
    let text_content = data.text_content.clone();
    if !bidi.SetParagraph(&text_content, base_direction) {
        data.DisableBidi();
        return;
    }
    data.SetBaseDirection(if IsLtr(bidi.BaseDirection()) {
        InputTextDirection::kLtr
    } else {
        InputTextDirection::kRtl
    });
    if bidi.IsUnidirectional() && IsLtr(bidi.BaseDirection()) {
        data.is_bidi_enabled_ = false;
        return;
    }

    // Out-of-flow items have zero source length. Insert U+FFFC into the
    // temporary bidi paragraph so ICU treats them as neutral characters.
    struct OutOfFlowItem {
        text_offset: u32,
        #[cfg(feature = "expensive_dchecks")]
        level: u8,
    }
    let mut out_of_flow_items = Vec::<OutOfFlowItem>::new();
    let mut text_len = text_content.length();
    if data.HasFloatingOrOutOfFlowPositioned() {
        let text_units = text_content.Span16().expect("non-null UTF-16 text");
        let mut units = Vec::<u16>::new();
        let mut last_offset = 0usize;
        for item_ptr in data.items.iter() {
            let item = unsafe { &*item_ptr.Get() };
            if item.IsFloatingOrOutOfFlowPositioned() {
                let offset = item.StartOffset() as usize;
                if units.is_empty() {
                    units.reserve(text_len as usize + 16);
                }
                units.extend_from_slice(&text_units[last_offset..offset]);
                last_offset = offset;
                out_of_flow_items.push(OutOfFlowItem {
                    text_offset: units.len() as u32,
                    #[cfg(feature = "expensive_dchecks")]
                    level: 0,
                });
                units.push(0xfffc);
            }
        }
        debug_assert_eq!(units.is_empty(), out_of_flow_items.is_empty());
        if !units.is_empty() {
            units.extend_from_slice(&text_units[last_offset..]);
            let text_content_with_out_of_flow = String::from_utf16(&units);
            if !bidi.SetParagraph(&text_content_with_out_of_flow, base_direction) {
                data.DisableBidi();
                return;
            }
            text_len = text_content_with_out_of_flow.length();
            out_of_flow_items.push(OutOfFlowItem {
                text_offset: u32::MAX,
                #[cfg(feature = "expensive_dchecks")]
                level: 0,
            });
        }
    }

    let items = &mut data.items;
    let mut out_of_flow_item_index = 0usize;
    let mut item_index = 0u32;
    let mut start = 0u32;
    while start < text_len {
        debug_assert_eq!(
            unsafe { &*items[item_index as usize].Get() }.StartOffset(),
            start - out_of_flow_item_index as u32
        );
        let mut level = 0u8;
        let end = bidi.GetLogicalRun(start, &mut level);
        if out_of_flow_items.is_empty() {
            item_index = InlineItem::SetBidiLevelForItemsDefault(items, item_index, end, level);
        } else {
            let mut num_out_of_flow_in_this_run = 0u32;
            while end > out_of_flow_items[out_of_flow_item_index].text_offset {
                #[cfg(feature = "expensive_dchecks")]
                {
                    out_of_flow_items[out_of_flow_item_index].level = level;
                }
                out_of_flow_item_index += 1;
                num_out_of_flow_in_this_run += 1;
            }
            item_index = InlineItem::SetBidiLevelForItems(
                items,
                item_index,
                end - out_of_flow_item_index as u32,
                level,
                num_out_of_flow_in_this_run,
            );
        }
        start = end;
    }

    #[cfg(feature = "expensive_dchecks")]
    {
        if !out_of_flow_items.is_empty() {
            debug_assert_eq!(out_of_flow_item_index, out_of_flow_items.len() - 1);
            out_of_flow_item_index = 0;
            for item_ptr in items.iter() {
                let item = unsafe { &*item_ptr.Get() };
                if item.IsFloatingOrOutOfFlowPositioned() {
                    debug_assert_eq!(
                        item.BidiLevel(),
                        out_of_flow_items[out_of_flow_item_index].level
                    );
                    out_of_flow_item_index += 1;
                }
            }
        }
        while (item_index as usize) < items.len()
            && unsafe { &*items[item_index as usize].Get() }.Length() == 0
        {
            item_index += 1;
        }
        debug_assert_eq!(item_index as usize, items.len());
    }
}

// cpp: layoutng_inline/inline_node.cc:1489-1575
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeIsNGShapeCacheAllowed(
    _this: &InlineNode,
    text_content: &String,
    override_font: *const Font,
    items: &layoutng::internal::inline_item::InlineItems,
    spacing: &mut ShapeResultSpacing,
) -> bool {
    if text_content.empty() {
        return false;
    }

    let mut font = override_font;
    let mut previous_text_end_offset = 0u32;
    for (index, item_ptr) in items.iter().enumerate() {
        let item = unsafe { &*item_ptr.Get() };
        match item.Type() {
            InlineItemType::kControl | InlineItemType::kText => {
                if font.is_null() && item.Type() == InlineItemType::kText {
                    font = item.FontWithSvgScaling();
                }
                if previous_text_end_offset != item.StartOffset() {
                    return false;
                }
                previous_text_end_offset = item.EndOffset();
            }
            InlineItemType::kFloating | InlineItemType::kOutOfFlowPositioned => {}
            InlineItemType::kOpenTag => {
                if index != 0 {
                    return false;
                }
            }
            InlineItemType::kCloseTag => {
                if index + 1 != items.len() {
                    return false;
                }
            }
            InlineItemType::kAtomicInline
            | InlineItemType::kBlockInInline
            | InlineItemType::kInitialLetterBox
            | InlineItemType::kListMarker
            | InlineItemType::kBidiControl
            | InlineItemType::kOpenRubyColumn
            | InlineItemType::kCloseRubyColumn
            | InlineItemType::kRubyLinePlaceholder => return false,
        }
    }

    if previous_text_end_offset != text_content.length() {
        return false;
    }
    if font.is_null() {
        return false;
    }
    let font = unsafe { &*font };
    if !font.HasSimpleFontFeatures() {
        return false;
    }
    if spacing.SetSpacingFromDescription(font.GetFontDescription()) {
        return false;
    }
    true
}

// cpp: layoutng_inline/inline_node.cc:1577-1817
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeShapeText(
    this: &InlineNode,
    data: *mut InlineItemsData,
    previous_text: *const String,
    previous_items: *const InlineItems,
    override_font: *const Font,
) {
    let data_ref = unsafe { &mut *data };
    let text_content = data_ref.text_content.clone();
    #[cfg(feature = "expensive_dchecks")]
    InlineItem::CheckIndex(&mut data_ref.items);

    let mut spacing = ShapeResultSpacing::new(
        &text_content,
        this.IsSvgText()
            || (RuntimeEnabledFeatures::WordSpacingWhiteSpacePreEnabled()
                && this.Style().ShouldPreserveWhiteSpaces()),
    );
    let mut auto_space = TextAutoSpace::new(data_ref);
    let allow_shape_cache =
        this.IsNGShapeCacheAllowed(&text_content, override_font, &data_ref.items, &mut spacing)
            && !auto_space.MayApply();
    let reusable_items = if previous_items.is_null() {
        None
    } else {
        Some(unsafe { &*previous_items })
    };
    let shaper = ReusingTextShaper::new(data_ref, reusable_items, allow_shape_cache);
    let mut is_next_start_of_paragraph = true;
    let segments = data_ref.segments.Get();
    debug_assert!(segments.is_null() || unsafe { &*segments }.EndOffset() == text_content.length());

    {
        let items = &mut data_ref.items;
        let mut index = 0usize;
        while index < items.len() {
            let start_item = unsafe { &mut *items[index].Get() };
            if start_item.Type() != InlineItemType::kText || start_item.Length() == 0 {
                index += 1;
                if !start_item.IsOpaqueForTextProcessing() {
                    is_next_start_of_paragraph = start_item.IsForcedLineBreak();
                }
                continue;
            }

            let start_style = unsafe { &*start_item.Style() };
            let font = if override_font.is_null() {
                unsafe { &*start_item.FontWithSvgScaling() }
            } else {
                unsafe { &*override_font }
            };
            #[cfg(debug_assertions)]
            if !this.IsTextCombine() {
                debug_assert!(override_font.is_null());
            } else {
                debug_assert_eq!(
                    font.GetFontDescription().Orientation(),
                    FontOrientation::kHorizontal
                );
                LayoutTextCombine::AssertStyleIsValid(start_style);
                debug_assert!(
                    override_font.is_null()
                        || font.GetFontDescription().WidthVariant()
                            != FontWidthVariant::kRegularWidth
                );
            }

            let ch = text_content.Span16().expect("non-null UTF-16 text")
                [start_item.StartOffset() as usize];
            shaper.set_options(ShapeOptions {
                is_line_start: is_next_start_of_paragraph,
                han_kerning_start: is_next_start_of_paragraph
                    && ShouldTrimStartOfParagraph(font.GetFontDescription().GetTextSpacingTrim())
                    && Character::MaybeHanKerningOpen(i32::from(ch)),
                ..ShapeOptions::default()
            });
            is_next_start_of_paragraph = false;
            let direction = start_item.Direction();
            let mut end_index = index + 1;
            let mut end_offset = start_item.EndOffset();

            if start_item.IsSymbolMarker() {
                let support = unsafe { &*list_support() };
                let list_style =
                    LayoutCounter::ListStyle(start_item.GetLayoutObject(), start_style);
                let symbol_width = support.symbol_width.expect("installed list symbol width")(
                    start_style,
                    &list_style,
                );
                debug_assert!(symbol_width >= foundation::LayoutUnit::default());
                start_item.SetTextShapeResultForInline(ShapeResult::CreateForSpaces(
                    font,
                    direction,
                    start_item.StartOffset(),
                    start_item.Length(),
                    symbol_width.ToFloat(),
                ));
                index += 1;
                continue;
            }

            let mut num_text_items = 1u32;
            while end_index < items.len() {
                let item = unsafe { &*items[end_index].Get() };
                if item.Type() == InlineItemType::kControl {
                    break;
                }
                if item.Type() == InlineItemType::kText {
                    if item.Length() == 0 {
                        end_index += 1;
                        continue;
                    }
                    if item.TextType() == TextItemType::kSymbolMarker {
                        break;
                    }
                    if should_break_shaping_before_text(
                        item,
                        start_item,
                        start_style,
                        font,
                        direction,
                    ) {
                        break;
                    }
                    if text_content.Span16().expect("non-null UTF-16 text")
                        [item.StartOffset() as usize]
                        == 0x200c
                    {
                        break;
                    }
                    end_offset = item.EndOffset();
                    num_text_items += 1;
                } else if item.Type() == InlineItemType::kOpenTag {
                    if should_break_shaping_before_box(item) {
                        break;
                    }
                    debug_assert_eq!(item.Length(), 0);
                } else if item.Type() == InlineItemType::kCloseTag {
                    if should_break_shaping_after_box(item) {
                        break;
                    }
                    debug_assert_eq!(item.Length(), 0);
                } else {
                    break;
                }
                end_index += 1;
            }

            if !previous_text.is_null()
                && end_offset == start_item.EndOffset()
                && !needs_shaping(start_item)
            {
                if !this.IsTextCombine() {
                    let result = unsafe { &*start_item.TextShapeResult() };
                    debug_assert_eq!(start_item.StartOffset(), result.StartIndex());
                    debug_assert_eq!(start_item.EndOffset(), result.EndIndex());
                    index += 1;
                    continue;
                }
            }

            if !previous_text.is_null() {
                let mut has_valid_shape_results = true;
                for item_index in index..end_index {
                    if needs_shaping(unsafe { &*items[item_index].Get() }) {
                        has_valid_shape_results = false;
                        break;
                    }
                }
                let text_start = start_item.StartOffset();
                debug_assert!(end_offset >= text_start);
                let previous = unsafe { &*previous_text };
                if has_valid_shape_results
                    && end_offset <= previous.length()
                    && text_content.Span16().expect("non-null UTF-16 text")
                        [text_start as usize..end_offset as usize]
                        == previous.Span16().expect("non-null UTF-16 text")
                            [text_start as usize..end_offset as usize]
                {
                    index = end_index;
                    continue;
                }
            }

            let shape_result = shaper.shape(start_item, font, end_offset);
            if spacing.SetSpacingFromDescription(font.GetFontDescription()) {
                debug_assert!(!this.IsTextCombine());
                debug_assert!(!allow_shape_cache);
                unsafe { &mut *shape_result.cast_mut() }.ApplySpacing(&mut spacing, 0);
            }
            if end_offset == start_item.EndOffset() {
                start_item.SetTextShapeResultForInline(shape_result);
                let result = unsafe { &*start_item.TextShapeResult() };
                debug_assert_eq!(result.StartIndex(), start_item.StartOffset());
                debug_assert_eq!(result.EndIndex(), start_item.EndOffset());
                index += 1;
                continue;
            }

            debug_assert!(num_text_items > 0);
            let mut text_item_ranges = HeapVector::<ShapeRange, 32>::new();
            text_item_ranges.ReserveInitialCapacity(num_text_items);
            let _clear_scope = unsafe { ClearCollectionScope::new(&mut text_item_ranges) };
            let result = unsafe { &*shape_result };
            let has_ligatures = result.HasLigatures();
            if has_ligatures {
                result.EnsurePositionData(true);
            }
            while index < end_index {
                let item = unsafe { &mut *items[index].Get() };
                index += 1;
                if item.Type() != InlineItemType::kText || item.Length() == 0 {
                    continue;
                }
                let item_result = ShapeResult::CreateEmpty(result);
                text_item_ranges.push(ShapeRange::new(
                    item.StartOffset(),
                    item.EndOffset(),
                    item_result,
                ));
                if has_ligatures
                    && item.EndOffset() < result.EndIndex()
                    && result.CachedNextSafeToBreakOffset(item.EndOffset()) != item.EndOffset()
                {
                    item.SetUnsafeToReuseShapeResult();
                }
                item.SetTextShapeResultForInline(item_result);
            }
            debug_assert_eq!(text_item_ranges.len(), num_text_items as usize);
            result.CopyRanges(&text_item_ranges);
        }
    }

    auto_space.ApplyIfNeeded(this, data_ref);
    #[cfg(debug_assertions)]
    for item_ptr in &data_ref.items {
        let item = unsafe { &*item_ptr.Get() };
        if item.Type() == InlineItemType::kText && item.Length() != 0 {
            let result = unsafe { &*item.TextShapeResult() };
            debug_assert_eq!(result.StartIndex(), item.StartOffset());
            debug_assert_eq!(result.EndIndex(), item.EndOffset());
        }
    }
}

// cpp: layoutng_inline/inline_node.cc:1819-1910
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeShapeTextForFirstLineIfNeeded(
    this: &InlineNode,
    data: *mut InlineNodeData,
) {
    let data = unsafe { &mut *data };
    debug_assert!(data.first_line_items_.Get().is_null());
    let layout_object = this.GetLayoutBox();
    if !unsafe { &*layout_object }.HasFirstLineStylesForLayout() {
        return;
    }

    let block_style = unsafe { &*layout_object }.StyleRef();
    let first_line_style = unsafe { &*layout_object }.FirstLineStyleRef();
    if std::ptr::eq(block_style, first_line_style) {
        return;
    }

    let mut text_content = data.text_content.clone();
    let mut needs_reshape = false;
    let mut offset_map = TextOffsetMap::default();
    if first_line_style.TextTransform() != block_style.TextTransform() {
        if RuntimeEnabledFeatures::FirstLineTextTransformEnabled() {
            text_content = first_line_style.ApplyTextTransform(
                &text_content,
                b' ' as u16,
                Some(&mut offset_map),
            );
            if text_content != data.text_content {
                needs_reshape = true;
            }
        } else {
            text_content = first_line_style.ApplyTextTransform(&text_content, b' ' as u16, None);
            if text_content != data.text_content {
                truncate_or_pad_text(&mut text_content, data.text_content.length());
                needs_reshape = true;
            }
        }
    }
    let with_offset_map = !offset_map.IsEmpty();
    let first_line_items: *mut InlineItemsData = if with_offset_map {
        MakeGarbageCollected(InlineItemsDataWithOffsetMap::default()).cast()
    } else {
        MakeGarbageCollected(InlineItemsData::default())
    };
    let first_line_items_ref = unsafe { &mut *first_line_items };
    first_line_items_ref.text_content = text_content;

    first_line_items_ref.items.reserve(data.items.len());
    for item_ptr in &data.items {
        let item = unsafe { &*item_ptr.Get() };
        let first_line_item = MakeGarbageCollected(item.clone());
        let first_line_item_ref = unsafe { &mut *first_line_item };
        first_line_item_ref.SetStyleVariant(StyleVariant::kFirstLine);
        if RuntimeEnabledFeatures::FirstLineTextTransformEnabled()
            && needs_reshape
            && !offset_map.IsEmpty()
        {
            let new_start = offset_map.MapOffset(first_line_item_ref.StartOffset());
            let new_end = offset_map.MapOffset(first_line_item_ref.EndOffset());
            first_line_item_ref.SetOffset(new_start, new_end);
        }
        first_line_items_ref
            .items
            .push(Member::from_ptr(first_line_item));
    }
    let segments = data.segments.Get();
    if !segments.is_null() {
        let cloned_segments = unsafe { &*segments }.Clone();
        first_line_items_ref.segments = Member::from_ptr(cloned_segments);
        if RuntimeEnabledFeatures::FirstLineTextTransformEnabled()
            && needs_reshape
            && !offset_map.IsEmpty()
        {
            unsafe { &mut *cloned_segments }.AdjustOffsets(&offset_map);
        }
    }

    #[cfg(feature = "expensive_dchecks")]
    InlineItem::CheckIndex(&mut first_line_items_ref.items);

    if needs_reshape || first_line_needs_reshape(first_line_style, block_style) {
        this.ShapeTextDefault(first_line_items);
    }
    if with_offset_map {
        unsafe { &mut *first_line_items.cast::<InlineItemsDataWithOffsetMap>() }.offset_map =
            Some(offset_map);
    }

    data.first_line_items_ = Member::from_ptr(first_line_items);
    data.is_score_line_break_disabled_ = true;
}

// cpp: layoutng_inline/inline_node.cc:1912-1919
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeShapeTextIncludingFirstLine(
    this: &InlineNode,
    data: *mut InlineNodeData,
    previous_text: *const String,
    previous_items: *const layoutng::internal::inline_item::InlineItems,
) {
    let data_ref = unsafe { &mut *data };
    InlineItem::UpdateIndex(&mut data_ref.items);
    this.ShapeText(data.cast(), previous_text, previous_items, std::ptr::null());
    this.ShapeTextForFirstLineIfNeeded(data);
}

// cpp: layoutng_inline/inline_node.cc:1921-1953
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeAssociateItemsWithInlines(
    _this: &InlineNode,
    data: *mut InlineNodeData,
) {
    #[cfg(debug_assertions)]
    let mut associated_objects = std::collections::HashSet::new();
    let data = unsafe { &mut *data };
    let data_ptr = data as *mut InlineNodeData;
    let items = &data.items;
    let size = items.len();
    let mut i = 0usize;
    while i != size {
        let object = unsafe { &*items[i].Get() }.GetLayoutObject();
        let layout_text = DynamicTo::<LayoutText>(object);
        if !layout_text.is_null() && !unsafe { &*layout_text }.IsBR() {
            #[cfg(debug_assertions)]
            debug_assert!(associated_objects.insert(object));
            let layout_text = unsafe { &mut *layout_text };
            layout_text.ClearHasBidiControlInlineItems();
            let mut has_bidi_control = false;
            let begin = i;
            i += 1;
            while i != size {
                let item = unsafe { &*items[i].Get() };
                if item.GetLayoutObject() != object {
                    break;
                }
                if item.Type() == InlineItemType::kBidiControl {
                    has_bidi_control = true;
                }
                i += 1;
            }
            layout_text.SetInlineItems(data_ptr.cast::<InlineItemsData>(), begin, i - begin);
            if has_bidi_control {
                layout_text.SetHasBidiControlInlineItems();
            }
            continue;
        }
        i += 1;
    }
}

// cpp: layoutng_inline/inline_node.cc:1955-1966
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeLayout(
    this: &InlineNode,
    space: &ConstraintSpace,
    break_token: *const BreakToken,
    column_spanner_path: *const ColumnSpannerPath,
    context: *mut ForwardInlineChildLayoutContext,
) -> *const LayoutResult {
    this.PrepareLayoutIfNeeded();
    let inline_break_token = To::<InlineBreakToken>(break_token);
    let mut algorithm = InlineLayoutAlgorithm::new(
        this.clone(),
        space,
        inline_break_token,
        column_spanner_path,
        context.cast::<InlineChildLayoutContext>(),
    );
    algorithm.Layout()
}

// cpp: layoutng_inline/inline_node.cc:1073-1076
pub fn InlineNodeEnsureData(this: &InlineNode) -> *const InlineNodeData {
    this.PrepareLayoutIfNeeded();
    this.Data()
}

// cpp: layoutng_inline/inline_node.cc:1970-1985
fn create_text_content_for_sticky_images_quirk(
    text: &String,
    items: &[foundation::Member<layoutng::internal::inline_item::InlineItem>],
) -> String {
    let mut units = text.Span16().unwrap_or_default().to_vec();
    for item in items {
        let item = unsafe { &*item.Get() };
        if item.Type() == InlineItemType::kAtomicInline && item.IsImage() {
            let offset = item.StartOffset() as usize;
            debug_assert_eq!(units[offset], 0xfffc);
            units[offset] = 0x00a0;
        }
    }
    String::from_utf16(&units)
}

// cpp: layoutng_inline/inline_node.cc:1989-2007
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeTextContentForStickyImagesQuirk(data: &InlineItemsData) -> String {
    for (index, item) in data.items.iter().enumerate() {
        let item = unsafe { &*item.Get() };
        if item.Type() == InlineItemType::kAtomicInline && item.IsImage() {
            return create_text_content_for_sticky_images_quirk(
                &data.text_content,
                &data.items[index..],
            );
        }
    }
    data.text_content.clone()
}

// The local C++ class is scoped to ComputeContentSize. Keep its state and
// per-line reset here so float widths follow the same line-breaking sequence.
// cpp: layoutng_inline/inline_node.cc:2041-2101
struct FloatsMaxSize {
    floats_inline_size: LayoutUnit,
    floating_objects: Vec<FloatingObject>,
}

impl FloatsMaxSize {
    // cpp: layoutng_inline/inline_node.cc:2046-2050
    fn new(float_input: &MinMaxSizesFloatInput) -> Self {
        let floats_inline_size =
            float_input.float_left_inline_size + float_input.float_right_inline_size;
        debug_assert!(floats_inline_size >= LayoutUnit::default());
        Self {
            floats_inline_size,
            floating_objects: Vec::new(),
        }
    }

    // cpp: layoutng_inline/inline_node.cc:2052-2058
    fn add_float(
        &mut self,
        float_style: &ComputedStyle,
        style: &ComputedStyle,
        float_inline_max_size_with_margin: LayoutUnit,
    ) {
        self.floating_objects.push(FloatingObject {
            float_style: Member::from_ptr(float_style as *const _ as *mut _),
            style: Member::from_ptr(style as *const _ as *mut _),
            float_inline_max_size_with_margin,
        });
    }

    // cpp: layoutng_inline/inline_node.cc:2060-2096
    fn compute_max_size_for_line(
        &mut self,
        line_inline_size: LayoutUnit,
        mut max_inline_size: LayoutUnit,
    ) -> LayoutUnit {
        if self.floating_objects.is_empty() {
            return max_inline_size.max(line_inline_size);
        }

        let mut previous_float_type = EFloat::kNone;
        for floating_object in &self.floating_objects {
            let float_style = unsafe { &*floating_object.float_style.Get() };
            let style = unsafe { &*floating_object.style.Get() };
            let float_clear = float_style.ClearWithContainingStyle(style);
            if (previous_float_type == EFloat::kLeft
                && (float_clear == EClear::kBoth || float_clear == EClear::kLeft))
                || (previous_float_type == EFloat::kRight
                    && (float_clear == EClear::kBoth || float_clear == EClear::kRight))
            {
                max_inline_size = max_inline_size.max(line_inline_size + self.floats_inline_size);
                self.floats_inline_size = LayoutUnit::default();
            }

            self.floats_inline_size += floating_object
                .float_inline_max_size_with_margin
                .ClampNegativeToZero();
            previous_float_type = float_style.FloatingWithContainingStyle(style);
        }
        max_inline_size = max_inline_size.max(line_inline_size + self.floats_inline_size);
        self.floats_inline_size = LayoutUnit::default();
        self.floating_objects.clear();
        max_inline_size
    }
}

// C++ defines this helper inside ComputeContentSize. The raw float accumulator
// pointer keeps the same aliasing relationship while the enclosing loop also
// adds newly encountered floating items.
// cpp: layoutng_inline/inline_node.cc:2103-2260
struct MaxSizeFromMinSize<'a> {
    position: LayoutUnit,
    max_size: LayoutUnit,
    items_data: &'a InlineItemsData,
    next_item_index: usize,
    max_size_cache: *const MaxSizeCache,
    node: &'a InlineNode,
    floats: *mut FloatsMaxSize,
    is_after_break: bool,
    annotation_nesting_level: usize,
}

impl<'a> MaxSizeFromMinSize<'a> {
    // cpp: layoutng_inline/inline_node.cc:2121-2128
    fn new(
        items_data: &'a InlineItemsData,
        max_size_cache: *const MaxSizeCache,
        node: &'a InlineNode,
        floats: *mut FloatsMaxSize,
    ) -> Self {
        Self {
            position: LayoutUnit::default(),
            max_size: LayoutUnit::default(),
            items_data,
            next_item_index: 0,
            max_size_cache,
            node,
            floats,
            is_after_break: true,
            annotation_nesting_level: 0,
        }
    }

    // cpp: layoutng_inline/inline_node.cc:2133-2153
    fn add_text_until(&mut self, end_item_index: usize) {
        let items = &self.items_data.items[self.next_item_index..end_item_index];
        self.next_item_index = end_item_index;
        for item_ptr in items {
            let item = unsafe { &*item_ptr.Get() };
            if item.Type() == InlineItemType::kOpenTag
                && unsafe { &*item.GetLayoutObject() }.IsInlineRubyText()
            {
                self.annotation_nesting_level += 1;
            } else if item.Type() == InlineItemType::kCloseTag
                && unsafe { &*item.GetLayoutObject() }.IsInlineRubyText()
            {
                self.annotation_nesting_level -= 1;
            } else if item.Type() == InlineItemType::kText
                && item.Length() != 0
                && self.annotation_nesting_level == 0
            {
                let shape_result = item.TextShapeResult();
                debug_assert!(!shape_result.is_null());
                self.position += unsafe { &*shape_result }
                    .SnappedWidth()
                    .ClampNegativeToZero();
            }
        }
    }

    // cpp: layoutng_inline/inline_node.cc:2155-2164
    fn force_line_break(&mut self, line_info: &LineInfo) {
        assert!(line_info.EndItemIndex() as usize <= self.items_data.items.len());
        self.add_text_until(line_info.EndItemIndex() as usize);
        self.max_size = unsafe { &mut *self.floats }
            .compute_max_size_for_line(self.position.ClampNegativeToZero(), self.max_size);
        self.position = LayoutUnit::default();
        self.is_after_break = true;
    }

    // cpp: layoutng_inline/inline_node.cc:2166-2187
    fn add_tabulation_characters(&mut self, item: &InlineItem, length: u32) {
        debug_assert!(length >= 1);
        self.add_text_until(item.Index() as usize);
        let style_ptr = item.Style();
        debug_assert!(!style_ptr.is_null());
        let style = unsafe { &*style_ptr };
        let tab_size = style.GetTabSize();
        let font = if RuntimeEnabledFeatures::TabSizeAncestorEnabled() {
            self.node.FontForTab()
        } else {
            unsafe { &*style.GetFont() }
        };
        let font_data = font.PrimaryFontForTabSize();
        // Keep the same fixed-point rounding as ShapeResult tab shaping.
        let glyph_advance = TextRunLayoutUnit::FromFloatRound(font.TabWidthAt(
            font_data,
            tab_size,
            self.position.ToFloat(),
        ));
        let mut run_advance: InlineLayoutUnit = glyph_advance.To::<16, i64>();
        if length > 1 {
            let later_advance =
                TextRunLayoutUnit::FromFloatRound(font.TabWidth(font_data, tab_size));
            for _ in 1..length {
                run_advance += later_advance.To::<16, i64>();
            }
        }
        self.position += run_advance.ToCeil::<6, i32>().ClampNegativeToZero();
    }

    // cpp: layoutng_inline/inline_node.cc:2189-2193
    fn finish(&mut self) -> LayoutUnit {
        self.add_text_until(self.items_data.items.len());
        unsafe { &mut *self.floats }
            .compute_max_size_for_line(self.position.ClampNegativeToZero(), self.max_size)
    }

    // cpp: layoutng_inline/inline_node.cc:2195-2210
    fn compute_from_min_size(&mut self, line_info: &LineInfo) {
        if self.is_after_break {
            self.position += line_info.TextIndent();
            self.is_after_break = false;
        }
        self.compute_from_min_size_internal(line_info);
        if line_info.HasForcedBreak() {
            self.force_line_break(line_info);
        }
    }

    // cpp: layoutng_inline/inline_node.cc:2212-2260
    fn compute_from_min_size_internal(&mut self, line_info: &LineInfo) {
        for result in line_info.Results() {
            let item = unsafe { &*result.item.Get() };
            if item.Type() == InlineItemType::kText {
                continue;
            }
            if item.Type() == InlineItemType::kBlockInInline {
                debug_assert!(line_info.HasForcedBreak());
            }
            if item.Type() == InlineItemType::kAtomicInline
                || item.Type() == InlineItemType::kBlockInInline
            {
                self.position += unsafe { &*self.max_size_cache }[item.Index() as usize];
                continue;
            }
            if item.Type() == InlineItemType::kControl {
                let character =
                    self.items_data.text_content.Span16().unwrap()[item.StartOffset() as usize];
                if character == u16::from(b'\n') {
                    debug_assert!(line_info.HasForcedBreak());
                }
                if character == u16::from(b'\t') {
                    self.add_tabulation_characters(item, result.Length());
                    continue;
                }
            }
            if result.IsRubyColumn() {
                let column = unsafe {
                    &*result
                        .ruby_column
                        .Get()
                        .cast::<InlineItemResultRubyColumn>()
                };
                self.compute_from_min_size_internal(&column.base_line);
                continue;
            }
            self.position += result.inline_size;
        }
    }
}

// cpp: layoutng_inline/inline_node.cc:2009-2360
fn compute_content_size(
    node: InlineNode,
    container_writing_mode: WritingMode,
    space: &ConstraintSpace,
    float_input: &MinMaxSizesFloatInput,
    mode: LineBreakerMode,
    max_size_cache: *mut MaxSizeCache,
    max_size_out: *mut Option<LayoutUnit>,
    depends_on_block_constraints_out: *mut bool,
) -> LayoutUnit {
    let style = node.Style();
    let available_inline_size = match mode {
        LineBreakerMode::kMinContent => LayoutUnit::default(),
        LineBreakerMode::kContent => float_input.constrained_inline_size,
        LineBreakerMode::kMaxContent => LayoutUnit::Max(),
    };

    let mut empty_exclusion_space = ExclusionSpace::default();
    let empty_leading_floats = LeadingFloats::default();
    let line_opportunity = LineLayoutOpportunity::with_inline_size(available_inline_size);
    let mut result = LayoutUnit::default();
    let mut line_breaker = LineBreaker::new(
        node.clone(),
        mode,
        space,
        &line_opportunity,
        &empty_leading_floats,
        std::ptr::null(),
        std::ptr::null(),
        &mut empty_exclusion_space,
    );
    if mode != LineBreakerMode::kContent {
        line_breaker.SetIntrinsicSizeOutputs(max_size_cache, depends_on_block_constraints_out);
    }
    let items_data = line_breaker.ItemsData() as *const InlineItemsData;

    if node.IsInitialLetterBox() {
        let mut inline_size = LayoutUnit::default();
        let mut line_info = LineInfo::default();
        loop {
            line_breaker.NextLine(&mut line_info);
            if line_info.Results().is_empty() {
                break;
            }
            inline_size = inline_size.max(CalculateInitialLetterBoxInlineSize(&line_info));
            if line_breaker.IsFinished() {
                break;
            }
        }
        return inline_size;
    }

    let mut floats_max_size = FloatsMaxSize::new(float_input);
    let mut can_compute_max_size_from_min_size = true;
    // C++ constructs this helper for every mode, although it is used only
    // for min-content. Rust avoids dereferencing the null cache in content mode.
    let mut max_size_from_min_size = (mode == LineBreakerMode::kMinContent).then(|| {
        MaxSizeFromMinSize::new(
            unsafe { &*items_data },
            max_size_cache,
            &node,
            &mut floats_max_size,
        )
    });

    let mut line_info = LineInfo::default();
    loop {
        line_breaker.NextLine(&mut line_info);
        if line_info.Results().is_empty() {
            break;
        }

        let inline_size = line_info.Width();
        for item_result in line_info.Results() {
            let item = unsafe { &*item_result.item.Get() };
            if item.Type() != InlineItemType::kFloating {
                continue;
            }
            let floating_object = item.GetLayoutObject();
            debug_assert!(!floating_object.is_null() && unsafe { &*floating_object }.IsFloating());
            let float_node = BlockNode::new(To::<LayoutBox>(floating_object));

            let mut builder = MinMaxConstraintSpaceBuilder::new(space, style, &float_node, true);
            builder.SetAvailableBlockSize(space.AvailableSize().block_size);
            builder.SetPercentageResolutionBlockSize(if float_node.IsReplaced() {
                space.ReplacedChildPercentageResolutionBlockSize()
            } else {
                space.PercentageResolutionBlockSize()
            });
            let float_space = builder.ToConstraintSpace();

            let child_result = ComputeMinAndMaxContentContribution(
                style,
                &float_node,
                &float_space,
                MinMaxSizesFloatInput::default(),
            );
            let child_inline_margins =
                ComputeMarginsFor(&float_space, float_node.Style(), space).InlineSum();
            if !depends_on_block_constraints_out.is_null() {
                unsafe {
                    *depends_on_block_constraints_out |= child_result.depends_on_block_constraints;
                }
            }
            if mode == LineBreakerMode::kMinContent {
                result = result.max(child_result.sizes.min_size + child_inline_margins);
            }
            floats_max_size.add_float(
                float_node.Style(),
                style,
                child_result.sizes.max_size + child_inline_margins,
            );
        }

        if mode == LineBreakerMode::kMinContent {
            result = result.max(inline_size);
            can_compute_max_size_from_min_size = can_compute_max_size_from_min_size
                && !line_breaker.HasClonedBoxDecorations()
                && !line_info.MayHaveRubyOverhang();
            if can_compute_max_size_from_min_size {
                max_size_from_min_size
                    .as_mut()
                    .unwrap()
                    .compute_from_min_size(&line_info);
            }
        } else {
            result = floats_max_size.compute_max_size_for_line(inline_size, result);
        }
        if line_breaker.IsFinished() {
            break;
        }
    }

    if mode == LineBreakerMode::kMinContent && can_compute_max_size_from_min_size {
        if node.IsSvgText() {
            unsafe { *max_size_out = Some(result) };
            return result;
        }
        unsafe { *max_size_out = Some(max_size_from_min_size.as_mut().unwrap().finish()) };

        #[cfg(feature = "expensive_dchecks")]
        {
            let content_size = compute_content_size(
                node.clone(),
                container_writing_mode,
                space,
                float_input,
                LineBreakerMode::kMaxContent,
                max_size_cache,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            let max_size = unsafe { (*max_size_out).unwrap() };
            if !max_size.MightBeSaturated() && !content_size.MightBeSaturated() {
                debug_assert_eq!(max_size.Round(), content_size.Round());
            }
        }
    }

    let _ = container_writing_mode;
    result
}

// cpp: layoutng_inline/inline_node.cc:2362-2398
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeComputeMinMaxSizes(
    this: &InlineNode,
    container_writing_mode: WritingMode,
    space: &ConstraintSpace,
    float_input: &MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
    this.PrepareLayoutIfNeeded();

    let mut max_size_cache = MaxSizeCache::default();
    let mut sizes = MinMaxSizes::default();
    let mut max_size = None;
    let mut depends_on_block_constraints = false;
    sizes.min_size = compute_content_size(
        this.clone(),
        container_writing_mode,
        space,
        float_input,
        LineBreakerMode::kMinContent,
        &mut max_size_cache,
        &mut max_size,
        &mut depends_on_block_constraints,
    );
    if this.Style().IsInShrinkToFitSubtree() {
        sizes.max_size = if float_input.constrained_inline_size <= sizes.min_size {
            sizes.min_size
        } else {
            compute_content_size(
                this.clone(),
                container_writing_mode,
                space,
                float_input,
                LineBreakerMode::kContent,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
    } else if let Some(cached_max_size) = max_size {
        sizes.max_size = cached_max_size;
    } else {
        sizes.max_size = compute_content_size(
            this.clone(),
            container_writing_mode,
            space,
            float_input,
            LineBreakerMode::kMaxContent,
            &mut max_size_cache,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }
    sizes.max_size = sizes.max_size.max(sizes.min_size);
    MinMaxSizesResult::new(sizes, depends_on_block_constraints)
}

// cpp: layoutng_inline/inline_node.cc:2400-2404
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeUseFirstLineStyleItemsData(this: &InlineNode) -> bool {
    let box_ptr = this.GetLayoutBox();
    !box_ptr.is_null()
        && unsafe { &*box_ptr }.HasFirstLineStylesForLayout()
        && this.Data().HasFirstLineItems()
}

// cpp: layoutng_inline/inline_node.cc:2497-2499
pub fn InlineNodeNeedsShapingForTesting(item: &InlineItem) -> bool {
    needs_shaping(item)
}

// cpp: layoutng_inline/inline_node.cc:2406-2415
pub fn InlineNodeCheckConsistency(this: &InlineNode) {
    #[cfg(debug_assertions)]
    for item_ptr in &this.Data().items {
        let item = unsafe { &*item_ptr.Get() };
        let object = item.GetLayoutObject();
        let style = item.Style();
        debug_assert!(
            object.is_null()
                || style.is_null()
                || std::ptr::eq(style, unsafe { &*object }.StyleRef())
        );
    }
}

// cpp: layoutng_inline/inline_node.cc:2417-2421
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSvgCharacterDataList(
    this: &InlineNode,
) -> *const Vec<(u32, SvgCharacterData)> {
    debug_assert!(this.IsSvgText());
    &unsafe { &*this.Data().svg_node_data_.Get() }.character_data_list
}

// cpp: layoutng_inline/inline_node.cc:2423-2427
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSvgTextLengthRangeList(
    this: &InlineNode,
) -> *const HeapVector<SvgTextContentRange> {
    debug_assert!(this.IsSvgText());
    &unsafe { &*this.Data().svg_node_data_.Get() }.text_length_range_list
}

// cpp: layoutng_inline/inline_node.cc:2429-2433
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeSvgTextPathRangeList(
    this: &InlineNode,
) -> *const HeapVector<SvgTextContentRange> {
    debug_assert!(this.IsSvgText());
    &unsafe { &*this.Data().svg_node_data_.Get() }.text_path_range_list
}

// cpp: layoutng_inline/inline_node.cc:2435-2443
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeFontForTab(this: &InlineNode) -> *const Font {
    let layout_box_node = this.GetDOMNode();
    let is_first_letter =
        !layout_box_node.is_null() && unsafe { &*layout_box_node }.IsFirstLetterPseudoElement();
    let font = if is_first_letter {
        this.Style().ContainerFont()
    } else {
        this.Style().GetFont()
    };
    debug_assert!(!font.is_null());
    font
}

// cpp: layoutng_inline/inline_node.cc:2445-2447
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeMinimumFontPhysicalSize(this: &InlineNode) -> Option<f32> {
    unsafe { &*this.GetLayoutBox() }.MinimumFontPhysicalSizeForLayout()
}

// cpp: layoutng_inline/inline_node.cc:2449-2495
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineNodeAdjustFontForTextCombineUprightAll(this: &InlineNode) {
    debug_assert!(this.IsTextCombine());
    debug_assert!(this.IsPrepareLayoutFinished());
    let content_width = calculate_width_for_text_combine(this.ItemsData(false));
    if content_width == 0.0 {
        return;
    }
    let text_combine = To::<LayoutTextCombine>(this.GetLayoutBlockFlow());
    let text_combine_ref = unsafe { &mut *text_combine };
    let desired_width = text_combine_ref.DesiredWidth();
    text_combine_ref.ResetLayout();
    if desired_width == 0.0 || content_width <= desired_width {
        return;
    }

    let font = unsafe { &*this.Style().GetFont() };
    let font_selector = font.GetFontSelector();
    let mut description = font.GetFontDescription().clone();
    for width_variant in [
        FontWidthVariant::kHalfWidth,
        FontWidthVariant::kThirdWidth,
        FontWidthVariant::kQuarterWidth,
    ] {
        unsafe { FontDescriptionSetWidthVariantForInline(&mut description, width_variant) };
        let compressed_font =
            MakeGarbageCollected(Font::new_with_selector(description.clone(), font_selector));
        if unsafe { &*compressed_font }.PrimaryFont().is_null() {
            continue;
        }
        this.ShapeText(
            this.MutableData().cast(),
            std::ptr::null(),
            std::ptr::null(),
            compressed_font,
        );
        if calculate_width_for_text_combine(this.ItemsData(false)) <= desired_width {
            text_combine_ref.SetCompressedFont(compressed_font);
            return;
        }
    }

    this.ShapeTextDefault(this.MutableData().cast());
    debug_assert_eq!(
        content_width,
        calculate_width_for_text_combine(this.ItemsData(false))
    );
    debug_assert_ne!(content_width, 0.0);
    text_combine_ref.SetScaleX(desired_width / content_width);
}
