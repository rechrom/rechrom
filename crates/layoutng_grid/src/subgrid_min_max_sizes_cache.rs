use foundation::{Member, Traceable, Visitor};
use layoutng_assembly::internal::grid_layout_data::GridLayoutData;
use layoutng_assembly::internal::grid_track_collection::GridLayoutTrackCollection;
use layoutng_assembly::internal::min_max_sizes::MinMaxSizes;

// cpp: layoutng_grid/subgrid_min_max_sizes_cache.h:14-43
// Deleted C++ default/copy constructors map to no Default/Clone implementation.
pub struct SubgridMinMaxSizesCache {
    opposite_axis_subgridded_tracks_: Member<GridLayoutTrackCollection>,
    cached_min_max_sizes_: MinMaxSizes,
}
impl SubgridMinMaxSizesCache {
    // cpp: layoutng_grid/subgrid_min_max_sizes_cache.h:21-25
    pub fn new(min_max_sizes: MinMaxSizes, layout_data: &GridLayoutData) -> Self {
        Self {
            opposite_axis_subgridded_tracks_: Member::from_ptr(
                layout_data.OnlySubgriddedCollection().cast_mut(),
            ),
            cached_min_max_sizes_: min_max_sizes,
        }
    }
    // cpp: layoutng_grid/subgrid_min_max_sizes_cache.h:27-30
    pub fn IsValidFor(&self, layout_data: &GridLayoutData) -> bool {
        unsafe {
            &*layout_data.OnlySubgriddedCollection()
                == &*self.opposite_axis_subgridded_tracks_.Get()
        }
    }
    // cpp: layoutng_grid/subgrid_min_max_sizes_cache.h:32-32
    pub fn CachedMinMaxSizes(&self) -> &MinMaxSizes {
        &self.cached_min_max_sizes_
    }
    // cpp: layoutng_grid/subgrid_min_max_sizes_cache.h:34-36
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.opposite_axis_subgridded_tracks_);
    }
}
impl Traceable for SubgridMinMaxSizesCache {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        SubgridMinMaxSizesCache::Trace(self, visitor);
    }
}
