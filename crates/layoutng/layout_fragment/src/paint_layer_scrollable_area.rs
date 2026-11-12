#![allow(non_snake_case)]

use foundation::{
    gfx, CompositorElementId, CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace,
    DynamicTo, EOverflow, EOverscrollBehavior, EScrollbarWidth, EVisibility, LayoutUnit,
    MakeGarbageCollected, Member, MinimumValueForLength, OverlayScrollbarClipBehavior,
    PaintInvalidationReason, PhysicalOffset, PhysicalRect, PhysicalSize, ToFlooredPoint,
    ToPixelSnappedRect, ToRoundedPoint, ToRoundedSize, Visitor,
};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesBoth, kPhysicalAxesHorizontal, kPhysicalAxesNone, kPhysicalAxesVertical,
    PhysicalAxes,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{PseudoId, ScrollbarGutter};

use super::super::layout_box::LayoutBox;
use super::super::layout_node_metadata::{Element, Node};
use super::super::layout_object::LayoutObject;
use super::super::map_coordinates_flags::MapCoordinatesFlags;
use super::super::scroll_anchor::ScrollAnchor;
use super::super::scroll_types::{
    IncludeScrollbarsInRect, ScrollOffset, SnapScrollOffsetToPhysicalPixels,
};
use super::super::scrollbar::{
    ScrollCornerDisplayItemClient, Scrollbar, ScrollingBackgroundDisplayItemClient,
};
use super::super::scrollbar_mode::mojom::blink::ScrollbarMode;
use super::super::scrollbar_orientation::ScrollbarOrientation;
use super::super::scrollbar_theme_metrics::ScrollbarThemeThickness;
use super::super::text_overflow_post_layout_snapshot::TextOverflowPostLayoutSnapshot;
use std::cell::Cell;

// Source: core/paint/paint_layer_scrollable_area.cc:145.
const kResizerControlExpandRatioForTouch: i32 = 2;

/// Source PaintLayerScrollableArea::Resize, cc:2442-2515. Layout computes
/// typed used-size mutations; Page publishes them through the existing inline
/// CSS declaration owner and completes the requested style/layout lifecycle.
#[derive(Clone, Copy, Debug)]
pub struct NativeResizePlan {
    pub node_id: u64,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
}

// Source: core/paint/paint_layer_scrollable_area.h:66.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizerHitTestType {
    kResizerForPointer,
    kResizerForTouch,
}

// Source: core/paint/paint_layer_scrollable_area.cc:2289-2299.
fn ExpandResizerCornerRectForTouch(mut corner: gfx::Rect) -> gfx::Rect {
    corner.Offset(
        -corner.width() * (kResizerControlExpandRatioForTouch - 1),
        -corner.height() * (kResizerControlExpandRatioForTouch - 1),
    );
    gfx::Rect::new(
        corner.origin(),
        gfx::Size::new(
            corner.width() * kResizerControlExpandRatioForTouch,
            corner.height() * kResizerControlExpandRatioForTouch,
        ),
    )
}

// Source: core/paint/paint_layer_scrollable_area.cc:2304-2308.
fn ScrollCornerOrResizerRect(scroll_corner: gfx::Rect, resizer: gfx::Rect) -> gfx::Rect {
    if scroll_corner.IsEmpty() {
        resizer
    } else {
        scroll_corner
    }
}

fn RectContainsPoint(rect: gfx::Rect, point: &gfx::Point) -> bool {
    point.x() >= rect.x()
        && point.x() < rect.right()
        && point.y() >= rect.y()
        && point.y() < rect.bottom()
}

// Source: core/paint/paint_layer_scrollable_area.h:708-714.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleBasedScrollbarData {
    pub overflow_x: EOverflow,
    pub overflow_y: EOverflow,
    pub gutter: ScrollbarGutter,
    pub width: EScrollbarWidth,
    pub writing_mode: foundation::WritingMode,
}

// The source method reads these values through LayoutView, Settings and Page.
// StyleCascade runs before it can borrow the retained LayoutView, so its
// package boundary calls this same algorithm with the immutable host facts.
// Source: core/paint/paint_layer_scrollable_area.cc:1698-1738.
pub fn ComputeScrollbarWidthsForViewportUnitsFromInputs(
    scrollbar_properties: StyleBasedScrollbarData,
    visual_viewport_supplies_scrollbars: bool,
    hide_scrollbars: bool,
    theme: &super::super::layout_input_types::ScrollbarThemeMetrics,
) -> gfx::Size {
    if visual_viewport_supplies_scrollbars
        || hide_scrollbars
        || theme.uses_overlay_scrollbars
        || scrollbar_properties.width == EScrollbarWidth::kNone
    {
        return gfx::Size::default();
    }

    let mode = |overflow: EOverflow| match overflow {
        EOverflow::kScroll => ScrollbarMode::kAlwaysOn,
        EOverflow::kHidden | EOverflow::kClip => ScrollbarMode::kAlwaysOff,
        _ => ScrollbarMode::kAuto,
    };
    let horizontal_mode = mode(scrollbar_properties.overflow_x);
    let vertical_mode = mode(scrollbar_properties.overflow_y);
    let scrollbar_thickness = ScrollbarThemeThickness(theme, scrollbar_properties.width);
    let mut scrollbar_thicknesses = gfx::Size::default();
    if horizontal_mode == ScrollbarMode::kAlwaysOn {
        scrollbar_thicknesses.set_height(scrollbar_thickness);
    }
    if vertical_mode == ScrollbarMode::kAlwaysOn {
        scrollbar_thicknesses.set_width(scrollbar_thickness);
    }
    if scrollbar_properties.gutter.value() & ScrollbarGutter::kScrollbarGutterBothEdges.value() != 0
    {
        if foundation::IsVerticalWritingMode(scrollbar_properties.writing_mode) {
            scrollbar_thicknesses.set_height(scrollbar_thickness * 2);
        } else {
            scrollbar_thicknesses.set_width(scrollbar_thickness * 2);
        }
    } else if scrollbar_properties.gutter.value() & ScrollbarGutter::kScrollbarGutterStable.value()
        != 0
    {
        if foundation::IsVerticalWritingMode(scrollbar_properties.writing_mode) {
            scrollbar_thicknesses.set_height(scrollbar_thickness);
        } else {
            scrollbar_thicknesses.set_width(scrollbar_thickness);
        }
    }
    scrollbar_thicknesses
}

fn ResolveScrollbarModes(box_: &LayoutBox) -> (ScrollbarMode, ScrollbarMode) {
    if box_.IsLayoutView() {
        let view =
            unsafe { &*(box_ as *const LayoutBox as *const super::super::layout_view::LayoutView) };
        let (mut horizontal, mut vertical) = (ScrollbarMode::kAuto, ScrollbarMode::kAuto);
        view.CalculateScrollbarModes(&mut horizontal, &mut vertical, None, None);
        return (horizontal, vertical);
    }
    let style: &ComputedStyle = box_.StyleRef();
    let resolve = |overflow: EOverflow| match overflow {
        EOverflow::kScroll => ScrollbarMode::kAlwaysOn,
        EOverflow::kHidden | EOverflow::kClip | EOverflow::kVisible => ScrollbarMode::kAlwaysOff,
        _ => ScrollbarMode::kAuto,
    };
    (resolve(style.OverflowX()), resolve(style.OverflowY()))
}

// Source: core/paint/paint_layer_scrollable_area.cc:1417-1421.
// LayoutView is the viewport scrollbar owner. Rechrom's extracted LayoutView
// shares the document element's node storage, so retain the source type
// distinction before applying the viewport-defining-element identity test.
fn CanHaveOverflowScrollbars(box_: &LayoutBox) -> bool {
    box_.IsLayoutView() || box_.ViewportDefiningElementForLayout() as *mut Node != box_.GetNode()
}

// Source: core/paint/paint_layer_scrollable_area.cc:1149-1169.
fn ElementForScrollStart(box_: &LayoutBox) -> *mut Element {
    let element = DynamicTo::<Element>(box_.GetNode());
    if !element.is_null() {
        return element;
    }

    let node = box_.GetNode();
    let parent = box_.Parent();
    if node.is_null() && !parent.is_null() && unsafe { &*parent }.IsFieldset() {
        return DynamicTo::<Element>(unsafe { &*parent }.GetNode());
    }

    if !node.is_null() && unsafe { &*node }.IsDocumentNode() {
        let root = box_ as *const LayoutBox as *const LayoutObject;
        let mut object = box_.NextInPreOrder(root);
        while !object.is_null() {
            let current = unsafe { &*object };
            if current.IsDocumentElement() {
                return DynamicTo::<Element>(current.GetNode());
            }
            object = current.NextInPreOrder(root);
        }
    }

    std::ptr::null_mut()
}

// Source: core/paint/paint_layer_scrollable_area.cc:1554-1602.
fn ScrollbarStyleSource(box_: &LayoutBox) -> *mut LayoutObject {
    let layout_box = box_ as *const LayoutBox as *mut LayoutObject;
    if box_.IsLayoutView() {
        if box_.StyleRef().UsesStandardScrollbarStyle() {
            return layout_box;
        }

        let viewport = box_.ViewportGeometryForLayout();
        if !viewport.allow_custom_scrollbar_in_main_frame
            && viewport.is_main_frame
            && viewport.visual_viewport_is_active
        {
            return layout_box;
        }

        let root = layout_box as *const LayoutObject;
        let mut object = box_.NextInPreOrder(root);
        let mut document_element = std::ptr::null_mut();
        while !object.is_null() {
            let current = unsafe { &*object };
            let element = DynamicTo::<Element>(current.GetNode());
            let has_custom_style = current
                .StyleRef()
                .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>());
            if current.IsBody() && current.IsBox() && has_custom_style {
                return object;
            }
            if current.IsDocumentElement() && has_custom_style {
                document_element = object;
            }
            object = current.NextInPreOrder(root);
        }
        if !document_element.is_null() {
            return document_element;
        }
    } else if box_.GetNode().is_null() && !box_.Parent().is_null() {
        return box_.Parent();
    }

    layout_box
}

#[derive(Clone, Copy, Debug)]
pub enum NativeScrollbarActivityKind {
    Added,
    Removed,
    Scrolled,
    // Source: core/scroll/scrollable_area.cc:168-180 and
    // core/scroll/mac_scrollbar_animator_impl.mm:198-218. These remain
    // distinct from Scrolled: wheel kPhaseMayBegin reveals the scrollbar
    // before any offset changes and defers its fade-out until wheel end.
    FadeInDeferred,
    FadeOutIfNeeded,
    MouseEntered,
    MouseExited,
}
pub struct NativeScrollbarActivity {
    pub scrollbar: foundation::Persistent<Scrollbar>,
    pub area: CompositorElementId,
    pub kind: NativeScrollbarActivityKind,
}
// The source's GC owner has no paint-layer base in this extracted package.
#[repr(C)]
pub struct PaintLayerScrollableArea {
    overflow_rect_: PhysicalRect,
    scroll_origin_: gfx::Point,
    scroll_origin_changed_: bool,
    has_horizontal_scrollbar_: bool,
    has_vertical_scrollbar_: bool,
    horizontal_scrollbar_: Member<Scrollbar>,
    vertical_scrollbar_: Member<Scrollbar>,
    // Pending hypothetical source owners survive only across the embedding's
    // synchronous style-publication/layout retry, never as pseudo cache entries.
    hypothetical_horizontal_style_owner_: Cell<Member<Scrollbar>>,
    hypothetical_vertical_style_owner_: Cell<Member<Scrollbar>>,
    scroll_corner_client_: Member<ScrollCornerDisplayItemClient>,
    scroll_corner_:
        Cell<Member<super::super::layout_custom_scrollbar_part::LayoutCustomScrollbarPart>>,
    scroll_corner_style_pending_: Cell<bool>,
    scrolling_background_client_: Member<ScrollingBackgroundDisplayItemClient>,
    box_: Member<LayoutBox>,
    scroll_offset_: ScrollOffset,
    text_overflow_snapshot_: Member<TextOverflowPostLayoutSnapshot>,
    // Source: core/scroll/scrollable_area.h:803.  This is retained on the
    // ScrollableArea owner; Scrollbar::ShouldParticipateInHitTesting reads it
    // instead of Paint inferring overlay visibility from clip behavior.
    scrollbars_hidden_if_overlay_: bool,
    // Source: MacScrollbarAnimatorV2 / OverlayScrollbarAnimatorMac. The
    // animator is owned by ScrollableArea and its opacity outlives individual
    // Paint/cc adapters. Track alpha remains zero until hover support is wired;
    // scrolling shows only the native overlay thumb.
    mac_horizontal_thumb_alpha_: f32,
    mac_vertical_thumb_alpha_: f32,
    mac_horizontal_track_alpha_: f32,
    mac_vertical_track_alpha_: f32,
    mac_horizontal_thumb_width_: Option<i32>,
    mac_vertical_thumb_width_: Option<i32>,
    uses_composited_scrolling_: bool,
    mouse_over_scrollbar_: bool,
    scrollbar_captured_: bool,
    has_been_disposed_: bool,
    needs_scroll_offset_clamp_: bool,
    // Source: core/paint/paint_layer_scrollable_area.h:849-851.
    in_resize_mode_: bool,
    scrolls_overflow_: bool,
    // Source: core/paint/paint_layer_scrollable_area.h:904-907.
    hypothetical_horizontal_scrollbar_thickness_: i32,
    hypothetical_vertical_scrollbar_thickness_: i32,
    hypothetical_scrollbar_thicknesses_are_valid_: bool,
    // LayoutObject construction installs style before the extracted host
    // publishes the root's immutable Frame/Settings inputs. Retain the source
    // lifecycle call and replay it immediately after PrepareStandaloneLayout.
    pending_after_style_change_: bool,
    pending_after_style_change_old_style_: Member<ComputedStyle>,
    // The source reads DocumentLifecycle in ScrollbarFrameRectChanged(). This
    // extracted owner has exactly two frame-positioning entry points, so retain
    // the equivalent phase for the duration of the callback chain.
    positioning_overflow_controls_in_prepaint_: bool,
    // Source: core/paint/paint_layer_scrollable_area.h. The scroll owner keeps
    // the anchor state; Layout only queues its end-of-frame adjustment.
    scroll_anchor_: ScrollAnchor,
    pending_history_restore_: Option<NativePendingHistoryRestore>,
    // Frame-owned VisualViewport geometry crosses only as these raw facts.
    // ShouldSupplyScrollbarsForVisualViewport retains the source decision.
    visual_viewport_scrollability_: Option<(bool, bool, bool)>,
    // ScrollableArea::RareData source owner, never reconstructed by Page.
    snap_container_data_: Option<scroll_snap::scroll_snap_data::SnapContainerData>,
    targeted_snap_area_id_: Option<scroll_snap::ElementId>,
    scrollsnapchanging_target_ids_: Option<scroll_snap::scroll_snap_data::TargetSnapAreaElementIds>,
    scrollsnapchange_target_ids_: Option<scroll_snap::scroll_snap_data::TargetSnapAreaElementIds>,
    snapped_query_target_ids_: Option<scroll_snap::scroll_snap_data::TargetSnapAreaElementIds>,
    snap_target_removal_pending_: bool,
    registered_view_: foundation::WeakMember<super::super::layout_view::LayoutView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeScrollbarExistenceOption {
    kDependsOnOverflow,
    kOverflowIndependent,
}

impl PaintLayerScrollableArea {
    pub fn new(box_: &mut LayoutBox) -> Self {
        let mut area = Self {
            overflow_rect_: PhysicalRect::default(),
            scroll_origin_: gfx::Point::default(),
            scroll_origin_changed_: false,
            has_horizontal_scrollbar_: false,
            has_vertical_scrollbar_: false,
            horizontal_scrollbar_: Member::default(),
            vertical_scrollbar_: Member::default(),
            hypothetical_horizontal_style_owner_: Cell::new(Member::default()),
            hypothetical_vertical_style_owner_: Cell::new(Member::default()),
            scroll_corner_client_: Member::default(),
            scroll_corner_: Cell::new(Member::default()),
            scroll_corner_style_pending_: Cell::new(false),
            scrolling_background_client_: Member::default(),
            box_: Member::from_ptr(box_),
            scroll_offset_: ScrollOffset::default(),
            text_overflow_snapshot_: Member::default(),
            // Source: core/scroll/scrollable_area.cc:119.
            scrollbars_hidden_if_overlay_: true,
            mac_horizontal_thumb_alpha_: 0.0,
            mac_vertical_thumb_alpha_: 0.0,
            mac_horizontal_track_alpha_: 0.0,
            mac_vertical_track_alpha_: 0.0,
            mac_horizontal_thumb_width_: None,
            mac_vertical_thumb_width_: None,
            uses_composited_scrolling_: false,
            mouse_over_scrollbar_: false,
            scrollbar_captured_: false,
            has_been_disposed_: false,
            needs_scroll_offset_clamp_: false,
            in_resize_mode_: false,
            scrolls_overflow_: false,
            hypothetical_horizontal_scrollbar_thickness_: 0,
            hypothetical_vertical_scrollbar_thickness_: 0,
            hypothetical_scrollbar_thicknesses_are_valid_: false,
            pending_after_style_change_: false,
            pending_after_style_change_old_style_: Member::default(),
            positioning_overflow_controls_in_prepaint_: false,
            scroll_anchor_: ScrollAnchor::default(),
            pending_history_restore_: None,
            visual_viewport_scrollability_: None,
            snap_container_data_: None,
            targeted_snap_area_id_: None,
            scrollsnapchanging_target_ids_: None,
            scrollsnapchange_target_ids_: None,
            snapped_query_target_ids_: None,
            snap_target_removal_pending_: false,
            registered_view_: foundation::WeakMember::default(),
        };
        // Source: core/paint/paint_layer_scrollable_area.cc:170-176.
        let element = DynamicTo::<Element>(box_.GetNode());
        if !element.is_null() {
            area.scroll_offset_ = unsafe { &*element }.SavedLayerScrollOffset();
            unsafe { &mut *element }.SetSavedLayerScrollOffset(ScrollOffset::default());
        }
        area
    }

    pub fn InitializeScrollingBackgroundDisplayItemClient(&mut self) {
        assert!(self.scrolling_background_client_.Get().is_null());
        assert!(self.scroll_corner_client_.Get().is_null());
        self.scrolling_background_client_ = Member::from_ptr(MakeGarbageCollected(
            ScrollingBackgroundDisplayItemClient::new(self as *mut _),
        ));
        self.scroll_corner_client_ = Member::from_ptr(MakeGarbageCollected(
            ScrollCornerDisplayItemClient::new(self as *mut _),
        ));
    }

    pub fn GetScrollingBackgroundDisplayItemClient(
        &self,
    ) -> &foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient {
        let client = self.scrolling_background_client_.Get();
        assert!(!client.is_null());
        unsafe { &*client }.DisplayItemClient()
    }

    pub fn HorizontalScrollbar(&self) -> *mut Scrollbar {
        if self.has_horizontal_scrollbar_ {
            self.horizontal_scrollbar_.Get()
        } else {
            std::ptr::null_mut()
        }
    }
    pub fn VerticalScrollbar(&self) -> *mut Scrollbar {
        if self.has_vertical_scrollbar_ {
            self.vertical_scrollbar_.Get()
        } else {
            std::ptr::null_mut()
        }
    }

    // Source: core/scroll/scrollable_area.h:419-421.
    pub fn ScrollbarsHiddenIfOverlay(&self) -> bool {
        self.scrollbars_hidden_if_overlay_
    }

    // Source: core/scroll/scrollable_area.cc:911-932. The compositor passes the
    // requested state; the ScrollableArea owner performs theme admission and
    // the VisualViewport-supply exclusion before retaining it.
    pub fn SetScrollbarsHiddenIfOverlayFromOwner(&mut self, hidden: bool) -> bool {
        if self.HasBeenDisposed()
            || !unsafe { &*self.GetLayoutBox() }
                .ScrollbarThemeForLayout()
                .uses_overlay_scrollbars
        {
            return false;
        }
        let hidden = hidden && !self.ShouldAvoidHidingOverlayScrollbars();
        if self.scrollbars_hidden_if_overlay_ == hidden {
            return false;
        }
        self.scrollbars_hidden_if_overlay_ = hidden;
        self.ScrollbarVisibilityChanged();
        true
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:768-773. Clearing the
    // FrameView hit-test cache at 775-776 remains a host callback and is not
    // replaced with a Layout-side approximation.
    pub fn ScrollbarVisibilityChanged(&mut self) {
        self.UpdateScrollbarEnabledState(false, false);
        unsafe { &mut *self.GetLayoutBox() }.SetNeedsPaintPropertyUpdate();
    }
    pub fn ScrollCornerDisplayItemClient(
        &self,
    ) -> Option<&foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient>
    {
        let client = self.scroll_corner_client_.Get();
        if client.is_null() {
            None
        } else {
            Some(unsafe { &*client }.DisplayItemClient())
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:188-191.
    pub fn FromNode(node: &Node) -> *mut Self {
        let box_ = node.GetLayoutBox();
        if box_.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*box_ }.GetScrollableArea()
        }
    }

    pub fn GetLayoutBox(&self) -> *mut LayoutBox {
        self.box_.Get()
    }

    pub fn GetScrollAnchor(&mut self) -> &mut ScrollAnchor {
        &mut self.scroll_anchor_
    }

    pub fn NotifyScrollAnchorBeforeLayout(&mut self) {
        let area = self as *mut Self;
        unsafe { &mut (*area).scroll_anchor_ }.NotifyBeforeLayout(unsafe { &*area });
    }

    pub fn AdjustScrollAnchor(&mut self) {
        let area = self as *mut Self;
        unsafe { &mut (*area).scroll_anchor_ }.Adjust(unsafe { &mut *area });
    }

    pub fn ScrollAnchorRefersTo(&self, object: *const LayoutObject) -> bool {
        self.scroll_anchor_.RefersTo(object)
    }

    pub fn NotifyScrollAnchorObjectRemoved(&mut self, object: *mut LayoutObject) {
        self.scroll_anchor_.NotifyRemoved(object);
    }

    pub fn ClearScrollAnchorForExplicitScroll(&mut self) {
        let node = unsafe { &*self.GetLayoutBox() }.GetNode();
        browser_tracing::instant(
            "layout",
            "ScrollAnchor.ClearExplicit",
            &[(
                "scroll_node",
                if node.is_null() {
                    0.0
                } else {
                    unsafe { &*node }.InputId() as f64
                },
            )],
        );
        self.scroll_anchor_.Clear();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1300-1309.
    pub fn LocalToVisibleContentQuad(
        &self,
        quad: &gfx::QuadF,
        local_object: &LayoutObject,
        flags: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        let box_ = self.GetLayoutBox();
        if box_.is_null() {
            return *quad;
        }
        local_object.LocalToAncestorQuad(quad, box_.cast(), flags)
    }

    // Source: third_party/blink/renderer/core/paint/paint_layer_scrollable_area.cc:588-618.
    pub fn BackgroundNeedsRepaintOnScroll(&self) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let location = box_.GetBackgroundPaintLocation();
        let in_border_box =
            location & super::super::layout_object::kBackgroundPaintInBorderBoxSpace != 0;
        let in_contents =
            location & super::super::layout_object::kBackgroundPaintInContentsSpace != 0;
        let layers = box_.StyleRef().BackgroundLayers();
        if layers.AnyLayerHasLocalAttachmentImage() && in_border_box {
            return true;
        }
        if layers.AnyLayerHasDefaultAttachmentImage() && in_contents {
            return true;
        }
        if layers.AnyLayerHasLocalAttachment()
            && layers.AnyLayerUsesContentBox()
            && in_border_box
            && !box_.PaddingOutsets().IsZero()
        {
            return true;
        }
        false
    }

    // Source: third_party/blink/renderer/core/paint/paint_layer_scrollable_area.cc:2776-2797.
    pub fn PrefersNonCompositedScrolling(&self) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_
            .InputOwnerForLayout()
            .InputPreferNonCompositedScrollingEnabled()
        {
            return true;
        }
        let node = box_.GetNode();
        if !node.is_null() {
            let node = unsafe { &*node };
            if node.IsElementNode() {
                let element = unsafe { &*(node as *const Node as *const Element) };
                if element.GetElementType()
                    == super::super::layout_node_metadata::ElementType::kHTMLSelectElement
                {
                    return true;
                }
            }
            // Source: core/html/forms/text_control_element.cc:1271-1279.
            // Chromium metadata preserves the actual UA shadow host allocation.
            let host = node.OwnerShadowHost();
            if !host.is_null()
                && node.IsInUserAgentShadowRoot()
                && unsafe { &*host }.GetElementType()
                    == super::super::layout_node_metadata::ElementType::kHTMLInputElement
            {
                return true;
            }
        }
        if box_.CanvasDrawElementEnabledForLayout() && box_.InputCanvasLayoutSubtree() {
            return true;
        }
        false
    }

    pub fn GetScrollOffset(&self) -> ScrollOffset {
        self.scroll_offset_
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1399-1413.
    pub fn CanPropagateScroll(&self) -> bool {
        let box_ = self.GetLayoutBox();
        if box_.is_null() {
            return true;
        }
        let box_ = unsafe { &*box_ };
        if !box_.IsScrollContainer() || box_.StyleRef().Visibility() != EVisibility::kVisible {
            return true;
        }
        let style = box_.StyleRef();
        let propagates = |behavior| {
            matches!(
                behavior,
                EOverscrollBehavior::kAuto | EOverscrollBehavior::kChain
            )
        };
        propagates(style.OverscrollBehaviorX()) && propagates(style.OverscrollBehaviorY())
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1371-1374.
    pub fn IsGlobalRootNonOverlayScroller(&self) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        box_.IsGlobalRootScroller() && !box_.ScrollbarThemeForLayout().uses_overlay_scrollbars
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1349-1369.
    pub fn UsedColorSchemeScrollbarsChanged(&self, old_style: *const ComputedStyle) -> bool {
        if old_style.is_null() {
            return false;
        }
        let old_style = unsafe { &*old_style };
        let style = unsafe { &*self.GetLayoutBox() }.StyleRef();
        if old_style.UsedColorScheme() != style.UsedColorScheme() {
            return true;
        }
        if self.IsGlobalRootNonOverlayScroller()
            && old_style.ColorSchemeFlagsIsNormal() != style.ColorSchemeFlagsIsNormal()
        {
            return true;
        }
        false
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:427-432,449-453.
    // The FrameView owns scroll events, intersection-observer deltas and the
    // remaining host notifications; the source owner still performs the
    // source early return, property invalidation and offset commit here.
    pub(crate) fn UpdateScrollOffset(
        &mut self,
        offset: ScrollOffset,
        scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType,
    ) {
        if self.has_been_disposed_ || self.scroll_offset_ == offset {
            return;
        }
        let old_offset = self.scroll_offset_;
        unsafe { &mut *self.GetLayoutBox() }.SetNeedsPaintPropertyUpdate();
        self.scroll_offset_ = offset;
        let minimum = self.MinimumScrollOffset();
        let horizontal = self.HorizontalScrollbar();
        if !horizontal.is_null() {
            unsafe { &mut *horizontal }
                .OffsetDidChangeWithType(offset.x() - minimum.x(), scroll_type);
        }
        let vertical = self.VerticalScrollbar();
        if !vertical.is_null() {
            unsafe { &mut *vertical }
                .OffsetDidChangeWithType(offset.y() - minimum.y(), scroll_type);
        }
        let explicit = scroll_type
            == layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kUser
            || scroll_type
                == layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic
            || scroll_type
                == layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kCompositor;
        if explicit {
            self.DidChangeUserVisibleScrollOffset(ScrollOffset::new(
                offset.x() - old_offset.x(),
                offset.y() - old_offset.y(),
            ));
        }
    }

    // Source: core/scroll/mac_scrollbar_animator_impl.mm:190-196 and
    // ui/native_theme/scrollbar_animator_mac.cc:118-154.
    fn DidChangeUserVisibleScrollOffset(&mut self, delta: ScrollOffset) {
        for (changed, scrollbar) in [
            (delta.x() != 0., self.HorizontalScrollbar()),
            (delta.y() != 0., self.VerticalScrollbar()),
        ] {
            if changed {
                self.EmitScrollbarActivity(scrollbar, NativeScrollbarActivityKind::Scrolled);
            }
        }
    }
    fn EmitScrollbarActivity(&self, scrollbar: *mut Scrollbar, kind: NativeScrollbarActivityKind) {
        let Some(bar) = (unsafe { scrollbar.as_ref() }) else {
            return;
        };
        if bar.IsCustomScrollbar() || !bar.ThemeMetrics().uses_macos_painted_scrollbars {
            return;
        }
        if let Some(view) = unsafe { self.registered_view_.Get().as_ref() } {
            view.ScrollbarActivityQueue()
                .borrow_mut()
                .push(NativeScrollbarActivity {
                    scrollbar: foundation::Persistent::from_ptr(scrollbar),
                    area: self.GetScrollElementId(),
                    kind,
                });
        }
    }

    fn HasMacOverlayScrollbarAnimator(scrollbar: *mut Scrollbar) -> bool {
        let Some(bar) = (unsafe { scrollbar.as_ref() }) else {
            return false;
        };
        // MacScrollbarImplV2 creates OverlayScrollbarAnimatorMac only for a
        // registered macOS overlay scrollbar. Custom scrollbars are not
        // registered with ScrollbarThemeMac.
        !bar.IsCustomScrollbar()
            && bar.ThemeMetrics().uses_macos_painted_scrollbars
            && bar.IsOverlayScrollbar()
    }

    // Source: core/scroll/scrollable_area.cc:168-180 and
    // core/scroll/mac_scrollbar_animator_impl.mm:198-218.
    pub fn FadeInScrollbarIfExists(&self, horizontal: bool, vertical: bool) -> bool {
        if self.HasBeenDisposed() {
            return false;
        }
        let mut did_fade_in_and_begin_deferring_fade_out = false;
        for (requested, scrollbar) in [
            (horizontal, self.HorizontalScrollbar()),
            (vertical, self.VerticalScrollbar()),
        ] {
            if requested && Self::HasMacOverlayScrollbarAnimator(scrollbar) {
                self.EmitScrollbarActivity(scrollbar, NativeScrollbarActivityKind::FadeInDeferred);
                did_fade_in_and_begin_deferring_fade_out = true;
            }
        }
        did_fade_in_and_begin_deferring_fade_out
    }

    pub fn FadeOutScrollbarIfNeeded(&self) {
        if self.HasBeenDisposed() {
            return;
        }
        for scrollbar in [self.HorizontalScrollbar(), self.VerticalScrollbar()] {
            if Self::HasMacOverlayScrollbarAnimator(scrollbar) {
                self.EmitScrollbarActivity(scrollbar, NativeScrollbarActivityKind::FadeOutIfNeeded);
            }
        }
    }

    pub fn ScrollbarPaintedOpacity(&self, orientation: ScrollbarOrientation) -> f32 {
        let theme = unsafe { &*self.GetLayoutBox() }.ScrollbarThemeForLayout();
        if !theme.uses_macos_painted_scrollbars {
            return 1.0;
        }
        match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => self.mac_horizontal_thumb_alpha_,
            ScrollbarOrientation::kVerticalScrollbar => self.mac_vertical_thumb_alpha_,
        }
    }

    pub fn ScrollbarPaintedThumbWidth(&self, scrollbar: &Scrollbar) -> i32 {
        if !scrollbar.ThemeMetrics().uses_macos_painted_scrollbars {
            return scrollbar.ScrollbarThickness();
        }
        let animated = if scrollbar.Orientation() == ScrollbarOrientation::kHorizontalScrollbar {
            self.mac_horizontal_thumb_width_
        } else {
            self.mac_vertical_thumb_width_
        };
        if let Some(width) = animated {
            return width;
        }
        match scrollbar.CSSScrollbarWidth() {
            EScrollbarWidth::kNone => 0,
            EScrollbarWidth::kThin => 10,
            EScrollbarWidth::kAuto => 12,
        }
    }

    // Source: core/layout/scroll_anchor.cc:735-738 and
    // core/scroll/scrollable_area.cc:450-453. An anchoring adjustment is a
    // real ScrollableArea offset change. Keep the resident area and its
    // existing layout input owner on the same value so the ensuing paint
    // property commit and Page's post-layout reflection acknowledge one
    // offset instead of restoring the pre-adjustment compositor value.
    pub(crate) fn SetScrollOffsetFromAnchoring(&mut self, offset: ScrollOffset) {
        let before = self.GetScrollOffset();
        self.UpdateScrollOffset(
            offset,
            layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kAnchoring,
        );
        if self.GetScrollOffset() == before {
            return;
        }
        let node = unsafe { &*self.GetLayoutBox() }.GetNode();
        if !node.is_null() {
            unsafe { &mut *node }.ReflectInputScrollOffset(super::super::layout_input::Offset {
                x: f64::from(offset.x()),
                y: f64::from(offset.y()),
            });
        }
    }

    pub fn GetScrollElementId(&self) -> CompositorElementId {
        let box_ = unsafe { &*self.GetLayoutBox() };
        CompositorElementIdFromUniqueObjectId(
            box_.UniqueId(),
            CompositorElementIdNamespace::kScroll,
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1942-1947.
    pub fn GetScrollCornerElementId(&self) -> CompositorElementId {
        let box_ = unsafe { &*self.GetLayoutBox() };
        CompositorElementIdFromUniqueObjectId(
            box_.UniqueId(),
            CompositorElementIdNamespace::kScrollCorner,
        )
    }

    pub fn ScrollOffsetInt(&self) -> gfx::Vector2d {
        SnapScrollOffsetToPhysicalPixels(&self.scroll_offset_)
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:419-425.
    pub fn ScrollSize(&self, orientation: ScrollbarOrientation) -> i32 {
        let dimensions = self.MaximumScrollOffsetInt() - self.MinimumScrollOffsetInt();
        if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            dimensions.x()
        } else {
            dimensions.y()
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:831-859.
    pub fn UserInputScrollable(&self, orientation: ScrollbarOrientation) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if orientation == ScrollbarOrientation::kVerticalScrollbar
            && box_.InputOwnerForLayout().InputVerticalScrollEnforced()
        {
            return false;
        }
        if box_.IsLayoutView() {
            if box_
                .InputOwnerForLayout()
                .InputFullscreenElementBlocksLayoutViewScroll()
            {
                return false;
            }
            let (horizontal, vertical) = ResolveScrollbarModes(box_);
            let mode = if orientation == ScrollbarOrientation::kHorizontalScrollbar {
                horizontal
            } else {
                vertical
            };
            return matches!(mode, ScrollbarMode::kAuto | ScrollbarMode::kAlwaysOn);
        }
        let overflow = if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            box_.StyleRef().OverflowX()
        } else {
            box_.StyleRef().OverflowY()
        };
        matches!(
            overflow,
            EOverflow::kScroll | EOverflow::kAuto | EOverflow::kOverlay
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:861-863.
    pub fn ShouldPlaceVerticalScrollbarOnLeft(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.ShouldPlaceBlockDirectionScrollbarOnLogicalLeft()
    }

    // Source: core/scroll/scrollable_area.cc:1422-1427. The browser publishes
    // ChromeClient's window-to-viewport scale as the immutable device-pixel
    // ratio on Layout's document owner.
    pub fn ScaleFromDIP(&self) -> f64 {
        unsafe { &*self.GetLayoutBox() }.DevicePixelRatioForLayout()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:303-365.
    pub fn CornerRect(&self) -> gfx::Rect {
        let vertical = self.VerticalScrollbar();
        let horizontal = self.HorizontalScrollbar();
        let has_custom_scrollbar = (!vertical.is_null()
            && unsafe { &*vertical }.IsCustomScrollbar())
            || (!horizontal.is_null() && unsafe { &*horizontal }.IsCustomScrollbar());
        let box_ = unsafe { &*self.GetLayoutBox() };
        let use_fixed_size = foundation::RuntimeEnabledFeatures::TextAreaResizerFixedSizeEnabled()
            && box_.ScrollbarThemeForLayout().uses_overlay_scrollbars
            && box_.CanResize()
            && !has_custom_scrollbar;
        let (horizontal_thickness, vertical_thickness) = if use_fixed_size {
            let thickness = (15.0 * self.ScaleFromDIP()).round() as i32;
            (thickness, thickness)
        } else if vertical.is_null() && horizontal.is_null() {
            let thickness =
                ScrollbarThemeThickness(box_.ScrollbarThemeForLayout(), EScrollbarWidth::kAuto);
            (thickness, thickness)
        } else if horizontal.is_null() {
            let thickness = unsafe { &*vertical }.ScrollbarThickness();
            (thickness, thickness)
        } else if vertical.is_null() {
            let thickness = unsafe { &*horizontal }.ScrollbarThickness();
            (thickness, thickness)
        } else {
            (
                unsafe { &*vertical }.ScrollbarThickness(),
                unsafe { &*horizontal }.ScrollbarThickness(),
            )
        };
        let border_box_size = self.PixelSnappedBorderBoxSize();
        let left = if box_.ShouldPlaceBlockDirectionScrollbarOnLogicalLeft() {
            box_.StyleRef().BorderLeftWidth()
        } else {
            border_box_size.width() - horizontal_thickness - box_.StyleRef().BorderRightWidth()
        };
        gfx::Rect::new(
            gfx::Point::new(
                left,
                border_box_size.height() - vertical_thickness - box_.StyleRef().BorderBottomWidth(),
            ),
            gfx::Size::new(horizontal_thickness, vertical_thickness),
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2283-2301.
    pub fn ResizerCornerRect(&self, hit_test_type: ResizerHitTestType) -> gfx::Rect {
        if !unsafe { &*self.GetLayoutBox() }.CanResize() {
            return gfx::Rect::default();
        }
        let corner = self.CornerRect();
        if hit_test_type == ResizerHitTestType::kResizerForTouch {
            ExpandResizerCornerRectForTouch(corner)
        } else {
            corner
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2304-2308.
    pub fn ScrollCornerAndResizerRect(&self) -> gfx::Rect {
        ScrollCornerOrResizerRect(
            self.ScrollCornerRect(),
            self.ResizerCornerRect(ResizerHitTestType::kResizerForPointer),
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2324-2332.
    pub fn IsLocalPointInResizeControl(
        &self,
        local_point: &gfx::Point,
        hit_test_type: ResizerHitTestType,
    ) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_.StyleRef().Visibility() != EVisibility::kVisible || !box_.CanResize() {
            return false;
        }
        RectContainsPoint(self.ResizerCornerRect(hit_test_type), local_point)
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2311-2322.
    pub fn IsAbsolutePointInResizeControl(
        &self,
        absolute_point: &gfx::Point,
        hit_test_type: ResizerHitTestType,
    ) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_.StyleRef().Visibility() != EVisibility::kVisible || !box_.CanResize() {
            return false;
        }
        let absolute = PhysicalOffset::new(
            LayoutUnit::from_signed(absolute_point.x()),
            LayoutUnit::from_signed(absolute_point.y()),
        );
        let local = ToRoundedPoint(&box_.AbsoluteToLocalPointPhysical(
            &absolute,
            super::super::map_coordinates_flags::MapCoordinatesFlags::default(),
        ));
        RectContainsPoint(self.ResizerCornerRect(hit_test_type), &local)
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:367-381.
    pub fn ScrollCornerRect(&self) -> gfx::Rect {
        let has_horizontal = self.HasHorizontalScrollbar();
        let has_vertical = self.HasVerticalScrollbar();
        let has_resizer = unsafe { &*self.GetLayoutBox() }.CanResize();
        if (has_horizontal && has_vertical) || (has_resizer && (has_horizontal || has_vertical)) {
            self.CornerRect()
        } else {
            gfx::Rect::default()
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:299-301.
    pub fn IsScrollCornerVisible(&self) -> bool {
        !self.ScrollCornerRect().IsEmpty()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1496-1537.
    pub fn RectForHorizontalScrollbar(&self) -> gfx::Rect {
        if !self.HasHorizontalScrollbar() {
            return gfx::Rect::default();
        }
        let corner = self.ScrollCornerRect();
        let border_box_size = self.PixelSnappedBorderBoxSize();
        let border = unsafe { &*self.GetLayoutBox() }.BorderOutsets();
        let mut left = border.left.ToInt();
        if self.ShouldPlaceVerticalScrollbarOnLeft() {
            left += if self.HasVerticalScrollbar() {
                unsafe { &*self.VerticalScrollbar() }.ScrollbarThickness()
            } else {
                corner.width()
            };
        }
        let thickness = unsafe { &*self.HorizontalScrollbar() }.ScrollbarThickness();
        gfx::Rect::new(
            gfx::Point::new(
                left,
                border_box_size.height() - border.bottom.ToInt() - thickness,
            ),
            gfx::Size::new(
                border_box_size.width() - border.HorizontalSum().ToInt() - corner.width(),
                thickness,
            ),
        )
    }

    pub fn RectForVerticalScrollbar(&self) -> gfx::Rect {
        if !self.HasVerticalScrollbar() {
            return gfx::Rect::default();
        }
        let corner = self.ScrollCornerRect();
        let border_box_size = self.PixelSnappedBorderBoxSize();
        let border = unsafe { &*self.GetLayoutBox() }.BorderOutsets();
        let thickness = unsafe { &*self.VerticalScrollbar() }.ScrollbarThickness();
        let left = if self.ShouldPlaceVerticalScrollbarOnLeft() {
            border.left.ToInt()
        } else {
            border_box_size.width() - border.right.ToInt() - thickness
        };
        gfx::Rect::new(
            gfx::Point::new(left, border.top.ToInt()),
            gfx::Size::new(
                thickness,
                border_box_size.height() - border.VerticalSum().ToInt() - corner.height(),
            ),
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2179-2194.
    // Custom parts share the same native frame and canonical FragmentData.
    pub fn PositionOverflowControls(&mut self) {
        self.PositionOverflowControlsForPhase(false);
    }

    // PaintPropertyTreeBuilder calls the same Chromium method while the
    // document lifecycle is kInPrePaint. Preserve that source phase without
    // leaking lifecycle ownership into the immutable fragment payload.
    pub fn PositionOverflowControlsForPrePaint(&mut self) {
        self.PositionOverflowControlsForPhase(true);
    }

    fn PositionOverflowControlsForPhase(&mut self, in_prepaint: bool) {
        if !self.HasOverflowControls() {
            return;
        }
        debug_assert!(!self.positioning_overflow_controls_in_prepaint_);
        self.positioning_overflow_controls_in_prepaint_ = in_prepaint;
        let vertical = self.VerticalScrollbar();
        if !vertical.is_null() {
            let rect = self.RectForVerticalScrollbar();
            unsafe { &*vertical }.SetFrameRect(&rect);
            unsafe { &*vertical }.PositionCustomScrollbarParts();
        }
        let horizontal = self.HorizontalScrollbar();
        if !horizontal.is_null() {
            let rect = self.RectForHorizontalScrollbar();
            unsafe { &*horizontal }.SetFrameRect(&rect);
            unsafe { &*horizontal }.PositionCustomScrollbarParts();
        }
        if let Some(corner) = unsafe { self.scroll_corner_.get().Get().as_mut() } {
            corner.SetPartFrameRect(&self.ScrollCornerRect());
        }
        self.positioning_overflow_controls_in_prepaint_ = false;
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:779-797. Chromium
    // reaches this through Scrollbar::SetFrameRect; keep the same callback
    // boundary and its narrower invalidation while positioning in PrePaint.
    pub fn ScrollbarFrameRectChanged(&mut self) {
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        if self.positioning_overflow_controls_in_prepaint_ {
            box_.GetMutableForPainting()
                .SetOnlyThisNeedsPaintPropertyUpdate();
        } else {
            box_.SetNeedsPaintPropertyUpdate();
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1539-1552.
    pub fn ScrollbarOffset(&self, scrollbar: &Scrollbar) -> gfx::Vector2d {
        let scrollbar_ptr = scrollbar as *const Scrollbar as *mut Scrollbar;
        if scrollbar_ptr == self.VerticalScrollbar() {
            let rect = self.RectForVerticalScrollbar();
            return gfx::Vector2d::new(rect.x(), rect.y());
        }
        if scrollbar_ptr == self.HorizontalScrollbar() {
            let rect = self.RectForHorizontalScrollbar();
            return gfx::Vector2d::new(rect.x(), rect.y());
        }
        panic!("ScrollbarOffset requires an attached scrollbar owned by this area");
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1316-1319.
    pub fn ScrollBehaviorStyle(
        &self,
    ) -> layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior {
        unsafe { &*self.GetLayoutBox() }
            .StyleRef()
            .GetScrollBehavior()
    }

    pub fn MinimumScrollOffsetInt(&self) -> gfx::Vector2d {
        self.ClampNonScrollableAxesOffsets(-self.ScrollOrigin().OffsetFromOrigin())
    }

    pub fn MaximumScrollOffsetInt(&self) -> gfx::Vector2d {
        self.ClampNonScrollableAxesOffsets(
            -self.ScrollOrigin().OffsetFromOrigin() + self.ComputeScrollableSize(),
        )
    }

    pub fn MinimumScrollOffset(&self) -> ScrollOffset {
        ScrollOffset::from(self.MinimumScrollOffsetInt())
    }

    pub fn MaximumScrollOffset(&self) -> ScrollOffset {
        ScrollOffset::from(self.MaximumScrollOffsetInt())
    }

    pub fn ClampScrollOffsetInt(&self, scroll_offset: &gfx::Vector2d) -> gfx::Vector2d {
        let mut result = *scroll_offset;
        result.SetToMin(&self.MaximumScrollOffsetInt());
        result.SetToMax(&self.MinimumScrollOffsetInt());
        result
    }

    pub fn ClampScrollOffset(&self, scroll_offset: &ScrollOffset) -> ScrollOffset {
        let mut result = *scroll_offset;
        result.SetToMin(&self.MaximumScrollOffset());
        result.SetToMax(&self.MinimumScrollOffset());
        result
    }

    pub fn ScrollOrigin(&self) -> gfx::Point {
        self.scroll_origin_
    }

    pub fn ScrollOriginChanged(&self) -> bool {
        self.scroll_origin_changed_
    }

    pub fn ResetScrollOriginChanged(&mut self) {
        self.scroll_origin_changed_ = false;
    }

    pub fn ScrollOffsetToPosition(&self, offset: &ScrollOffset) -> gfx::PointF {
        gfx::PointF::from_point(self.ScrollOrigin()) + *offset
    }

    pub fn ScrollPositionToOffset(&self, position: &gfx::PointF) -> ScrollOffset {
        *position - gfx::PointF::from_point(self.ScrollOrigin())
    }

    pub fn ScrollPosition(&self) -> gfx::PointF {
        self.ScrollOffsetToPosition(&self.GetScrollOffset())
    }

    pub fn ScrollWidth(&self) -> LayoutUnit {
        self.overflow_rect_.Width()
    }

    pub fn ScrollHeight(&self) -> LayoutUnit {
        self.overflow_rect_.Height()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:900-933.
    pub fn InitializeResizeTransform(
        &self,
        absolute_drag_start_point: &gfx::Point,
    ) -> gfx::Transform {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let mut resize_transform = box_.LocalToAbsoluteTransform(MapCoordinatesFlags::default());

        if box_.ShouldPlaceBlockDirectionScrollbarOnLogicalLeft() {
            resize_transform.Translate(self.PixelSnappedBorderBoxSize().width() as f32, 0.0);
            resize_transform.Scale(-1.0, 1.0);
        }

        resize_transform = resize_transform.InverseOrIdentity();
        let drag_start_position = gfx::point_conversions::ToRoundedPoint(
            resize_transform.ProjectPoint(gfx::PointF::new(
                absolute_drag_start_point.x() as f32,
                absolute_drag_start_point.y() as f32,
            )),
        );
        let current_size = box_.StitchedSize();
        resize_transform.PostTranslate(
            current_size.width.ToFloat() - drag_start_position.x() as f32,
            current_size.height.ToFloat() - drag_start_position.y() as f32,
        );
        resize_transform
    }

    // Source: core/paint/paint_layer_scrollable_area.h:431-434.
    pub fn InResizeMode(&self) -> bool {
        self.in_resize_mode_
    }

    pub fn SetInResizeMode(&mut self, in_resize_mode: bool) {
        self.in_resize_mode_ = in_resize_mode;
    }

    // Source paint_layer_scrollable_area.cc:2442-2515. The FrameView
    // conversion is completed by the embedding before this native owner runs.
    pub fn PlanResizeInContainingFrame(
        &self,
        point: gfx::Point,
        position_to_size: &gfx::Transform,
    ) -> Option<NativeResizePlan> {
        let box_ = unsafe { self.GetLayoutBox().as_ref() }?;
        let node = unsafe { box_.GetNode().as_ref() }?;
        if !self.InResizeMode() || !box_.CanResize() {
            return None;
        }
        assert!(node.IsElementNode());
        let zoom = box_.StyleRef().EffectiveZoom();
        let mut current = box_.StitchedSize();
        current.ScaleFloat(1.0 / zoom);
        let local = gfx::point_conversions::ToRoundedPoint(
            position_to_size.ProjectPoint(gfx::PointF::new(point.x() as f32, point.y() as f32)),
        );
        let corner = self.CornerRect().size();
        let width = LayoutUnit::from_signed((local.x() as f32 / zoom) as i32)
            .max(LayoutUnit::from_f32(corner.width() as f32 / zoom));
        let height = LayoutUnit::from_signed((local.y() as f32 / zoom) as i32)
            .max(LayoutUnit::from_f32(corner.height() as f32 / zoom));
        let difference = PhysicalSize::new(width - current.width, height - current.height);
        let border_box = box_.StyleRef().BoxSizing() == foundation::EBoxSizing::kBorderBox;
        let border_padding = box_.BorderOutsets() + box_.PaddingOutsets();
        let resize = box_.StyleRef().UsedResize();
        let base_width = box_.StitchedSize().width
            - if border_box {
                LayoutUnit::default()
            } else {
                border_padding.HorizontalSum()
            };
        let base_height = box_.StitchedSize().height
            - if border_box {
                LayoutUnit::default()
            } else {
                border_padding.VerticalSum()
            };
        Some(NativeResizePlan {
            node_id: node.InputId(),
            width_px: (resize != foundation::EResize::kVertical
                && difference.width != LayoutUnit::default())
            .then(|| {
                (LayoutUnit::from_f32(base_width.ToFloat() / zoom) + difference.width).Round()
            }),
            height_px: (resize != foundation::EResize::kHorizontal
                && difference.height != LayoutUnit::default())
            .then(|| {
                (LayoutUnit::from_f32(base_height.ToFloat() / zoom) + difference.height).Round()
            }),
        })
    }

    pub fn Size(&self) -> PhysicalSize {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_.IsLayoutView() {
            let viewport_size = box_.ViewportGeometryForLayout().size;
            PhysicalSize::new(
                LayoutUnit::from_signed(viewport_size.width),
                LayoutUnit::from_signed(viewport_size.height),
            )
        } else {
            box_.StitchedSize()
        }
    }

    pub fn LayoutContentRect(&self, inclusion: IncludeScrollbarsInRect) -> PhysicalRect {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let layer_size = self.Size();
        let border_scrollbar = box_.BorderOutsets()
            + if inclusion == IncludeScrollbarsInRect::kExcludeScrollbars {
                box_.ComputeScrollbars()
            } else {
                PhysicalBoxStrut::default()
            };
        let mut size = PhysicalSize::new(
            layer_size.width - border_scrollbar.HorizontalSum(),
            layer_size.height - border_scrollbar.VerticalSum(),
        );
        size.ClampNegativeToZero();
        PhysicalRect::new(
            PhysicalOffset::FromPointFRound(&self.ScrollPosition()),
            size,
        )
    }

    pub fn LayoutContentRectDefault(&self) -> PhysicalRect {
        self.LayoutContentRect(IncludeScrollbarsInRect::kExcludeScrollbars)
    }

    pub fn VisibleContentRect(&self, inclusion: IncludeScrollbarsInRect) -> gfx::Rect {
        let rect = self.LayoutContentRect(inclusion);
        gfx::Rect::new(ToFlooredPoint(rect.offset), ToRoundedSize(rect.size))
    }

    pub fn VisibleScrollSnapportRect(&self, inclusion: IncludeScrollbarsInRect) -> PhysicalRect {
        let style = unsafe { &*self.GetLayoutBox() }.StyleRef();
        let mut content_rect = self.LayoutContentRect(inclusion);
        let scroll_origin = self.ScrollOrigin();
        content_rect.Move(&PhysicalOffset::new(
            LayoutUnit::from_signed(-scroll_origin.x()),
            LayoutUnit::from_signed(-scroll_origin.y()),
        ));
        let padding = PhysicalBoxStrut::new(
            MinimumValueForLength(style.ScrollPaddingTop(), content_rect.Height()),
            MinimumValueForLength(style.ScrollPaddingRight(), content_rect.Width()),
            MinimumValueForLength(style.ScrollPaddingBottom(), content_rect.Height()),
            MinimumValueForLength(style.ScrollPaddingLeft(), content_rect.Width()),
        );
        content_rect.Contract(&padding);
        content_rect
    }

    pub fn ContentsSize(&self) -> gfx::Size {
        self.PixelSnappedContentsSize(
            &unsafe { &*self.GetLayoutBox() }
                .PhysicalPaddingBoxRect()
                .offset,
        )
    }

    pub fn PixelSnappedContentsSize(&self, paint_offset: &PhysicalOffset) -> gfx::Size {
        let mut size = self.overflow_rect_.size;
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_.IsLayoutView() {
            if let Some(snapshot) = box_.ViewportGeometryForLayout().transition_snapshot_size {
                let container_size = PhysicalSize::new(
                    LayoutUnit::from_signed(snapshot.width),
                    LayoutUnit::from_signed(snapshot.height),
                );
                size.width = std::cmp::max(container_size.width, size.width);
                size.height = std::cmp::max(container_size.height, size.height);
            }
        }
        ToPixelSnappedRect(PhysicalRect::new(*paint_offset, size)).size()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3385-3394.
    pub fn PixelSnappedBorderBoxSize(&self) -> gfx::Size {
        let box_ = unsafe { &*self.GetLayoutBox() };
        ToPixelSnappedRect(PhysicalRect::new(
            box_.FirstFragment().PaintOffset(),
            box_.StitchedSize(),
        ))
        .size()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3407-3418.
    pub fn ScrollingBackgroundVisualRect(&self, paint_offset: &PhysicalOffset) -> gfx::Rect {
        let mut clip_rect = unsafe { &*self.GetLayoutBox() }.OverflowClipRect();
        clip_rect.Move(paint_offset);
        let overflow_clip_rect = ToPixelSnappedRect(clip_rect);
        let mut scroll_size = self.PixelSnappedContentsSize(&clip_rect.offset);
        scroll_size.SetToMax(&overflow_clip_rect.size());
        gfx::Rect::new(overflow_clip_rect.origin(), scroll_size)
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:729-735. The extracted
    // compositor and intersection-observer owners have no source hooks here;
    // retain the paint-property invalidation owned by LayoutBox.
    fn ContentsResized(&mut self) {
        unsafe { &mut *self.GetLayoutBox() }.SetNeedsPaintPropertyUpdate();
    }

    pub fn LocalToScrollOriginOffset(&self) -> PhysicalOffset {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let adjustment = box_.OriginAdjustmentForScrollbars();
        let mut border_to_scroll_origin = -box_.BorderOutsets().Offset();
        border_to_scroll_origin.left -= LayoutUnit::from_signed(adjustment.x());
        border_to_scroll_origin.top -= LayoutUnit::from_signed(adjustment.y());
        border_to_scroll_origin += PhysicalOffset::FromVector2dFFloor(&self.GetScrollOffset());
        border_to_scroll_origin
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3751-3782.
    pub fn ComputeScrollableSize(&self) -> gfx::Vector2d {
        let box_ = self.GetLayoutBox();
        if box_.is_null() || !unsafe { &*box_ }.IsScrollContainer() {
            return gfx::Vector2d::default();
        }
        let mut contents_size = self.ContentsSize();
        let visible_size = if let Some(size) = self.RootScrollerVisibleSizeForLayout() {
            size
        } else {
            ToRoundedSize(
                unsafe { &*box_ }
                    .OverflowClipRectWithBehavior(
                        OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
                    )
                    .size,
            )
        };
        contents_size.SetToMax(&visible_size);
        gfx::Vector2d::new(
            contents_size.width() - visible_size.width(),
            contents_size.height() - visible_size.height(),
        )
    }

    pub fn HasHorizontalOverflow(&self) -> bool {
        let client_width =
            self.LayoutContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
                .Width()
                - LayoutUnit::from_signed(self.VerticalScrollbarWidth(
                    OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
                ));
        self.ScrollWidth().Round() > client_width.Round()
    }

    pub fn HasVerticalOverflow(&self) -> bool {
        let client_height = self
            .LayoutContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
            .Height()
            - LayoutUnit::from_signed(self.HorizontalScrollbarHeight(
                OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            ));
        self.ScrollHeight().Round() > client_height.Round()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1653-1696.
    pub fn NeedsScrollbarReconstruction(&self) -> bool {
        if !self.HasScrollbar() {
            return false;
        }

        let box_ = unsafe { &*self.GetLayoutBox() };
        let style_source = ScrollbarStyleSource(box_);
        let element = ElementForScrollStart(box_);
        let needs_custom = unsafe { &*style_source }.IsBox()
            && unsafe { &*style_source }
                .StyleRef()
                .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>());

        for scrollbar in [self.HorizontalScrollbar(), self.VerticalScrollbar()] {
            if scrollbar.is_null() {
                continue;
            }
            let scrollbar = unsafe { &*scrollbar };
            if scrollbar.IsCustomScrollbar() != needs_custom {
                return true;
            }
            if scrollbar.StyleSource() != style_source {
                return true;
            }
            if needs_custom {
                continue;
            }
            if scrollbar.ThemeMetrics() != box_.ScrollbarThemeForLayout() {
                return true;
            }
            if scrollbar.CSSScrollbarWidth()
                != unsafe { &*style_source }.StyleRef().UsedScrollbarWidth()
            {
                return true;
            }
        }
        false
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1698-1738.
    pub fn ComputeScrollbarWidthsForViewportUnits(
        &self,
        scrollbar_properties: StyleBasedScrollbarData,
    ) -> gfx::Size {
        let box_ = unsafe { &*self.GetLayoutBox() };
        debug_assert!(!box_.IsFieldset());
        debug_assert!(!box_.IsFrameSet());
        let viewport = box_.ViewportGeometryForLayout();
        ComputeScrollbarWidthsForViewportUnitsFromInputs(
            scrollbar_properties,
            viewport.visual_viewport_supplies_scrollbars,
            viewport.hide_scrollbars,
            box_.ScrollbarThemeForLayout(),
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1604-1618.
    pub fn HypotheticalScrollbarThickness(
        &self,
        orientation: ScrollbarOrientation,
        should_include_overlay_thickness: bool,
    ) -> i32 {
        debug_assert!(self.NeedsHypotheticalScrollbarThickness(orientation));
        // The retained values are refreshed at the end of every represented
        // layout pass. A caller asking to include overlay thickness is the
        // LayoutBox stable-gutter path and consumes that cache, as Chromium does
        // after DocumentLifecycle::kLayoutClean.
        if should_include_overlay_thickness
            && self.hypothetical_scrollbar_thicknesses_are_valid_
            && !super::super::layout_pass_scope::LayoutPassScope::IsActive()
        {
            return match orientation {
                ScrollbarOrientation::kHorizontalScrollbar => {
                    self.hypothetical_horizontal_scrollbar_thickness_
                }
                ScrollbarOrientation::kVerticalScrollbar => {
                    self.hypothetical_vertical_scrollbar_thickness_
                }
            };
        }
        self.ComputeHypotheticalScrollbarThickness(orientation, should_include_overlay_thickness)
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1620-1628.
    fn NeedsHypotheticalScrollbarThickness(&self, orientation: ScrollbarOrientation) -> bool {
        let box_ = unsafe { &*self.GetLayoutBox() };
        box_.IsLayoutView() || box_.HasScrollbarGutters(orientation)
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1630-1651.
    fn ComputeHypotheticalScrollbarThickness(
        &self,
        orientation: ScrollbarOrientation,
        should_include_overlay_thickness: bool,
    ) -> i32 {
        let scrollbar = match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => self.HorizontalScrollbar(),
            ScrollbarOrientation::kVerticalScrollbar => self.VerticalScrollbar(),
        };
        if !scrollbar.is_null() {
            return unsafe { &*scrollbar }.ScrollbarThickness();
        }

        let box_ = unsafe { &*self.GetLayoutBox() };
        let style_source = ScrollbarStyleSource(box_);
        let element = ElementForScrollStart(box_);
        if unsafe { &*style_source }
            .StyleRef()
            .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>())
        {
            let owner = if orientation == ScrollbarOrientation::kHorizontalScrollbar {
                &self.hypothetical_horizontal_style_owner_
            } else {
                &self.hypothetical_vertical_style_owner_
            };
            if owner.get().Get().is_null() {
                owner.set(Member::from_ptr(self.CreateScrollbar(orientation)));
            }
            // A fresh source owner requests the uncached background style.
            // Page resolves its typed pending publication before exposing layout.
            return unsafe { &*owner.get().Get() }.ScrollbarThickness();
        }

        let theme = box_.ScrollbarThemeForLayout();
        if theme.uses_overlay_scrollbars && !should_include_overlay_thickness {
            return 0;
        }
        ScrollbarThemeThickness(
            theme,
            unsafe { &*style_source }.StyleRef().UsedScrollbarWidth(),
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1421-1467. Scrollbar
    // existence, reconstruction, corner ownership, style notification and
    // invalidation run at the same post-style-change lifecycle point.
    pub fn UpdateAfterStyleChange(&mut self, old_style: *const ComputedStyle) {
        if unsafe { &*self.GetLayoutBox() }
            .InputOwnerForLayout()
            .InputViewport()
            .is_none()
        {
            if !self.pending_after_style_change_ {
                self.pending_after_style_change_old_style_ = Member::from_ptr(old_style.cast_mut());
            }
            self.pending_after_style_change_ = true;
            return;
        }
        self.pending_after_style_change_ = false;
        self.pending_after_style_change_old_style_ = Member::default();
        self.hypothetical_scrollbar_thicknesses_are_valid_ = false;
        if !self.overflow_rect_.size.IsZero() {
            self.UpdateScrollableAreaSet();
        }

        if self.NeedsScrollbarReconstruction() {
            self.RemoveScrollbarsForReconstruction();
            return;
        }

        let mut needs_horizontal_scrollbar = false;
        let mut needs_vertical_scrollbar = false;
        self.ComputeScrollbarExistence(
            &mut needs_horizontal_scrollbar,
            &mut needs_vertical_scrollbar,
            ComputeScrollbarExistenceOption::kOverflowIndependent,
        );

        if !self.HasScrollbar() && !needs_horizontal_scrollbar && !needs_vertical_scrollbar {
            return;
        }

        self.SetHasHorizontalScrollbar(needs_horizontal_scrollbar);
        self.SetHasVerticalScrollbar(needs_vertical_scrollbar);

        for scrollbar in [self.HorizontalScrollbar(), self.VerticalScrollbar()] {
            if !scrollbar.is_null() {
                unsafe { &mut *scrollbar }.StyleChanged();
            }
        }

        self.UpdateScrollCornerStyle();
        let style = unsafe { &*self.GetLayoutBox() }.StyleRef();
        if old_style.is_null()
            || self.UsedColorSchemeScrollbarsChanged(old_style)
            || unsafe { &*old_style }.ScrollbarThumbColorResolved()
                != style.ScrollbarThumbColorResolved()
            || unsafe { &*old_style }.ScrollbarTrackColorResolved()
                != style.ScrollbarTrackColorResolved()
            || unsafe { &*old_style }.UsedPointerEvents() != style.UsedPointerEvents()
        {
            self.SetScrollControlsNeedFullPaintInvalidation();
        }
    }

    pub fn RunPendingUpdateAfterStyleChange(&mut self) {
        if !self.pending_after_style_change_ {
            return;
        }
        let old_style = self.pending_after_style_change_old_style_.Get();
        self.UpdateAfterStyleChange(old_style);
    }

    pub fn ComputeScrollbarExistence(
        &self,
        needs_horizontal_scrollbar: &mut bool,
        needs_vertical_scrollbar: &mut bool,
        option: ComputeScrollbarExistenceOption,
    ) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let viewport = box_.ViewportGeometryForLayout();
        if viewport.visual_viewport_supplies_scrollbars
            || !CanHaveOverflowScrollbars(box_)
            || viewport.hide_scrollbars
            || box_.IsFieldset()
            || box_.IsFrameSet()
            || box_.StyleRef().UsedScrollbarWidth() == EScrollbarWidth::kNone
        {
            *needs_horizontal_scrollbar = false;
            *needs_vertical_scrollbar = false;
            return;
        }

        let (mut horizontal_mode, mut vertical_mode) = ResolveScrollbarModes(box_);
        let style_source = ScrollbarStyleSource(box_);
        let element = ElementForScrollStart(box_);
        let has_custom_scrollbar_style = unsafe { &*style_source }
            .StyleRef()
            .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>());
        let will_be_overlay =
            box_.ScrollbarThemeForLayout().uses_overlay_scrollbars && !has_custom_scrollbar_style;
        if will_be_overlay {
            if self
                .ShouldSupplyScrollbarsForVisualViewport(ScrollbarOrientation::kHorizontalScrollbar)
            {
                if horizontal_mode != ScrollbarMode::kAlwaysOff {
                    horizontal_mode = ScrollbarMode::kAlwaysOn;
                }
            } else if horizontal_mode == ScrollbarMode::kAlwaysOn {
                horizontal_mode = ScrollbarMode::kAuto;
            }
            if self
                .ShouldSupplyScrollbarsForVisualViewport(ScrollbarOrientation::kVerticalScrollbar)
            {
                if vertical_mode != ScrollbarMode::kAlwaysOff {
                    vertical_mode = ScrollbarMode::kAlwaysOn;
                }
            } else if vertical_mode == ScrollbarMode::kAlwaysOn {
                vertical_mode = ScrollbarMode::kAuto;
            }
        }

        *needs_horizontal_scrollbar = self.HasHorizontalScrollbar();
        *needs_vertical_scrollbar = self.HasVerticalScrollbar();
        if horizontal_mode == ScrollbarMode::kAlwaysOn {
            *needs_horizontal_scrollbar = true;
        } else if horizontal_mode == ScrollbarMode::kAlwaysOff {
            *needs_horizontal_scrollbar = false;
        }
        if vertical_mode == ScrollbarMode::kAlwaysOn {
            *needs_vertical_scrollbar = true;
        } else if vertical_mode == ScrollbarMode::kAlwaysOff {
            *needs_vertical_scrollbar = false;
        }
        if option == ComputeScrollbarExistenceOption::kOverflowIndependent {
            return;
        }

        if horizontal_mode == ScrollbarMode::kAuto {
            *needs_horizontal_scrollbar = box_.IsRooted()
                && self.HasHorizontalOverflow()
                && self
                    .VisibleContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
                    .height()
                    != 0;
        }
        if vertical_mode == ScrollbarMode::kAuto {
            *needs_vertical_scrollbar = box_.IsRooted()
                && self.HasVerticalOverflow()
                && self
                    .VisibleContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
                    .width()
                    != 0;
        }
    }

    pub fn ComputeScrollbarExistenceDefault(
        &self,
        needs_horizontal_scrollbar: &mut bool,
        needs_vertical_scrollbar: &mut bool,
    ) {
        self.ComputeScrollbarExistence(
            needs_horizontal_scrollbar,
            needs_vertical_scrollbar,
            ComputeScrollbarExistenceOption::kDependsOnOverflow,
        )
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2839-2872. Frame,
    // Settings, root-scroller and VisualViewport ownership are resolved by the
    // host into the two immutable per-axis Layout input facts.
    pub fn ShouldAvoidHidingOverlayScrollbars(&self) -> bool {
        self.ShouldSupplyScrollbarsForVisualViewport(ScrollbarOrientation::kHorizontalScrollbar)
            || self
                .ShouldSupplyScrollbarsForVisualViewport(ScrollbarOrientation::kVerticalScrollbar)
    }

    pub fn ShouldSupplyScrollbarsForVisualViewport(
        &self,
        orientation: ScrollbarOrientation,
    ) -> bool {
        if let Some((active, horizontal, vertical)) = self.visual_viewport_scrollability_ {
            let box_ = unsafe { &*self.GetLayoutBox() };
            if !box_.IsGlobalRootScroller()
                || box_
                    .ViewportGeometryForLayout()
                    .visual_viewport_supplies_scrollbars
                || !box_.ScrollbarThemeForLayout().uses_overlay_scrollbars
            {
                return false;
            }
            let style_source = ScrollbarStyleSource(box_);
            let element = ElementForScrollStart(box_);
            if unsafe { &*style_source }
                .StyleRef()
                .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>())
            {
                return false;
            }
            return active
                && match orientation {
                    ScrollbarOrientation::kHorizontalScrollbar => horizontal,
                    ScrollbarOrientation::kVerticalScrollbar => vertical,
                };
        }
        let viewport = unsafe { &*self.GetLayoutBox() }.ViewportGeometryForLayout();
        match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => {
                viewport.should_supply_horizontal_scrollbar_for_visual_viewport
            }
            ScrollbarOrientation::kVerticalScrollbar => {
                viewport.should_supply_vertical_scrollbar_for_visual_viewport
            }
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2874-2895.
    pub fn SetVisualViewportScrollability(
        &mut self,
        active: bool,
        horizontal: bool,
        vertical: bool,
    ) {
        self.visual_viewport_scrollability_ = Some((active, horizontal, vertical));
    }
    pub fn DidUpdateVisualViewport(&mut self) {
        self.scroll_anchor_.Clear();
        let mut needs_horizontal_scrollbar = false;
        let mut needs_vertical_scrollbar = false;
        self.ComputeScrollbarExistenceDefault(
            &mut needs_horizontal_scrollbar,
            &mut needs_vertical_scrollbar,
        );
        let scrollbar_existence_changed = needs_horizontal_scrollbar
            != self.HasHorizontalScrollbar()
            || needs_vertical_scrollbar != self.HasVerticalScrollbar();
        self.SetHasHorizontalScrollbar(needs_horizontal_scrollbar);
        self.SetHasVerticalScrollbar(needs_vertical_scrollbar);

        self.UpdateScrollbarProportions();
        if scrollbar_existence_changed {
            self.UpdateScrollbarEnabledState(false, false);
            self.UpdateScrollableAreaSet();
            self.PositionOverflowControls();
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1859-1898.
    pub fn TryRemovingAutoScrollbars(
        &self,
        needs_horizontal_scrollbar: bool,
        needs_vertical_scrollbar: bool,
    ) -> bool {
        if !needs_horizontal_scrollbar && !needs_vertical_scrollbar {
            return false;
        }
        let box_ = unsafe { &*self.GetLayoutBox() };
        let viewport = box_.ViewportGeometryForLayout();
        if viewport.should_supply_horizontal_scrollbar_for_visual_viewport
            || viewport.should_supply_vertical_scrollbar_for_visual_viewport
        {
            return false;
        }
        if box_.IsLayoutView() {
            let (horizontal_mode, vertical_mode) = ResolveScrollbarModes(box_);
            if horizontal_mode != ScrollbarMode::kAuto || vertical_mode != ScrollbarMode::kAuto {
                return false;
            }
            let visible_size = self
                .VisibleContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
                .size();
            if self.ScrollWidth() <= LayoutUnit::from_signed(visible_size.width())
                && self.ScrollHeight() <= LayoutUnit::from_signed(visible_size.height())
            {
                return true;
            }
        } else {
            if !box_.HasAutoVerticalScrollbar() || !box_.HasAutoHorizontalScrollbar() {
                return false;
            }
            let client_size = self
                .LayoutContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
                .size;
            if self.ScrollWidth() <= client_size.width && self.ScrollHeight() <= client_size.height
            {
                return true;
            }
        }
        false
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1029,1033-1043.
    // Native snap queue retains the FrameView update order; sticky updates
    // remain a separate translated lifecycle stage.
    pub fn UpdateAfterLayout(&mut self) {
        self.EnqueueForSnapUpdateIfNeeded();
        let freeze_state = unsafe { &*self.GetLayoutBox() }.GetScrollbarFreezeState();
        let horizontal_frozen =
            !freeze_state.is_null() && unsafe { &*freeze_state }.IsHorizontalScrollbarFrozen();
        let vertical_frozen =
            !freeze_state.is_null() && unsafe { &*freeze_state }.IsVerticalScrollbarFrozen();

        if self.NeedsScrollbarReconstruction() {
            self.RemoveScrollbarsForReconstruction();
            self.DestroyDetachedScrollbars();
        }

        self.UpdateScrollDimensions();

        let had_horizontal_scrollbar = self.HasHorizontalScrollbar();
        let had_vertical_scrollbar = self.HasVerticalScrollbar();
        let mut needs_horizontal_scrollbar = false;
        let mut needs_vertical_scrollbar = false;
        self.ComputeScrollbarExistence(
            &mut needs_horizontal_scrollbar,
            &mut needs_vertical_scrollbar,
            ComputeScrollbarExistenceOption::kDependsOnOverflow,
        );
        if !horizontal_frozen
            && !vertical_frozen
            && self.TryRemovingAutoScrollbars(needs_horizontal_scrollbar, needs_vertical_scrollbar)
        {
            needs_horizontal_scrollbar = false;
            needs_vertical_scrollbar = false;
        }

        let horizontal_changes = needs_horizontal_scrollbar != had_horizontal_scrollbar;
        let vertical_changes = needs_vertical_scrollbar != had_vertical_scrollbar;
        if (horizontal_changes && !horizontal_frozen) || (vertical_changes && !vertical_frozen) {
            self.SetHasHorizontalScrollbar(needs_horizontal_scrollbar);
            self.SetHasVerticalScrollbar(needs_vertical_scrollbar);
            self.UpdateScrollOrigin();
            self.UpdateScrollCornerStyle();
            // Chromium's proprietary overflow:overlay controls never consume a
            // layout gutter. Only a non-overlay scrollbar change invalidates
            // intrinsic inline contributions and refreshes layout dimensions.
            // cpp: paint_layer_scrollable_area.cc:1097-1112
            let box_ = unsafe { &mut *self.GetLayoutBox() };
            let affects_layout = (horizontal_changes
                && box_.StyleRef().OverflowX() != EOverflow::kOverlay)
                || (vertical_changes && box_.StyleRef().OverflowY() != EOverflow::kOverlay);
            if affects_layout {
                if (vertical_changes && box_.IsHorizontalWritingMode())
                    || (horizontal_changes && !box_.IsHorizontalWritingMode())
                {
                    box_.SetIntrinsicLogicalWidthsDirty(
                        super::super::layout_object::MarkingBehavior::kMarkContainerChain,
                    );
                }
                self.UpdateScrollDimensions();
            }
        }
        self.UpdateScrollbarEnabledState(horizontal_frozen, vertical_frozen);
        self.UpdateScrollbarProportions();
        // Source: core/paint/paint_layer_scrollable_area.cc:1124-1132.
        self.hypothetical_horizontal_scrollbar_thickness_ = 0;
        if self.NeedsHypotheticalScrollbarThickness(ScrollbarOrientation::kHorizontalScrollbar) {
            self.hypothetical_horizontal_scrollbar_thickness_ = self
                .ComputeHypotheticalScrollbarThickness(
                    ScrollbarOrientation::kHorizontalScrollbar,
                    true,
                );
        }
        self.hypothetical_vertical_scrollbar_thickness_ = 0;
        if self.NeedsHypotheticalScrollbarThickness(ScrollbarOrientation::kVerticalScrollbar) {
            self.hypothetical_vertical_scrollbar_thickness_ = self
                .ComputeHypotheticalScrollbarThickness(
                    ScrollbarOrientation::kVerticalScrollbar,
                    true,
                );
        }
        self.hypothetical_scrollbar_thicknesses_are_valid_ = true;
        self.DelayableClampScrollOffsetAfterOverflowChange();
        if !horizontal_frozen || !vertical_frozen {
            self.UpdateScrollableAreaSet();
        }
        // Source: paint_layer_scrollable_area.cc:1140.
        self.PositionOverflowControls();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2633-2638,
    // 2651-2669,2686-2695,2718-2725. FrameView registration, counters and
    // observer notifications stay outside the Layout/Paint package contract.
    fn UpdateScrollableAreaSet(&mut self) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let has_horizontal_overflow = self.HasHorizontalOverflow();
        let has_vertical_overflow = self.HasVerticalOverflow();
        let mut has_overflow = !box_.StitchedSize().IsZero()
            && ((has_horizontal_overflow && box_.ScrollsOverflowX())
                || (has_vertical_overflow && box_.ScrollsOverflowY()));

        let was_scrolls_overflow = self.scrolls_overflow_;
        if box_.IsLayoutView() {
            let (horizontal, vertical) = ResolveScrollbarModes(box_);
            if horizontal == ScrollbarMode::kAlwaysOff && vertical == ScrollbarMode::kAlwaysOff {
                has_overflow = false;
            }
        }
        let viewport = box_.ViewportGeometryForLayout();
        self.scrolls_overflow_ = (has_overflow
            || viewport.should_supply_horizontal_scrollbar_for_visual_viewport
            || viewport.should_supply_vertical_scrollbar_for_visual_viewport)
            && box_.StyleRef().Visibility() == EVisibility::kVisible;
        if was_scrolls_overflow == self.scrolls_overflow_ {
            return;
        }

        for scrollbar in [self.HorizontalScrollbar(), self.VerticalScrollbar()] {
            if !scrollbar.is_null() {
                unsafe { &*scrollbar }
                    .DisplayItemClient()
                    .Invalidate(PaintInvalidationReason::kScrollControl);
            }
        }
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        box_.SetNeedsPaintPropertyUpdate();
        box_.SetBackgroundNeedsFullPaintInvalidation();
    }

    // Source: core/paint/paint_layer_scrollable_area.h:471.
    pub fn ScrollsOverflow(&self) -> bool {
        self.scrolls_overflow_
    }

    // Source: core/paint/paint_layer_scrollable_area.h:494.
    pub fn IsPaintLayerScrollableArea(&self) -> bool {
        true
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1469-1494.
    pub fn UpdateAfterOverflowRecalc(&mut self) {
        self.UpdateScrollDimensions();
        self.UpdateScrollbarProportions();
        self.UpdateScrollbarEnabledState(false, false);

        let mut needs_horizontal_scrollbar = false;
        let mut needs_vertical_scrollbar = false;
        self.ComputeScrollbarExistenceDefault(
            &mut needs_horizontal_scrollbar,
            &mut needs_vertical_scrollbar,
        );

        let horizontal_scrollbar_should_change =
            needs_horizontal_scrollbar != self.HasHorizontalScrollbar();
        let vertical_scrollbar_should_change =
            needs_vertical_scrollbar != self.HasVerticalScrollbar();
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        if (box_.HasAutoHorizontalScrollbar() && horizontal_scrollbar_should_change)
            || (box_.HasAutoVerticalScrollbar() && vertical_scrollbar_should_change)
        {
            let reason = super::super::layout_invalidation_reason::UnknownReason();
            box_.SetNeedsLayoutAndFullPaintInvalidation(reason);
        }

        self.ClampScrollOffsetAfterOverflowChange();
        self.UpdateScrollableAreaSet();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:964-991. Page theme
    // and VisualViewport ownership are projected through immutable Layout
    // input facts; Scrollbar retains its own enabled state.
    fn UpdateScrollbarEnabledState(
        &mut self,
        is_horizontal_scrollbar_frozen: bool,
        is_vertical_scrollbar_frozen: bool,
    ) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let force_disable = box_
            .ScrollbarThemeForLayout()
            .should_disable_invisible_scrollbars
            && self.ScrollbarsHiddenIfOverlay();
        let viewport = box_.ViewportGeometryForLayout();

        let horizontal = self.HorizontalScrollbar();
        if !horizontal.is_null()
            && (!unsafe { &*horizontal }.IsCustomScrollbar() || !is_horizontal_scrollbar_frozen)
        {
            let enabled = (self.HasHorizontalOverflow() && !force_disable)
                || viewport.should_supply_horizontal_scrollbar_for_visual_viewport;
            unsafe { &mut *horizontal }.SetEnabled(enabled);
        }

        let vertical = self.VerticalScrollbar();
        if !vertical.is_null()
            && (!unsafe { &*vertical }.IsCustomScrollbar() || !is_vertical_scrollbar_frozen)
        {
            let enabled = (self.HasVerticalOverflow() && !force_disable)
                || viewport.should_supply_vertical_scrollbar_for_visual_viewport;
            unsafe { &mut *vertical }.SetEnabled(enabled);
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:994-999.
    fn UpdateScrollbarProportions(&mut self) {
        let visible = self.VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars);
        let contents = self.ContentsSize();
        let horizontal = self.HorizontalScrollbar();
        if !horizontal.is_null() {
            unsafe { &mut *horizontal }.SetProportion(visible.width(), contents.width());
        }
        let vertical = self.VerticalScrollbar();
        if !vertical.is_null() {
            unsafe { &mut *vertical }.SetProportion(visible.height(), contents.height());
        }
    }

    // Chromium paint_layer_scrollable_area.cc:1202-1254, pinned
    // 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. Scrollbar relayouts
    // share the existing DelayScrollOffsetClampScope; only final geometry clamps.
    pub fn NeedsScrollOffsetClamp(&self) -> bool {
        self.needs_scroll_offset_clamp_
    }
    pub fn SetNeedsScrollOffsetClamp(&mut self, value: bool) {
        self.needs_scroll_offset_clamp_ = value;
    }
    fn DelayableClampScrollOffsetAfterOverflowChange(&mut self) {
        if self.HasBeenDisposed() {
            return;
        }
        use super::super::scroll_layout_scope::DelayScrollOffsetClampScope;
        if DelayScrollOffsetClampScope::ClampingIsDelayed() {
            DelayScrollOffsetClampScope::SetNeedsClamp(self);
        } else {
            self.ClampScrollOffsetAfterOverflowChange();
        }
    }
    pub fn ClampScrollOffsetAfterOverflowChange(&mut self) {
        if self.HasBeenDisposed() {
            return;
        }
        self.UpdateScrollDimensions();
        let before = self.GetScrollOffset();
        let offset = self.ClampScrollOffset(&before);
        let geometry_changed = offset != before || self.ScrollOriginChanged();
        // Chromium always enters one of its SetScrollOffset paths here, even
        // when clamping leaves the numeric offset unchanged. Keep the source
        // owner synchronized on every clamp; only the immutable Paint input
        // and property invalidation need the stronger geometry-change gate.
        // cpp: paint_layer_scrollable_area.cc:1238-1250
        self.UpdateScrollOffset(
            offset,
            layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kClamping,
        );
        if geometry_changed {
            let box_ = unsafe { &mut *self.GetLayoutBox() };
            box_.SetNeedsPaintPropertyUpdate();
            let node = box_.GetNode();
            if !node.is_null() {
                // Keep the immutable export coherent without re-entering this
                // mutable scrollable-area owner through UpdateInputScrollOffset.
                unsafe { &mut *node }.ReflectInputScrollOffset(
                    super::super::layout_input::Offset {
                        x: f64::from(offset.x()),
                        y: f64::from(offset.y()),
                    },
                );
            }
        }
        self.SetNeedsScrollOffsetClamp(false);
        self.ResetScrollOriginChanged();
        // Source: core/paint/paint_layer_scrollable_area.cc:1254. A delayed
        // clamp is also the lifetime boundary for temporarily detached bars.
        self.DestroyDetachedScrollbars();
    }

    pub fn UpdateScrollOrigin(&mut self) {
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        let mut scrollable_overflow = self.overflow_rect_;
        scrollable_overflow.Move(&-box_.BorderOutsets().Offset());
        let new_origin =
            ToFlooredPoint(-scrollable_overflow.offset) + box_.OriginAdjustmentForScrollbars();
        if new_origin != self.scroll_origin_ {
            self.scroll_origin_changed_ = true;
            box_.SetSubtreeShouldCheckForPaintInvalidation();
        }
        self.scroll_origin_ = new_origin;
    }

    pub fn UpdateScrollDimensions(&mut self) {
        let mut new_overflow_rect = unsafe { &*self.GetLayoutBox() }.ScrollableOverflowRect();
        new_overflow_rect.Unite(&PhysicalRect::new(
            new_overflow_rect.offset,
            self.LayoutContentRect(IncludeScrollbarsInRect::kExcludeScrollbars)
                .size,
        ));
        let resized = self.overflow_rect_.size != new_overflow_rect.size;
        self.overflow_rect_ = new_overflow_rect;
        if resized {
            self.ContentsResized();
        }
        self.UpdateScrollOrigin();
    }

    pub fn ScrollableAxes(&self) -> PhysicalAxes {
        let box_ = self.GetLayoutBox();
        if box_.is_null() || !unsafe { &*box_ }.IsScrollContainer() {
            return kPhysicalAxesNone;
        }
        if unsafe { &*box_ }.IsLayoutView() {
            return kPhysicalAxesBoth;
        }
        let mut axes = kPhysicalAxesNone;
        let style = unsafe { &*box_ }.StyleRef();
        if style.IsOverflowValueScrollableX() {
            axes |= kPhysicalAxesHorizontal;
        }
        if style.IsOverflowValueScrollableY() {
            axes |= kPhysicalAxesVertical;
        }
        axes
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:3785-3796.
    pub fn ClampNonScrollableAxesOffsets(&self, mut offset: gfx::Vector2d) -> gfx::Vector2d {
        let axes = self.ScrollableAxes();
        if !(axes & kPhysicalAxesHorizontal).is_nonzero() {
            offset.set_x(0);
        }
        if !(axes & kPhysicalAxesVertical).is_nonzero() {
            offset.set_y(0);
        }
        offset
    }

    pub fn RootScrollerVisibleSizeForLayout(&self) -> Option<gfx::Size> {
        let box_ = self.GetLayoutBox();
        assert!(!box_.is_null());
        let box_ = unsafe { &*box_ };
        let size = if box_.IsLayoutView() {
            box_.ViewportGeometryForLayout().root_scroller_visible_size
        } else {
            let element = DynamicTo::<Element>(box_.GetNode());
            if element.is_null() {
                None
            } else {
                unsafe { &*element }.InputRootScrollerVisibleSize()
            }
        };
        size.map(|size| gfx::Size::new(size.width, size.height))
    }

    pub fn HasHorizontalScrollbar(&self) -> bool {
        self.has_horizontal_scrollbar_
    }

    pub fn HasVerticalScrollbar(&self) -> bool {
        self.has_vertical_scrollbar_
    }

    // Source: core/paint/paint_layer_scrollable_area.h:401-403.
    pub fn HasScrollbar(&self) -> bool {
        self.HasHorizontalScrollbar() || self.HasVerticalScrollbar()
    }

    // Source: core/paint/paint_layer_scrollable_area.h:415-417.
    pub fn HasOverflow(&self) -> bool {
        self.HasHorizontalOverflow() || self.HasVerticalOverflow()
    }

    // ScrollableArea::HasOverlayScrollbars observes the actual scrollbar.
    // Rechrom's source scrollbar owner selects its concrete kind from these
    // same theme and custom-style inputs when it is constructed.
    pub fn HasOverlayScrollbars(&self) -> bool {
        let vertical = self.VerticalScrollbar();
        if !vertical.is_null() && unsafe { &*vertical }.IsOverlayScrollbar() {
            return true;
        }
        let horizontal = self.HorizontalScrollbar();
        !horizontal.is_null() && unsafe { &*horizontal }.IsOverlayScrollbar()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2141-2146.
    pub fn HasOverflowControls(&self) -> bool {
        self.HasScrollbar() || unsafe { &*self.GetLayoutBox() }.CanResize()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2148-2157.
    pub fn HasOverlayOverflowControls(&self) -> bool {
        if self.HasOverlayScrollbars() {
            return true;
        }
        let box_ = unsafe { &*self.GetLayoutBox() };
        if !self.HasScrollbar() && box_.CanResize() {
            return true;
        }
        box_.StyleRef().OverflowX() == EOverflow::kOverlay
            || box_.StyleRef().OverflowY() == EOverflow::kOverlay
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2159-2164.
    pub fn NeedsScrollCorner(&self) -> bool {
        self.HasScrollbar() && !self.HasOverlayScrollbars()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2166-2178.
    pub fn ShouldOverflowControlsPaintAsOverlay(&self) -> bool {
        if self.HasOverlayOverflowControls() {
            return true;
        }
        let box_ = unsafe { &*self.GetLayoutBox() };
        box_.IsLayoutView() || box_.IsGlobalRootScroller()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1949-1968. The
    // document-owned draggable-region dirty bit at 1970-1972 is not inferred
    // inside this Layout/Paint package.
    pub fn SetHasHorizontalScrollbar(&mut self, has_scrollbar: bool) {
        let state = unsafe { &*self.GetLayoutBox() }.GetScrollbarFreezeState();
        if !state.is_null() && unsafe { &*state }.IsHorizontalScrollbarFrozen() {
            return;
        }
        if has_scrollbar == self.has_horizontal_scrollbar_ {
            return;
        }
        self.SetScrollbarNeedsPaintInvalidation(ScrollbarOrientation::kHorizontalScrollbar);
        self.SetHasHorizontalScrollbarInManager(has_scrollbar);
        self.NotifyScrollbarStylesChanged();
        self.SetScrollCornerNeedsPaintInvalidation();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1975-2001. The
    // document-owned draggable-region dirty bit at 2003-2005 is not inferred
    // inside this Layout/Paint package.
    pub fn SetHasVerticalScrollbar(&mut self, has_scrollbar: bool) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let state = box_.GetScrollbarFreezeState();
        if !state.is_null() && unsafe { &*state }.IsVerticalScrollbarFrozen() {
            return;
        }
        if box_.InputOwnerForLayout().InputVerticalScrollEnforced() {
            return;
        }
        if has_scrollbar == self.has_vertical_scrollbar_ {
            return;
        }
        self.SetScrollbarNeedsPaintInvalidation(ScrollbarOrientation::kVerticalScrollbar);
        self.SetHasVerticalScrollbarInManager(has_scrollbar);
        self.NotifyScrollbarStylesChanged();
        self.SetScrollCornerNeedsPaintInvalidation();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2932-2948.
    fn SetHasHorizontalScrollbarInManager(&mut self, has_scrollbar: bool) {
        if has_scrollbar {
            if self.horizontal_scrollbar_.Get().is_null() {
                self.mac_horizontal_thumb_alpha_ = 0.0;
                self.horizontal_scrollbar_ = Member::from_ptr(
                    self.CreateScrollbar(ScrollbarOrientation::kHorizontalScrollbar),
                );
            }
            self.EmitScrollbarActivity(
                self.horizontal_scrollbar_.Get(),
                NativeScrollbarActivityKind::Added,
            );
            self.has_horizontal_scrollbar_ = true;
        } else {
            self.has_horizontal_scrollbar_ = false;
            if !super::super::scroll_layout_scope::DelayScrollOffsetClampScope::ClampingIsDelayed()
            {
                self.DestroyScrollbar(ScrollbarOrientation::kHorizontalScrollbar);
            }
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2950-2966.
    fn SetHasVerticalScrollbarInManager(&mut self, has_scrollbar: bool) {
        if has_scrollbar {
            if self.vertical_scrollbar_.Get().is_null() {
                self.mac_vertical_thumb_alpha_ = 0.0;
                self.vertical_scrollbar_ = Member::from_ptr(
                    self.CreateScrollbar(ScrollbarOrientation::kVerticalScrollbar),
                );
            }
            self.EmitScrollbarActivity(
                self.vertical_scrollbar_.Get(),
                NativeScrollbarActivityKind::Added,
            );
            self.has_vertical_scrollbar_ = true;
        } else {
            self.has_vertical_scrollbar_ = false;
            if !super::super::scroll_layout_scope::DelayScrollOffsetClampScope::ClampingIsDelayed()
            {
                self.DestroyScrollbar(ScrollbarOrientation::kVerticalScrollbar);
            }
        }
    }

    fn NotifyScrollbarStylesChanged(&mut self) {
        for scrollbar in [self.HorizontalScrollbar(), self.VerticalScrollbar()] {
            if !scrollbar.is_null() {
                unsafe { &mut *scrollbar }.StyleChanged();
            }
        }
    }

    // Source: core/scroll/scrollable_area.cc:786-826 and
    // core/paint/paint_layer_scrollable_area.cc:3328-3330.
    pub(crate) fn SetScrollbarNeedsPaintInvalidation(&mut self, orientation: ScrollbarOrientation) {
        let scrollbar = match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => self.horizontal_scrollbar_.Get(),
            ScrollbarOrientation::kVerticalScrollbar => self.vertical_scrollbar_.Get(),
        };
        if !scrollbar.is_null() {
            unsafe { &*scrollbar }
                .DisplayItemClient()
                .Invalidate(PaintInvalidationReason::kScrollControl);
        }
        self.SetShouldCheckForPaintInvalidation();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1171-1182. During the
    // represented PrePaint positioning pass, ancestor invalidation traversal
    // may already be complete, so mutate only this LayoutObject's paint flag.
    fn SetShouldCheckForPaintInvalidation(&mut self) {
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        if self.positioning_overflow_controls_in_prepaint_ {
            box_.GetMutableForPainting()
                .SetShouldCheckForPaintInvalidation();
        } else {
            box_.SetShouldCheckForPaintInvalidation();
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:383-385 delegates to
    // ScrollableArea; the base implementation is translated inline here.
    fn SetScrollCornerNeedsPaintInvalidation(&mut self) {
        let client = self.scroll_corner_client_.Get();
        if !client.is_null() {
            unsafe { &*client }
                .DisplayItemClient()
                .Invalidate(PaintInvalidationReason::kScrollControl);
        }
        self.SetShouldCheckForPaintInvalidation();
    }

    // Source: core/scroll/scrollable_area.cc:829-835.
    fn SetScrollControlsNeedFullPaintInvalidation(&mut self) {
        if self.HasHorizontalScrollbar() {
            self.SetScrollbarNeedsPaintInvalidation(ScrollbarOrientation::kHorizontalScrollbar);
        }
        if self.HasVerticalScrollbar() {
            self.SetScrollbarNeedsPaintInvalidation(ScrollbarOrientation::kVerticalScrollbar);
        }
        self.SetScrollCornerNeedsPaintInvalidation();
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2992-3030. FrameView,
    // compositor and EventHandler notifications are outside this package;
    // ownership, invalidation, delayed detachment and disconnection stay here.
    fn DestroyScrollbar(&mut self, orientation: ScrollbarOrientation) {
        debug_assert!(match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => !self.has_horizontal_scrollbar_,
            ScrollbarOrientation::kVerticalScrollbar => !self.has_vertical_scrollbar_,
        });
        let scrollbar = match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => self.horizontal_scrollbar_.Get(),
            ScrollbarOrientation::kVerticalScrollbar => self.vertical_scrollbar_.Get(),
        };
        if scrollbar.is_null() {
            return;
        }
        self.SetScrollbarNeedsPaintInvalidation(orientation);
        self.EmitScrollbarActivity(scrollbar, NativeScrollbarActivityKind::Removed);
        unsafe { &mut *scrollbar }.DisconnectFromScrollableArea();
        match orientation {
            ScrollbarOrientation::kHorizontalScrollbar => {
                self.mac_horizontal_thumb_alpha_ = 0.0;
                self.horizontal_scrollbar_ = Member::default()
            }
            ScrollbarOrientation::kVerticalScrollbar => {
                self.mac_vertical_thumb_alpha_ = 0.0;
                self.vertical_scrollbar_ = Member::default()
            }
        }
    }

    fn DestroyDetachedScrollbars(&mut self) {
        debug_assert!(
            !self.has_horizontal_scrollbar_ || !self.horizontal_scrollbar_.Get().is_null()
        );
        debug_assert!(!self.has_vertical_scrollbar_ || !self.vertical_scrollbar_.Get().is_null());
        if !self.has_horizontal_scrollbar_ && !self.horizontal_scrollbar_.Get().is_null() {
            self.DestroyScrollbar(ScrollbarOrientation::kHorizontalScrollbar);
        }
        if !self.has_vertical_scrollbar_ && !self.vertical_scrollbar_.Get().is_null() {
            self.DestroyScrollbar(ScrollbarOrientation::kVerticalScrollbar);
        }
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:1900-1917. The
    // document-owned draggable-region dirty bit remains outside this package.
    pub fn RemoveScrollbarsForReconstruction(&mut self) {
        if !self.HasHorizontalScrollbar() && !self.HasVerticalScrollbar() {
            return;
        }
        if self.HasHorizontalScrollbar() {
            self.SetScrollbarNeedsPaintInvalidation(ScrollbarOrientation::kHorizontalScrollbar);
            self.SetHasHorizontalScrollbarInManager(false);
        }
        if self.HasVerticalScrollbar() {
            self.SetScrollbarNeedsPaintInvalidation(ScrollbarOrientation::kVerticalScrollbar);
            self.SetHasVerticalScrollbarInManager(false);
        }
        self.UpdateScrollCornerStyle();
        self.SetScrollCornerNeedsPaintInvalidation();
    }

    fn CreateScrollbar(&self, orientation: ScrollbarOrientation) -> *mut Scrollbar {
        let minimum = self.MinimumScrollOffset();
        let offset = self.GetScrollOffset();
        let current_pos = if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            offset.x() - minimum.x()
        } else {
            offset.y() - minimum.y()
        };
        let box_ = unsafe { &*self.GetLayoutBox() };
        let style_source = ScrollbarStyleSource(box_);
        let element = DynamicTo::<Element>(unsafe { &*style_source }.GetNode());
        let is_custom = unsafe { &*style_source }
            .StyleRef()
            .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>());
        let is_overlay = box_.ScrollbarThemeForLayout().uses_overlay_scrollbars && !is_custom;
        let css_scrollbar_width = unsafe { &*style_source }.StyleRef().UsedScrollbarWidth();
        let thickness =
            ScrollbarThemeThickness(box_.ScrollbarThemeForLayout(), css_scrollbar_width);
        let scrollbar = MakeGarbageCollected(Scrollbar::new(
            self as *const Self as *mut Self,
            orientation,
            style_source,
            is_custom,
            is_overlay,
            *box_.ScrollbarThemeForLayout(),
            css_scrollbar_width,
            thickness,
            current_pos,
        ));
        scrollbar
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2008-2022.
    pub fn VerticalScrollbarWidth(&self, behavior: OverlayScrollbarClipBehavior) -> i32 {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let scrollbar = self.VerticalScrollbar();
        if scrollbar.is_null() {
            return 0;
        }
        if behavior == OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize
            && box_.StyleRef().OverflowY() == EOverflow::kOverlay
        {
            return 0;
        }
        let scrollbar = unsafe { &*scrollbar };
        if (behavior == OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize
            || !scrollbar.ShouldParticipateInHitTesting())
            && scrollbar.IsOverlayScrollbar()
        {
            return 0;
        }
        scrollbar.ScrollbarThickness()
    }

    // Source: core/paint/paint_layer_scrollable_area.cc:2024-2038.
    pub fn HorizontalScrollbarHeight(&self, behavior: OverlayScrollbarClipBehavior) -> i32 {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let scrollbar = self.HorizontalScrollbar();
        if scrollbar.is_null() {
            return 0;
        }
        if behavior == OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize
            && box_.StyleRef().OverflowX() == EOverflow::kOverlay
        {
            return 0;
        }
        let scrollbar = unsafe { &*scrollbar };
        if (behavior == OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize
            || !scrollbar.ShouldParticipateInHitTesting())
            && scrollbar.IsOverlayScrollbar()
        {
            return 0;
        }
        scrollbar.ScrollbarThickness()
    }

    pub fn VerticalScrollbarWidthDefault(&self) -> i32 {
        self.VerticalScrollbarWidth(OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize)
    }

    pub fn HorizontalScrollbarHeightDefault(&self) -> i32 {
        self.HorizontalScrollbarHeight(OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize)
    }

    pub fn HasBeenDisposed(&self) -> bool {
        self.has_been_disposed_
    }

    pub fn RegisterTextOverflowPostLayoutSnapshot(
        &mut self,
        snapshot: *mut TextOverflowPostLayoutSnapshot,
    ) {
        self.text_overflow_snapshot_ = Member::from_ptr(snapshot);
    }

    pub fn GetTextOverflowPostLayoutSnapshot(&self) -> *mut TextOverflowPostLayoutSnapshot {
        self.text_overflow_snapshot_.Get()
    }

    pub(crate) fn UserScrollDisposalQueue(
        &self,
    ) -> Option<std::rc::Rc<std::cell::RefCell<Vec<(usize, foundation::CompositorElementId)>>>>
    {
        unsafe { self.registered_view_.Get().as_ref() }.map(|view| view.ScrollAreaDisposalQueue())
    }
    pub fn Dispose(&mut self) {
        if self.HasBeenDisposed() {
            return;
        }
        if let Some(view) = unsafe { self.registered_view_.Get().as_ref() } {
            view.UnregisterScrollableArea(self);
        }
        self.registered_view_ = foundation::WeakMember::default();
        self.scroll_anchor_.Dispose();
        // Source: core/paint/paint_layer_scrollable_area.cc:218-220.
        let element = DynamicTo::<Element>(unsafe { &*self.GetLayoutBox() }.GetNode());
        if !element.is_null() {
            unsafe { &mut *element }.SetSavedLayerScrollOffset(self.scroll_offset_);
        }
        // Source: core/paint/paint_layer_scrollable_area.cc:3026-3030.
        self.has_horizontal_scrollbar_ = false;
        self.has_vertical_scrollbar_ = false;
        self.DestroyScrollbar(ScrollbarOrientation::kHorizontalScrollbar);
        self.DestroyScrollbar(ScrollbarOrientation::kVerticalScrollbar);
        for owner in [
            &self.hypothetical_horizontal_style_owner_,
            &self.hypothetical_vertical_style_owner_,
        ] {
            if let Some(scrollbar) = unsafe { owner.get().Get().as_mut() } {
                scrollbar.DisconnectFromScrollableArea();
            }
            owner.set(Member::default());
        }
        self.scroll_corner_client_ = Member::default();
        self.scroll_corner_.set(Member::default());
        self.scrolling_background_client_ = Member::default();
        self.box_ = Member::default();
        self.RemovePendingSnapUpdate();
        self.has_been_disposed_ = true;
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
        visitor.Trace(&self.registered_view_);
        visitor.Trace(&self.text_overflow_snapshot_);
        visitor.Trace(&self.horizontal_scrollbar_);
        visitor.Trace(&self.vertical_scrollbar_);
        visitor.Trace(&self.scroll_corner_client_);
        visitor.Trace(&self.scroll_corner_.get());
        visitor.Trace(&self.hypothetical_horizontal_style_owner_.get());
        visitor.Trace(&self.hypothetical_vertical_style_owner_.get());
        visitor.Trace(&self.scrolling_background_client_);
        visitor.Trace(&self.pending_after_style_change_old_style_);
    }
}

impl Drop for PaintLayerScrollableArea {
    // Source: core/paint/paint_layer_scrollable_area.cc:184-186.
    fn drop(&mut self) {
        assert!(self.HasBeenDisposed());
    }
}

#[cfg(test)]
mod viewport_unit_scrollbar_tests {
    use super::*;
    use crate::internal::layout_input_types::ScrollbarThemeMetrics;

    fn properties(
        overflow_x: EOverflow,
        overflow_y: EOverflow,
        gutter: ScrollbarGutter,
        writing_mode: foundation::WritingMode,
    ) -> StyleBasedScrollbarData {
        StyleBasedScrollbarData {
            overflow_x,
            overflow_y,
            gutter,
            width: EScrollbarWidth::kAuto,
            writing_mode,
        }
    }

    fn classic_theme() -> ScrollbarThemeMetrics {
        ScrollbarThemeMetrics {
            auto_thickness: 15,
            thin_thickness: 7,
            uses_overlay_scrollbars: false,
            ..Default::default()
        }
    }

    #[test]
    fn unconditional_scrollbars_follow_overflow_axes() {
        let size = ComputeScrollbarWidthsForViewportUnitsFromInputs(
            properties(
                EOverflow::kScroll,
                EOverflow::kScroll,
                ScrollbarGutter::kScrollbarGutterAuto,
                foundation::WritingMode::kHorizontalTb,
            ),
            false,
            false,
            &classic_theme(),
        );
        assert_eq!(size, gfx::Size::new(15, 15));
    }

    #[test]
    fn stable_gutter_reserves_the_writing_mode_axis() {
        let horizontal = ComputeScrollbarWidthsForViewportUnitsFromInputs(
            properties(
                EOverflow::kAuto,
                EOverflow::kAuto,
                ScrollbarGutter::kScrollbarGutterStable
                    | ScrollbarGutter::kScrollbarGutterBothEdges,
                foundation::WritingMode::kHorizontalTb,
            ),
            false,
            false,
            &classic_theme(),
        );
        assert_eq!(horizontal, gfx::Size::new(30, 0));

        let vertical = ComputeScrollbarWidthsForViewportUnitsFromInputs(
            properties(
                EOverflow::kAuto,
                EOverflow::kAuto,
                ScrollbarGutter::kScrollbarGutterStable,
                foundation::WritingMode::kVerticalRl,
            ),
            false,
            false,
            &classic_theme(),
        );
        assert_eq!(vertical, gfx::Size::new(0, 15));
    }

    #[test]
    fn overlay_or_host_supplied_scrollbars_do_not_reduce_viewport_units() {
        let mut overlay = classic_theme();
        overlay.uses_overlay_scrollbars = true;
        let data = properties(
            EOverflow::kScroll,
            EOverflow::kScroll,
            ScrollbarGutter::kScrollbarGutterStable,
            foundation::WritingMode::kHorizontalTb,
        );
        assert_eq!(
            ComputeScrollbarWidthsForViewportUnitsFromInputs(data, false, false, &overlay),
            gfx::Size::default()
        );
        assert_eq!(
            ComputeScrollbarWidthsForViewportUnitsFromInputs(data, true, false, &classic_theme(),),
            gfx::Size::default()
        );
        assert_eq!(
            ComputeScrollbarWidthsForViewportUnitsFromInputs(data, false, true, &classic_theme(),),
            gfx::Size::default()
        );
    }

    #[test]
    fn touch_resizer_expands_toward_the_content() {
        let pointer = gfx::Rect::new(gfx::Point::new(85, 45), gfx::Size::new(15, 15));
        assert_eq!(
            ExpandResizerCornerRectForTouch(pointer),
            gfx::Rect::new(gfx::Point::new(70, 30), gfx::Size::new(30, 30))
        );
    }

    #[test]
    fn scroll_corner_or_resizer_prefers_the_real_corner() {
        let resizer = gfx::Rect::new(gfx::Point::new(85, 45), gfx::Size::new(15, 15));
        assert_eq!(
            ScrollCornerOrResizerRect(gfx::Rect::default(), resizer),
            resizer
        );
        let corner = gfx::Rect::new(gfx::Point::new(80, 40), gfx::Size::new(20, 20));
        assert_eq!(ScrollCornerOrResizerRect(corner, resizer), corner);
        assert!(RectContainsPoint(resizer, &gfx::Point::new(85, 45)));
        assert!(RectContainsPoint(resizer, &gfx::Point::new(99, 59)));
        assert!(!RectContainsPoint(resizer, &gfx::Point::new(100, 60)));
    }
}

/// Read-only native overflow-control result. Page supplies identity aliases
/// without exposing an Interaction result or retaining a copied geometry tree.
#[derive(Clone, Copy)]
pub enum OverflowControlHit {
    None,
    Resizer,
    Scrollbar(*mut Scrollbar),
    Corner,
}
impl PaintLayerScrollableArea {
    // Source: core/paint/paint_layer_scrollable_area.cc:2243-2280.
    pub fn HitTestOverflowControls(&self, local_point: &gfx::Point) -> OverflowControlHit {
        if !self.HasOverflowControls() {
            return OverflowControlHit::None;
        }
        if unsafe { &*self.GetLayoutBox() }.CanResize()
            && RectContainsPoint(
                self.ResizerCornerRect(ResizerHitTestType::kResizerForPointer),
                local_point,
            )
        {
            return OverflowControlHit::Resizer;
        }
        let vertical = self.VerticalScrollbar();
        if self.HasVerticalScrollbar()
            && !vertical.is_null()
            && unsafe { &*vertical }.ShouldParticipateInHitTesting()
            && RectContainsPoint(self.RectForVerticalScrollbar(), local_point)
        {
            return OverflowControlHit::Scrollbar(vertical);
        }
        let horizontal = self.HorizontalScrollbar();
        if self.HasHorizontalScrollbar()
            && !horizontal.is_null()
            && unsafe { &*horizontal }.ShouldParticipateInHitTesting()
            && RectContainsPoint(self.RectForHorizontalScrollbar(), local_point)
        {
            return OverflowControlHit::Scrollbar(horizontal);
        }
        if !self.scroll_corner_.get().Get().is_null()
            && RectContainsPoint(self.ScrollCornerRect(), local_point)
        {
            return OverflowControlHit::Corner;
        }
        OverflowControlHit::None
    }
    pub fn ScrollCornerStyle(&self) -> Option<&ComputedStyle> {
        unsafe { self.scroll_corner_.get().Get().as_ref() }.map(|corner| corner.StyleRef())
    }
    // cpp: paint_layer_scrollable_area.cc:2215-2241. Layout publishes a typed
    // request; the embedding resolves Style and returns the exact owner here.
    pub fn UpdateScrollCornerStyle(&mut self) {
        if self.ScrollCornerStyleSource().is_none() {
            self.scroll_corner_.set(Member::default());
            self.scroll_corner_style_pending_.set(false);
        } else {
            self.scroll_corner_style_pending_.set(true);
        }
    }
    pub fn ScrollCornerStyleSource(&self) -> Option<*mut LayoutObject> {
        if !self.NeedsScrollCorner() {
            return None;
        }
        let box_ = unsafe { &*self.GetLayoutBox() };
        let source = ScrollbarStyleSource(box_);
        (box_.IsScrollContainer() && !unsafe { &*source }.StyleRef().UsesStandardScrollbarStyle())
            .then_some(source)
    }
    pub fn ScrollCornerStyleUpdatePending(&self) -> bool {
        self.scroll_corner_style_pending_.get()
    }
    pub fn PublishScrollCornerStyle(&self, style: *const ComputedStyle) {
        if style.is_null() {
            self.scroll_corner_.set(Member::default());
        } else if let Some(corner) = unsafe { self.scroll_corner_.get().Get().as_mut() } {
            corner.SetStyle(style);
        } else {
            self.scroll_corner_
                .set(Member::from_ptr(MakeGarbageCollected(
                    super::super::layout_custom_scrollbar_part::LayoutCustomScrollbarPart::new(
                        std::ptr::null_mut(),
                        super::super::scrollbar_part::ScrollbarPart::kNoPart,
                        style,
                    ),
                )));
        }
        if let Some(corner) = unsafe { self.scroll_corner_.get().Get().as_mut() } {
            corner.SetPartFrameRect(&self.ScrollCornerRect());
        }
        self.scroll_corner_style_pending_.set(false);
    }
    pub fn HypotheticalScrollbarStyleOwners(&self) -> [*mut Scrollbar; 2] {
        [
            self.hypothetical_horizontal_style_owner_.get().Get(),
            self.hypothetical_vertical_style_owner_.get().Get(),
        ]
    }
    // cpp: custom_scrollbar.cc:195-205; the actual area/layout owners retain
    // their invalidation decisions, while Page observes only the retry effect.
    pub fn CustomScrollbarThicknessChanged(&mut self) {
        self.hypothetical_scrollbar_thicknesses_are_valid_ = false;
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        box_.SetChildNeedsLayout();
        box_.SetNeedsLayout(b"Scrollbar changed\0".as_ptr().cast());
        self.SetScrollCornerNeedsPaintInvalidation();
    }
}

#[path = "snap_coordinator.rs"]
mod native_snap;
pub use native_snap::{
    NativeLayoutSnapPlan, NativePageScrollCommand, NativePageScrollRequest,
    NativePendingSnapTarget, NativeSnapAnimationState, NativeSnapError,
    NativeSnapTargetNotification, NativeUserSnapScrollKind,
};

impl PaintLayerScrollableArea {
    // paint_layer_scrollable_area.cc:179-181, after allocation gives `this` its
    // original stable address (Rust's source constructor returns by value).
    pub(crate) fn RegisterWithFrameView(&mut self) {
        let view = unsafe { &*self.GetLayoutBox() }
            .InputOwnerForLayout()
            .InputLayoutView();
        if let Some(view) = unsafe { view.as_ref() } {
            view.RegisterScrollableArea(self);
            self.registered_view_ = foundation::WeakMember::from_ptr(view as *const _ as *mut _);
        }
    }
}

impl PaintLayerScrollableArea {
    pub fn PublishMacScrollbarAnimation(
        &mut self,
        horizontal: bool,
        thumb: f32,
        track: f32,
        width: i32,
        thumb_dirty: bool,
        track_dirty: bool,
        hidden: Option<bool>,
    ) -> bool {
        if self.HasBeenDisposed() {
            return false;
        }
        let orientation = if horizontal {
            ScrollbarOrientation::kHorizontalScrollbar
        } else {
            ScrollbarOrientation::kVerticalScrollbar
        };
        if horizontal {
            self.mac_horizontal_thumb_alpha_ = thumb;
            self.mac_horizontal_track_alpha_ = track;
            self.mac_horizontal_thumb_width_ = Some(width);
        } else {
            self.mac_vertical_thumb_alpha_ = thumb;
            self.mac_vertical_track_alpha_ = track;
            self.mac_vertical_thumb_width_ = Some(width);
        }
        if thumb_dirty || track_dirty {
            self.SetScrollbarNeedsPaintInvalidation(orientation);
        }
        if let Some(hidden) = hidden {
            self.SetScrollbarsHiddenIfOverlayFromOwner(hidden);
        }
        thumb_dirty || track_dirty || hidden.is_some()
    }
    pub fn ScrollbarPaintedTrackOpacity(&self, orientation: ScrollbarOrientation) -> f32 {
        if !unsafe { &*self.GetLayoutBox() }
            .ScrollbarThemeForLayout()
            .uses_macos_painted_scrollbars
        {
            return 1.;
        }
        if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            self.mac_horizontal_track_alpha_
        } else {
            self.mac_vertical_track_alpha_
        }
    }
}

impl PaintLayerScrollableArea {
    // PLSA.cc:2798-2808; updated from the actual PAC scroll-node membership
    // in the same commit, before source animation/visibility work resumes.
    pub fn PublishUsesCompositedScrolling(&mut self, value: bool) {
        self.uses_composited_scrolling_ = value;
    }
    pub fn UsesCompositedScrolling(&self) -> bool {
        self.uses_composited_scrolling_
    }
    pub fn SetScrollbarMouseOver(&mut self, value: bool) {
        self.mouse_over_scrollbar_ = value;
    }
    pub fn SetScrollbarCaptured(&mut self, value: bool) {
        self.scrollbar_captured_ = value;
    }
    pub fn MouseOverScrollbar(&self) -> bool {
        self.mouse_over_scrollbar_
    }
    pub fn ScrollbarCaptured(&self) -> bool {
        self.scrollbar_captured_
    }
    pub fn UsesCompositedOverlayScrollbars(&self) -> bool {
        if !unsafe { &*self.GetLayoutBox() }
            .ScrollbarThemeForLayout()
            .uses_overlay_scrollbars
        {
            return false;
        }
        if !foundation::RuntimeEnabledFeatures::RasterInducingScrollEnabled()
            && !self.UsesCompositedScrolling()
        {
            return false;
        }
        [self.HorizontalScrollbar(), self.VerticalScrollbar()]
            .into_iter()
            .any(|p| {
                unsafe { p.as_ref() }.is_some_and(|bar| {
                    (self.ScrollsOverflow() && bar.TotalSize() > bar.VisibleSize()
                        || self.ShouldSupplyScrollbarsForVisualViewport(bar.Orientation()))
                        && !bar.IsCustomScrollbar()
                        && !(unsafe { &*self.GetLayoutBox() }
                            .InputOwnerForLayout()
                            .InputCanvasDrawElementEnabled()
                            && unsafe { &*self.GetLayoutBox() }
                                .InputOwnerForLayout()
                                .InputCanvasLayoutSubtree())
                })
            })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NativeScrollbarInteraction {
    Entered,
    Exited,
    Captured,
    Released,
}
pub struct NativeOverlayVisibilityUpdate {
    pub stop_timer: bool,
    pub schedule_after_us: Option<u64>,
}
impl PaintLayerScrollableArea {
    // ScrollableArea.cc:682-715,961-1002. Theme/area eligibility is evaluated
    // here against the live source owner; Page only schedules the decided timer.
    pub fn HandleScrollbarInteraction(
        &mut self,
        event: NativeScrollbarInteraction,
        is_popup: bool,
    ) -> NativeOverlayVisibilityUpdate {
        use NativeScrollbarInteraction::*;
        match event {
            Entered => self.mouse_over_scrollbar_ = true,
            Exited => self.mouse_over_scrollbar_ = false,
            Captured => self.scrollbar_captured_ = true,
            Released => self.scrollbar_captured_ = false,
        }
        let mut update = NativeOverlayVisibilityUpdate {
            stop_timer: false,
            schedule_after_us: None,
        };
        if !matches!(event, Exited)
            || (self.HasOverlayScrollbars() && !self.ScrollbarsHiddenIfOverlay())
        {
            update = self.ShowNonMacOverlayScrollbars(is_popup);
        }
        if matches!(event, Entered | Captured) {
            update.stop_timer = true;
            update.schedule_after_us = None;
        }
        update
    }
    pub fn NotifyMacScrollbarMouseInteraction(&self, scrollbar: *mut Scrollbar, entered: bool) {
        self.EmitScrollbarActivity(
            scrollbar,
            if entered {
                NativeScrollbarActivityKind::MouseEntered
            } else {
                NativeScrollbarActivityKind::MouseExited
            },
        );
    }
    fn ShowNonMacOverlayScrollbars(&mut self, is_popup: bool) -> NativeOverlayVisibilityUpdate {
        let mut update = NativeOverlayVisibilityUpdate {
            stop_timer: false,
            schedule_after_us: None,
        };
        let theme = unsafe { &*self.GetLayoutBox() }.ScrollbarThemeForLayout();
        if !theme.uses_overlay_scrollbars
            || theme.uses_macos_painted_scrollbars
            || self.UsesCompositedOverlayScrollbars()
        {
            return update;
        }
        self.SetScrollbarsHiddenIfOverlayFromOwner(false);
        use foundation::scrollbar_theme_settings::ScrollbarThemeSettings as Settings;
        if Settings::MockScrollbarsEnabled() || !Settings::ScrollbarAnimationsEnabled() || is_popup
        {
            return update;
        }
        if !self.scrollbar_captured_ && !self.mouse_over_scrollbar_ {
            // WebThemeEngineDefault.cc:241-247 and ui/native_theme/
            // overlay_scrollbar_constants.cc:12-20.
            update.schedule_after_us = Some(if Settings::FluentScrollbarsEnabled() {
                850_000
            } else {
                700_000
            });
        }
        update
    }
    pub fn FadeOverlayScrollbarsTimerFired(&mut self) {
        if !self.UsesCompositedOverlayScrollbars() {
            self.SetScrollbarsHiddenIfOverlayFromOwner(true);
        }
    }
}

impl PaintLayerScrollableArea {
    pub fn SnappedQueryTargetIds(
        &self,
    ) -> Option<scroll_snap::scroll_snap_data::TargetSnapAreaElementIds> {
        self.snapped_query_target_ids_
    }
}

/// PLSA.h:620-629,cc:246-262. Copied source HistoryItem view-state fields
/// needed by this scroller; visual viewport scale remains on its own owner.
#[derive(Clone, Debug)]
pub struct NativePendingHistoryRestore {
    pub scroll_offset: ScrollOffset,
    pub anchor_selector: String,
    pub anchor_offset: gfx::PointF,
    pub scroll_behavior: layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior,
}
impl PaintLayerScrollableArea {
    pub fn SetPendingHistoryRestoreScrollOffset(
        &mut self,
        state: NativePendingHistoryRestore,
        should_restore_scroll: bool,
    ) {
        if should_restore_scroll {
            self.pending_history_restore_ = Some(state);
        }
    }
    pub fn HasPendingHistoryRestoreScrollOffset(&self) -> bool {
        self.pending_history_restore_.is_some()
    }
    pub fn PendingHistoryRestore(&self) -> Option<&NativePendingHistoryRestore> {
        self.pending_history_restore_.as_ref()
    }
    pub fn PendingHistoryRestoreNeedsSelectorQuery(&self) -> bool {
        let Some(pending) = self.pending_history_restore_.as_ref() else {
            return false;
        };
        let box_ = unsafe { &*self.GetLayoutBox() };
        box_.StyleRef().OverflowAnchor() != foundation::EOverflowAnchor::kNone
            && !box_.IsPrintingForLayout()
            && self
                .scroll_anchor_
                .RestoreNeedsSelectorQuery(&pending.anchor_selector)
    }
    pub fn ApplyPendingHistoryRestoreScrollOffset(
        &mut self,
        candidates: &[*mut LayoutObject],
    ) -> Option<NativePageScrollRequest> {
        let pending = self.pending_history_restore_.as_ref()?.clone();
        let box_ = unsafe { &*self.GetLayoutBox() };
        let area = self as *mut Self;
        let restore = box_.StyleRef().OverflowAnchor() != foundation::EOverflowAnchor::kNone
            && !box_.IsPrintingForLayout()
            && unsafe { &mut (*area).scroll_anchor_ }.RestoreAnchor(
                unsafe { &mut *area },
                &pending.anchor_selector,
                pending.anchor_offset,
                candidates,
            );
        let request = if restore {
            None
        } else {
            let mut request = self.ProgrammaticScrollRequest(
                pending.scroll_offset,
                scroll_snap::scroll_source_type::ScrollSourceType::kStationaryScroll,
                foundation::RuntimeEnabledFeatures::FractionalScrollOffsetsEnabled(),
            );
            use layoutng_style::style::scroll_enums::mojom::blink::ScrollBehavior;
            let behavior = if pending.scroll_behavior == ScrollBehavior::kAuto {
                self.ScrollBehaviorStyle()
            } else {
                pending.scroll_behavior
            };
            request.behavior = if behavior == ScrollBehavior::kSmooth {
                ScrollBehavior::kSmooth
            } else {
                ScrollBehavior::kInstant
            };
            Some(request)
        };
        self.pending_history_restore_ = None;
        request
    }
}
