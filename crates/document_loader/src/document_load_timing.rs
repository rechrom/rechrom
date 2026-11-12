//! Source DocumentLoadTiming.cc62-105/126-144. DOM's render-blocking input
//! borrows this same scalar timing owner; the loader owns its construction and
//! navigation-start transitions. This facade does not copy a timing model.
pub use dom::render_blocking_resource_manager::DocumentLoadTiming;
