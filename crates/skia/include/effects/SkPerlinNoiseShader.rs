// Copyright 2013 Google Inc. BSD-style license.
// Source: include/effects/SkPerlinNoiseShader.h:45-50
// Native nullable tile-size pointer is Option<&SkISize>; factories retain no
// borrowed tile pointer. Explicit None projects the native default argument.
pub use crate::src::shaders::SkPerlinNoiseShaderImpl::{MakeFractalNoise, MakeTurbulence};
