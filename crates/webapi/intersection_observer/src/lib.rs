//! Chromium native IntersectionObserver, revision
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15. Pure registration, observation,
//! caching, threshold and delivery owners; no DOM/Layout/Browser/JS dependency.
#![allow(non_snake_case)]
mod controller;
mod model;
mod observer;
pub use controller::*;
pub use model::*;
pub use observer::*;
