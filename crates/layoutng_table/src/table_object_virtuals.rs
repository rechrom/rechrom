//! Rust dispatch of the source table subclasses' virtual methods.
#![allow(non_snake_case)]
use crate::{
    layout_table::LayoutTable, layout_table_caption::LayoutTableCaption,
    layout_table_cell::LayoutTableCell, layout_table_column::LayoutTableColumn,
    layout_table_row::LayoutTableRow, layout_table_section::LayoutTableSection,
};
use foundation::{OverlayScrollbarClipBehavior, PhysicalOffset, PhysicalRect, PhysicalSize};
use layoutng_assembly::internal::{
    layout_block::LayoutBlock,
    layout_block_flow::LayoutBlockFlow,
    layout_box::LayoutBox,
    layout_object::{LayoutObject, LayoutObjectClass, StyleChangeContext},
    layout_object_child_list::LayoutObjectChildList,
    layout_object_factory_set::TableObjectVirtuals,
};
use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
use layoutng_style::style::{computed_style::ComputedStyle, style_difference::StyleDifference};
// All source subclasses embed their immediate base at offset zero; the runtime
// class is checked before selecting the concrete Rust override.
macro_rules! object_mut {
    ($object:expr,$ty:ty) => {
        unsafe { &mut *(($object as *mut LayoutObject).cast::<$ty>()) }
    };
}
macro_rules! box_ref {
    ($object:expr,$ty:ty) => {
        unsafe { &*(($object as *const LayoutBox).cast::<$ty>()) }
    };
}
macro_rules! box_mut {
    ($object:expr,$ty:ty) => {
        unsafe { &mut *(($object as *mut LayoutBox).cast::<$ty>()) }
    };
}
fn AddChild(object: &mut LayoutObject, child: *mut LayoutObject, before: *mut LayoutObject) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => object_mut!(object, LayoutTable).AddChild(child, before),
        LayoutObjectClass::TableRow => object_mut!(object, LayoutTableRow).AddChild(child, before),
        LayoutObjectClass::TableSection => {
            object_mut!(object, LayoutTableSection).AddChild(child, before)
        }
        LayoutObjectClass::TableCell | LayoutObjectClass::TableCaption => {
            object_mut!(object, LayoutBlockFlow).AddChildBase(child, before)
        }
        LayoutObjectClass::TableColumn => object.AddChildBase(child, before),
        other => panic!("table virtual called for {other:?}"),
    }
}
fn RemoveChild(object: &mut LayoutObject, child: *mut LayoutObject) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => object_mut!(object, LayoutTable).RemoveChild(child),
        LayoutObjectClass::TableRow => object_mut!(object, LayoutTableRow).RemoveChild(child),
        LayoutObjectClass::TableSection => {
            object_mut!(object, LayoutTableSection).RemoveChild(child)
        }
        LayoutObjectClass::TableCell | LayoutObjectClass::TableCaption => {
            object_mut!(object, LayoutBlockFlow).RemoveChildBase(child)
        }
        LayoutObjectClass::TableColumn => object.RemoveChildBase(child),
        other => panic!("table virtual called for {other:?}"),
    }
}
fn StyleDidChange(
    object: &mut LayoutObject,
    diff: StyleDifference,
    old: *const ComputedStyle,
    new: &ComputedStyle,
    context: &StyleChangeContext,
) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => {
            object_mut!(object, LayoutTable).StyleDidChange(diff, old, new, context)
        }
        LayoutObjectClass::TableCaption => {
            object_mut!(object, LayoutTableCaption).StyleDidChange(diff, old, new, context)
        }
        LayoutObjectClass::TableCell => {
            object_mut!(object, LayoutTableCell).StyleDidChange(diff, old, new, context)
        }
        LayoutObjectClass::TableColumn => {
            object_mut!(object, LayoutTableColumn).StyleDidChange(diff, old, new, context)
        }
        LayoutObjectClass::TableRow => {
            object_mut!(object, LayoutTableRow).StyleDidChange(diff, old, new, context)
        }
        LayoutObjectClass::TableSection => {
            object_mut!(object, LayoutTableSection).StyleDidChange(diff, old, new, context)
        }
        other => panic!("table virtual called for {other:?}"),
    }
}
fn InsertedIntoTree(object: &mut LayoutObject) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableColumn => object_mut!(object, LayoutTableColumn).InsertedIntoTree(),
        _ => object.InsertedIntoTreeBase(),
    }
}
fn WillBeRemovedFromTree(object: &mut LayoutObject) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableColumn => {
            object_mut!(object, LayoutTableColumn).WillBeRemovedFromTree()
        }
        LayoutObjectClass::TableCell => {
            object_mut!(object, LayoutTableCell).WillBeRemovedFromTree()
        }
        LayoutObjectClass::TableRow => object_mut!(object, LayoutTableRow).WillBeRemovedFromTree(),
        LayoutObjectClass::TableSection => {
            object_mut!(object, LayoutTableSection).WillBeRemovedFromTree()
        }
        _ => object.WillBeRemovedFromTreeBase(),
    }
}
fn VirtualChildren(object: &LayoutObject) -> *mut LayoutObjectChildList {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableColumn => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutTableColumn>() }
                .VirtualChildren()
        }
        _ => {
            unsafe { &*(object as *const LayoutObject).cast::<LayoutBlock>() }.VirtualChildrenBase()
        }
    }
}
fn CreateAnonymous(object: &LayoutBox, parent: *const LayoutObject) -> *mut LayoutBox {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => {
            box_ref!(object, LayoutTable).CreateAnonymousBoxWithSameTypeAs(parent)
        }
        LayoutObjectClass::TableCell => {
            box_ref!(object, LayoutTableCell).CreateAnonymousBoxWithSameTypeAs(parent)
        }
        LayoutObjectClass::TableRow => {
            box_ref!(object, LayoutTableRow).CreateAnonymousBoxWithSameTypeAs(parent)
        }
        LayoutObjectClass::TableSection => {
            box_ref!(object, LayoutTableSection).CreateAnonymousBoxWithSameTypeAs(parent)
        }
        LayoutObjectClass::TableCaption => {
            box_ref!(object, LayoutBlock).CreateAnonymousBoxWithSameTypeAs(parent)
        }
        LayoutObjectClass::TableColumn => object.CreateAnonymousBoxWithSameTypeAsBase(parent),
        other => panic!("table virtual called for {other:?}"),
    }
}
fn BorderOutsets(object: &LayoutBox) -> PhysicalBoxStrut {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => box_ref!(object, LayoutTable).BorderOutsets(),
        LayoutObjectClass::TableCell => box_ref!(object, LayoutTableCell).BorderOutsets(),
        _ => object.BorderOutsetsBase(),
    }
}
fn PaddingOutsets(object: &LayoutBox) -> PhysicalBoxStrut {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => box_ref!(object, LayoutTable).PaddingOutsets(),
        _ => object.PaddingOutsetsBase(),
    }
}
fn StickyContainer(object: &LayoutBox) -> *mut LayoutBlock {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableRow => box_ref!(object, LayoutTableRow).StickyContainer(),
        LayoutObjectClass::TableCell => box_ref!(object, LayoutTableCell).StickyContainer(),
        _ => object.ContainingBlock(),
    }
}
fn InvalidateAfterMeasure(object: &LayoutBox) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableCell => {
            box_ref!(object, LayoutTableCell).InvalidateLayoutResultCacheAfterMeasure()
        }
        _ => object.InvalidateLayoutResultCacheAfterMeasureBase(),
    }
}
fn IntrinsicBorders(object: &LayoutBox) -> *const BoxStrut {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableCell => {
            box_ref!(object, LayoutTableCell).IntrinsicLogicalWidthsBorderSizes()
        }
        _ => object.IntrinsicLogicalWidthsBorderSizesBase(),
    }
}
fn SetIntrinsicBorders(object: &mut LayoutBox, borders: &BoxStrut) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableCell => {
            box_mut!(object, LayoutTableCell).SetIntrinsicLogicalWidthsBorderSizes(borders)
        }
        _ => object.SetIntrinsicLogicalWidthsBorderSizesBase(borders),
    }
}
fn StitchedSize(object: &LayoutBox) -> PhysicalSize {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableColumn => box_ref!(object, LayoutTableColumn).StitchedSize(),
        _ => object.StitchedSizeBase(),
    }
}
fn PhysicalLocation(object: &LayoutBox) -> PhysicalOffset {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableColumn => box_ref!(object, LayoutTableColumn).PhysicalLocation(),
        _ => object.PhysicalLocationBase(),
    }
}
fn OverflowClipRect(object: &LayoutBox, behavior: OverlayScrollbarClipBehavior) -> PhysicalRect {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::Table => box_ref!(object, LayoutTable).OverflowClipRect(behavior),
        _ => object.OverflowClipRectWithBehaviorBase(behavior),
    }
}
fn UpdateFromElement(object: &mut LayoutObject) {
    debug_assert!(object.RuntimeClass().IsTableClass());
    match object.RuntimeClass() {
        LayoutObjectClass::TableColumn => {
            object_mut!(object, LayoutTableColumn).UpdateFromElement()
        }
        _ => object.UpdateFromElementBase(),
    }
}
pub(crate) fn Virtuals() -> TableObjectVirtuals {
    TableObjectVirtuals {
        add_child: AddChild,
        remove_child: RemoveChild,
        style_did_change: StyleDidChange,
        inserted_into_tree: InsertedIntoTree,
        will_be_removed_from_tree: WillBeRemovedFromTree,
        virtual_children: VirtualChildren,
        create_anonymous: CreateAnonymous,
        border_outsets: BorderOutsets,
        padding_outsets: PaddingOutsets,
        sticky_container: StickyContainer,
        invalidate_after_measure: InvalidateAfterMeasure,
        intrinsic_borders: IntrinsicBorders,
        set_intrinsic_borders: SetIntrinsicBorders,
        stitched_size: StitchedSize,
        physical_location: PhysicalLocation,
        overflow_clip_rect: OverflowClipRect,
        update_from_element: UpdateFromElement,
    }
}
