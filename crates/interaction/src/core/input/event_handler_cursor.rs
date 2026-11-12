/*
 * Copyright (C) 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2006 Alexey Proskuryakov (ap@webkit.org)
 * Copyright (C) 2012 Digia Plc. and/or its subsidiary(-ies)
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL APPLE COMPUTER, INC. OR
 * CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
 * EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY
 * OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/input/event_handler.h:251,292-299,340-347,350,465
// Source: third_party/blink/renderer/core/input/event_handler.cc:174-192,215-252,260-263,485-854
// Cursor-only slice of EventHandler. The main manager retains its original
// frame, selection, mouse, scroll and capture owners; this facade borrows them
// for one synchronous call. No copied hit-test/layout/style tree is retained.
// Dependencies below project only source facts or source operations owned by
// the corresponding DOM, Layout, image and platform objects. Cursor choice,
// scrollbar/area precedence, resize, image rejection and fallback stay here.

#![allow(non_snake_case, non_camel_case_types)]

use std::{fmt, rc::Rc, sync::Arc};

// Source: ui/gfx/geometry/{point,point_f,size,size_f,rect_f}.h (value ports).
// Small coordinate values cross the boundary; resident geometry does not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SizeF {
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub origin: PointF,
    pub size: SizeF,
}
impl Rect {
    fn Contains(&self, other: &Self) -> bool {
        other.origin.x >= self.origin.x
            && other.origin.y >= self.origin.y
            && other.origin.x + other.size.width <= self.origin.x + self.size.width
            && other.origin.y + other.size.height <= self.origin.y + self.size.height
    }
}

// Source: ui/base/cursor/mojom/cursor_type.mojom:10-67.
// Kept source-shaped here until the shared ui cursor value is translated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CursorType {
    kNull = -1,
    kPointer,
    kCross,
    kHand,
    kIBeam,
    kWait,
    kHelp,
    kEastResize,
    kNorthResize,
    kNorthEastResize,
    kNorthWestResize,
    kSouthResize,
    kSouthEastResize,
    kSouthWestResize,
    kWestResize,
    kNorthSouthResize,
    kEastWestResize,
    kNorthEastSouthWestResize,
    kNorthWestSouthEastResize,
    kColumnResize,
    kRowResize,
    kMiddlePanning,
    kEastPanning,
    kNorthPanning,
    kNorthEastPanning,
    kNorthWestPanning,
    kSouthPanning,
    kSouthEastPanning,
    kSouthWestPanning,
    kWestPanning,
    kMove,
    kVerticalText,
    kCell,
    kContextMenu,
    kAlias,
    kProgress,
    kNoDrop,
    kCopy,
    kNone,
    kNotAllowed,
    kZoomIn,
    kZoomOut,
    kGrab,
    kGrabbing,
    kMiddlePanningVertical,
    kMiddlePanningHorizontal,
    kCustom,
    kDndNone,
    kDndMove,
    kDndCopy,
    kDndLink,
    kEastWestNoResize,
    kNorthSouthNoResize,
    kNorthEastSouthWestNoResize,
    kNorthWestSouthEastNoResize,
}

// Source: core/css/css_properties.json5:3393-3407,5446-5452.
// Generated ECursor/EResize values, without a Layout engine dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ECursor {
    kAuto,
    kDefault,
    kNone,
    kContextMenu,
    kHelp,
    kPointer,
    kProgress,
    kWait,
    kCell,
    kCrosshair,
    kText,
    kVerticalText,
    kAlias,
    kCopy,
    kMove,
    kNoDrop,
    kNotAllowed,
    kEResize,
    kNResize,
    kNeResize,
    kNwResize,
    kSResize,
    kSeResize,
    kSwResize,
    kWResize,
    kEwResize,
    kNsResize,
    kNeswResize,
    kNwseResize,
    kColResize,
    kRowResize,
    kAllScroll,
    kZoomIn,
    kZoomOut,
    kGrab,
    kGrabbing,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EResize {
    kNone,
    kBoth,
    kHorizontal,
    kVertical,
    kBlock,
    kInline,
}

// Source: ui/base/cursor/cursor.h:66-104; cursor.cc:38-61,92-108.
// A bitmap reference retains the existing immutable resource. Pixel reads borrow
// that owner; only the native host copies bytes into a platform cursor.
// Native cursor creation is performed only when CursorView::SetCursor receives
// this already selected value.
pub trait CursorBitmap: Send + Sync {
    fn Size(&self) -> Size;
    /// Immutable, unpremultiplied RGBA8 pixels in display orientation.
    /// Rows may be padded; `RowBytes` is at least `Size().width * 4`.
    fn Rgba8(&self) -> &[u8];
    fn RowBytes(&self) -> usize;
}
#[derive(Clone)]
pub enum Cursor {
    Standard(CursorType),
    Custom {
        bitmap: Arc<dyn CursorBitmap>,
        hot_spot: Point,
        image_scale_factor: f32,
    },
}
impl Default for Cursor {
    fn default() -> Self {
        Self::Standard(CursorType::kPointer)
    }
}
impl PartialEq for Cursor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Standard(a), Self::Standard(b)) => a == b,
            (
                Self::Custom {
                    bitmap: a,
                    hot_spot: ah,
                    image_scale_factor: ascale,
                },
                Self::Custom {
                    bitmap: b,
                    hot_spot: bh,
                    image_scale_factor: bscale,
                },
            ) => Arc::ptr_eq(a, b) && ah == bh && ascale == bscale,
            _ => false,
        }
    }
}
impl fmt::Debug for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard(kind) => f.debug_tuple("Standard").field(kind).finish(),
            Self::Custom {
                bitmap,
                hot_spot,
                image_scale_factor,
            } => f
                .debug_struct("Custom")
                .field("bitmap", &Arc::as_ptr(bitmap))
                .field("size", &bitmap.Size())
                .field("hot_spot", hot_spot)
                .field("image_scale_factor", image_scale_factor)
                .finish(),
        }
    }
}
impl Cursor {
    pub fn NewCustom(bitmap: Arc<dyn CursorBitmap>, hot_spot: Point, scale: f32) -> Self {
        assert!(scale > 0.0);
        let size = bitmap.Size();
        let hot_spot = if size.width > 0 && size.height > 0 {
            Point {
                x: hot_spot.x.clamp(0, size.width - 1),
                y: hot_spot.y.clamp(0, size.height - 1),
            }
        } else {
            Point::default()
        };
        Self::Custom {
            bitmap,
            hot_spot,
            image_scale_factor: scale,
        }
    }

    pub fn AreDimensionsValidForWeb(size: Size, scale_factor: f32) -> bool {
        if scale_factor == 0.0 {
            return false;
        }
        let inverse_scale = 1.0 / scale_factor;
        (size.width as f32 * inverse_scale).ceil() <= 128.0
            && (size.height as f32 * inverse_scale).ceil() <= 128.0
    }
}

// Source: event_handler.cc:215-252,525-757.
// Every reference borrows its canonical owner. GetCursor invokes the existing
// LayoutObject virtual override; it must not implement CSS cursor selection.
pub trait CursorHitTestLocation {
    fn Point(&self) -> PointF;
}
pub trait CursorFrameHitTestResult: CursorHitTestResult {
    fn InnerNodeFrame(&self) -> Option<&dyn CursorFrame>;
}
pub trait CursorHitTestResult {
    fn InnerPossiblyPseudoNode(&self) -> Option<&dyn CursorNode>;
    fn GetScrollbar(&self) -> Option<&dyn CursorScrollbar>;
    fn IsOverScrollCorner(&self) -> bool;
    fn IsOverLink(&self) -> bool;
    fn LocalPoint(&self) -> PointF;
}
pub trait CursorNode {
    fn IsLink(&self) -> bool;
    fn AsHTMLInputElement(&self) -> Option<&dyn CursorInputElement>;
    fn IsEditable(&self) -> bool;
    fn IsTextNode(&self) -> bool;
    fn CanStartSelection(&self) -> bool;
    fn IsHTMLAreaElement(&self) -> bool;
    fn GetLayoutObject(&self) -> Option<&dyn CursorLayoutObject>;
    fn GetComputedStyle(&self) -> Option<&dyn CursorComputedStyle>;
    fn ImageElement(&self) -> Option<Rc<dyn CursorNode>>;
}
pub trait CursorInputElement {
    fn FormControlType(&self) -> CursorFormControlType;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorFormControlType {
    kInputImage,
    kOther,
}
pub enum CursorDirective {
    kSetCursorBasedOnStyle,
    kSetCursor(Cursor),
    kDoNotSetCursor,
}
pub trait CursorLayoutObject {
    fn EnclosingLayer(&self) -> &dyn CursorPaintLayer;
    fn GetScrollableArea(&self) -> Option<&dyn CursorScrollableArea>;
    fn IsDocumentElement(&self) -> bool;
    fn ViewScrollableArea(&self) -> Option<&dyn CursorScrollableArea>;
    fn GetCursor(&self, local_point: PointF) -> CursorDirective;
    fn StyleRef(&self) -> &dyn CursorComputedStyle;
}
pub trait CursorPaintLayer {
    fn GetScrollableArea(&self) -> Option<&dyn CursorScrollableArea>;
    fn GetLayoutBox(&self) -> &dyn CursorLayoutBox;
}
pub trait CursorLayoutBox {
    fn StyleRef(&self) -> &dyn CursorComputedStyle;
    fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeft(&self) -> bool;
}
pub trait CursorScrollableArea {
    fn IsAbsolutePointInResizeControlForPointer(&self, point: Point) -> bool;
    fn ScrollCornerStyle(&self) -> Option<&dyn CursorComputedStyle>;
}
pub trait CursorScrollbar {
    fn IsCustomScrollbar(&self) -> bool;
    fn HoveredPart(&self) -> u32;
    fn GetScrollbarPartStyleForCursor(&self, part: u32) -> Option<&dyn CursorComputedStyle>;
}
pub trait CursorComputedStyle {
    fn Cursors(&self) -> Option<Box<dyn Iterator<Item = Box<dyn CursorData + '_>> + '_>>;
    fn Cursor(&self) -> ECursor;
    fn IsHorizontalWritingMode(&self) -> bool;
    fn UsedResize(&self) -> EResize;
    // Source: core/css/style_engine.cc:4659-4671. The frame's StyleEngine
    // resolves embedding color scheme from these native computed facts.
    fn ColorSchemeFlagsIsNormal(&self) -> bool;
    fn DarkColorScheme(&self) -> bool;
}
pub trait CursorData {
    fn GetImage(&self) -> Option<&dyn CursorStyleImage>;
    fn HotSpot(&self) -> Point;
    fn HotSpotSpecified(&self) -> bool;
}
pub trait CursorStyleImage {
    fn CanRender(&self) -> bool;
    fn CachedImage(&self) -> Option<&dyn CursorImageContent>;
    fn ImageScaleFactor(&self) -> f32;
    // The image owner computes concrete object size, respecting orientation.
    fn ImageSize(&self, multiplier: f32, default_object_size: SizeF) -> Option<SizeF>;
}
pub trait CursorImageContent {
    fn GetImage(&self) -> &dyn CursorImage;
}
pub trait CursorImage {
    fn IsSVGImage(&self) -> bool;
    fn SizeRespectingOrientation(&self) -> Size;
    fn GetHotSpot(&self) -> Option<Point>;
    fn RectContains(&self, point: Point) -> bool;
    // Image::AsSkBitmapForCurrentFrame can return an empty SkBitmap when
    // its producer cannot supply the current frame (image.cc:399-425).
    // Option preserves that absence without manufacturing drawable pixels.
    fn AsBitmapForCurrentFrameRespectingOrientation(&self) -> Option<Arc<dyn CursorBitmap>>;
    // SVGImageForContainer::Create remains with the document-image owner.
    fn SVGImageForContainer(
        &self,
        size_in_device_pixels: SizeF,
        device_scale_factor: f32,
        color_scheme: ColorScheme,
    ) -> Option<Rc<dyn CursorImage>>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    kLight,
    kDark,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorDocumentUpdateReason {
    kInput,
    kSelection,
}
/// The exact LocalFrame facts consumed by SelectCursor/SelectAutoCursor.
/// Keeping this policy projection separate from View/host cursor publication
/// lets a prepared resident query borrow its real owners without fabricating a
/// second EventHandler/View graph.
pub trait CursorPolicyFrame {
    fn Page(&self) -> Option<&dyn CursorPage>;
    fn UpdateStyleAndLayout(&self, reason: CursorDocumentUpdateReason);
    fn HTMLAreaElementDisplayNoneEnabled(&self) -> bool;
    fn ResolveColorSchemeForEmbedding(&self, style: &dyn CursorComputedStyle) -> ColorScheme;
    fn VisibleSelectionInDomTreeIsNone(&self) -> bool;
    fn ContentLocalToRootAncestorPoint(&self, point: PointF) -> PointF;
    fn MapContentRectToVisualRectInRootAncestorSpace(&self, rect: Rect) -> Rect;
}
pub trait CursorFrame: CursorPolicyFrame {
    fn IsLocalFrameRoot(&self) -> bool;
    fn View(&self) -> Option<&dyn CursorView>;
    fn EventHandlerCursor(&self) -> EventHandlerCursor<'_>;
}
pub trait CursorView {
    fn ShouldSetCursor(&self) -> bool;
    fn LayoutView(&self) -> Option<&dyn CursorLayoutView>;
    fn ViewportToFrame(&self, point: PointF) -> Box<dyn CursorHitTestLocation + '_>;
    fn SetCursor(&self, cursor: Cursor);
}
pub trait CursorLayoutView {
    // HitTestRequest::kReadOnly | HitTestRequest::kAllowChildFrameContent.
    // Named request facts avoid exposing a second hit-test bitmask type.
    fn HitTest(
        &self,
        location: &dyn CursorHitTestLocation,
        read_only: bool,
        allow_child_frame_content: bool,
    ) -> Box<dyn CursorFrameHitTestResult + '_>;
}
pub trait CursorPage {
    fn DeviceScaleFactor(&self, frame: &dyn CursorPolicyFrame) -> f32;
    fn SystemCursorSize(&self, frame: &dyn CursorPolicyFrame) -> SizeF;
    fn VisibleContentRectExcludingScrollbars(&self) -> Rect;
}
pub trait CursorMouseEventManager {
    fn LastKnownMousePositionInViewport(&self) -> PointF;
    fn MousePressed(&self) -> bool;
    fn MouseDownMayStartDrag(&self) -> bool;
}
pub trait CursorScrollManager {
    fn InResizeMode(&self) -> bool;
    fn MiddleClickAutoscrollInProgress(&self) -> bool;
}
pub trait CursorSelectionController {
    fn MouseDownMayStartSelect(&self) -> bool;
}
// Capture and accessibility values stay live across style/layout reentry.
pub trait CursorEventHandlerState {
    fn HasCapturingMouseEventsElement(&self) -> bool;
    fn cursor_accessibility_scale_factor(&self) -> f32;
}

// Source: event_handler.h:292-299,340-347,465.
// Values are borrowed from the existing EventHandler, including the capture
// slot and scale (not a second retained state store).
pub struct EventHandlerCursor<'a> {
    pub frame: &'a dyn CursorFrame,
    pub mouse_event_manager: &'a dyn CursorMouseEventManager,
    pub scroll_manager: &'a dyn CursorScrollManager,
    pub selection_controller: &'a dyn CursorSelectionController,
    pub state: &'a dyn CursorEventHandlerState,
}

pub struct EventHandlerCursorPolicy<'a> {
    pub frame: &'a dyn CursorPolicyFrame,
    pub mouse_event_manager: &'a dyn CursorMouseEventManager,
    pub scroll_manager: &'a dyn CursorScrollManager,
    pub selection_controller: &'a dyn CursorSelectionController,
    pub state: &'a dyn CursorEventHandlerState,
}
impl EventHandlerCursor<'_> {
    pub fn Policy(&self) -> EventHandlerCursorPolicy<'_> {
        EventHandlerCursorPolicy {
            frame: self.frame,
            mouse_event_manager: self.mouse_event_manager,
            scroll_manager: self.scroll_manager,
            selection_controller: self.selection_controller,
            state: self.state,
        }
    }
    pub fn SelectCursor(
        &self,
        location: &dyn CursorHitTestLocation,
        result: &dyn CursorHitTestResult,
    ) -> Option<Cursor> {
        self.Policy().SelectCursor(location, result)
    }
    pub fn SelectAutoCursor(
        &self,
        result: &dyn CursorHitTestResult,
        node: Option<&dyn CursorNode>,
        i_beam: CursorType,
    ) -> Option<Cursor> {
        self.Policy().SelectAutoCursor(result, node, i_beam)
    }
}

// Source: event_handler.cc:174-192.
fn DetermineHotSpot(image: &dyn CursorImage, specified: bool, hot_spot: Point) -> Point {
    if specified {
        return hot_spot;
    }
    if let Some(intrinsic) = image.GetHotSpot() {
        if image.RectContains(intrinsic) {
            return intrinsic;
        }
    }
    Point::default()
}

// Source: event_handler.cc:215-252.
fn GetComputedStyleFromScrollbar<'a>(
    layout_object: &'a dyn CursorLayoutObject,
    result: &'a dyn CursorHitTestResult,
) -> Option<&'a dyn CursorComputedStyle> {
    if result.IsOverScrollCorner() {
        let mut scrollable_area = layout_object.GetScrollableArea();
        if layout_object.IsDocumentElement() {
            scrollable_area = layout_object.ViewScrollableArea();
        }
        let scrollable_area = scrollable_area?;
        if let Some(style) = scrollable_area.ScrollCornerStyle() {
            return Some(style);
        }
    }
    if let Some(scrollbar) = result.GetScrollbar() {
        if scrollbar.IsCustomScrollbar() {
            if let Some(style) = scrollbar.GetScrollbarPartStyleForCursor(scrollbar.HoveredPart()) {
                return Some(style);
            }
        }
    }
    None
}

// Source: event_handler.cc:260-263.
const MAXIMUM_CURSOR_SIZE_WITHOUT_FALLBACK: f32 = 32.0;

// Source: event_handler.cc:485-489.
fn IsSubmitImage(node: &dyn CursorNode) -> bool {
    node.AsHTMLInputElement()
        .is_some_and(|input| input.FormControlType() == CursorFormControlType::kInputImage)
}

impl EventHandlerCursor<'_> {
    // Source: event_handler.h:251; event_handler.cc:491-495.
    pub fn UsesHandCursor(node: Option<&dyn CursorNode>) -> bool {
        node.is_some_and(|node| (node.IsLink() || IsSubmitImage(node)) && !node.IsEditable())
    }

    // Source: event_handler.cc:497-502.
    pub fn CursorUpdateTimerFired(&self) {
        self.UpdateCursor();
    }

    // Source: event_handler.h:292; event_handler.cc:504-535.
    pub fn UpdateCursor(&self) {
        // TRACE_EVENT0 is an instrumentation-only embedding concern.
        debug_assert!(self.frame.IsLocalFrameRoot());
        let Some(view) = self.frame.View().filter(|view| view.ShouldSetCursor()) else {
            return;
        };
        let Some(layout_view) = view.LayoutView() else {
            return;
        };
        self.frame
            .UpdateStyleAndLayout(CursorDocumentUpdateReason::kInput);
        let location =
            view.ViewportToFrame(self.mouse_event_manager.LastKnownMousePositionInViewport());
        let result = layout_view.HitTest(location.as_ref(), true, true);
        if let Some(frame) = result.InnerNodeFrame() {
            if let Some(cursor) = frame
                .EventHandlerCursor()
                .SelectCursor(location.as_ref(), result.as_ref())
            {
                view.SetCursor(cursor);
            }
        }
    }
}

impl EventHandlerCursorPolicy<'_> {
    // Source: event_handler.h:342; event_handler.cc:537-544.
    fn ShouldShowResizeForNode(
        &self,
        layout_object: &dyn CursorLayoutObject,
        location: &dyn CursorHitTestLocation,
    ) -> bool {
        let point = location.Point();
        layout_object
            .EnclosingLayer()
            .GetScrollableArea()
            .is_some_and(|area| {
                area.IsAbsolutePointInResizeControlForPointer(Point {
                    x: (point.x + 0.5).floor() as i32,
                    y: (point.y + 0.5).floor() as i32,
                })
            })
    }

    // Source: event_handler.h:340; event_handler.cc:546-560.
    fn IsSelectingLink(&self, result: &dyn CursorHitTestResult) -> bool {
        self.frame
            .UpdateStyleAndLayout(CursorDocumentUpdateReason::kSelection);
        let mouse_selection = !self.state.HasCapturingMouseEventsElement()
            && self.mouse_event_manager.MousePressed()
            && self.selection_controller.MouseDownMayStartSelect()
            && !self.mouse_event_manager.MouseDownMayStartDrag()
            && !self.frame.VisibleSelectionInDomTreeIsNone();
        mouse_selection && result.IsOverLink()
    }

    // Source: event_handler.h:343-344; event_handler.cc:573-844.
    pub fn SelectCursor(
        &self,
        location: &dyn CursorHitTestLocation,
        result: &dyn CursorHitTestResult,
    ) -> Option<Cursor> {
        if self.scroll_manager.InResizeMode() {
            return None;
        }
        let page = self.frame.Page()?;
        if self.scroll_manager.MiddleClickAutoscrollInProgress() {
            return None;
        }
        if result
            .GetScrollbar()
            .is_some_and(|scrollbar| !scrollbar.IsCustomScrollbar())
        {
            return Some(Cursor::Standard(CursorType::kPointer));
        }
        let node = result.InnerPossiblyPseudoNode();
        let Some(hit_node) = node else {
            return self.SelectAutoCursor(result, node, CursorType::kIBeam);
        };
        let mut hit_layout_object = hit_node.GetLayoutObject();
        let mut area_style = None;
        let area_image;
        if hit_node.IsHTMLAreaElement()
            && hit_layout_object.is_none()
            && self.frame.HTMLAreaElementDisplayNoneEnabled()
        {
            area_style = hit_node.GetComputedStyle();
            if area_style.is_some() {
                area_image = hit_node.ImageElement();
                hit_layout_object = area_image
                    .as_ref()
                    .and_then(|image| image.GetLayoutObject());
            }
        }
        let Some(layout_object) = hit_layout_object else {
            return self.SelectAutoCursor(result, node, CursorType::kIBeam);
        };
        if area_style.is_none() && self.ShouldShowResizeForNode(layout_object, location) {
            let box_ = layout_object.EnclosingLayer().GetLayoutBox();
            return Some(Cursor::Standard(match box_.StyleRef().UsedResize() {
                EResize::kVertical => CursorType::kNorthSouthResize,
                EResize::kHorizontal => CursorType::kEastWestResize,
                EResize::kBoth => {
                    if box_.ShouldPlaceBlockDirectionScrollbarOnLogicalLeft() {
                        CursorType::kSouthWestResize
                    } else {
                        CursorType::kSouthEastResize
                    }
                }
                _ => CursorType::kPointer,
            }));
        }
        match layout_object.GetCursor(result.LocalPoint()) {
            CursorDirective::kSetCursorBasedOnStyle => (),
            CursorDirective::kSetCursor(cursor) => return Some(cursor),
            CursorDirective::kDoNotSetCursor => return None,
        }
        let scrollbar_style = GetComputedStyleFromScrollbar(layout_object, result);
        let style = area_style
            .or(scrollbar_style)
            .unwrap_or_else(|| layout_object.StyleRef());
        if let Some(cursors) = style.Cursors() {
            for cursor in cursors {
                let Some(style_image) = cursor.GetImage().filter(|image| image.CanRender()) else {
                    continue;
                };
                let mut scale = style_image.ImageScaleFactor();
                let mut image = style_image
                    .CachedImage()
                    .expect("renderable cursor has CachedImage")
                    .GetImage();
                let device_scale_factor = page.DeviceScaleFactor(self.frame);
                let svg_image_holder;
                if image.IsSVGImage() {
                    scale *= device_scale_factor;
                    let default_size = page.SystemCursorSize(self.frame);
                    let Some(size_in_device_pixels) = style_image.ImageSize(
                        device_scale_factor,
                        SizeF {
                            width: default_size.width * device_scale_factor,
                            height: default_size.height * device_scale_factor,
                        },
                    ) else {
                        continue;
                    };
                    let Some(container_image) = image.SVGImageForContainer(
                        size_in_device_pixels,
                        device_scale_factor,
                        self.frame.ResolveColorSchemeForEmbedding(style),
                    ) else {
                        continue;
                    };
                    svg_image_holder = container_image;
                    image = svg_image_holder.as_ref();
                }
                let size = image.SizeRespectingOrientation();
                if size.width <= 0
                    || size.height <= 0
                    || !Cursor::AreDimensionsValidForWeb(size, scale)
                {
                    continue;
                }
                let specified_hot_spot = cursor.HotSpot();
                let hot_spot = Point {
                    // gfx::ScaleToRoundedPoint uses ClampRound (ties away
                    // from zero), unlike PhysicalOffset's LayoutUnit round.
                    x: (specified_hot_spot.x as f32 * scale).round() as i32,
                    y: (specified_hot_spot.y as f32 * scale).round() as i32,
                };
                let Some(bitmap) = image.AsBitmapForCurrentFrameRespectingOrientation() else {
                    continue;
                };
                let custom_cursor = Cursor::NewCustom(
                    bitmap,
                    DetermineHotSpot(image, cursor.HotSpotSpecified(), hot_spot),
                    scale,
                );
                let Cursor::Custom {
                    bitmap,
                    hot_spot,
                    image_scale_factor,
                } = &custom_cursor
                else {
                    unreachable!()
                };
                let bitmap_size = bitmap.Size();
                let inverse_scale = 1.0 / image_scale_factor;
                let custom_bitmap_size = SizeF {
                    width: bitmap_size.width as f32 * inverse_scale,
                    height: bitmap_size.height as f32 * inverse_scale,
                };
                if custom_bitmap_size.width > MAXIMUM_CURSOR_SIZE_WITHOUT_FALLBACK
                    || custom_bitmap_size.height > MAXIMUM_CURSOR_SIZE_WITHOUT_FALLBACK
                {
                    let ancestor_location =
                        self.frame.ContentLocalToRootAncestorPoint(location.Point());
                    let scale_factor = self.state.cursor_accessibility_scale_factor()
                        * if image.IsSVGImage() {
                            1.0
                        } else {
                            device_scale_factor
                        };
                    let scaled_hot_spot = PointF {
                        x: hot_spot.x as f32 * (scale_factor / image_scale_factor),
                        y: hot_spot.y as f32 * (scale_factor / image_scale_factor),
                    };
                    let cursor_rect = Rect {
                        origin: PointF {
                            x: ancestor_location.x - scaled_hot_spot.x.floor(),
                            y: ancestor_location.y - scaled_hot_spot.y.floor(),
                        },
                        size: SizeF {
                            width: (custom_bitmap_size.width * scale_factor).floor(),
                            height: (custom_bitmap_size.height * scale_factor).floor(),
                        },
                    };
                    let frame_rect = self.frame.MapContentRectToVisualRectInRootAncestorSpace(
                        page.VisibleContentRectExcludingScrollbars(),
                    );
                    if !frame_rect.Contains(&cursor_rect) {
                        continue;
                    }
                }
                return Some(custom_cursor);
            }
        }
        let i_beam = if style.IsHorizontalWritingMode() {
            CursorType::kIBeam
        } else {
            CursorType::kVerticalText
        };
        let cursor = style.Cursor();
        if cursor == ECursor::kAuto {
            return self.SelectAutoCursor(result, node, i_beam);
        }
        let selecting_link = cursor == ECursor::kPointer && self.IsSelectingLink(result);
        let selected = SelectStandardCursor(cursor, i_beam, selecting_link)
            .expect("auto cursor was handled by source SelectAutoCursor");
        Some(Cursor::Standard(selected))
    }

    // Source: event_handler.h:345-347; event_handler.cc:846-854.
    pub fn SelectAutoCursor(
        &self,
        result: &dyn CursorHitTestResult,
        node: Option<&dyn CursorNode>,
        i_beam: CursorType,
    ) -> Option<Cursor> {
        Some(SelectAutoCursorForNode(node, result.IsOverLink(), i_beam))
    }
}

// Source: event_handler.h:341; event_handler.cc:562-571,846-854.
// These source decisions consume only the hit node/link fact. Both the full
// EventHandler entry and the resident compatibility tail borrow this same
// implementation; no frame, manager state or hit result is fabricated.
pub fn SelectAutoCursorForNode(
    node: Option<&dyn CursorNode>,
    over_link: bool,
    i_beam: CursorType,
) -> Cursor {
    let show_i_beam = node.is_some_and(|node| {
        (node.IsTextNode() && (node.CanStartSelection() || over_link)) || node.IsEditable()
    });
    Cursor::Standard(if show_i_beam {
        i_beam
    } else {
        CursorType::kPointer
    })
}

// Source: event_handler.cc:757-843. Auto policy remains in SelectAutoCursor;
// the style switch is reused by the transitional adapter without duplicating it.
pub fn SelectStandardCursor(
    cursor: ECursor,
    i_beam: CursorType,
    selecting_link: bool,
) -> Option<CursorType> {
    Some(match cursor {
        ECursor::kAuto => return None,
        ECursor::kCrosshair => CursorType::kCross,
        ECursor::kPointer => {
            if selecting_link {
                i_beam
            } else {
                CursorType::kHand
            }
        }
        ECursor::kMove => CursorType::kMove,
        ECursor::kAllScroll => CursorType::kMove,
        ECursor::kEResize => CursorType::kEastResize,
        ECursor::kWResize => CursorType::kWestResize,
        ECursor::kNResize => CursorType::kNorthResize,
        ECursor::kSResize => CursorType::kSouthResize,
        ECursor::kNeResize => CursorType::kNorthEastResize,
        ECursor::kSwResize => CursorType::kSouthWestResize,
        ECursor::kNwResize => CursorType::kNorthWestResize,
        ECursor::kSeResize => CursorType::kSouthEastResize,
        ECursor::kNsResize => CursorType::kNorthSouthResize,
        ECursor::kEwResize => CursorType::kEastWestResize,
        ECursor::kNeswResize => CursorType::kNorthEastSouthWestResize,
        ECursor::kNwseResize => CursorType::kNorthWestSouthEastResize,
        ECursor::kColResize => CursorType::kColumnResize,
        ECursor::kRowResize => CursorType::kRowResize,
        ECursor::kText => i_beam,
        ECursor::kWait => CursorType::kWait,
        ECursor::kHelp => CursorType::kHelp,
        ECursor::kVerticalText => CursorType::kVerticalText,
        ECursor::kCell => CursorType::kCell,
        ECursor::kContextMenu => CursorType::kContextMenu,
        ECursor::kProgress => CursorType::kProgress,
        ECursor::kNoDrop => CursorType::kNoDrop,
        ECursor::kAlias => CursorType::kAlias,
        ECursor::kCopy => CursorType::kCopy,
        ECursor::kNone => CursorType::kNone,
        ECursor::kNotAllowed => CursorType::kNotAllowed,
        ECursor::kDefault => CursorType::kPointer,
        ECursor::kZoomIn => CursorType::kZoomIn,
        ECursor::kZoomOut => CursorType::kZoomOut,
        ECursor::kGrab => CursorType::kGrab,
        ECursor::kGrabbing => CursorType::kGrabbing,
    })
}

// The facade borrows the translated manager instances directly. These adapters
// project live source values and never retain their own cursor/input state.
impl CursorMouseEventManager for super::mouse_event_manager::MouseEventManager {
    fn LastKnownMousePositionInViewport(&self) -> PointF {
        let point =
            super::mouse_event_manager::MouseEventManagerState::LastKnownMousePositionInViewport(
                self,
            );
        PointF {
            x: point.x(),
            y: point.y(),
        }
    }
    fn MousePressed(&self) -> bool {
        super::mouse_event_manager::MouseEventManagerState::MousePressed(self)
    }
    fn MouseDownMayStartDrag(&self) -> bool {
        super::mouse_event_manager::MouseEventManagerState::MouseDownMayStartDrag(self)
    }
}
impl CursorScrollManager for interaction_scroll::core::input::scroll_manager::ScrollManager {
    fn InResizeMode(&self) -> bool {
        self.InResizeMode()
    }
    fn MiddleClickAutoscrollInProgress(&self) -> bool {
        self.MiddleClickAutoscrollInProgress()
    }
}
impl<S, H> CursorSelectionController
    for interaction_default::core::editing::selection_controller::SelectionController<'_, S, H>
where
    S: interaction_default::core::editing::selection_controller::FlatSelection,
{
    fn MouseDownMayStartSelect(&self) -> bool {
        self.MouseDownMayStartSelect()
    }
}
