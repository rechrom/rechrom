// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/index_rect.h:1-125
// Source: cc/base/index_rect.cc:1-27

// Source: cc/base/index_rect.h:13-109
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexRect {
    left_: i32,
    right_: i32,
    top_: i32,
    bottom_: i32,
}

impl IndexRect {
    // Source: cc/base/index_rect.h:66-67
    pub const fn new(left: i32, right: i32, top: i32, bottom: i32) -> Self {
        Self {
            left_: left,
            right_: right,
            top_: top,
            bottom_: bottom,
        }
    }

    // Source: cc/base/index_rect.h:71-74
    pub const fn left(&self) -> i32 {
        self.left_
    }
    pub const fn right(&self) -> i32 {
        self.right_
    }
    pub const fn top(&self) -> i32 {
        self.top_
    }
    pub const fn bottom(&self) -> i32 {
        self.bottom_
    }

    // Source: cc/base/index_rect.h:76-79
    pub const fn num_indices_x(&self) -> i32 {
        self.right_.wrapping_sub(self.left_).wrapping_add(1)
    }
    pub const fn num_indices_y(&self) -> i32 {
        self.bottom_.wrapping_sub(self.top_).wrapping_add(1)
    }

    // Source: cc/base/index_rect.h:81-88
    pub const fn is_valid(&self) -> bool {
        self.left_ <= self.right_ && self.top_ <= self.bottom_
    }
    pub const fn is_in_positive_quadrant(&self) -> bool {
        self.is_valid() && self.left_ >= 0 && self.top_ >= 0
    }

    // Source: cc/base/index_rect.h:90-97
    pub fn valid_column(&self, index_x: i32) -> bool {
        index_x >= self.left() && index_x <= self.right()
    }
    pub fn valid_row(&self, index_y: i32) -> bool {
        index_y >= self.top() && index_y <= self.bottom()
    }

    // Source: cc/base/index_rect.h:99-101
    // Source: cc/base/index_rect.cc:13-18
    pub fn ClampTo(&mut self, other: &Self) {
        self.left_ = self.left_.max(other.left());
        self.top_ = self.top_.max(other.top());
        self.right_ = self.right_.min(other.right());
        self.bottom_ = self.bottom_.min(other.bottom());
    }

    // Source: cc/base/index_rect.h:103-105
    // Source: cc/base/index_rect.cc:20-23
    pub fn Contains(&self, index_x: i32, index_y: i32) -> bool {
        index_x >= self.left_
            && index_x <= self.right_
            && index_y >= self.top_
            && index_y <= self.bottom_
    }

    // Source: cc/base/index_rect.h:107
    // Source: cc/base/index_rect.cc:25-27
    pub fn ToString(&self) -> String {
        format!(
            "{},{},{},{}",
            self.left(),
            self.right(),
            self.top(),
            self.bottom()
        )
    }
}

// Source: cc/base/index_rect.h:111-121. Derived PartialEq/Eq implements the
// field-wise equality and inequality operators.
// Source: cc/base/index_rect.h:5-11,123-125; cc/base/index_rect.cc:5-11.
// Includes, namespaces, include guards, and default destruction map to Rust's
// module system and ordinary ownership.
