// Translation status: complete
// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/animated_paint_worklet_tracker.h:1-110;
//         cc/trees/animated_paint_worklet_tracker.cc:1-14.
use crate::adapter::paint_worklet::{PaintWorkletInput, PaintWorkletTypes, PropertyValue};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
// Source: cc/trees/animated_paint_worklet_tracker.h:40-43.
pub type PropertyChange<V> = (V, V);
pub type PropertyChangeMap<K, V> = BTreeMap<K, PropertyChange<V>>;
// Source: cc/trees/animated_paint_worklet_tracker.h:85-102;
//         cc/trees/animated_paint_worklet_tracker.cc:20-28.
#[derive(Clone, Default)]
struct PropertyState<V: PropertyValue> {
    animation_value: V,
    last_animation_value: V,
}
impl<V: PropertyValue> PropertyState<V> {
    fn new(value: V) -> Self {
        Self {
            animation_value: value,
            last_animation_value: V::default(),
        }
    }
}
// Source: cc/trees/animated_paint_worklet_tracker.h:33-106;
//         cc/trees/animated_paint_worklet_tracker.cc:16-18.
pub struct AnimatedPaintWorkletTracker<B: PaintWorkletTypes> {
    input_properties_: BTreeMap<B::PropertyKey, PropertyState<B::PropertyValue>>,
    input_properties_animated_on_impl_: BTreeSet<B::PropertyKey>,
}
impl<B: PaintWorkletTypes> Default for AnimatedPaintWorkletTracker<B> {
    fn default() -> Self {
        Self {
            input_properties_: BTreeMap::new(),
            input_properties_animated_on_impl_: BTreeSet::new(),
        }
    }
}
impl<B: PaintWorkletTypes> AnimatedPaintWorkletTracker<B> {
    // Source: cc/trees/animated_paint_worklet_tracker.cc:30-48.
    pub fn OnCustomPropertyMutated(&mut self, key: B::PropertyKey, value: B::PropertyValue) {
        let Some(state) = self.input_properties_.get_mut(&key) else {
            return;
        };
        state.animation_value = value;
        self.input_properties_animated_on_impl_.insert(key);
    }
    // Source: cc/trees/animated_paint_worklet_tracker.cc:51-68.
    pub fn TakeAndResetAnimatedProperties(
        &mut self,
    ) -> PropertyChangeMap<B::PropertyKey, B::PropertyValue> {
        let mut result = BTreeMap::new();
        for key in &self.input_properties_animated_on_impl_ {
            let Some(state) = self.input_properties_.get_mut(key) else {
                continue;
            };
            result.insert(
                key.clone(),
                (
                    state.last_animation_value.clone(),
                    state.animation_value.clone(),
                ),
            );
            state.last_animation_value = state.animation_value.clone();
        }
        self.input_properties_animated_on_impl_.clear();
        result
    }
    // Source: cc/trees/animated_paint_worklet_tracker.cc:71-74.
    pub fn GetPropertyAnimationValue(&self, key: &B::PropertyKey) -> B::PropertyValue {
        self.input_properties_
            .get(key)
            .expect("source requires tracked input property")
            .animation_value
            .clone()
    }
    // Source: cc/trees/animated_paint_worklet_tracker.cc:76-84.
    pub fn UpdatePaintWorkletInputProperties(
        &mut self,
        inputs: &[(Arc<B::Input>, B::PaintImageId)],
    ) {
        for (input, _) in inputs {
            for key in input.GetPropertyKeys() {
                self.input_properties_.entry(key.clone()).or_default();
            }
        }
    }
    // Source: cc/trees/animated_paint_worklet_tracker.cc:86-97.
    pub fn ClearUnusedInputProperties(&mut self, used: BTreeSet<B::PropertyKey>) {
        self.input_properties_.retain(|key, _| used.contains(key));
        for state in self.input_properties_.values_mut() {
            state.animation_value.reset();
        }
    }
    // Source: cc/trees/animated_paint_worklet_tracker.cc:99-101.
    pub fn HasInputPropertiesAnimatedOnImpl(&self) -> bool {
        !self.input_properties_animated_on_impl_.is_empty()
    }
}
