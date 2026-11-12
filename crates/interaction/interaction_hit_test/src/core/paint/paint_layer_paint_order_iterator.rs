/*
 * Copyright (C) 2013 Google Inc. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.h:1-148
// Source: third_party/blink/renderer/core/paint/paint_layer_paint_order_iterator.cc:1-179
// Rust modules replace include guards, namespaces, includes and allocation macros.

pub use super::paint_layer::PaintLayerIteration;

// Source: paint_layer_paint_order_iterator.h:36,55-60,73-78; .cc:40-76,83-152
// These borrowed projections read the canonical PaintLayer links and stacking
// lists. They do not retain another layer tree or copy the stacking vectors.
pub trait PaintLayer {
    fn FirstChild(&self) -> Option<&dyn PaintLayer>;
    fn LastChild(&self) -> Option<&dyn PaintLayer>;
    fn NextSibling(&self) -> Option<&dyn PaintLayer>;
    fn PreviousSibling(&self) -> Option<&dyn PaintLayer>;
    fn IsStacked(&self) -> bool;
    fn StackingNode(&self) -> Option<&dyn PaintLayerStackingNode>;

    // Source: paint_layer.h:877-894
    // Returns the previous state, exactly as the source debug scope saves it.
    #[cfg(debug_assertions)]
    fn SetLayerListMutationAllowed(&self, allowed: bool) -> bool;
}

pub trait PaintLayerStackingNode {
    fn NegZOrderList(&self) -> &dyn PaintLayerList;
    fn PosZOrderList(&self) -> &dyn PaintLayerList;
    fn LayersPaintingOverlayOverflowControlsAfter(
        &self,
        layer: &dyn PaintLayer,
    ) -> Option<&dyn PaintLayerList>;
}

// A borrowed view of the existing GCedHeapVector owner; adapters need not
// allocate a parallel vector of layer references to implement this port.
pub trait PaintLayerList {
    fn len(&self) -> usize;
    fn at(&self, index: usize) -> &dyn PaintLayer;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Source: paint_layer.h:877-894
#[cfg(debug_assertions)]
struct PaintLayerListMutationDetector<'a> {
    layer_: &'a dyn PaintLayer,
    previous_mutation_allowed_state_: bool,
}

#[cfg(debug_assertions)]
impl<'a> PaintLayerListMutationDetector<'a> {
    fn new(layer: &'a dyn PaintLayer) -> Self {
        Self {
            layer_: layer,
            previous_mutation_allowed_state_: layer.SetLayerListMutationAllowed(false),
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for PaintLayerListMutationDetector<'_> {
    fn drop(&mut self) {
        self.layer_
            .SetLayerListMutationAllowed(self.previous_mutation_allowed_state_);
    }
}

// Source: paint_layer_paint_order_iterator.h:41-46
pub trait PaintLayerPaintOrderIteratorBase<'a> {
    fn Next(&mut self) -> Option<&'a dyn PaintLayer>;
}

// Source: paint_layer_paint_order_iterator.h:48-89
pub struct PaintLayerPaintOrderIterator<'a> {
    root_: &'a dyn PaintLayer,
    remaining_children_: u32,
    index_: usize,
    current_normal_flow_child_: Option<&'a dyn PaintLayer>,
    #[cfg(debug_assertions)]
    _mutation_detector_: PaintLayerListMutationDetector<'a>,
}

impl<'a> PaintLayerPaintOrderIterator<'a> {
    // Source: paint_layer_paint_order_iterator.h:55-69
    // No Clone/Copy implementation, matching deleted source copy operations.
    pub fn new(root: &'a dyn PaintLayer, which_children: PaintLayerIteration) -> Self {
        Self {
            root_: root,
            remaining_children_: which_children.0,
            index_: 0,
            current_normal_flow_child_: root.FirstChild(),
            #[cfg(debug_assertions)]
            _mutation_detector_: PaintLayerListMutationDetector::new(root),
        }
    }

    // Source: paint_layer_paint_order_iterator.h:73-79
    pub fn LayersPaintingOverlayOverflowControlsAfter(
        &self,
        layer: &dyn PaintLayer,
    ) -> Option<&'a dyn PaintLayerList> {
        self.root_
            .StackingNode()?
            .LayersPaintingOverlayOverflowControlsAfter(layer)
    }

    // Source: paint_layer_paint_order_iterator.h:71; .cc:39-80
    pub fn Next(&mut self) -> Option<&'a dyn PaintLayer> {
        if self.remaining_children_ & PaintLayerIteration::kNegativeZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.NegZOrderList();
                if self.index_ < list.len() {
                    let layer = list.at(self.index_);
                    self.index_ += 1;
                    return Some(layer);
                }
            }
            self.index_ = 0;
            self.remaining_children_ &= !PaintLayerIteration::kNegativeZOrderChildren.0;
        }
        if self.remaining_children_ & PaintLayerIteration::kNormalFlowChildren.0 != 0 {
            while let Some(child) = self.current_normal_flow_child_ {
                self.current_normal_flow_child_ = child.NextSibling();
                if child.IsStacked() {
                    continue;
                }
                return Some(child);
            }
            self.current_normal_flow_child_ = self.root_.FirstChild();
            self.remaining_children_ &= !PaintLayerIteration::kNormalFlowChildren.0;
        }
        if self.remaining_children_ & PaintLayerIteration::kPositiveZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.PosZOrderList();
                if self.index_ < list.len() {
                    let layer = list.at(self.index_);
                    self.index_ += 1;
                    return Some(layer);
                }
            }
            self.index_ = 0;
            self.remaining_children_ &= !PaintLayerIteration::kPositiveZOrderChildren.0;
        }
        None
    }
}

impl<'a> PaintLayerPaintOrderIteratorBase<'a> for PaintLayerPaintOrderIterator<'a> {
    fn Next(&mut self) -> Option<&'a dyn PaintLayer> {
        Self::Next(self)
    }
}

// Source: paint_layer_paint_order_iterator.h:91-126
pub struct PaintLayerPaintOrderReverseIterator<'a> {
    root_: &'a dyn PaintLayer,
    remaining_children_: u32,
    index_: isize,
    current_normal_flow_child_: Option<&'a dyn PaintLayer>,
    #[cfg(debug_assertions)]
    _mutation_detector_: PaintLayerListMutationDetector<'a>,
}

impl<'a> PaintLayerPaintOrderReverseIterator<'a> {
    // Source: paint_layer_paint_order_iterator.h:98-112
    pub fn new(root: &'a dyn PaintLayer, which_children: PaintLayerIteration) -> Self {
        let mut iterator = Self {
            root_: root,
            remaining_children_: which_children.0,
            index_: -1,
            current_normal_flow_child_: None,
            #[cfg(debug_assertions)]
            _mutation_detector_: PaintLayerListMutationDetector::new(root),
        };
        iterator.SetIndexToLastItem();
        iterator
    }

    // Source: paint_layer_paint_order_iterator.h:114; .cc:82-123
    pub fn Next(&mut self) -> Option<&'a dyn PaintLayer> {
        if self.remaining_children_ & PaintLayerIteration::kNegativeZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                if self.index_ >= 0 {
                    let layer = node.NegZOrderList().at(self.index_ as usize);
                    self.index_ -= 1;
                    return Some(layer);
                }
            }
            self.remaining_children_ &= !PaintLayerIteration::kNegativeZOrderChildren.0;
            self.SetIndexToLastItem();
        }
        if self.remaining_children_ & PaintLayerIteration::kNormalFlowChildren.0 != 0 {
            while let Some(child) = self.current_normal_flow_child_ {
                self.current_normal_flow_child_ = child.PreviousSibling();
                if child.IsStacked() {
                    continue;
                }
                return Some(child);
            }
            self.remaining_children_ &= !PaintLayerIteration::kNormalFlowChildren.0;
            self.SetIndexToLastItem();
        }
        if self.remaining_children_ & PaintLayerIteration::kPositiveZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                if self.index_ >= 0 {
                    let layer = node.PosZOrderList().at(self.index_ as usize);
                    self.index_ -= 1;
                    return Some(layer);
                }
            }
            self.remaining_children_ &= !PaintLayerIteration::kPositiveZOrderChildren.0;
            self.SetIndexToLastItem();
        }
        None
    }

    // Source: paint_layer_paint_order_iterator.h:117; .cc:125-158
    fn SetIndexToLastItem(&mut self) {
        if self.remaining_children_ & PaintLayerIteration::kNegativeZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.NegZOrderList();
                if !list.is_empty() {
                    self.index_ = (list.len() - 1) as isize;
                    return;
                }
            }
            self.remaining_children_ &= !PaintLayerIteration::kNegativeZOrderChildren.0;
        }
        if self.remaining_children_ & PaintLayerIteration::kNormalFlowChildren.0 != 0 {
            self.current_normal_flow_child_ = self.root_.LastChild();
            return;
        }
        if self.remaining_children_ & PaintLayerIteration::kPositiveZOrderChildren.0 != 0 {
            if let Some(node) = self.root_.StackingNode() {
                let list = node.PosZOrderList();
                if !list.is_empty() {
                    self.index_ = (list.len() - 1) as isize;
                    return;
                }
            }
            self.remaining_children_ &= !PaintLayerIteration::kPositiveZOrderChildren.0;
        }
        debug_assert_eq!(self.remaining_children_, 0);
        self.index_ = -1;
    }
}

impl<'a> PaintLayerPaintOrderIteratorBase<'a> for PaintLayerPaintOrderReverseIterator<'a> {
    fn Next(&mut self) -> Option<&'a dyn PaintLayer> {
        Self::Next(self)
    }
}

// Source: paint_layer_paint_order_iterator.cc:160-176
// The iterator supplied here is borrowed from the canvas's existing linked
// hash set, retaining its reverse drawing order and weak element semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanvasIdentity(pub usize);

pub trait CanvasDrawnElement {
    fn CanvasForDrawingIdentity(&self) -> Option<CanvasIdentity>;
    fn BoxModelLayer(&self) -> Option<&dyn PaintLayer>;
}

pub trait HitTestableCanvas {
    type ReverseIterator<'a>: Iterator<Item = &'a dyn CanvasDrawnElement>
    where
        Self: 'a;
    fn Identity(&self) -> CanvasIdentity;
    fn HitTestableDescendantsReverse(&self) -> Self::ReverseIterator<'_>;
}

// Source: paint_layer_paint_order_iterator.h:128-144
pub struct CanvasDrawnElementPaintOrderReverseIterator<'a, C: HitTestableCanvas + 'a> {
    canvas_: &'a C,
    current_: C::ReverseIterator<'a>,
}

impl<'a, C: HitTestableCanvas> CanvasDrawnElementPaintOrderReverseIterator<'a, C> {
    // Source: paint_layer_paint_order_iterator.h:137-138; .cc:160-162
    pub fn new(canvas: &'a C) -> Self {
        Self {
            canvas_: canvas,
            current_: canvas.HitTestableDescendantsReverse(),
        }
    }

    // Source: paint_layer_paint_order_iterator.h:139; .cc:164-177
    pub fn Next(&mut self) -> Option<&'a dyn PaintLayer> {
        for element in self.current_.by_ref() {
            if element.CanvasForDrawingIdentity() == Some(self.canvas_.Identity()) {
                if let Some(layer) = element.BoxModelLayer() {
                    return Some(layer);
                }
            }
        }
        None
    }
}

impl<'a, C: HitTestableCanvas> PaintLayerPaintOrderIteratorBase<'a>
    for CanvasDrawnElementPaintOrderReverseIterator<'a, C>
{
    fn Next(&mut self) -> Option<&'a dyn PaintLayer> {
        Self::Next(self)
    }
}
