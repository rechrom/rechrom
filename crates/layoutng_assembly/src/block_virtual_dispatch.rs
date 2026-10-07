// C++ virtual calls dispatch through the most-derived LayoutObject vtable.
// The translated block hierarchy is base-first, so its explicit runtime
// class selects the same overrides. Other derived classes must register and
// implement their own dispatch before they can execute through this host.
#![allow(non_snake_case)]

use crate::internal::layout_block::LayoutBlock;
use crate::internal::layout_block_flow::LayoutBlockFlow;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_inline::LayoutInline;
use crate::internal::layout_input::NodeKind;
use crate::internal::layout_object::{LayoutObject, LayoutObjectClass};
use crate::internal::layout_object::{RecalcScrollableOverflowResult, StyleChangeContext};
use crate::internal::layout_object_child_list::LayoutObjectChildList;
use crate::internal::layout_pass_scope::LayoutObjectFactoryScope;
use crate::internal::layout_replaced::LayoutReplaced;
use crate::internal::layout_text::LayoutText;
use crate::internal::layout_view::LayoutView;
use crate::internal::map_coordinates_flags::MapCoordinatesFlags;
use crate::physical_box_fragment::PhysicalBoxFragment;
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::{LayoutUnit, PhysicalOffset, PhysicalSize};
use layoutng_geometry::geometry::box_strut::{BoxStrut, PhysicalBoxStrut};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::style_difference::StyleDifference;
use std::sync::OnceLock;

// The flex object lives in a downstream crate, while LayoutObject's virtual
// call site lives here. Registration preserves the C++ most-derived override
// without introducing a Cargo dependency cycle.
type FlexStitchedRowGapIndex =
    fn(*const LayoutObject, &PhysicalBoxFragment, usize, Option<usize>) -> usize;
static FLEX_STITCHED_ROW_GAP_INDEX: OnceLock<FlexStitchedRowGapIndex> = OnceLock::new();

type GridStyleDidChange = fn(
    *mut LayoutObject,
    StyleDifference,
    *const ComputedStyle,
    &ComputedStyle,
    &StyleChangeContext,
);
static GRID_STYLE_DID_CHANGE: OnceLock<GridStyleDidChange> = OnceLock::new();

pub fn RegisterGridStyleDidChange(callback: GridStyleDidChange) {
    let existing = GRID_STYLE_DID_CHANGE.get_or_init(|| callback);
    assert_eq!(*existing as usize, callback as usize);
}

pub fn RegisterFlexStitchedRowGapIndex(callback: FlexStitchedRowGapIndex) {
    let existing = FLEX_STITCHED_ROW_GAP_INDEX.get_or_init(|| callback);
    assert_eq!(*existing as usize, callback as usize);
}

// cpp: layoutng/internal/layout_object.h:2023-2029
// cpp: layoutng_flex/layout_flexible_box.h:33-37
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectStitchedRowGapIndex(
    object: *const LayoutObject,
    fragment: &PhysicalBoxFragment,
    gap_index: usize,
    line_index: Option<usize>,
) -> usize {
    if class(object) == LayoutObjectClass::FlexibleBox {
        let callback = FLEX_STITCHED_ROW_GAP_INDEX
            .get()
            .expect("flex stitched row-gap virtual dispatch is not registered");
        callback(object, fragment, gap_index, line_index)
    } else if class(object) == LayoutObjectClass::Grid {
        (grid_virtuals().stitched_row_gap_index)(object, fragment, gap_index, line_index)
    } else {
        unsafe { &*object }.StitchedRowGapIndexBase(fragment, gap_index, line_index)
    }
}

fn class(object: *const LayoutObject) -> LayoutObjectClass {
    assert!(!object.is_null());
    let class = unsafe { &*object }.RuntimeClass();
    assert_ne!(
        class,
        LayoutObjectClass::Base,
        "unregistered LayoutObject subclass"
    );
    class
}

// cpp: layoutng/internal/layout_object.h:835-842
// cpp: layoutng/internal/layout_box_model_object.h:362-365
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsBoxModelObject(object: *const LayoutObject) -> bool {
    !matches!(
        class(object),
        LayoutObjectClass::Base | LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText
    )
}

// cpp: layoutng/internal/layout_box.h:1286-1289
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsBox(object: *const LayoutObject) -> bool {
    if class(object).IsTableClass() {
        return true;
    }

    matches!(
        class(object),
        LayoutObjectClass::Box
            | LayoutObjectClass::Block
            | LayoutObjectClass::FlexibleBox
            | LayoutObjectClass::Grid
            | LayoutObjectClass::BlockFlow
            | LayoutObjectClass::ListItem
            | LayoutObjectClass::OutsideListMarker
            | LayoutObjectClass::TextCombine
            | LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::TextControlInnerEditor
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::Replaced
            | LayoutObjectClass::SvgRoot
            | LayoutObjectClass::View
    )
}

// cpp: layoutng/internal/layout_block.h:225-228
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutBlock(object: *const LayoutObject) -> bool {
    if class(object).IsTableClass() {
        return class(object) != LayoutObjectClass::TableColumn;
    }

    matches!(
        class(object),
        LayoutObjectClass::Block
            | LayoutObjectClass::FlexibleBox
            | LayoutObjectClass::Grid
            | LayoutObjectClass::BlockFlow
            | LayoutObjectClass::ListItem
            | LayoutObjectClass::OutsideListMarker
            | LayoutObjectClass::TextCombine
            | LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::TextControlInnerEditor
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::View
    )
}

// cpp: layoutng/internal/layout_block_flow.h:70-73
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutBlockFlow(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::BlockFlow
            | LayoutObjectClass::ListItem
            | LayoutObjectClass::OutsideListMarker
            | LayoutObjectClass::TextCombine
            | LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::TextControlInnerEditor
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::View
    )
}

// cpp: layoutng/internal/layout_view.h:107-110
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutView(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::View
}

// cpp: layoutng/internal/layout_object.h:1033-1036,1044-1047
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutInline(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::Inline
            | LayoutObjectClass::SvgInline
            | LayoutObjectClass::SvgTSpan
            | LayoutObjectClass::SvgTextPath
            | LayoutObjectClass::InlineListItem
            | LayoutObjectClass::InsideListMarker
    )
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsText(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText
    )
}

// The C++ base returns false for these virtual predicates, and none of the
// registered block/box/view classes overrides them. An unregistered derived
// class is rejected by class() until its actual override is connected.
macro_rules! block_false_predicates {
    ($($name:ident),* $(,)?) => {$ (
        #[unsafe(no_mangle)]
        pub extern "Rust" fn $name(object: *const LayoutObject) -> bool {
            class(object);
            false
        }
    )*};
}

block_false_predicates!(
    DispatchLayoutObjectIsCanvas,
    DispatchLayoutObjectIsFrameSet,
    DispatchLayoutObjectIsLayoutCustom,
    DispatchLayoutObjectIsCustomLayoutLoaded,
    DispatchLayoutObjectIsImage,
    DispatchLayoutObjectIsLayoutIFrame,
    DispatchLayoutObjectIsLayoutImage,
    DispatchLayoutObjectIsCounter,
    DispatchLayoutObjectIsQuote,
    DispatchLayoutObjectIsRuby,
    DispatchLayoutObjectIsLayoutEmbeddedContent,
    DispatchLayoutObjectIsLayoutGridLanes,
    DispatchLayoutObjectIsMathML,
    DispatchLayoutObjectIsMedia,
    DispatchLayoutObjectIsVideo,
    DispatchLayoutObjectIsViewTransitionRoot,
);

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutTableCol(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TableColumn
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTable(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::Table
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTableCaption(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TableCaption
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTableCell(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TableCell
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTableRow(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TableRow
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTableSection(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TableSection
}

// cpp: layoutng_svg/layout_svg_group.h:26-29
// cpp: layoutng_svg/layout_svg_shape.h:20-21
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVG(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgRoot
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgInline
            | LayoutObjectClass::SvgTSpan
            | LayoutObjectClass::SvgTextPath
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::SvgInlineText
    )
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGContainer(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::SvgGroup | LayoutObjectClass::SvgForeignObject
    )
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGShape(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgShape
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGRoot(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgRoot
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGForeignObject(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgForeignObject
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGInline(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::SvgInline | LayoutObjectClass::SvgTSpan | LayoutObjectClass::SvgTextPath
    )
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGText(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgText
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGInlineText(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgInlineText
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectLocalToSVGParentTransform(
    object: *const LayoutObject,
) -> AffineTransform {
    class(object);
    unsafe { &*object }.LocalToSVGParentTransformBase()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectSetNeedsTextMetricsUpdate(object: *mut LayoutObject) {
    if class(object) == LayoutObjectClass::SvgText {
        let objects = LayoutObjectFactoryScope::Objects();
        let update = unsafe { objects.as_ref() }
            .and_then(|objects| objects.svg_text_needs_metrics_update)
            .expect("SVG text metrics invalidation is not installed");
        update(unsafe { &mut *object });
    } else {
        unsafe { &mut *object }.SetNeedsTextMetricsUpdateBase();
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGTSpan(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgTSpan
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGTextPath(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::SvgTextPath
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsSVGTransformableContainer(
    object: *const LayoutObject,
) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::SvgGroup | LayoutObjectClass::SvgForeignObject
    )
}

// cpp: layoutng_flex/layout_flexible_box.h:57-60
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsFlexibleBox(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::FlexibleBox
}

// LayoutBR shares LayoutText's runtime class in the translated hierarchy.
// Its source node preserves the more specific kind needed by IsBR().
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsBR(object: *const LayoutObject) -> bool {
    if !matches!(
        class(object),
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText
    ) {
        return false;
    }
    let node = unsafe { &*object }.GetNode();
    !node.is_null() && unsafe { &*node }.InputKind() == NodeKind::kLineBreak
}

// cpp: layoutng/internal/layout_replaced.h:124-126
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutReplaced(object: *const LayoutObject) -> bool {
    matches!(
        class(object),
        LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot
    )
}

// LayoutReplaced itself inherits LayoutObject's default. The only override
// in the translated native hierarchy is SVGForeignObject. LayoutVideo is not
// a represented native class and must supply its own override when translated.
// cpp: core/layout/layout_object.h:737-740
// cpp: core/layout/svg/layout_svg_foreign_object.h:62-65
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsReplacedNormalFlowStackingContext(
    object: *const LayoutObject,
    style: &ComputedStyle,
) -> bool {
    match class(object) {
        LayoutObjectClass::SvgForeignObject => {
            unsafe { &*object }.CheckIsNotDestroyed();
            true
        }
        _ => unsafe { &*object }.IsReplacedNormalFlowStackingContextBase(style),
    }
}

// cpp: layoutng/internal/layout_replaced.h:68-70
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectCanHaveChildren(object: *const LayoutObject) -> bool {
    if class(object) == LayoutObjectClass::TableColumn {
        return unsafe { &*object }.StyleRef().Display() == foundation::EDisplay::kTableColumnGroup;
    }

    if matches!(
        class(object),
        LayoutObjectClass::Replaced | LayoutObjectClass::SvgShape
    ) {
        false
    } else {
        unsafe { &*object }.CanHaveChildrenBase()
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsChildAllowed(
    object: *const LayoutObject,
    child: *mut LayoutObject,
    style: &ComputedStyle,
) -> bool {
    if class(object) == LayoutObjectClass::TableColumn {
        return unsafe { &*child }.IsLayoutTableCol()
            && style.Display() == foundation::EDisplay::kTableColumn;
    }

    match class(object) {
        LayoutObjectClass::SvgRoot
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::SvgText => {
            let objects = LayoutObjectFactoryScope::Objects();
            let allowed = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_is_child_allowed)
                .expect("SVG IsChildAllowed is not installed");
            allowed(unsafe { &*object }, child, style)
        }
        LayoutObjectClass::View => {
            unsafe { &*object.cast::<LayoutView>() }.IsChildAllowed(child, style)
        }
        _ => unsafe { &*object }.IsChildAllowedBase(child, style),
    }
}

// cpp: layoutng_forms/layout_text_control_multi_line.h:18-21
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTextArea(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TextControlMultiLine
}

// cpp: layoutng_forms/layout_text_control_single_line.h:18-21
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTextField(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TextControlSingleLine
}

// cpp: layoutng_forms/layout_text_control_inner_editor.h:23-26
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsTextControlInnerEditor(
    object: *const LayoutObject,
) -> bool {
    class(object) == LayoutObjectClass::TextControlInnerEditor
}

// cpp: layoutng_forms/layout_fieldset.h:44-47
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsFieldset(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::Fieldset
}

// cpp: layoutng_list/layout_list_item.h:50-53
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutListItem(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::ListItem
}

// cpp: layoutng_list/layout_inline_list_item.h:32-34
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsInlineListItem(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::InlineListItem
}

// cpp: layoutng_list/layout_inside_list_marker.h:43-47
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutInsideListMarker(
    object: *const LayoutObject,
) -> bool {
    class(object) == LayoutObjectClass::InsideListMarker
}

// cpp: layoutng_list/layout_outside_list_marker.h:39-43
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutOutsideListMarker(
    object: *const LayoutObject,
) -> bool {
    class(object) == LayoutObjectClass::OutsideListMarker
}

// The registered block and inline host classes inherit this no-op base
// implementation. The list-item package will supply its override when added.
// cpp: layoutng/internal/layout_block_flow.h:133-133
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBlockFlowWillCollectInlines(flow: *mut LayoutBlockFlow) {
    unsafe { &mut *flow }.WillCollectInlinesBase();
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutTextCombine(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::TextCombine
}

// cpp: layoutng/internal/layout_box.h:1181-1184
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsEligibleForPaintOrLayoutContainment(
    object: *const LayoutObject,
) -> bool {
    if class(object).IsTableClass() {
        return matches!(
            class(object),
            LayoutObjectClass::Table
                | LayoutObjectClass::TableCell
                | LayoutObjectClass::TableCaption
        );
    }

    matches!(
        class(object),
        LayoutObjectClass::Box
            | LayoutObjectClass::Block
            | LayoutObjectClass::FlexibleBox
            | LayoutObjectClass::Grid
            | LayoutObjectClass::BlockFlow
            | LayoutObjectClass::ListItem
            | LayoutObjectClass::OutsideListMarker
            | LayoutObjectClass::TextCombine
            | LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::TextControlInnerEditor
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::Replaced
            | LayoutObjectClass::SvgRoot
            | LayoutObjectClass::View
    )
}

// cpp: layoutng/internal/layout_block_flow.cc:147-151
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsInitialLetterBox(object: *const LayoutObject) -> bool {
    match class(object) {
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::View => {
            unsafe { &*object.cast::<LayoutBlockFlow>() }.IsInitialLetterBox()
        }
        _ => false,
    }
}

// cpp: layoutng/internal/layout_box.h:624-627
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsValidColumnSpannerInTree(
    object: *const LayoutObject,
) -> bool {
    if class(object).IsTableClass() {
        return unsafe { &*object.cast::<LayoutBox>() }
            .IsValidColumnSpannerInTreeWithCurrentStyle();
    }
    match class(object) {
        LayoutObjectClass::Box
        | LayoutObjectClass::Block
        | LayoutObjectClass::FlexibleBox
        | LayoutObjectClass::Grid
        | LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::Replaced
        | LayoutObjectClass::SvgRoot
        | LayoutObjectClass::View => {
            unsafe { &*object.cast::<LayoutBox>() }.IsValidColumnSpannerInTreeWithCurrentStyle()
        }
        _ => false,
    }
}

// cpp: layoutng/internal/layout_block.cc:102-135
// cpp: layoutng/internal/layout_block_flow.cc:214-291
// cpp: layoutng/internal/layout_view_layout.cc:59-65
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectAddChild(
    object: *mut LayoutObject,
    child: *mut LayoutObject,
    before_child: *mut LayoutObject,
) {
    if class(object).IsTableClass() {
        (table_virtuals().add_child)(unsafe { &mut *object }, child, before_child);
        return;
    }

    match class(object) {
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &mut *object.cast::<LayoutInline>() }.AddChild(child, before_child)
        }
        LayoutObjectClass::TextControlInnerEditor => {
            let objects = LayoutObjectFactoryScope::Objects();
            let insert = unsafe { objects.as_ref() }
                .and_then(|objects| objects.inner_editor_add_child)
                .expect("forms inner-editor AddChild is not installed");
            insert(
                unsafe { &mut *object.cast::<LayoutBlockFlow>() },
                child,
                before_child,
            );
        }
        LayoutObjectClass::Fieldset => {
            let objects = LayoutObjectFactoryScope::Objects();
            let insert = unsafe { objects.as_ref() }
                .and_then(|objects| objects.fieldset_add_child)
                .expect("forms fieldset AddChild is not installed");
            insert(
                unsafe { &mut *object.cast::<LayoutBlockFlow>() },
                child,
                before_child,
            );
        }
        LayoutObjectClass::SvgGroup | LayoutObjectClass::SvgForeignObject => {
            let objects = LayoutObjectFactoryScope::Objects();
            let insert = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_group_add_child)
                .expect("SVG group child insertion is not installed");
            insert(
                unsafe { &mut *object.cast::<LayoutBlockFlow>() },
                child,
                before_child,
            );
        }
        LayoutObjectClass::SvgText => {
            let objects = LayoutObjectFactoryScope::Objects();
            let insert = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_text_add_child)
                .expect("SVG text child insertion is not installed");
            insert(
                unsafe { &mut *object.cast::<LayoutBlockFlow>() },
                child,
                before_child,
            );
        }
        LayoutObjectClass::SvgRoot => {
            let objects = LayoutObjectFactoryScope::Objects();
            let insert = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_root_add_child)
                .expect("SVG root child insertion is not installed");
            insert(unsafe { &mut *object }, child, before_child);
        }
        LayoutObjectClass::View => {
            unsafe { &mut *object.cast::<LayoutView>() }.AddChild(child, before_child)
        }
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine => {
            unsafe { &mut *object.cast::<LayoutBlockFlow>() }.AddChild(child, before_child)
        }
        LayoutObjectClass::Grid => (grid_virtuals().add_child)(object, child, before_child),
        LayoutObjectClass::Block | LayoutObjectClass::FlexibleBox => {
            unsafe { &mut *object.cast::<LayoutBlock>() }.AddChildBase(child, before_child)
        }
        other => panic!("AddChild is not implemented for {other:?}"),
    }
}

// cpp: layoutng/internal/layout_block.h:180-183
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectVirtualChildren(
    object: *const LayoutObject,
) -> *mut LayoutObjectChildList {
    if class(object).IsTableClass() {
        return (table_virtuals().virtual_children)(unsafe { &*object });
    }

    match class(object) {
        LayoutObjectClass::SvgRoot => {
            let objects = LayoutObjectFactoryScope::Objects();
            let children = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_root_children)
                .expect("SVG root children are not installed");
            children(unsafe { &*object })
        }
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &*object.cast::<LayoutInline>() }.VirtualChildren()
                as *const LayoutObjectChildList as *mut LayoutObjectChildList
        }
        LayoutObjectClass::Block
        | LayoutObjectClass::FlexibleBox
        | LayoutObjectClass::Grid
        | LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::View => {
            unsafe { &*object.cast::<LayoutBlock>() }.VirtualChildrenBase()
        }
        _ => unsafe { &*object }.VirtualChildrenBase(),
    }
}

// cpp: layoutng/internal/layout_block.h:96-99
// cpp: layoutng/internal/layout_block_flow.h:107-110
// cpp: layoutng/internal/layout_view.h:102-105
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectGetName(object: *const LayoutObject) -> &'static str {
    match class(object) {
        LayoutObjectClass::Table => return "LayoutTable",
        LayoutObjectClass::TableCaption => return "LayoutTableCaption",
        LayoutObjectClass::TableCell => return "LayoutTableCell",
        LayoutObjectClass::TableRow => return "LayoutTableRow",
        LayoutObjectClass::TableSection => return "LayoutTableSection",
        LayoutObjectClass::TableColumn => {
            return if unsafe { &*object }.StyleRef().Display() == foundation::EDisplay::kTableColumn
            {
                "LayoutTableCol"
            } else {
                "LayoutTableColGroup"
            }
        }
        _ => {}
    }

    match class(object) {
        LayoutObjectClass::View => "LayoutView",
        LayoutObjectClass::BlockFlow => "LayoutBlockFlow",
        LayoutObjectClass::ListItem => "LayoutListItem",
        LayoutObjectClass::InlineListItem => "LayoutInlineListItem",
        LayoutObjectClass::SvgInline => "LayoutSVGInline",
        LayoutObjectClass::SvgTSpan => "LayoutSVGTSpan",
        LayoutObjectClass::SvgTextPath => "LayoutSVGTextPath",
        LayoutObjectClass::SvgText => "LayoutSVGText",
        LayoutObjectClass::SvgInlineText => "LayoutSVGInlineText",
        LayoutObjectClass::InsideListMarker => "LayoutInsideListMarker",
        LayoutObjectClass::OutsideListMarker => "LayoutOutsideListMarker",
        LayoutObjectClass::TextCombine => "LayoutTextCombine",
        LayoutObjectClass::TextControlSingleLine => "LayoutTextControlSingleLine",
        LayoutObjectClass::TextControlMultiLine => "LayoutTextControlMultiLine",
        LayoutObjectClass::TextControlInnerEditor => "LayoutTextControlInnerEditor",
        LayoutObjectClass::SvgGroup => "LayoutSVGGroup",
        LayoutObjectClass::SvgForeignObject => "LayoutSVGForeignObject",
        LayoutObjectClass::SvgShape => "LayoutSVGShape",
        LayoutObjectClass::SvgRoot => "LayoutSVGRoot",
        LayoutObjectClass::Fieldset => "LayoutFieldset",
        LayoutObjectClass::Replaced => "LayoutInputReplaced",
        LayoutObjectClass::FlexibleBox => "LayoutFlexibleBox",
        LayoutObjectClass::Grid => "LayoutGrid",
        LayoutObjectClass::Block => "LayoutBlock",
        other => panic!("GetName is not implemented for {other:?}"),
    }
}

// cpp: layoutng/internal/layout_block_flow.cc:132-135,565-603
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBlockFlowAllowsInlineChildren(
    flow: *const LayoutBlockFlow,
) -> bool {
    assert!(!flow.is_null());
    if class(flow.cast::<LayoutObject>()) == LayoutObjectClass::TextControlInnerEditor {
        let objects = LayoutObjectFactoryScope::Objects();
        let allows = unsafe { objects.as_ref() }
            .and_then(|objects| objects.inner_editor_allows_inline_children)
            .expect("forms inner-editor AllowsInlineChildren is not installed");
        return allows(unsafe { &*flow });
    }
    unsafe { &*flow }.AllowsInlineChildrenBase()
}

// cpp: layoutng/internal/layout_block_flow.h:95-95
// cpp: layoutng/internal/layout_block_flow.cc:536-563
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBlockFlowAllowsColumns(flow: *const LayoutBlockFlow) -> bool {
    assert!(!flow.is_null());
    class(flow.cast::<LayoutObject>());
    unsafe { &*flow }.AllowsColumnsBase()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBlockFlowCreatesNewFormattingContext(
    flow: *const LayoutBlockFlow,
) -> bool {
    assert!(!flow.is_null());
    if matches!(
        class(flow.cast::<LayoutObject>()),
        LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
    ) {
        return true;
    }
    unsafe { &*flow }.CreatesNewFormattingContextBase()
}

// cpp: layoutng/internal/layout_block_flow.h:87-90
// cpp: layoutng/internal/layout_view_layout.cc:223-226
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBlockFlowIsFragmentationContextRoot(
    flow: *const LayoutBlockFlow,
) -> bool {
    assert!(!flow.is_null());
    if class(flow.cast::<LayoutObject>()) == LayoutObjectClass::View {
        unsafe { &*flow.cast::<LayoutView>() }.IsFragmentationContextRoot()
    } else {
        unsafe { &*flow }.IsFragmentationContextRootBase()
    }
}

// cpp: layoutng/internal/layout_box.h:773-787
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxCreatesNewFormattingContextProvider(box_: &LayoutBox) -> bool {
    match class(box_ as *const LayoutBox as *const LayoutObject) {
        LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText => true,
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::View => {
            unsafe { &*(box_ as *const LayoutBox as *const LayoutBlockFlow) }
                .CreatesNewFormattingContextBase()
        }
        _ => box_.CreatesNewFormattingContextBase(),
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxIsFragmentationContextRootProvider(box_: &LayoutBox) -> bool {
    let object = box_ as *const LayoutBox as *const LayoutObject;
    match class(object) {
        LayoutObjectClass::View => {
            unsafe { &*object.cast::<LayoutView>() }.IsFragmentationContextRoot()
        }
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption => {
            unsafe { &*object.cast::<LayoutBlockFlow>() }.IsFragmentationContextRootBase()
        }
        LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText => {
            unsafe { &*object.cast::<LayoutBlockFlow>() }.IsFragmentationContextRootBase()
        }
        _ => box_.IsFragmentationContextRootBase(),
    }
}

// cpp: layoutng/internal/layout_object.h:634-637
// cpp: layoutng/internal/layout_box.h:1195-1198
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectIsEligibleForSizeContainmentProvider(
    object: &LayoutObject,
) -> bool {
    if class(object).IsTableClass() {
        return class(object) == LayoutObjectClass::TableCaption;
    }

    is_box_class(class(object))
}

// cpp: layoutng/internal/layout_box.h:718-721
// cpp: layoutng/internal/layout_view_layout.cc:184-187
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxShouldPlaceBlockDirectionScrollbarOnLogicalLeftProvider(
    box_: &LayoutBox,
) -> bool {
    assert!(is_box_class(class(
        box_ as *const LayoutBox as *const LayoutObject
    )));
    box_.ShouldPlaceBlockDirectionScrollbarOnLogicalLeftBase()
}

// cpp: layoutng/internal/layout_box_geometry.cc:244-252
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxEffectiveRootScrollerViewportSizeProvider(
    box_: &LayoutBox,
) -> PhysicalSize {
    assert!(is_box_class(class(
        box_ as *const LayoutBox as *const LayoutObject
    )));
    unsafe { &*box_.View() }.ViewRect().size
}

// cpp: layoutng/internal/layout_box_model_tree.cc:200-203
// cpp: layoutng/internal/layout_box_geometry.cc:376-387
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelObjectContainingBlockLogicalWidthForContent(
    owner: *const LayoutBoxModelObject,
) -> LayoutUnit {
    let runtime_class = class(owner.cast::<LayoutObject>());
    if is_box_class(runtime_class) {
        unsafe { &*owner.cast::<LayoutBox>() }.ContainingBlockLogicalWidthForContent()
    } else {
        unsafe { &*owner }.ContainingBlockLogicalWidthForContentBase()
    }
}

// cpp: layoutng/internal/layout_box_model_tree.cc:77-92
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelObjectMoveChildrenTo(
    owner: *mut LayoutBoxModelObject,
    to: *mut LayoutBoxModelObject,
    start_child: *mut LayoutObject,
    end_child: *mut LayoutObject,
    before_child: *mut LayoutObject,
    full_remove_insert: bool,
) {
    class(owner.cast::<LayoutObject>());
    unsafe { &mut *owner }.MoveChildrenToBase(
        to,
        start_child,
        end_child,
        before_child,
        full_remove_insert,
    );
}

// cpp: layoutng/internal/layout_sticky_constraints.cc:103-106
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelObjectStickyContainer(
    owner: *const LayoutBoxModelObject,
) -> *mut LayoutBlock {
    if class(owner.cast()).IsTableClass() {
        return (table_virtuals().sticky_container)(unsafe { &*owner.cast::<LayoutBox>() });
    }

    class(owner.cast::<LayoutObject>());
    unsafe { &*owner }.ContainingBlock()
}

// cpp: layoutng/internal/layout_object_core_services.cc:436-446
// cpp: layoutng/internal/layout_sticky_constraints.cc:137-149
// cpp: layoutng/internal/layout_box_geometry.cc:389-402
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectOffsetFromContainerInternal(
    object: *const LayoutObject,
    container: *const LayoutObject,
    mode: MapCoordinatesFlags,
) -> PhysicalOffset {
    let runtime_class = class(object);
    if is_box_class(runtime_class) {
        unsafe { &*object.cast::<LayoutBox>() }.OffsetFromContainerInternal(container, mode)
    } else {
        unsafe { &*object.cast::<LayoutBoxModelObject>() }
            .OffsetFromContainerInternal(container, mode)
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:280-292
// cpp: layoutng/internal/layout_block.cc:334-338
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectRecalcScrollableOverflow(
    object: *mut LayoutObject,
) -> RecalcScrollableOverflowResult {
    if matches!(
        class(object),
        LayoutObjectClass::Table | LayoutObjectClass::TableRow | LayoutObjectClass::TableSection
    ) {
        return unsafe { &mut *object.cast::<LayoutBlock>() }.RecalcScrollableOverflow();
    }

    let runtime_class = class(object);
    if matches!(
        runtime_class,
        LayoutObjectClass::Block
            | LayoutObjectClass::FlexibleBox
            | LayoutObjectClass::Grid
            | LayoutObjectClass::BlockFlow
            | LayoutObjectClass::ListItem
            | LayoutObjectClass::OutsideListMarker
            | LayoutObjectClass::TextCombine
            | LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::TextControlInnerEditor
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::View
    ) {
        unsafe { &mut *object.cast::<LayoutBlock>() }.RecalcScrollableOverflow()
    } else {
        unsafe { &mut *object }.RecalcScrollableOverflow()
    }
}

// cpp: layoutng/internal/layout_object.h:2854-2857
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectUpdateAnonymousChildStyle(
    object: *const LayoutObject,
    child: *const LayoutObject,
    builder: &mut ComputedStyleBuilder,
) {
    if class(object) == LayoutObjectClass::Fieldset {
        let objects = LayoutObjectFactoryScope::Objects();
        let update = unsafe { objects.as_ref() }
            .and_then(|objects| objects.fieldset_update_anonymous_child_style)
            .expect("forms fieldset anonymous style updater is not installed");
        update(
            unsafe { &*object.cast::<LayoutBlockFlow>() },
            child,
            builder,
        );
        return;
    }
    unsafe { &*object }.UpdateAnonymousChildStyleBase(child, builder)
}

// cpp: layoutng_forms/layout_fieldset.h:48-48
// cpp: layoutng_forms/layout_fieldset.cc:68-114
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectInsertedIntoTree(object: *mut LayoutObject) {
    if class(object).IsTableClass() {
        (table_virtuals().inserted_into_tree)(unsafe { &mut *object });
        return;
    }

    if class(object) == LayoutObjectClass::SvgText {
        let objects = LayoutObjectFactoryScope::Objects();
        let insert = unsafe { objects.as_ref() }
            .and_then(|objects| objects.svg_text_inserted_into_tree)
            .expect("SVG text tree insertion is not installed");
        insert(unsafe { &mut *object.cast::<LayoutBlockFlow>() });
    } else if class(object) == LayoutObjectClass::Fieldset {
        let objects = LayoutObjectFactoryScope::Objects();
        let insert = unsafe { objects.as_ref() }
            .and_then(|objects| objects.fieldset_inserted_into_tree)
            .expect("forms fieldset tree insertion is not installed");
        insert(unsafe { &mut *object.cast::<LayoutBlockFlow>() });
    } else {
        unsafe { &mut *object }.InsertedIntoTreeBase();
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectWillBeRemovedFromTree(object: *mut LayoutObject) {
    if class(object).IsTableClass() {
        (table_virtuals().will_be_removed_from_tree)(unsafe { &mut *object });
        return;
    }

    if class(object) == LayoutObjectClass::SvgText {
        let objects = LayoutObjectFactoryScope::Objects();
        let remove = unsafe { objects.as_ref() }
            .and_then(|objects| objects.svg_text_will_be_removed_from_tree)
            .expect("SVG text tree removal is not installed");
        remove(unsafe { &mut *object.cast::<LayoutBlockFlow>() });
    } else {
        unsafe { &mut *object }.WillBeRemovedFromTreeBase();
    }
}

// cpp: layoutng_forms/layout_fieldset.h:30-35
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxContentLayoutBoxProvider(box_: &mut LayoutBox) -> *mut LayoutBox {
    if class(box_ as *mut LayoutBox as *mut LayoutObject) == LayoutObjectClass::Fieldset {
        let objects = LayoutObjectFactoryScope::Objects();
        let content = unsafe { objects.as_ref() }
            .and_then(|objects| objects.fieldset_content_layout_box)
            .expect("forms fieldset content box is not installed");
        content(unsafe { &mut *(box_ as *mut LayoutBox as *mut LayoutBlockFlow) })
    } else {
        box_.ContentLayoutBoxBase()
    }
}

// cpp: layoutng_forms/layout_fieldset.h:65-67
// cpp: layoutng_forms/layout_fieldset.cc:214-226
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxScrollWidthProvider(box_: &LayoutBox) -> LayoutUnit {
    if class(box_ as *const LayoutBox as *const LayoutObject) == LayoutObjectClass::Fieldset {
        let objects = LayoutObjectFactoryScope::Objects();
        let scroll_width = unsafe { objects.as_ref() }
            .and_then(|objects| objects.fieldset_scroll_width)
            .expect("forms fieldset scroll width is not installed");
        scroll_width(unsafe { &*(box_ as *const LayoutBox as *const LayoutBlockFlow) })
    } else {
        box_.ScrollWidthBase()
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxScrollHeightProvider(box_: &LayoutBox) -> LayoutUnit {
    if class(box_ as *const LayoutBox as *const LayoutObject) == LayoutObjectClass::Fieldset {
        let objects = LayoutObjectFactoryScope::Objects();
        let scroll_height = unsafe { objects.as_ref() }
            .and_then(|objects| objects.fieldset_scroll_height)
            .expect("forms fieldset scroll height is not installed");
        scroll_height(unsafe { &*(box_ as *const LayoutBox as *const LayoutBlockFlow) })
    } else {
        box_.ScrollHeightBase()
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:190-203
// cpp: layoutng/internal/layout_box_lifecycle.cc:52-99
// cpp: layoutng/internal/layout_block.cc:340-379
// cpp: layoutng/internal/layout_block_flow.cc:605-624
// cpp: layoutng/internal/layout_view_layout.cc:214-221
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectStyleWillChange(
    object: *mut LayoutObject,
    difference: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    context: &mut StyleChangeContext,
) {
    let runtime_class = class(object);
    if matches!(
        runtime_class,
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText
    ) {
        #[cfg(feature = "translation_in_progress")]
        {
            unsafe { &*object.cast::<LayoutText>() }
                .StyleWillChange(difference, old_style, new_style, context);
            return;
        }
        #[cfg(not(feature = "translation_in_progress"))]
        panic!("text style dispatch requires the inline assembly");
    } else if is_box_class(runtime_class) {
        unsafe { &mut *object.cast::<LayoutBox>() }
            .StyleWillChange(difference, old_style, new_style, context);
    } else {
        unsafe { &mut *object.cast::<LayoutBoxModelObject>() }
            .StyleWillChange(difference, old_style, new_style, context);
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectStyleDidChange(
    object: *mut LayoutObject,
    difference: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    context: &StyleChangeContext,
) {
    match class(object) {
        LayoutObjectClass::Grid => GRID_STYLE_DID_CHANGE
            .get()
            .expect("grid StyleDidChange virtual dispatch is not registered")(
            object, difference, old_style, new_style, context,
        ),
        LayoutObjectClass::Table
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableColumn
        | LayoutObjectClass::TableRow
        | LayoutObjectClass::TableSection => (table_virtuals().style_did_change)(
            unsafe { &mut *object },
            difference,
            old_style,
            new_style,
            context,
        ),

        LayoutObjectClass::TextControlInnerEditor => {
            let objects = LayoutObjectFactoryScope::Objects();
            let update = unsafe { objects.as_ref() }
                .and_then(|objects| objects.inner_editor_style_did_change)
                .expect("forms inner-editor StyleDidChange is not installed");
            update(
                unsafe { &mut *object.cast::<LayoutBlockFlow>() },
                difference,
                old_style,
                new_style,
                context,
            );
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &mut *object.cast::<LayoutText>() }
                .StyleDidChange(difference, old_style, new_style, context)
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => unsafe { &mut *object.cast::<LayoutInline>() }
            .StyleDidChange(difference, old_style, new_style, context),
        #[cfg(not(feature = "translation_in_progress"))]
        LayoutObjectClass::Text
        | LayoutObjectClass::SvgInlineText
        | LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            panic!("inline style dispatch requires the inline assembly")
        }
        LayoutObjectClass::View => unsafe { &mut *object.cast::<LayoutView>() }.StyleDidChange(
            difference,
            unsafe { old_style.as_ref() },
            new_style,
            context,
        ),
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::Fieldset => unsafe { &mut *object.cast::<LayoutBlockFlow>() }
            .StyleDidChange(difference, old_style, new_style, context),
        LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText => unsafe { &mut *object.cast::<LayoutBlockFlow>() }
            .StyleDidChange(difference, old_style, new_style, context),
        LayoutObjectClass::Block | LayoutObjectClass::FlexibleBox => {
            unsafe { &mut *object.cast::<LayoutBlock>() }
                .StyleDidChange(difference, old_style, new_style, context)
        }
        LayoutObjectClass::Box | LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot => {
            unsafe { &mut *object.cast::<LayoutBox>() }
                .StyleDidChange(difference, old_style, new_style, context)
        }
        LayoutObjectClass::BoxModelObject => unsafe { &mut *object.cast::<LayoutBoxModelObject>() }
            .StyleDidChange(difference, old_style, new_style, context),
        LayoutObjectClass::Base => unreachable!(),
    }
}

// cpp: layoutng/internal/layout_object_core_services.cc:224-230
// cpp: layoutng/internal/layout_box_model_lifecycle.cc:39-43
// cpp: layoutng/internal/layout_box_lifecycle.cc:33-38
// cpp: layoutng/internal/layout_block.cc:63-67
// cpp: layoutng/internal/layout_view_layout.cc:194-202
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectWillBeDestroyed(object: *mut LayoutObject) {
    match class(object) {
        LayoutObjectClass::Grid => unsafe { &mut *object.cast::<LayoutBlock>() }.WillBeDestroyed(),
        LayoutObjectClass::Table
        | LayoutObjectClass::TableRow
        | LayoutObjectClass::TableSection => {
            unsafe { &mut *object.cast::<LayoutBlock>() }.WillBeDestroyed()
        }
        LayoutObjectClass::TableColumn => {
            unsafe { &mut *object.cast::<LayoutBox>() }.WillBeDestroyed()
        }

        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &mut *object.cast::<LayoutText>() }.WillBeDestroyed()
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &mut *object.cast::<LayoutInline>() }.WillBeDestroyed()
        }
        #[cfg(not(feature = "translation_in_progress"))]
        LayoutObjectClass::Text
        | LayoutObjectClass::SvgInlineText
        | LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            panic!("inline destruction requires the inline assembly")
        }
        LayoutObjectClass::View => unsafe { &mut *object.cast::<LayoutView>() }.WillBeDestroyed(),
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::Block
        | LayoutObjectClass::FlexibleBox => {
            unsafe { &mut *object.cast::<LayoutBlock>() }.WillBeDestroyed()
        }
        LayoutObjectClass::Box | LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot => {
            unsafe { &mut *object.cast::<LayoutBox>() }.WillBeDestroyed()
        }
        LayoutObjectClass::BoxModelObject => {
            unsafe { &mut *object.cast::<LayoutBoxModelObject>() }.WillBeDestroyed()
        }
        LayoutObjectClass::Base => unreachable!(),
    }
}

// cpp: layoutng/internal/layout_object.h:3398-3402
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectAnonymousHasStylePropagationOverride(
    object: *mut LayoutObject,
) -> bool {
    class(object);
    false
}

// cpp: layoutng/internal/layout_object_core_services.cc:150-162
// cpp: layoutng/internal/layout_block_flow.cc:293-348
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectRemoveChild(
    object: *mut LayoutObject,
    old_child: *mut LayoutObject,
) {
    if class(object).IsTableClass() {
        (table_virtuals().remove_child)(unsafe { &mut *object }, old_child);
        return;
    }

    match class(object) {
        LayoutObjectClass::Grid => (grid_virtuals().remove_child)(object, old_child),
        LayoutObjectClass::SvgRoot => {
            let objects = LayoutObjectFactoryScope::Objects();
            let remove = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_root_remove_child)
                .expect("SVG root child removal is not installed");
            remove(unsafe { &mut *object }, old_child);
        }
        LayoutObjectClass::SvgText => {
            let objects = LayoutObjectFactoryScope::Objects();
            let remove = unsafe { objects.as_ref() }
                .and_then(|objects| objects.svg_text_remove_child)
                .expect("SVG text child removal is not installed");
            remove(unsafe { &mut *object.cast::<LayoutBlockFlow>() }, old_child);
        }
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::View => {
            unsafe { &mut *object.cast::<LayoutBlockFlow>() }.RemoveChildBase(old_child)
        }
        _ => unsafe { &mut *object }.RemoveChildBase(old_child),
    }
}

// cpp: layoutng/internal/layout_box_model_object.h:350-353
// cpp: layoutng/internal/layout_block_flow.cc:350-358
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelObjectCanMergeWith(
    owner: *const LayoutBoxModelObject,
    other: &LayoutBoxModelObject,
) -> bool {
    match class(owner.cast()) {
        LayoutObjectClass::Table => return other.IsTable(),
        LayoutObjectClass::TableCell => return other.IsTableCell(),
        LayoutObjectClass::TableRow => return other.IsTableRow(),
        LayoutObjectClass::TableSection => return other.IsTableSection(),
        _ => {}
    }

    match class(owner.cast::<LayoutObject>()) {
        LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::View => {
            unsafe { &*owner.cast::<LayoutBlockFlow>() }.CanMergeWithBase(other)
        }
        _ => unsafe { &*owner }.CanMergeWithBase(other),
    }
}

// cpp: layoutng/internal/layout_block.cc:254-258
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxCreateAnonymousBoxWithSameTypeAsProvider(
    box_: &LayoutBox,
    parent: *const LayoutObject,
) -> *mut LayoutBox {
    if class(box_ as *const LayoutBox as *const LayoutObject).IsTableClass() {
        return (table_virtuals().create_anonymous)(box_, parent);
    }

    match class(box_ as *const LayoutBox as *const LayoutObject) {
        LayoutObjectClass::Block
        | LayoutObjectClass::FlexibleBox
        | LayoutObjectClass::Grid
        | LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::View => unsafe { &*(box_ as *const LayoutBox as *const LayoutBlock) }
            .CreateAnonymousBoxWithSameTypeAs(parent),
        _ => box_.CreateAnonymousBoxWithSameTypeAsBase(parent),
    }
}

// cpp: layoutng/internal/layout_box.h:909-921
#[unsafe(no_mangle)]
pub extern "Rust" fn InvalidateLayoutResultCacheAfterMeasureProvider(box_: &LayoutBox) {
    if class(box_ as *const LayoutBox as *const LayoutObject).IsTableClass() {
        (table_virtuals().invalidate_after_measure)(box_);
        return;
    }

    assert!(is_box_class(class(
        box_ as *const LayoutBox as *const LayoutObject
    )));
    box_.InvalidateLayoutResultCacheAfterMeasureBase()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxIntrinsicLogicalWidthsBorderSizesProvider(
    box_: &LayoutBox,
) -> *const BoxStrut {
    if class(box_ as *const LayoutBox as *const LayoutObject).IsTableClass() {
        return (table_virtuals().intrinsic_borders)(box_);
    }

    assert!(is_box_class(class(
        box_ as *const LayoutBox as *const LayoutObject
    )));
    box_.IntrinsicLogicalWidthsBorderSizesBase()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxSetIntrinsicLogicalWidthsBorderSizesProvider(
    box_: &mut LayoutBox,
    borders: &BoxStrut,
) {
    if class(box_ as *mut LayoutBox as *mut LayoutObject).IsTableClass() {
        (table_virtuals().set_intrinsic_borders)(box_, borders);
        return;
    }

    assert!(is_box_class(class(
        box_ as *mut LayoutBox as *mut LayoutObject
    )));
    box_.SetIntrinsicLogicalWidthsBorderSizesBase(borders)
}

// cpp: layoutng/internal/layout_box_model_object.h:192-203
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelObjectPaddingOutsets(
    owner: *const LayoutBoxModelObject,
) -> PhysicalBoxStrut {
    if class(owner.cast()).IsTableClass() {
        return (table_virtuals().padding_outsets)(unsafe { &*owner.cast::<LayoutBox>() });
    }

    class(owner.cast::<LayoutObject>());
    unsafe { &*owner }.PaddingOutsetsBase()
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelObjectBorderOutsets(
    owner: *const LayoutBoxModelObject,
) -> PhysicalBoxStrut {
    if class(owner.cast()).IsTableClass() {
        return (table_virtuals().border_outsets)(unsafe { &*owner.cast::<LayoutBox>() });
    }

    class(owner.cast::<LayoutObject>());
    unsafe { &*owner }.BorderOutsetsBase()
}

fn is_box_class(class: LayoutObjectClass) -> bool {
    if class.IsTableClass() {
        return true;
    }

    matches!(
        class,
        LayoutObjectClass::Box
            | LayoutObjectClass::Block
            | LayoutObjectClass::FlexibleBox
            | LayoutObjectClass::Grid
            | LayoutObjectClass::BlockFlow
            | LayoutObjectClass::ListItem
            | LayoutObjectClass::OutsideListMarker
            | LayoutObjectClass::TextCombine
            | LayoutObjectClass::TextControlSingleLine
            | LayoutObjectClass::TextControlMultiLine
            | LayoutObjectClass::TextControlInnerEditor
            | LayoutObjectClass::Fieldset
            | LayoutObjectClass::TableCell
            | LayoutObjectClass::TableCaption
            | LayoutObjectClass::SvgGroup
            | LayoutObjectClass::SvgForeignObject
            | LayoutObjectClass::SvgShape
            | LayoutObjectClass::SvgText
            | LayoutObjectClass::Replaced
            | LayoutObjectClass::SvgRoot
            | LayoutObjectClass::View
    )
}

// cpp: layoutng/internal/layout_box_model_lifecycle.cc:63
// cpp: layoutng/internal/layout_box.h:1220
// cpp: layoutng/internal/layout_view.h:355
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutBoxModelUpdateFromStyle(object: *mut LayoutBoxModelObject) {
    match class(object.cast::<LayoutObject>()) {
        LayoutObjectClass::Grid => unsafe { &mut *object.cast::<LayoutBox>() }.UpdateFromStyle(),
        LayoutObjectClass::Table
        | LayoutObjectClass::TableCaption
        | LayoutObjectClass::TableCell
        | LayoutObjectClass::TableColumn
        | LayoutObjectClass::TableRow
        | LayoutObjectClass::TableSection => {
            unsafe { &mut *object.cast::<LayoutBox>() }.UpdateFromStyle()
        }

        LayoutObjectClass::View => unsafe { &mut *object.cast::<LayoutView>() }.UpdateFromStyle(),
        LayoutObjectClass::Box
        | LayoutObjectClass::Block
        | LayoutObjectClass::FlexibleBox
        | LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::Fieldset => {
            unsafe { &mut *object.cast::<LayoutBox>() }.UpdateFromStyle()
        }
        LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText => {
            unsafe { &mut *object.cast::<LayoutBox>() }.UpdateFromStyle()
        }
        LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot => {
            unsafe { &mut *object.cast::<LayoutBox>() }.UpdateFromStyle()
        }
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker
        | LayoutObjectClass::BoxModelObject => unsafe { &mut *object }.UpdateFromStyle(),
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText | LayoutObjectClass::Base => {
            unreachable!()
        }
    }
}

// cpp: layoutng/internal/layout_box_fragment_data.cc:133-155
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectHasInlineFragments(object: *const LayoutObject) -> bool {
    match class(object) {
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &*object.cast::<LayoutText>() }.HasInlineFragments()
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &*object.cast::<LayoutInline>() }.HasInlineFragments()
        }
        class if is_box_class(class) => {
            unsafe { &*object.cast::<LayoutBox>() }.HasInlineFragments()
        }
        _ => unsafe { &*object }.HasInlineFragmentsBase(),
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectFirstInlineFragmentItemIndex(
    object: *const LayoutObject,
) -> usize {
    match class(object) {
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &*object.cast::<LayoutText>() }.FirstInlineFragmentItemIndex()
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &*object.cast::<LayoutInline>() }.FirstInlineFragmentItemIndex()
        }
        class if is_box_class(class) => {
            unsafe { &*object.cast::<LayoutBox>() }.FirstInlineFragmentItemIndex()
        }
        _ => unsafe { &*object }.FirstInlineFragmentItemIndexBase(),
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectClearFirstInlineFragmentItemIndex(
    object: *mut LayoutObject,
) {
    match class(object) {
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &mut *object.cast::<LayoutText>() }.ClearFirstInlineFragmentItemIndex()
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &mut *object.cast::<LayoutInline>() }.ClearFirstInlineFragmentItemIndex()
        }
        class if is_box_class(class) => {
            unsafe { &mut *object.cast::<LayoutBox>() }.ClearFirstInlineFragmentItemIndex()
        }
        _ => unsafe { &mut *object }.ClearFirstInlineFragmentItemIndexBase(),
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectSetFirstInlineFragmentItemIndex(
    object: *mut LayoutObject,
    index: usize,
) {
    match class(object) {
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &mut *object.cast::<LayoutText>() }.SetFirstInlineFragmentItemIndex(index)
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => {
            unsafe { &mut *object.cast::<LayoutInline>() }.SetFirstInlineFragmentItemIndex(index)
        }
        class if is_box_class(class) => {
            unsafe { &mut *object.cast::<LayoutBox>() }.SetFirstInlineFragmentItemIndex(index)
        }
        _ => unsafe { &mut *object }.SetFirstInlineFragmentItemIndexBase(index),
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectInlineFormattingContextWillChange(
    object: *mut LayoutObject,
    new_value: bool,
) {
    match class(object) {
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Text | LayoutObjectClass::SvgInlineText => {
            unsafe { &mut *object.cast::<LayoutText>() }
                .InLayoutNGInlineFormattingContextWillChange(new_value)
        }
        #[cfg(feature = "translation_in_progress")]
        LayoutObjectClass::Inline
        | LayoutObjectClass::SvgInline
        | LayoutObjectClass::SvgTSpan
        | LayoutObjectClass::SvgTextPath
        | LayoutObjectClass::InlineListItem
        | LayoutObjectClass::InsideListMarker => unsafe { &mut *object.cast::<LayoutInline>() }
            .InLayoutNGInlineFormattingContextWillChange(new_value),
        class if is_box_class(class) => unsafe { &mut *object.cast::<LayoutBox>() }
            .InLayoutNGInlineFormattingContextWillChange(new_value),
        _ => unsafe { &mut *object }.InLayoutNGInlineFormattingContextWillChangeBase(new_value),
    }
}

// cpp: layoutng/internal/layout_block.cc:63-67
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectRespectsCSSOverflow(object: &LayoutObject) -> bool {
    if matches!(
        class(object),
        LayoutObjectClass::TableRow | LayoutObjectClass::TableSection
    ) {
        return false;
    }
    if matches!(
        class(object),
        LayoutObjectClass::Table | LayoutObjectClass::TableCell | LayoutObjectClass::TableCaption
    ) {
        return unsafe { &*(object as *const LayoutObject).cast::<LayoutBlock>() }
            .RespectsCSSOverflow();
    }
    if class(object) == LayoutObjectClass::TableColumn {
        return object.RespectsCSSOverflowBase();
    }

    match class(object) {
        LayoutObjectClass::Fieldset => false,
        LayoutObjectClass::Replaced | LayoutObjectClass::SvgRoot => {
            unsafe { &*(object as *const LayoutObject as *const LayoutReplaced) }
                .RespectsCSSOverflow()
        }
        LayoutObjectClass::Block
        | LayoutObjectClass::FlexibleBox
        | LayoutObjectClass::Grid
        | LayoutObjectClass::BlockFlow
        | LayoutObjectClass::ListItem
        | LayoutObjectClass::OutsideListMarker
        | LayoutObjectClass::TextCombine
        | LayoutObjectClass::TextControlSingleLine
        | LayoutObjectClass::TextControlMultiLine
        | LayoutObjectClass::TextControlInnerEditor
        | LayoutObjectClass::SvgGroup
        | LayoutObjectClass::SvgForeignObject
        | LayoutObjectClass::SvgShape
        | LayoutObjectClass::SvgText
        | LayoutObjectClass::View => {
            unsafe { &*(object as *const LayoutObject as *const LayoutBlock) }.RespectsCSSOverflow()
        }
        LayoutObjectClass::Box => object.RespectsCSSOverflowBase(),
        other => panic!("RespectsCSSOverflow is not registered for {other:?}"),
    }
}

fn table_virtuals() -> crate::internal::layout_object_factory_set::TableObjectVirtuals {
    unsafe { LayoutObjectFactoryScope::Objects().as_ref() }
        .and_then(|o| o.table_virtuals)
        .expect("native table virtual dispatch is not installed")
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxStitchedSizeProvider(box_: &LayoutBox) -> PhysicalSize {
    if box_.RuntimeClass().IsTableClass() {
        (table_virtuals().stitched_size)(box_)
    } else {
        box_.StitchedSizeBase()
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxPhysicalLocationProvider(box_: &LayoutBox) -> PhysicalOffset {
    if box_.RuntimeClass().IsTableClass() {
        (table_virtuals().physical_location)(box_)
    } else {
        box_.PhysicalLocationBase()
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxOverflowClipRectProvider(
    box_: &LayoutBox,
    behavior: foundation::OverlayScrollbarClipBehavior,
) -> foundation::PhysicalRect {
    if box_.RuntimeClass().IsTableClass() {
        (table_virtuals().overflow_clip_rect)(box_, behavior)
    } else {
        box_.OverflowClipRectWithBehaviorBase(behavior)
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectVisualRectRespectsVisibility(
    object: *const LayoutObject,
) -> bool {
    if matches!(
        class(object),
        LayoutObjectClass::Table
            | LayoutObjectClass::TableRow
            | LayoutObjectClass::TableSection
            | LayoutObjectClass::TableColumn
    ) {
        false
    } else {
        unsafe { &*object }.VisualRectRespectsVisibilityBase()
    }
}
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectUpdateFromElement(object: *mut LayoutObject) {
    if class(object).IsTableClass() {
        (table_virtuals().update_from_element)(unsafe { &mut *object });
    } else {
        unsafe { &mut *object }.UpdateFromElementBase();
    }
}

// cpp: layoutng_grid/layout_grid.h:87-96,108-109
pub struct GridObjectVirtuals {
    pub add_child: fn(*mut LayoutObject, *mut LayoutObject, *mut LayoutObject),
    pub remove_child: fn(*mut LayoutObject, *mut LayoutObject),
    pub stitched_row_gap_index:
        fn(*const LayoutObject, &PhysicalBoxFragment, usize, Option<usize>) -> usize,
    pub adjust_anchor_containing_block:
        fn(
            &LayoutBox,
            *const crate::internal::grid_layout_data::GridLayoutData,
            &ComputedStyle,
            &layoutng_geometry::geometry::logical_rect::LogicalRect,
            &LayoutBox,
        ) -> layoutng_geometry::geometry::logical_rect::LogicalRect,
}
static GRID_OBJECT_VIRTUALS: OnceLock<GridObjectVirtuals> = OnceLock::new();
pub fn RegisterGridObjectVirtuals(callbacks: GridObjectVirtuals) {
    GRID_OBJECT_VIRTUALS.get_or_init(|| callbacks);
}
fn grid_virtuals() -> &'static GridObjectVirtuals {
    GRID_OBJECT_VIRTUALS
        .get()
        .expect("grid virtual dispatch is not registered")
}
// cpp: layoutng_grid/layout_grid.h:102-105
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutGrid(object: *const LayoutObject) -> bool {
    class(object) == LayoutObjectClass::Grid
}
// cpp: layoutng/internal/layout_object.h:943-945
#[unsafe(no_mangle)]
pub extern "Rust" fn DispatchLayoutObjectIsLayoutGridOrGridLanes(
    object: *const LayoutObject,
) -> bool {
    DispatchLayoutObjectIsLayoutGrid(object) || DispatchLayoutObjectIsLayoutGridLanes(object)
}
// cpp: layoutng/internal/layout_box_geometry.cc:61-67
// cpp: layoutng_grid/layout_grid.cc:27-38
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutBoxAdjustOutOfFlowContainingBlockForAnchorProvider(
    box_: &LayoutBox,
    data: *const crate::internal::grid_layout_data::GridLayoutData,
    style: &ComputedStyle,
    padding: &layoutng_geometry::geometry::logical_rect::LogicalRect,
    query: &LayoutBox,
) -> layoutng_geometry::geometry::logical_rect::LogicalRect {
    if box_.RuntimeClass() == LayoutObjectClass::Grid {
        (grid_virtuals().adjust_anchor_containing_block)(box_, data, style, padding, query)
    } else {
        *padding
    }
}
