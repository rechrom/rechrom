#![allow(non_snake_case)]

use foundation::{HeapHashMap, LayoutUnit, Member, PhysicalOffset, PhysicalRect, PhysicalSize, To};
use layoutng_assembly::box_fragment_builder::BoxFragmentBuilder;
use layoutng_assembly::fragment_item::FragmentItem;
use layoutng_assembly::fragment_items_builder::FragmentItemWithOffset;
use layoutng_assembly::internal::inline_containing_block_utils::{
    InlineContainingBlockGeometry, InlineContainingBlockMap,
};
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;
use layoutng_assembly::physical_line_box_fragment::PhysicalLineBoxFragment;
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::ToPhysicalSize;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

// cpp: layoutng_out_of_flow/inline_containing_block_utils.cc:21-25
type LineBoxPair = (
    *const PhysicalLineBoxFragment,
    *const PhysicalLineBoxFragment,
);

trait InlineFragmentItemView {
    fn Item(&self) -> &FragmentItem;
}

impl InlineFragmentItemView for FragmentItem {
    fn Item(&self) -> &FragmentItem {
        self
    }
}

impl InlineFragmentItemView for FragmentItemWithOffset {
    fn Item(&self) -> &FragmentItem {
        &self.item
    }
}

// cpp: layoutng_out_of_flow/inline_containing_block_utils.cc:32-115
fn GatherInlineContainerFragmentsFromItems<Items: InlineFragmentItemView>(
    items: &[Items],
    box_offset: PhysicalOffset,
    inline_containing_block_map: &mut InlineContainingBlockMap,
    containing_linebox_map: &mut HeapHashMap<Member<LayoutObject>, LineBoxPair>,
    fragment_converter: Option<&WritingModeConverter>,
    containing_block_converter: Option<&WritingModeConverter>,
) {
    debug_assert_eq!(
        fragment_converter.is_some(),
        containing_block_converter.is_some()
    );
    let mut linebox: *const PhysicalLineBoxFragment = std::ptr::null();
    for item in items {
        let item = item.Item();
        let current_linebox = item.LineBoxFragment();
        if !current_linebox.is_null() {
            linebox = current_linebox;
            continue;
        }
        let box_fragment = item.BoxFragment();
        if box_fragment.is_null() {
            continue;
        }
        let key =
            Member::from_ptr(unsafe { &*box_fragment }.GetLayoutObject() as *mut LayoutObject);
        let Some(geometry_slot) = inline_containing_block_map.get_mut(&key) else {
            continue;
        };
        let lineboxes = containing_linebox_map
            .entry(key)
            .or_insert((std::ptr::null(), std::ptr::null()));
        debug_assert!(geometry_slot.is_some() || lineboxes.0.is_null());

        let mut fragment_rect = *item.RectInContainerFragment();
        if let (Some(fragment_converter), Some(containing_block_converter)) =
            (fragment_converter, containing_block_converter)
        {
            fragment_rect.offset = containing_block_converter.ToPhysicalOffset(
                fragment_converter.ToLogicalOffset(fragment_rect.offset, fragment_rect.size),
                fragment_rect.size,
            );
        }
        fragment_rect.offset += box_offset;

        if lineboxes.0 == linebox {
            geometry_slot
                .as_mut()
                .expect("inline geometry must exist")
                .start_fragment_union_rect
                .Unite(&fragment_rect);
        } else if lineboxes.0.is_null() {
            debug_assert!(lineboxes.1.is_null());
            lineboxes.0 = linebox;
            lineboxes.1 = linebox;
            let should_hide_abspos = unsafe { &*linebox }.IsHiddenForPaint();
            let relative_offset = geometry_slot
                .as_ref()
                .expect("inline geometry must exist")
                .relative_offset;
            *geometry_slot = Some(InlineContainingBlockGeometry {
                start_fragment_union_rect: fragment_rect,
                end_fragment_union_rect: fragment_rect,
                relative_offset,
                is_hidden_for_paint: should_hide_abspos,
            });
        }

        if lineboxes.1 == linebox {
            geometry_slot
                .as_mut()
                .expect("inline geometry must exist")
                .end_fragment_union_rect
                .Unite(&fragment_rect);
        } else if !unsafe { &*linebox }.IsEmptyLineBox() {
            lineboxes.1 = linebox;
            geometry_slot
                .as_mut()
                .expect("inline geometry must exist")
                .end_fragment_union_rect = fragment_rect;
        }
    }
}

// cpp: layoutng_out_of_flow/inline_containing_block_utils.cc:119-172
#[unsafe(no_mangle)]
pub extern "Rust" fn ComputeInlineContainerGeometryProvider(
    inline_containing_block_map: *mut InlineContainingBlockMap,
    container_builder: *mut BoxFragmentBuilder,
) {
    let map = unsafe { &mut *inline_containing_block_map };
    if map.is_empty() {
        return;
    }
    let builder = unsafe { &mut *container_builder };
    debug_assert!(builder.InlineSize() >= LayoutUnit::default());
    debug_assert!(builder.FragmentBlockSize() >= LayoutUnit::default());
    let mut containing_linebox_map = HeapHashMap::default();
    let items_builder = builder.ItemsBuilder();
    if !items_builder.is_null() {
        let items_builder = unsafe { &mut *items_builder };
        debug_assert_eq!(items_builder.GetWritingMode(), builder.GetWritingMode());
        debug_assert_eq!(items_builder.Direction(), builder.Direction());
        let size = ToPhysicalSize(*builder.Size(), builder.GetWritingMode());
        GatherInlineContainerFragmentsFromItems(
            items_builder.Items(size),
            PhysicalOffset::default(),
            map,
            &mut containing_linebox_map,
            None,
            None,
        );
        return;
    }
    for child in builder.Children() {
        let child_fragment = child.fragment.Get();
        if !unsafe { &*child_fragment }.IsAnonymousBlockFlow() {
            continue;
        }
        let child_fragment = To::<PhysicalBoxFragment>(child_fragment);
        let child_fragment = unsafe { &*child_fragment };
        let items = child_fragment.Items();
        if items.is_null() {
            continue;
        }
        let outer_size = ToPhysicalSize(*builder.Size(), builder.GetWritingMode());
        let child_offset = child.offset.ConvertToPhysical(
            builder.GetWritingDirection(),
            outer_size,
            child_fragment.Size(),
        );
        GatherInlineContainerFragmentsFromItems(
            unsafe { &*items }.Items(),
            child_offset,
            map,
            &mut containing_linebox_map,
            None,
            None,
        );
    }
}

// cpp: layoutng_out_of_flow/inline_containing_block_utils.cc:174-232
#[unsafe(no_mangle)]
pub extern "Rust" fn ComputeInlineContainerGeometryForFragmentainerProvider(
    box_: *const LayoutBox,
    accumulated_containing_block_size: PhysicalSize,
    inline_containing_block_map: *mut InlineContainingBlockMap,
) {
    let map = unsafe { &mut *inline_containing_block_map };
    if map.is_empty() {
        return;
    }
    let box_ = unsafe { &*box_ };
    let writing_direction = box_.StyleRef().GetWritingDirection();
    let containing_block_converter =
        WritingModeConverter::new(writing_direction, accumulated_containing_block_size);
    let mut current_block_offset = LayoutUnit::default();
    let mut containing_linebox_map = HeapHashMap::default();
    for physical_fragment in box_.PhysicalFragments() {
        let logical_offset = LogicalOffset::new(LayoutUnit::default(), current_block_offset);
        let offset = containing_block_converter
            .ToPhysicalOffset(logical_offset, accumulated_containing_block_size);
        let current_fragment_converter =
            WritingModeConverter::new(writing_direction, physical_fragment.Size());
        if physical_fragment.HasItems() {
            let items = unsafe { &*physical_fragment.Items() };
            GatherInlineContainerFragmentsFromItems(
                items.Items(),
                offset,
                map,
                &mut containing_linebox_map,
                Some(&current_fragment_converter),
                Some(&containing_block_converter),
            );
        } else {
            for child in physical_fragment.Children() {
                let child_fragment = child.fragment.Get();
                if !unsafe { &*child_fragment }.IsAnonymousBlockFlow() {
                    continue;
                }
                let child_fragment = To::<PhysicalBoxFragment>(child_fragment);
                let child_fragment = unsafe { &*child_fragment };
                if !child_fragment.HasItems() {
                    continue;
                }
                let items = unsafe { &*child_fragment.Items() };
                GatherInlineContainerFragmentsFromItems(
                    items.Items(),
                    child.offset + offset,
                    map,
                    &mut containing_linebox_map,
                    Some(&current_fragment_converter),
                    Some(&containing_block_converter),
                );
            }
        }
        let break_token = physical_fragment.GetBreakToken();
        if !break_token.is_null() {
            current_block_offset = unsafe { &*break_token }.ConsumedBlockSize();
        }
    }
}
