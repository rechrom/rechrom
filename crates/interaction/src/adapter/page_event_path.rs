//! Page's read-only view of the canonical translated dispatch path.
//!
//! DOM freezes tree/slot facts; the translated EventPath owns dispatch contexts
//! and composedPath filtering. This adapter retains no independent path.
pub use super::event::{EventDispatchState, EventPath};
