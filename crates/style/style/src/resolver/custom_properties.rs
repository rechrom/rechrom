// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Facade exports for pure custom-property resolution and document adapters.
pub use style_resolve::custom_properties::*;

#[path = "attribute_variable_resolver.rs"]
pub mod attributes;
#[path = "environment_variable_resolver.rs"]
pub mod environment;
