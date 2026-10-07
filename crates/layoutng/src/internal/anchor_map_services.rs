#![allow(non_snake_case)]

use super::layout_node_metadata::{Element, TransformAnimation};
use super::layout_object::LayoutObject;

pub mod anchor_map_internal {
    use super::*;

    // cpp: layoutng/internal/anchor_map_services.h:12-12
    // cpp: layoutng/internal/anchor_map_services.cc:18-28
    pub fn HasRunningTransformAnimation(layout_object: &LayoutObject) -> bool {
        let node = layout_object.GetNode();
        if node.is_null() || !unsafe { &*node }.IsElementNode() {
            return false;
        }
        let element = unsafe { &*node.cast::<Element>() };
        element.HasActiveTransformAnimation(TransformAnimation::kTransform)
            || element.HasActiveTransformAnimation(TransformAnimation::kTranslate)
            || element.HasActiveTransformAnimation(TransformAnimation::kRotate)
            || element.HasActiveTransformAnimation(TransformAnimation::kScale)
    }
}
