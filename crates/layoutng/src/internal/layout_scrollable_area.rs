#![allow(non_snake_case)]

use foundation::{
    gfx, CompositorElementId, CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace,
    DynamicTo, EOverflow, EScrollbarWidth, LayoutUnit, MakeGarbageCollected, Member,
    MinimumValueForLength, OverlayScrollbarClipBehavior, PhysicalOffset, PhysicalRect,
    PhysicalSize, ToFlooredPoint, ToPixelSnappedRect, ToRoundedSize, Visitor,
};
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesBoth, kPhysicalAxesHorizontal, kPhysicalAxesNone, kPhysicalAxesVertical,
    PhysicalAxes,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_style::style::computed_style::ComputedStyle;

use super::layout_box::LayoutBox;
use super::layout_node_metadata::{Element, Node};
use super::native_scrollbar::{ScrollCornerDisplayItemClient, Scrollbar};
use super::scroll_types::{
    IncludeScrollbarsInRect, ScrollOffset, SnapScrollOffsetToPhysicalPixels,
};
use super::scrollbar_mode::mojom::blink::ScrollbarMode;
use super::scrollbar_orientation::ScrollbarOrientation;
use super::scrollbar_theme_metrics::ScrollbarThemeThickness;
use super::text_overflow_post_layout_snapshot::TextOverflowPostLayoutSnapshot;

// cpp: layoutng/internal/layout_scrollable_area.cc:127-151
fn ResolveScrollbarModes(box_: &LayoutBox) -> (ScrollbarMode, ScrollbarMode) {
    let mut style: &ComputedStyle = box_.StyleRef();
    if box_.IsLayoutView() {
        let viewport = box_.ViewportDefiningElementForLayout();
        if !viewport.is_null() {
            let viewport_object = unsafe { &*viewport }.GetLayoutObject();
            if !viewport_object.is_null() {
                style = unsafe { &*viewport_object }.StyleRef();
            }
        }
    }
    let resolve = |overflow: EOverflow| match overflow {
        EOverflow::kScroll => ScrollbarMode::kAlwaysOn,
        EOverflow::kHidden | EOverflow::kClip | EOverflow::kVisible => ScrollbarMode::kAlwaysOff,
        _ => ScrollbarMode::kAuto,
    };
    (resolve(style.OverflowX()), resolve(style.OverflowY()))
}

// The source's GC owner has no paint-layer base in this extracted package.
// cpp: layoutng/internal/layout_scrollable_area.h:65-68
// cpp: layoutng/internal/layout_scrollable_area.h:157-165
#[repr(C)]
pub struct PaintLayerScrollableArea {
    overflow_rect_: PhysicalRect,
    scroll_origin_: gfx::Point,
    scroll_origin_changed_: bool,
    has_horizontal_scrollbar_: bool,
    has_vertical_scrollbar_: bool,
    horizontal_scrollbar_: Member<Scrollbar>,
    vertical_scrollbar_: Member<Scrollbar>,
    scroll_corner_client_: Member<ScrollCornerDisplayItemClient>,
    box_: Member<LayoutBox>,
    scroll_offset_: ScrollOffset,
    text_overflow_snapshot_: Member<TextOverflowPostLayoutSnapshot>,
    has_been_disposed_: bool,
}

// cpp: layoutng/internal/layout_scrollable_area.h:141-144
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeScrollbarExistenceOption {
    kDependsOnOverflow,
    kOverflowIndependent,
}

impl PaintLayerScrollableArea {
    // cpp: layoutng/internal/layout_scrollable_area.h:70-70
    // cpp: layoutng/internal/layout_scrollable_area.cc:56-63
    pub fn new(box_: &mut LayoutBox) -> Self {
        let mut area = Self {
            overflow_rect_: PhysicalRect::default(),
            scroll_origin_: gfx::Point::default(),
            scroll_origin_changed_: false,
            has_horizontal_scrollbar_: false,
            has_vertical_scrollbar_: false,
            horizontal_scrollbar_: Member::default(),
            vertical_scrollbar_: Member::default(),
            scroll_corner_client_: Member::default(),
            box_: Member::from_ptr(box_),
            scroll_offset_: ScrollOffset::default(),
            text_overflow_snapshot_: Member::default(),
            has_been_disposed_: false,
        };
        let element = DynamicTo::<Element>(box_.GetNode());
        if !element.is_null() {
            area.scroll_offset_ = unsafe { &*element }.SavedLayerScrollOffset();
            unsafe { &mut *element }.SetSavedLayerScrollOffset(ScrollOffset::default());
        }
        area
    }

    pub fn HorizontalScrollbar(&self) -> *mut Scrollbar {
        self.horizontal_scrollbar_.Get()
    }
    pub fn VerticalScrollbar(&self) -> *mut Scrollbar {
        self.vertical_scrollbar_.Get()
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

    // cpp: layoutng/internal/layout_scrollable_area.h:73-73
    // cpp: layoutng/internal/layout_scrollable_area.cc:69-72
    pub fn FromNode(node: &Node) -> *mut Self {
        let box_ = node.GetLayoutBox();
        if box_.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*box_ }.GetScrollableArea()
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:74-74
    pub fn GetLayoutBox(&self) -> *mut LayoutBox {
        self.box_.Get()
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:75-75
    // cpp: layoutng/internal/layout_scrollable_area.cc:285-287
    pub fn GetScrollOffset(&self) -> ScrollOffset {
        self.scroll_offset_
    }

    // Adapter for Blink PaintLayerScrollableArea::UpdateScrollOffset. The host
    // owns scroll events and repaint scheduling; this extracted layout package
    // synchronizes the offset used by sticky paint properties without reflow.
    pub(crate) fn UpdateScrollOffset(&mut self, offset: ScrollOffset) {
        if !self.has_been_disposed_ {
            self.scroll_offset_ = offset;
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:76-76
    // cpp: layoutng/internal/scroll_geometry.cc:55-58
    pub fn GetScrollElementId(&self) -> CompositorElementId {
        let box_ = unsafe { &*self.GetLayoutBox() };
        CompositorElementIdFromUniqueObjectId(
            box_.UniqueId(),
            CompositorElementIdNamespace::kScroll,
        )
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:77-77
    // cpp: layoutng/internal/scroll_geometry.cc:301-303
    pub fn ScrollOffsetInt(&self) -> gfx::Vector2d {
        SnapScrollOffsetToPhysicalPixels(&self.scroll_offset_)
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:93-93
    // cpp: layoutng/internal/scroll_geometry.cc:74-80
    pub fn ScrollSize(&self, orientation: ScrollbarOrientation) -> i32 {
        let dimensions = self.MaximumScrollOffsetInt() - self.MinimumScrollOffsetInt();
        if orientation == ScrollbarOrientation::kHorizontalScrollbar {
            dimensions.x()
        } else {
            dimensions.y()
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:103-110
    // cpp: layoutng/internal/scroll_geometry.cc:82-89
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

    // cpp: layoutng/internal/layout_scrollable_area.h:111-112
    // cpp: layoutng/internal/scroll_geometry.cc:285-299
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

    // cpp: layoutng/internal/layout_scrollable_area.h:78-80
    pub fn ScrollOrigin(&self) -> gfx::Point {
        self.scroll_origin_
    }

    pub fn ScrollOriginChanged(&self) -> bool {
        self.scroll_origin_changed_
    }

    pub fn ResetScrollOriginChanged(&mut self) {
        self.scroll_origin_changed_ = false;
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:81-83
    pub fn ScrollOffsetToPosition(&self, offset: &ScrollOffset) -> gfx::PointF {
        gfx::PointF::from_point(self.ScrollOrigin()) + *offset
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:84-86
    pub fn ScrollPositionToOffset(&self, position: &gfx::PointF) -> ScrollOffset {
        *position - gfx::PointF::from_point(self.ScrollOrigin())
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:87-89
    pub fn ScrollPosition(&self) -> gfx::PointF {
        self.ScrollOffsetToPosition(&self.GetScrollOffset())
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:91-92
    // cpp: layoutng/internal/scroll_geometry.cc:169-175
    pub fn ScrollWidth(&self) -> LayoutUnit {
        self.overflow_rect_.Width()
    }

    pub fn ScrollHeight(&self) -> LayoutUnit {
        self.overflow_rect_.Height()
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:90-90
    // cpp: layoutng/internal/scroll_geometry.cc:161-167
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

    // cpp: layoutng/internal/layout_scrollable_area.h:113-113
    // cpp: layoutng/internal/scroll_geometry.cc:91-106
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

    // cpp: layoutng/internal/layout_scrollable_area.h:114-114
    // cpp: layoutng/internal/scroll_geometry.cc:108-115
    pub fn VisibleContentRect(&self, inclusion: IncludeScrollbarsInRect) -> gfx::Rect {
        let rect = self.LayoutContentRect(inclusion);
        gfx::Rect::new(ToFlooredPoint(rect.offset), ToRoundedSize(rect.size))
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:115-115
    // cpp: layoutng/internal/scroll_geometry.cc:117-132
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

    // cpp: layoutng/internal/layout_scrollable_area.h:116-117
    // cpp: layoutng/internal/scroll_geometry.cc:134-159
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

    // cpp: layoutng/internal/layout_scrollable_area.h:118-118
    // cpp: layoutng/internal/scroll_geometry.cc:229-240
    pub fn LocalToScrollOriginOffset(&self) -> PhysicalOffset {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let adjustment = box_.OriginAdjustmentForScrollbars();
        let mut border_to_scroll_origin = -box_.BorderOutsets().Offset();
        border_to_scroll_origin.left -= LayoutUnit::from_signed(adjustment.x());
        border_to_scroll_origin.top -= LayoutUnit::from_signed(adjustment.y());
        border_to_scroll_origin += PhysicalOffset::FromVector2dFFloor(&self.GetScrollOffset());
        border_to_scroll_origin
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:154-154
    // cpp: layoutng/internal/scroll_geometry.cc:242-271
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

    // cpp: layoutng/internal/layout_scrollable_area.h:145-146
    // cpp: layoutng/internal/layout_scrollable_area.cc:108-123
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

    // cpp: layoutng/internal/layout_scrollable_area.h:147-150
    // cpp: layoutng/internal/layout_scrollable_area.cc:155-218
    pub fn ComputeScrollbarExistence(
        &self,
        needs_horizontal_scrollbar: &mut bool,
        needs_vertical_scrollbar: &mut bool,
        option: ComputeScrollbarExistenceOption,
    ) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if box_.InputOwnerForLayout().InputPrinting()
            || box_.ViewportDefiningElementForLayout() as *mut Node == box_.GetNode()
            || box_.IsFieldset()
            || box_.IsFrameSet()
            || box_.StyleRef().UsedScrollbarWidth() == EScrollbarWidth::kNone
        {
            *needs_horizontal_scrollbar = false;
            *needs_vertical_scrollbar = false;
            return;
        }

        let (mut horizontal_mode, mut vertical_mode) = ResolveScrollbarModes(box_);
        let element = DynamicTo::<Element>(box_.GetNode());
        let has_custom_scrollbar_style = box_
            .StyleRef()
            .HasCustomScrollbarStyle(element.cast::<layoutng_style::style::forward::Element>());
        let will_be_overlay =
            box_.ScrollbarThemeForLayout().uses_overlay_scrollbars && !has_custom_scrollbar_style;
        if will_be_overlay {
            if horizontal_mode == ScrollbarMode::kAlwaysOn {
                horizontal_mode = ScrollbarMode::kAuto;
            }
            if vertical_mode == ScrollbarMode::kAlwaysOn {
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

    // cpp: layoutng/internal/layout_scrollable_area.h:151-152
    // cpp: layoutng/internal/layout_scrollable_area.cc:220-248
    pub fn TryRemovingAutoScrollbars(
        &self,
        needs_horizontal_scrollbar: bool,
        needs_vertical_scrollbar: bool,
    ) -> bool {
        if !needs_horizontal_scrollbar && !needs_vertical_scrollbar {
            return false;
        }
        let box_ = unsafe { &*self.GetLayoutBox() };
        let (horizontal_mode, vertical_mode) = ResolveScrollbarModes(box_);
        if horizontal_mode != ScrollbarMode::kAuto || vertical_mode != ScrollbarMode::kAuto {
            return false;
        }
        if box_.IsLayoutView() {
            let visible_size = self
                .VisibleContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
                .size();
            return self.ScrollWidth() <= LayoutUnit::from_signed(visible_size.width())
                && self.ScrollHeight() <= LayoutUnit::from_signed(visible_size.height());
        }
        if !box_.HasAutoVerticalScrollbar() || !box_.HasAutoHorizontalScrollbar() {
            return false;
        }
        let client_size = self
            .LayoutContentRect(IncludeScrollbarsInRect::kIncludeScrollbars)
            .size;
        self.ScrollWidth() <= client_size.width && self.ScrollHeight() <= client_size.height
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:120-124
    // cpp: layoutng/internal/layout_scrollable_area.cc:250-283
    pub fn UpdateAfterLayout(&mut self) {
        let freeze_state = unsafe { &*self.GetLayoutBox() }.GetScrollbarFreezeState();
        let horizontal_frozen =
            !freeze_state.is_null() && unsafe { &*freeze_state }.IsHorizontalScrollbarFrozen();
        let vertical_frozen =
            !freeze_state.is_null() && unsafe { &*freeze_state }.IsVerticalScrollbarFrozen();

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
        }
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:153-153
    // cpp: layoutng/internal/scroll_geometry.cc:177-190
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

    // cpp: layoutng/internal/layout_scrollable_area.h:119-119
    // cpp: layoutng/internal/scroll_geometry.cc:190-201
    pub fn UpdateScrollDimensions(&mut self) {
        let mut new_overflow_rect = unsafe { &*self.GetLayoutBox() }.ScrollableOverflowRect();
        new_overflow_rect.Unite(&PhysicalRect::new(
            new_overflow_rect.offset,
            self.LayoutContentRect(IncludeScrollbarsInRect::kExcludeScrollbars)
                .size,
        ));
        self.overflow_rect_ = new_overflow_rect;
        self.UpdateScrollOrigin();
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:94-94
    // cpp: layoutng/internal/scroll_geometry.cc:203-227
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

    // cpp: layoutng/internal/layout_scrollable_area.h:155-155
    // cpp: layoutng/internal/scroll_geometry.cc:273-283
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

    // cpp: layoutng/internal/layout_scrollable_area.h:156-156
    // cpp: layoutng/internal/scroll_geometry.cc:61-72
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

    // cpp: layoutng/internal/layout_scrollable_area.h:95-96
    pub fn HasHorizontalScrollbar(&self) -> bool {
        self.has_horizontal_scrollbar_
    }

    pub fn HasVerticalScrollbar(&self) -> bool {
        self.has_vertical_scrollbar_
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:97-97
    // cpp: layoutng/internal/layout_scrollable_area.cc:74-79
    pub fn SetHasHorizontalScrollbar(&mut self, has_scrollbar: bool) {
        let state = unsafe { &*self.GetLayoutBox() }.GetScrollbarFreezeState();
        if !state.is_null() && unsafe { &*state }.IsHorizontalScrollbarFrozen() {
            return;
        }
        if has_scrollbar == self.has_horizontal_scrollbar_ {
            return;
        }
        // ScrollbarManager creates the real client once and keeps it while
        // attached. Removing the bar disconnects that entity before releasing it.
        if has_scrollbar {
            let style_source = self
                .GetLayoutBox()
                .cast::<super::layout_object::LayoutObject>();
            self.horizontal_scrollbar_ = Member::from_ptr(MakeGarbageCollected(Scrollbar::new(
                self as *mut _,
                ScrollbarOrientation::kHorizontalScrollbar,
                style_source,
            )));
            if self.scroll_corner_client_.Get().is_null() {
                self.scroll_corner_client_ = Member::from_ptr(MakeGarbageCollected(
                    ScrollCornerDisplayItemClient::new(self as *mut _),
                ));
            }
        } else {
            let scrollbar = self.horizontal_scrollbar_.Get();
            if !scrollbar.is_null() {
                unsafe { &mut *scrollbar }.DisconnectFromScrollableArea();
            }
            self.horizontal_scrollbar_ = Member::default();
        }
        self.has_horizontal_scrollbar_ = has_scrollbar;
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:98-98
    // cpp: layoutng/internal/layout_scrollable_area.cc:81-88
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
        // ScrollbarManager creates the real client once and keeps it while
        // attached. Removing the bar disconnects that entity before releasing it.
        if has_scrollbar {
            let style_source = self
                .GetLayoutBox()
                .cast::<super::layout_object::LayoutObject>();
            self.vertical_scrollbar_ = Member::from_ptr(MakeGarbageCollected(Scrollbar::new(
                self as *mut _,
                ScrollbarOrientation::kVerticalScrollbar,
                style_source,
            )));
            if self.scroll_corner_client_.Get().is_null() {
                self.scroll_corner_client_ = Member::from_ptr(MakeGarbageCollected(
                    ScrollCornerDisplayItemClient::new(self as *mut _),
                ));
            }
        } else {
            let scrollbar = self.vertical_scrollbar_.Get();
            if !scrollbar.is_null() {
                unsafe { &mut *scrollbar }.DisconnectFromScrollableArea();
            }
            self.vertical_scrollbar_ = Member::default();
        }
        self.has_vertical_scrollbar_ = has_scrollbar;
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:99-102
    // cpp: layoutng/internal/layout_scrollable_area.cc:90-106
    pub fn VerticalScrollbarWidth(&self, _behavior: OverlayScrollbarClipBehavior) -> i32 {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if !self.HasVerticalScrollbar() || box_.ScrollbarThemeForLayout().uses_overlay_scrollbars {
            return 0;
        }
        ScrollbarThemeThickness(
            box_.ScrollbarThemeForLayout(),
            box_.StyleRef().UsedScrollbarWidth(),
        )
    }

    pub fn HorizontalScrollbarHeight(&self, _behavior: OverlayScrollbarClipBehavior) -> i32 {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if !self.HasHorizontalScrollbar() || box_.ScrollbarThemeForLayout().uses_overlay_scrollbars
        {
            return 0;
        }
        ScrollbarThemeThickness(
            box_.ScrollbarThemeForLayout(),
            box_.StyleRef().UsedScrollbarWidth(),
        )
    }

    pub fn VerticalScrollbarWidthDefault(&self) -> i32 {
        self.VerticalScrollbarWidth(OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize)
    }

    pub fn HorizontalScrollbarHeightDefault(&self) -> i32 {
        self.HorizontalScrollbarHeight(OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize)
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:128-128
    pub fn HasBeenDisposed(&self) -> bool {
        self.has_been_disposed_
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:130-136
    pub fn RegisterTextOverflowPostLayoutSnapshot(
        &mut self,
        snapshot: *mut TextOverflowPostLayoutSnapshot,
    ) {
        self.text_overflow_snapshot_ = Member::from_ptr(snapshot);
    }

    pub fn GetTextOverflowPostLayoutSnapshot(&self) -> *mut TextOverflowPostLayoutSnapshot {
        self.text_overflow_snapshot_.Get()
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:126-126
    // cpp: layoutng/internal/layout_scrollable_area.cc:289-299
    pub fn Dispose(&mut self) {
        if self.HasBeenDisposed() {
            return;
        }
        let element = DynamicTo::<Element>(unsafe { &*self.GetLayoutBox() }.GetNode());
        if !element.is_null() {
            unsafe { &mut *element }.SetSavedLayerScrollOffset(self.scroll_offset_);
        }
        for scrollbar in [
            self.horizontal_scrollbar_.Get(),
            self.vertical_scrollbar_.Get(),
        ] {
            if !scrollbar.is_null() {
                unsafe { &mut *scrollbar }.DisconnectFromScrollableArea();
            }
        }
        self.horizontal_scrollbar_ = Member::default();
        self.vertical_scrollbar_ = Member::default();
        self.scroll_corner_client_ = Member::default();
        self.box_ = Member::default();
        self.has_been_disposed_ = true;
    }

    // cpp: layoutng/internal/layout_scrollable_area.h:141-141
    // cpp: layoutng/internal/layout_scrollable_area.cc:301-304
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
        visitor.Trace(&self.text_overflow_snapshot_);
        visitor.Trace(&self.horizontal_scrollbar_);
        visitor.Trace(&self.vertical_scrollbar_);
        visitor.Trace(&self.scroll_corner_client_);
    }
}

impl Drop for PaintLayerScrollableArea {
    // cpp: layoutng/internal/layout_scrollable_area.h:71-71
    // cpp: layoutng/internal/layout_scrollable_area.cc:65-67
    fn drop(&mut self) {
        assert!(self.HasBeenDisposed());
    }
}
