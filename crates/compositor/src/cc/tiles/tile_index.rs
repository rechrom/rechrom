// Translation status: complete
// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_index.h:1-55

use std::hash::{Hash, Hasher};

// Source: cc/tiles/tile_index.h:14-40
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileIndex {
    // Source: cc/tiles/tile_index.h:35-36
    pub i: i32,
    // Source: cc/tiles/tile_index.h:38-39
    pub j: i32,
}

impl TileIndex {
    // Source: cc/tiles/tile_index.h:17-18
    pub const fn new(i: i32, j: i32) -> Self {
        Self { i, j }
    }
}

// Source: cc/tiles/tile_index.h:20-21
impl From<(i32, i32)> for TileIndex {
    fn from(index: (i32, i32)) -> Self {
        Self::new(index.0, index.1)
    }
}

// Source: cc/tiles/tile_index.h:23-33. Rust's derived equality and ordering
// compare the fields lexicographically, matching std::tie(i, j).

// Source: cc/tiles/tile_index.h:42-53
impl Hash for TileIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let value1 = self.i as u16;
        let value2 = self.j as u16;
        let value = ((value1 as u32) << 16) | value2 as u32;
        state.write_u32(value);
    }
}

// Source: cc/tiles/tile_index.h:41,55. Namespace and include-guard closure are
// represented by this Rust module and one-time module inclusion.
