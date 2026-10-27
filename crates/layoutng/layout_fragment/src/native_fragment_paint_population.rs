//! Native physical-fragment and inline-item paint population.
//!
//! This assembly-only fragment owner retains reciprocal native types while
//! placing the two recursive hot paths under `layout_fragment` symbols.

#![allow(non_snake_case)]

use super::*;

fn PopulateInlineItemLocalPaintData(
    item: &FragmentItem,
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    parent_offset: PhysicalOffset,
    container_absolute_offset: PhysicalOffset,
    cull: Option<PaintCullRect>,
    containing_line_offset: Option<PhysicalOffset>,
    containing_line_height: Option<LayoutUnit>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state_owners: &mut FragmentOwners,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) {
    let object = item.GetLayoutObject();
    let fragment_instance_id = paint_state_owners.fragment_instance(
        1,
        item as *const FragmentItem as usize,
        next_fragment_instance_id,
    );
    let effective_style = if item.UsesFirstLineStyle() {
        item.Style() as *const NativeComputedStyle
    } else {
        std::ptr::null()
    };
    let mut output = FragmentPaintData::new(ExportProperties(
        object,
        sources,
        style_sources,
        effective_style,
        resources,
        purpose,
        Some(&mut paint_state_owners.exported_styles),
        Some(&mut paint_state_owners.debug_names),
    ));
    output.fragment_instance_id = fragment_instance_id;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    output.node_id = InputId(object);
    // cpp: core/paint/box_fragment_painter.cc:2025-2026,2071-2072
    // cpp: core/paint/inline_box_fragment_painter.cc:63-64
    output.paint.display_item_fragment = item.FragmentId();
    output.paint.hidden =
        item.IsHiddenForPaint() || output.paint.svg_viewport.is_some_and(|v| v.disabled);
    let rect = *item.RectInContainerFragment();
    output.offset = Offset {
        x: Number(rect.offset.left - parent_offset.left),
        y: Number(rect.offset.top - parent_offset.top),
    };
    output.size = Size {
        width: Number(rect.size.width),
        height: Number(rect.size.height),
    };
    {
        output.content_size = output.size;
    }
    if purpose == LayoutUpdateKind::Paint && item.Type() == ItemType::kBox && !object.is_null() {
        let native_style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        ExportNativeNinePieceImages(native_style, output.size, resources, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportInlineInkOverflow(item, &mut output.paint);
    }
    {
        ExportSvgTextPaintData(item, rect, &mut output.paint);
    }
    match item.Type() {
        ItemType::kText | ItemType::kGeneratedText => {
            output.kind = if item.Type() == ItemType::kText {
                FragmentKind::kText
            } else {
                FragmentKind::kGeneratedText
            };
            if let Some(line_offset) = containing_line_offset {
                output.paint.text_line_top_offset = Some(Number(line_offset.top - rect.offset.top));
                if TraceInlineExport() {
                    eprintln!(
                        "inline-export node={} item={:.4},{:.4} line={:.4},{:.4} parent={:.4},{:.4} local={:.4},{:.4}",
                        output.node_id,
                        rect.offset.left.ToDouble(),
                        rect.offset.top.ToDouble(),
                        line_offset.left.ToDouble(),
                        line_offset.top.ToDouble(),
                        parent_offset.left.ToDouble(),
                        parent_offset.top.ToDouble(),
                        output.offset.x,
                        output.offset.y,
                    );
                }
            }
            output.paint.text_line_height = containing_line_height.map(Number);
            output.text_start = Some(item.StartOffset());
            output.text_end = Some(item.EndOffset());
            if purpose == LayoutUpdateKind::Paint {
                ExportListMarkerSymbol(item, &mut output.paint);
                ExportTextCombineInlineOffset(item, &mut output.paint);
            }
        }
        ItemType::kLine => {
            output.kind = FragmentKind::kLine;
            output.node_id = 0;
        }
        ItemType::kBox => {
            output.kind = FragmentKind::kBox;
            let box_fragment = item.BoxFragment();
            if !box_fragment.is_null() {
                let box_fragment = unsafe { &*box_fragment };
                {
                    ExportBoxPaintGeometry(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint
                    && cull.is_some()
                    && output.paint.scroll_container.is_some()
                {
                    output.paint_cull_scroll_offset = Some(output.paint.scroll_offset);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportOverflowClipMargin(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportScrollbarPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportTablePaintData(box_fragment, sources, &mut output.paint);
                }
                if box_fragment.IsTableCell() {
                    output.paint.table_cell_column =
                        Some(box_fragment.TableCellColumnIndex() as usize);
                }
                output.paint.painted_atomically = box_fragment.IsPaintedAtomically();
                output.paint.has_collapsed_borders = box_fragment.HasCollapsedBorders();
                let owner_fragment = item.PostLayoutBoxFragment();
                let owner_fragment = if owner_fragment.is_null() {
                    box_fragment as *const PhysicalBoxFragment
                } else {
                    owner_fragment
                };
                output.paint.establishes_paint_state =
                    paint_state_owners.insert(owner_fragment, output.fragment_instance_id);
                paint_state_owners
                    .instances
                    .entry(box_fragment as *const PhysicalBoxFragment)
                    .or_insert(output.fragment_instance_id);
                {
                    output.content_size = ExportContentSize(box_fragment);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportMathMLPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportFrameSetPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportFieldsetPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportColumnRulePaintData(box_fragment, &mut output.paint);
                }
                if item.DescendantsCount() <= 1 {
                    let item_absolute_offset =
                        AddPhysicalOffsets(container_absolute_offset, rect.offset);
                    let nested_items = box_fragment.Items();
                    for child in box_fragment.Children() {
                        if !child.is_present() || (!nested_items.is_null() && child.IsLineBox()) {
                            continue;
                        }
                        PopulatePhysicalFragmentPaintData(
                            child,
                            child.Offset(),
                            AddPhysicalOffsets(
                                SubtractScrollOffset(
                                    item_absolute_offset,
                                    output.paint.scroll_offset,
                                ),
                                child.Offset(),
                            ),
                            cull,
                            false,
                            sources,
                            style_sources,
                            resources,
                            purpose,
                            paint_state_owners,
                            fragmentainer_instance_id,
                            item.FragmentId(),
                            next_fragment_instance_id,
                            next_fragmentainer_instance_id,
                        );
                    }
                    if !nested_items.is_null() {
                        let span = unsafe { &*nested_items }.Items();
                        let mut index = 0;
                        while index < span.len() {
                            index = PopulateInlineItemPaintData(
                                span,
                                unsafe { &*nested_items },
                                index,
                                span.len(),
                                PhysicalOffset::default(),
                                item_absolute_offset,
                                cull,
                                false,
                                None,
                                None,
                                sources,
                                style_sources,
                                resources,
                                purpose,
                                paint_state_owners,
                                fragmentainer_instance_id,
                                next_fragment_instance_id,
                                next_fragmentainer_instance_id,
                            );
                        }
                    }
                }
                PopulateSVGChildPaintData(
                    object,
                    sources,
                    style_sources,
                    resources,
                    purpose,
                    paint_state_owners,
                    fragmentainer_instance_id,
                    item.FragmentId(),
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                );
            }
        }
        ItemType::kInvalid => unreachable!("invalid inline item"),
    }
    let item_index =
        unsafe { (item as *const FragmentItem).offset_from(fragment_items.Items().as_ptr()) };
    assert!(item_index >= 0);
    fragment_items.SetPaintInput(item_index as usize, output);
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1850-1889
#[inline(never)]
fn PopulateInlineItemPaintData(
    items: &[FragmentItem],
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    index: usize,
    limit: usize,
    parent_offset: PhysicalOffset,
    container_absolute_offset: PhysicalOffset,
    cull: Option<PaintCullRect>,
    mut containing_line_culled: bool,
    containing_line_offset: Option<PhysicalOffset>,
    containing_line_height: Option<LayoutUnit>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state_owners: &mut FragmentOwners,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> usize {
    assert!(index < limit);
    let item = &items[index];
    let subtree_size = 1_usize.max(item.DescendantsCount() as usize);
    let subtree_end = limit.min(index + subtree_size);
    let starts_line = containing_line_offset.is_none() && item.Type() == ItemType::kLine;
    if purpose == LayoutUpdateKind::Paint && starts_line {
        if let Some(cull) = cull {
            let rect = item.RectInContainerFragment();
            let outside = if item.IsHorizontal() {
                let top = Number(container_absolute_offset.top + rect.offset.top);
                !cull.intersects_vertical_range(top, top + Number(rect.size.height))
            } else {
                let left = Number(container_absolute_offset.left + rect.offset.left);
                !cull.intersects_horizontal_range(left, left + Number(rect.size.width))
            };
            let has_escaping_descendant = items[index..subtree_end].iter().any(|descendant| {
                let object = descendant.GetLayoutObject();
                !object.is_null() && paint_state_owners.uncullable_objects.contains(&object)
            });
            if outside && !has_escaping_descendant {
                containing_line_culled = true;
            }
        }
    }
    if containing_line_culled && matches!(item.Type(), ItemType::kText | ItemType::kGeneratedText) {
        fragment_items.ClearPaintInputs(index..subtree_end);
        return subtree_end;
    }
    let line_offset = containing_line_offset
        .or_else(|| (item.Type() == ItemType::kLine).then(|| *item.OffsetInContainerFragment()));
    let line_height = containing_line_height
        .or_else(|| (item.Type() == ItemType::kLine).then(|| item.Size().height));
    PopulateInlineItemLocalPaintData(
        item,
        fragment_items,
        parent_offset,
        container_absolute_offset,
        cull,
        line_offset,
        line_height,
        sources,
        style_sources,
        resources,
        purpose,
        paint_state_owners,
        fragmentainer_instance_id,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    let mut child_index = index + 1;
    while child_index < subtree_end {
        child_index = PopulateInlineItemPaintData(
            items,
            fragment_items,
            child_index,
            subtree_end,
            *item.OffsetInContainerFragment(),
            container_absolute_offset,
            cull,
            containing_line_culled,
            line_offset,
            line_height,
            sources,
            style_sources,
            resources,
            purpose,
            paint_state_owners,
            fragmentainer_instance_id,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
    }
    subtree_end
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1891-1967
#[inline(never)]
pub(super) fn PopulatePhysicalFragmentPaintData(
    fragment: &PhysicalFragment,
    offset: PhysicalOffset,
    absolute_offset: PhysicalOffset,
    cull: Option<PaintCullRect>,
    is_root: bool,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state_owners: &mut FragmentOwners,
    mut fragmentainer_instance_id: u64,
    mut display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) {
    let object = fragment.GetLayoutObject();
    let kind = if fragment.IsBox() {
        FragmentKind::kBox
    } else {
        FragmentKind::kLine
    };
    let fragment_instance_id = paint_state_owners.fragment_instance(
        0,
        fragment as *const PhysicalFragment as usize,
        next_fragment_instance_id,
    );
    let effective_style = if fragment.UsesFirstLineStyle() {
        fragment.Style() as *const NativeComputedStyle
    } else {
        std::ptr::null()
    };
    let mut output = FragmentPaintData::new(ExportProperties(
        object,
        sources,
        style_sources,
        effective_style,
        resources,
        purpose,
        Some(&mut paint_state_owners.exported_styles),
        Some(&mut paint_state_owners.debug_names),
    ));
    output.fragment_instance_id = fragment_instance_id;
    output.node_id = InputId(object);
    output.paint.display_item_fragment = display_item_fragment;
    output.paint.hidden =
        fragment.IsHiddenForPaint() || output.paint.svg_viewport.is_some_and(|v| v.disabled);
    output.kind = kind;
    output.offset = Offset {
        x: Number(offset.left),
        y: Number(offset.top),
    };
    output.size = Size {
        width: Number(fragment.Size().width),
        height: Number(fragment.Size().height),
    };
    {
        output.content_size = output.size;
    }
    {
        output.fragmentainer_instance_id = fragmentainer_instance_id;
    }
    if !fragment.IsBox() {
        let fragment_data = fragment.GetFragmentData();
        if let Some(fragment_data) = unsafe { fragment_data.as_ref() } {
            fragment_data.SetPaintInput(output);
        }
        return;
    }
    let box_fragment = To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
    let box_fragment = unsafe { &*box_fragment };
    if purpose == LayoutUpdateKind::Paint && !object.is_null() {
        let native_style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        ExportNativeNinePieceImages(native_style, output.size, resources, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportBoxInkOverflow(box_fragment, &mut output.paint);
    }
    if box_fragment.IsFragmentainerBox() {
        fragmentainer_instance_id = *next_fragmentainer_instance_id;
        *next_fragmentainer_instance_id += 1;
        // cpp: core/paint/box_fragment_painter.cc:398-403,1211-1212
        let token = box_fragment.GetBreakToken();
        display_item_fragment = if token.is_null() {
            0
        } else {
            unsafe { &*token }.SequenceNumber() + 1
        };
    } else if !object.is_null() && unsafe { &*object }.HasLayer() {
        // PaintLayerPainter scopes each physical fragment using its index in
        // the owning LayoutBox, independently of exported preorder numbering.
        // cpp: core/paint/paint_layer_painter.cc:776-778
        let layout_box = DynamicTo::<LayoutBox>(object);
        if !layout_box.is_null() {
            let layout_box = unsafe { &*layout_box };
            if let Some(index) = (0..layout_box.PhysicalFragmentCount())
                .find(|&index| layout_box.GetPhysicalFragment(index) == box_fragment as *const _)
            {
                display_item_fragment = index as u32;
            }
        }
    }
    output.paint.display_item_fragment = display_item_fragment;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    {
        ExportBoxPaintGeometry(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint
        && cull.is_some()
        && output.paint.scroll_container.is_some()
    {
        output.paint_cull_scroll_offset = Some(output.paint.scroll_offset);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportOverflowClipMargin(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportScrollbarPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportTablePaintData(box_fragment, sources, &mut output.paint);
    }
    if box_fragment.IsTableCell() {
        output.paint.table_cell_column = Some(box_fragment.TableCellColumnIndex() as usize);
    }
    output.paint.painted_atomically = fragment.IsPaintedAtomically();
    output.paint.has_collapsed_borders = fragment.HasCollapsedBorders();
    output.paint.establishes_paint_state = paint_state_owners.insert(
        box_fragment as *const PhysicalBoxFragment,
        output.fragment_instance_id,
    );
    {
        output.content_size = ExportContentSize(box_fragment);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportMathMLPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportFrameSetPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportFieldsetPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportColumnRulePaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportCollapsedTablePaintData(box_fragment, style_sources, &mut output.paint);
    }
    let items = box_fragment.Items();
    // The top-level cull rect is expressed in the root scroller's contents
    // coordinates, so retain that scroll there. Descendant scrollers map
    // their contents back into the same coordinate space by subtracting the
    // live local scroll translation before culling.
    let contents_absolute_offset = if is_root {
        absolute_offset
    } else {
        SubtractScrollOffset(absolute_offset, output.paint.scroll_offset)
    };
    for child in box_fragment.Children() {
        if !child.is_present() || (!items.is_null() && child.IsLineBox()) {
            continue;
        }
        let child_absolute_offset = AddPhysicalOffsets(contents_absolute_offset, child.Offset());
        if cull.is_some_and(|cull| {
            ShouldCullPhysicalFragment(child, child_absolute_offset, cull, paint_state_owners)
        }) {
            if let Some(data) = unsafe { child.GetFragmentData().as_ref() } {
                data.ClearPaintInput();
            }
            continue;
        }
        PopulatePhysicalFragmentPaintData(
            child,
            child.Offset(),
            child_absolute_offset,
            cull,
            false,
            sources,
            style_sources,
            resources,
            purpose,
            paint_state_owners,
            fragmentainer_instance_id,
            display_item_fragment,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
    }
    if !items.is_null() {
        let span = unsafe { &*items }.Items();
        let mut index = 0;
        while index < span.len() {
            index = PopulateInlineItemPaintData(
                span,
                unsafe { &*items },
                index,
                span.len(),
                PhysicalOffset::default(),
                contents_absolute_offset,
                cull,
                false,
                None,
                None,
                sources,
                style_sources,
                resources,
                purpose,
                paint_state_owners,
                fragmentainer_instance_id,
                next_fragment_instance_id,
                next_fragmentainer_instance_id,
            );
        }
    }
    PopulateSVGChildPaintData(
        object,
        sources,
        style_sources,
        resources,
        purpose,
        paint_state_owners,
        fragmentainer_instance_id,
        display_item_fragment,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    let fragment_data = fragment.GetFragmentData();
    if let Some(fragment_data) = unsafe { fragment_data.as_ref() } {
        fragment_data.SetPaintInput(output);
    }
}
