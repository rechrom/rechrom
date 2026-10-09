/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003-2011 Apple Inc. All rights reserved.
 * GNU Library General Public License version 2 or later; see COPYING.LIB.
 */
// cpp: third_party/blink/renderer/core/css/resolver/style_resolver_state.h
// cpp: third_party/blink/renderer/core/css/resolver/style_resolver_state.cc
// Existing ComputedStyle/ComputedStyleBuilder and CSSValue are used directly.
// Persistent roots preserve the identity/lifetime of existing GC style objects.
// This source has no MatchResult field: callers use resolver::match_result.
// Source ledger (Chromium commit 6c1d401fcca5e1b0030563a90c2f2bba168e0c15):
// /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
// style_resolver_state.h: physical=412 effective=166 mapped=157 omitted=9 pending=0.
// style_resolver_state.cc: physical=570 effective=331 mapped=322 omitted=9 pending=0.
// Effective counts exclude copyright/comments, blanks, preprocessing/includes,
// namespaces and lines consisting only of braces/parentheses/semicolons.
// Header omitted: forward declarations 43-46, STACK_ALLOCATED 52, access labels
// 54/289 and deleted copy boilerplate 59-60 (the Rust state has no Clone).
// Implementation omitted: DCHECK 121/124/138 and metrics-only blocks 332-334,
// 362-367. The three metrics blocks' closing braces are excluded above.
// All remaining effective declarations/logic in h:51-407 and cc:45-568 map
// below. Required traits only provide real DOM/font/animation/resources/value
// and length-conversion collaborators; the state's branches and bookkeeping
// stay here. No production logic remains pending for this source pair.

#![allow(non_snake_case, non_camel_case_types)]

use super::style_resolver::{StyleResolver, StyleResolverBackend};
use crate::css_value::{CSSValue, CSSValueDispatch, CSSValuePayload};
use crate::parser::css_parser_mode::CSSParserMode;
use crate::properties::css_property::{CSSProperty, Flags as PropertyFlags};
use foundation::{
    gfx, AtomicString, CSSPropertyID, CSSValueID, EDisplay, EInsideLink, ETextOrientation,
    Persistent, WritingDirectionMode, WritingMode,
};
use layoutng_style::style::color_scheme::mojom::blink::ColorScheme;
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::computed_style_base::IsAtShadowBoundary;
use layoutng_style::style::computed_style_constants::{IsHighlightPseudoElement, PseudoId};
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues;
use layoutng_style::style::default_anchor_data::DefaultAnchorData;
use layoutng_style::style::display_style::DisplayStyle;
use layoutng_style::style::font_size_style::FontSizeStyle;
use layoutng_style::style::position_area::{PositionArea, PositionAreaOffsets};
use layoutng_style::style::style_image::StyleImage;
use layoutng_style::style::style_position_anchor::StylePositionAnchor;
use layoutng_style::style::text_size_adjust::TextSizeAdjust;
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
use std::rc::Rc;

pub type ComputedStyleHandle = Persistent<ComputedStyle>;
pub type ResolverValue<B> = CSSValue<<B as StyleResolverStateBackend>::ValueDispatch>;
pub type ResolverURIValue<B> =
    <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSURIValue;

// Required StyleRequest::RequestType dependency, style_request.h:42.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleResolverRequestType {
    kForRenderer,
    kForComputedStyle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueResolutionMode {
    Const,
    Mutable,
}

/// FontBuilder's actual API consumed by this source pair. The existing
/// font_builder module currently exposes only PropertySetFlag; its eventual
/// FontBuilder implements this interface, rather than a second font model.
pub trait StyleResolverStateFontBuilder {
    fn CreateFont(&mut self, builder: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>);
    fn DidChangeEffectiveZoom(&mut self);
    fn DidChangeWritingMode(&mut self);
    fn DidChangeTextSizeAdjust(&mut self);
    fn DidChangeTextOrientation(&mut self);
}

/// Actual CSSToLengthConversionData mutators. Flags are shared with the state,
/// so conversions and copies OR their observed units into the same flag cell.
pub trait ResolverLengthConversionData<B: StyleResolverStateBackend> {
    fn SetFontSizes(&mut self, sizes: B::FontSizes);
    fn SetZoom(&mut self, zoom: f32);
    fn SetLineHeightSize(&mut self, size: B::LineHeightSize);
    fn SetAnchorData(&mut self, data: B::AnchorData);
    fn SubtractScrollbars(&mut self, scrollbars: &gfx::Size);
}

/// All entries are calls owned by other source classes. No default result or
/// fallback DOM, builder, CSSValue, animation, conversion or font is supplied.
/// Builder operations and state branches are implemented below, using the real
/// layoutng_style types. Inherited StyleResolverBackend builder/style methods
/// likewise refer to those exact types, with Persistent owning their outputs.
pub trait StyleResolverStateBackend:
    StyleResolverBackend<
        ComputedStyle = ComputedStyleHandle,
        ComputedStyleBuilder = ComputedStyleBuilder,
    > + Sized
{
    type Element;
    type PseudoElement;
    type ElementResolveContext;
    type StyleRecalcContext;
    type StyleRequest;
    type AnchorEvaluator;
    type AnimationUpdate;
    type FontBuilder: StyleResolverStateFontBuilder;
    type ElementStyleResources;
    type SVGResource;
    type ValueDispatch: CSSValueDispatch<Document = Self::Document>;
    type LengthConversionData: ResolverLengthConversionData<Self>;
    type FontSizes;
    type LineHeightSize;
    type ViewportSize;
    type ContainerSizes;
    type AnchorData;

    fn DefaultStyleRequest(&self) -> Self::StyleRequest;
    fn RequestParentOverride(&self, request: &Self::StyleRequest) -> Option<ComputedStyleHandle>;
    fn RequestLayoutParentOverride(
        &self,
        request: &Self::StyleRequest,
    ) -> Option<ComputedStyleHandle>;
    fn RequestOriginatingElementStyle(
        &self,
        request: &Self::StyleRequest,
    ) -> Option<ComputedStyleHandle>;
    fn RequestStyledElement(&self, request: &Self::StyleRequest) -> Option<Rc<Self::Element>>;
    fn RequestPseudoId(&self, request: &Self::StyleRequest) -> PseudoId;
    fn RequestPseudoArgument(&self, request: &Self::StyleRequest) -> AtomicString;
    fn RequestType(&self, request: &Self::StyleRequest) -> StyleResolverRequestType;
    fn RequestCanTriggerAnimations(&self, request: &Self::StyleRequest) -> bool;
    fn RecalcOldStyle(&self, context: &Self::StyleRecalcContext) -> Option<ComputedStyleHandle>;
    fn RecalcSizeContainer(&self, context: &Self::StyleRecalcContext) -> Option<Rc<Self::Element>>;
    fn RecalcAnchorEvaluator(
        &self,
        context: &Self::StyleRecalcContext,
    ) -> Option<Rc<Self::AnchorEvaluator>>;
    fn NewElementResolveContext(&self, element: Rc<Self::Element>) -> Self::ElementResolveContext;
    fn ContextElement(&self, context: &Self::ElementResolveContext) -> Rc<Self::Element>;
    fn ContextUltimateOriginatingElementOrSelf(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Rc<Self::Element>;
    fn ContextParentElement(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<Rc<Self::Element>>;
    fn ContextRootElementStyle(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<ComputedStyleHandle>;
    fn ContextParentStyle(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<ComputedStyleHandle>;
    fn ContextLayoutParentStyle(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<ComputedStyleHandle>;
    fn ContextElementLinkState(&self, context: &Self::ElementResolveContext) -> EInsideLink;
    fn ContextPseudoElement(
        &self,
        context: &Self::ElementResolveContext,
    ) -> Option<Rc<Self::PseudoElement>>;
    fn GetStyledPseudoElement(
        &self,
        element: &Rc<Self::Element>,
        id: PseudoId,
        argument: &AtomicString,
    ) -> Option<Rc<Self::Element>>;
    fn ElementAsPseudoElement(
        &self,
        element: &Rc<Self::Element>,
    ) -> Option<Rc<Self::PseudoElement>>;
    fn PseudoElementAsElement(&self, element: Rc<Self::PseudoElement>) -> Rc<Self::Element>;
    fn ElementPseudoIdForStyling(&self, element: &Self::Element) -> PseudoId;
    fn ElementIsLink(&self, element: &Self::Element) -> bool;
    fn ElementIsInCanvasSubtree(&self, element: &Self::Element) -> bool;
    fn FlatTreeParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn LayoutObjectIsNeeded(&self, element: &Self::Element, display: DisplayStyle<'_>) -> bool;
    fn IsAnimatingDisplayProperty(&self, element: &Self::Element) -> bool;
    fn DocumentDevicePixelRatio(&self, document: &Self::Document) -> f32;
    fn DocumentInQuirksMode(&self, document: &Self::Document) -> bool;
    fn DocumentElementComputedStyle(
        &self,
        document: &Self::Document,
    ) -> Option<ComputedStyleHandle>;
    fn DocumentViewportSize(&self, document: &Self::Document) -> Self::ViewportSize;
    fn ViewportSizeFromDocumentLayoutView(&self, document: &Self::Document) -> Self::ViewportSize;
    fn NewAnimationUpdate(&self) -> Self::AnimationUpdate;
    fn ClearAnimationUpdate(&self, update: &mut Self::AnimationUpdate);
    fn NewFontBuilder(&self, document: &Self::Document) -> Self::FontBuilder;
    fn FontDescriptionFromStyle(&self, style: &ComputedStyle) -> Self::FontDescription;
    fn NewElementStyleResources(
        &self,
        element: &Self::Element,
        device_pixel_ratio: f32,
    ) -> Self::ElementStyleResources;
    fn ResourcesUpdateLengthConversionData(
        &self,
        resources: &mut Self::ElementStyleResources,
        data: &Rc<RefCell<Self::LengthConversionData>>,
    );
    fn ResourcesLoadPendingResources(
        &self,
        resources: &mut Self::ElementStyleResources,
        builder: &mut ComputedStyleBuilder,
        data: &Self::LengthConversionData,
    );
    fn ResourcesGetStyleImage(
        &self,
        resources: &mut Self::ElementStyleResources,
        property: CSSPropertyID,
        value: Rc<ResolverValue<Self>>,
    ) -> Option<Rc<StyleImage>>;
    fn ResourcesGetSVGResource(
        &self,
        resources: &mut Self::ElementStyleResources,
        property: CSSPropertyID,
        value: &ResolverURIValue<Self>,
    ) -> Option<Rc<Self::SVGResource>>;
    fn URIIsLocal(&self, value: &ResolverURIValue<Self>, document: &Self::Document) -> bool;
    fn NewElementLengthConversionData(&self, element: &Self::Element)
        -> Self::LengthConversionData;
    fn NewFontSizes(
        &self,
        font: FontSizeStyle<'_>,
        root_style: Option<&ComputedStyle>,
    ) -> Self::FontSizes;
    fn NewLineHeightSize(
        &self,
        font: FontSizeStyle<'_>,
        root_style: Option<&ComputedStyle>,
    ) -> Self::LineHeightSize;
    fn NewContainerSizes(&self, element: Option<Rc<Self::Element>>) -> Self::ContainerSizes;
    fn NewAnchorData(
        &self,
        evaluator: Option<Rc<Self::AnchorEvaluator>>,
        default: DefaultAnchorData,
        offsets: Option<PositionAreaOffsets>,
    ) -> Self::AnchorData;
    fn NewBuilderLengthConversionData(
        &self,
        builder: &ComputedStyleBuilder,
        parent: Option<&ComputedStyle>,
        root: Option<&ComputedStyle>,
        viewport: Self::ViewportSize,
        containers: Self::ContainerSizes,
        anchors: Self::AnchorData,
        zoom: f32,
        flags: Rc<Cell<u32>>,
        element: Rc<Self::Element>,
    ) -> Self::LengthConversionData;
    fn NewUnzoomedLengthConversionData(
        &self,
        writing_mode: WritingMode,
        fonts: Self::FontSizes,
        line_height: Self::LineHeightSize,
        viewport: Self::ViewportSize,
        containers: Self::ContainerSizes,
        anchors: Self::AnchorData,
        zoom: f32,
        flags: Rc<Cell<u32>>,
        element: Rc<Self::Element>,
    ) -> Self::LengthConversionData;
    fn ComputePositionAreaOffsetsForLayout(
        &self,
        evaluator: &Self::AnchorEvaluator,
        default: DefaultAnchorData,
    ) -> Option<PositionAreaOffsets>;
    fn AnchorContainerWritingDirection(
        &self,
        evaluator: &Self::AnchorEvaluator,
    ) -> WritingDirectionMode;
    fn LightDarkFirst(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSLightDarkValuePair,
    ) -> Rc<ResolverValue<Self>>;
    fn LightDarkSecond(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSLightDarkValuePair,
    ) -> Rc<ResolverValue<Self>>;
    fn IdentifierValueID(
        &self,
        value: &<Self::ValueDispatch as CSSValueDispatch>::CSSIdentifierValue,
    ) -> CSSValueID;
    fn CreateIdentifierValue(&self, id: CSSValueID) -> Rc<ResolverValue<Self>>;
    fn NewColorImageValue(&self, color: Rc<ResolverValue<Self>>) -> Rc<ResolverValue<Self>>;
    /// Storage only: this cell is shared per value-dispatch/thread. The Rust
    /// TransparentImage helper owns lazy initialization and singleton reuse.
    fn TransparentImageCache(&self) -> &OnceCell<Rc<ResolverValue<Self>>>;
    fn ResolveGradientValuesIfNeeded(
        &self,
        value: Rc<ResolverValue<Self>>,
        state: &StyleResolverState<'_, Self>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<Self>>;
    fn ResolveImageSetValuesIfNeeded(
        &self,
        value: Rc<ResolverValue<Self>>,
        state: &StyleResolverState<'_, Self>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<Self>>;
    fn ResolveCrossfadeValuesIfNeeded(
        &self,
        value: Rc<ResolverValue<Self>>,
        state: &StyleResolverState<'_, Self>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<Self>>;
}

fn StyleRef(style: &ComputedStyleHandle) -> &ComputedStyle {
    let pointer = style.Get();
    assert!(
        !pointer.is_null(),
        "computed style handle must retain a style"
    );
    // Persistent checks that this is a live managed address and roots it for
    // the entire borrow; cloning a handle retains the same style allocation.
    unsafe { &*pointer }
}

// cpp: style_resolver_state.cc:45-78
fn MayReturnNullRenderingStyleForPseudoElement(
    id: PseudoId,
    parent: Option<&ComputedStyle>,
) -> bool {
    if id == PseudoId::kPseudoIdColumn {
        return false;
    }
    if id == PseudoId::kPseudoIdScrollMarkerGroup {
        return parent
            .expect("scroll-marker-group requires parent style")
            .ScrollMarkerGroupNone();
    }
    true
}

// cpp: style_resolver_state.h:324-407
pub struct StyleResolverState<'a, B: StyleResolverStateBackend> {
    backend_: &'a B,
    element_context_: B::ElementResolveContext,
    style_recalc_context_: Option<&'a B::StyleRecalcContext>,
    document_: &'a B::Document,
    style_builder_: RefCell<Option<ComputedStyleBuilder>>,
    length_conversion_flags_: Rc<Cell<u32>>,
    css_to_length_conversion_data_dirty_: Cell<bool>,
    should_update_line_height_: Cell<bool>,
    css_to_length_conversion_data_: Rc<RefCell<B::LengthConversionData>>,
    parent_style_: RefCell<Option<ComputedStyleHandle>>,
    layout_parent_style_: RefCell<Option<ComputedStyleHandle>>,
    old_style_: RefCell<Option<ComputedStyleHandle>>,
    animation_update_: RefCell<B::AnimationUpdate>,
    pseudo_request_type_: StyleResolverRequestType,
    font_builder_: RefCell<B::FontBuilder>,
    styled_element_: Option<Rc<B::Element>>,
    element_style_resources_: RefCell<B::ElementStyleResources>,
    pseudo_id_: PseudoId,
    inside_link_: Cell<Option<EInsideLink>>,
    originating_element_style_: Option<ComputedStyleHandle>,
    is_for_highlight_: bool,
    can_trigger_animations_: bool,
    had_no_matched_properties_: Cell<bool>,
    conditionally_affects_animations_: Cell<bool>,
    affects_compositor_snapshots_: Cell<bool>,
    rejected_legacy_overlapping_: Cell<bool>,
    has_tree_scoped_reference_: Cell<bool>,
    has_unsupported_guaranteed_invalid_: Cell<bool>,
}

impl<'a, B: StyleResolverStateBackend> StyleResolverState<'a, B> {
    // cpp: style_resolver_state.cc:67-78,98-139
    pub fn new(
        backend: &'a B,
        document: &'a B::Document,
        element: Rc<B::Element>,
        recalc: Option<&'a B::StyleRecalcContext>,
        request: Option<&B::StyleRequest>,
    ) -> Self {
        let default_request = request.is_none().then(|| backend.DefaultStyleRequest());
        let request = request
            .or(default_request.as_ref())
            .expect("default style request");
        let element_context = backend.NewElementResolveContext(element.clone());
        let pseudo_id = backend.RequestPseudoId(request);
        let mut styled_element = Some(
            backend
                .RequestStyledElement(request)
                .unwrap_or_else(|| element.clone()),
        );
        if pseudo_id != PseudoId::kPseudoIdNone {
            styled_element = backend.GetStyledPseudoElement(
                styled_element.as_ref().unwrap(),
                pseudo_id,
                &backend.RequestPseudoArgument(request),
            );
        }
        let resources = backend.NewElementStyleResources(
            styled_element.as_deref().unwrap_or(&element),
            backend.DocumentDevicePixelRatio(document),
        );
        let is_for_highlight = IsHighlightPseudoElement(pseudo_id);
        let mut parent = backend.RequestParentOverride(request);
        let mut layout_parent = backend.RequestLayoutParentOverride(request);
        if !is_for_highlight {
            if parent.is_none() {
                parent = backend.ContextParentStyle(&element_context);
            }
            if layout_parent.is_none() {
                layout_parent = backend.ContextLayoutParentStyle(&element_context);
            }
        }
        if layout_parent.is_none() {
            layout_parent = parent.clone();
        }
        Self {
            backend_: backend,
            element_context_: element_context,
            style_recalc_context_: recalc,
            document_: document,
            style_builder_: RefCell::new(None),
            length_conversion_flags_: Rc::new(Cell::new(0)),
            css_to_length_conversion_data_dirty_: Cell::new(false),
            should_update_line_height_: Cell::new(false),
            css_to_length_conversion_data_: Rc::new(RefCell::new(
                backend.NewElementLengthConversionData(&element),
            )),
            parent_style_: RefCell::new(parent),
            layout_parent_style_: RefCell::new(layout_parent),
            old_style_: RefCell::new(recalc.and_then(|context| backend.RecalcOldStyle(context))),
            animation_update_: RefCell::new(backend.NewAnimationUpdate()),
            pseudo_request_type_: backend.RequestType(request),
            font_builder_: RefCell::new(backend.NewFontBuilder(document)),
            styled_element_: styled_element,
            element_style_resources_: RefCell::new(resources),
            pseudo_id_: pseudo_id,
            inside_link_: Cell::new(None),
            originating_element_style_: backend.RequestOriginatingElementStyle(request),
            is_for_highlight_: is_for_highlight,
            can_trigger_animations_: backend.RequestCanTriggerAnimations(request),
            had_no_matched_properties_: Cell::new(false),
            conditionally_affects_animations_: Cell::new(false),
            affects_compositor_snapshots_: Cell::new(false),
            rejected_legacy_overlapping_: Cell::new(false),
            has_tree_scoped_reference_: Cell::new(false),
            has_unsupported_guaranteed_invalid_: Cell::new(false),
        }
    }
    // cpp: style_resolver_state.cc:147-177
    pub fn IsInheritedForUnset(&self, property: &CSSProperty) -> bool {
        property.IsInherited() || self.IsForHighlight()
    }
    pub fn InsideLink(&self) -> EInsideLink {
        if let Some(value) = self.inside_link_.get() {
            return value;
        }
        let parent = self.ParentStyle();
        let mut inside = parent
            .as_ref()
            .map_or(EInsideLink::kNotInsideLink, |parent| {
                StyleRef(parent).InsideLink()
            });
        let element = self.GetElement();
        if (!self.IsForPseudoElement() && self.backend_.ElementIsLink(&element))
            || self.IsForHighlight()
        {
            inside = self.ElementLinkState();
        }
        if inside == EInsideLink::kInsideVisitedLink
            && self.backend_.ElementIsInCanvasSubtree(&element)
        {
            inside = EInsideLink::kInsideUnvisitedLink;
        }
        self.inside_link_.set(Some(inside));
        inside
    }
    // cpp: style_resolver_state.cc:179-196
    pub fn TakeStyle(&self) -> Option<ComputedStyleHandle> {
        if self.had_no_matched_properties_.get()
            && MayReturnNullRenderingStyleForPseudoElement(
                self.backend_.ElementPseudoIdForStyling(&self.GetElement()),
                self.ParentStyle().as_ref().map(StyleRef),
            )
            && self.pseudo_request_type_ == StyleResolverRequestType::kForRenderer
        {
            return None;
        }
        Some(Persistent::from_ptr(
            self.StyleBuilderMut().TakeStyle().cast_mut(),
        ))
    }
    pub fn CloneStyle(&self) -> Option<ComputedStyleHandle> {
        if self.had_no_matched_properties_.get()
            && self.pseudo_request_type_ == StyleResolverRequestType::kForRenderer
        {
            return None;
        }
        Some(Persistent::from_ptr(
            self.StyleBuilder().CloneStyle().cast_mut(),
        ))
    }
    // cpp: style_resolver_state.cc:198-217
    fn UpdateLengthConversionData(&self) {
        let builder = self.StyleBuilder();
        let parent = self.ParentStyle();
        let root = self.RootElementStyle();
        let containers = self.backend_.NewContainerSizes(self.ContainerUnitContext());
        let anchors = self.backend_.NewAnchorData(
            self.GetAnchorEvaluator(),
            builder.GetDefaultAnchorData(),
            *builder.PositionAreaOffsets(),
        );
        let data = self.backend_.NewBuilderLengthConversionData(
            &builder,
            parent.as_ref().map(StyleRef),
            root.as_ref().map(StyleRef),
            self.backend_.DocumentViewportSize(self.document_),
            containers,
            anchors,
            builder.EffectiveZoom(),
            self.length_conversion_flags_.clone(),
            self.GetElement(),
        );
        *self.css_to_length_conversion_data_.borrow_mut() = data;
        if self.should_update_line_height_.get() {
            let document_root = self.backend_.DocumentElementComputedStyle(self.document_);
            let line_height = self.backend_.NewLineHeightSize(
                builder.GetFontSizeStyle(),
                document_root.as_ref().map(StyleRef),
            );
            self.css_to_length_conversion_data_
                .borrow_mut()
                .SetLineHeightSize(line_height);
            self.should_update_line_height_.set(false);
        }
        self.css_to_length_conversion_data_dirty_.set(false);
        self.backend_.ResourcesUpdateLengthConversionData(
            &mut self.element_style_resources_.borrow_mut(),
            &self.css_to_length_conversion_data_,
        );
    }
    // cpp: style_resolver_state.cc:219-247
    fn UnzoomedLengthConversionDataWithFont(
        &self,
        font: FontSizeStyle<'_>,
    ) -> B::LengthConversionData {
        let root = self.RootElementStyle();
        let font_sizes = self
            .backend_
            .NewFontSizes(font, root.as_ref().map(StyleRef));
        let parent = self.ParentStyle();
        let builder = self.StyleBuilder();
        let line_height_font = parent.as_ref().map(StyleRef).map_or_else(
            || builder.GetFontSizeStyle(),
            ComputedStyle::GetFontSizeStyle,
        );
        let line_height = self
            .backend_
            .NewLineHeightSize(line_height_font, root.as_ref().map(StyleRef));
        let viewport = self
            .backend_
            .ViewportSizeFromDocumentLayoutView(self.document_);
        let containers = self.backend_.NewContainerSizes(self.ContainerUnitContext());
        let anchors = self.backend_.NewAnchorData(
            self.GetAnchorEvaluator(),
            builder.GetDefaultAnchorData(),
            *builder.PositionAreaOffsets(),
        );
        self.backend_.NewUnzoomedLengthConversionData(
            builder.GetWritingMode(),
            font_sizes,
            line_height,
            viewport,
            containers,
            anchors,
            1.0,
            self.length_conversion_flags_.clone(),
            self.GetElement(),
        )
    }
    pub fn FontSizeConversionData(&self) -> B::LengthConversionData {
        let parent = self
            .ParentStyle()
            .expect("font-size conversion requires parent style");
        self.UnzoomedLengthConversionDataWithFont(StyleRef(&parent).GetFontSizeStyle())
    }
    pub fn UnzoomedLengthConversionData(&self) -> B::LengthConversionData {
        self.UnzoomedLengthConversionDataWithFont(self.StyleBuilder().GetFontSizeStyle())
    }
    // cpp: style_resolver_state.cc:249-259
    fn ContainerUnitContext(&self) -> Option<Rc<B::Element>> {
        match self.style_recalc_context_ {
            Some(context) => self.backend_.RecalcSizeContainer(context),
            None => self.backend_.FlatTreeParentElement(&self.GetElement()),
        }
    }
    fn GetAnchorEvaluator(&self) -> Option<Rc<B::AnchorEvaluator>> {
        self.style_recalc_context_
            .and_then(|context| self.backend_.RecalcAnchorEvaluator(context))
    }
    // cpp: style_resolver_state.cc:261-279
    pub fn SetParentStyle(&self, parent: Option<ComputedStyleHandle>) {
        *self.parent_style_.borrow_mut() = parent;
        if self.style_builder_.borrow().is_some() {
            self.InvalidateLengthConversionData();
        }
    }
    pub fn EnsureParentStyle(&self) {
        if self.ParentStyle().is_none() {
            let parent = StyleResolver::new(self.document_, self.backend_).InitialStyleForElement();
            self.SetParentStyle(Some(parent));
            self.SetLayoutParentStyle(self.ParentStyle());
        }
    }
    pub fn SetLayoutParentStyle(&self, parent: Option<ComputedStyleHandle>) {
        *self.layout_parent_style_.borrow_mut() = parent;
    }
    // cpp: style_resolver_state.cc:281-319
    pub fn LoadPendingResources(&self) {
        let parent = self.ParentStyle();
        if self.pseudo_request_type_ == StyleResolverRequestType::kForComputedStyle
            || parent
                .as_ref()
                .is_some_and(|parent| StyleRef(parent).IsEnsuredInDisplayNone())
        {
            return;
        }
        {
            let builder = self.StyleBuilder();
            if builder.Display() == EDisplay::kNone {
                if let Some(element) = self.GetStyledElement() {
                    if !self
                        .backend_
                        .LayoutObjectIsNeeded(&element, builder.GetDisplayStyle())
                    {
                        let animating = self.GetAnimatingElement();
                        if animating.as_ref().is_none_or(|element| {
                            !self.backend_.IsAnimatingDisplayProperty(element)
                        }) {
                            return;
                        }
                    }
                }
            }
            if matches!(
                builder.StyleType(),
                PseudoId::kPseudoIdSearchText | PseudoId::kPseudoIdTargetText
            ) {
                return;
            }
        }
        let data = self.CssToLengthConversionData();
        self.backend_.ResourcesLoadPendingResources(
            &mut self.element_style_resources_.borrow_mut(),
            &mut self.StyleBuilderMut(),
            &data,
        );
    }
    pub fn GetSVGResource(
        &self,
        property: CSSPropertyID,
        value: &ResolverURIValue<B>,
    ) -> Option<Rc<B::SVGResource>> {
        let resource = self.backend_.ResourcesGetSVGResource(
            &mut self.element_style_resources_.borrow_mut(),
            property,
            value,
        );
        if resource.is_some() && self.backend_.URIIsLocal(value, self.document_) {
            self.SetHasTreeScopedReference();
        }
        resource
    }
    // cpp: style_resolver_state.cc:321-381. Use-counter-only blocks omitted.
    pub fn ParentFontDescription(&self) -> B::FontDescription {
        self.backend_.FontDescriptionFromStyle(StyleRef(
            &self
                .ParentStyle()
                .expect("parent font description requires parent style"),
        ))
    }
    pub fn SetZoom(&self, zoom: f32) {
        let parent = self.ParentStyle();
        let parent_zoom = parent
            .as_ref()
            .map_or_else(ComputedStyleInitialValues::InitialZoom, |style| {
                StyleRef(style).EffectiveZoom()
            });
        let mut builder = self.StyleBuilderMut();
        builder.SetZoom(zoom);
        if builder.SetEffectiveZoom(parent_zoom * zoom) {
            self.font_builder_.borrow_mut().DidChangeEffectiveZoom();
        }
    }
    pub fn SetEffectiveZoom(&self, zoom: f32) {
        if self.StyleBuilderMut().SetEffectiveZoom(zoom) {
            self.font_builder_.borrow_mut().DidChangeEffectiveZoom();
        }
    }
    pub fn SetWritingMode(&self, writing_mode: WritingMode) {
        if self.StyleBuilder().GetWritingMode() == writing_mode {
            return;
        }
        self.StyleBuilderMut().SetWritingMode(writing_mode);
        self.InvalidateLengthConversionData();
        self.font_builder_.borrow_mut().DidChangeWritingMode();
    }
    pub fn SetTextSizeAdjust(&self, adjust: TextSizeAdjust) {
        if *self.StyleBuilder().GetTextSizeAdjust() == adjust {
            return;
        }
        self.StyleBuilderMut().SetTextSizeAdjust(&adjust);
        self.InvalidateLengthConversionData();
        self.font_builder_.borrow_mut().DidChangeTextSizeAdjust();
    }
    pub fn SetTextOrientation(&self, orientation: ETextOrientation) {
        if self.StyleBuilder().GetTextOrientation() != orientation {
            self.StyleBuilderMut().SetTextOrientation(orientation);
            self.font_builder_.borrow_mut().DidChangeTextOrientation();
        }
    }
    // cpp: style_resolver_state.cc:383-441
    fn CurrentAnchorData(&self) -> B::AnchorData {
        let builder = self.StyleBuilder();
        self.backend_.NewAnchorData(
            self.GetAnchorEvaluator(),
            builder.GetDefaultAnchorData(),
            *builder.PositionAreaOffsets(),
        )
    }
    pub fn SetPositionAnchor(&self, anchor: &StylePositionAnchor) {
        if self.StyleBuilder().PositionAnchor() == anchor {
            return;
        }
        self.StyleBuilderMut().SetPositionAnchor(anchor);
        self.MutableCssToLengthConversionData()
            .SetAnchorData(self.CurrentAnchorData());
    }
    pub fn SetPositionArea(&self, area: PositionArea) {
        if *self.StyleBuilder().GetPositionArea() == area {
            return;
        }
        self.StyleBuilderMut().SetPositionArea(&area);
        self.MutableCssToLengthConversionData()
            .SetAnchorData(self.CurrentAnchorData());
        if area.IsNone() {
            return;
        }
        self.StyleBuilderMut().SetHasAnchorFunctions();
        let Some(evaluator) = self.GetAnchorEvaluator() else {
            return;
        };
        let offsets = self.backend_.ComputePositionAreaOffsetsForLayout(
            &evaluator,
            self.StyleBuilder().GetDefaultAnchorData(),
        );
        if *self.StyleBuilder().PositionAreaOffsets() == offsets {
            return;
        }
        self.StyleBuilderMut().SetPositionAreaOffsets(&offsets);
        let default = self.StyleBuilder().GetDefaultAnchorData();
        let data = self
            .backend_
            .NewAnchorData(Some(evaluator), default, offsets);
        self.MutableCssToLengthConversionData().SetAnchorData(data);
    }
    pub fn GetAnchoredContainerWritingDirection(&self) -> WritingDirectionMode {
        self.backend_.AnchorContainerWritingDirection(
            &self
                .GetAnchorEvaluator()
                .expect("flips require the out-of-flow AnchorEvaluator"),
        )
    }
    // cpp: style_resolver_state.cc:443-469
    pub fn GetParserMode(&self) -> CSSParserMode {
        if self.backend_.DocumentInQuirksMode(self.document_) {
            CSSParserMode::kHTMLQuirksMode
        } else {
            CSSParserMode::kHTMLStandardMode
        }
    }
    pub fn GetAnimatingElement(&self) -> Option<Rc<B::Element>> {
        if self.IsForPseudoElement() {
            self.GetPseudoElement()
                .map(|element| self.backend_.PseudoElementAsElement(element))
        } else {
            self.styled_element_.clone()
        }
    }
    pub fn GetPseudoElement(&self) -> Option<Rc<B::PseudoElement>> {
        self.styled_element_
            .as_ref()
            .and_then(|element| self.backend_.ElementAsPseudoElement(element))
    }
    pub fn ResolveLightDarkPair(&self, value: Rc<ResolverValue<B>>) -> Rc<ResolverValue<B>> {
        if let CSSValuePayload::kLightDarkValuePairClass(pair) = value.Payload() {
            let resolved = if self.StyleBuilder().UsedColorScheme() == ColorScheme::kLight {
                self.backend_.LightDarkFirst(pair)
            } else {
                self.backend_.LightDarkSecond(pair)
            };
            return self.ResolveLightDarkPair(resolved);
        }
        value
    }
    // cpp: style_resolver_state.cc:83-94,471-514. The const and mutable overloads
    // share these source branches and select the actual dependency overload.
    fn TransparentImage(&self) -> Rc<ResolverValue<B>> {
        self.backend_
            .TransparentImageCache()
            .get_or_init(|| {
                self.backend_.NewColorImageValue(
                    self.backend_
                        .CreateIdentifierValue(CSSValueID::kTransparent),
                )
            })
            .clone()
    }
    fn IsNoneValue(&self, value: &ResolverValue<B>) -> bool {
        matches!(value.Payload(), CSSValuePayload::kIdentifierClass(ident) if self.backend_.IdentifierValueID(ident) == CSSValueID::kNone)
    }
    fn ResolveGradientsImpl(
        &self,
        value: Rc<ResolverValue<B>>,
        mode: ValueResolutionMode,
    ) -> Rc<ResolverValue<B>> {
        let was_pair = value.IsLightDarkValuePair();
        let resolved = self.ResolveLightDarkPair(value);
        if was_pair && self.IsNoneValue(&resolved) {
            return self.TransparentImage();
        }
        if resolved.IsGradientValue() {
            return self
                .backend_
                .ResolveGradientValuesIfNeeded(resolved, self, mode);
        }
        if resolved.IsImageSetValue() {
            return self
                .backend_
                .ResolveImageSetValuesIfNeeded(resolved, self, mode);
        }
        if resolved.IsCrossfadeValue() {
            return self
                .backend_
                .ResolveCrossfadeValuesIfNeeded(resolved, self, mode);
        }
        resolved
    }
    pub fn ResolveGradients(&self, value: Rc<ResolverValue<B>>) -> Rc<ResolverValue<B>> {
        self.ResolveGradientsImpl(value, ValueResolutionMode::Const)
    }
    pub fn ResolveGradientsMutable(&self, value: Rc<ResolverValue<B>>) -> Rc<ResolverValue<B>> {
        self.ResolveGradientsImpl(value, ValueResolutionMode::Mutable)
    }
    // cpp: style_resolver_state.cc:516-544
    pub fn UpdateFont(&self) {
        let parent = self.ParentStyle();
        self.font_builder_
            .borrow_mut()
            .CreateFont(&mut self.StyleBuilderMut(), parent.as_ref().map(StyleRef));
        if !self.css_to_length_conversion_data_dirty_.get() {
            let root = self.RootElementStyle();
            let fonts = self.backend_.NewFontSizes(
                self.StyleBuilder().GetFontSizeStyle(),
                root.as_ref().map(StyleRef),
            );
            self.SetConversionFontSizes(fonts);
            self.SetConversionZoom(self.StyleBuilder().EffectiveZoom());
        }
    }
    pub fn UpdateLineHeight(&self) {
        if self.css_to_length_conversion_data_dirty_.get() {
            self.should_update_line_height_.set(true);
        } else {
            let root = self.backend_.DocumentElementComputedStyle(self.document_);
            let height = self.backend_.NewLineHeightSize(
                self.StyleBuilder().GetFontSizeStyle(),
                root.as_ref().map(StyleRef),
            );
            self.MutableCssToLengthConversionData()
                .SetLineHeightSize(height);
        }
    }
    pub fn CanAffectAnimations(&self) -> bool {
        self.conditionally_affects_animations_.get() || self.StyleBuilder().CanAffectAnimations()
    }
    // cpp: style_resolver_state.cc:546-568
    pub fn SetComputedStyleFlagsFromAuthorFlags(&self, flags: PropertyFlags) {
        if flags & CSSProperty::kBackground != 0 {
            self.StyleBuilderMut().SetHasAuthorBackground();
        }
        if flags & CSSProperty::kBorder != 0 {
            self.StyleBuilderMut().SetHasAuthorBorder();
        }
        if flags & CSSProperty::kBorderRadius != 0 {
            self.StyleBuilderMut().SetHasAuthorBorderRadius();
        }
        if (self.InsideLink() != EInsideLink::kInsideVisitedLink
            && flags & CSSProperty::kHighlightColors != 0)
            || (self.InsideLink() == EInsideLink::kInsideVisitedLink
                && flags & CSSProperty::kVisitedHighlightColors != 0)
        {
            self.StyleBuilderMut().SetHasAuthorHighlightColors();
        }
    }

    // cpp: style_resolver_state.h:62-318 (inline/accessor bodies)
    pub fn IsForPseudoElement(&self) -> bool {
        self.pseudo_id_ != PseudoId::kPseudoIdNone
            || self
                .backend_
                .ContextPseudoElement(&self.element_context_)
                .is_some()
    }
    pub fn GetDocument(&self) -> &B::Document {
        self.document_
    }
    pub fn GetStyledElement(&self) -> Option<Rc<B::Element>> {
        self.styled_element_.clone()
    }
    pub fn GetElement(&self) -> Rc<B::Element> {
        self.backend_.ContextElement(&self.element_context_)
    }
    pub fn GetUltimateOriginatingElementOrSelf(&self) -> Rc<B::Element> {
        self.backend_
            .ContextUltimateOriginatingElementOrSelf(&self.element_context_)
    }
    pub fn ParentElement(&self) -> Option<Rc<B::Element>> {
        self.backend_.ContextParentElement(&self.element_context_)
    }
    pub fn RootElementStyle(&self) -> Option<ComputedStyleHandle> {
        self.backend_
            .ContextRootElementStyle(&self.element_context_)
    }
    pub fn ElementLinkState(&self) -> EInsideLink {
        self.backend_
            .ContextElementLinkState(&self.element_context_)
    }
    pub fn ElementContext(&self) -> &B::ElementResolveContext {
        &self.element_context_
    }
    pub fn CreateNewClonedStyle(&self, style: &ComputedStyle) {
        *self.style_builder_.borrow_mut() = Some(ComputedStyleBuilder::from_style(style));
        self.InvalidateLengthConversionData();
    }
    pub fn CreateNewStyle(
        &self,
        noninherited: &ComputedStyle,
        parent: &ComputedStyle,
        boundary: Option<IsAtShadowBoundary>,
    ) {
        *self.style_builder_.borrow_mut() = Some(ComputedStyleBuilder::from_initial_and_parent(
            noninherited,
            parent,
            boundary.unwrap_or(IsAtShadowBoundary::kNotAtShadowBoundary),
        ));
        self.InvalidateLengthConversionData();
    }
    pub fn StyleBuilder(&self) -> Ref<'_, ComputedStyleBuilder> {
        Ref::map(self.style_builder_.borrow(), |builder| {
            builder.as_ref().expect("style builder must be initialized")
        })
    }
    pub fn StyleBuilderMut(&self) -> RefMut<'_, ComputedStyleBuilder> {
        RefMut::map(self.style_builder_.borrow_mut(), |builder| {
            builder.as_mut().expect("style builder must be initialized")
        })
    }
    pub fn CssToLengthConversionData(&self) -> Ref<'_, B::LengthConversionData> {
        if self.css_to_length_conversion_data_dirty_.get() {
            self.UpdateLengthConversionData();
        }
        self.css_to_length_conversion_data_.borrow()
    }
    fn MutableCssToLengthConversionData(&self) -> RefMut<'_, B::LengthConversionData> {
        if self.css_to_length_conversion_data_dirty_.get() {
            self.UpdateLengthConversionData();
        }
        self.css_to_length_conversion_data_.borrow_mut()
    }
    pub fn TakeLengthConversionFlags(&self) -> u32 {
        if self.css_to_length_conversion_data_dirty_.get() {
            self.UpdateLengthConversionData();
        }
        self.length_conversion_flags_.replace(0)
    }
    pub fn SubtractScrollbarsFromViewportUnits(&self, scrollbars: &gfx::Size) {
        self.MutableCssToLengthConversionData()
            .SubtractScrollbars(scrollbars);
    }
    pub fn AnimationUpdate(&self) -> Ref<'_, B::AnimationUpdate> {
        self.animation_update_.borrow()
    }
    pub fn AnimationUpdateMut(&self) -> RefMut<'_, B::AnimationUpdate> {
        self.animation_update_.borrow_mut()
    }
    pub fn ParentStyle(&self) -> Option<ComputedStyleHandle> {
        self.parent_style_.borrow().clone()
    }
    pub fn LayoutParentStyle(&self) -> Option<ComputedStyleHandle> {
        self.layout_parent_style_.borrow().clone()
    }
    pub fn SetOldStyle(&self, style: Option<ComputedStyleHandle>) {
        *self.old_style_.borrow_mut() = style;
    }
    pub fn OldStyle(&self) -> Option<ComputedStyleHandle> {
        self.old_style_.borrow().clone()
    }
    pub fn GetElementStyleResources(&self) -> RefMut<'_, B::ElementStyleResources> {
        self.element_style_resources_.borrow_mut()
    }
    pub fn GetStyleImage(
        &self,
        property: CSSPropertyID,
        value: Rc<ResolverValue<B>>,
    ) -> Option<Rc<StyleImage>> {
        let resolved = self.ResolveGradients(value);
        self.backend_.ResourcesGetStyleImage(
            &mut self.element_style_resources_.borrow_mut(),
            property,
            resolved,
        )
    }
    pub fn GetFontBuilder(&self) -> Ref<'_, B::FontBuilder> {
        self.font_builder_.borrow()
    }
    pub fn GetFontBuilderMut(&self) -> RefMut<'_, B::FontBuilder> {
        self.font_builder_.borrow_mut()
    }
    pub fn OriginatingElementStyle(&self) -> Option<ComputedStyleHandle> {
        self.originating_element_style_.clone()
    }
    pub fn IsForHighlight(&self) -> bool {
        self.is_for_highlight_
    }
    pub fn CanTriggerAnimations(&self) -> bool {
        self.can_trigger_animations_
    }
    pub fn HadNoMatchedProperties(&self) -> bool {
        self.had_no_matched_properties_.get()
    }
    pub fn SetHadNoMatchedProperties(&self) {
        self.had_no_matched_properties_.set(true);
    }
    pub fn SetConditionallyAffectsAnimations(&self) {
        self.conditionally_affects_animations_.set(true);
    }
    pub fn AffectsCompositorSnapshots(&self) -> bool {
        self.affects_compositor_snapshots_.get()
    }
    pub fn SetAffectsCompositorSnapshots(&self) {
        self.affects_compositor_snapshots_.set(true);
    }
    pub fn RejectedLegacyOverlapping(&self) -> bool {
        self.rejected_legacy_overlapping_.get()
    }
    pub fn SetRejectedLegacyOverlapping(&self) {
        self.rejected_legacy_overlapping_.set(true);
    }
    pub fn InvalidateLengthConversionData(&self) {
        self.css_to_length_conversion_data_dirty_.set(true);
    }
    pub fn SetHasTreeScopedReference(&self) {
        self.has_tree_scoped_reference_.set(true);
    }
    pub fn HasTreeScopedReference(&self) -> bool {
        self.has_tree_scoped_reference_.get()
    }
    pub fn SetHasUnsupportedGuaranteedInvalid(&self) {
        self.has_unsupported_guaranteed_invalid_.set(true);
    }
    pub fn HasUnsupportedGuaranteedInvalid(&self) -> bool {
        self.has_unsupported_guaranteed_invalid_.get()
    }
    pub fn NearestSizeContainer(&self) -> Option<Rc<B::Element>> {
        self.style_recalc_context_
            .and_then(|context| self.backend_.RecalcSizeContainer(context))
    }
    pub fn GetPseudoId(&self) -> PseudoId {
        self.pseudo_id_
    }
    fn SetConversionFontSizes(&self, sizes: B::FontSizes) {
        self.MutableCssToLengthConversionData().SetFontSizes(sizes);
    }
    fn SetConversionZoom(&self, zoom: f32) {
        self.MutableCssToLengthConversionData().SetZoom(zoom);
    }
}
// cpp: style_resolver_state.cc:141-145. Rust releases the owned containers;
// the explicit animation Clear preserves the source's lifecycle callback.
impl<B: StyleResolverStateBackend> Drop for StyleResolverState<'_, B> {
    fn drop(&mut self) {
        self.backend_
            .ClearAnimationUpdate(self.animation_update_.get_mut());
    }
}
