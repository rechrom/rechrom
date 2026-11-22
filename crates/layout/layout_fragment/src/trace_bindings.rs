// Bind the source Trace methods of fragment data to the layout heap without
// moving their translated bodies out of their owning files. Rust coherence
// requires `impl Traceable` (a foundation trait) to sit in the crate that
// declares the type, so these left base's `layoutng_trace_bindings.rs` with
// their types.
// cpp: foundation/blink_base/heap/trace_traits.h:20-28
use foundation::{Traceable, Visitor};

use crate::internal::devtools_flex_info::DevtoolsFlexInfo;
use crate::internal::grid_layout_data::{
    GridLayoutData, GridLayoutSubtree, GridLayoutTree, GridTreeNode,
};
use crate::internal::grid_track_collection::{GridLayoutTrackCollection, GridTrackBaselines};
use crate::internal::mathml_paint_info::MathMLPaintInfo;
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
    DevtoolsFlexInfo,
    GridLayoutData,
    GridLayoutSubtree,
    GridLayoutTree,
    GridTreeNode,
    GridLayoutTrackCollection,
    GridTrackBaselines,
    MathMLPaintInfo,
    TextFitScale,
    UsedFont,
);

impl<T> Traceable for SplitAxisItem<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        SplitAxisItem::Trace(self, visitor);
    }
}
