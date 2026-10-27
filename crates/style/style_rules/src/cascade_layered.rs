// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/cascade_layered.h:17-72

use crate::cascade_layer::CascadeLayer;

// Rust ownership replaces AddMemberIfNeeded<T>/Member<T>; the value and the
// optional layer identity otherwise have the same meaning as the C++ record.
#[derive(Clone, Debug, Default)]
pub struct CascadeLayered<T> {
    pub value: T,
    pub layer: Option<CascadeLayer>,
}

impl<T> CascadeLayered<T> {
    pub fn new(value: T, layer: Option<CascadeLayer>) -> Self {
        Self { value, layer }
    }

    pub fn map<U>(self, convert: impl FnOnce(T) -> U) -> CascadeLayered<U> {
        CascadeLayered {
            value: convert(self.value),
            layer: self.layer,
        }
    }
}
