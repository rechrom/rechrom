#![allow(non_snake_case)]

//! Public Layout facade.
//!
//! `layout` is the only Layout package other engines depend on, as `paint`
//! is for `prepaint`/`paint_*`. It owns no algorithm: [`LayoutEngine`] wraps
//! the core engine of `layout_core` (compiled once in `layout_assembly`) and
//! forwards a chosen set of methods, and the modules below re-export the
//! Layout values that crossing code names. The core engine's remaining
//! methods and the assembly's `internal` module stay private to Layout.
//!
//! # Module scheme
//!
//! Items live at `layout::<topic>::<Item>`; the topic names the boundary role,
//! not the source package that defines the item:
//!
//! | Module | Contents |
//! |---|---|
//! | crate root | [`LayoutEngine`], [`LayoutAssembly`], [`CreateLayoutAssembly`], [`LayoutPlatformServices`] |
//! | [`assembly`] | the translated algorithm/factory assembly value |
//! | [`lifecycle`] | lifecycle results and errors (`LayoutEngineError`, `ScrollLayoutUpdate`, `LayoutUpdateKind`) |
//! | [`mutation`] | boundary 3: `LayoutMutation`, `LayoutTreeUpdate` and mutation results |
//! | [`input`] | Style/DOM-projected inputs: `ConstraintSpace`, `ElementData`, `NodeKind`, style projection, theme metrics |
//! | [`output`] | borrowed `LayoutOutput` access, `LayoutNode`, hit-test cursors, intersection roots |
//! | [`fragment`] | the resident fragment tree: `LayoutResultHandle`, `PhysicalFragment`, `FragmentItem`, inline cursors |
//! | [`paint`] | boundary 4: `LayoutPaintOutput`, `PaintFragmentCursor`, fragment Paint sidecars, Paint-owned sources compiled in the assembly |
//! | [`paint_input`] | fragment paint-input value schema (`paint_input_types`) and paint style export |
//! | [`editing`] | editing positions, selection/caret state, password echo |
//! | [`scroll`] | native scroll, scrollbar, autoscroll, pointer-resize and mouse-coordinate sources |
//! | [`object`] | resident LayoutObject/PaintLayer types still named by consumers (wave E: replace with query views) |
//!
//! Each topic module is assembled from one submodule per consumer group in
//! `src/exports/<group>.rs` (browser, document, paint, interaction, webapi), so
//! consumer migrations edit disjoint files. The same item may be listed by
//! several groups; all of them resolve to one re-export.

mod engine;
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

/// Borrowed query views of the facade, one per legacy family (wave E).
pub mod views {
    pub mod selection;
}

pub use engine::LayoutEngine;
pub use layout_assembly::engine_assembly::LayoutAssembly;

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
    /// The translated algorithm and LayoutObject factory assembly.
    assembly
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

/// Platform services the embedding injects into the Layout assembly. They are
/// implemented by Paint's Skia geometry package, which depends on Layout, so
/// the facade receives them as values instead of linking them.
#[derive(Clone, Copy, Default)]
pub struct LayoutPlatformServices {
    /// Injected platform Path construction; SVG Layout owns the returned path.
    pub svg_shape_path: Option<
        fn(
            &layout_assembly::internal::layout_input::SvgShapeData,
        ) -> std::sync::Arc<dyn foundation::native_path::NativePath>,
    >,
    /// Shape coverage rasterizer for `shape-outside` image/shape thresholds.
    pub shape_coverage:
        Option<&'static dyn layout_geometry::shape_coverage::ShapeCoverageRasterizer>,
}

/// Assemble the full translated algorithm set from the linked algorithm
/// packages. Source package slots that are not linked keep their source
/// capability errors and are not replaced with generic boxes.
pub fn CreateLayoutAssembly(services: LayoutPlatformServices) -> LayoutAssembly {
    let mut assembly = LayoutAssembly::default();
    layout_algo_block::assembly::InstallBlockAlgorithm(&mut assembly);
    layout_algo_multicol::assembly::InstallMulticolAlgorithm(&mut assembly);
    layout_algo_flex::assembly::InstallFlexAlgorithm(&mut assembly);
    layout_algo_grid::assembly::InstallGridAlgorithm(&mut assembly);
    layout_algo_table::assembly::InstallTableAlgorithm(&mut assembly);
    layout_algo_float::assembly::InstallFloatAlgorithm(&mut assembly);
    layout_algo_forms::assembly::InstallFormsAlgorithm(&mut assembly);
    layout_algo_replaced::assembly::InstallReplacedAlgorithm(&mut assembly);
    layout_algo_list::assembly::InstallListModule(&mut assembly);
    layout_algo_svg::assembly::InstallSvgModule(&mut assembly);
    assembly.objects.svg_shape_path = services.svg_shape_path;
    assembly.shape_coverage = services.shape_coverage;
    #[cfg(feature = "inline_layout")]
    {
        layout_assembly::assembly::InstallInlineAlgorithm(&mut assembly);
        layout_algo_block::block_inline_layout::InstallBlockInlineSupport(&mut assembly);
        layout_algo_out_of_flow::assembly::InstallOutOfFlowAlgorithm(&mut assembly);
    }
    layout_assembly::internal::boundary::assembly::InstallLayoutBoundary(&mut assembly);
    assembly
}
