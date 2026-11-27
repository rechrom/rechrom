#![allow(non_snake_case)]

//! Public Layout facade.
//!
//! `layout` is the only Layout package other engines depend on, as `paint`
//! is for `prepaint`/`paint_*`. It owns no algorithm: [`LayoutEngine`] wraps
//! the core engine of the base crate `layout_base` and
//! forwards a chosen set of methods, and the modules below re-export the
//! Layout values that crossing code names. The core engine's remaining
//! methods and base's `internal` module stay private to Layout.
//!
//! # Module scheme
//!
//! Items live at `layout::<topic>::<Item>`; the topic names the boundary role,
//! not the source package that defines the item:
//!
//! | Module | Contents |
//! |---|---|
//! | crate root | [`LayoutEngine`], [`LayoutRegistry`], [`CreateLayoutRegistry`], [`LayoutPlatformServices`] |
//! | [`registry`] | the [`LayoutRegistry`] that Layout packages register their algorithms and factories into |
//! | [`lifecycle`] | lifecycle results and errors (`LayoutEngineError`, `ScrollLayoutUpdate`, `LayoutUpdateKind`) |
//! | [`mutation`] | boundary 3: `LayoutMutation`, `LayoutTreeUpdate`, the tree builder's parent `LayoutObject`, and mutation results |
//! | [`input`] | Style/DOM-projected inputs: `ConstraintSpace`, `ElementData`, `NodeKind`, style projection, theme metrics |
//! | [`result`] | what Layout produces for Paint/Interaction: `LayoutResultHandle`, `LayoutPaintOutput`, `PaintFragmentCursor`, fragment views (`ResidentBoxGeometry`, `FragmentCursor`), fragment Paint sidecars, `PaintResources` |
//! | [`output`] | borrowed `LayoutOutput` access, `LayoutNode`, hit-test cursors, intersection roots |
//! | [`fragment`] | the resident fragment tree: `LayoutResultHandle`, `PhysicalFragment`, `FragmentItem`, inline cursors |
//! | [`paint`] | boundary 4: `LayoutPaintOutput`, `PaintFragmentCursor`, fragment Paint sidecars, Paint-owned sources compiled in `layout_base` |
//! | [`paint_input`] | fragment paint-input value schema (`paint_input_types`) and paint style export |
//! | [`editing`] | editing positions, selection/caret state, password echo |
//! | [`scroll`] | native scroll, scrollbar, autoscroll, pointer-resize and mouse-coordinate sources |
//! | [`object`] | resident LayoutObject/PaintLayer types still named by consumers (wave E: replace with query views) |
//! | `testing` | hidden: Layout internals for `#[cfg(test)]` code and integration tests only |
//!
//! Each topic module is assembled from one submodule per consumer group in
//! `src/exports/<group>.rs` (browser, document, paint, interaction, webapi), so
//! consumer migrations edit disjoint files. The same item may be listed by
//! several groups; all of them resolve to one re-export.

mod engine;
/// Borrowed per-family query views on [`LayoutEngine`] and the command values
/// their families send through `LayoutMutation` (wave E).
pub mod views {
    pub mod event_registry;
    pub mod form_control;
    pub mod hit_test;
    pub mod paint_layer;
    pub mod pointer;
    pub mod resource;
    pub mod scroll;
    pub mod selection;
}
mod engine_groups {
    mod browser;
    mod document;
    mod interaction;
    mod paint;
    mod webapi;
}
// Group seeds are consumed only through the topic globs below.
#[allow(unused_imports)]
mod exports {
    pub mod browser;
    pub mod document;
    pub mod interaction;
    pub mod paint;
    pub mod webapi;
}


pub use engine::LayoutEngine;
pub use layout_base::layout_registry::LayoutRegistry;

macro_rules! topic {
    ($(#[$doc:meta])* $topic:ident) => {
        $(#[$doc])*
        #[allow(unused_imports)] // several groups may list one item
        pub mod $topic {
            pub use crate::exports::browser::$topic::*;
            pub use crate::exports::document::$topic::*;
            pub use crate::exports::interaction::$topic::*;
            pub use crate::exports::paint::$topic::*;
            pub use crate::exports::webapi::$topic::*;
        }
    };
}

topic!(
    /// The registry Layout packages register algorithms and LayoutObject
    /// factories into.
    registry
);
topic!(
    /// Lifecycle results and errors.
    lifecycle
);
topic!(
    /// Boundary 3: Style/DOM -> Layout mutations and their results.
    mutation
);
topic!(
    /// Inputs projected from DOM and Style.
    input
);
topic!(
    /// Borrowed access to the resident layout output and hit-test cursors.
    output
);
topic!(
    /// The resident fragment tree.
    fragment
);
topic!(
    /// Boundary 4: Layout -> Paint output and fragment-local Paint sidecars.
    paint
);
topic!(
    /// Fragment paint-input value schema.
    paint_input
);

/// What Layout produces for Paint and Interaction (boundary 4 and the
/// fragment queries): the retained source result, the borrowed views that
/// walk its resident fragments, and the fragment-local Paint sidecars those
/// views lend out.
///
/// The fragment tree is part of Layout's base (`layout_base/src/fragment/`);
/// nothing here copies it. Every item is also
/// reachable at its older topic path (`fragment`, `paint`, `output`), which
/// names the same re-export.
pub mod result {
    // The retained source result: a lifetime handle for Chromium's canonical
    // LayoutResult plus its result-local Paint state.
    pub use layout_base::fragment_tree::LayoutResultHandle;
    // Boundary 4: the published paint output and its fragment cursor, the
    // lease that keeps borrowed fragments alive, and the LayoutObject
    // identity Paint keys display-item clients by.
    pub use layout_base::layout_engine::{
        LayoutPaintOutput, PaintFragmentCursor, PaintFragmentLease, PaintLayoutObjectIdentity,
    };
    // The borrowed fragment view Interaction hit-tests through.
    pub use layout_base::layout_engine::FragmentCursor;
    // Fragment views: the resident box-geometry reader, fragment kind,
    // PrePaint's previous geometry and delayed-invalidation source object.
    pub use layout_base::fragment_tree::{
        FragmentKind, PrePaintSourceObject, PreviousBoxGeometry, ResidentBoxGeometry,
    };
    // Fragment-local Paint sidecars and the resources they reference.
    pub use layout_base::fragment_tree::{
        FragmentPaintData, FragmentPaintRect, PaintProperties, PaintResources, SvgTextPaintData,
    };
    // The receipt Paint hands back when its artifact is complete.
    pub use layout_base::layout_engine::{PaintClientCommit, PaintCommitOutput};
}
topic!(
    /// Editing positions, selection and caret state.
    editing
);
topic!(
    /// Native scroll, scrollbar and pointer sources.
    scroll
);
topic!(
    /// Resident LayoutObject types still named across the boundary.
    object
);

/// Layout internals for test fixtures and diagnostics only.
///
/// `#[cfg(test)]` code and integration tests (`crates/*/tests`) that build or
/// inspect the resident LayoutObject tree directly name these items here.
/// Production code must not use this module: it reaches Layout through
/// [`LayoutEngine`], the topic modules and the [`views`] query views.
#[doc(hidden)]
pub mod testing {
    pub use layout_algo_grid::layout_grid::LayoutGrid;
    pub use layout_base::internal::block_node::BlockNode;
    pub use layout_base::internal::hit_test_phase::HitTestPhase;
    pub use layout_base::internal::layout_block_flow::LayoutBlockFlow;
    pub use layout_base::internal::layout_box::LayoutBox;
    pub use layout_base::internal::layout_box_model_object::LayoutBoxModelObject;
    pub use layout_base::internal::layout_object::{LayoutObject, StyleChangeContext};
    pub use layout_base::internal::layout_object_builder::LayoutObjectTree;
    pub use layout_base::internal::layout_pass_scope::{
        LayoutObjectFactoryScope, LayoutPassScope,
    };
    pub use layout_base::internal::layout_replaced::{LayoutInputReplaced, LayoutReplaced};
    pub use layout_base::internal::layout_view::LayoutView;
    pub use layout_base::internal::layout_block_flow::text_overflow_members::LayoutBlockFlowShouldTruncateOverflowingTextFromInline;
    pub use layout_base::internal::css_zoom::ZoomedStyle;
    pub use layout_base::style_projection::{ProjectComputedStyle, ProjectListMarkerImage};
}

/// Platform services the embedding injects into Layout. They are
/// implemented by Paint's Skia geometry package, which depends on Layout, so
/// the facade receives them as values instead of linking them.
#[derive(Clone, Copy, Default)]
pub struct LayoutPlatformServices {
    /// Injected platform Path construction; SVG Layout owns the returned path.
    pub svg_shape_path: Option<
        fn(
            &layout_base::internal::layout_input::SvgShapeData,
        ) -> std::sync::Arc<dyn foundation::native_path::NativePath>,
    >,
    /// Shape coverage rasterizer for `shape-outside` image/shape thresholds.
    pub shape_coverage:
        Option<&'static dyn layout_geometry::shape_coverage::ShapeCoverageRasterizer>,
}

/// Build the registry the core [`LayoutEngine`] is constructed from: every
/// linked Layout package registers its algorithms and LayoutObject factories
/// into one [`LayoutRegistry`]. Source package slots that are not linked keep
/// their source capability errors and are not replaced with generic boxes.
///
/// Registration order (a later registration may overwrite a slot an earlier
/// one filled, so the order is part of the contract):
///
/// 1. `layout_algo_block::registration::Register`
/// 2. `layout_algo_multicol::registration::Register`
/// 3. `layout_algo_flex::registration::Register`
/// 4. `layout_algo_grid::registration::Register`
/// 5. `layout_algo_table::registration::Register`
/// 6. `layout_algo_float::registration::Register`
/// 7. `layout_algo_forms::registration::Register`
/// 8. `layout_algo_replaced::registration::Register`
/// 9. `layout_algo_list::registration::Register`
/// 10. `layout_algo_svg::registration::Register`
/// 11. the embedding's platform services (`svg_shape_path`, `shape_coverage`)
/// 12. only with feature `inline_layout`: `layout_algo_inline`'s
///     `registration::Register` (the `layout_algo_inline` crate), then
///     `layout_algo_block::block_inline_layout::RegisterInlineSupport`, then
///     `layout_algo_out_of_flow::registration::Register`
/// 13. the base boundary, `layout_base`'s
///     `internal::boundary::registration::Register`
pub fn CreateLayoutRegistry(services: LayoutPlatformServices) -> LayoutRegistry {
    let mut registry = LayoutRegistry::default();
    layout_algo_block::registration::Register(&mut registry);
    layout_algo_multicol::registration::Register(&mut registry);
    layout_algo_flex::registration::Register(&mut registry);
    layout_algo_grid::registration::Register(&mut registry);
    layout_algo_table::registration::Register(&mut registry);
    layout_algo_float::registration::Register(&mut registry);
    layout_algo_forms::registration::Register(&mut registry);
    layout_algo_replaced::registration::Register(&mut registry);
    layout_algo_list::registration::Register(&mut registry);
    layout_algo_svg::registration::Register(&mut registry);
    registry.objects.svg_shape_path = services.svg_shape_path;
    registry.shape_coverage = services.shape_coverage;
    #[cfg(feature = "inline_layout")]
    {
        layout_algo_inline::registration::Register(&mut registry);
        layout_algo_block::block_inline_layout::RegisterInlineSupport(&mut registry);
        layout_algo_out_of_flow::registration::Register(&mut registry);
    }
    layout_base::internal::boundary::registration::Register(&mut registry);
    registry
}
