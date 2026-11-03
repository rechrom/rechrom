//! Chromium physical-fragment and inline-item paint population.
//!
//! This assembly-only fragment owner retains reciprocal source types while
//! placing the two recursive hot paths under `layout_fragment` symbols.

#![allow(non_snake_case)]

use super::*;
use crate::inline_cursor::InlineCursor;

// Chromium: core/paint/box_fragment_painter.cc:102-127.  Line truncation
// marks the FragmentItem in the inline formatting context, while a
// self-painting atomic inline is later entered through its stored physical
// fragment.  Export the effective IsHiddenForPaint result once at the source
// Layout-to-Paint boundary so the paint packages do not have to recreate an
// InlineCursor from flattened data.
fn IsHiddenForPaintIncludingInlineItem(fragment: &PhysicalFragment) -> bool {
    if fragment.IsHiddenForPaint() {
        return true;
    }
    if !fragment.IsAtomicInline() || !fragment.HasSelfPaintingLayer() {
        return false;
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() || !unsafe { &*object }.IsInLayoutNGInlineFormattingContext() {
        return false;
    }
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(unsafe { &*object });
    cursor.IsNotNull() && cursor.Current().IsHiddenForPaint()
}

// Chromium grows and shrinks a LayoutObject's FragmentData list while
// PrePaint walks its physical fragments (core/paint/pre_paint_tree_walk.cc,
// GetOrCreateFragmentData). This publication walk is Rechrom's equivalent
// source-fragment traversal, so establish the same one-entry-per-physical-
// fragment invariant before asking a fragment for its resident data.
fn SynchronizeBoxFragmentData(fragment: &PhysicalFragment) {
    if !fragment.IsBox() {
        return;
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    let Some(layout_box) = (unsafe { layout_box.as_ref() }) else {
        return;
    };
    let physical_count = layout_box.PhysicalFragmentCount() as u32;
    debug_assert_ne!(physical_count, 0);

    let owner = unsafe { &*object };
    let old_count = owner.GetMutableForPainting().FragmentList().size();
    if old_count == physical_count {
        return;
    }

    let mut mutable = owner.GetMutableForPainting();
    mutable.FragmentCountChanged();
    if old_count < physical_count {
        while mutable.FragmentList().size() < physical_count {
            mutable.FragmentList().AppendNewFragment();
        }
        // Chromium forces paint-property construction for newly added entries.
        mutable.SetOnlyThisNeedsPaintPropertyUpdate();
    } else {
        mutable.FragmentList().Shrink(physical_count);
    }
}

fn PopulateInlineItemLocalPaintData(
    item: &FragmentItem,
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    parent_offset: PhysicalOffset,
    containing_line_offset: Option<PhysicalOffset>,
    containing_line_height: Option<LayoutUnit>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
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
    let uses_first_line_style = item.UsesFirstLineStyle();
    let effective_style = if uses_first_line_style {
        if item.Type() == ItemType::kLine && !object.is_null() {
            unsafe { &*object }.FirstLineStyleRef() as *const StyleComputedStyle
        } else {
            item.Style() as *const StyleComputedStyle
        }
    } else {
        std::ptr::null()
    };
    let mut output = FragmentPaintData::new(ExportProperties(
        object,
        std::ptr::null_mut(),
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
    output.paint.is_line_break = item.IsText() && item.IsLineBreak();
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
    if purpose == LayoutUpdateKind::Paint && item.IsText() && !item.IsSvgText() {
        // Source: core/paint/box_fragment_painter.cc:2492-2510 and
        // fragment_item.cc:1090-1109. Preserve LayoutTextCombine's compressed
        // bounds when present; otherwise use FragmentItem's pixel-snapped
        // ordinary point-test rectangle. Export it relative to this resident
        // item because interaction already owns the accumulated parent offset.
        let text_combine = if object.is_null() {
            std::ptr::null_mut()
        } else {
            DynamicTo::<LayoutTextCombine>(unsafe { &*object }.Parent())
        };
        let mut bounds = if text_combine.is_null() {
            item.ComputeTextBoundsRectForHitTest(PhysicalOffset::default(), false)
        } else {
            unsafe { &*text_combine }
                .ComputeTextBoundsRectForHitTest(item, &PhysicalOffset::default())
        };
        bounds.offset -= rect.offset;
        output.paint.text_hit_test_bounds = Some(FragmentPaintRect {
            offset: Offset {
                x: Number(bounds.offset.left),
                y: Number(bounds.offset.top),
            },
            size: Size {
                width: Number(bounds.size.width),
                height: Number(bounds.size.height),
            },
        });
    }
    {
        output.content_size = output.size;
    }
    if purpose == LayoutUpdateKind::Paint && item.Type() == ItemType::kBox && !object.is_null() {
        let style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        ExportNinePieceImages(style, output.size, resources, &mut output.paint);
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
            }
            output.paint.text_line_height = containing_line_height.map(Number);
            output.text_start = Some(item.StartOffset());
            output.text_end = Some(item.EndOffset());
            output.paint.is_ellipsis = item.IsEllipsis();
            output.paint.text_fit_scale = if item.IsSvgText() {
                1.0
            } else {
                f64::from(item.GetTextFitScale())
            };
            if purpose == LayoutUpdateKind::Paint {
                ExportListMarkerSymbol(item, &mut output.paint);
                ExportTextCombineInlineOffset(item, &mut output.paint);
            }
        }
        ItemType::kLine => {
            output.kind = FragmentKind::kLine;
            output.node_id = 0;
            if uses_first_line_style && !object.is_null() {
                let line_style = unsafe { &*effective_style };
                let normal_style = unsafe { &*object }.StyleRef();
                if !std::ptr::eq(line_style, normal_style) {
                    let line_box = item.LineBoxFragment();
                    debug_assert!(!line_box.is_null());
                    let line_box = unsafe { &*line_box };
                    let line_metrics = line_box.Metrics();
                    let text_metrics = line_style.GetFontHeightForDefaultBaseline();
                    output.paint.first_line_decoration_rect =
                        Some(if line_style.IsHorizontalWritingMode() {
                            FragmentPaintRect {
                                offset: Offset {
                                    x: 0.0,
                                    y: Number(line_metrics.ascent - text_metrics.ascent),
                                },
                                size: Size {
                                    width: output.size.width,
                                    height: Number(text_metrics.LineHeight()),
                                },
                            }
                        } else {
                            FragmentPaintRect {
                                offset: Offset {
                                    x: output.size.width
                                        - Number(line_metrics.ascent)
                                        - Number(text_metrics.descent),
                                    y: 0.0,
                                },
                                size: Size {
                                    width: Number(text_metrics.LineHeight()),
                                    height: output.size.height,
                                },
                            }
                        });
                }
            }
        }
        ItemType::kBox => {
            output.kind = FragmentKind::kBox;
            let box_fragment = item.BoxFragment();
            if !box_fragment.is_null() {
                let box_fragment = unsafe { &*box_fragment };
                {
                    ExportBoxPaintGeometry(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint && output.paint.scroll_container.is_some() {
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
                    let nested_items = box_fragment.Items();
                    for child in box_fragment.Children() {
                        if !child.is_present() || (!nested_items.is_null() && child.IsLineBox()) {
                            continue;
                        }
                        PopulatePhysicalFragmentPaintData(
                            child,
                            child.Offset(),
                            sources,
                            style_sources,
                            resources,
                            purpose,
                            paint_state,
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
                                None,
                                None,
                                sources,
                                style_sources,
                                resources,
                                purpose,
                                paint_state,
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
                    paint_state,
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
    paint_state.SetInline(fragment_items, item_index as usize, output);
}

#[inline(never)]
fn PopulateInlineItemPaintData(
    items: &[FragmentItem],
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    index: usize,
    limit: usize,
    parent_offset: PhysicalOffset,
    containing_line_offset: Option<PhysicalOffset>,
    containing_line_height: Option<LayoutUnit>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> usize {
    assert!(index < limit);
    let item = &items[index];
    let subtree_size = 1_usize.max(item.DescendantsCount() as usize);
    let subtree_end = limit.min(index + subtree_size);
    let line_offset = containing_line_offset
        .or_else(|| (item.Type() == ItemType::kLine).then(|| *item.OffsetInContainerFragment()));
    let line_height = containing_line_height
        .or_else(|| (item.Type() == ItemType::kLine).then(|| item.Size().height));
    PopulateInlineItemLocalPaintData(
        item,
        fragment_items,
        parent_offset,
        line_offset,
        line_height,
        sources,
        style_sources,
        resources,
        purpose,
        paint_state,
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
            line_offset,
            line_height,
            sources,
            style_sources,
            resources,
            purpose,
            paint_state,
            paint_state_owners,
            fragmentainer_instance_id,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
    }
    subtree_end
}

#[inline(never)]
pub(super) fn PopulatePhysicalFragmentPaintData(
    fragment: &PhysicalFragment,
    offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    mut fragmentainer_instance_id: u64,
    mut display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) {
    SynchronizeBoxFragmentData(fragment);
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
        fragment.Style() as *const StyleComputedStyle
    } else {
        std::ptr::null()
    };
    let mut output = FragmentPaintData::new(ExportProperties(
        object,
        if fragment.IsBox() {
            fragment.GetFragmentData().cast_mut()
        } else {
            std::ptr::null_mut()
        },
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
    output.paint.hidden = IsHiddenForPaintIncludingInlineItem(fragment)
        || output.paint.svg_viewport.is_some_and(|v| v.disabled);
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
        paint_state.SetPhysical(fragment, output);
        return;
    }
    let box_fragment = To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
    let box_fragment = unsafe { &*box_fragment };
    output.paint.is_page_border_box =
        box_fragment.GetBoxType() == crate::physical_fragment::BoxType::kPageBorderBox;
    output.paint.is_paginated_root = box_fragment.IsPaginatedRoot();
    output.paint.is_inline_formatting_context = box_fragment.IsInlineFormattingContext();
    output.paint.is_first_for_node = box_fragment.IsFirstForNode();
    output.paint.can_paint_multiple_fragments =
        crate::internal::fragmentation_utils::CanPaintMultipleFragmentsForFragment(box_fragment);
    if purpose == LayoutUpdateKind::Paint && !object.is_null() {
        let style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        ExportNinePieceImages(style, output.size, resources, &mut output.paint);
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
    if purpose == LayoutUpdateKind::Paint && output.paint.scroll_container.is_some() {
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
    for child in box_fragment.Children() {
        if !child.is_present() || (!items.is_null() && child.IsLineBox()) {
            continue;
        }
        PopulatePhysicalFragmentPaintData(
            child,
            child.Offset(),
            sources,
            style_sources,
            resources,
            purpose,
            paint_state,
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
                None,
                None,
                sources,
                style_sources,
                resources,
                purpose,
                paint_state,
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
        paint_state,
        paint_state_owners,
        fragmentainer_instance_id,
        display_item_fragment,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    paint_state.SetPhysical(fragment, output);
}
