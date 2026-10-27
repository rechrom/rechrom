#![allow(non_snake_case)]

//! Ownership boundary for the canonical native fragment result.
//!
//! The translated native graph is still composed with LayoutNG while its
//! concrete types are mutually recursive. This crate owns the lifetime handle
//! without importing those types: only the assembly adapter may recover a
//! typed pointer from the opaque managed root.

use foundation::Persistent;

pub mod boundary_types;
pub mod revision;
pub mod trace;

#[cfg(test)]
pub(crate) static TRACE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub use boundary_types::{
    BackgroundPaintLocation, CanvasBackgroundPaintData, CollapsedTableEdge,
    CollapsedTablePaintData, CollapsedTableSection, ColumnRule, ColumnRulePaintData,
    FieldsetPaintData, FormControlPaintData, FragmentBoxSides, FragmentKind, FragmentPaintData,
    FragmentPaintRect, FrameSetPaintData, InlineGeometry, InlineGeometryRect, MathMLPaintData,
    MathMLPaintKind, PaintGlyph, PaintGlyphRun, PaintPropertiesHandle, PaintResources,
    PrePaintInvalidation, ReplacedContentPaintData, ScrollContainerPaintData, ScrollbarPaintAxis,
    ScrollbarPaintData, StickyAxisPaintData, StickyPositionPaintData, StitchedDecorationData,
    SvgTextPaintData, SvgViewportGeometry, TablePaintColumn, TablePaintData,
    TextCombineEmphasisPaintData, TextCombinePaintData, TextControlCaretMetrics,
};

/// Lifetime handle for Chromium's canonical native `LayoutResult`.
///
/// The handle owns no projected geometry or copied child topology. `T` is
/// intentionally chosen only by the assembly-side adapter at construction and
/// dereference sites; downstream packages can clone, compare, and move the
/// result without depending on native fragment implementation types.
pub struct LayoutResultHandle {
    owner: Persistent<()>,
}

impl LayoutResultHandle {
    /// Adopt a managed native result without exposing its concrete type in
    /// this package. The caller must use the same `T` when dereferencing.
    pub unsafe fn FromNative<T: 'static>(owner: *const T) -> Self {
        assert!(!owner.is_null());
        Self {
            owner: Persistent::from_ptr(owner.cast_mut().cast()),
        }
    }

    /// Recover the native result inside the assembly adapter.
    ///
    /// # Safety
    /// `T` must be the concrete type originally passed to `FromNative`.
    pub unsafe fn NativeResult<T: 'static>(&self) -> *const T {
        self.owner.Get().cast()
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
        }
    }
}

impl Clone for LayoutResultHandle {
    fn clone(&self) -> Self {
        Self {
            owner: self.owner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LayoutResultHandle;
    use foundation::{LayoutHeapScope, MakeGarbageCollected};

    #[test]
    fn empty_handles_compare_by_native_identity() {
        let first = LayoutResultHandle::default();
        let second = first.clone();
        assert!(first.IsEmpty());
        assert!(first.PtrEq(&second));
    }

    #[test]
    fn opaque_handle_preserves_the_managed_native_pointer() {
        let _heap = LayoutHeapScope::new();
        let native = MakeGarbageCollected(42_u64);
        let handle = unsafe { LayoutResultHandle::FromNative(native) };
        let recovered = unsafe { handle.NativeResult::<u64>() };
        assert_eq!(recovered, native);
        assert_eq!(unsafe { *recovered }, 42);
    }
}
