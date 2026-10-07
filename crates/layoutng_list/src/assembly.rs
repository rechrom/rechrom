#![allow(non_snake_case)]

use foundation::{AtomicString, EListStylePosition, LayoutUnit, MakeGarbageCollected, To};
use layoutng_assembly::internal::layout_input::{ExtendedStyle, ListStyleType, NodeKind};
use layoutng_assembly::internal::layout_node_metadata::{Element, Node};
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_assembly::layout_assembly::LayoutAssembly;
use layoutng_style::style::computed_style::ComputedStyle;

use crate::layout_inline_list_item::LayoutInlineListItem;
use crate::layout_inside_list_marker::LayoutInsideListMarker;
use crate::layout_list_item::LayoutListItem;
use crate::layout_outside_list_marker::LayoutOutsideListMarker;
use crate::list_marker::ListMarker;

// cpp: layoutng_list/assembly.cc:15-23
fn UpdateMarkerText(object: &mut LayoutObject) {
    if object.IsLayoutListItem() {
        unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutListItem>() }
            .UpdateMarkerTextIfNeeded();
    } else if object.IsInlineListItem() {
        unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutInlineListItem>() }
            .UpdateMarkerTextIfNeeded();
    }
}

// cpp: layoutng_list/assembly.cc:25-28
fn MarkerOccupiesWholeLine(marker: &LayoutObject) -> bool {
    debug_assert!(marker.IsLayoutOutsideListMarker());
    unsafe { &*(marker as *const LayoutObject).cast::<LayoutOutsideListMarker>() }
        .NeedsOccupyWholeLine()
}

// cpp: layoutng_list/assembly.cc:30-32
fn SymbolMarkerText(item: *const LayoutObject) -> *const LayoutObject {
    LayoutListItem::FindSymbolMarkerLayoutText(item)
}

// cpp: layoutng_list/assembly.cc:34-37
fn SymbolWidth(style: &ComputedStyle, list_style: &AtomicString) -> LayoutUnit {
    ListMarker::WidthOfSymbol(style, list_style)
}

// cpp: layoutng_list/assembly.cc:39-46
fn OutsideMarkerInlineOffset(object: &LayoutObject, marker_inline_size: LayoutUnit) -> LayoutUnit {
    debug_assert!(object.IsLayoutOutsideListMarker());
    let marker = unsafe { &*(object as *const LayoutObject).cast::<LayoutOutsideListMarker>() };
    let list_item = marker.Marker().ListItem(marker);
    ListMarker::InlineMarginsForOutside(
        marker.StyleRef(),
        unsafe { &*list_item }.StyleRef(),
        marker_inline_size,
    )
    .0
}

// cpp: layoutng_list/assembly.cc:48-71
fn CreateListObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let element = To::<Element>(&mut *node);
    if node.InputKind() == NodeKind::kListMarker {
        let parent = node.parentElement();
        let marker = if !parent.is_null()
            && unsafe { &*parent }.ComputedStyleRef().ListStylePosition()
                == EListStylePosition::kInside
        {
            MakeGarbageCollected(LayoutInsideListMarker::new(element)) as *mut LayoutObject
        } else {
            MakeGarbageCollected(LayoutOutsideListMarker::new(element)) as *mut LayoutObject
        };
        if node.InputIsStyleGenerated() {
            let defaults = ExtendedStyle::default();
            let parent_style = unsafe { &*parent }.InputStyle();
            let extra = parent_style.extended.as_ref().unwrap_or(&defaults);
            let symbol = matches!(
                extra.list_style_type,
                ListStyleType::kDisc | ListStyleType::kCircle | ListStyleType::kSquare
            );
            let marker_data = ListMarker::GetMut(marker);
            unsafe { &mut *marker_data }.SetStandaloneSymbolMarker(symbol);
        }
        return marker;
    }
    MakeGarbageCollected(LayoutListItem::new(element)) as *mut LayoutObject
}

// cpp: layoutng_list/assembly.h:2-2
// cpp: layoutng_list/assembly.cc:73-79
pub fn InstallListModule(assembly: &mut LayoutAssembly) {
    assembly.objects.list = Some(CreateListObject);
    assembly.algorithms.list_support.update_marker_text = Some(UpdateMarkerText);
    assembly.algorithms.list_support.marker_occupies_whole_line = Some(MarkerOccupiesWholeLine);
    assembly.algorithms.list_support.symbol_marker_text = Some(SymbolMarkerText);
    assembly.algorithms.list_support.symbol_width = Some(SymbolWidth);
    assembly
        .algorithms
        .list_support
        .outside_marker_inline_offset = Some(OutsideMarkerInlineOffset);
}
