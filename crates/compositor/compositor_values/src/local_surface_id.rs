//! Validity facet of viz::LocalSurfaceId named by both sides. Moved from
//! compositor_frame::unbounded, which re-exports it.

pub trait LocalSurfaceId: Default + Clone + PartialEq {
    fn is_valid(&self) -> bool;
}
