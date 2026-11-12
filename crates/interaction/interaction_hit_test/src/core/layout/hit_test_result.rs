/*
 * Copyright (C) 2006, 2008, 2011 Apple Inc. All rights reserved.
 * Copyright (C) 2012 Nokia Corporation and/or its subsidiary(-ies)
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the GNU Library General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or (at your
 * option) any later version.
 * This library is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE. See the GNU Library General Public License
 * for more details. You should have received a copy along with this library;
 * see COPYING.LIB. If not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/layout/hit_test_result.h:1-264
// Source: third_party/blink/renderer/core/layout/hit_test_result.cc:1-721
// Rust modules replace source includes, export annotations and namespaces.

use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use foundation::{gfx, CompositorElementId, PhysicalOffset, PhysicalRect, TextDirection};

use super::geometry::physical_rect::PhysicalRectHitTestExt;
use super::hit_test_location::HitTestLocation;
use super::hit_test_request::{HitTestRequest, ListBasedHitTestBehavior};
use crate::ui::gfx::geometry::quad_f::QuadFHitTestExt;

// Source: hit_test_result.h:47-59,230-244
// Handles retain the existing DOM/resource identities; no tree, layout object,
// or media state is copied into the interaction package.
pub type NodeRef = Rc<dyn HitTestNode>;
pub type ScrollbarRef = Rc<dyn HitTestScrollbar>;
pub type ImageRef = Rc<dyn HitTestImage>;
pub type ImageContentRef = Rc<dyn HitTestImageContent>;
pub type FrameRef = Rc<dyn HitTestFrame>;
pub type Url = Option<Rc<dyn HitTestUrl>>;
pub type PositionWithAffinity = Option<Rc<dyn HitTestPosition>>;
pub trait HitTestScrollbar {
    /// The result roots the exact source Scrollbar selected by the native
    /// overflow-control query, just as NativeNodeOwner roots its DOM owner.
    fn NativeScrollbarOwner(
        &self,
    ) -> Option<&foundation::Persistent<layoutng_assembly::internal::scrollbar::Scrollbar>> {
        None
    }
}
pub trait HitTestImage {}
pub trait HitTestUrl {
    fn GetString(&self) -> &str;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTestPositionAnchor {
    OffsetInAnchor,
    BeforeAnchor,
    AfterAnchor,
    AfterChildren,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTestTextAffinity {
    Downstream,
    Upstream,
}
/// Semantic value projection of the original retained PositionWithAffinity.
/// This carries no layout geometry or alternate document/selection storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitTestPositionData {
    pub document: u128,
    pub node_id: Option<u64>,
    pub offset: usize,
    pub anchor: HitTestPositionAnchor,
    pub affinity: HitTestTextAffinity,
}
pub trait HitTestPosition {
    fn Value(&self) -> HitTestPositionData;
}
pub trait HitTestMediaStreamDescriptor {}
pub trait HitTestMediaSourceHandle {}

// Source: hit_test_result.cc:224-244,550-554
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PseudoKind {
    Before,
    Marker,
    After,
    CheckMark,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementKind {
    Html,
    Image,
    InputImage,
    Input,
    TextArea,
    Embed,
    Object,
    Area,
    Map,
    Media,
    Other,
}
impl ElementKind {
    pub fn IsHtml(self) -> bool {
        self != Self::Other
    }
    pub fn IsInput(self) -> bool {
        matches!(self, Self::Input | Self::InputImage)
    }
}

// Source: hit_test_result.cc:221-244,273-278
// These are requests to the existing editing owner, retaining its returned
// PositionWithAffinity rather than constructing a parallel editing model.
#[derive(Clone, Copy, Debug)]
pub enum CaretPositionQuery {
    NodeOffsetZeroDefaultAffinity,
    FirstMostForward,
    LastMostBackward,
    FirstInOrBefore,
}

// Source: hit_test_result.cc:203-280,282-301,316-434,705-718
// Each port projects only facts used by this source unit. Nodes and their
// document, layout, shadow and resource owners remain with their engines.
pub trait HitTestNode {
    /// Canonical DOM owner identity, independent of call-scoped Rust adapters.
    fn DOMIdentity(&self) -> u128 {
        self as *const Self as *const () as usize as u128
    }
    fn DOMNodeId(&self) -> Option<u64> {
        None
    }
    fn NativeNodeOwner(
        &self,
    ) -> Option<&foundation::Persistent<layoutng_assembly::internal::layout_node_metadata::Node>>
    {
        None
    }

    fn Element(&self) -> Option<&dyn HitTestElement>;
    fn Layout(&self) -> Option<&dyn HitTestLayout>;
    fn Document(&self) -> &dyn HitTestDocument;
    fn FlatTreeParent(&self) -> Option<NodeRef>;
    fn FlatTreeParentElement(&self) -> Option<NodeRef>;
    fn Pseudo(&self) -> Option<&dyn HitTestPseudoElement>;
    fn ContainingShadowRoot(&self) -> Option<Rc<dyn HitTestShadowRoot>>;
    fn OwnerShadowHost(&self) -> Option<NodeRef>;
    fn IsInShadowTree(&self) -> bool;
    fn IsEditable(&self) -> bool;
    fn CaretPosition(&self, query: CaretPositionQuery) -> PositionWithAffinity;
    fn ScrollableAreaIds(&self) -> Box<dyn Iterator<Item = CompositorElementId> + '_>;
}
pub trait HitTestPseudoElement {
    fn Kind(&self) -> PseudoKind;
    fn UltimateOriginatingElement(&self) -> NodeRef;
    fn InnerNodeForHitTesting(&self) -> NodeRef;
    fn SupportsHitTesting(&self) -> bool;
}
pub trait HitTestShadowRoot {
    fn IsUserAgent(&self) -> bool;
    fn Host(&self) -> NodeRef;
}
pub trait HitTestDocument {
    fn IsPrePaintClean(&self) -> bool;
    fn Frame(&self) -> Option<FrameRef>;
    fn CompleteURL(&self, value: &str) -> Url;
    fn DisallowInvalidation(&self) -> Box<dyn HitTestInvalidationGuard + '_>;
}
pub trait HitTestInvalidationGuard {}
pub trait HitTestFrame {
    fn SelectionContains(&self, point: &PhysicalOffset) -> bool;
}
pub trait HitTestElement {
    fn Kind(&self) -> ElementKind;
    fn DirectionalTitle(&self) -> Option<(String, TextDirection)>;
    fn Title(&self) -> Option<String>;
    fn Alt(&self) -> Option<String>;
    fn ImageSourceURL(&self) -> String;
    fn HrefURL(&self) -> Url;
    fn TextContent(&self) -> String;
    fn IsLiveLink(&self) -> bool;
    fn IsLink(&self) -> bool;
    fn IsDisabledOrReadOnly(&self) -> bool;
    fn IsTextField(&self) -> bool;
    fn ImageElement(&self) -> Option<NodeRef>;
    fn ImageMap(&self) -> Option<Rc<dyn HitTestImageMap>>;
    fn Media(&self) -> Option<Rc<dyn HitTestMedia>>;
}
pub trait HitTestImageMap {
    fn AreaForPoint(&self, point: &PhysicalOffset, layout: &dyn HitTestLayout) -> Option<NodeRef>;
}
pub trait HitTestImageContent {
    fn ErrorOccurred(&self) -> bool;
    fn HasImage(&self) -> bool;
    fn Image(&self) -> Option<ImageRef>;
    fn IsAutomaticUpgrade(&self) -> bool;
    fn Url(&self) -> Url;
}
pub enum HitTestMediaSrcObject {
    Stream(Option<Rc<dyn HitTestMediaStreamDescriptor>>),
    Source(Option<Rc<dyn HitTestMediaSourceHandle>>),
}
pub trait HitTestMedia {
    fn CurrentSrc(&self) -> Url;
    fn SrcObject(&self) -> HitTestMediaSrcObject;
}

// Source: hit_test_result.cc:699-700; layout_object.h:2237
// A synchronous geometry query returns only the owning Layout object's source
// UpdateHitTestResult output. It does not expose a mutable LayoutObject.
pub struct HitTestNodeAndPosition {
    pub node: NodeRef,
    pub point: PhysicalOffset,
}
pub trait HitTestLayout {
    fn IsImage(&self) -> bool;
    fn IsSVGImage(&self) -> bool;
    fn IsBox(&self) -> bool;
    fn IsMedia(&self) -> bool;
    fn LockedAncestorPreventingPaint(&self) -> bool;
    fn ChildPaintBlockedByDisplayLock(&self) -> bool;
    fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity;
    fn Direction(&self) -> TextDirection;
    fn AbsoluteContentQuad(&self) -> gfx::QuadF;
    fn ImageMapContainer(&self) -> foundation::native_path::ImageMapContainer;
    fn ImageContent(&self) -> Option<ImageContentRef>;
    fn UpdateHitTestResult(&self, point: PhysicalOffset) -> Option<HitTestNodeAndPosition>;
}
pub trait HitTestBoxFragment {
    fn OffsetFromOwnerLayoutBox(&self) -> PhysicalOffset;
}
pub trait HitTestRegion {
    fn Contains(&self, rect: &gfx::Rect) -> bool;
}

// Source: hit_test_result.h:65,257; hit_test_result.cc:570,666-684
// The source linked hash set's insertion order and identity uniqueness are
// preserved with a vector of shared owners plus an identity hash index.
#[derive(Clone, Default)]
pub struct NodeSet {
    nodes: Vec<NodeRef>,
    identities: HashSet<u128>,
}
impl NodeSet {
    fn insert(&mut self, node: NodeRef) {
        let identity = node.DOMIdentity();
        if self.identities.insert(identity) {
            self.nodes.push(node);
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &NodeRef> {
        self.nodes.iter()
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

pub type HitTestResultRequest<'a, L> = HitTestRequest<'a, L, dyn HitTestNode>;

// Source: hit_test_result.h:61-65,227-257
pub struct HitTestResult<'a, L: ?Sized> {
    hit_test_request_: HitTestResultRequest<'a, L>,
    cacheable_: bool,
    inner_node_: Option<NodeRef>,
    inner_element_: Option<NodeRef>,
    inner_possibly_pseudo_node_: Option<NodeRef>,
    point_in_inner_node_frame_: PhysicalOffset,
    local_point_: PhysicalOffset,
    inner_url_element_: Option<NodeRef>,
    scrollbar_: Option<ScrollbarRef>,
    is_over_embedded_content_view_: bool,
    is_over_resizer_: bool,
    is_over_scroll_corner_: bool,
    list_based_test_result_: RefCell<Option<NodeSet>>,
}

// Source: hit_test_result.cc:70-93
fn HasImageSourceURL(node: &dyn HitTestNode) -> bool {
    let kind = node.Element().map(HitTestElement::Kind);
    if matches!(kind, Some(ElementKind::Image | ElementKind::InputImage)) {
        return true;
    }
    let Some(layout) = node.Layout() else {
        return false;
    };
    if layout.IsImage() && matches!(kind, Some(ElementKind::Embed | ElementKind::Object)) {
        return true;
    }
    if layout.IsSVGImage() {
        return true;
    }
    false
}

// Source: hit_test_result.cc:97-100
impl<L: ?Sized> Default for HitTestResult<'_, L> {
    fn default() -> Self {
        Self {
            hit_test_request_: HitTestRequest::new(
                HitTestRequest::<L, dyn HitTestNode>::kReadOnly
                    | HitTestRequest::<L, dyn HitTestNode>::kActive,
                None,
                None,
            ),
            cacheable_: true,
            inner_node_: None,
            inner_element_: None,
            inner_possibly_pseudo_node_: None,
            point_in_inner_node_frame_: PhysicalOffset::default(),
            local_point_: PhysicalOffset::default(),
            inner_url_element_: None,
            scrollbar_: None,
            is_over_embedded_content_view_: false,
            is_over_resizer_: false,
            is_over_scroll_corner_: false,
            list_based_test_result_: RefCell::new(None),
        }
    }
}
fn SameNodeOwner(a: &Option<NodeRef>, b: &Option<NodeRef>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.DOMIdentity() == b.DOMIdentity(),
        (None, None) => true,
        _ => false,
    }
}

impl<'a, L: ?Sized> HitTestResult<'a, L> {
    // Source: hit_test_result.cc:102-107
    pub fn new(request: &HitTestResultRequest<'a, L>, location: &HitTestLocation) -> Self {
        Self {
            hit_test_request_: request.clone(),
            point_in_inner_node_frame_: *location.Point(),
            ..Self::default()
        }
    }
}
// Source: hit_test_result.cc:109-127
impl<L: ?Sized> Clone for HitTestResult<'_, L> {
    fn clone(&self) -> Self {
        Self {
            hit_test_request_: self.hit_test_request_.clone(),
            cacheable_: self.cacheable_,
            inner_node_: self.inner_node_.clone(),
            inner_element_: self.inner_element_.clone(),
            inner_possibly_pseudo_node_: self.inner_possibly_pseudo_node_.clone(),
            point_in_inner_node_frame_: self.point_in_inner_node_frame_,
            local_point_: self.local_point_,
            inner_url_element_: self.inner_url_element_.clone(),
            scrollbar_: self.scrollbar_.clone(),
            is_over_embedded_content_view_: self.is_over_embedded_content_view_,
            is_over_resizer_: self.is_over_resizer_,
            is_over_scroll_corner_: self.is_over_scroll_corner_,
            list_based_test_result_: RefCell::new(self.list_based_test_result_.borrow().clone()),
        }
    }
}
// Source: hit_test_result.cc:129,176-184
// Rc fields and the live request borrows express the source GC edges; Rust
// destruction releases the same retained identities.

fn SameOwner<T: ?Sized>(a: &Option<Rc<T>>, b: &Option<Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

impl<'a, L: ?Sized> HitTestResult<'a, L> {
    // Source: hit_test_result.cc:131-136
    pub fn Assign(&mut self, other: &Self) {
        self.hit_test_request_ = other.hit_test_request_.clone();
        self.PopulateFromCachedResult(other);
    }
    // Source: hit_test_result.cc:138-148
    pub fn EqualForCacheability(&self, other: &Self) -> bool {
        self.hit_test_request_
            .EqualForCacheability(&other.hit_test_request_)
            && SameNodeOwner(&self.inner_node_, &other.inner_node_)
            && SameNodeOwner(&self.inner_element_, &other.inner_element_)
            && SameNodeOwner(
                &self.inner_possibly_pseudo_node_,
                &other.inner_possibly_pseudo_node_,
            )
            && self.point_in_inner_node_frame_ == other.point_in_inner_node_frame_
            && self.local_point_ == other.local_point_
            && SameNodeOwner(&self.inner_url_element_, &other.inner_url_element_)
            && SameOwner(&self.scrollbar_, &other.scrollbar_)
            && self.is_over_embedded_content_view_ == other.is_over_embedded_content_view_
    }
    // Source: hit_test_result.cc:150-153
    pub fn CacheValues(&mut self, other: &HitTestResult<'_, L>) {
        self.hit_test_request_ = HitTestRequest::new(
            other.hit_test_request_.GetType() & !HitTestRequest::<L, dyn HitTestNode>::kAvoidCache,
            None,
            None,
        );
    }
    /// Assign the source cache entry after its CacheValues projection has
    /// removed request-scoped stop/callback borrows.
    pub fn AssignCachedValues(&mut self, other: &HitTestResult<'static, L>)
    where
        L: 'static,
    {
        self.CacheValues(other);
        self.PopulateFromCachedResult(other);
    }
    /// CacheValues removes the borrowed stop node and callback in Chromium.
    /// Retain only the canonical result owners so a LayoutView cache outlives
    /// an individual request without extending any Layout borrow.
    pub fn OwnedCachedValues(&self) -> HitTestResult<'static, L>
    where
        L: 'static,
    {
        let mut cached = HitTestResult::default();
        cached.PopulateFromCachedResult(self);
        cached.CacheValues(self);
        cached
    }
    // Source: hit_test_result.cc:155-174
    pub fn PopulateFromCachedResult(&mut self, other: &HitTestResult<'_, L>) {
        self.inner_node_ = other.inner_node_.clone();
        self.inner_element_ = other.inner_element_.clone();
        self.inner_possibly_pseudo_node_ = other.inner_possibly_pseudo_node_.clone();
        self.point_in_inner_node_frame_ = other.point_in_inner_node_frame_;
        self.local_point_ = other.local_point_;
        self.inner_url_element_ = other.inner_url_element_.clone();
        self.scrollbar_ = other.scrollbar_.clone();
        self.is_over_embedded_content_view_ = other.is_over_embedded_content_view_;
        self.cacheable_ = other.cacheable_;
        self.is_over_resizer_ = other.is_over_resizer_;
        self.is_over_scroll_corner_ = other.is_over_scroll_corner_;
        *self.list_based_test_result_.get_mut() = other.list_based_test_result_.borrow().clone();
    }
    // Source: hit_test_result.h:87-90,92,103-128,145,151-162,182-183,202-204
    pub fn InnerNode(&self) -> Option<&NodeRef> {
        self.inner_node_.as_ref()
    }
    pub fn InnerPossiblyPseudoNode(&self) -> Option<&NodeRef> {
        self.inner_possibly_pseudo_node_.as_ref()
    }
    pub fn InnerElement(&self) -> Option<&NodeRef> {
        self.inner_element_.as_ref()
    }
    pub fn URLElement(&self) -> Option<&NodeRef> {
        self.inner_url_element_.as_ref()
    }
    pub fn GetScrollbar(&self) -> Option<&ScrollbarRef> {
        self.scrollbar_.as_ref()
    }
    pub fn IsOverEmbeddedContentView(&self) -> bool {
        self.is_over_embedded_content_view_
    }
    pub fn PointInInnerNodeFrame(&self) -> &PhysicalOffset {
        &self.point_in_inner_node_frame_
    }
    pub fn SetPointInInnerNodeFrame(&mut self, point: PhysicalOffset) {
        self.point_in_inner_node_frame_ = point;
    }
    pub fn RoundedPointInInnerNodeFrame(&self) -> gfx::Point {
        foundation::ToRoundedPoint(&self.point_in_inner_node_frame_)
    }
    pub fn LocalPoint(&self) -> &PhysicalOffset {
        &self.local_point_
    }
    pub fn SetNodeAndPosition(&mut self, node: Option<NodeRef>, point: PhysicalOffset) {
        self.local_point_ = point;
        self.SetInnerNode(node);
    }
    pub fn GetHitTestRequest(&self) -> &HitTestResultRequest<'a, L> {
        &self.hit_test_request_
    }
    pub fn SetIsOverEmbeddedContentView(&mut self, value: bool) {
        self.is_over_embedded_content_view_ = value;
    }
    pub fn SetIsOverResizer(&mut self, value: bool) {
        self.is_over_resizer_ = value;
    }
    pub fn IsOverResizer(&self) -> bool {
        self.is_over_resizer_
    }
    pub fn SetIsOverScrollCorner(&mut self, value: bool) {
        self.is_over_scroll_corner_ = value;
    }
    pub fn IsOverScrollCorner(&self) -> bool {
        self.is_over_scroll_corner_
    }
    pub fn IsCacheable(&self) -> bool {
        self.cacheable_
    }
    pub fn SetCacheable(&mut self, value: bool) {
        self.cacheable_ = value;
    }
    pub fn HasListBasedResult(&self) -> bool {
        self.GetHitTestRequest().ListBased() && self.InnerNode().is_some()
    }

    // Source: hit_test_result.cc:186-195
    pub fn SetNodeAndFragmentPosition(
        &mut self,
        node: Option<NodeRef>,
        fragment: Option<&dyn HitTestBoxFragment>,
        position: PhysicalOffset,
    ) {
        self.local_point_ = fragment.map_or(position, |fragment| {
            position + fragment.OffsetFromOwnerLayoutBox()
        });
        self.SetInnerNode(node);
    }
    // Source: hit_test_result.cc:197-201
    pub fn OverrideNodeAndPosition(&mut self, node: Option<NodeRef>, position: PhysicalOffset) {
        self.local_point_ = position;
        self.SetInnerNode(node);
    }
    // Source: hit_test_result.cc:203-251
    pub fn GetPosition(&self) -> PositionWithAffinity {
        let node = self.inner_possibly_pseudo_node_.as_ref()?;
        debug_assert!(node.Document().IsPrePaintClean());
        let layout = node.Layout()?;
        assert!(!layout.LockedAncestorPreventingPaint());
        if layout.ChildPaintBlockedByDisplayLock() {
            return node.CaretPosition(CaretPositionQuery::NodeOffsetZeroDefaultAffinity);
        }
        if let Some(pseudo) = node.Pseudo() {
            match pseudo.Kind() {
                PseudoKind::Before | PseudoKind::Marker => {
                    return pseudo
                        .UltimateOriginatingElement()
                        .CaretPosition(CaretPositionQuery::FirstMostForward);
                }
                PseudoKind::After => {
                    return pseudo
                        .UltimateOriginatingElement()
                        .CaretPosition(CaretPositionQuery::LastMostBackward);
                }
                PseudoKind::CheckMark => {
                    return self
                        .inner_node_
                        .as_ref()
                        .expect("a pseudo has an originating node")
                        .CaretPosition(CaretPositionQuery::FirstMostForward);
                }
                PseudoKind::Other => {}
            }
        }
        layout.PositionForPoint(self.LocalPoint())
    }
    // Source: hit_test_result.cc:253-280
    pub fn GetPositionForInnerNodeOrImageMapImage(&self) -> PositionWithAffinity {
        let mut node = self.inner_possibly_pseudo_node_.clone();
        if node.as_ref().is_some_and(|node| node.Pseudo().is_none()) {
            node = self.InnerNodeOrImageMapImage();
        }
        let node = node?;
        debug_assert!(node.Document().IsPrePaintClean());
        let layout = node.Layout()?;
        assert!(!layout.LockedAncestorPreventingPaint());
        if layout.ChildPaintBlockedByDisplayLock() {
            return node.CaretPosition(CaretPositionQuery::NodeOffsetZeroDefaultAffinity);
        }
        let position = layout.PositionForPoint(self.LocalPoint());
        if position.is_none() {
            return node.CaretPosition(CaretPositionQuery::FirstInOrBefore);
        }
        position
    }
    // Source: hit_test_result.cc:282-302
    pub fn SetToShadowHostIfInUAShadowRoot(&mut self) {
        let Some(node) = self.inner_node_.clone() else {
            return;
        };
        let mut root = node.ContainingShadowRoot();
        let mut shadow_host = None;
        while let Some(containing_root) = root.filter(|root| root.IsUserAgent()) {
            let host = containing_root.Host();
            root = host.ContainingShadowRoot();
            shadow_host = Some(host);
            self.OverrideNodeAndPosition(node.OwnerShadowHost(), self.local_point_);
        }
        if shadow_host.is_some() {
            self.OverrideNodeAndPosition(shadow_host, self.local_point_);
        }
    }
    // Source: hit_test_result.cc:304-314
    pub fn GetScrollableContainer(&self) -> CompositorElementId {
        self.inner_node_
            .as_ref()
            .and_then(|node| node.ScrollableAreaIds().next())
            .unwrap_or_default()
    }
    // Source: hit_test_result.cc:316-333
    pub fn ImageAreaForImage(&self) -> Option<NodeRef> {
        let inner = self
            .inner_node_
            .as_ref()
            .expect("ImageAreaForImage requires an inner node");
        let mut image = inner
            .Element()
            .filter(|element| element.Kind() == ElementKind::Image)
            .map(|_| inner.clone());
        if image.is_none() && inner.IsInShadowTree() {
            if inner
                .ContainingShadowRoot()
                .is_some_and(|root| root.IsUserAgent())
            {
                image = inner.OwnerShadowHost().filter(|host| {
                    host.Element()
                        .is_some_and(|element| element.Kind() == ElementKind::Image)
                });
            }
        }
        let image = image?;
        let layout = image.Layout().filter(|layout| layout.IsBox())?;
        image
            .Element()?
            .ImageMap()?
            .AreaForPoint(self.LocalPoint(), layout)
    }
    // Source: hit_test_result.cc:335-362
    pub fn SetInnerNode(&mut self, node: Option<NodeRef>) {
        let Some(mut node) = node else {
            self.inner_possibly_pseudo_node_ = None;
            self.inner_node_ = None;
            self.inner_element_ = None;
            return;
        };
        self.inner_possibly_pseudo_node_ = Some(node.clone());
        if let Some(pseudo) = node.Pseudo() {
            node = pseudo.InnerNodeForHitTesting();
        }
        self.inner_node_ = Some(node);
        if let Some(area) = self.ImageAreaForImage() {
            self.inner_node_ = Some(area.clone());
            self.inner_possibly_pseudo_node_ = Some(area);
        }
        let node = self.inner_node_.as_ref().unwrap();
        self.inner_element_ = if node.Element().is_some() {
            Some(node.clone())
        } else {
            node.FlatTreeParentElement()
        };
    }
    // Source: hit_test_result.cc:364-371
    pub fn InnerPossiblyPseudoElement(&self) -> Option<&NodeRef> {
        if let Some(node) = &self.inner_possibly_pseudo_node_ {
            if node
                .Pseudo()
                .is_some_and(|pseudo| pseudo.SupportsHitTesting())
            {
                return Some(node);
            }
        }
        self.inner_element_.as_ref()
    }
    // Source: hit_test_result.cc:373-379
    pub fn SetURLElement(&mut self, node: Option<NodeRef>) {
        debug_assert!(node.as_ref().is_none_or(|node| node.Element().is_some()));
        self.inner_url_element_ = node;
    }
    pub fn SetScrollbar(&mut self, scrollbar: Option<ScrollbarRef>) {
        self.scrollbar_ = scrollbar;
    }
    // Source: hit_test_result.cc:381-385
    pub fn InnerNodeFrame(&self) -> Option<FrameRef> {
        self.inner_node_.as_ref()?.Document().Frame()
    }
    // Source: hit_test_result.cc:387-394
    pub fn IsSelected(&self, location: &HitTestLocation) -> bool {
        self.InnerNodeFrame()
            .is_some_and(|frame| frame.SelectionContains(location.Point()))
    }
    // Source: hit_test_result.cc:396-421
    pub fn Title(&self, direction: &mut TextDirection) -> Option<String> {
        *direction = TextDirection::kLtr;
        let mut title_node = self.inner_node_.clone();
        while let Some(node) = title_node {
            if let Some(element) = node.Element() {
                if element.Kind().IsHtml() {
                    if let Some((title, title_direction)) = element.DirectionalTitle() {
                        *direction = title_direction;
                        return Some(title);
                    }
                } else if let Some(title) = element.Title() {
                    if let Some(layout) = node.Layout() {
                        *direction = layout.Direction();
                    }
                    return Some(title);
                }
            }
            title_node = node.FlatTreeParent();
        }
        None
    }
    // Source: hit_test_result.cc:423-435
    pub fn AltDisplayString(&self) -> Option<String> {
        let node = self.InnerNodeOrImageMapImage()?;
        let element = node.Element()?;
        if element.Kind() == ElementKind::Image || element.Kind().IsInput() {
            return element.Alt();
        }
        None
    }
    // Source: hit_test_result.cc:437-448
    pub fn GetImage(&self) -> Option<ImageRef> {
        Self::GetImageForNode(self.InnerNodeOrImageMapImage().as_deref())
    }
    pub fn GetImageForNode(node: Option<&dyn HitTestNode>) -> Option<ImageRef> {
        let content = Self::GetImageContent(node)?;
        if !content.ErrorOccurred() && content.HasImage() {
            return content.Image();
        }
        None
    }
    // Source: hit_test_result.cc:450-457
    pub fn ImageRect(&self) -> gfx::Rect {
        if self.GetImage().is_none() {
            return gfx::Rect::default();
        }
        let node = self
            .InnerNodeOrImageMapImage()
            .expect("a loaded image has an image node");
        gfx::ToEnclosingRect(
            node.Layout()
                .expect("a loaded image has layout")
                .AbsoluteContentQuad()
                .BoundingBox(),
        )
    }
    // Source: hit_test_result.cc:459-478
    pub fn AbsoluteImageURLForNode(node: Option<&dyn HitTestNode>) -> Url {
        let node = node.filter(|node| HasImageSourceURL(*node))?;
        if let Some(content) = Self::GetImageContent(Some(node)) {
            if content.IsAutomaticUpgrade() {
                return content.Url();
            }
        }
        let value = node
            .Element()
            .expect("an image source belongs to an element")
            .ImageSourceURL();
        if value.is_empty() {
            return None;
        }
        node.Document().CompleteURL(
            value.trim_matches(|c| {
                matches!(c, '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | ' ')
            }),
        )
    }
    pub fn AbsoluteImageURL(&self) -> Url {
        Self::AbsoluteImageURLForNode(self.InnerNodeOrImageMapImage().as_deref())
    }
    // Source: hit_test_result.cc:480-484
    pub fn AbsoluteMediaURL(&self) -> Url {
        self.MediaElement()?.CurrentSrc()
    }
    // Source: hit_test_result.cc:486-508
    pub fn GetMediaStreamDescriptor(&self) -> Option<Rc<dyn HitTestMediaStreamDescriptor>> {
        match self.MediaElement()?.SrcObject() {
            HitTestMediaSrcObject::Stream(stream) => stream,
            _ => None,
        }
    }
    pub fn GetMediaSourceHandle(&self) -> Option<Rc<dyn HitTestMediaSourceHandle>> {
        match self.MediaElement()?.SrcObject() {
            HitTestMediaSrcObject::Source(source) => source,
            _ => None,
        }
    }
    // Source: hit_test_result.cc:510-519
    fn MediaElement(&self) -> Option<Rc<dyn HitTestMedia>> {
        let node = self.inner_node_.as_ref()?;
        if !node.Layout().is_some_and(|layout| layout.IsMedia()) {
            return None;
        }
        node.Element()?.Media()
    }
    // Source: hit_test_result.cc:521-539
    pub fn AbsoluteLinkURL(&self) -> Url {
        self.inner_url_element_.as_ref()?.Element()?.HrefURL()
    }
    pub fn IsLiveLink(&self) -> bool {
        self.inner_url_element_
            .as_ref()
            .and_then(|node| node.Element())
            .is_some_and(HitTestElement::IsLiveLink)
    }
    pub fn IsOverLink(&self) -> bool {
        self.inner_url_element_
            .as_ref()
            .and_then(|node| node.Element())
            .is_some_and(HitTestElement::IsLink)
    }
    pub fn TextContent(&self) -> String {
        self.inner_url_element_
            .as_ref()
            .and_then(|node| node.Element())
            .map_or_else(String::new, HitTestElement::TextContent)
    }
    // Source: hit_test_result.cc:541-557
    pub fn IsContentEditable(&self) -> bool {
        let Some(node) = self.inner_node_.as_ref() else {
            return false;
        };
        if let Some(element) = node.Element() {
            if element.Kind() == ElementKind::TextArea {
                return !element.IsDisabledOrReadOnly();
            }
            if element.Kind().IsInput() {
                return !element.IsDisabledOrReadOnly() && element.IsTextField();
            }
        }
        node.IsEditable()
    }
    // Source: hit_test_result.cc:559-583
    fn AddNodeToListBasedTestResultInternal(
        &mut self,
        node: Option<NodeRef>,
        _location: &HitTestLocation,
    ) -> (bool, ListBasedHitTestBehavior) {
        use ListBasedHitTestBehavior::*;
        if !self.GetHitTestRequest().ListBased() {
            return (false, kStopHitTesting);
        }
        let Some(node) = node else {
            return (false, kContinueHitTesting);
        };
        self.MutableListBasedTestResult().insert(node.clone());
        if self.GetHitTestRequest().PenetratingList() {
            let mut behavior = kContinueHitTesting;
            if self.GetHitTestRequest().UseHitNodeCb() {
                let _invalidation_disallowed = node.Document().DisallowInvalidation();
                behavior = self.GetHitTestRequest().RunHitNodeCb(node.as_ref());
            }
            return (false, behavior);
        }
        (true, kContinueHitTesting)
    }
    // Source: hit_test_result.cc:585-602
    fn GetImageContent(node: Option<&dyn HitTestNode>) -> Option<ImageContentRef> {
        let layout = node?.Layout()?;
        if layout.IsImage() || layout.IsSVGImage() {
            return layout.ImageContent();
        }
        None
    }
    // Source: hit_test_result.cc:604-616
    pub fn AddNodeToListBasedTestResult(
        &mut self,
        node: Option<NodeRef>,
        location: &HitTestLocation,
        rect: &PhysicalRect,
    ) -> ListBasedHitTestBehavior {
        let (check_containment, behavior) =
            self.AddNodeToListBasedTestResultInternal(node, location);
        if !check_containment {
            return behavior;
        }
        if rect.Contains(location.BoundingBox()) {
            ListBasedHitTestBehavior::kStopHitTesting
        } else {
            ListBasedHitTestBehavior::kContinueHitTesting
        }
    }
    // Source: hit_test_result.cc:618-631
    pub fn AddNodeToListBasedTestResultQuad(
        &mut self,
        node: Option<NodeRef>,
        location: &HitTestLocation,
        quad: &gfx::QuadF,
    ) -> ListBasedHitTestBehavior {
        let (check_containment, behavior) =
            self.AddNodeToListBasedTestResultInternal(node, location);
        if !check_containment {
            return behavior;
        }
        if quad.ContainsQuad(&gfx::QuadF::from(gfx::RectF::from(*location.BoundingBox()))) {
            ListBasedHitTestBehavior::kStopHitTesting
        } else {
            ListBasedHitTestBehavior::kContinueHitTesting
        }
    }
    // Source: hit_test_result.cc:633-645
    pub fn AddNodeToListBasedTestResultRegion(
        &mut self,
        node: Option<NodeRef>,
        location: &HitTestLocation,
        region: &dyn HitTestRegion,
    ) -> ListBasedHitTestBehavior {
        let (check_containment, behavior) =
            self.AddNodeToListBasedTestResultInternal(node, location);
        if !check_containment {
            return behavior;
        }
        if region.Contains(&location.ToEnclosingRect()) {
            ListBasedHitTestBehavior::kStopHitTesting
        } else {
            ListBasedHitTestBehavior::kContinueHitTesting
        }
    }
    // Source: hit_test_result.cc:647-673
    pub fn Append(&mut self, other: &Self) {
        debug_assert!(self.GetHitTestRequest().ListBased());
        if self.scrollbar_.is_none() && other.scrollbar_.is_some() {
            self.SetScrollbar(other.scrollbar_.clone());
        }
        if self.inner_node_.is_none() && other.inner_node_.is_some() {
            self.inner_node_ = other.inner_node_.clone();
            self.inner_element_ = other.inner_element_.clone();
            self.inner_possibly_pseudo_node_ = other.inner_possibly_pseudo_node_.clone();
            self.local_point_ = other.local_point_;
            self.point_in_inner_node_frame_ = other.point_in_inner_node_frame_;
            self.inner_url_element_ = other.inner_url_element_.clone();
            self.is_over_embedded_content_view_ = other.is_over_embedded_content_view_;
            self.is_over_resizer_ = other.is_over_resizer_;
            self.is_over_scroll_corner_ = other.is_over_scroll_corner_;
        }
        if let Some(nodes) = other.list_based_test_result_.borrow().as_ref() {
            let set = self.MutableListBasedTestResult();
            for node in nodes.iter() {
                set.insert(node.clone());
            }
        }
    }
    // Source: hit_test_result.cc:675-685
    pub fn ListBasedTestResult(&self) -> Ref<'_, NodeSet> {
        if self.list_based_test_result_.borrow().is_none() {
            *self.list_based_test_result_.borrow_mut() = Some(NodeSet::default());
        }
        Ref::map(self.list_based_test_result_.borrow(), |set| {
            set.as_ref().unwrap()
        })
    }
    fn MutableListBasedTestResult(&mut self) -> &mut NodeSet {
        self.list_based_test_result_
            .get_mut()
            .get_or_insert_with(NodeSet::default)
    }
    // Source: hit_test_result.cc:687-703
    pub fn ResolveRectBasedTest(
        &mut self,
        resolved_node: &NodeRef,
        resolved_point_in_main_frame: PhysicalOffset,
    ) -> HitTestLocation {
        self.point_in_inner_node_frame_ = resolved_point_in_main_frame;
        self.SetInnerNode(None);
        *self.list_based_test_result_.get_mut() = None;
        if let Some(layout) = resolved_node.Layout() {
            if let Some(update) = layout.UpdateHitTestResult(PhysicalOffset::default()) {
                self.SetNodeAndPosition(Some(update.node), update.point);
            }
        }
        HitTestLocation::FromPhysicalOffset(resolved_point_in_main_frame)
    }
    // Source: hit_test_result.cc:705-719
    pub fn InnerNodeOrImageMapImage(&self) -> Option<NodeRef> {
        let node = self.inner_node_.as_ref()?;
        if let Some(element) = node.Element() {
            if matches!(element.Kind(), ElementKind::Area | ElementKind::Map) {
                if let Some(image) = element.ImageElement() {
                    return Some(image);
                }
            }
        }
        Some(node.clone())
    }
}
