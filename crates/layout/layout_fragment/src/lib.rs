// Crate root of the `layout_fragment` crate: fragment data only. It depends
// on the shared Layout roots and never on base (`layout_core`, compiled
// inside `layout_assembly`); base depends on it.
//
// Compile units in `layout_fragment/src/` (each file has exactly one):
//
// * This crate compiles `lib.rs`, `boundary_types.rs`, `trace.rs` and the
//   modules declared below.
// * The fragment-tree files whose methods still dereference base
//   LayoutObject behaviour are compiled by `layout_assembly`, through
//   `layout_assembly/src/layout_fragment_package.rs` (declared by both
//   assemblies), through `#[path]` from `layout_core` facades, or from
//   `layout_assembly`'s root. `crates/layout/PACKAGE_MAP.md` §7 (R4 status)
//   lists the side of every file.
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
pub mod opaque;
pub mod trace;
mod trace_bindings;

/// Fragment data, mounted at the module paths under which base re-exports it
/// (`layout_core::internal::<module>`; see `layout_core/src/lib.rs`). A module
/// that base used to reach through a compatibility facade keeps that facade's
/// shape here (a private source module plus a glob re-export), so the
/// translated `super::` paths inside the source files resolve unchanged.
pub mod internal {
    #[path = "../core/disable_layout_side_effects_scope.rs"]
    pub mod disable_layout_side_effects_scope;
    #[path = "../core/hit_test_phase.rs"]
    pub mod hit_test_phase;
    #[path = "../core/layout_theme.rs"]
    pub mod layout_theme;
    #[path = "../core/map_coordinates_flags.rs"]
    pub mod map_coordinates_flags;
    #[path = "../core/selection_state.rs"]
    pub mod selection_state;
    #[path = "../core/split_axis_item.rs"]
    pub mod split_axis_item;
    #[path = "../core/style_variant.rs"]
    pub mod style_variant;
    #[path = "../core/text_decoration_offset.rs"]
    pub mod text_decoration_offset;
    #[path = "../core/text_offset_range.rs"]
    pub mod text_offset_range;

    pub mod exclusions {
        #[path = "../../core/exclusions/line_layout_opportunity.rs"]
        pub mod line_layout_opportunity;
    }

    pub mod gap {
        pub mod cross_gap {
            #[path = "../../../cross_gap.rs"]
            mod layout_fragment_cross_gap;
            pub use layout_fragment_cross_gap::*;
        }
        pub mod gap_intersection {
            #[path = "../../../gap_intersection.rs"]
            mod layout_fragment_gap_intersection;
            pub use layout_fragment_gap_intersection::*;
        }
        #[allow(non_upper_case_globals)]
        pub mod gap_utils {
            #[path = "../../../gap_utils.rs"]
            mod layout_fragment_gap_utils;
            pub use layout_fragment_gap_utils::*;
        }
        pub mod main_gap {
            #[path = "../../../main_gap.rs"]
            mod layout_fragment_main_gap;
            pub use layout_fragment_main_gap::*;
        }
    }

    pub mod shapes {
        pub mod shape {
            #[path = "../../../float/shape.rs"]
            mod layout_algo_float_shape;
            pub use layout_algo_float_shape::*;
        }
    }

    pub mod break_appeal {
        #[path = "../../fragmentation/break_appeal.rs"]
        mod layout_algo_fragmentation_break_appeal;
        pub use layout_algo_fragmentation_break_appeal::*;
    }
    pub mod custom_layout_payload {
        #[path = "../../custom/custom_layout_payload.rs"]
        mod layout_algo_custom_custom_layout_payload;
        pub use layout_algo_custom_custom_layout_payload::*;
    }
    pub mod devtools_flex_info {
        #[path = "../../flex/devtools_flex_info.rs"]
        mod layout_algo_flex_devtools_flex_info;
        pub use layout_algo_flex_devtools_flex_info::*;
    }
    pub mod form_control_types {
        #[path = "../../forms/form_control_types.rs"]
        mod layout_algo_forms_form_control_types;
        pub use layout_algo_forms_form_control_types::*;
    }
    pub mod frame_set_layout_data {
        #[path = "../../frame_set_layout_data.rs"]
        mod layout_fragment_frame_set_layout_data;
        pub use layout_fragment_frame_set_layout_data::*;
    }
    pub mod grid_layout_data {
        #[path = "../../grid/grid_layout_data.rs"]
        mod layout_algo_grid_grid_layout_data;
        pub use layout_algo_grid_grid_layout_data::*;
    }
    pub mod grid_subtree {
        #[path = "../../grid/grid_subtree.rs"]
        mod layout_algo_grid_grid_subtree;
        pub use layout_algo_grid_grid_subtree::*;
    }
    pub mod grid_track_collection {
        #[path = "../../grid/grid_track_collection_types.rs"]
        mod layout_algo_grid_grid_track_collection_types;
        pub use layout_algo_grid_grid_track_collection_types::*;
    }
    pub mod inline_item_text_index {
        #[path = "../../inline/inline_item_text_index.rs"]
        mod layout_algo_inline_inline_item_text_index;
        pub use layout_algo_inline_inline_item_text_index::*;
    }
    pub mod mathml_paint_info {
        #[path = "../../mathml_paint_info.rs"]
        mod layout_fragment_mathml_paint_info;
        pub use layout_fragment_mathml_paint_info::*;
    }
    pub mod outline_info {
        #[path = "../../outline_info.rs"]
        mod layout_fragment_outline_info;
        pub use layout_fragment_outline_info::*;
    }
    pub mod table_column_location {
        #[path = "../../table/table_column_location.rs"]
        mod layout_algo_table_table_column_location;
        pub use layout_algo_table_table_column_location::*;
    }
    pub mod table_constraint_space_data {
        #[path = "../../table/table_constraint_space_data.rs"]
        mod layout_algo_table_table_constraint_space_data;
        pub use layout_algo_table_table_constraint_space_data::*;
    }
    pub mod text_fit_scale {
        #[path = "../../inline/text_fit_scale.rs"]
        mod layout_algo_inline_text_fit_scale;
        pub use layout_algo_inline_text_fit_scale::*;
    }
    pub mod text_item_type {
        #[path = "../../inline/text_item_type.rs"]
        mod layout_algo_inline_text_item_type;
        pub use layout_algo_inline_text_item_type::*;
    }
    pub mod used_font {
        #[path = "../../inline/used_font_interface.rs"]
        mod layout_algo_inline_used_font_interface;
        pub use layout_algo_inline_used_font_interface::*;
    }
}

// Method bodies of fragment data types that the snapshot declared as
// link-time symbols (`extern "Rust"` in the type's file, `#[no_mangle]`
// definition in the algorithm package). Both halves now compile here.
// cpp: layoutng_grid/grid_track_collection.cc
#[path = "grid/grid_track_collection.rs"]
pub mod grid_track_collection_implementation;
// cpp: layoutng_inline/used_font.cc
#[path = "inline/used_font.rs"]
mod used_font_implementation;
// Gap segment-state aggregation over the fragment-owned gap types
// (`internal::gap`); its trait and inherent impls must sit in the crate that
// declares `GapSegmentStateAggregator`.
#[path = "grid/gap_segment_state_aggregator.rs"]
pub mod grid_gap_segment_state_aggregator;

#[cfg(test)]
pub(crate) static TRACE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub use boundary_types::{
    BackgroundPaintLocation, CanvasBackgroundPaintData, CollapsedTableEdge,
    CollapsedTablePaintData, CollapsedTableSection, ColumnRule, ColumnRulePaintData,
    FieldsetPaintData, FormControlPaintData, FragmentBoxSides, FragmentKind, FragmentPaintData,
    FragmentPaintRect, FrameSetPaintData, GapRule, GapRulePaintData, GapRuleSide,
    MathMLPaintData, MathMLPaintKind, PaintGlyph, PaintGlyphRun,
    PaintPropertiesHandle, PaintResources, PrePaintInvalidation, PreviousBoxGeometry,
    ResidentFragmentOwner,
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
