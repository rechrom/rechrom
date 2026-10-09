//! Translation of Chromium resolver/css_to_style_map.{h,cc}.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! css_to_style_map.h: 154 / 109 / 101 / 8 / 0.
//! css_to_style_map.cc: 868 / 577 / 567 / 10 / 0.
//! Effective excludes blank/comments/copyright/preprocessor/includes/namespace
//! and lines containing only brackets/punctuation. Header omissions: forward
//! declarations 37-42, STATIC_ONLY 45, access label 47. cc omissions: pure debug
//! assertions 333,353-354,374,394,408-409,466,743,745. All other production lines
//! in cc:61-866 map: FillLayer 61-238; animation/timing/timeline 242-535;
//! NinePieceImage plus source-local helpers 537-732; trigger/attachments 734-866.
//! This source version contains no font/list/transform/grid Apply functions.
//! CSSLengthResolver overload is named MapAnimationTimingFunctionWithResolver.
//! The existing resolver conversion-data type supplies that required interface.
//! Missing concrete CSSValue getters and animation/timeline owner constructors
//! are required typed operations without defaults; this module retains every
//! map branch, side-effect order, recursion/delegation and builder mutation.
#![allow(non_snake_case)]
use super::style_resolver_state::{ResolverValue, StyleResolverState, StyleResolverStateBackend};
use crate::css_value::{CSSValueDispatch, CSSValueListSubclass, CSSValuePayload, CSSValueSubclass};
use foundation::scoped_css_name::TreeScope;
use foundation::{
    AtomicString, BlendMode, CSSPropertyID, CSSValueID, Length, LengthBox, LengthSize,
    ScopedCSSName,
};
use layoutng_style::style::border_image_length::BorderImageLength;
use layoutng_style::style::border_image_length_box::BorderImageLengthBox;
use layoutng_style::style::computed_style_constants::{
    BackgroundEdgeOrigin, CompositingOperator, EAnimPlayState, EAnimationTriggerBehavior,
    EFillAttachment, EFillBox, EFillLayerType, EFillMaskMode, EFillRepeat, EFillSizeType,
    TimelineAxis, TimelineScroller,
};
use layoutng_style::style::fill_layer::{FillLayer, FillRepeat};
use layoutng_style::style::nine_piece_image::{ENinePieceImageRule, NinePieceImage};
use layoutng_style::style::style_image::StyleImage;
use layoutng_style::style::timeline_inset::TimelineInset;
use std::marker::PhantomData;
use std::rc::Rc;

type Dispatch<B> = <B as StyleResolverStateBackend>::ValueDispatch;
type Identifier<B> = <Dispatch<B> as CSSValueDispatch>::CSSIdentifierValue;
type CustomIdent<B> = <Dispatch<B> as CSSValueDispatch>::CSSCustomIdentValue;
type StringValue<B> = <Dispatch<B> as CSSValueDispatch>::CSSStringValue;
type Pair<B> = <Dispatch<B> as CSSValueDispatch>::CSSValuePair;
type Quad<B> = <Dispatch<B> as CSSValueDispatch>::CSSQuadValue;
type List<B> = <Dispatch<B> as CSSValueDispatch>::CSSValueList;
type Repeat<B> = <Dispatch<B> as CSSValueDispatch>::CSSRepeatStyleValue;
type Slice<B> = <Dispatch<B> as CSSValueDispatch>::CSSBorderImageSliceValue;
type View<B> = <Dispatch<B> as CSSValueDispatch>::CSSViewValue;
type Scroll<B> = <Dispatch<B> as CSSValueDispatch>::CSSScrollValue;
type Linear<B> = <Dispatch<B> as CSSValueDispatch>::CSSLinearTimingFunctionValue;
type Cubic<B> = <Dispatch<B> as CSSValueDispatch>::CSSCubicBezierTimingFunctionValue;
type Steps<B> = <Dispatch<B> as CSSValueDispatch>::CSSStepsTimingFunctionValue;
type Attachment<B> = <Dispatch<B> as CSSValueDispatch>::CSSTriggerAttachmentValue;

/// Required operations of untranslated value subclasses, StyleBuilderConverter
/// and animation/timeline owners. No fallback value, style or animation model is
/// supplied. All CSSToStyleMap decisions and builder mutations stay below.
pub trait CSSToStyleMapBackend: StyleResolverStateBackend {
    type TimingDelay;
    type PlaybackDirection;
    type TimingFillMode;
    type TransitionBehavior;
    type CompositeOperation;
    type TransitionProperty;
    type TimingFunction;
    type LinearTimingPoint;
    type StepPosition: PartialEq;
    type StyleTimeline;
    type TimelineNamedRange;
    type TimelineOffset;
    type TimelineOffsetOrAuto;
    type StyleTriggerAttachment;
    type StyleTriggerAttachmentVector;

    fn IdentifierID(&self, value: &Identifier<Self>) -> CSSValueID;
    fn IdentifierFillAttachment(&self, value: &Identifier<Self>) -> EFillAttachment;
    fn IdentifierFillBox(&self, value: &Identifier<Self>) -> EFillBox;
    fn IdentifierCompositingOperator(&self, value: &Identifier<Self>) -> CompositingOperator;
    fn IdentifierBlendMode(&self, value: &Identifier<Self>) -> BlendMode;
    fn IdentifierFillRepeat(&self, value: &Identifier<Self>) -> EFillRepeat;
    fn IdentifierFillMaskMode(&self, value: &Identifier<Self>) -> EFillMaskMode;
    fn IdentifierBackgroundOrigin(&self, value: &Identifier<Self>) -> BackgroundEdgeOrigin;
    fn IdentifierTimelineAxis(&self, value: &Identifier<Self>) -> TimelineAxis;
    fn IdentifierTimelineScroller(&self, value: &Identifier<Self>) -> TimelineScroller;
    fn IdentifierNamedRange(&self, value: &Identifier<Self>) -> Self::TimelineNamedRange;
    fn IdentifierTriggerBehavior(&self, value: &Identifier<Self>) -> EAnimationTriggerBehavior;
    fn PairValues(&self, value: &Pair<Self>) -> (Rc<ResolverValue<Self>>, Rc<ResolverValue<Self>>);
    fn QuadValues(
        &self,
        value: &Quad<Self>,
    ) -> (
        Rc<ResolverValue<Self>>,
        Rc<ResolverValue<Self>>,
        Rc<ResolverValue<Self>>,
        Rc<ResolverValue<Self>>,
    );
    fn ListValues(&self, value: &List<Self>) -> Vec<Rc<ResolverValue<Self>>>;
    fn RepeatValues(
        &self,
        value: &Repeat<Self>,
    ) -> (Rc<ResolverValue<Self>>, Rc<ResolverValue<Self>>);
    fn SliceQuad(&self, value: &Slice<Self>) -> Rc<Quad<Self>>;
    fn SliceFill(&self, value: &Slice<Self>) -> bool;
    fn ViewAxis(&self, value: &View<Self>) -> Option<Rc<ResolverValue<Self>>>;
    fn ViewInset(&self, value: &View<Self>) -> Option<Rc<ResolverValue<Self>>>;
    fn ScrollAxis(&self, value: &Scroll<Self>) -> Option<Rc<ResolverValue<Self>>>;
    fn ScrollScroller(&self, value: &Scroll<Self>) -> Option<Rc<ResolverValue<Self>>>;
    fn CustomIdentValue(&self, value: &CustomIdent<Self>) -> AtomicString;
    fn CustomIdentTreeScope(&self, value: &CustomIdent<Self>) -> *const TreeScope;
    fn CustomIdentComputeIdent(
        &self,
        value: &CustomIdent<Self>,
        resolver: &Self::LengthConversionData,
    ) -> AtomicString;
    fn CustomIdentIsKnownProperty(&self, value: &CustomIdent<Self>) -> bool;
    fn CustomIdentPropertyID(&self, value: &CustomIdent<Self>) -> CSSPropertyID;
    fn StringValueText(&self, value: &StringValue<Self>) -> AtomicString;
    fn DocumentTreeScope(&self, document: &Self::Document) -> *const TreeScope;
    fn PrimitiveSeconds(
        &self,
        value: &ResolverValue<Self>,
        resolver: &Self::LengthConversionData,
    ) -> f64;
    fn PrimitiveNumber(
        &self,
        value: &ResolverValue<Self>,
        resolver: &Self::LengthConversionData,
    ) -> f64;
    fn PrimitiveInteger(
        &self,
        value: &ResolverValue<Self>,
        resolver: &Self::LengthConversionData,
    ) -> i32;
    fn PrimitivePercentage(
        &self,
        value: &ResolverValue<Self>,
        resolver: &Self::LengthConversionData,
    ) -> f64;
    fn PrimitiveIsPercentage(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveIsNumber(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveHasUnresolvablePercentages(&self, value: &ResolverValue<Self>) -> bool;
    fn PrimitiveToLength(
        &self,
        value: &ResolverValue<Self>,
        resolver: &Self::LengthConversionData,
    ) -> Length;
    fn ConvertLength(
        &self,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
    ) -> Length;
    fn ConvertLengthOrAuto(
        &self,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
    ) -> Length;
    fn ConvertPositionLength(
        &self,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
        start: CSSValueID,
        end: CSSValueID,
    ) -> Length;
    fn ConvertSingleTimelineInset(
        &self,
        state: &StyleResolverState<'_, Self>,
        value: &ResolverValue<Self>,
    ) -> TimelineInset;
    /// Ownership-only bridge: roots this exact image for the built native style,
    /// including after the resolver state dies. A FillLayer may already be
    /// borrowed from the builder, so this operation must not reborrow that
    /// builder's RefCell or mutate layer/property data.
    fn RetainMappedStyleImage(
        &self,
        state: &StyleResolverState<'_, Self>,
        image: Rc<StyleImage>,
    ) -> *mut StyleImage;
    fn NewTimingDelaySeconds(&self, seconds: f64) -> Self::TimingDelay;
    fn DefaultTimingDelay(&self) -> Self::TimingDelay;
    fn DirectionNormal(&self) -> Self::PlaybackDirection;
    fn DirectionAlternateNormal(&self) -> Self::PlaybackDirection;
    fn DirectionReverse(&self) -> Self::PlaybackDirection;
    fn DirectionAlternateReverse(&self) -> Self::PlaybackDirection;
    fn FillModeNone(&self) -> Self::TimingFillMode;
    fn FillModeForwards(&self) -> Self::TimingFillMode;
    fn FillModeBackwards(&self) -> Self::TimingFillMode;
    fn FillModeBoth(&self) -> Self::TimingFillMode;
    fn InitialAnimationName(&self) -> Option<Rc<ScopedCSSName>>;
    fn TransitionBehaviorNormal(&self) -> Self::TransitionBehavior;
    fn TransitionBehaviorAllowDiscrete(&self) -> Self::TransitionBehavior;
    fn NewKeywordTimeline(&self, keyword: CSSValueID) -> Self::StyleTimeline;
    fn NewNamedTimeline(&self, name: AtomicString) -> Self::StyleTimeline;
    fn NewViewTimeline(&self, axis: TimelineAxis, inset: TimelineInset) -> Self::StyleTimeline;
    fn NewScrollTimeline(
        &self,
        axis: TimelineAxis,
        scroller: TimelineScroller,
    ) -> Self::StyleTimeline;
    fn ViewDefaultAxis(&self) -> TimelineAxis;
    fn ScrollDefaultAxis(&self) -> TimelineAxis;
    fn ScrollDefaultScroller(&self) -> TimelineScroller;
    fn NamedRangeNone(&self) -> Self::TimelineNamedRange;
    fn NewTimelineOffset(
        &self,
        name: Self::TimelineNamedRange,
        offset: Length,
    ) -> Self::TimelineOffset;
    fn DefaultTimelineOffsetOrAuto(&self) -> Self::TimelineOffsetOrAuto;
    fn NewTimelineOffsetOrAuto(
        &self,
        offset: Option<Self::TimelineOffset>,
    ) -> Self::TimelineOffsetOrAuto;
    fn CompositeAdd(&self) -> Self::CompositeOperation;
    fn CompositeAccumulate(&self) -> Self::CompositeOperation;
    fn CompositeReplace(&self) -> Self::CompositeOperation;
    fn NewTransitionPropertyID(&self, id: CSSPropertyID) -> Self::TransitionProperty;
    fn NewTransitionPropertyName(&self, name: AtomicString) -> Self::TransitionProperty;
    fn InitialTransitionProperty(&self) -> Self::TransitionProperty;
    fn TransitionPropertyNone(&self) -> Self::TransitionProperty;
    fn LinearTimingShared(&self) -> Rc<Self::TimingFunction>;
    fn CubicTimingEase(&self) -> Rc<Self::TimingFunction>;
    fn CubicTimingEaseIn(&self) -> Rc<Self::TimingFunction>;
    fn CubicTimingEaseOut(&self) -> Rc<Self::TimingFunction>;
    fn CubicTimingEaseInOut(&self) -> Rc<Self::TimingFunction>;
    fn StepsTimingStart(&self) -> Rc<Self::TimingFunction>;
    fn StepsTimingEnd(&self) -> Rc<Self::TimingFunction>;
    fn LinearTimingPoints(&self, value: &Linear<Self>) -> Vec<Self::LinearTimingPoint>;
    fn NewLinearTiming(&self, points: Vec<Self::LinearTimingPoint>) -> Rc<Self::TimingFunction>;
    fn CubicTimingCoordinates(&self, value: &Cubic<Self>) -> (f64, f64, f64, f64);
    fn NewCubicTiming(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> Rc<Self::TimingFunction>;
    fn StepsTimingNumber(&self, value: &Steps<Self>) -> Rc<ResolverValue<Self>>;
    fn StepsTimingPosition(&self, value: &Steps<Self>) -> Self::StepPosition;
    fn StepsJumpNonePosition(&self) -> Self::StepPosition;
    fn NewStepsTiming(&self, steps: i32, position: Self::StepPosition) -> Rc<Self::TimingFunction>;
    fn AttachmentTriggerName(&self, value: &Attachment<Self>) -> Rc<ResolverValue<Self>>;
    fn AttachmentEnterBehavior(&self, value: &Attachment<Self>) -> Rc<ResolverValue<Self>>;
    fn AttachmentExitBehavior(&self, value: &Attachment<Self>) -> Option<Rc<ResolverValue<Self>>>;
    fn NewStyleTriggerAttachment(
        &self,
        name: Rc<ScopedCSSName>,
        enter: EAnimationTriggerBehavior,
        exit: Option<EAnimationTriggerBehavior>,
    ) -> Rc<Self::StyleTriggerAttachment>;
    fn NewStyleTriggerAttachmentVector(
        &self,
        attachments: Vec<Rc<Self::StyleTriggerAttachment>>,
    ) -> Rc<Self::StyleTriggerAttachmentVector>;
}

// Shared source-local native conversion, css_to_style_map.cc:617-627.
pub(crate) fn BorderImageSliceSideFromResolved(value: f64, percentage: bool) -> Length {
    if percentage { Length::Percent(value) } else { Length::Fixed(value.round()) }
}
// css_to_style_map.cc:669-681, with already-converted real native sides.
pub(crate) fn NinePieceImageQuadFromSides(sides: &[BorderImageLength; 4]) -> BorderImageLengthBox {
    BorderImageLengthBox::new(&sides[0], &sides[1], &sides[2], &sides[3])
}
// css_to_style_map.cc:684-732. Both generic and production dispatch share it.
pub(crate) fn MapNinePieceImageRepeatFromIdentifiers(first: CSSValueID, second: CSSValueID, image: &mut NinePieceImage) {
    let rule = |identifier| match identifier {
        CSSValueID::kStretch => ENinePieceImageRule::kStretchImageRule,
        CSSValueID::kRound => ENinePieceImageRule::kRoundImageRule,
        CSSValueID::kSpace => ENinePieceImageRule::kSpaceImageRule,
        _ => ENinePieceImageRule::kRepeatImageRule,
    };
    image.SetHorizontalRule(rule(first)); image.SetVerticalRule(rule(second));
}
// css_to_style_map.cc:591-615. Preserve legacy fixed-width side effects.
pub(crate) fn ApplyLegacyBorderImageWidths(image: &NinePieceImage, builder: &mut layoutng_style::style::computed_style::ComputedStyleBuilder) {
    let slices = image.BorderSlices();
    if slices.Top().IsLength() && slices.Top().length().IsFixed() { builder.SetBorderTopWidthOwned(slices.Top().length().Pixels() as i32); }
    if slices.Right().IsLength() && slices.Right().length().IsFixed() { builder.SetBorderRightWidthOwned(slices.Right().length().Pixels() as i32); }
    if slices.Bottom().IsLength() && slices.Bottom().length().IsFixed() { builder.SetBorderBottomWidthOwned(slices.Bottom().length().Pixels() as i32); }
    if slices.Left().IsLength() && slices.Left().length().IsFixed() { builder.SetBorderLeftWidthOwned(slices.Left().length().Pixels() as i32); }
}
pub struct CSSToStyleMap<B: CSSToStyleMapBackend>(PhantomData<fn() -> B>);
impl<B: CSSToStyleMapBackend> CSSToStyleMap<B> {
    fn Identifier(value: &ResolverValue<B>) -> &Identifier<B> {
        let CSSValuePayload::kIdentifierClass(identifier) = value.Payload() else {
            panic!("CSSIdentifierValue required")
        };
        identifier
    }
    fn ID(backend: &B, value: &ResolverValue<B>) -> CSSValueID {
        backend.IdentifierID(Self::Identifier(value))
    }
    pub fn MapFillAttachment(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetAttachment(FillLayer::InitialFillAttachment(layer.GetType()));
            return;
        }
        layer.SetAttachment(backend.IdentifierFillAttachment(Self::Identifier(value)));
    }
    pub fn MapFillCompositingOperator(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer
                .SetCompositingOperator(FillLayer::InitialFillCompositingOperator(layer.GetType()));
            return;
        }
        layer
            .SetCompositingOperator(backend.IdentifierCompositingOperator(Self::Identifier(value)));
    }
    pub fn MapFillBlendMode(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetBlendMode(FillLayer::InitialFillBlendMode(layer.GetType()));
            return;
        }
        layer.SetBlendMode(backend.IdentifierBlendMode(Self::Identifier(value)));
    }
    pub fn MapFillOrigin(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetOrigin(FillLayer::InitialFillOrigin(layer.GetType()));
            return;
        }
        layer.SetOrigin(backend.IdentifierFillBox(Self::Identifier(value)));
    }
    pub fn MapFillMaskMode(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetMaskMode(FillLayer::InitialFillMaskMode(layer.GetType()));
            return;
        }
        layer.SetMaskMode(backend.IdentifierFillMaskMode(Self::Identifier(value)));
    }
    pub fn MapFillClip(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetClip(FillLayer::InitialFillClip(layer.GetType()));
            return;
        }
        if value.IsValuePair() {
            layer.SetClip(EFillBox::kBorderAreaText);
            return;
        }
        layer.SetClip(backend.IdentifierFillBox(Self::Identifier(value)));
    }
    fn StyleImagePointer(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        property: CSSPropertyID,
        value: Rc<ResolverValue<B>>,
    ) -> *mut StyleImage {
        state
            .GetStyleImage(property, value)
            .map_or(std::ptr::null_mut(), |image| {
                backend.RetainMappedStyleImage(state, image)
            })
    }
    pub fn MapFillImage(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: Rc<ResolverValue<B>>,
    ) {
        if value.IsInitialValue() {
            layer.SetImage(FillLayer::InitialFillImage(layer.GetType()));
            return;
        }
        let property = if layer.GetType() == EFillLayerType::kBackground {
            CSSPropertyID::kBackgroundImage
        } else {
            CSSPropertyID::kMaskImage
        };
        layer.SetImage(Self::StyleImagePointer(backend, state, property, value));
    }
    pub fn MapFillRepeat(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetRepeat(&FillLayer::InitialFillRepeat(layer.GetType()));
            return;
        }
        let CSSValuePayload::kRepeatStyleClass(repeat) = value.Payload() else {
            panic!("CSSRepeatStyleValue required")
        };
        let (x, y) = backend.RepeatValues(repeat);
        layer.SetRepeat(&FillRepeat {
            x: backend.IdentifierFillRepeat(Self::Identifier(&x)),
            y: backend.IdentifierFillRepeat(Self::Identifier(&y)),
        });
    }
    pub fn MapFillSize(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetSizeType(FillLayer::InitialFillSizeType(layer.GetType()));
            layer.SetSizeLength(&FillLayer::InitialFillSizeLength(layer.GetType()));
            return;
        }
        let size_type = if value.IsIdentifierValue() {
            match Self::ID(backend, value) {
                CSSValueID::kContain => EFillSizeType::kContain,
                CSSValueID::kCover => EFillSizeType::kCover,
                _ => EFillSizeType::kSizeLength,
            }
        } else {
            EFillSizeType::kSizeLength
        };
        layer.SetSizeType(size_type);
        if size_type != EFillSizeType::kSizeLength {
            layer.SetSizeLength(&FillLayer::InitialFillSizeLength(layer.GetType()));
            return;
        }
        let first;
        let mut second = Length::default();
        if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
            let (a, b) = backend.PairValues(pair);
            first = backend.ConvertLengthOrAuto(state, &a);
            second = backend.ConvertLengthOrAuto(state, &b);
        } else {
            first = backend.ConvertLengthOrAuto(state, value);
        }
        layer.SetSizeLength(&LengthSize::new(&first, &second));
    }
    pub fn MapFillPositionX(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetPositionX(&FillLayer::InitialFillPositionX(layer.GetType()));
            return;
        }
        if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
            let (first, second) = backend.PairValues(pair);
            layer.SetPositionX(
                &backend.PrimitiveToLength(&second, &state.CssToLengthConversionData()),
            );
            layer
                .SetBackgroundXOrigin(backend.IdentifierBackgroundOrigin(Self::Identifier(&first)));
        } else {
            layer.SetPositionX(&backend.ConvertPositionLength(
                state,
                value,
                CSSValueID::kLeft,
                CSSValueID::kRight,
            ));
        }
    }
    pub fn MapFillPositionY(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        layer: &mut FillLayer,
        value: &ResolverValue<B>,
    ) {
        if value.IsInitialValue() {
            layer.SetPositionY(&FillLayer::InitialFillPositionY(layer.GetType()));
            return;
        }
        if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
            let (first, second) = backend.PairValues(pair);
            layer.SetPositionY(
                &backend.PrimitiveToLength(&second, &state.CssToLengthConversionData()),
            );
            layer
                .SetBackgroundYOrigin(backend.IdentifierBackgroundOrigin(Self::Identifier(&first)));
        } else {
            layer.SetPositionY(&backend.ConvertPositionLength(
                state,
                value,
                CSSValueID::kTop,
                CSSValueID::kBottom,
            ));
        }
    }
    fn List(backend: &B, value: &ResolverValue<B>) -> Option<Vec<Rc<ResolverValue<B>>>> {
        if !value.IsValueList() {
            return None;
        }
        let list = match value.Payload() {
            CSSValuePayload::kValueListClass(list) => list,
            CSSValuePayload::kFunctionClass(value) => value.AsValueList(),
            CSSValuePayload::kImageSetClass(value) => value.AsValueList(),
            CSSValuePayload::kGridLineNamesClass(value) => value.AsValueList(),
            CSSValuePayload::kGridAutoRepeatClass(value) => value.AsValueList(),
            CSSValuePayload::kGridIntegerRepeatClass(value) => value.AsValueList(),
            CSSValuePayload::kAxisClass(value) => value.AsValueList(),
            _ => unreachable!("CSSValueList class range"),
        };
        Some(backend.ListValues(list))
    }
    fn MapAnimationTimingDelay(
        backend: &B,
        resolver: &B::LengthConversionData,
        value: &ResolverValue<B>,
    ) -> B::TimingDelay {
        if value.IsPrimitiveValue() {
            return backend.NewTimingDelaySeconds(backend.PrimitiveSeconds(value, resolver));
        }
        backend.DefaultTimingDelay()
    }
    pub fn MapAnimationDelayStart(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TimingDelay {
        Self::MapAnimationTimingDelay(backend, &state.CssToLengthConversionData(), value)
    }
    pub fn MapAnimationDelayEnd(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TimingDelay {
        Self::MapAnimationTimingDelay(backend, &state.CssToLengthConversionData(), value)
    }
    pub fn MapAnimationDirection(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::PlaybackDirection {
        match Self::ID(backend, value) {
            CSSValueID::kNormal => backend.DirectionNormal(),
            CSSValueID::kAlternate => backend.DirectionAlternateNormal(),
            CSSValueID::kReverse => backend.DirectionReverse(),
            CSSValueID::kAlternateReverse => backend.DirectionAlternateReverse(),
            _ => unreachable!("invalid animation-direction"),
        }
    }
    pub fn MapAnimationDuration(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<f64> {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kAuto {
            return None;
        }
        Some(backend.PrimitiveSeconds(value, &state.CssToLengthConversionData()))
    }
    pub fn MapAnimationFillMode(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TimingFillMode {
        match Self::ID(backend, value) {
            CSSValueID::kNone => backend.FillModeNone(),
            CSSValueID::kForwards => backend.FillModeForwards(),
            CSSValueID::kBackwards => backend.FillModeBackwards(),
            CSSValueID::kBoth => backend.FillModeBoth(),
            _ => unreachable!("invalid animation-fill-mode"),
        }
    }
    pub fn MapAnimationIterationCount(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> f64 {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kInfinite {
            return f64::INFINITY;
        }
        backend.PrimitiveNumber(value, &state.CssToLengthConversionData())
    }
    pub fn MapAnimationName(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<Rc<ScopedCSSName>> {
        if let CSSValuePayload::kCustomIdentClass(ident) = value.Payload() {
            state.SetHasTreeScopedReference();
            return Some(Rc::new(ScopedCSSName::new(
                &backend.CustomIdentValue(ident),
                backend.CustomIdentTreeScope(ident),
            )));
        }
        if let CSSValuePayload::kStringClass(string) = value.Payload() {
            state.SetHasTreeScopedReference();
            return Some(Rc::new(ScopedCSSName::new(
                &backend.StringValueText(string),
                backend.DocumentTreeScope(state.GetDocument()),
            )));
        }
        backend.InitialAnimationName()
    }
    pub fn MapAnimationBehavior(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TransitionBehavior {
        match Self::ID(backend, value) {
            CSSValueID::kNormal => backend.TransitionBehaviorNormal(),
            CSSValueID::kAllowDiscrete => backend.TransitionBehaviorAllowDiscrete(),
            _ => unreachable!("invalid transition-behavior"),
        }
    }
    pub fn MapAnimationTimeline(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::StyleTimeline {
        if value.IsIdentifierValue() {
            return backend.NewKeywordTimeline(Self::ID(backend, value));
        }
        if let CSSValuePayload::kCustomIdentClass(ident) = value.Payload() {
            return backend.NewNamedTimeline(
                backend.CustomIdentComputeIdent(ident, &state.CssToLengthConversionData()),
            );
        }
        if let CSSValuePayload::kViewClass(view) = value.Payload() {
            let axis = backend
                .ViewAxis(view)
                .filter(|value| value.IsIdentifierValue())
                .map_or_else(
                    || backend.ViewDefaultAxis(),
                    |value| backend.IdentifierTimelineAxis(Self::Identifier(&value)),
                );
            let inset = backend
                .ViewInset(view)
                .map_or_else(TimelineInset::default, |value| {
                    backend.ConvertSingleTimelineInset(state, &value)
                });
            return backend.NewViewTimeline(axis, inset);
        }
        let CSSValuePayload::kScrollClass(scroll) = value.Payload() else {
            panic!("CSSScrollValue required")
        };
        let axis_value = backend
            .ScrollAxis(scroll)
            .filter(|value| value.IsIdentifierValue());
        let scroller_value = backend
            .ScrollScroller(scroll)
            .filter(|value| value.IsIdentifierValue());
        let axis = axis_value.map_or_else(
            || backend.ScrollDefaultAxis(),
            |value| backend.IdentifierTimelineAxis(Self::Identifier(&value)),
        );
        let scroller = scroller_value.map_or_else(
            || backend.ScrollDefaultScroller(),
            |value| backend.IdentifierTimelineScroller(Self::Identifier(&value)),
        );
        backend.NewScrollTimeline(axis, scroller)
    }
    pub fn MapAnimationPlayState(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> EAnimPlayState {
        if Self::ID(backend, value) == CSSValueID::kPaused {
            EAnimPlayState::kPaused
        } else {
            EAnimPlayState::kPlaying
        }
    }
    fn MapAnimationRange(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        default_percent: f64,
    ) -> Option<B::TimelineOffset> {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kNormal {
            return None;
        }
        let list = Self::List(backend, value).expect("CSSValueList required for animation range");
        let mut name = backend.NamedRangeNone();
        let mut offset = Length::Percent(default_percent);
        if list[0].IsIdentifierValue() {
            name = backend.IdentifierNamedRange(Self::Identifier(&list[0]));
            if list.len() == 2 {
                offset = backend.ConvertLength(state, &list[1]);
            }
        } else {
            offset = backend.ConvertLength(state, &list[0]);
        }
        Some(backend.NewTimelineOffset(name, offset))
    }
    pub fn MapAnimationRangeStart(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<B::TimelineOffset> {
        Self::MapAnimationRange(backend, state, value, 0.0)
    }
    pub fn MapAnimationRangeEnd(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<B::TimelineOffset> {
        Self::MapAnimationRange(backend, state, value, 100.0)
    }
    pub fn MapAnimationComposition(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::CompositeOperation {
        match Self::ID(backend, value) {
            CSSValueID::kAdd => backend.CompositeAdd(),
            CSSValueID::kAccumulate => backend.CompositeAccumulate(),
            _ => backend.CompositeReplace(),
        }
    }
    pub fn MapAnimationProperty(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TransitionProperty {
        if let CSSValuePayload::kCustomIdentClass(ident) = value.Payload() {
            if backend.CustomIdentIsKnownProperty(ident) {
                return backend.NewTransitionPropertyID(backend.CustomIdentPropertyID(ident));
            }
            return backend.NewTransitionPropertyName(backend.CustomIdentValue(ident));
        }
        if Self::ID(backend, value) == CSSValueID::kAll {
            return backend.InitialTransitionProperty();
        }
        backend.TransitionPropertyNone()
    }
    /// CSSLengthResolver overload; the resolver's real conversion-data owner
    /// provides the source primitive computations through required operations.
    pub fn MapAnimationTimingFunctionWithResolver(
        backend: &B,
        resolver: &B::LengthConversionData,
        value: &ResolverValue<B>,
    ) -> Rc<B::TimingFunction> {
        if value.IsIdentifierValue() {
            return match Self::ID(backend, value) {
                CSSValueID::kLinear => backend.LinearTimingShared(),
                CSSValueID::kEase => backend.CubicTimingEase(),
                CSSValueID::kEaseIn => backend.CubicTimingEaseIn(),
                CSSValueID::kEaseOut => backend.CubicTimingEaseOut(),
                CSSValueID::kEaseInOut => backend.CubicTimingEaseInOut(),
                CSSValueID::kStepStart => backend.StepsTimingStart(),
                CSSValueID::kStepEnd => backend.StepsTimingEnd(),
                _ => unreachable!("invalid animation timing keyword"),
            };
        }
        if let CSSValuePayload::kLinearTimingFunctionClass(linear) = value.Payload() {
            return backend.NewLinearTiming(backend.LinearTimingPoints(linear));
        }
        if let CSSValuePayload::kCubicBezierTimingFunctionClass(cubic) = value.Payload() {
            let (x1, y1, x2, y2) = backend.CubicTimingCoordinates(cubic);
            return backend.NewCubicTiming(x1, y1, x2, y2);
        }
        let CSSValuePayload::kStepsTimingFunctionClass(steps_value) = value.Payload() else {
            panic!("CSSStepsTimingFunctionValue required")
        };
        let number = backend.StepsTimingNumber(steps_value);
        let mut steps = backend.PrimitiveInteger(&number, resolver);
        if backend.StepsTimingPosition(steps_value) == backend.StepsJumpNonePosition() && steps < 2
        {
            steps = 2;
        }
        backend.NewStepsTiming(steps, backend.StepsTimingPosition(steps_value))
    }
    pub fn MapAnimationTimingFunction(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Rc<B::TimingFunction> {
        Self::MapAnimationTimingFunctionWithResolver(
            backend,
            &state.CssToLengthConversionData(),
            value,
        )
    }
    pub fn MapNinePieceImage(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        property: CSSPropertyID,
        value: &ResolverValue<B>,
        image: &mut NinePieceImage,
    ) {
        let Some(border_image) = Self::List(backend, value) else {
            return;
        };
        let image_property = match property {
            CSSPropertyID::kWebkitBorderImage => CSSPropertyID::kBorderImageSource,
            CSSPropertyID::kWebkitMaskBoxImage => CSSPropertyID::kWebkitMaskBoxImageSource,
            _ => property,
        };
        for current in border_image {
            if current.IsImageValue()
                || current.IsImageGeneratorValue()
                || current.IsImageSetValue()
            {
                image.SetImage(Self::StyleImagePointer(
                    backend,
                    state,
                    image_property,
                    current,
                ));
            } else if current.IsBorderImageSliceValue() {
                Self::MapNinePieceImageSlice(backend, state, &current, image);
            } else if let Some(slash) = Self::List(backend, &current) {
                if let Some(first) = slash
                    .first()
                    .filter(|first| first.IsBorderImageSliceValue())
                {
                    Self::MapNinePieceImageSlice(backend, state, first, image);
                }
                if slash.len() > 1 {
                    image.SetBorderSlices(&Self::MapNinePieceImageQuad(backend, state, &slash[1]));
                }
                if slash.len() > 2 {
                    image.SetOutset(&Self::MapNinePieceImageQuad(backend, state, &slash[2]));
                }
            } else if current.IsPrimitiveValue() || current.IsValuePair() {
                Self::MapNinePieceImageRepeat(backend, state, &current, image);
            }
        }
        if property == CSSPropertyID::kWebkitBorderImage {
            ApplyLegacyBorderImageWidths(image, &mut state.StyleBuilderMut());
        }
    }
    fn ConvertBorderImageSliceSide(
        backend: &B,
        resolver: &B::LengthConversionData,
        value: &ResolverValue<B>,
    ) -> Length {
        if backend.PrimitiveIsPercentage(value) {
            if backend.PrimitiveHasUnresolvablePercentages(value) {
                return backend.PrimitiveToLength(value, resolver);
            }
            return BorderImageSliceSideFromResolved(backend.PrimitivePercentage(value, resolver), true);
        }
        BorderImageSliceSideFromResolved(backend.PrimitiveNumber(value, resolver), false)
    }
    pub fn MapNinePieceImageSlice(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        image: &mut NinePieceImage,
    ) {
        let CSSValuePayload::kBorderImageSliceClass(slice) = value.Payload() else {
            return;
        };
        let quad = backend.SliceQuad(slice);
        let (top, right, bottom, left) = backend.QuadValues(&quad);
        let top =
            Self::ConvertBorderImageSliceSide(backend, &state.CssToLengthConversionData(), &top);
        let bottom =
            Self::ConvertBorderImageSliceSide(backend, &state.CssToLengthConversionData(), &bottom);
        let left =
            Self::ConvertBorderImageSliceSide(backend, &state.CssToLengthConversionData(), &left);
        let right =
            Self::ConvertBorderImageSliceSide(backend, &state.CssToLengthConversionData(), &right);
        image.SetImageSlices(&LengthBox::new(top, right, bottom, left));
        image.SetFill(backend.SliceFill(slice));
    }
    fn ToBorderImageLength(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> BorderImageLength {
        if value.IsPrimitiveValue() && backend.PrimitiveIsNumber(value) {
            return BorderImageLength::from_number(
                backend.PrimitiveNumber(value, &state.CssToLengthConversionData()),
            );
        }
        BorderImageLength::from_length(&backend.ConvertLengthOrAuto(state, value))
    }
    pub fn MapNinePieceImageQuad(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> BorderImageLengthBox {
        let CSSValuePayload::kQuadClass(quad) = value.Payload() else {
            return BorderImageLengthBox::from_length(Length::Auto().clone());
        };
        let (top, right, bottom, left) = backend.QuadValues(quad);
        let top = Self::ToBorderImageLength(backend, state, &top);
        let right = Self::ToBorderImageLength(backend, state, &right);
        let bottom = Self::ToBorderImageLength(backend, state, &bottom);
        let left = Self::ToBorderImageLength(backend, state, &left);
        NinePieceImageQuadFromSides(&[top, right, bottom, left])
    }
    pub fn MapNinePieceImageRepeat(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        image: &mut NinePieceImage,
    ) {
        let (first, second) = if let CSSValuePayload::kValuePairClass(pair) = value.Payload() {
            let (first, second) = backend.PairValues(pair);
            (Self::ID(backend, &first), Self::ID(backend, &second))
        } else {
            let identifier = Self::ID(backend, value);
            (identifier, identifier)
        };
        MapNinePieceImageRepeatFromIdentifiers(first, second, image);
    }
    pub fn MapAnimationTriggerBehavior(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> EAnimationTriggerBehavior {
        backend.IdentifierTriggerBehavior(Self::Identifier(value))
    }
    pub fn MapAnimationTimelineTriggerName(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<Rc<ScopedCSSName>> {
        if value.IsIdentifierValue() {
            return None;
        }
        if let CSSValuePayload::kCustomIdentClass(ident) = value.Payload() {
            return Some(Rc::new(ScopedCSSName::new(
                &backend.CustomIdentComputeIdent(ident, &state.CssToLengthConversionData()),
                backend.CustomIdentTreeScope(ident),
            )));
        }
        None
    }
    pub fn MapAnimationTimelineTriggerBehavior(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> EAnimationTriggerBehavior {
        Self::MapAnimationTriggerBehavior(backend, state, value)
    }
    pub fn MapAnimationTimelineTriggerActivationRangeStart(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<B::TimelineOffset> {
        Self::MapAnimationRange(backend, state, value, 0.0)
    }
    pub fn MapAnimationTimelineTriggerActivationRangeEnd(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<B::TimelineOffset> {
        Self::MapAnimationRange(backend, state, value, 100.0)
    }
    pub fn MapAnimationTimelineTriggerActiveRangeStart(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TimelineOffsetOrAuto {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kAuto {
            return backend.DefaultTimelineOffsetOrAuto();
        }
        backend.NewTimelineOffsetOrAuto(Self::MapAnimationRange(backend, state, value, 0.0))
    }
    pub fn MapAnimationTimelineTriggerActiveRangeEnd(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::TimelineOffsetOrAuto {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kAuto {
            return backend.DefaultTimelineOffsetOrAuto();
        }
        backend.NewTimelineOffsetOrAuto(Self::MapAnimationRange(backend, state, value, 100.0))
    }
    pub fn MapAnimationTimelineTriggerSource(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> B::StyleTimeline {
        Self::MapAnimationTimeline(backend, state, value)
    }
    pub fn MapAnimationTriggerNames(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<Vec<AtomicString>> {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kNone {
            return None;
        }
        if let Some(list) = Self::List(backend, value) {
            let names = list
                .into_iter()
                .map(|value| {
                    let CSSValuePayload::kCustomIdentClass(ident) = value.Payload() else {
                        panic!("CSSCustomIdentValue required")
                    };
                    {
                        let text = ident.CustomCSSText();
                        match text.Span16() {
                            Some(units) => AtomicString::from_utf16(units),
                            None => AtomicString::default(),
                        }
                    }
                })
                .collect();
            return Some(names);
        }
        None
    }
    fn MapSingleAnimationTriggerAttachment(
        backend: &B,
        _state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Rc<B::StyleTriggerAttachment> {
        let CSSValuePayload::kTriggerAttachmentClass(attachment) = value.Payload() else {
            panic!("CSSTriggerAttachmentValue required")
        };
        let name_value = backend.AttachmentTriggerName(attachment);
        let CSSValuePayload::kCustomIdentClass(name_value) = name_value.Payload() else {
            panic!("CSSCustomIdentValue required")
        };
        let name = Rc::new(ScopedCSSName::new(
            &backend.CustomIdentValue(name_value),
            backend.CustomIdentTreeScope(name_value),
        ));
        let enter = backend.AttachmentEnterBehavior(attachment);
        let enter = backend.IdentifierTriggerBehavior(Self::Identifier(&enter));
        let exit = backend
            .AttachmentExitBehavior(attachment)
            .filter(|exit| exit.IsIdentifierValue())
            .map(|exit| backend.IdentifierTriggerBehavior(Self::Identifier(&exit)));
        backend.NewStyleTriggerAttachment(name, enter, exit)
    }
    pub fn MapAnimationTriggerAttachments(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
    ) -> Option<Rc<B::StyleTriggerAttachmentVector>> {
        if value.IsIdentifierValue() && Self::ID(backend, value) == CSSValueID::kNone {
            return None;
        }
        let list = Self::List(backend, value).expect("CSSValueList required");
        let attachments = list
            .into_iter()
            .map(|attachment| {
                Self::MapSingleAnimationTriggerAttachment(backend, state, &attachment)
            })
            .collect();
        Some(backend.NewStyleTriggerAttachmentVector(attachments))
    }
}
