#![allow(non_snake_case)]

//! Ownership boundary for the canonical source fragment result.
//!
//! The translated source graph is still composed with LayoutNG while its
//! concrete types are mutually recursive. This crate owns the lifetime handle
//! without importing those types: only the assembly adapter may recover a
//! typed pointer from the opaque managed root.

use foundation::Persistent;
use std::any::Any;
use std::sync::Arc;

pub mod boundary_types;
pub mod revision;
pub mod trace;

#[cfg(test)]
pub(crate) static TRACE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub use boundary_types::{
    BackgroundPaintLocation, CanvasBackgroundPaintData, CollapsedTableEdge,
    CollapsedTablePaintData, CollapsedTableSection, ColumnRule, ColumnRulePaintData,
    FieldsetPaintData, FormControlPaintData, FragmentBoxSides, FragmentKind, FragmentPaintData,
    FragmentPaintRect, FrameSetPaintData, GapRule, GapRulePaintData, GapRuleSide, InlineGeometry,
    InlineGeometryRect, MathMLPaintData, MathMLPaintKind, PaintGlyph, PaintGlyphRun,
    PaintPropertiesHandle, PaintResources, PrePaintInvalidation, PreviousBoxGeometry,
    ReplacedContentPaintData, ScrollContainerPaintData, ScrollSnapAxisPaintData,
    ScrollSnapStrictnessPaintData, ScrollSnapTypePaintData, ScrollbarPaintAxis, ScrollbarPaintData,
    SelfOutlinePaintData, StickyAxisPaintData, StickyPositionPaintData, StitchedDecorationData,
    SvgTextPaintData, SvgViewportGeometry, TablePaintColumn, TablePaintData,
    TextCombineEmphasisPaintData, TextCombinePaintData,
};

/// Lifetime handle for Chromium's canonical source `LayoutResult`.
///
/// The handle owns no copied fragment topology. Its optional opaque state is a
/// flat, result-local Paint sidecar keyed by source owners; `T` and that state's
/// concrete type are chosen only by the assembly-side adapter. Downstream
/// packages can clone, compare, and move the result without depending on source
/// fragment implementation types.
pub struct LayoutResultHandle {
    owner: Persistent<()>,
    // Rust's package split keeps Paint outside the mutually-recursive source
    // LayoutNG assembly.  The opaque sidecar is nevertheless owned by one
    // canonical LayoutResult handle, so publishing a different LayoutResult
    // cannot mutate the Paint state observed through an older handle.
    paint_state: Option<Arc<dyn Any>>,
}

impl LayoutResultHandle {
    /// Adopt a managed source result without exposing its concrete type in
    /// this package. The caller must use the same `T` when dereferencing.
    pub unsafe fn FromLayoutResult<T: 'static>(owner: *const T) -> Self {
        assert!(!owner.is_null());
        Self {
            owner: Persistent::from_ptr(owner.cast_mut().cast()),
            paint_state: None,
        }
    }

    /// Adopt a source result together with assembly-private per-result state.
    /// The concrete sidecar type never crosses this package's public surface.
    pub unsafe fn FromLayoutResultWithPaintState<T: 'static, P: 'static>(
        owner: *const T,
        paint_state: Arc<P>,
    ) -> Self {
        assert!(!owner.is_null());
        Self {
            owner: Persistent::from_ptr(owner.cast_mut().cast()),
            paint_state: Some(paint_state),
        }
    }

    /// Recover the source result inside the assembly adapter.
    ///
    /// # Safety
    /// `T` must be the concrete type originally passed to `FromLayoutResult`.
    pub unsafe fn LayoutResult<T: 'static>(&self) -> *const T {
        self.owner.Get().cast()
    }

    /// Recover assembly-private state using the same concrete type supplied
    /// to `FromLayoutResultWithPaintState`.
    pub fn PaintState<P: 'static>(&self) -> Option<&P> {
        self.paint_state.as_deref()?.downcast_ref::<P>()
    }

    /// Keep the opaque per-result state alive without exposing its concrete
    /// type. Downstream stages use this only as a lifetime lease while they
    /// retain pointers into the canonical sidecar.
    pub fn PaintStateLease(&self) -> Option<Arc<dyn Any>> {
        self.paint_state.clone()
    }

    /// Keep the canonical source result while publishing an independent
    /// result-local Paint snapshot. This changes no fragment topology.
    pub fn WithPaintState<P: 'static>(&self, paint_state: Arc<P>) -> Self {
        Self {
            owner: self.owner.clone(),
            paint_state: Some(paint_state),
        }
    }

    pub fn IsEmpty(&self) -> bool {
        self.owner.Get().is_null()
    }

    pub fn PtrEq(&self, other: &Self) -> bool {
        self.owner.Get() == other.owner.Get()
    }
}

impl Default for LayoutResultHandle {
    fn default() -> Self {
        Self {
            owner: Persistent::default(),
            paint_state: None,
        }
    }
}

impl Clone for LayoutResultHandle {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
            paint_state: self.paint_state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutResultHandle;
    use foundation::{LayoutHeapScope, MakeGarbageCollected};
    use std::sync::Arc;

    #[test]
    fn empty_handles_compare_by_source_identity() {
        let first = LayoutResultHandle::default();
        let second = first.clone();
        assert!(first.IsEmpty());
        assert!(first.PtrEq(&second));
    }

    #[test]
    fn opaque_handle_preserves_the_managed_source_pointer() {
        let _heap = LayoutHeapScope::new();
        let source = MakeGarbageCollected(42_u64);
        let handle = unsafe { LayoutResultHandle::FromLayoutResult(source) };
        let recovered = unsafe { handle.LayoutResult::<u64>() };
        assert_eq!(recovered, source);
        assert_eq!(unsafe { *recovered }, 42);
    }

    #[test]
    fn cloned_handle_shares_only_its_result_owned_paint_state() {
        let _heap = LayoutHeapScope::new();
        let first_source = MakeGarbageCollected(1_u64);
        let second_source = MakeGarbageCollected(2_u64);
        let first_state = Arc::new(11_u64);
        let first = unsafe {
            LayoutResultHandle::FromLayoutResultWithPaintState(first_source, first_state.clone())
        };
        let clone = first.clone();
        let second = unsafe {
            LayoutResultHandle::FromLayoutResultWithPaintState(second_source, Arc::new(22_u64))
        };

        assert!(first.PtrEq(&clone));
        assert!(!first.PtrEq(&second));
        assert_eq!(first.PaintState::<u64>(), Some(&11));
        assert!(std::ptr::eq(
            first.PaintState::<u64>().unwrap(),
            clone.PaintState::<u64>().unwrap()
        ));
        assert_eq!(second.PaintState::<u64>(), Some(&22));
    }
}
