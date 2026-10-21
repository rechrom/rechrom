// Copyright 2006 The Android Open Source Project. BSD-style license.
// Public/native-base inheritance becomes a single required CPU trait object. Arc<dyn
// SkMaskFilter> retains ownership; no filterMask default or identity-only object exists.
// Source: include/core/SkMaskFilter.h:26-26
pub use crate::src::core::SkMaskFilterBase::SkMaskFilterBase as SkMaskFilter;

pub use crate::src::core::SkBlurMaskFilterImpl::MakeBlur;

// Source: include/core/SkMaskFilter.h:41-41
// Source: src/core/SkMaskFilter.cpp:15-17
/// Safety: exclusive native registration phase, before once publication;
/// no concurrent lookup/registration, with space in the fixed factory table.
pub(crate) unsafe fn RegisterFlattenables() {
    unsafe {
        super::SkBlurMaskFilterImpl::sk_register_blur_maskfilter_createproc();
    }
}

// Source: include/core/SkMaskFilter.h:37-38
// Source: src/core/SkMaskFilter.cpp:19-24
/// Transfer the actual retained MaskFilter owner from the typed native factory.
/// Rust factory failures propagate through Result without changing parse failure.
pub fn Deserialize(
    data: &[u8],
    procs: Option<&crate::include::core::SkSerialProcs::SkDeserialProcs>,
) -> Result<Option<std::sync::Arc<dyn SkMaskFilter>>, crate::RasterError> {
    match super::SkFlattenable::Deserialize(
        super::SkFlattenable::Type::kSkMaskFilter_Type,
        data,
        procs,
    )? {
        None => Ok(None),
        Some(super::SkFlattenable::FlattenableOwner::MaskFilter(owner)) => Ok(Some(owner)),
        Some(_) => unreachable!("MaskFilter factory must return its canonical MaskFilter owner"),
    }
}
