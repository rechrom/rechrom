//! Translation of Chromium resolver/filter_operation_resolver.{h,cc}.
//! Source: /Users/zhenghuaiyu/chromium/src/third_party/blink/renderer/core/css/resolver/
//! Chromium commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Ledger (physical / effective / mapped / omitted / pending):
//! filter_operation_resolver.h: 57 / 16 / 10 / 6 / 0.
//! filter_operation_resolver.cc: 359 / 254 / 197 / 57 / 0.
//! Effective excludes copyright/comments, blanks, preprocessing/includes,
//! namespaces and pure bracket/punctuation lines. Header omissions: forward
//! declarations 34-36,38, STATIC_ONLY 41 and access label 43. cc omissions are
//! metrics-only CountFilterUse 75-126 plus calls 209-210,224, and pure debug
//! checks 199,225,281,310. All remaining production cc:44-357 maps: enum/function
//! selection, percent/number/angle defaults and clamping, reference URL/resource
//! identity plus old filter reuse, native operation creation and list append
//! order, blur/drop-shadow conversion and offscreen initialization/URL rejection.
//! Existing FilterOperations/FilterOperation subclasses/ShadowData/CSSValue and
//! StyleResolverState are reused. Only absent reference-filter/resource and value/
//! conversion owners cross required typed interfaces, without defaults.
#![allow(non_snake_case)]
use super::style_resolver_state::{
    ComputedStyleHandle, ResolverValue, StyleResolverState, StyleResolverStateBackend,
};
use crate::css_value::{CSSValueDispatch, CSSValueListSubclass, CSSValuePayload};
use font_engine::Font;
use foundation::{
    CSSPropertyID, CSSValueID, Length, MakeGarbageCollected, Member, String, WritingMode,
};
use layoutng_style::style::filter_operation::{
    BasicColorMatrixFilterOperation, BasicComponentTransferFilterOperation, BlurFilterOperation,
    DropShadowFilterOperation, FilterOperation, OperationType,
};
use layoutng_style::style::filter_operations::FilterOperations;
use layoutng_style::style::shadow_data::ShadowData;
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

type Function<B> =
    <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSFunctionValue;
type List<B> = <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSValueList;
type URI<B> = <<B as StyleResolverStateBackend>::ValueDispatch as CSSValueDispatch>::CSSURIValue;

/// Required CSSValue/StyleBuilderConverter/CSSToLengthConversionData and missing
/// native ReferenceFilterOperation owner methods. The list/numeric decisions,
/// old-style filter reuse and creation of existing native CSS classes stay Rust.
pub trait FilterOperationResolverBackend: StyleResolverStateBackend {
    type Filter;
    fn FilterListValues(&self, value: &List<Self>) -> Vec<Rc<ResolverValue<Self>>>;
    fn FilterFunctionType(&self, value: &Function<Self>) -> CSSValueID;
    fn FilterPrimitiveIsPercentage(&self, value: &ResolverValue<Self>) -> bool;
    fn FilterPrimitivePercentage(
        &self,
        value: &ResolverValue<Self>,
        data: &Self::LengthConversionData,
    ) -> f64;
    fn FilterPrimitiveNumber(
        &self,
        value: &ResolverValue<Self>,
        data: &Self::LengthConversionData,
    ) -> f64;
    fn FilterPrimitiveDegrees(
        &self,
        value: &ResolverValue<Self>,
        data: &Self::LengthConversionData,
    ) -> f64;
    fn FilterPrimitiveToLength(
        &self,
        value: &ResolverValue<Self>,
        data: &Self::LengthConversionData,
    ) -> Length;
    fn FilterConvertShadow(
        &self,
        data: &Self::LengthConversionData,
        state: Option<&StyleResolverState<'_, Self>>,
        value: &ResolverValue<Self>,
    ) -> ShadowData;
    fn URIValueForSerialization(&self, value: &URI<Self>) -> String;
    /// Must allocate the actual, non-null managed ReferenceFilterOperation;
    /// the returned base has type kReference and retains the given resource.
    fn NewReferenceFilterOperation(
        &self,
        url: String,
        resource: Option<Rc<Self::SVGResource>>,
    ) -> *mut FilterOperation;
    fn ReferenceFilterURL(&self, operation: &FilterOperation) -> String;
    fn ReferenceFilterResource(&self, operation: &FilterOperation)
        -> Option<Rc<Self::SVGResource>>;
    fn ReferenceFilterGetFilter(&self, operation: &FilterOperation) -> Option<Rc<Self::Filter>>;
    fn ReferenceFilterSetFilter(
        &self,
        operation: &mut FilterOperation,
        filter: Option<Rc<Self::Filter>>,
    );
    /// Actual conversion-data construction uses default line-height/container/
    /// anchor objects and a null element; flags are shared with the returned data.
    fn NewOffscreenFilterConversionData(
        &self,
        font: Option<&Font>,
        em: f32,
        rem: f32,
        font_zoom: f32,
        viewport_width: f32,
        viewport_height: f32,
        writing_mode: WritingMode,
        zoom: f32,
        ignored_flags: Rc<Cell<u32>>,
    ) -> Self::LengthConversionData;
}

pub const kOffScreenCanvasEmFontSize: f32 = 16.0;
pub const kOffScreenCanvasRemFontSize: f32 = 16.0;
pub struct FilterOperationResolver<B: FilterOperationResolverBackend>(PhantomData<fn() -> B>);
impl<B: FilterOperationResolverBackend> FilterOperationResolver<B> {
    pub fn FilterOperationForType(value: CSSValueID) -> OperationType {
        match value {
            CSSValueID::kGrayscale => OperationType::kGrayscale,
            CSSValueID::kSepia => OperationType::kSepia,
            CSSValueID::kSaturate => OperationType::kSaturate,
            CSSValueID::kHueRotate => OperationType::kHueRotate,
            CSSValueID::kInvert => OperationType::kInvert,
            CSSValueID::kOpacity => OperationType::kOpacity,
            CSSValueID::kBrightness => OperationType::kBrightness,
            CSSValueID::kContrast => OperationType::kContrast,
            CSSValueID::kBlur => OperationType::kBlur,
            CSSValueID::kDropShadow => OperationType::kDropShadow,
            _ => unreachable!("invalid CSS filter function"),
        }
    }
    fn List(backend: &B, value: &ResolverValue<B>) -> Vec<Rc<ResolverValue<B>>> {
        let list = match value.Payload() {
            CSSValuePayload::kValueListClass(list) => list,
            CSSValuePayload::kFunctionClass(value) => value.AsValueList(),
            CSSValuePayload::kImageSetClass(value) => value.AsValueList(),
            CSSValuePayload::kGridLineNamesClass(value) => value.AsValueList(),
            CSSValuePayload::kGridAutoRepeatClass(value) => value.AsValueList(),
            CSSValuePayload::kGridIntegerRepeatClass(value) => value.AsValueList(),
            CSSValuePayload::kAxisClass(value) => value.AsValueList(),
            _ => panic!("CSSValueList required for filter list"),
        };
        backend.FilterListValues(list)
    }
    pub fn ResolveNumericArgumentForFunction(
        backend: &B,
        filter: &Function<B>,
        resolver: &B::LengthConversionData,
    ) -> f64 {
        let kind = backend.FilterFunctionType(filter);
        match kind {
            CSSValueID::kGrayscale
            | CSSValueID::kSepia
            | CSSValueID::kSaturate
            | CSSValueID::kInvert
            | CSSValueID::kBrightness
            | CSSValueID::kContrast
            | CSSValueID::kOpacity => {
                let values = backend.FilterListValues(filter.AsValueList());
                if values.len() == 1 {
                    let value = &values[0];
                    let number = if backend.FilterPrimitiveIsPercentage(value) {
                        backend.FilterPrimitivePercentage(value, resolver) / 100.0
                    } else {
                        backend.FilterPrimitiveNumber(value, resolver)
                    };
                    if kind != CSSValueID::kBrightness
                        && kind != CSSValueID::kSaturate
                        && kind != CSSValueID::kContrast
                    {
                        number.clamp(0.0, 1.0)
                    } else {
                        number
                    }
                } else {
                    1.0
                }
            }
            CSSValueID::kHueRotate => {
                let values = backend.FilterListValues(filter.AsValueList());
                if values.len() == 1 {
                    backend.FilterPrimitiveDegrees(&values[0], resolver)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
    fn GetFilterFromOldStyle(
        backend: &B,
        old_style: Option<&ComputedStyleHandle>,
        new_ref: &FilterOperation,
        index: usize,
    ) -> Option<Rc<B::Filter>> {
        let old_style = old_style?;
        let old_style = unsafe { &*old_style.Get() };
        if !old_style.HasFilter() {
            return None;
        }
        let operations = old_style.Filter().Operations();
        let operation = operations.get(index)?;
        let old_ref = unsafe { operation.Get().as_ref() }?;
        if old_ref.GetType() != OperationType::kReference {
            return None;
        }
        if backend.ReferenceFilterURL(old_ref) != backend.ReferenceFilterURL(new_ref) {
            return None;
        }
        let old_resource = backend.ReferenceFilterResource(old_ref);
        let new_resource = backend.ReferenceFilterResource(new_ref);
        let same_resource = match (old_resource.as_ref(), new_resource.as_ref()) {
            (None, None) => true,
            (Some(old), Some(new)) => Rc::ptr_eq(old, new),
            _ => false,
        };
        if same_resource {
            backend.ReferenceFilterGetFilter(old_ref)
        } else {
            None
        }
    }
    fn AppendFunction(
        backend: &B,
        operations: &mut FilterOperations,
        filter: &Function<B>,
        resolver: &B::LengthConversionData,
        state: Option<&StyleResolverState<'_, B>>,
    ) {
        let kind = backend.FilterFunctionType(filter);
        let operation_type = Self::FilterOperationForType(kind);
        let pointer = match kind {
            CSSValueID::kGrayscale
            | CSSValueID::kSepia
            | CSSValueID::kSaturate
            | CSSValueID::kHueRotate => {
                let value = Self::ResolveNumericArgumentForFunction(backend, filter, resolver);
                MakeGarbageCollected(BasicColorMatrixFilterOperation::new(value, operation_type))
                    .cast::<FilterOperation>()
            }
            CSSValueID::kInvert
            | CSSValueID::kBrightness
            | CSSValueID::kContrast
            | CSSValueID::kOpacity => {
                let value = Self::ResolveNumericArgumentForFunction(backend, filter, resolver);
                MakeGarbageCollected(BasicComponentTransferFilterOperation::new(
                    value,
                    operation_type,
                ))
                .cast::<FilterOperation>()
            }
            CSSValueID::kBlur => {
                let values = backend.FilterListValues(filter.AsValueList());
                let mut deviation = Length::Fixed(0.0);
                if !values.is_empty() {
                    deviation = backend.FilterPrimitiveToLength(&values[0], resolver);
                }
                MakeGarbageCollected(BlurFilterOperation::new(&deviation)).cast::<FilterOperation>()
            }
            CSSValueID::kDropShadow => {
                let values = backend.FilterListValues(filter.AsValueList());
                let shadow = backend.FilterConvertShadow(resolver, state, &values[0]);
                MakeGarbageCollected(DropShadowFilterOperation::new(shadow))
                    .cast::<FilterOperation>()
            }
            _ => unreachable!("invalid CSS filter function"),
        };
        // Native derived operation classes use repr(C), base offset zero, and
        // their registered concrete allocation owns tracing/destruction.
        operations.OperationsMut().push(Member::from_ptr(pointer));
    }
    pub fn CreateFilterOperations(
        backend: &B,
        state: &StyleResolverState<'_, B>,
        value: &ResolverValue<B>,
        property_id: CSSPropertyID,
    ) -> FilterOperations {
        let mut operations = FilterOperations::new();
        if value.IsIdentifierValue() {
            return operations;
        }
        let data = state.CssToLengthConversionData();
        for current in Self::List(backend, value) {
            if let CSSValuePayload::kURIClass(url) = current.Payload() {
                let url_string = backend.URIValueForSerialization(url);
                let resource = state.GetSVGResource(property_id, url);
                let pointer = backend.NewReferenceFilterOperation(url_string, resource);
                let new_ref = unsafe { &mut *pointer };
                let filter = Self::GetFilterFromOldStyle(
                    backend,
                    state.OldStyle().as_ref(),
                    new_ref,
                    operations.size() as usize,
                );
                backend.ReferenceFilterSetFilter(new_ref, filter);
                operations.OperationsMut().push(Member::from_ptr(pointer));
                continue;
            }
            let CSSValuePayload::kFunctionClass(filter) = current.Payload() else {
                panic!("CSSFunctionValue required")
            };
            Self::AppendFunction(backend, &mut operations, filter, &data, Some(state));
        }
        operations
    }
    pub fn CreateOffscreenFilterOperations(
        backend: &B,
        value: &ResolverValue<B>,
        font: Option<&Font>,
    ) -> FilterOperations {
        let mut operations = FilterOperations::new();
        if value.IsIdentifierValue() {
            return operations;
        }
        let flags = Rc::new(Cell::new(0));
        let data = backend.NewOffscreenFilterConversionData(
            font,
            kOffScreenCanvasEmFontSize,
            kOffScreenCanvasRemFontSize,
            1.0,
            0.0,
            0.0,
            WritingMode::kHorizontalTb,
            1.0,
            flags,
        );
        for current in Self::List(backend, value) {
            if current.IsURIValue() {
                continue;
            }
            let CSSValuePayload::kFunctionClass(filter) = current.Payload() else {
                panic!("CSSFunctionValue required")
            };
            Self::AppendFunction(backend, &mut operations, filter, &data, None);
        }
        operations
    }
}
