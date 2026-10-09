/*
 * Copyright (C) 2013 Google, Inc.
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2003, 2004, 2005, 2006, 2007, 2008, 2009, 2010, 2011 Apple Inc.
 * All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */
// cpp: third_party/blink/renderer/core/css/resolver/style_adjuster.h
// cpp: third_party/blink/renderer/core/css/resolver/style_adjuster.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   style_adjuster.h: 146 / 66 / 50 / 16 / 0.
//   style_adjuster.cc: 1849 / 1342 / 1227 / 115 / 0.
// Effective excludes comments/blanks; braces retained. Every production
// declaration and function body is mapped. Required traits expose foreign
// DOM/frame/theme/ListMarker/console/UI feature owners only; no internal
// StyleAdjuster body is delegated or represented by a replacement style model.
// h mapped:
// 41,45-52,87-88,90-92,94-100,108-112,116-119,122-129,131-142.
// h omitted: 23-24,26-28,30,32-36,42,44,121,144,146.
// cc mapped:
// 115-117,119-126,129-138,141-155,157-167,171-183,187-192,197-205,212-213,216-217,219-222,228-229
// 232-235,238-240,244-245,249-255,258-292,294-311,314-338,340-349,351-363,365-369,371-374,376-386
// 394-400,402-408,410-412,417-418,420-422,424-431,433,436,438-452,454-463,465-467,475,477-481,484-487
// 489-494,496-498,500-512,514,517,524-525,527-535,537-546,549-558,560,563-568,570-571,575-578,580
// 585-587,589-590,598-600,602,604-606,608,610-615,618-620,622,624-633,635-639,641-650,652-655,657-660
// 662-664,671-674,680-685,689-694,697-699,703-707,710-712,716-720,741-747,749-754,756-758,760-761
// 766-775,777-782,784-799,801-804,806-820,822-824,826-828,830-832,836-840,846-851,857-860,863-865
// 868-880,882-886,888-890,892-894,896-898,900-901,903-912,914-919,921-924,929-931,934-937,940-944
// 946-949,951-953,955-959,972-975,979-981,986-989,991,994-999,1008-1009,1011-1018,1022-1027,1030-1031
// 1034-1041,1043-1048,1050-1060,1062-1070,1076-1080,1087-1102,1104,1109-1133,1135-1136,1139-1141
// 1143-1148,1150-1165,1167,1175-1181,1185-1187,1191-1195,1199-1202,1205-1209,1211,1213-1216,1218
// 1225-1230,1234-1237,1241-1245,1250,1252,1257-1259,1262-1269,1271-1275,1277-1283,1285-1289,1292-1298
// 1303-1312,1316-1317,1321,1323,1325-1326,1328,1330-1335,1337-1339,1347-1349,1354-1357,1359-1360
// 1362-1369,1379-1382,1384-1387,1389-1393,1397-1413,1417-1422,1427-1432,1434-1436,1439-1440,1442-1446
// 1449-1453,1455-1458,1460-1465,1468-1470,1472-1478,1481-1484,1488-1491,1494-1497,1499-1503,1506-1509
// 1511-1513,1519-1522,1525-1527,1531-1533,1538-1540,1545-1547,1551-1553,1557-1560,1564-1566,1568-1569
// 1574,1576,1580,1582,1585,1587,1591-1594,1596-1597,1599-1601,1603-1605,1609,1611,1613,1615,1620-1627
// 1632-1633,1635,1637,1639,1641,1643,1645,1647-1648,1650,1652-1657,1661,1664-1676,1678-1679,1682-1683
// 1685-1686,1691-1692,1695-1697,1701-1702,1705-1706,1710-1768,1772-1841,1846-1847.
// cc omitted: 31,33-107,109,111,113,185,434,437,469-474,665-666,668-669,700-702,713-715,722-723,727-731,733-737,821,1137-1138,1370,1849.
// Omissions: preprocessor/namespace/forward/access/static-class scaffolding,
// DCHECK and debug-only combined-text verification; overflow metrics-only
// variables/assignments/UseCounter blocks. Console diagnostics are retained.
// DOM adapters retain existing Element identity and provide a typed view for
// the canonical ComputedStyle Element ABI. State and builder are the existing
// StyleResolverState and layoutng_style::ComputedStyleBuilder, respectively.

use super::style_resolver::StyleResolver;
use super::style_resolver_state::{StyleResolverState, StyleResolverStateBackend};
use crate::PreferredColorScheme;
use font_engine::FontVariantEmoji;
use foundation::{
    AtomicString, EBoxOrient, EContentVisibility, EContinue, EDisplay, EDominantBaseline, EFloat,
    EForcedColorAdjust, EInternalOverscrollPosition, EIsolation, EOverflow, EOverlay, EPosition,
    ETextAlign, ETextDecorationSkipInk, ETextDecorationStyle, EUserModify, EViewTransitionScope,
    EWordBreak, IsHorizontalWritingMode, Length, Member, PhysicalToLogical, TextDecorationLine,
    TextDirection, TextEmphasisMark, TouchAction, ValuesEquivalent, Vector, WritingMode,
};
use layoutng_style::css::forward::ui::ColorProvider;
use layoutng_style::css::white_space::EWhiteSpace;
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::color_scheme::mojom::blink::ColorScheme;
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::computed_style_constants::{
    Containment, EVerticalAlign, ItemPosition, ItemPositionType, OverflowAlignment, PseudoId,
    TextUnderlinePosition,
};
use layoutng_style::style::computed_style_initial_values::ComputedStyleInitialValues as Initial;
use layoutng_style::style::forward::Element as ComputedStyleElement;
use layoutng_style::style::style_intrinsic_length::{
    StyleIntrinsicLength, StyleIntrinsicLengthOptions,
};
use layoutng_style::style::style_self_alignment_data::StyleSelfAlignmentData;
use layoutng_style::style::text_overflow_data::TextOverflowData;
use std::rc::Rc;

/// The foreign DOM class discriminator consumed by this source pair. This
/// class-key enum introduces no Element or style object implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElementType {
    kHTMLImageElement,
    kHTMLTableElement,
    kHTMLFrameElement,
    kHTMLFrameSetElement,
    kHTMLFencedFrameElement,
    kHTMLLegendElement,
    kHTMLMarqueeElement,
    kHTMLTextAreaElement,
    kHTMLEmbedElement,
    kHTMLObjectElement,
    kHTMLBodyElement,
    kHTMLBRElement,
    kHTMLWBRElement,
    kHTMLMeterElement,
    kHTMLProgressElement,
    kHTMLCanvasElement,
    kHTMLAudioElement,
    kHTMLVideoElement,
    kHTMLInputElement,
    kHTMLSelectElement,
    kHTMLIFrameElement,
    kIsNotElement,
    kMathMLAnchorElement,
    kMathMLElement,
    kMathMLFractionElement,
    kMathMLOperatorElement,
    kMathMLPaddedElement,
    kMathMLRadicalElement,
    kMathMLRowElement,
    kMathMLScriptsElement,
    kMathMLSpaceElement,
    kMathMLTableCellElement,
    kMathMLTokenElement,
    kMathMLUnderOverElement,
    kSVGSVGElement,
    kSVGUseElement,
    kSVGGElement,
    kSVGTSpanElement,
    kSVGForeignObjectElement,
    kSVGTextElement,
    kSVGAElement,
    kSVGAnimateElement,
    kSVGAnimateMotionElement,
    kSVGAnimateTransformElement,
    kSVGCircleElement,
    kSVGClipPathElement,
    kSVGDefsElement,
    kSVGDescElement,
    kSVGEllipseElement,
    kSVGFEBlendElement,
    kSVGFEColorMatrixElement,
    kSVGFEComponentTransferElement,
    kSVGFECompositeElement,
    kSVGFEConvolveMatrixElement,
    kSVGFEDiffuseLightingElement,
    kSVGFEDisplacementMapElement,
    kSVGFEDistantLightElement,
    kSVGFEDropShadowElement,
    kSVGFEFloodElement,
    kSVGFEFuncAElement,
    kSVGFEFuncBElement,
    kSVGFEFuncGElement,
    kSVGFEFuncRElement,
    kSVGFEGaussianBlurElement,
    kSVGFEImageElement,
    kSVGFEMergeElement,
    kSVGFEMergeNodeElement,
    kSVGFEMorphologyElement,
    kSVGFEOffsetElement,
    kSVGFEPointLightElement,
    kSVGFESpecularLightingElement,
    kSVGFESpotLightElement,
    kSVGFETileElement,
    kSVGFETurbulenceElement,
    kSVGFilterElement,
    kSVGImageElement,
    kSVGLinearGradientElement,
    kSVGLineElement,
    kSVGMarkerElement,
    kSVGMaskElement,
    kSVGMetadataElement,
    kSVGMPathElement,
    kSVGPathElement,
    kSVGPatternElement,
    kSVGPolygonElement,
    kSVGPolylineElement,
    kSVGRadialGradientElement,
    kSVGRectElement,
    kSVGScriptElement,
    kSVGSetElement,
    kSVGStopElement,
    kSVGStyleElement,
    kSVGSwitchElement,
    kSVGSymbolElement,
    kSVGTextPathElement,
    kSVGTitleElement,
    kSVGUnknownElement,
    kSVGViewElement,
    kHTMLAnchorElement,
    kHTMLAreaElement,
    kHTMLBaseElement,
    kHTMLBDIElement,
    kHTMLButtonElement,
    kHTMLCredentialElement,
    kHTMLDataElement,
    kHTMLDataListElement,
    kHTMLDetailsElement,
    kHTMLDialogElement,
    kHTMLDirectoryElement,
    kHTMLDivElement,
    kHTMLDListElement,
    kHTMLElement,
    kHTMLFieldSetElement,
    kHTMLFontElement,
    kHTMLFormElement,
    kHTMLGeolocationElement,
    kHTMLHeadElement,
    kHTMLHeadingElement,
    kHTMLHRElement,
    kHTMLHtmlElement,
    kHTMLInstallElement,
    kHTMLLabelElement,
    kHTMLLIElement,
    kHTMLLinkElement,
    kHTMLLoginElement,
    kHTMLMapElement,
    kHTMLMenuBarElement,
    kHTMLMenuElement,
    kHTMLMenuItemElement,
    kHTMLMenuListElement,
    kHTMLMetaElement,
    kHTMLModElement,
    kHTMLNoEmbedElement,
    kHTMLNoScriptElement,
    kHTMLOListElement,
    kHTMLOptGroupElement,
    kHTMLOptionElement,
    kHTMLOutputElement,
    kHTMLParagraphElement,
    kHTMLParamElement,
    kHTMLPictureElement,
    kHTMLPreElement,
    kHTMLQuoteElement,
    kHTMLScriptElement,
    kHTMLSearchElement,
    kHTMLSelectedContentElement,
    kHTMLSlotElement,
    kHTMLSourceElement,
    kHTMLSpanElement,
    kHTMLStyleElement,
    kHTMLSubMenuElement,
    kHTMLSummaryElement,
    kHTMLTableCaptionElement,
    kHTMLTableCellElement,
    kHTMLTableColElement,
    kHTMLTableRowElement,
    kHTMLTableSectionElement,
    kHTMLTemplateElement,
    kHTMLTimeElement,
    kHTMLTitleElement,
    kHTMLTrackElement,
    kHTMLUListElement,
    kHTMLUnknownElement,
    kHTMLUserMediaElement,
    kHTMLCameraElement,
    kHTMLMicrophoneElement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputControlType {
    File,
    Password,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleAdjustmentFeature {
    SvgTextDecorationCssStyling,
    NonStandardSliderVertical,
    SingleAxisScrollContainers,
    SwipeToMoveCursor,
    StylusHandwriting,
    EmojiMonochromeRendering,
    OverlayProperty,
    OverlayGlobalRuleRemoval,
    StackingContextIsNotStacked,
    ScopedViewTransitionSizeContainment,
}

/// Foreign DOM/frame ownership only. All returned Rc values must retain the
/// actual existing object identity; there are no fallback DOM or style models.
pub trait StyleAdjusterDOMBackend: StyleResolverStateBackend {
    type ContentFrame;
    type ViewTransition;
    fn ElementType(&self, element: &Self::Element) -> ElementType;
    fn ElementDocument<'e>(&self, element: &'e Self::Element) -> &'e Self::Document;
    fn DocumentElement(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn FirstBodyElement(&self, document: &Self::Document) -> Option<Rc<Self::Element>>;
    fn HasLocalOwner(&self, document: &Self::Document) -> bool;
    fn AsComputedStyleElement<'e>(&self, element: &'e Self::Element) -> &'e ComputedStyleElement;
    fn ElementComputedStyle<'e>(&self, element: &'e Self::Element) -> Option<&'e ComputedStyle>;
    fn FlatTreeParentElementSkippingSlots(
        &self,
        element: &Self::Element,
    ) -> Option<Rc<Self::Element>>;
    fn ParentElement(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn ParentShadowRootHost(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn OwnerShadowHost(&self, element: &Self::Element) -> Option<Rc<Self::Element>>;
    fn IsHTMLElement(&self, element: &Self::Element) -> bool;
    fn IsSVGElement(&self, element: &Self::Element) -> bool;
    fn IsMathMLElement(&self, element: &Self::Element) -> bool;
    fn IsMediaElement(&self, element: &Self::Element) -> bool;
    fn IsFrameOwnerElement(&self, element: &Self::Element) -> bool;
    fn IsPseudoElement(&self, element: &Self::Element) -> bool;
    fn IsInUserAgentShadowRoot(&self, element: &Self::Element) -> bool;
    fn IsOutermostSVGSVGElement(&self, element: &Self::Element) -> bool;
    fn HasEditContext(&self, element: &Self::Element) -> bool;
    fn InputControlType(&self, input: &Self::Element) -> InputControlType;
    fn IsDisabledOrReadOnly(&self, element: &Self::Element) -> bool;
    fn IsTextField(&self, input: &Self::Element) -> bool;
    fn ShouldRevealPassword(&self, input: &Self::Element) -> bool;
    fn InputHasDataList(&self, input: &Self::Element) -> bool;
    fn ImageIsCollapsed(&self, image: &Self::Element) -> bool;
    fn PluginShouldAccelerate(&self, plugin: &Self::Element) -> bool;
    fn GetStyleResolver<'d>(&self, document: &'d Self::Document) -> &'d StyleResolver<'d, Self>;
    fn IsCanvasOrInCanvasSubtree(&self, element: &Self::Element) -> bool;
    fn IsInCanvasSubtree(&self, element: &Self::Element) -> bool;
    fn CanvasDrawElementEnabled(&self, element: &Self::Element) -> bool;
    fn CanvasLayoutSubtree(&self, canvas: &Self::Element) -> bool;
    fn CanExecuteScripts(&self, element: &Self::Element) -> bool;
    fn IsInTopLayer(&self, element: &Self::Element) -> bool;
    fn IsRenderedInTopLayer(&self, element: &Self::Element) -> bool;
    fn CanvasForDrawingExists(&self, element: &Self::Element) -> bool;
    fn HasControlsAttribute(&self, element: &Self::Element) -> bool;
    fn HasPopoverAttribute(&self, element: &Self::Element) -> bool;
    fn IsInMainFrame(&self, document: &Self::Document) -> bool;
    fn IsSliderContainer(&self, element: &Self::Element) -> bool;
    fn IsInShadowTree(&self, element: &Self::Element) -> bool;
    fn ShadowPseudoId(&self, element: &Self::Element) -> AtomicString;
    fn InputPlaceholderPseudoId(&self) -> AtomicString;
    fn InternalInputSuggestedPseudoId(&self) -> AtomicString;
    fn ValueForTextOverflow(&self, text_control: &Self::Element) -> TextOverflowData;
    fn IsVerticalScrollEnforced(&self, document: &Self::Document) -> bool;
    fn FrameInheritedEffectiveTouchAction(&self, frame: &Self::LocalFrame) -> TouchAction;
    fn ContentFrame(&self, owner: &Self::Element) -> Option<Rc<Self::ContentFrame>>;
    fn SetInheritedEffectiveTouchAction(&self, frame: &Self::ContentFrame, action: TouchAction);
    fn DocumentHasViewTransitions(&self, document: &Self::Document) -> bool;
    fn GetTransition(&self, element: &Self::Element) -> Option<Rc<Self::ViewTransition>>;
    fn TransitionScope(&self, transition: &Self::ViewTransition) -> Rc<Self::Element>;
    fn NeedsContainmentForDurationOfCapture(&self, transition: &Self::ViewTransition) -> bool;
    fn IsViewTransitionElementExcludingRootFromSupplement(&self, element: &Self::Element) -> bool;
    fn SupportsBaseAppearance(&self, element: &Self::Element, appearance: AppearanceValue) -> bool;
    fn HasCustomStyleCallbacks(&self, element: &Self::Element) -> bool;
    fn ElementAdjustStyle(&self, element: &Self::Element, builder: &mut ComputedStyleBuilder);
    fn IsInWebAppScope(&self, document: &Self::Document) -> bool;
    fn IsInitialProfile(&self, document: &Self::Document) -> bool;
}

/// Existing LayoutTheme, ListMarker, console and UI/platform feature owners.
pub trait StyleAdjusterBackend: StyleAdjusterDOMBackend {
    fn FeatureEnabled(&self, feature: StyleAdjustmentFeature) -> bool;
    fn InlineMarginsForInside(
        &self,
        document: &Self::Document,
        builder: &ComputedStyleBuilder,
        parent: &ComputedStyle,
    ) -> (f32, f32);
    fn ThemeAdjustStyle(&self, element: &Self::Element, builder: &mut ComputedStyleBuilder);
    fn AddRenderingInfoConsoleMessage(
        &self,
        document: &Self::Document,
        message: &str,
        discard_duplicates: bool,
    );
    fn ColorProviderForPainting(
        &self,
        document: &Self::Document,
        scheme: ColorScheme,
    ) -> *const ColorProvider;
}

// cpp: style_adjuster.h:87-99
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElementTypeForCache {
    pub element_type: ElementType,
}
impl ElementTypeForCache {
    pub fn CacheEntryIsStyleAdjusted(&self) -> bool {
        self.element_type != ElementType::kIsNotElement
    }
}

pub struct StyleAdjuster<'a, B: StyleAdjusterBackend> {
    backend: &'a B,
}
impl<'a, B: StyleAdjusterBackend> StyleAdjuster<'a, B> {
    pub fn new(backend: &'a B) -> Self {
        Self { backend }
    }
    fn Feature(&self, feature: StyleAdjustmentFeature) -> bool {
        self.backend.FeatureEnabled(feature)
    }
    fn Is(&self, element: Option<&B::Element>, class: ElementType) -> bool {
        element.is_some_and(|e| self.backend.ElementType(e) == class)
    }
    fn IsDocumentElement(&self, element: &B::Element) -> bool {
        self.backend
            .DocumentElement(self.backend.ElementDocument(element))
            .is_some_and(|root| std::ptr::eq(root.as_ref(), element))
    }
    // cpp: style_adjuster.cc:115-117
    fn IsOverflowClipOrVisible(overflow: EOverflow) -> bool {
        overflow == EOverflow::kClip || overflow == EOverflow::kVisible
    }
    // cpp: style_adjuster.cc:119-155
    fn AdjustTouchActionForElement(
        &self,
        touch_action: TouchAction,
        builder: &ComputedStyleBuilder,
        parent_style: &ComputedStyle,
        element: &B::Element,
    ) -> TouchAction {
        let b = self.backend;
        let document = b.ElementDocument(element);
        let document_element = b.DocumentElement(document);
        let mut scrolls_overflow = builder.ScrollsOverflow();
        if self.Is(Some(element), ElementType::kHTMLBodyElement)
            && b.FirstBodyElement(document)
                .is_some_and(|body| std::ptr::eq(body.as_ref(), element))
        {
            if parent_style.IsOverflowVisibleAlongBothAxes() {
                let root = document_element
                    .as_ref()
                    .expect("body requires a document element");
                if !parent_style.ShouldApplyAnyContainment(b.AsComputedStyleElement(root))
                    && !builder.ShouldApplyAnyContainment(b.AsComputedStyleElement(element))
                {
                    scrolls_overflow = false;
                }
            }
        }
        let is_child_document = document_element
            .as_ref()
            .is_some_and(|root| std::ptr::eq(root.as_ref(), element))
            && b.HasLocalOwner(document);
        if scrolls_overflow || is_child_document {
            let mut enabled = TouchAction::kNone;
            if is_child_document || ComputedStyle::ScrollsOverflowValue(builder.OverflowX()) {
                enabled |= TouchAction::kPanX | TouchAction::kInternalPanXScrolls;
            }
            if is_child_document || ComputedStyle::ScrollsOverflowValue(builder.OverflowY()) {
                enabled |= TouchAction::kPanY;
            }
            return touch_action | enabled | TouchAction::kInternalNotWritable;
        }
        touch_action
    }
    // cpp: style_adjuster.cc:157-167
    fn HostIsInputFile(&self, element: Option<&B::Element>) -> bool {
        let b = self.backend;
        let Some(element) = element else {
            return false;
        };
        if !b.IsInUserAgentShadowRoot(element) {
            return false;
        }
        let Some(host) = b.OwnerShadowHost(element) else {
            return false;
        };
        self.Is(Some(&host), ElementType::kHTMLInputElement)
            && b.InputControlType(&host) == InputControlType::File
    }
    // cpp: style_adjuster.cc:171-183
    fn ShouldBeInlinified(&self, element: Option<&B::Element>) -> bool {
        let b = self.backend;
        let Some(element) = element else {
            return true;
        };
        let mut parent = b.FlatTreeParentElement(element);
        while let Some(p) = parent.as_ref() {
            if !b
                .ElementComputedStyle(p)
                .is_some_and(|s| s.Display() == EDisplay::kContents)
            {
                break;
            }
            parent = b.FlatTreeParentElement(p);
        }
        !self.Is(parent.as_deref(), ElementType::kHTMLFieldSetElement)
            && !parent.as_ref().is_some_and(|p| b.IsMediaElement(p))
    }
    // cpp: style_adjuster.cc:187-255
    fn AdjustStyleForSvgElement(
        &self,
        element: &B::Element,
        styled_element: &B::Element,
        builder: &mut ComputedStyleBuilder,
        layout_parent_style: &ComputedStyle,
    ) {
        if builder.Display() != EDisplay::kNone {
            builder.SetTextDecorationSkipInk(ETextDecorationSkipInk::kAuto);
            if !self.Feature(StyleAdjustmentFeature::SvgTextDecorationCssStyling) {
                builder.SetTextDecorationStyle(ETextDecorationStyle::kSolid);
            }
            builder.SetTextEmphasisMark(TextEmphasisMark::kNone);
            builder.SetTextUnderlineOffset(&Length::default());
            builder.SetTextUnderlinePosition(TextUnderlinePosition::kAuto);
        }
        let is_svg_root = self.backend.IsOutermostSVGSVGElement(styled_element);
        if !is_svg_root {
            builder.SetPosition(Initial::InitialPosition());
        }
        if builder.Display() == EDisplay::kContents
            && (is_svg_root
                || !matches!(
                    self.backend.ElementType(element),
                    ElementType::kSVGSVGElement
                        | ElementType::kSVGGElement
                        | ElementType::kSVGUseElement
                        | ElementType::kSVGTSpanElement
                ))
        {
            builder.SetDisplay(EDisplay::kNone);
        }
        if matches!(
            self.backend.ElementType(element),
            ElementType::kSVGForeignObjectElement | ElementType::kSVGTextElement
        ) && builder.IsDisplayInlineType()
        {
            builder.SetDisplay(EDisplay::kBlock);
        }
        if self.Is(Some(element), ElementType::kSVGTextElement) {
            Self::AdjustForSVGTextElement(builder);
        }
        let mut baseline = builder.DominantBaseline();
        if baseline == EDominantBaseline::kUseScript {
            baseline = EDominantBaseline::kAlphabetic;
        } else if matches!(
            baseline,
            EDominantBaseline::kNoChange | EDominantBaseline::kResetSize
        ) {
            baseline = layout_parent_style.CssDominantBaseline();
        }
        builder.SetCssDominantBaseline(baseline);
    }
    // cpp: style_adjuster.cc:258-311
    fn EquivalentBlockDisplay(display: EDisplay) -> EDisplay {
        match display {
            EDisplay::kFlowRootListItem
            | EDisplay::kBlock
            | EDisplay::kTable
            | EDisplay::kWebkitBox
            | EDisplay::kFlex
            | EDisplay::kGrid
            | EDisplay::kBlockMath
            | EDisplay::kBlockRuby
            | EDisplay::kListItem
            | EDisplay::kFlowRoot
            | EDisplay::kLayoutCustom
            | EDisplay::kGridLanes => display,
            EDisplay::kInlineTable => EDisplay::kTable,
            EDisplay::kWebkitInlineBox => EDisplay::kWebkitBox,
            EDisplay::kInlineFlex => EDisplay::kFlex,
            EDisplay::kInlineGrid => EDisplay::kGrid,
            EDisplay::kMath => EDisplay::kBlockMath,
            EDisplay::kRuby => EDisplay::kBlockRuby,
            EDisplay::kInlineLayoutCustom => EDisplay::kLayoutCustom,
            EDisplay::kInlineListItem => EDisplay::kListItem,
            EDisplay::kInlineFlowRootListItem => EDisplay::kFlowRootListItem,
            EDisplay::kInlineGridLanes => EDisplay::kGridLanes,
            EDisplay::kContents
            | EDisplay::kInline
            | EDisplay::kInlineBlock
            | EDisplay::kTableRowGroup
            | EDisplay::kTableHeaderGroup
            | EDisplay::kTableFooterGroup
            | EDisplay::kTableRow
            | EDisplay::kTableColumnGroup
            | EDisplay::kTableColumn
            | EDisplay::kTableCell
            | EDisplay::kTableCaption
            | EDisplay::kRubyText => EDisplay::kBlock,
            EDisplay::kNone => unreachable!("none is excluded before display adjustment"),
        }
    }
    // cpp: style_adjuster.cc:314-369
    fn EquivalentInlineDisplay(display: EDisplay) -> EDisplay {
        match display {
            EDisplay::kFlowRootListItem => EDisplay::kInlineFlowRootListItem,
            EDisplay::kBlock | EDisplay::kFlowRoot => EDisplay::kInlineBlock,
            EDisplay::kTable => EDisplay::kInlineTable,
            EDisplay::kWebkitBox => EDisplay::kWebkitInlineBox,
            EDisplay::kFlex => EDisplay::kInlineFlex,
            EDisplay::kGrid => EDisplay::kInlineGrid,
            EDisplay::kGridLanes => EDisplay::kInlineGridLanes,
            EDisplay::kBlockMath => EDisplay::kMath,
            EDisplay::kBlockRuby => EDisplay::kRuby,
            EDisplay::kListItem => EDisplay::kInlineListItem,
            EDisplay::kLayoutCustom => EDisplay::kInlineLayoutCustom,
            EDisplay::kInlineFlex
            | EDisplay::kInlineFlowRootListItem
            | EDisplay::kInlineGrid
            | EDisplay::kInlineLayoutCustom
            | EDisplay::kInlineListItem
            | EDisplay::kInlineGridLanes
            | EDisplay::kInlineTable
            | EDisplay::kMath
            | EDisplay::kRuby
            | EDisplay::kWebkitInlineBox
            | EDisplay::kContents
            | EDisplay::kInline
            | EDisplay::kInlineBlock
            | EDisplay::kTableRowGroup
            | EDisplay::kTableHeaderGroup
            | EDisplay::kTableFooterGroup
            | EDisplay::kTableRow
            | EDisplay::kTableColumnGroup
            | EDisplay::kTableColumn
            | EDisplay::kTableCell
            | EDisplay::kTableCaption
            | EDisplay::kRubyText => display,
            EDisplay::kNone => unreachable!("none is excluded before display adjustment"),
        }
    }
    // cpp: style_adjuster.cc:371-374
    fn IsOutermostSVGElement(&self, element: Option<&B::Element>) -> bool {
        element.is_some_and(|e| {
            self.backend.IsSVGElement(e) && self.backend.IsOutermostSVGSVGElement(e)
        })
    }
    // cpp: style_adjuster.cc:376-386
    fn IsAtMediaUAShadowBoundary(&self, element: Option<&B::Element>) -> bool {
        element
            .and_then(|e| self.backend.ParentShadowRootHost(e))
            .is_some_and(|host| self.backend.IsMediaElement(&host))
    }
    // cpp: style_adjuster.cc:394-400
    fn StopPropagateTextDecorations(
        &self,
        builder: &ComputedStyleBuilder,
        element: Option<&B::Element>,
    ) -> bool {
        builder.IsAtomicInlineDisplayType()
            || self.IsAtMediaUAShadowBoundary(element)
            || builder.IsFloating()
            || builder.HasOutOfFlowPosition()
            || self.IsOutermostSVGElement(element)
            || builder.Display() == EDisplay::kRubyText
    }
    // cpp: style_adjuster.cc:402-408
    fn LayoutParentStyleForcesZIndexToCreateStackingContext(style: &ComputedStyle) -> bool {
        style.IsDisplayFlex()
            || style.IsDisplayWebkitBox()
            || style.IsDisplayGrid()
            || style.IsDisplayGridLanes()
    }
    // cpp: style_adjuster.cc:410-431
    pub fn AdjustStyleForEditing(
        &self,
        builder: &mut ComputedStyleBuilder,
        element: Option<&B::Element>,
    ) {
        if element.is_some_and(|e| self.backend.HasEditContext(e)) {
            builder.SetUserModify(EUserModify::kReadWrite);
        }
        if builder.UserModify() != EUserModify::kReadWritePlaintextOnly {
            return;
        }
        match builder.WhiteSpace() {
            EWhiteSpace::kNormal | EWhiteSpace::kPreLine => {
                builder.SetWhiteSpace(EWhiteSpace::kPreWrap)
            }
            EWhiteSpace::kNowrap => builder.SetWhiteSpace(EWhiteSpace::kPre),
            _ => {}
        }
    }
    // cpp: style_adjuster.cc:433-452
    pub fn AdjustStyleForTextCombine(builder: &mut ComputedStyleBuilder) {
        let font = unsafe { &*builder.GetFont() };
        let one_em = ComputedStyle::ComputedFontSizeAsFixed(font);
        let line_height = builder.FontHeight();
        let width = Length::Fixed(line_height.ToFloat());
        let height = Length::Fixed(one_em.ToFloat());
        builder.SetContainIntrinsicWidth(&StyleIntrinsicLength::new(
            &Some(width.clone()),
            StyleIntrinsicLengthOptions::default(),
        ));
        builder.SetContainIntrinsicHeight(&StyleIntrinsicLength::new(
            &Some(height.clone()),
            StyleIntrinsicLengthOptions::default(),
        ));
        builder.SetHeight(&height);
        builder.SetLineHeight(&height);
        builder.SetMaxHeight(&height);
        builder.SetMaxWidth(&width);
        builder.SetMinHeight(&height);
        builder.SetMinWidth(&width);
        builder.SetWidth(&width);
        Self::AdjustStyleForCombinedText(builder);
    }
    // cpp: style_adjuster.cc:454-475
    pub fn AdjustStyleForCombinedText(builder: &mut ComputedStyleBuilder) {
        builder.ResetTextCombine();
        builder.SetLetterSpacing(&Length::Fixed(0.0));
        builder.SetTextAlign(ETextAlign::kCenter);
        builder.SetTextDecorationLine(TextDecorationLine::kNone);
        builder.SetTextEmphasisMark(TextEmphasisMark::kNone);
        builder.SetVerticalAlign(EVerticalAlign::kMiddle);
        builder.SetWordBreak(EWordBreak::kKeepAll);
        builder.SetWordSpacing(&Length::Fixed(0.0));
        builder.SetWritingMode(WritingMode::kHorizontalTb);
        builder.SetBaseTextDecorationData(Member::default());
        builder.ResetTextIndent();
        builder.UpdateFontOrientation();
    }
    // cpp: style_adjuster.cc:477-487
    fn AdjustStyleForFirstLetter(builder: &mut ComputedStyleBuilder, parent: &ComputedStyle) {
        if builder.StyleType() != PseudoId::kPseudoIdFirstLetter {
            return;
        }
        let display = if builder.IsFloating() {
            EDisplay::kBlock
        } else {
            EDisplay::kInline
        };
        builder.SetDisplay(display);
        builder.SetContainerFont(Member::from_ptr(parent.GetFont()));
    }
    // cpp: style_adjuster.cc:489-525
    fn AdjustStyleForMarker(
        &self,
        builder: &mut ComputedStyleBuilder,
        parent: &ComputedStyle,
        parent_element: &B::Element,
    ) {
        if builder.StyleType() != PseudoId::kPseudoIdMarker {
            return;
        }
        let owner = self.backend.IsPseudoElement(parent_element).then(|| {
            self.backend
                .ParentElement(parent_element)
                .expect("marker parent pseudo has a parent")
        });
        let parent_element = owner.as_deref().unwrap_or(parent_element);
        if parent.MarkerShouldBeInside(
            self.backend.AsComputedStyleElement(parent_element),
            &builder.GetDisplayStyle(),
        ) {
            let document = self.backend.ElementDocument(parent_element);
            let (start, end) = self
                .backend
                .InlineMarginsForInside(document, builder, parent);
            let setters = PhysicalToLogical::new(builder.GetWritingDirection(), 0, 1, 2, 3);
            let mut set_margin = |side, value| match side {
                0 => builder.SetMarginTop(&Length::Fixed(value)),
                1 => builder.SetMarginRight(&Length::Fixed(value)),
                2 => builder.SetMarginBottom(&Length::Fixed(value)),
                3 => builder.SetMarginLeft(&Length::Fixed(value)),
                _ => unreachable!(),
            };
            set_margin(setters.InlineStart(), start);
            set_margin(setters.InlineEnd(), end);
        } else {
            builder.SetDisplay(EDisplay::kInlineBlock);
            builder.SetWhiteSpace(EWhiteSpace::kPre);
        }
    }
    // cpp: style_adjuster.cc:527-546
    fn AdjustSliderContainerStyle(&self, element: &B::Element, builder: &mut ComputedStyleBuilder) {
        if !IsHorizontalWritingMode(builder.GetWritingMode()) {
            builder.SetTouchAction(TouchAction::kPanX);
        } else if self.Feature(StyleAdjustmentFeature::NonStandardSliderVertical)
            && builder.Appearance() == AppearanceValue::kSliderVertical
        {
            builder.SetTouchAction(TouchAction::kPanX);
            builder.SetWritingMode(WritingMode::kVerticalRl);
            builder.SetDirection(TextDirection::kRtl);
        } else {
            builder.SetTouchAction(TouchAction::kPanY);
            builder.SetWritingMode(WritingMode::kHorizontalTb);
            let host = self
                .backend
                .OwnerShadowHost(element)
                .expect("slider has input host");
            if self.backend.InputHasDataList(&host) {
                builder.SetAlignSelf(&StyleSelfAlignmentData::new_nonlegacy(
                    ItemPosition::kCenter,
                    OverflowAlignment::kUnsafe,
                ));
            }
        }
    }
    // cpp: style_adjuster.cc:549-660
    fn AdjustStyleForHTMLElement(&self, builder: &mut ComputedStyleBuilder, element: &B::Element) {
        let b = self.backend;
        match b.ElementType(element) {
            ElementType::kHTMLImageElement => {
                if b.ImageIsCollapsed(element) || builder.Display() == EDisplay::kContents {
                    builder.SetDisplay(EDisplay::kNone);
                }
            }
            ElementType::kHTMLTableElement => {
                if matches!(
                    builder.GetTextAlign(),
                    ETextAlign::kWebkitLeft | ETextAlign::kWebkitCenter | ETextAlign::kWebkitRight
                ) {
                    builder.SetTextAlign(ETextAlign::kStart);
                }
            }
            ElementType::kHTMLFrameElement | ElementType::kHTMLFrameSetElement => {
                builder.SetPosition(EPosition::kStatic);
                builder.SetDisplay(EDisplay::kBlock);
                builder.SetFloating(EFloat::kNone);
            }
            ElementType::kHTMLFencedFrameElement => {
                builder
                    .SetEffectiveZoom(b.GetStyleResolver(b.ElementDocument(element)).InitialZoom());
            }
            ElementType::kHTMLLegendElement => {
                if builder.Display() != EDisplay::kContents {
                    let display = Self::EquivalentBlockDisplay(builder.Display());
                    builder.SetDisplay(display);
                }
            }
            ElementType::kHTMLMarqueeElement => {
                builder.SetOverflowX(EOverflow::kHidden);
                builder.SetOverflowY(EOverflow::kHidden);
            }
            ElementType::kHTMLTextAreaElement => {
                if builder.OverflowX() == EOverflow::kVisible {
                    builder.SetOverflowX(EOverflow::kAuto);
                }
                if builder.OverflowY() == EOverflow::kVisible {
                    builder.SetOverflowY(EOverflow::kAuto);
                }
                if builder.Display() == EDisplay::kContents {
                    builder.SetDisplay(EDisplay::kNone);
                }
            }
            ElementType::kHTMLEmbedElement | ElementType::kHTMLObjectElement => {
                builder.SetRequiresAcceleratedCompositingForExternalReasons(
                    b.PluginShouldAccelerate(element),
                );
                if builder.Display() == EDisplay::kContents {
                    builder.SetDisplay(EDisplay::kNone);
                }
            }
            ElementType::kHTMLBodyElement => {
                if !b
                    .FirstBodyElement(b.ElementDocument(element))
                    .is_some_and(|body| std::ptr::eq(body.as_ref(), element))
                {
                    builder.SetIsSecondaryBodyElement();
                }
            }
            ElementType::kHTMLBRElement
            | ElementType::kHTMLWBRElement
            | ElementType::kHTMLMeterElement
            | ElementType::kHTMLProgressElement
            | ElementType::kHTMLCanvasElement
            | ElementType::kHTMLAudioElement
            | ElementType::kHTMLVideoElement
            | ElementType::kHTMLInputElement
            | ElementType::kHTMLSelectElement
            | ElementType::kHTMLIFrameElement => {
                if builder.Display() == EDisplay::kContents {
                    builder.SetDisplay(EDisplay::kNone);
                }
            }
            _ => {}
        }
    }
    // cpp: style_adjuster.cc:662-747
    fn AdjustOverflow(&self, builder: &mut ComputedStyleBuilder) {
        let clip_or_visible = Self::IsOverflowClipOrVisible(builder.OverflowY())
            && Self::IsOverflowClipOrVisible(builder.OverflowX());
        if !clip_or_visible && builder.IsDisplayTable() {
            if builder.OverflowX() != EOverflow::kHidden {
                builder.SetOverflowX(EOverflow::kVisible);
            }
            if builder.OverflowY() != EOverflow::kHidden {
                builder.SetOverflowY(EOverflow::kVisible);
            }
            if builder.OverflowX() == EOverflow::kVisible {
                builder.SetOverflowY(EOverflow::kVisible);
            } else if builder.OverflowY() == EOverflow::kVisible {
                builder.SetOverflowX(EOverflow::kVisible);
            }
        } else if !Self::IsOverflowClipOrVisible(builder.OverflowY()) {
            if builder.OverflowX() == EOverflow::kVisible {
                builder.SetOverflowX(EOverflow::kAuto);
            } else if builder.OverflowX() == EOverflow::kClip
                && !self.Feature(StyleAdjustmentFeature::SingleAxisScrollContainers)
            {
                builder.SetOverflowX(EOverflow::kHidden);
            }
        } else if !Self::IsOverflowClipOrVisible(builder.OverflowX()) {
            if builder.OverflowY() == EOverflow::kVisible {
                builder.SetOverflowY(EOverflow::kAuto);
            } else if builder.OverflowY() == EOverflow::kClip
                && !self.Feature(StyleAdjustmentFeature::SingleAxisScrollContainers)
            {
                builder.SetOverflowY(EOverflow::kHidden);
            }
        }
        if builder.OverflowY() == EOverflow::kOverlay {
            builder.SetOverflowY(EOverflow::kAuto);
        }
        if builder.OverflowX() == EOverflow::kOverlay {
            builder.SetOverflowX(EOverflow::kAuto);
        }
    }
    // cpp: style_adjuster.cc:749-761
    fn IsCanvasWithLayoutSubtree(&self, element: Option<&B::Element>) -> bool {
        let Some(element) = element else {
            return false;
        };
        if !self.backend.IsCanvasOrInCanvasSubtree(element)
            || !self.backend.CanvasDrawElementEnabled(element)
        {
            return false;
        }
        self.Is(Some(element), ElementType::kHTMLCanvasElement)
            && self.backend.CanvasLayoutSubtree(element)
    }
    // cpp: style_adjuster.cc:766-775
    fn IsLayoutSubtreeCanvasChild(&self, element: Option<&B::Element>) -> bool {
        let Some(element) = element else {
            return false;
        };
        if !self.backend.IsInCanvasSubtree(element)
            || !self.backend.CanvasDrawElementEnabled(element)
        {
            return false;
        }
        let parent = self.backend.FlatTreeParentElementSkippingSlots(element);
        self.Is(parent.as_deref(), ElementType::kHTMLCanvasElement)
            && self.backend.CanvasLayoutSubtree(parent.as_deref().unwrap())
    }
    // cpp: style_adjuster.cc:777-880
    pub fn AdjustStyleForDisplay(
        &self,
        builder: &mut ComputedStyleBuilder,
        layout_parent_style: &ComputedStyle,
        element: Option<&B::Element>,
        document: Option<&B::Document>,
    ) {
        let is_immediate_canvas_child = self.IsLayoutSubtreeCanvasChild(element);
        if (layout_parent_style.BlockifiesChildren() && !self.HostIsInputFile(element))
            || is_immediate_canvas_child
        {
            builder.SetIsInBlockifyingDisplay();
            if builder.Display() != EDisplay::kContents {
                let display = Self::EquivalentBlockDisplay(builder.Display());
                builder.SetDisplay(display);
                if !builder.HasOutOfFlowPosition() {
                    builder.SetIsFlexOrGridOrCustomItem();
                }
            }
            if layout_parent_style.IsDisplayFlex()
                || layout_parent_style.IsDisplayWebkitBox()
                || layout_parent_style.IsDisplayGrid()
                || layout_parent_style.IsDisplayGridLanes()
                || layout_parent_style.IsDisplayMath()
                || is_immediate_canvas_child
            {
                builder.SetIsInsideDisplayIgnoringFloatingChildren();
            }
            if is_immediate_canvas_child {
                builder.SetPosition(EPosition::kStatic);
            }
        }
        if layout_parent_style.InlinifiesChildren()
            && !builder.HasOutOfFlowPosition()
            && self.ShouldBeInlinified(element)
            && !is_immediate_canvas_child
        {
            if builder.IsFloating() {
                builder.SetFloating(EFloat::kNone);
                if let Some(document) = document {
                    self.backend.AddRenderingInfoConsoleMessage(document, "`float` property is not supported correctly inside an element with `display: ruby` or `display: ruby-text`.", true);
                }
            }
            builder.SetIsInInlinifyingDisplay();
            let display = Self::EquivalentInlineDisplay(builder.Display());
            builder.SetDisplay(display);
        }
        if builder.StyleType() == PseudoId::kPseudoIdScrollMarkerGroup {
            let display = Self::EquivalentBlockDisplay(builder.Display());
            builder.SetDisplay(display);
        }
        if builder.Display() == EDisplay::kBlock {
            return;
        }
        if builder.Display() == EDisplay::kInline
            && builder.StyleType() == PseudoId::kPseudoIdNone
            && builder.GetWritingMode() != layout_parent_style.GetWritingMode()
        {
            builder.SetDisplay(EDisplay::kInlineBlock);
        }
        if matches!(
            builder.Display(),
            EDisplay::kTableColumn
                | EDisplay::kTableColumnGroup
                | EDisplay::kTableFooterGroup
                | EDisplay::kTableHeaderGroup
                | EDisplay::kTableRow
                | EDisplay::kTableRowGroup
        ) {
            builder.SetWritingMode(layout_parent_style.GetWritingMode());
            builder.SetTextOrientation(layout_parent_style.GetTextOrientation());
            builder.UpdateFontOrientation();
        }
        if self.IsAtMediaUAShadowBoundary(element) {
            let display = Self::EquivalentBlockDisplay(builder.Display());
            builder.SetDisplay(display);
        }
        if builder.BoxOrient() == EBoxOrient::kVertical
            && (builder.WebkitLineClamp() != 0
                || matches!(
                    builder.Continue(),
                    EContinue::kCollapse | EContinue::kWebkitLegacy
                ))
        {
            if builder.Display() == EDisplay::kWebkitBox {
                builder.SetDisplay(EDisplay::kFlowRoot);
                builder.SetIsSpecifiedDisplayWebkitBox();
            } else if builder.Display() == EDisplay::kWebkitInlineBox {
                builder.SetDisplay(EDisplay::kInlineBlock);
                builder.SetIsSpecifiedDisplayWebkitBox();
            }
        }
    }
    // cpp: style_adjuster.cc:882-901
    fn IsEditableElement(
        &self,
        element: Option<&B::Element>,
        builder: &ComputedStyleBuilder,
    ) -> bool {
        if builder.UserModify() != EUserModify::kReadOnly {
            return true;
        }
        let Some(element) = element else {
            return false;
        };
        if self.Is(Some(element), ElementType::kHTMLTextAreaElement) {
            return !self.backend.IsDisabledOrReadOnly(element);
        }
        if self.Is(Some(element), ElementType::kHTMLInputElement) {
            return !self.backend.IsDisabledOrReadOnly(element)
                && self.backend.IsTextField(element);
        }
        false
    }
    // cpp: style_adjuster.cc:903-912
    fn IsPasswordFieldWithUnrevealedPassword(&self, element: Option<&B::Element>) -> bool {
        let Some(element) = element else {
            return false;
        };
        self.Is(Some(element), ElementType::kHTMLInputElement)
            && self.backend.InputControlType(element) == InputControlType::Password
            && !self.backend.ShouldRevealPassword(element)
    }
    // cpp: style_adjuster.cc:914-1041
    fn AdjustEffectiveTouchAction(
        &self,
        builder: &mut ComputedStyleBuilder,
        parent: &ComputedStyle,
        element: Option<&B::Element>,
        is_svg_root: bool,
    ) {
        let b = self.backend;
        let mut inherited_action = parent.EffectiveTouchAction();
        let Some(element) = element else {
            builder.SetEffectiveTouchAction(TouchAction::kAuto & inherited_action);
            return;
        };
        let mut element_action = builder.GetTouchAction();
        let ignore =
            if element_action == TouchAction::kAuto || builder.IsDisplayTableRowOrColumnType() {
                true
            } else if self.Is(Some(element), ElementType::kHTMLImageElement) || is_svg_root {
                false
            } else if self.Is(Some(element), ElementType::kHTMLCanvasElement)
                && b.CanExecuteScripts(element)
            {
                false
            } else {
                builder.IsNonAtomicInlineDisplayType()
            };
        if ignore || !b.LayoutObjectIsNeeded(element, builder.GetDisplayStyle()) {
            element_action = TouchAction::kAuto;
            if inherited_action == TouchAction::kAuto
                && !self.IsDocumentElement(element)
                && !b.IsFrameOwnerElement(element)
                && !self.Feature(StyleAdjustmentFeature::SwipeToMoveCursor)
                && !self.Feature(StyleAdjustmentFeature::StylusHandwriting)
            {
                builder.SetEffectiveTouchAction(TouchAction::kAuto);
                return;
            }
        } else {
            if element_action & TouchAction::kPanX != TouchAction::kNone {
                element_action |= TouchAction::kInternalPanXScrolls;
            }
            if element_action & TouchAction::kPan != TouchAction::kNone {
                element_action |= TouchAction::kInternalNotWritable;
            }
        }
        let document = b.ElementDocument(element);
        if self.IsDocumentElement(element) {
            if let Some(frame) = b.GetFrame(document) {
                inherited_action &= TouchAction::kPan
                    | TouchAction::kInternalPanXScrolls
                    | TouchAction::kInternalNotWritable
                    | b.FrameInheritedEffectiveTouchAction(frame);
            }
        }
        inherited_action =
            self.AdjustTouchActionForElement(inherited_action, builder, parent, element);
        let policy = if b.IsVerticalScrollEnforced(document) {
            TouchAction::kPanY
        } else {
            TouchAction::kNone
        };
        if self.Feature(StyleAdjustmentFeature::SwipeToMoveCursor)
            && self.IsEditableElement(Some(element), builder)
        {
            element_action &= !TouchAction::kInternalPanXScrolls;
        }
        if self.Feature(StyleAdjustmentFeature::StylusHandwriting)
            && element_action & TouchAction::kPan == TouchAction::kPan
            && self.IsEditableElement(Some(element), builder)
            && !self.IsPasswordFieldWithUnrevealedPassword(Some(element))
        {
            element_action &= !TouchAction::kInternalNotWritable;
        }
        builder.SetEffectiveTouchAction((element_action & inherited_action) | policy);
        if b.IsFrameOwnerElement(element) {
            if let Some(frame) = b.ContentFrame(element) {
                b.SetInheritedEffectiveTouchAction(&frame, builder.EffectiveTouchAction());
            }
        }
    }
    // cpp: style_adjuster.cc:1043-1102
    fn AdjustForForcedColorsMode(
        &self,
        builder: &mut ComputedStyleBuilder,
        document: &B::Document,
    ) {
        if !builder.InForcedColorsMode() || builder.ForcedColorAdjust() != EForcedColorAdjust::kAuto
        {
            return;
        }
        builder.SetTextShadow(Member::from_ptr(
            Initial::InitialTextShadow().unwrap_or(std::ptr::null_mut()),
        ));
        builder.SetBoxShadow(Member::from_ptr(
            Initial::InitialBoxShadow().unwrap_or(std::ptr::null_mut()),
        ));
        let schemes: Vector<AtomicString> = vec![
            AtomicString::from_str("light"),
            AtomicString::from_str("dark"),
        ];
        builder.SetColorScheme(&schemes);
        builder.SetScrollbarColor(Member::from_ptr(
            Initial::InitialScrollbarColor().unwrap_or(std::ptr::null_mut()),
        ));
        // StyleAutoColor wraps the canonical StyleColor. CSS 'auto' is not a
        // system color; the source implicit conversion forces it to auto again.
        if builder.AccentColor().IsAutoColor()
            || builder.ShouldForceColor(builder.AccentColor().ToStyleColor())
        {
            builder.SetAccentColor(&Initial::InitialAccentColor());
        }
        if !builder.HasUrlBackgroundImage() {
            builder.ClearBackgroundImage();
        }
        let b = self.backend;
        let scheme = if b.GetPreferredColorScheme(b.GetStyleEngine(document))
            == PreferredColorScheme::kDark
        {
            ColorScheme::kDark
        } else {
            ColorScheme::kLight
        };
        let provider = b.ColorProviderForPainting(document, scheme);
        let can_expose = b.IsInWebAppScope(document) && b.IsInitialProfile(document);
        if builder.InternalForcedBackgroundColor().IsSystemColor() {
            let color = builder
                .InternalForcedBackgroundColor()
                .ResolveSystemColor(scheme, provider, can_expose);
            builder.SetInternalForcedBackgroundColor(&color);
        }
        let variant = builder.GetFontDescription().VariantEmoji();
        if self.Feature(StyleAdjustmentFeature::EmojiMonochromeRendering)
            && matches!(
                variant,
                FontVariantEmoji::kNormalVariantEmoji | FontVariantEmoji::kUnicodeVariantEmoji
            )
        {
            builder.SetFontVariantEmoji(FontVariantEmoji::kTextVariantEmoji);
        }
        if builder.InternalForcedColor().IsSystemColor() {
            let color = builder
                .InternalForcedColor()
                .ResolveSystemColor(scheme, provider, can_expose);
            builder.SetInternalForcedColor(&color);
        }
        if builder.InternalForcedVisitedColor().IsSystemColor() {
            let color = builder
                .InternalForcedVisitedColor()
                .ResolveSystemColor(scheme, provider, can_expose);
            builder.SetInternalForcedVisitedColor(&color);
        }
    }
    // cpp: style_adjuster.cc:1104-1133
    fn AdjustForSVGTextElement(builder: &mut ComputedStyleBuilder) {
        builder.SetColumnGap(&Initial::InitialColumnGap());
        builder.SetColumnWidthInternal(Initial::InitialColumnWidth());
        builder.SetColumnHeightInternal(Initial::InitialColumnHeight());
        builder.SetColumnRuleStyle(&Initial::InitialColumnRuleStyle());
        builder.SetColumnRuleWidthInternal(&Initial::InitialColumnRuleWidth());
        builder.SetColumnRuleColor(&Initial::InitialColumnRuleColor());
        builder
            .SetInternalVisitedColumnRuleColor(&Initial::InitialInternalVisitedColumnRuleColor());
        builder.SetColumnCountInternal(Initial::InitialColumnCount());
        builder.SetHasAutoColumnCountInternal(Initial::InitialHasAutoColumnCount());
        builder.SetHasAutoColumnWidthInternal(Initial::InitialHasAutoColumnWidth());
        builder.SetHasAutoColumnHeightInternal(Initial::InitialHasAutoColumnHeight());
        builder.ResetColumnFill();
        builder.ResetColumnWrap();
        builder.ResetColumnSpan();
    }
    // cpp: style_adjuster.cc:1135-1387
    pub fn AdjustComputedStyle(
        &self,
        state: &StyleResolverState<'_, B>,
        element: Option<&B::Element>,
    ) {
        let parent_handle = state
            .ParentStyle()
            .expect("style adjustment requires parent style");
        let layout_parent_handle = state
            .LayoutParentStyle()
            .expect("style adjustment requires layout parent style");
        let parent = unsafe { &*parent_handle.Get() };
        let layout_parent = unsafe { &*layout_parent_handle.Get() };
        let mut builder = state.StyleBuilderMut();
        let builder = &mut *builder;
        let b = self.backend;
        if let Some(element) = element.filter(|e| b.IsHTMLElement(e)) {
            if builder.Display() != EDisplay::kNone
                || b.LayoutObjectIsNeeded(element, builder.GetDisplayStyle())
            {
                self.AdjustStyleForHTMLElement(builder, element);
            }
        }
        let is_document_element = element.is_some_and(|e| self.IsDocumentElement(e));
        let mut in_top_layer = false;
        if !is_document_element {
            if let Some(element) = element {
                if self.Feature(StyleAdjustmentFeature::OverlayProperty) {
                    if builder.Overlay() == EOverlay::kAuto {
                        in_top_layer =
                            if self.Feature(StyleAdjustmentFeature::OverlayGlobalRuleRemoval) {
                                b.IsInTopLayer(element)
                            } else {
                                true
                            };
                    }
                } else {
                    in_top_layer = b.IsRenderedInTopLayer(element);
                }
            }
        }
        if builder.Display() != EDisplay::kNone {
            if in_top_layer
                || builder.StyleType() == PseudoId::kPseudoIdBackdrop
                || builder.InternalOverscrollPosition() == EInternalOverscrollPosition::kAuto
            {
                if !builder.HasOutOfFlowPosition() {
                    builder.SetPosition(EPosition::kAbsolute);
                }
                if builder.Display() == EDisplay::kContents {
                    builder.SetDisplay(EDisplay::kBlock);
                }
            }
            if is_document_element
                || (builder.Display() != EDisplay::kContents
                    && (builder.HasOutOfFlowPosition() || builder.IsFloating()))
            {
                let display = Self::EquivalentBlockDisplay(builder.Display());
                builder.SetDisplay(display);
            }
            if self.Is(element, ElementType::kMathMLTableCellElement)
                && builder.Display() == EDisplay::kTableCell
            {
                builder.SetForcesBlockifiesChildren();
            }
            if !element.is_some_and(|e| b.IsMathMLElement(e)) && builder.IsDisplayMath() {
                let display = if builder.Display() == EDisplay::kBlockMath {
                    EDisplay::kBlock
                } else {
                    EDisplay::kInline
                };
                builder.SetDisplay(display);
            }
            let originating_element = state.GetElement();
            self.AdjustStyleForMarker(builder, parent, &originating_element);
            if builder.StyleType() != PseudoId::kPseudoIdScrollMarker {
                self.AdjustStyleForDisplay(
                    builder,
                    layout_parent,
                    element,
                    element.map(|e| b.ElementDocument(e)),
                );
            }
            if builder.StyleType() == PseudoId::kPseudoIdScrollMarkerGroup {
                let mut containment =
                    builder.Contain() | Containment::kContainsLayout.value() as u32;
                if !builder.HasOutOfFlowPosition() {
                    containment |= Containment::kContainsSize.value() as u32;
                }
                builder.SetContain(containment);
            }
            if layout_parent.IsDisplayLayoutCustom() {
                builder.SetDisplayLayoutCustomParentName(layout_parent.DisplayLayoutCustomName());
            }
            if is_document_element
                && builder.HasBackdropFilter()
                && b.IsInMainFrame(b.ElementDocument(element.unwrap()))
            {
                builder.SetBackdropFilter(Initial::InitialBackdropFilter());
            }
        }
        Self::AdjustStyleForFirstLetter(builder, parent);
        builder.SetForcesStackingContext(false);
        if element.is_some_and(|e| b.CanvasForDrawingExists(e)) {
            builder.SetIsolation(EIsolation::kIsolate);
        }
        if builder.GetPosition() != EPosition::kStatic
            || Self::LayoutParentStyleForcesZIndexToCreateStackingContext(layout_parent)
        {
            builder.SetAllowsZIndex(true);
            if !builder.HasAutoZIndex() {
                builder.SetForcesStackingContext(true);
            }
        }
        let replaced_video = self.Feature(StyleAdjustmentFeature::StackingContextIsNotStacked)
            && self.Is(element, ElementType::kHTMLVideoElement)
            && builder.GetPosition() == EPosition::kStatic
            && b.HasControlsAttribute(element.unwrap());
        if is_document_element
            || replaced_video
            || self.Is(element, ElementType::kSVGForeignObjectElement)
            || in_top_layer
            || builder.StyleType() == PseudoId::kPseudoIdBackdrop
            || builder.StyleType() == PseudoId::kPseudoIdViewTransition
            || self.IsCanvasWithLayoutSubtree(element)
        {
            builder.SetForcesStackingContext(true);
        }
        if builder.OverflowX() != EOverflow::kVisible || builder.OverflowY() != EOverflow::kVisible
        {
            self.AdjustOverflow(builder);
        }
        if self.StopPropagateTextDecorations(builder, element) || state.IsForHighlight() {
            builder.SetBaseTextDecorationData(Member::default());
        } else {
            builder.SetBaseTextDecorationData(Member::from_ptr(
                layout_parent.AppliedTextDecorationData(),
            ));
        }
        if state.IsForHighlight() {
            if let Some(origin) = state.OriginatingElementStyle() {
                let origin = unsafe { &*origin.Get() };
                if builder.ColorIsCurrentColor() {
                    builder.SetColor(origin.Color());
                }
                if builder.InternalVisitedColorIsCurrentColor() {
                    builder.SetInternalVisitedColor(origin.InternalVisitedColor());
                }
            }
        }
        builder.AdjustBackgroundLayers();
        builder.AdjustMaskLayers();
        self.AdjustForForcedColorsMode(builder, state.GetDocument());
        if let Some(element) = element.filter(|e| b.IsSliderContainer(e)) {
            self.AdjustSliderContainerStyle(element, builder);
        }
        self.AdjustStyleForEditing(builder, element);
        if let Some(element) = element.filter(|e| b.IsSVGElement(e)) {
            let styled = state
                .GetStyledElement()
                .filter(|e| b.IsSVGElement(e))
                .expect("SVG style adjustment requires styled SVG owner");
            self.AdjustStyleForSvgElement(element, &styled, builder, layout_parent);
        } else if element.is_some_and(|e| b.IsMathMLElement(e))
            && builder.Display() == EDisplay::kContents
        {
            builder.SetDisplay(EDisplay::kNone);
        }
        if builder.GetPosition() == EPosition::kSticky {
            builder.SetSubtreeIsSticky(true);
        }
        if parent.JustifyItems().PositionType() == ItemPositionType::kLegacy
            && builder.JustifyItems().GetPosition() == ItemPosition::kLegacy
        {
            builder.SetJustifyItems(parent.JustifyItems());
        }
        self.AdjustEffectiveTouchAction(
            builder,
            parent,
            element,
            self.IsOutermostSVGElement(element),
        );
        if let Some(element) = element.filter(|e| b.IsInShadowTree(e)) {
            let pseudo_id = b.ShadowPseudoId(element);
            if !pseudo_id.IsNull()
                && !builder.TextOverflow().IsClip()
                && (pseudo_id == b.InputPlaceholderPseudoId()
                    || pseudo_id == b.InternalInputSuggestedPseudoId())
            {
                let host = b
                    .OwnerShadowHost(element)
                    .expect("input shadow pseudo requires text control");
                builder.SetTextOverflow(&b.ValueForTextOverflow(&host));
            }
        }
        if builder.ContentVisibility() == EContentVisibility::kAuto {
            builder.SetContainIntrinsicSizeAuto();
        }
    }
    // cpp: style_adjuster.cc:1389-1458
    pub fn RunUncacheableStyleAdjustment(
        &self,
        builder: &mut ComputedStyleBuilder,
        element: &B::Element,
        element_or_pseudo: Option<&B::Element>,
        styled_element: Option<&B::Element>,
    ) {
        let b = self.backend;
        if b.DocumentHasViewTransitions(b.ElementDocument(element)) {
            if let Some(transition) = b.GetTransition(element) {
                if std::ptr::eq(b.TransitionScope(&transition).as_ref(), element) {
                    if !self.IsDocumentElement(element) {
                        let containment =
                            builder.Contain() | Containment::kContainsLayout.value() as u32;
                        builder.SetContain(containment);
                        if b.NeedsContainmentForDurationOfCapture(&transition)
                            && self.Feature(
                                StyleAdjustmentFeature::ScopedViewTransitionSizeContainment,
                            )
                        {
                            builder.SetHasSizeContainmentForViewTransitionScope(true);
                        }
                        builder.SetViewTransitionScope(EViewTransitionScope::kAll);
                    }
                    builder.SetForcesStackingContext(true);
                }
            }
            if styled_element
                .is_some_and(|e| b.IsViewTransitionElementExcludingRootFromSupplement(e))
            {
                builder.SetElementIsViewTransitionParticipant();
            }
        }
        if builder.Appearance() == AppearanceValue::kNone || element_or_pseudo.is_none() {
            builder.SetEffectiveAppearance(AppearanceValue::kNone);
        } else {
            b.ThemeAdjustStyle(element_or_pseudo.unwrap(), builder);
        }
        let pseudo_id = b.ShadowPseudoId(element);
        let media_control_prefix: Vec<u16> = "-webkit-media-controls".encode_utf16().collect();
        let is_media_control = pseudo_id
            .utf16_units()
            .is_some_and(|s| s.starts_with(&media_control_prefix));
        if is_media_control && !builder.HasEffectiveAppearance() {
            builder.MutableBackgroundInternal().ClearImage();
        }
        if builder.HasBaseAppearance() && b.SupportsBaseAppearance(element, builder.Appearance()) {
            builder.SetInBaseAppearance(true);
        }
        if builder.InBaseAppearance()
            && !builder.HasBaseAppearance()
            && (b.SupportsBaseAppearance(element, AppearanceValue::kBase)
                || b.SupportsBaseAppearance(element, AppearanceValue::kBaseSelect))
        {
            builder.SetInBaseAppearance(false);
        }
        if b.HasCustomStyleCallbacks(element) {
            b.ElementAdjustStyle(element, builder);
        }
    }
    // cpp: style_adjuster.cc:1460-1509
    pub fn IsCacheCompatible(
        parent_a: &ComputedStyle,
        layout_parent_a: &ComputedStyle,
        parent_b: &ComputedStyle,
        layout_parent_b: &ComputedStyle,
    ) -> bool {
        if parent_a.JustifyItems() != parent_b.JustifyItems() {
            return false;
        }
        if std::ptr::eq(layout_parent_a, layout_parent_b) {
            return true;
        }
        if layout_parent_a.Display() != layout_parent_b.Display()
            || layout_parent_a.IsInBlockifyingDisplay() != layout_parent_b.IsInBlockifyingDisplay()
            || layout_parent_a.IsInInlinifyingDisplay() != layout_parent_b.IsInInlinifyingDisplay()
        {
            return false;
        }
        if layout_parent_a.GetWritingMode() != layout_parent_b.GetWritingMode() {
            return false;
        }
        if layout_parent_a.GetTextOrientation() != layout_parent_b.GetTextOrientation() {
            return false;
        }
        if layout_parent_a.CssDominantBaseline() != layout_parent_b.CssDominantBaseline() {
            return false;
        }
        ValuesEquivalent(
            layout_parent_a.AppliedTextDecorationData(),
            layout_parent_b.AppliedTextDecorationData(),
        )
    }
    // cpp: style_adjuster.cc:1511-1847
    pub fn GetElementTypeCacheKey(
        &self,
        layout_parent: &ComputedStyle,
        element: &B::Element,
    ) -> ElementTypeForCache {
        let b = self.backend;
        let uncached = ElementTypeForCache {
            element_type: ElementType::kIsNotElement,
        };
        if self.IsDocumentElement(element) {
            return uncached;
        }
        if b.HasEditContext(element) {
            return uncached;
        }
        if b.IsCanvasOrInCanvasSubtree(element) {
            return uncached;
        }
        if !b.ShadowPseudoId(element).IsNull() {
            return uncached;
        }
        if layout_parent.InlinifiesChildren() {
            return uncached;
        }
        if self.IsAtMediaUAShadowBoundary(Some(element)) {
            return uncached;
        }
        if b.HasPopoverAttribute(element) || self.Is(Some(element), ElementType::kHTMLDialogElement)
        {
            return uncached;
        }
        if b.IsInTopLayer(element) {
            return uncached;
        }
        let kind = b.ElementType(element);
        let element_type = match kind {
            ElementType::kHTMLCanvasElement => ElementType::kIsNotElement,
            ElementType::kHTMLTextAreaElement => ElementType::kIsNotElement,
            ElementType::kHTMLVideoElement => ElementType::kIsNotElement,
            ElementType::kHTMLBodyElement => {
                if !b
                    .FirstBodyElement(b.ElementDocument(element))
                    .is_some_and(|body| std::ptr::eq(body.as_ref(), element))
                {
                    return uncached;
                } else {
                    kind
                }
            }
            ElementType::kHTMLImageElement => {
                if b.ImageIsCollapsed(element) {
                    return uncached;
                } else {
                    kind
                }
            }
            ElementType::kHTMLFrameElement
            | ElementType::kHTMLIFrameElement
            | ElementType::kHTMLFencedFrameElement => ElementType::kIsNotElement,
            ElementType::kHTMLFrameSetElement => kind,
            ElementType::kHTMLInputElement => {
                if matches!(
                    b.InputControlType(element),
                    InputControlType::File | InputControlType::Password
                ) || b.IsDisabledOrReadOnly(element)
                {
                    return uncached;
                } else {
                    ElementType::kHTMLInputElement
                }
            }
            ElementType::kHTMLTableElement => kind,
            ElementType::kHTMLLegendElement => kind,
            ElementType::kHTMLMarqueeElement => kind,
            ElementType::kHTMLEmbedElement | ElementType::kHTMLObjectElement => {
                ElementType::kHTMLEmbedElement
            }
            ElementType::kHTMLBRElement
            | ElementType::kHTMLWBRElement
            | ElementType::kHTMLMeterElement
            | ElementType::kHTMLProgressElement
            | ElementType::kHTMLAudioElement
            | ElementType::kHTMLSelectElement => ElementType::kHTMLBRElement,
            ElementType::kMathMLAnchorElement
            | ElementType::kMathMLElement
            | ElementType::kMathMLFractionElement
            | ElementType::kMathMLOperatorElement
            | ElementType::kMathMLPaddedElement
            | ElementType::kMathMLRadicalElement
            | ElementType::kMathMLRowElement
            | ElementType::kMathMLScriptsElement
            | ElementType::kMathMLSpaceElement
            | ElementType::kMathMLTableCellElement
            | ElementType::kMathMLTokenElement
            | ElementType::kMathMLUnderOverElement => ElementType::kMathMLElement,
            ElementType::kSVGSVGElement => {
                if !b.IsOutermostSVGSVGElement(element) {
                    return uncached;
                } else {
                    kind
                }
            }
            ElementType::kSVGUseElement => ElementType::kIsNotElement,
            ElementType::kSVGGElement | ElementType::kSVGTSpanElement => ElementType::kSVGGElement,
            ElementType::kSVGForeignObjectElement => kind,
            ElementType::kSVGTextElement => kind,
            ElementType::kSVGAElement
            | ElementType::kSVGAnimateElement
            | ElementType::kSVGAnimateMotionElement
            | ElementType::kSVGAnimateTransformElement
            | ElementType::kSVGCircleElement
            | ElementType::kSVGClipPathElement
            | ElementType::kSVGDefsElement
            | ElementType::kSVGDescElement
            | ElementType::kSVGEllipseElement
            | ElementType::kSVGFEBlendElement
            | ElementType::kSVGFEColorMatrixElement
            | ElementType::kSVGFEComponentTransferElement
            | ElementType::kSVGFECompositeElement
            | ElementType::kSVGFEConvolveMatrixElement
            | ElementType::kSVGFEDiffuseLightingElement
            | ElementType::kSVGFEDisplacementMapElement
            | ElementType::kSVGFEDistantLightElement
            | ElementType::kSVGFEDropShadowElement
            | ElementType::kSVGFEFloodElement
            | ElementType::kSVGFEFuncAElement
            | ElementType::kSVGFEFuncBElement
            | ElementType::kSVGFEFuncGElement
            | ElementType::kSVGFEFuncRElement
            | ElementType::kSVGFEGaussianBlurElement
            | ElementType::kSVGFEImageElement
            | ElementType::kSVGFEMergeElement
            | ElementType::kSVGFEMergeNodeElement
            | ElementType::kSVGFEMorphologyElement
            | ElementType::kSVGFEOffsetElement
            | ElementType::kSVGFEPointLightElement
            | ElementType::kSVGFESpecularLightingElement
            | ElementType::kSVGFESpotLightElement
            | ElementType::kSVGFETileElement
            | ElementType::kSVGFETurbulenceElement
            | ElementType::kSVGFilterElement
            | ElementType::kSVGImageElement
            | ElementType::kSVGLinearGradientElement
            | ElementType::kSVGLineElement
            | ElementType::kSVGMarkerElement
            | ElementType::kSVGMaskElement
            | ElementType::kSVGMetadataElement
            | ElementType::kSVGMPathElement
            | ElementType::kSVGPathElement
            | ElementType::kSVGPatternElement
            | ElementType::kSVGPolygonElement
            | ElementType::kSVGPolylineElement
            | ElementType::kSVGRadialGradientElement
            | ElementType::kSVGRectElement
            | ElementType::kSVGScriptElement
            | ElementType::kSVGSetElement
            | ElementType::kSVGStopElement
            | ElementType::kSVGStyleElement
            | ElementType::kSVGSwitchElement
            | ElementType::kSVGSymbolElement
            | ElementType::kSVGTextPathElement
            | ElementType::kSVGTitleElement
            | ElementType::kSVGUnknownElement
            | ElementType::kSVGViewElement => ElementType::kSVGAElement,
            ElementType::kIsNotElement
            | ElementType::kHTMLAnchorElement
            | ElementType::kHTMLAreaElement
            | ElementType::kHTMLBaseElement
            | ElementType::kHTMLBDIElement
            | ElementType::kHTMLButtonElement
            | ElementType::kHTMLCredentialElement
            | ElementType::kHTMLDataElement
            | ElementType::kHTMLDataListElement
            | ElementType::kHTMLDetailsElement
            | ElementType::kHTMLDialogElement
            | ElementType::kHTMLDirectoryElement
            | ElementType::kHTMLDivElement
            | ElementType::kHTMLDListElement
            | ElementType::kHTMLElement
            | ElementType::kHTMLFieldSetElement
            | ElementType::kHTMLFontElement
            | ElementType::kHTMLFormElement
            | ElementType::kHTMLGeolocationElement
            | ElementType::kHTMLHeadElement
            | ElementType::kHTMLHeadingElement
            | ElementType::kHTMLHRElement
            | ElementType::kHTMLHtmlElement
            | ElementType::kHTMLInstallElement
            | ElementType::kHTMLLabelElement
            | ElementType::kHTMLLIElement
            | ElementType::kHTMLLinkElement
            | ElementType::kHTMLLoginElement
            | ElementType::kHTMLMapElement
            | ElementType::kHTMLMenuBarElement
            | ElementType::kHTMLMenuElement
            | ElementType::kHTMLMenuItemElement
            | ElementType::kHTMLMenuListElement
            | ElementType::kHTMLMetaElement
            | ElementType::kHTMLModElement
            | ElementType::kHTMLNoEmbedElement
            | ElementType::kHTMLNoScriptElement
            | ElementType::kHTMLOListElement
            | ElementType::kHTMLOptGroupElement
            | ElementType::kHTMLOptionElement
            | ElementType::kHTMLOutputElement
            | ElementType::kHTMLParagraphElement
            | ElementType::kHTMLParamElement
            | ElementType::kHTMLPictureElement
            | ElementType::kHTMLPreElement
            | ElementType::kHTMLQuoteElement
            | ElementType::kHTMLScriptElement
            | ElementType::kHTMLSearchElement
            | ElementType::kHTMLSelectedContentElement
            | ElementType::kHTMLSlotElement
            | ElementType::kHTMLSourceElement
            | ElementType::kHTMLSpanElement
            | ElementType::kHTMLStyleElement
            | ElementType::kHTMLSubMenuElement
            | ElementType::kHTMLSummaryElement
            | ElementType::kHTMLTableCaptionElement
            | ElementType::kHTMLTableCellElement
            | ElementType::kHTMLTableColElement
            | ElementType::kHTMLTableRowElement
            | ElementType::kHTMLTableSectionElement
            | ElementType::kHTMLTemplateElement
            | ElementType::kHTMLTimeElement
            | ElementType::kHTMLTitleElement
            | ElementType::kHTMLTrackElement
            | ElementType::kHTMLUListElement
            | ElementType::kHTMLUnknownElement
            | ElementType::kHTMLUserMediaElement
            | ElementType::kHTMLCameraElement
            | ElementType::kHTMLMicrophoneElement => ElementType::kHTMLDivElement,
        };
        ElementTypeForCache { element_type }
    }
}
