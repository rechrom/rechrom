use crate::{adapter::region_backend::SkRegionBackend, cc::base::region::Region};

pub type TouchActionRegion<B> = compositor_input::touch_action_region::TouchActionRegion<Region<B>>;
