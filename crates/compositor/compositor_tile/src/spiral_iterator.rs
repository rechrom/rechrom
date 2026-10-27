// Translation status: complete; owned by compositor_tile.
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/spiral_iterator.h:1-70
// Source: cc/base/spiral_iterator.cc:1-165

use super::index_rect::IndexRect;

// Source: cc/base/spiral_iterator.h:54
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    kUp,
    kLeft,
    kDown,
    kRight,
}

// Source: cc/base/spiral_iterator.h:12-67
pub struct SpiralIterator {
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

impl Default for SpiralIterator {
    // Source: cc/base/spiral_iterator.cc:13-18
    fn default() -> Self {
        Self {
            around_index_rect_: IndexRect::new(-1, -1, -1, -1),
            consider_index_rect_: IndexRect::new(-1, -1, -1, -1),
            ignore_index_rect_: IndexRect::new(-1, -1, -1, -1),
            index_x_: -1,
            index_y_: -1,
            direction_: Direction::kRight,
            delta_x_: 0,
            delta_y_: 0,
            current_step_: 0,
            horizontal_step_count_: 0,
            vertical_step_count_: 0,
        }
    }
}

impl SpiralIterator {
    // Source: cc/base/spiral_iterator.h:31-35
    // Source: cc/base/spiral_iterator.cc:20-44
    pub fn new(
        around_index_rect: IndexRect,
        consider_index_rect: IndexRect,
        ignore_index_rect: IndexRect,
    ) -> Self {
        let mut result = Self {
            around_index_rect_: around_index_rect,
            consider_index_rect_: consider_index_rect,
            ignore_index_rect_: ignore_index_rect,
            index_x_: around_index_rect.right(),
            index_y_: around_index_rect.bottom(),
            direction_: Direction::kRight,
            delta_x_: 1,
            delta_y_: 0,
            current_step_: around_index_rect.num_indices_x() - 1,
            horizontal_step_count_: around_index_rect.num_indices_x(),
            vertical_step_count_: around_index_rect.num_indices_y(),
        };
        result.Advance();
        result
    }

    // Source: cc/base/spiral_iterator.cc:46-48
    pub fn IsValid(&self) -> bool {
        self.index_x_ != -1 && self.index_y_ != -1
    }

    // Source: cc/base/spiral_iterator.h:39-40
    pub fn index_x(&self) -> i32 {
        self.index_x_
    }
    pub fn index_y(&self) -> i32 {
        self.index_y_
    }

    // Source: cc/base/spiral_iterator.h:43-47
    fn current_step_count(&self) -> i32 {
        if self.direction_ == Direction::kUp || self.direction_ == Direction::kDown {
            self.vertical_step_count_
        } else {
            self.horizontal_step_count_
        }
    }

    // Source: cc/base/spiral_iterator.cc:50-136
    pub fn Advance(&mut self) -> &mut Self {
        let mut cannot_hit_consider_count = 0;
        while cannot_hit_consider_count < 4 {
            if self.needs_direction_switch() {
                self.switch_direction();
            }
            self.index_x_ += self.delta_x_;
            self.index_y_ += self.delta_y_;
            self.current_step_ += 1;
            if self
                .consider_index_rect_
                .Contains(self.index_x_, self.index_y_)
            {
                cannot_hit_consider_count = 0;
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
                let mut can_hit_consider_rect = false;
                match self.direction_ {
                    Direction::kUp => {
                        if self.consider_index_rect_.valid_column(self.index_x_)
                            && self.consider_index_rect_.bottom() < self.index_y_
                        {
                            steps_to_take = self.index_y_ - self.consider_index_rect_.bottom() - 1;
                        }
                        can_hit_consider_rect |= self.consider_index_rect_.right() >= self.index_x_;
                    }
                    Direction::kLeft => {
                        if self.consider_index_rect_.valid_row(self.index_y_)
                            && self.consider_index_rect_.right() < self.index_x_
                        {
                            steps_to_take = self.index_x_ - self.consider_index_rect_.right() - 1;
                        }
                        can_hit_consider_rect |= self.consider_index_rect_.top() <= self.index_y_;
                    }
                    Direction::kDown => {
                        if self.consider_index_rect_.valid_column(self.index_x_)
                            && self.consider_index_rect_.top() > self.index_y_
                        {
                            steps_to_take = self.consider_index_rect_.top() - self.index_y_ - 1;
                        }
                        can_hit_consider_rect |= self.consider_index_rect_.left() <= self.index_x_;
                    }
                    Direction::kRight => {
                        if self.consider_index_rect_.valid_row(self.index_y_)
                            && self.consider_index_rect_.left() > self.index_x_
                        {
                            steps_to_take = self.consider_index_rect_.left() - self.index_x_ - 1;
                        }
                        can_hit_consider_rect |=
                            self.consider_index_rect_.bottom() >= self.index_y_;
                    }
                }
                steps_to_take = steps_to_take.min(max_steps);
                debug_assert!(steps_to_take >= 0);
                self.index_x_ += steps_to_take * self.delta_x_;
                self.index_y_ += steps_to_take * self.delta_y_;
                self.current_step_ += steps_to_take;
                if can_hit_consider_rect {
                    cannot_hit_consider_count = 0;
                } else {
                    cannot_hit_consider_count += 1;
                }
            }
        }
        if cannot_hit_consider_count >= 4 {
            self.index_x_ = -1;
            self.index_y_ = -1;
        }
        self
    }

    // Source: cc/base/spiral_iterator.cc:138-140
    fn needs_direction_switch(&self) -> bool {
        self.current_step_ >= self.current_step_count()
    }

    // Source: cc/base/spiral_iterator.cc:142-162
    fn switch_direction(&mut self) {
        let new_delta_x = self.delta_y_;
        self.delta_y_ = -self.delta_x_;
        self.delta_x_ = new_delta_x;
        self.current_step_ = 0;
        self.direction_ = match self.direction_ {
            Direction::kUp => Direction::kLeft,
            Direction::kLeft => Direction::kDown,
            Direction::kDown => Direction::kRight,
            Direction::kRight => Direction::kUp,
        };
        if self.direction_ == Direction::kRight || self.direction_ == Direction::kLeft {
            self.vertical_step_count_ += 1;
            self.horizontal_step_count_ += 1;
        }
    }
}

// Source: cc/base/spiral_iterator.h:5-11,37,49-52,69-70;
// Source: cc/base/spiral_iterator.cc:5-11,164-165.
// Includes, namespaces, default destruction, and C++ operators map to Rust
// imports, ownership, IsValid(), and Advance().
