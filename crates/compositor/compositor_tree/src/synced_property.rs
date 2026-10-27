// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/synced_property.h:1-222
// Rust ownership adaptation: the embedding may retain this value through Arc;
// reference counting does not change the source's conflict-resolution state.

use std::marker::PhantomData;
use std::ops::{Add, Sub};

// Source: cc/base/synced_property.h:24-28,195-218
// Rust associated types express the C++ mathematical-group template contract.
pub trait SyncedPropertyGroup {
    type BaseType: Copy + PartialEq;
    type DeltaType: Copy + PartialEq;
    fn IdentityBase() -> Self::BaseType;
    fn IdentityDelta() -> Self::DeltaType;
    fn ApplyDelta(value: Self::BaseType, delta: Self::DeltaType) -> Self::BaseType;
    fn DeltaBetweenBases(first: Self::BaseType, second: Self::BaseType) -> Self::DeltaType;
    fn DeltaBetweenDeltas(first: Self::DeltaType, second: Self::DeltaType) -> Self::DeltaType;
    fn CombineDeltas(first: Self::DeltaType, second: Self::DeltaType) -> Self::DeltaType;
}

// Source: cc/base/synced_property.h:12-28,162-185
pub struct SyncedProperty<T: SyncedPropertyGroup> {
    // Source: cc/base/synced_property.h:166-184
    pending_base_: T::BaseType,
    active_base_: T::BaseType,
    active_delta_: T::DeltaType,
    reflected_delta_in_main_tree_: T::DeltaType,
    next_reflected_delta_in_main_tree_: T::DeltaType,
    reflected_delta_in_pending_tree_: T::DeltaType,
    clobber_active_value_: bool,
}

impl<T: SyncedPropertyGroup> Default for SyncedProperty<T> {
    // Source: cc/base/synced_property.h:166-184
    fn default() -> Self {
        Self {
            pending_base_: T::IdentityBase(),
            active_base_: T::IdentityBase(),
            active_delta_: T::IdentityDelta(),
            reflected_delta_in_main_tree_: T::IdentityDelta(),
            next_reflected_delta_in_main_tree_: T::IdentityDelta(),
            reflected_delta_in_pending_tree_: T::IdentityDelta(),
            clobber_active_value_: false,
        }
    }
}

impl<T: SyncedPropertyGroup> SyncedProperty<T> {
    // Source: cc/base/synced_property.h:30-37
    pub fn Current(&self, is_active_tree: bool) -> T::BaseType {
        if is_active_tree {
            return T::ApplyDelta(self.active_base_, self.active_delta_);
        }
        T::ApplyDelta(self.pending_base_, self.PendingDelta())
    }

    // Source: cc/base/synced_property.h:39-50
    pub fn SetCurrent(&mut self, current: T::BaseType) -> bool {
        let delta = T::DeltaBetweenBases(current, self.active_base_);
        if self.active_delta_ == delta {
            return false;
        }
        self.active_delta_ = delta;
        true
    }

    // Source: cc/base/synced_property.h:52-54
    pub fn Delta(&self) -> T::DeltaType {
        self.active_delta_
    }

    // Source: cc/base/synced_property.h:56-64
    pub fn PullDeltaForMainThread(&mut self, next_bmf: bool) -> T::DeltaType {
        // Evaluate the immutable value before the exclusive field borrow.
        // This is solely the Rust borrow equivalent of target = UnsentDelta().
        let unsent_delta = self.UnsentDelta();
        let target = if next_bmf {
            &mut self.next_reflected_delta_in_main_tree_
        } else {
            &mut self.reflected_delta_in_main_tree_
        };
        debug_assert!(*target == T::IdentityDelta());
        *target = unsent_delta;
        *target
    }

    // Source: cc/base/synced_property.h:66-76
    pub fn PushMainToPending(&mut self, main_thread_value: T::BaseType) -> bool {
        self.reflected_delta_in_pending_tree_ = self.reflected_delta_in_main_tree_;
        self.reflected_delta_in_main_tree_ = self.next_reflected_delta_in_main_tree_;
        self.next_reflected_delta_in_main_tree_ = T::IdentityDelta();
        self.pending_base_ = main_thread_value;
        self.Current(false) != main_thread_value
    }

    // Source: cc/base/synced_property.h:78-101
    pub fn PushPendingToActive(&mut self) -> bool {
        let pending_value_before_push = self.Current(false);
        let active_value_before_push = self.Current(true);
        self.active_base_ = self.pending_base_;
        self.active_delta_ = self.PendingDelta();
        self.reflected_delta_in_pending_tree_ = T::IdentityDelta();
        self.clobber_active_value_ = false;
        let current_active_value = self.Current(true);
        pending_value_before_push != current_active_value
            || active_value_before_push != current_active_value
    }

    // Source: cc/base/synced_property.h:103-131
    pub fn AbortCommit(&mut self, next_bmf: bool, main_frame_applied_deltas: bool) {
        if next_bmf {
            if main_frame_applied_deltas {
                self.reflected_delta_in_main_tree_ = T::CombineDeltas(
                    self.reflected_delta_in_main_tree_,
                    self.next_reflected_delta_in_main_tree_,
                );
            }
            self.next_reflected_delta_in_main_tree_ = T::IdentityDelta();
        } else {
            if main_frame_applied_deltas {
                let delta = self.reflected_delta_in_main_tree_;
                self.pending_base_ = T::ApplyDelta(self.pending_base_, delta);
                self.active_base_ = T::ApplyDelta(self.active_base_, delta);
                self.active_delta_ = T::DeltaBetweenDeltas(self.active_delta_, delta);
            }
            self.reflected_delta_in_main_tree_ = T::IdentityDelta();
        }
    }

    // Source: cc/base/synced_property.h:133-137
    pub fn reflected_delta_in_main_tree(&self) -> T::DeltaType {
        self.reflected_delta_in_main_tree_
    }

    // Source: cc/base/synced_property.h:138-140
    pub fn next_reflected_delta_in_main_tree(&self) -> T::DeltaType {
        self.next_reflected_delta_in_main_tree_
    }

    // Source: cc/base/synced_property.h:141-143
    pub fn PendingBase(&self) -> T::BaseType {
        self.pending_base_
    }

    // Source: cc/base/synced_property.h:144
    pub fn ActiveBase(&self) -> T::BaseType {
        self.active_base_
    }

    // Source: cc/base/synced_property.h:146-154
    pub fn PendingDelta(&self) -> T::DeltaType {
        if self.clobber_active_value_ {
            return T::IdentityDelta();
        }
        T::DeltaBetweenDeltas(self.active_delta_, self.reflected_delta_in_pending_tree_)
    }

    // Source: cc/base/synced_property.h:156-158
    pub fn UnsentDelta(&self) -> T::DeltaType {
        T::DeltaBetweenDeltas(self.PendingDelta(), self.reflected_delta_in_main_tree_)
    }

    // Source: cc/base/synced_property.h:160
    pub fn set_clobber_active_value(&mut self) {
        self.clobber_active_value_ = true;
    }
}

// Source: cc/base/synced_property.h:187-206
pub struct AdditionGroup<BaseT, DeltaT = BaseT>(PhantomData<(BaseT, DeltaT)>);

impl<BaseT, DeltaT> SyncedPropertyGroup for AdditionGroup<BaseT, DeltaT>
where
    BaseT: Copy + PartialEq + Default + Add<DeltaT, Output = BaseT> + Sub<BaseT, Output = DeltaT>,
    DeltaT:
        Copy + PartialEq + Default + Add<DeltaT, Output = DeltaT> + Sub<DeltaT, Output = DeltaT>,
{
    type BaseType = BaseT;
    type DeltaType = DeltaT;
    // Source: cc/base/synced_property.h:200
    fn IdentityBase() -> BaseT {
        BaseT::default()
    }
    // Source: cc/base/synced_property.h:201
    fn IdentityDelta() -> DeltaT {
        DeltaT::default()
    }
    // Source: cc/base/synced_property.h:202
    fn ApplyDelta(value: BaseT, delta: DeltaT) -> BaseT {
        value + delta
    }
    // Source: cc/base/synced_property.h:203
    fn DeltaBetweenBases(first: BaseT, second: BaseT) -> DeltaT {
        first - second
    }
    // Source: cc/base/synced_property.h:204
    fn DeltaBetweenDeltas(first: DeltaT, second: DeltaT) -> DeltaT {
        first - second
    }
    // Source: cc/base/synced_property.h:205
    fn CombineDeltas(first: DeltaT, second: DeltaT) -> DeltaT {
        first + second
    }
}

// Source: cc/base/synced_property.h:208-218
pub struct ScaleGroup;
impl SyncedPropertyGroup for ScaleGroup {
    type BaseType = f32;
    type DeltaType = f32;
    // Source: cc/base/synced_property.h:212
    fn IdentityBase() -> f32 {
        1.0
    }
    // Source: cc/base/synced_property.h:213
    fn IdentityDelta() -> f32 {
        1.0
    }
    // Source: cc/base/synced_property.h:214
    fn ApplyDelta(value: f32, delta: f32) -> f32 {
        value * delta
    }
    // Source: cc/base/synced_property.h:215
    fn DeltaBetweenBases(first: f32, second: f32) -> f32 {
        first / second
    }
    // Source: cc/base/synced_property.h:216
    fn DeltaBetweenDeltas(first: f32, second: f32) -> f32 {
        first / second
    }
    // Source: cc/base/synced_property.h:217
    fn CombineDeltas(first: f32, second: f32) -> f32 {
        first * second
    }
}
