// Translation status: complete
// Copyright 2013 The Chromium Authors. BSD-style license in LICENSE.
// Source: cc/paint/filter_operations.h:1-112; cc/paint/filter_operations.cc:1-267.
#![allow(non_snake_case, dead_code)]
use super::filter_operation::{FilterOperation, FilterType};
use foundation::gfx::Rect;
use skia::include::core::SkRRect::SkMatrix3x3;
// Source: cc/paint/filter_operations.h:28-110; cc/paint/filter_operations.cc:23-61.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilterOperations {
    operations_: Vec<FilterOperation>,
}
impl FilterOperations {
    pub fn new(operations: Vec<FilterOperation>) -> Self {
        Self {
            operations_: operations,
        }
    }
    pub fn Append(&mut self, filter: &FilterOperation) {
        self.operations_.push(filter.clone());
    }
    pub fn Clear(&mut self) {
        self.operations_.clear();
    }
    pub fn IsEmpty(&self) -> bool {
        self.operations_.is_empty()
    }
    pub fn size(&self) -> usize {
        self.operations_.len()
    }
    pub fn operations(&self) -> &[FilterOperation] {
        &self.operations_
    }
    pub fn at(&self, index: usize) -> &FilterOperation {
        &self.operations_[index]
    }
    // Source: cc/paint/filter_operations.cc:63-88.
    pub fn MapRect(&self, rect: Rect, ctm: Option<&SkMatrix3x3>) -> Rect {
        self.operations_
            .iter()
            .fold(rect, |r, op| op.MapRect(r, ctm))
    }
    pub fn MapRectReverse(&self, rect: Rect, ctm: &SkMatrix3x3) -> Rect {
        self.operations_
            .iter()
            .rev()
            .fold(rect, |r, op| op.MapRectReverse(r, ctm))
    }
    pub fn ExpandRect(&self, rect: Rect, ctm: &SkMatrix3x3) -> Rect {
        let mut result = self.MapRect(rect, Some(ctm));
        result.Union(self.MapRectReverse(rect, ctm));
        result
    }
    // Source: cc/paint/filter_operations.cc:90-159.
    pub fn HasFilterThatMovesPixels(&self) -> bool {
        use FilterType::*;
        for op in &self.operations_ {
            match op.r#type() {
                BLUR | DROP_SHADOW | ZOOM | OFFSET | REFERENCE => return true,
                OPACITY
                | COLOR_MATRIX
                | GRAYSCALE
                | SEPIA
                | SATURATE
                | HUE_ROTATE
                | INVERT
                | BRIGHTNESS
                | CONTRAST
                | SATURATING_BRIGHTNESS
                | ALPHA_THRESHOLD => {}
            }
        }
        false
    }
    pub fn HasFilterThatAffectsOpacity(&self) -> bool {
        use FilterType::*;
        for op in &self.operations_ {
            match op.r#type() {
                OPACITY | BLUR | DROP_SHADOW | ZOOM | REFERENCE | ALPHA_THRESHOLD => return true,
                COLOR_MATRIX => {
                    let m = op.matrix();
                    if m[15] != 0. || m[16] != 0. || m[17] != 0. || m[18] != 1. || m[19] != 0. {
                        return true;
                    }
                }
                GRAYSCALE
                | SEPIA
                | SATURATE
                | HUE_ROTATE
                | INVERT
                | BRIGHTNESS
                | CONTRAST
                | SATURATING_BRIGHTNESS
                | OFFSET => {}
            }
        }
        false
    }
    pub fn HasReferenceFilter(&self) -> bool {
        self.HasFilterOfType(FilterType::REFERENCE)
    }
    pub fn HasFilterOfType(&self, t: FilterType) -> bool {
        self.operations_.iter().any(|op| op.r#type() == t)
    }
    // Source: cc/paint/filter_operations.cc:161-214.
    pub fn Blend(&self, from: &Self, progress: f64) -> Self {
        if self.HasReferenceFilter() || from.HasReferenceFilter() {
            return self.clone();
        }
        let longer = from.size() > self.size();
        let (short, long) = if self.size() == from.size() {
            (self.size(), self.size())
        } else if longer {
            (self.size(), from.size())
        } else {
            (from.size(), self.size())
        };
        for i in 0..short {
            if from.at(i).r#type() != self.at(i).r#type() {
                return self.clone();
            }
        }
        let mut result = Self::default();
        for i in 0..short {
            result.Append(&FilterOperation::Blend(
                Some(from.at(i)),
                Some(self.at(i)),
                progress,
            ));
        }
        if longer {
            for i in short..long {
                result.Append(&FilterOperation::Blend(Some(from.at(i)), None, progress));
            }
        } else {
            for i in short..long {
                result.Append(&FilterOperation::Blend(None, Some(self.at(i)), progress));
            }
        }
        result
    }
    // Source: cc/paint/filter_operations.cc:216-249.
    pub fn AllowsLCDText(&self) -> bool {
        use FilterType::*;
        if self.IsEmpty() {
            return true;
        }
        if self.size() > 1 {
            return false;
        }
        let op = &self.operations_[0];
        match op.r#type() {
            GRAYSCALE | SEPIA | OPACITY | BRIGHTNESS | BLUR | DROP_SHADOW => true,
            CONTRAST | SATURATE => op.amount() <= 1.,
            INVERT => op.amount() <= 0.5,
            COLOR_MATRIX
            | HUE_ROTATE
            | ZOOM
            | REFERENCE
            | SATURATING_BRIGHTNESS
            | ALPHA_THRESHOLD
            | OFFSET => false,
        }
    }
    // Source: cc/paint/filter_operations.cc:251-265.
    pub fn AsValueInto(&self) -> Vec<serde_json::Value> {
        self.operations_
            .iter()
            .map(FilterOperation::AsValueInto)
            .collect()
    }
    pub fn ToString(&self) -> String {
        serde_json::json!({"FilterOperations":self.AsValueInto()}).to_string()
    }
}
