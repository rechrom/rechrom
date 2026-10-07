use crate::UniqueObjectId;

// cpp: foundation/graphics_types/cc/paint/element_id.h:25
pub const kElementIdReservedBitCount: u32 = 1;

// cpp: foundation/graphics_types/cc/paint/element_id.h:47-85
#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositorElementId(u64);

#[allow(non_snake_case)]
impl CompositorElementId {
    pub fn new(value: u64) -> Self {
        assert!(value != 0 && value != u64::MAX);
        Self(value)
    }

    pub fn DeletedValue() -> Self {
        Self(u64::MAX)
    }

    pub fn GetInternalValue(self) -> u64 {
        self.0
    }

    pub fn IsValidInternalValue(value: u64) -> bool {
        value != 0 && value != u64::MAX
    }
}

// cpp: foundation/graphics_types/graphics/compositor_element_id.h:16-20,22-54
pub const kCompositorNamespaceBitCount: u32 = 5;
pub const kCompositorReservedBitCount: u32 = kElementIdReservedBitCount;

#[repr(u64)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositorElementIdNamespace {
    kPrimary,
    kUniqueObjectId,
    kScroll,
    kStickyTranslation,
    kAnchorPositionScrollTranslation,
    kElementCanvasTransform,
    kPrimaryEffect,
    kPrimaryTransform,
    kEffectFilter,
    kEffectMask,
    kEffectClipPath,
    kScaleTransform,
    kRotateTransform,
    kTranslateTransform,
    kVerticalScrollbar,
    kHorizontalScrollbar,
    kScrollCorner,
    kViewTransitionScopeRoot,
    kViewTransitionElement,
    kElementCapture,
    kUnboundedWrapperEffect,
    kDOMNodeId,
}

#[allow(non_upper_case_globals)]
impl CompositorElementIdNamespace {
    pub const kMax: Self = Self::kDOMNodeId;
    pub const kMaxRepresentable: u64 = 1 << kCompositorNamespaceBitCount;
}

const _: () = assert!(
    (CompositorElementIdNamespace::kMax as u64) < CompositorElementIdNamespace::kMaxRepresentable
);

// cpp: foundation/graphics_types/graphics/compositor_element_id.cc:16-42
#[allow(non_snake_case)]
pub fn CompositorElementIdFromUniqueObjectId(
    id: UniqueObjectId,
    namespace_id: CompositorElementIdNamespace,
) -> CompositorElementId {
    assert!(namespace_id as u64 <= CompositorElementIdNamespace::kMax as u64);
    assert!(id != 0);
    assert!(
        id < u64::MAX
            / (CompositorElementIdNamespace::kMaxRepresentable << kCompositorReservedBitCount)
    );
    let value =
        ((id << kCompositorNamespaceBitCount) + namespace_id as u64) << kCompositorReservedBitCount;
    CompositorElementId::new(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_id_and_namespace_in_source_bit_order() {
        let id = CompositorElementIdFromUniqueObjectId(
            7,
            CompositorElementIdNamespace::kStickyTranslation,
        );
        assert_eq!(id.GetInternalValue(), ((7 << 5) + 3) << 1);
    }
}
