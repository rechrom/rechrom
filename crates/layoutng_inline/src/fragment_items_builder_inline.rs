// C++: layoutng_inline/fragment_items_builder_inline.cc.
#![allow(non_snake_case)]

use foundation::{
    LayoutUnit, PhysicalSize, TextDirection, ToLineWritingMode, WritingDirectionMode,
};
use layoutng::internal::inline_node::InlineNode;
use layoutng_fragment_tree::box_fragment_builder::BoxFragmentBuilder;
use layoutng_fragment_tree::fragment_item::{FragmentItem, ItemType};
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_fragment_tree::fragment_items_builder::{
    AddPreviousItemsResult, FragmentItemWithOffset, FragmentItemsBuilder,
};
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;
use layoutng_fragment_tree::inline_cursor::InlineCursor;
#[cfg(debug_assertions)]
use layoutng_fragment_tree::physical_box_fragment::AllowPostLayoutScope;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;

// cpp: layoutng_inline/fragment_items_builder_inline.cc:14-45
#[unsafe(no_mangle)]
pub extern "Rust" fn FragmentItemsBuilderNewForInline(
    node: &InlineNode,
    writing_direction: WritingDirectionMode,
    is_block_fragmented: bool,
) -> FragmentItemsBuilder {
    let mut builder = FragmentItemsBuilder::new(writing_direction);
    builder.node_ = node.clone();
    let items_data = node.ItemsData(false);
    builder.text_content_ = items_data.text_content.clone();
    let first_line = node.ItemsData(true);
    if !std::ptr::eq(items_data, first_line) {
        builder.first_line_text_content_ = first_line.text_content.clone();
    }

    if !is_block_fragmented {
        let estimated_item_count = builder.text_content_.length() / 40 * 3;
        if estimated_item_count as usize > builder.items_.capacity() * 2 {
            builder.items_.reserve(estimated_item_count as usize);
        }
    }
    builder
}

// cpp: layoutng_inline/fragment_items_builder_inline.cc:47-169
#[unsafe(no_mangle)]
pub extern "Rust" fn FragmentItemsBuilderAddPreviousItemsFromInline(
    builder: &mut FragmentItemsBuilder,
    container: &PhysicalBoxFragment,
    items: &FragmentItems,
    end_item: &FragmentItem,
    container_builder: &mut BoxFragmentBuilder,
    max_lines: u32,
) -> AddPreviousItemsResult {
    debug_assert!(!builder.node_.IsNull());
    debug_assert!(!builder.text_content_.IsNull());

    if !items.FirstLineText().IsNull() && builder.first_line_text_content_.IsNull() {
        return AddPreviousItemsResult::default();
    }

    debug_assert!(builder.items_.is_empty());
    let source_items = items.Items();
    let estimated_size = u32::try_from(source_items.len()).expect("fragment item count overflow");
    builder.items_.reserve(estimated_size as usize);

    debug_assert!(!builder.is_converted_to_physical_);
    let converter = WritingModeConverter::new(builder.GetWritingDirection(), container.Size());
    let writing_mode = builder.GetWritingMode();
    let mut line_converter = WritingModeConverter::new(
        WritingDirectionMode::new(ToLineWritingMode(writing_mode), TextDirection::kLtr),
        PhysicalSize::default(),
    );

    let mut last_break_token: *const InlineBreakToken = std::ptr::null();
    let mut items_data = None;
    let mut used_block_size = LayoutUnit::default();
    let mut line_count = 0u32;

    let mut cursor = InlineCursor::new_with_items(container, items);
    while cursor.IsNotNull() {
        let item = unsafe { &*cursor.Current().Item() };
        if std::ptr::eq(item, end_item) {
            break;
        }
        debug_assert!(!item.IsDirty());

        let item_offset = converter.ToLogicalOffset(*item.OffsetInContainerFragment(), item.Size());
        debug_assert_eq!(item.Type(), ItemType::kLine);
        let line_fragment = item.LineBoxFragment();
        debug_assert!(!line_fragment.is_null());
        debug_assert!(!unsafe { &*line_fragment }.IsBlockInInline());
        let break_token = unsafe { &*line_fragment }.GetBreakToken() as *const InlineBreakToken;
        debug_assert!(!break_token.is_null());
        let current_items_data = if unsafe { &*break_token }.UseFirstLineStyle() {
            builder.node_.ItemsData(true)
        } else {
            *items_data.get_or_insert_with(|| builder.node_.ItemsData(false))
        };
        if !current_items_data.IsValidOffsetAt(unsafe { &*break_token }.Start()) {
            eprintln!("FragmentItemsBuilder: reusable line has an invalid item offset");
            break;
        }

        last_break_token = break_token;
        container_builder.AddChild(
            unsafe { &*line_fragment },
            item_offset,
            None,
            false,
            None,
            std::ptr::null(),
        );
        used_block_size += ToLogicalSize(item.Size(), writing_mode).block_size;

        builder
            .items_
            .push(FragmentItemWithOffset::new(item_offset, item.clone()));
        let line_box_bounds = *item.RectInContainerFragment();
        line_converter.SetOuterSize(line_box_bounds.size);
        let mut line = cursor.CursorForDescendants();
        while line.IsNotNull() {
            let line_child = unsafe { &*line.Current().Item() };
            if line_child.Type() != ItemType::kLine {
                debug_assert!(line_child.CanReuse());
            }
            #[cfg(debug_assertions)]
            let _allow_post_layout = if line_child.IsRelayoutBoundary() {
                Some(AllowPostLayoutScope::new())
            } else {
                None
            };
            let offset_in_line = *line_child.OffsetInContainerFragment() - line_box_bounds.offset;
            builder.items_.push(FragmentItemWithOffset::new(
                line_converter.ToLogicalOffset(offset_in_line, line_child.Size()),
                line_child.clone(),
            ));

            let new_item = &builder.items_.last().expect("reused line item").item;
            let box_fragment = new_item.BoxFragment();
            if !box_fragment.is_null() {
                let post_layout = unsafe { &*box_fragment }.PostLayout();
                new_item
                    .GetMutableForCloning()
                    .ReplaceBoxFragment(unsafe { &*post_layout });
            }
            line.MoveToNext();
        }
        line_count += 1;
        if line_count == max_lines {
            break;
        }
        cursor.MoveToNextSkippingChildren();
    }
    debug_assert!(builder.items_.len() <= estimated_size as usize);

    if !last_break_token.is_null() {
        debug_assert!(line_count > 0);
        debug_assert!(max_lines == 0 || line_count <= max_lines);
        return AddPreviousItemsResult {
            inline_break_token: last_break_token,
            used_block_size,
            line_count,
            succeeded: true,
        };
    }
    AddPreviousItemsResult::default()
}
