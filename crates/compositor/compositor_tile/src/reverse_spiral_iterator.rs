// Translation status: complete; owned by compositor_tile.
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/reverse_spiral_iterator.h:1-72
// Source: cc/base/reverse_spiral_iterator.cc:1-204

use super::index_rect::IndexRect;

// Source: cc/base/reverse_spiral_iterator.h:56
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    kLeft,
    kUp,
    kRight,
    kDown,
}

// Source: cc/base/reverse_spiral_iterator.h:12-69
pub struct ReverseSpiralIterator {
    around_index_rect_: IndexRect,
    consider_index_rect_: IndexRect,
    ignore_index_rect_: IndexRect,
    index_x_: i32,
    index_y_: i32,
    direction_: Direction,
    delta_x_: i32,
    delta_y_: i32,
    current_step_: i32,
    horizontal_step_count_: i32,
    vertical_step_count_: i32,
}

impl Default for ReverseSpiralIterator {
    // Source: cc/base/reverse_spiral_iterator.cc:13-18
    fn default() -> Self {
        Self {
            around_index_rect_: IndexRect::new(-1, -1, -1, -1),
            consider_index_rect_: IndexRect::new(-1, -1, -1, -1),
            ignore_index_rect_: IndexRect::new(-1, -1, -1, -1),
            index_x_: -1,
            index_y_: -1,
            direction_: Direction::kLeft,
            delta_x_: 0,
            delta_y_: 0,
            current_step_: 0,
            horizontal_step_count_: 0,
            vertical_step_count_: 0,
        }
    }
}

impl ReverseSpiralIterator {
    // Source: cc/base/reverse_spiral_iterator.h:31-35
    // Source: cc/base/reverse_spiral_iterator.cc:20-80
    pub fn new(
        around_index_rect: IndexRect,
        consider_index_rect: IndexRect,
        ignore_index_rect: IndexRect,
    ) -> Self {
        let mut max_distance = 0;
        max_distance = max_distance.max(around_index_rect.top() - consider_index_rect.top());
        max_distance = max_distance.max(around_index_rect.left() - consider_index_rect.left());
        max_distance = max_distance.max(consider_index_rect.bottom() - around_index_rect.bottom());
        max_distance = max_distance.max(consider_index_rect.right() - around_index_rect.right());
        let mut result = Self {
            around_index_rect_: around_index_rect,
            consider_index_rect_: consider_index_rect,
            ignore_index_rect_: ignore_index_rect,
            index_x_: around_index_rect.right() + max_distance + 1,
            index_y_: around_index_rect.bottom() + max_distance,
            direction_: Direction::kLeft,
            delta_x_: -1,
            delta_y_: 0,
            current_step_: 0,
            horizontal_step_count_: around_index_rect.num_indices_x() + 2 * max_distance,
            vertical_step_count_: around_index_rect.num_indices_y() + 2 * max_distance,
        };
        result.Advance();
        result
    }

    // Source: cc/base/reverse_spiral_iterator.cc:82-84
    pub fn IsValid(&self) -> bool {
        self.index_x_ != -1 && self.index_y_ != -1
    }

    // Source: cc/base/reverse_spiral_iterator.h:41-42
    pub fn index_x(&self) -> i32 {
        self.index_x_
    }
    pub fn index_y(&self) -> i32 {
        self.index_y_
    }

    // Source: cc/base/reverse_spiral_iterator.h:47-51
    fn current_step_count(&self) -> i32 {
        if self.direction_ == Direction::kUp || self.direction_ == Direction::kDown {
            self.vertical_step_count_
        } else {
            self.horizontal_step_count_
        }
    }

    // Source: cc/base/reverse_spiral_iterator.cc:86-173
    pub fn Advance(&mut self) -> &mut Self {
        while !self
            .around_index_rect_
            .Contains(self.index_x_, self.index_y_)
        {
            if self.needs_direction_switch() {
                self.switch_direction();
            }
            self.index_x_ += self.delta_x_;
            self.index_y_ += self.delta_y_;
            self.current_step_ += 1;
            if self
                .around_index_rect_
                .Contains(self.index_x_, self.index_y_)
            {
                break;
            } else if self
                .consider_index_rect_
                .Contains(self.index_x_, self.index_y_)
            {
                if !self
                    .ignore_index_rect_
                    .Contains(self.index_x_, self.index_y_)
                {
                    break;
                }
                let steps_to_edge = match self.direction_ {
                    Direction::kUp => self.index_y_ - self.ignore_index_rect_.top(),
                    Direction::kLeft => self.index_x_ - self.ignore_index_rect_.left(),
                    Direction::kDown => self.ignore_index_rect_.bottom() - self.index_y_,
                    Direction::kRight => self.ignore_index_rect_.right() - self.index_x_,
                };
                let max_steps = self.current_step_count() - self.current_step_;
                let steps_to_take = steps_to_edge.min(max_steps);
                debug_assert!(steps_to_take >= 0);
                self.index_x_ += steps_to_take * self.delta_x_;
                self.index_y_ += steps_to_take * self.delta_y_;
                self.current_step_ += steps_to_take;
            } else {
                let max_steps = self.current_step_count() - self.current_step_;
                let mut steps_to_take = max_steps;
                match self.direction_ {
                    Direction::kUp => {
                        if self.consider_index_rect_.valid_column(self.index_x_)
                            && self.consider_index_rect_.bottom() < self.index_y_
                        {
                            steps_to_take = self.index_y_ - self.consider_index_rect_.bottom() - 1;
                        }
                    }
                    Direction::kLeft => {
                        if self.consider_index_rect_.valid_row(self.index_y_)
                            && self.consider_index_rect_.right() < self.index_x_
                        {
                            steps_to_take = self.index_x_ - self.consider_index_rect_.right() - 1;
                        }
                    }
                    Direction::kDown => {
                        if self.consider_index_rect_.valid_column(self.index_x_)
                            && self.consider_index_rect_.top() > self.index_y_
                        {
                            steps_to_take = self.consider_index_rect_.top() - self.index_y_ - 1;
                        }
                    }
                    Direction::kRight => {
                        if self.consider_index_rect_.valid_row(self.index_y_)
                            && self.consider_index_rect_.left() > self.index_x_
                        {
                            steps_to_take = self.consider_index_rect_.left() - self.index_x_ - 1;
                        }
                    }
                }
                steps_to_take = steps_to_take.min(max_steps);
                debug_assert!(steps_to_take >= 0);
                self.index_x_ += steps_to_take * self.delta_x_;
                self.index_y_ += steps_to_take * self.delta_y_;
                self.current_step_ += steps_to_take;
            }
        }
        if self
            .around_index_rect_
            .Contains(self.index_x_, self.index_y_)
        {
            self.index_x_ = -1;
            self.index_y_ = -1;
        }
        self
    }

    // Source: cc/base/reverse_spiral_iterator.cc:175-177
    fn needs_direction_switch(&self) -> bool {
        self.current_step_ >= self.current_step_count()
    }

    // Source: cc/base/reverse_spiral_iterator.cc:179-201
    fn switch_direction(&mut self) {
        let new_delta_y = self.delta_x_;
        self.delta_x_ = -self.delta_y_;
        self.delta_y_ = new_delta_y;
        self.current_step_ = 0;
        self.direction_ = match self.direction_ {
            Direction::kLeft => Direction::kUp,
            Direction::kUp => Direction::kRight,
            Direction::kRight => Direction::kDown,
            Direction::kDown => Direction::kLeft,
        };
        if self.direction_ == Direction::kUp || self.direction_ == Direction::kDown {
            self.vertical_step_count_ -= 1;
            self.horizontal_step_count_ -= 1;
            debug_assert!(self.horizontal_step_count_ >= 1);
            debug_assert!(self.vertical_step_count_ >= 1);
        }
    }
}

// Source: cc/base/reverse_spiral_iterator.h:5-11,37-39,44-45,53-54,71-72;
// Source: cc/base/reverse_spiral_iterator.cc:5-11,203-204.
// Includes, namespaces, move assignment, default destruction, and C++
// operators map to Rust imports, ordinary moves, IsValid(), and Advance().
