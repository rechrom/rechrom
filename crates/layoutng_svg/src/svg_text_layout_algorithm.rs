#![allow(non_snake_case)]

use crate::layout_svg_inline_text::LayoutSVGInlineText;
use crate::layout_svg_text_path::{
    LayoutSVGTextPath, PathPositionMapper, PointAndTangent, PositionType,
};
use foundation::style_constants::ETextAnchor;
use foundation::{
    gfx, DynamicTo, IsHorizontalWritingMode, LayoutUnit, MakeGarbageCollected, PhysicalDirection,
    PhysicalRect, PhysicalSize, StringView, TextDirection, To, WritingDirectionMode, WritingMode,
};
use layoutng_assembly::fragment_item::{ItemType, TextFragmentRareData};
use layoutng_assembly::fragment_items_builder::{FragmentItemsBuilder, ItemWithOffsetList};
use layoutng_assembly::internal::inline_node::InlineNode;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::resolved_text_layout_attributes_iterator::ResolvedTextLayoutAttributesIterator;
use layoutng_assembly::internal::svg_inline_node_data::SvgTextContentRange;
use layoutng_assembly::internal::svg_length_adjust_type::SVGLengthAdjustType;

fn ClampToFloat(value: f32) -> f32 {
    value.clamp(f32::MIN, f32::MAX)
}

fn ClampDoubleToFloat(value: f64) -> f32 {
    if value >= f32::MAX as f64 {
        f32::MAX
    } else if value <= f32::MIN as f64 {
        f32::MIN
    } else {
        value as f32
    }
}

// cpp: layoutng_svg/svg_text_layout_algorithm.h:76-92
#[derive(Clone)]
struct SvgPerCharacterInfo {
    x: Option<f32>,
    y: Option<f32>,
    rotate: Option<f32>,
    hidden: bool,
    middle: bool,
    anchored_chunk: bool,
    in_text_path: bool,
    text_length_resolved: bool,
    baseline_shift: f32,
    inline_size: f32,
    length_adjust_scale: f32,
    text_length_shift_x: f32,
    text_length_shift_y: f32,
    item_index: usize,
}

impl Default for SvgPerCharacterInfo {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            rotate: None,
            hidden: false,
            middle: false,
            anchored_chunk: false,
            in_text_path: false,
            text_length_resolved: false,
            baseline_shift: 0.0,
            inline_size: 0.0,
            length_adjust_scale: 1.0,
            text_length_shift_x: 0.0,
            text_length_shift_y: 0.0,
            item_index: usize::MAX,
        }
    }
}

// cpp: layoutng_svg/svg_text_layout_algorithm.h:21-30,64-106
pub struct SvgTextLayoutAlgorithm {
    inline_node: InlineNode,
    addressable_count: usize,
    horizontal: bool,
    inline_direction: PhysicalDirection,
    result: Vec<SvgPerCharacterInfo>,
    css_positions: Vec<gfx::PointF>,
}

impl SvgTextLayoutAlgorithm {
    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:25-35
    pub fn new(node: InlineNode, writing_mode: WritingMode) -> Self {
        assert!(node.IsSvgText());
        Self {
            inline_node: node,
            addressable_count: 0,
            horizontal: IsHorizontalWritingMode(writing_mode),
            inline_direction: WritingDirectionMode::new(writing_mode, TextDirection::kLtr)
                .InlineEnd(),
            result: Vec::new(),
            css_positions: Vec::new(),
        }
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:36-89
    pub fn Layout(
        &mut self,
        builder: &FragmentItemsBuilder,
        items: &mut ItemWithOffsetList,
    ) -> PhysicalSize {
        if !self.Setup(builder.TextContentLengthMax()) {
            return PhysicalSize::default();
        }
        self.SetFlags(builder, items);
        if self.addressable_count == 0 {
            return PhysicalSize::default();
        }
        let mut iterator =
            ResolvedTextLayoutAttributesIterator::new(self.inline_node.SvgCharacterDataList());
        for index in 0..self.result.len() {
            let resolved = iterator.AdvanceTo(index as u32);
            if resolved.HasRotate() {
                self.result[index].rotate = Some(resolved.rotate);
            }
            if resolved.anchored_chunk {
                self.result[index].anchored_chunk = true;
            }
        }
        self.AdjustPositionsDxDy(items);
        self.ApplyTextLengthAttribute(items);
        self.AdjustPositionsXY(items);
        self.ApplyAnchoring(items);
        self.PositionOnPath(items);
        self.WriteBackToFragmentItems(items)
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:90-112
    fn Setup(&mut self, approximate_count: u32) -> bool {
        if approximate_count == 0 {
            return false;
        }
        self.result.reserve(approximate_count as usize);
        self.css_positions.reserve(approximate_count as usize);
        true
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:113-192
    fn SetFlags(&mut self, builder: &FragmentItemsBuilder, items: &ItemWithOffsetList) {
        let mut indexes: Vec<usize> = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (item.item.Type() == ItemType::kText).then_some(index))
            .collect();
        if self.inline_node.IsBidiEnabled() {
            indexes.sort_by_key(|&index| items[index].item.StartOffset());
        }
        let mut found_first_character = false;
        for index in indexes {
            let item = &items[index].item;
            if item.TextLength() == 0 {
                continue;
            }
            let mut info = SvgPerCharacterInfo::default();
            info.item_index = index;
            if !found_first_character {
                found_first_character = true;
                info.anchored_chunk = true;
            }
            let offset = items[index].offset;
            let font_data = item.ScaledFont().PrimaryFont();
            let ascent = if font_data.is_null() {
                0.0
            } else {
                unsafe { &*font_data }
                    .GetFontMetrics()
                    .FixedAscent(item.Style().GetFontBaseline())
                    .ToFloat()
            };
            let inline_offset = offset.inline_offset.ToFloat();
            let block_offset = offset.block_offset.ToFloat();
            let position = if self.IsHorizontal() {
                gfx::PointF::new(inline_offset, block_offset + ascent)
            } else if self.IsVerticalDownward() {
                gfx::PointF::new(-(block_offset + ascent), inline_offset)
            } else {
                gfx::PointF::new(block_offset + ascent, -inline_offset)
            };
            self.css_positions.push(position);
            let size = item.Size();
            info.inline_size = if self.horizontal {
                size.width.ToFloat()
            } else {
                size.height.ToFloat()
            };
            self.result.push(info.clone());

            let text = builder.TextContent(item.UsesFirstLineStyle());
            let view =
                StringView::from_blink_string_range(text, item.StartOffset(), item.TextLength());
            let mut character_offset = view.NextCodePointOffset(0);
            while character_offset < view.length() {
                let mut middle = SvgPerCharacterInfo::default();
                middle.middle = true;
                middle.item_index = index;
                self.result.push(middle);
                self.css_positions.push(position);
                character_offset = view.NextCodePointOffset(character_offset);
            }
        }
        self.addressable_count = self.result.len();
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:193-237
    fn AdjustPositionsDxDy(&mut self, items: &ItemWithOffsetList) {
        let mut shift = gfx::PointF::default();
        let mut iterator =
            ResolvedTextLayoutAttributesIterator::new(self.inline_node.SvgCharacterDataList());
        for index in 0..self.addressable_count {
            let resolved = iterator.AdvanceTo(index as u32);
            if resolved.HasX() {
                shift.set_x(0.0);
            }
            if resolved.HasY() {
                shift.set_y(0.0);
            }
            if self.IsFirstCharacterInTextPath(index) {
                shift.set_x(0.0);
                shift.set_y(0.0);
            }
            shift.set_x(shift.x() + if resolved.HasDx() { resolved.dx } else { 0.0 });
            shift.set_y(shift.y() + if resolved.HasDy() { resolved.dy } else { 0.0 });
            let scale = self.ScalingFactorAt(items, index);
            self.result[index].x = Some(ClampToFloat(
                self.css_positions[index].x() + shift.x() * scale,
            ));
            self.result[index].y = Some(ClampToFloat(
                self.css_positions[index].y() + shift.y() * scale,
            ));
        }
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:238-259
    fn ApplyTextLengthAttribute(&mut self, items: &ItemWithOffsetList) {
        let mut resolved_descendants = Vec::new();
        let ranges: Vec<_> = self
            .inline_node
            .SvgTextLengthRangeList()
            .iter()
            .cloned()
            .collect();
        for range in &ranges {
            self.ResolveTextLength(items, range, &mut resolved_descendants);
        }
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:260-424
    fn ResolveTextLength(
        &mut self,
        items: &ItemWithOffsetList,
        range: &SvgTextContentRange,
        resolved_descendants: &mut Vec<usize>,
    ) {
        let start = range.start_index as usize;
        let end = range.end_index as usize + 1;
        let object = unsafe { &*range.layout_object.Get() };
        let element = DynamicTo::<Element>(object.GetNode());
        assert!(!element.is_null());
        let element = unsafe { &*element };
        let input = element
            .InputElementData()
            .as_ref()
            .expect("SVG text metadata is missing");
        let length = input.svg_text_length.expect("SVG textLength is missing");
        assert!(length > 0.0);
        let text_length = ClampDoubleToFloat(length * self.ScalingFactorAt(items, start) as f64);
        let length_adjust = element.InputSvgLengthAdjust();
        let mut min_position = f32::INFINITY;
        let mut max_position = f32::NEG_INFINITY;
        for info in &self.result[start..end] {
            let position = if self.IsHorizontal() {
                info.x.unwrap()
            } else if self.IsVerticalDownward() {
                info.y.unwrap()
            } else {
                -info.y.unwrap()
            };
            min_position = min_position.min(position);
            max_position = max_position.max(position + info.inline_size);
        }
        if min_position == f32::INFINITY {
            return;
        }
        let delta = text_length - (max_position - min_position);
        let shift;
        if length_adjust == SVGLengthAdjustType::kSVGLengthAdjustSpacingAndGlyphs {
            if min_position >= max_position {
                return;
            }
            let scale = text_length / (max_position - min_position);
            for info in &mut self.result[start..end] {
                let original_x = info.x.unwrap();
                let original_y = info.y.unwrap();
                if self.inline_direction == PhysicalDirection::kRight {
                    info.x = Some(min_position + (original_x - min_position) * scale);
                } else if self.inline_direction == PhysicalDirection::kDown {
                    info.y = Some(min_position + (original_y - min_position) * scale);
                } else {
                    info.y = Some(-min_position + (original_y + min_position) * scale);
                }
                info.text_length_shift_x += info.x.unwrap() - original_x;
                info.text_length_shift_y += info.y.unwrap() - original_y;
                if !info.middle && !info.text_length_resolved {
                    info.length_adjust_scale = scale;
                    info.inline_size *= scale;
                }
                info.text_length_resolved = true;
            }
            shift = delta;
        } else {
            let unresolved = self.result[start..end]
                .iter()
                .filter(|info| !info.middle && !info.text_length_resolved)
                .count() as isize;
            let descendant_count = resolved_descendants
                .iter()
                .filter(|&&index| start <= index && index < end)
                .count() as isize;
            let count = unresolved + descendant_count - 1;
            let character_delta = if count != 0 {
                delta / count as f32
            } else {
                0.0
            };
            let mut visual_indexes: Vec<usize> = (start..end).collect();
            if self.inline_node.IsBidiEnabled() {
                visual_indexes.sort_by_key(|&index| self.result[index].item_index);
            }
            let mut running_shift = 0.0;
            for index in visual_indexes {
                let info = &mut self.result[index];
                if self.inline_direction == PhysicalDirection::kRight {
                    info.x = Some(info.x.unwrap() + running_shift);
                    info.text_length_shift_x += running_shift;
                } else if self.inline_direction == PhysicalDirection::kDown {
                    info.y = Some(info.y.unwrap() + running_shift);
                    info.text_length_shift_y += running_shift;
                } else {
                    info.y = Some(info.y.unwrap() - running_shift);
                    info.text_length_shift_y -= running_shift;
                }
                if !info.middle
                    && (resolved_descendants.contains(&index) || !info.text_length_resolved)
                {
                    running_shift += character_delta;
                }
                info.text_length_resolved = true;
            }
            shift = running_shift;
        }
        for info in &mut self.result[end..] {
            if info.anchored_chunk {
                break;
            }
            if self.inline_direction == PhysicalDirection::kRight {
                info.x = Some(info.x.unwrap() + shift);
            } else if self.inline_direction == PhysicalDirection::kDown {
                info.y = Some(info.y.unwrap() + shift);
            } else {
                info.y = Some(info.y.unwrap() - shift);
            }
        }
        resolved_descendants.retain(|&index| index < start || index >= end);
        resolved_descendants.push(start);
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:425-498
    fn AdjustPositionsXY(&mut self, items: &ItemWithOffsetList) {
        let mut shift = gfx::PointF::default();
        let mut iterator =
            ResolvedTextLayoutAttributesIterator::new(self.inline_node.SvgCharacterDataList());
        for index in 0..self.result.len() {
            let scale = self.ScalingFactorAt(items, index);
            let resolved = iterator.AdvanceTo(index as u32);
            if resolved.HasX() {
                let mut value = resolved.x * scale
                    - self.css_positions[index].x()
                    - self.result[index].text_length_shift_x;
                if !self.horizontal {
                    value += self.css_positions[index].x();
                }
                shift.set_x(ClampToFloat(value));
            }
            if resolved.HasY() {
                let mut value = resolved.y * scale
                    - self.css_positions[index].y()
                    - self.result[index].text_length_shift_y;
                if self.horizontal {
                    value += self.css_positions[index].y();
                }
                shift.set_y(ClampToFloat(value));
            }
            if self.IsFirstCharacterInTextPath(index) {
                if self.horizontal {
                    shift.set_y(0.0);
                } else {
                    shift.set_x(0.0);
                }
            }
            self.result[index].x = Some(self.result[index].x.unwrap() + shift.x());
            self.result[index].y = Some(self.result[index].y.unwrap() + shift.y());
            if self.result[index].middle && self.result[index].anchored_chunk {
                self.result[index].anchored_chunk = false;
                if index + 1 < self.result.len() {
                    self.result[index + 1].anchored_chunk = true;
                }
            }
        }
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:499-603
    fn ApplyAnchoring(&mut self, items: &ItemWithOffsetList) {
        assert!(!self.result.is_empty());
        assert!(self.result[0].anchored_chunk);
        let mut first = 0usize;
        while first < self.result.len() {
            let following = first + 1;
            let next_anchor = self.result[following..]
                .iter()
                .position(|info| info.anchored_chunk)
                .map_or(self.result.len(), |offset| following + offset);
            let mut last = next_anchor - 1;
            if let Some(range) = self
                .inline_node
                .SvgTextPathRangeList()
                .iter()
                .find(|range| {
                    range.start_index as usize <= first && first <= range.end_index as usize
                })
            {
                last = last.min(range.end_index as usize);
            }
            let mut min_position = f32::INFINITY;
            let mut max_position = f32::NEG_INFINITY;
            for info in &self.result[first..=last] {
                let position = if self.IsHorizontal() {
                    info.x.unwrap()
                } else if self.IsVerticalDownward() {
                    info.y.unwrap()
                } else {
                    -info.y.unwrap()
                };
                min_position = min_position.min(position);
                max_position = max_position.max(position + info.inline_size);
            }
            if min_position != f32::INFINITY {
                let position = if self.IsHorizontal() {
                    self.result[first].x.unwrap()
                } else if self.IsVerticalDownward() {
                    self.result[first].y.unwrap()
                } else {
                    -self.result[first].y.unwrap()
                };
                let style = items[self.result[first].item_index].item.Style();
                let shift = match style.TextAnchor() {
                    ETextAnchor::kStart => {
                        if style.IsLeftToRightDirection() {
                            position - min_position
                        } else {
                            position - max_position
                        }
                    }
                    ETextAnchor::kEnd => {
                        if style.IsLeftToRightDirection() {
                            position - max_position
                        } else {
                            position - min_position
                        }
                    }
                    ETextAnchor::kMiddle => position - (min_position + max_position) / 2.0,
                };
                for info in &mut self.result[first..=last] {
                    if self.inline_direction == PhysicalDirection::kRight {
                        info.x = Some(info.x.unwrap() + shift);
                    } else if self.inline_direction == PhysicalDirection::kDown {
                        info.y = Some(info.y.unwrap() + shift);
                    } else {
                        info.y = Some(info.y.unwrap() - shift);
                    }
                }
            }
            first = last + 1;
        }
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:604-817
    fn PositionOnPath(&mut self, items: &ItemWithOffsetList) {
        let ranges: Vec<_> = self
            .inline_node
            .SvgTextPathRangeList()
            .iter()
            .cloned()
            .collect();
        if ranges.is_empty() {
            return;
        }
        let mut range_index = 0usize;
        let mut in_path_index = usize::MAX;
        let mut path_mapper: Option<PathPositionMapper> = None;
        let mut in_path = false;
        let mut after_path = false;
        let mut path_end_x = 0.0;
        let mut path_end_y = 0.0;
        for index in 0..self.result.len() {
            let is_in_range = range_index < ranges.len()
                && index >= ranges[range_index].start_index as usize
                && index <= ranges[range_index].end_index as usize;
            if is_in_range {
                if !in_path || in_path_index != range_index {
                    let object = To::<LayoutSVGTextPath>(ranges[range_index].layout_object.Get());
                    path_mapper = unsafe { &*object }.LayoutPath();
                }
                in_path = true;
                in_path_index = range_index;
                self.result[index].in_text_path = true;
                if !self.result[index].middle {
                    let scale = self.ScalingFactorAt(items, index);
                    let info = &mut self.result[index];
                    if let Some(mapper) = path_mapper.as_mut() {
                        let char_offset = if self.inline_direction == PhysicalDirection::kRight {
                            info.x.unwrap()
                        } else if self.inline_direction == PhysicalDirection::kDown {
                            info.y.unwrap()
                        } else {
                            -info.y.unwrap()
                        };
                        let mid =
                            (char_offset + info.inline_size / 2.0) / scale + mapper.StartOffset();
                        if !info.hidden {
                            let mut point_tangent = PointAndTangent::default();
                            let position = mapper.PointAndNormalAtLength(mid, &mut point_tangent);
                            if position != PositionType::kOnPath {
                                info.hidden = true;
                            }
                            point_tangent.tangent_in_degrees += info.rotate.unwrap_or(0.0);
                            if self.inline_direction == PhysicalDirection::kDown {
                                point_tangent.tangent_in_degrees -= 90.0;
                            } else if self.inline_direction == PhysicalDirection::kUp {
                                point_tangent.tangent_in_degrees += 90.0;
                            }
                            info.rotate = Some(point_tangent.tangent_in_degrees);
                            if info.rotate == Some(0.0) {
                                if self.inline_direction == PhysicalDirection::kRight {
                                    info.x = Some(
                                        point_tangent.point.x() * scale - info.inline_size / 2.0,
                                    );
                                    info.y =
                                        Some(point_tangent.point.y() * scale + info.y.unwrap());
                                } else if self.inline_direction == PhysicalDirection::kDown {
                                    info.x =
                                        Some(point_tangent.point.x() * scale + info.x.unwrap());
                                    info.y = Some(
                                        point_tangent.point.y() * scale - info.inline_size / 2.0,
                                    );
                                } else {
                                    info.x =
                                        Some(point_tangent.point.x() * scale + info.x.unwrap());
                                    info.y = Some(
                                        point_tangent.point.y() * scale + info.inline_size / 2.0,
                                    );
                                }
                            } else {
                                info.baseline_shift =
                                    if self.inline_direction == PhysicalDirection::kRight {
                                        info.y.unwrap()
                                    } else if self.inline_direction == PhysicalDirection::kDown {
                                        info.x.unwrap()
                                    } else {
                                        -info.x.unwrap()
                                    };
                                info.x = Some(point_tangent.point.x() * scale);
                                info.y = Some(point_tangent.point.y() * scale);
                            }
                            info.x = Some(ClampToFloat(info.x.unwrap()));
                            info.y = Some(ClampToFloat(info.y.unwrap()));
                        }
                    } else {
                        info.hidden = true;
                    }
                } else {
                    let previous = self.result[index - 1].clone();
                    self.result[index].x = previous.x;
                    self.result[index].y = previous.y;
                    self.result[index].rotate = previous.rotate;
                }
            } else {
                if in_path {
                    in_path = false;
                    after_path = true;
                    if let Some(mapper) = path_mapper.as_mut() {
                        let scale = self.ScalingFactorAt(items, index);
                        let mut point_tangent = PointAndTangent::default();
                        mapper.PointAndNormalAtLength(mapper.length(), &mut point_tangent);
                        path_end_x = ClampToFloat(
                            point_tangent.point.x() * scale - self.result[index].x.unwrap(),
                        );
                        path_end_y = ClampToFloat(
                            point_tangent.point.y() * scale - self.result[index].y.unwrap(),
                        );
                    } else {
                        if let Some(info) = self.result[index..]
                            .iter()
                            .rev()
                            .find(|info| !info.hidden && !info.middle)
                        {
                            if self.IsHorizontal() {
                                path_end_x = info.x.unwrap() + info.inline_size;
                                path_end_y = info.y.unwrap();
                            } else if self.IsVerticalDownward() {
                                path_end_x = info.x.unwrap();
                                path_end_y = info.y.unwrap() + info.inline_size;
                            } else {
                                path_end_x = info.x.unwrap();
                                path_end_y = info.y.unwrap() - info.inline_size;
                            }
                        } else {
                            path_end_x = 0.0;
                            path_end_y = 0.0;
                        }
                        path_end_x -= self.result[index].x.unwrap();
                        path_end_y -= self.result[index].y.unwrap();
                    }
                }
                if after_path {
                    if self.result[index].anchored_chunk {
                        after_path = false;
                    } else {
                        self.result[index].x = Some(self.result[index].x.unwrap() + path_end_x);
                        self.result[index].y = Some(self.result[index].y.unwrap() + path_end_y);
                    }
                }
            }
            if range_index < ranges.len() && index == ranges[range_index].end_index as usize {
                range_index += 1;
            }
        }
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:818-885
    fn WriteBackToFragmentItems(&self, items: &mut ItemWithOffsetList) -> PhysicalSize {
        let mut visual_rect = gfx::RectF::default();
        for info in &self.result {
            if info.middle {
                continue;
            }
            let item = &mut items[info.item_index].item;
            let layout_object = To::<LayoutSVGInlineText>(item.GetLayoutObject());
            let layout_object = unsafe { &*layout_object };
            let font_data = layout_object.ScaledFont().PrimaryFont();
            let (ascent, descent) = if font_data.is_null() {
                (0.0, 0.0)
            } else {
                let metrics = unsafe { &*font_data }.GetFontMetrics();
                let baseline = item.Style().GetFontBaseline();
                (
                    metrics.FixedAscent(baseline).ToFloat(),
                    metrics.FixedDescent(baseline).ToFloat(),
                )
            };
            let mut x = info.x.unwrap();
            let mut y = info.y.unwrap();
            let size = item.Size();
            let (width, height) = if self.IsHorizontal() {
                y -= ascent;
                (info.inline_size, size.height.ToFloat())
            } else if self.IsVerticalDownward() {
                x -= descent;
                (size.width.ToFloat(), info.inline_size)
            } else {
                x -= ascent;
                y -= info.inline_size;
                (size.width.ToFloat(), info.inline_size)
            };
            let scaled_rect = gfx::RectF::new(
                gfx::PointF::new(ClampToFloat(x), ClampToFloat(y)),
                gfx::SizeF::new(ClampToFloat(width), ClampToFloat(height)),
            );
            let scale = layout_object.ScalingFactor();
            assert_ne!(scale, 0.0);
            let mut unscaled_rect = scaled_rect;
            unscaled_rect.Scale(1.0 / scale, 1.0 / scale);
            let mut data = TextFragmentRareData::default();
            data.rect = scaled_rect;
            data.length_adjust_scale = info.length_adjust_scale;
            data.is_svg = true;
            data.angle = info.rotate.unwrap_or(0.0);
            data.baseline_shift = info.baseline_shift;
            data.in_text_path = info.in_text_path;
            let data = MakeGarbageCollected(data);
            item.SetSvgFragmentData(
                data,
                PhysicalRect::EnclosingRect(&unscaled_rect),
                info.hidden,
            );
            let mut transformed = if item.HasSvgTransformForBoundingBox() {
                item.BuildSvgTransformForBoundingBox().MapRect(scaled_rect)
            } else {
                scaled_rect
            };
            transformed.Scale(1.0 / scale, 1.0 / scale);
            visual_rect.Union(transformed);
        }
        if items[0].item.Type() == ItemType::kLine {
            let left = visual_rect.x().floor() as i32;
            let top = visual_rect.y().floor() as i32;
            let right = if visual_rect.width() == 0.0 {
                left
            } else {
                visual_rect.right().ceil() as i32
            };
            let bottom = if visual_rect.height() == 0.0 {
                top
            } else {
                visual_rect.bottom().ceil() as i32
            };
            let pixel_rect = gfx::Rect::new(
                gfx::Point::new(left, top),
                gfx::Size::new(right.saturating_sub(left), bottom.saturating_sub(top)),
            );
            items[0]
                .item
                .SetSvgLineLocalRect(PhysicalRect::from(pixel_rect));
        }
        PhysicalSize::new(
            LayoutUnit::from_f64(visual_rect.right() as f64),
            LayoutUnit::from_f64(visual_rect.bottom() as f64),
        )
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.h:52-59
    fn IsHorizontal(&self) -> bool {
        self.inline_direction == PhysicalDirection::kRight
    }
    fn IsVerticalDownward(&self) -> bool {
        self.inline_direction == PhysicalDirection::kDown
    }
    fn IsVerticalUpward(&self) -> bool {
        self.inline_direction == PhysicalDirection::kUp
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:886-891
    fn ScalingFactorAt(&self, items: &ItemWithOffsetList, index: usize) -> f32 {
        items[self.result[index].item_index].item.SvgScalingFactor()
    }

    // cpp: layoutng_svg/svg_text_layout_algorithm.cc:892-902
    fn IsFirstCharacterInTextPath(&self, index: usize) -> bool {
        self.result[index].anchored_chunk
            && self
                .inline_node
                .SvgTextPathRangeList()
                .iter()
                .any(|range| range.start_index as usize == index)
    }
}
