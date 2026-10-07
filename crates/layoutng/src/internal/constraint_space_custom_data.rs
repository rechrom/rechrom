#![allow(non_snake_case)]

use std::sync::Arc;

use super::constraint_space::RareData;
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::custom_layout_payload::SerializedScriptValue;

// cpp: layoutng/internal/constraint_space_custom_data.cc:14-16
// Default construction, copying, and destruction are Rust value operations;
// Arc retains the source scoped_refptr ownership of the payload.
#[derive(Clone, Default)]
pub struct CustomData {
    pub data: Option<Arc<SerializedScriptValue>>,
}

impl CustomData {
    // cpp: layoutng/internal/constraint_space.h:1470-1474
    pub fn MaySkipLayout(&self, other: &Self) -> bool {
        match (&self.data, &other.data) {
            (None, None) => true,
            (Some(left), Some(right)) => Arc::ptr_eq(left, right),
            _ => false,
        }
    }

    pub fn IsInitialForMaySkipLayout(&self) -> bool {
        self.data.is_none()
    }
}

impl RareData {
    // cpp: layoutng/internal/constraint_space_custom_data.cc:18-21
    pub fn SetCustomLayoutData(&mut self, custom_layout_data: Option<Arc<SerializedScriptValue>>) {
        unsafe { &mut *self.EnsureCustomData() }.data = custom_layout_data;
    }
}

impl ConstraintSpaceBuilder {
    // cpp: layoutng/internal/constraint_space_custom_data.cc:23-33
    pub fn SetCustomLayoutData(&mut self, custom_layout_data: Option<Arc<SerializedScriptValue>>) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_custom_layout_data_set_);
            self.is_custom_layout_data_set_ = true;
        }
        if let Some(data) = custom_layout_data {
            unsafe { &mut *self.EnsureRareData() }.SetCustomLayoutData(Some(data));
        }
    }
}
