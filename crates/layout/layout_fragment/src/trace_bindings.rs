// Bind the source Trace methods of fragment data to the layout heap without
// moving their translated bodies out of their owning files. Rust coherence
// requires `impl Traceable` (a foundation trait) to sit in the crate that
// declares the type, so these left base's `layoutng_trace_bindings.rs` with
// their types.
// cpp: foundation/blink_base/heap/trace_traits.h:20-28
use foundation::{Traceable, Visitor};

use crate::internal::constraint_space::{ConstraintSpace, RareData};
use crate::internal::devtools_flex_info::DevtoolsFlexInfo;
use crate::internal::exclusions::exclusion_area::{ExclusionArea, ExclusionShapeData};
use crate::internal::exclusions::exclusion_space::{
    ClosedArea, DerivedGeometry, ExclusionSpace, ExclusionSpaceInternal, Shelf,
};
use crate::internal::exclusions::layout_opportunity::LayoutOpportunity;
use crate::internal::exclusions::shape_exclusions::ShapeExclusions;
use crate::internal::grid_layout_data::{
    GridLayoutData, GridLayoutSubtree, GridLayoutTree, GridTreeNode,
};
use crate::internal::grid_track_collection::{GridLayoutTrackCollection, GridTrackBaselines};
use crate::internal::line_clamp_data::LineClampAncestorChain;
use crate::internal::mathml_paint_info::MathMLPaintInfo;
use crate::internal::naming_scope::NamingScope;
use crate::internal::non_overflowing_scroll_range::NonOverflowingScrollRange;
use crate::internal::snap_area::SnapArea;
use crate::internal::split_axis_item::SplitAxisItem;
use crate::internal::text_fit_scale::TextFitScale;
use crate::internal::used_font::UsedFont;

macro_rules! trace_source_method {
    ($($type:ty),+ $(,)?) => {
        $(impl Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                <$type>::Trace(self, visitor);
            }
        })+
    };
}

trace_source_method!(
    ConstraintSpace,
    RareData,
    ExclusionArea,
    ExclusionShapeData,
    ClosedArea,
    DerivedGeometry,
    ExclusionSpace,
    ExclusionSpaceInternal,
    Shelf,
    LayoutOpportunity,
    ShapeExclusions,
    LineClampAncestorChain,
    DevtoolsFlexInfo,
    GridLayoutData,
    GridLayoutSubtree,
    GridLayoutTree,
    GridTreeNode,
    GridLayoutTrackCollection,
    GridTrackBaselines,
    MathMLPaintInfo,
    NamingScope,
    NonOverflowingScrollRange,
    SnapArea,
    TextFitScale,
    UsedFont,
);

impl<T> Traceable for SplitAxisItem<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        SplitAxisItem::Trace(self, visitor);
    }
}
