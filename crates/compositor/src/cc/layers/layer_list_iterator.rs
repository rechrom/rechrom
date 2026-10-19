// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/layer_list_iterator.h:1-18; cc/layers/layer_list_iterator.cc:1-9.
use super::layer::LayerBorrow;
use crate::adapter::main_layer::{LayerContext, MainLayerTypes};
// Native STACK_ALLOCATED and difference_type=ptrdiff_t correspond to by-value
// iterators and isize. The source pointers retain allocation liveness only;
// they never retain a native LayerRef owner or a materialized layer list.
pub type DifferenceType = isize;

// Source: cc/layers/layer_list_iterator.h:21-56.
pub struct LayerListIterator<B: MainLayerTypes> {
    current_layer_: Option<LayerBorrow<B>>,
    list_indices_: Vec<usize>,
}
// Source: cc/layers/layer_list_iterator.cc:11-11. Defaulted source constructor.
impl<B: MainLayerTypes> Default for LayerListIterator<B> {
    fn default() -> Self {
        Self {
            current_layer_: None,
            list_indices_: Vec::new(),
        }
    }
}
// Source: cc/layers/layer_list_iterator.cc:19-19. Defaulted source copy constructor.
impl<B: MainLayerTypes> Clone for LayerListIterator<B> {
    fn clone(&self) -> Self {
        Self {
            current_layer_: self.current_layer_.clone(),
            list_indices_: self.list_indices_.clone(),
        }
    }
}
impl<B: MainLayerTypes> PartialEq for LayerListIterator<B> {
    fn eq(&self, other: &Self) -> bool {
        self.current_layer_ == other.current_layer_
    }
}
impl<B: MainLayerTypes> Eq for LayerListIterator<B> {}
impl<B: MainLayerTypes> LayerListIterator<B> {
    // Source: cc/layers/layer_list_iterator.cc:13-17.
    pub fn new(root_layer: Option<LayerBorrow<B>>, c: &LayerContext<'_, B>) -> Self {
        let mut result = Self {
            current_layer_: root_layer,
            list_indices_: Vec::new(),
        };
        debug_assert!(result
            .current_layer_
            .as_ref()
            .is_none_or(|root| root.WithSourceRef(c, |layer, c| layer.base().parent(c).is_none())));
        result.list_indices_.push(0);
        result
    }
    // Source: cc/layers/layer_list_iterator.cc:21-49.
    pub fn Increment(&mut self, c: &LayerContext<'_, B>) -> &mut Self {
        if self.current_layer_.is_none() {
            return self;
        }
        let current = self.current_layer_.as_ref().unwrap();
        if !current.WithSourceRef(c, |layer, c| layer.base().children(c).is_empty()) {
            self.current_layer_ =
                Some(current.WithSourceRef(c, |layer, c| layer.base().children(c)[0].borrow()));
            self.list_indices_.push(0);
            return self;
        }
        let mut parent = current.WithSourceRef(c, |layer, c| layer.base().mutable_parent(c));
        while let Some(layer) = parent {
            if *self.list_indices_.last().unwrap() + 1
                < layer.WithSourceRef(c, |layer, c| layer.base().children(c).len())
            {
                *self.list_indices_.last_mut().unwrap() += 1;
                self.current_layer_ = Some(layer.WithSourceRef(c, |layer, c| {
                    layer.base().children(c)[*self.list_indices_.last().unwrap()].borrow()
                }));
                return self;
            }
            self.list_indices_.pop();
            parent = layer.WithSourceRef(c, |layer, c| layer.base().mutable_parent(c));
        }
        self.current_layer_ = None;
        self
    }
    // Source: cc/layers/layer_list_iterator.h:21-56. Native postfix++ and raw pointer accessors.
    pub fn PostIncrement(&mut self, c: &LayerContext<'_, B>) -> Self {
        let that = self.clone();
        self.Increment(c);
        that
    }
    pub fn Arrow(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
    pub fn Dereference(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
}
// Source: cc/layers/layer_list_iterator.cc:51-51. Default member destruction
// releases only raw-pointer allocation borrows and the native index vector.

// Source: cc/layers/layer_list_iterator.h:58-89.
pub struct LayerListConstIterator<B: MainLayerTypes> {
    current_layer_: Option<LayerBorrow<B>>,
    list_indices_: Vec<usize>,
}
// Source: cc/layers/layer_list_iterator.cc:53-53. Defaulted source constructor.
impl<B: MainLayerTypes> Default for LayerListConstIterator<B> {
    fn default() -> Self {
        Self {
            current_layer_: None,
            list_indices_: Vec::new(),
        }
    }
}
// Source: cc/layers/layer_list_iterator.cc:61-62. Defaulted source copy constructor.
impl<B: MainLayerTypes> Clone for LayerListConstIterator<B> {
    fn clone(&self) -> Self {
        Self {
            current_layer_: self.current_layer_.clone(),
            list_indices_: self.list_indices_.clone(),
        }
    }
}
impl<B: MainLayerTypes> PartialEq for LayerListConstIterator<B> {
    fn eq(&self, other: &Self) -> bool {
        self.current_layer_ == other.current_layer_
    }
}
impl<B: MainLayerTypes> Eq for LayerListConstIterator<B> {}
impl<B: MainLayerTypes> LayerListConstIterator<B> {
    // Source: cc/layers/layer_list_iterator.cc:55-59.
    pub fn new(root_layer: Option<LayerBorrow<B>>, c: &LayerContext<'_, B>) -> Self {
        let mut result = Self {
            current_layer_: root_layer,
            list_indices_: Vec::new(),
        };
        debug_assert!(result
            .current_layer_
            .as_ref()
            .is_none_or(|root| root.WithSourceRef(c, |layer, c| layer.base().parent(c).is_none())));
        result.list_indices_.push(0);
        result
    }
    // Source: cc/layers/layer_list_iterator.cc:64-92.
    pub fn Increment(&mut self, c: &LayerContext<'_, B>) -> &mut Self {
        if self.current_layer_.is_none() {
            return self;
        }
        let current = self.current_layer_.as_ref().unwrap();
        if !current.WithSourceRef(c, |layer, c| layer.base().children(c).is_empty()) {
            self.current_layer_ =
                Some(current.WithSourceRef(c, |layer, c| layer.base().children(c)[0].borrow()));
            self.list_indices_.push(0);
            return self;
        }
        let mut parent = current.WithSourceRef(c, |layer, c| layer.base().parent(c));
        while let Some(layer) = parent {
            if *self.list_indices_.last().unwrap() + 1
                < layer.WithSourceRef(c, |layer, c| layer.base().children(c).len())
            {
                *self.list_indices_.last_mut().unwrap() += 1;
                self.current_layer_ = Some(layer.WithSourceRef(c, |layer, c| {
                    layer.base().children(c)[*self.list_indices_.last().unwrap()].borrow()
                }));
                return self;
            }
            self.list_indices_.pop();
            parent = layer.WithSourceRef(c, |layer, c| layer.base().parent(c));
        }
        self.current_layer_ = None;
        self
    }
    // Source: cc/layers/layer_list_iterator.h:58-89. Native postfix++ and raw pointer accessors.
    pub fn PostIncrement(&mut self, c: &LayerContext<'_, B>) -> Self {
        let that = self.clone();
        self.Increment(c);
        that
    }
    pub fn Arrow(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
    pub fn Dereference(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
}
// Source: cc/layers/layer_list_iterator.cc:94-94. Default member destruction
// releases only raw-pointer allocation borrows and the native index vector.

// Source: cc/layers/layer_list_iterator.h:91-124.
pub struct LayerListReverseIterator<B: MainLayerTypes> {
    current_layer_: Option<LayerBorrow<B>>,
    list_indices_: Vec<usize>,
}
// Source: cc/layers/layer_list_iterator.cc:96-96. Defaulted source constructor.
impl<B: MainLayerTypes> Default for LayerListReverseIterator<B> {
    fn default() -> Self {
        Self {
            current_layer_: None,
            list_indices_: Vec::new(),
        }
    }
}
// Source: cc/layers/layer_list_iterator.cc:105-106. Defaulted source copy constructor.
impl<B: MainLayerTypes> Clone for LayerListReverseIterator<B> {
    fn clone(&self) -> Self {
        Self {
            current_layer_: self.current_layer_.clone(),
            list_indices_: self.list_indices_.clone(),
        }
    }
}
impl<B: MainLayerTypes> PartialEq for LayerListReverseIterator<B> {
    fn eq(&self, other: &Self) -> bool {
        self.current_layer_ == other.current_layer_
    }
}
impl<B: MainLayerTypes> Eq for LayerListReverseIterator<B> {}
impl<B: MainLayerTypes> LayerListReverseIterator<B> {
    // Source: cc/layers/layer_list_iterator.cc:98-103.
    pub fn new(root_layer: Option<LayerBorrow<B>>, c: &LayerContext<'_, B>) -> Self {
        let mut result = Self {
            current_layer_: root_layer,
            list_indices_: Vec::new(),
        };
        debug_assert!(result
            .current_layer_
            .as_ref()
            .is_none_or(|root| root.WithSourceRef(c, |layer, c| layer.base().parent(c).is_none())));
        result.list_indices_.push(0);
        result.DescendToRightmostInSubtree(c);
        result
    }
    // Source: cc/layers/layer_list_iterator.cc:109-129.
    pub fn Increment(&mut self, c: &LayerContext<'_, B>) -> &mut Self {
        if self.current_layer_.is_none() {
            return self;
        }
        if *self.list_indices_.last().unwrap() == 0 {
            self.list_indices_.pop();
            self.current_layer_ = self
                .current_layer_
                .as_ref()
                .unwrap()
                .WithSourceRef(c, |layer, c| layer.base().mutable_parent(c));
            return self;
        }
        assert!(self
            .current_layer_
            .as_ref()
            .unwrap()
            .WithSourceRef(c, |layer, c| layer.base().parent(c).is_some()));
        *self.list_indices_.last_mut().unwrap() -= 1;
        self.current_layer_ = Some(
            self.current_layer_
                .as_ref()
                .unwrap()
                .WithSourceRef(c, |layer, c| layer.base().mutable_parent(c))
                .unwrap()
                .WithSourceRef(c, |layer, c| {
                    layer.base().children(c)[*self.list_indices_.last().unwrap()].borrow()
                }),
        );
        self.DescendToRightmostInSubtree(c);
        self
    }
    // Source: cc/layers/layer_list_iterator.cc:131-142.
    fn DescendToRightmostInSubtree(&mut self, c: &LayerContext<'_, B>) {
        let Some(current) = self.current_layer_.as_ref() else {
            return;
        };
        if current.WithSourceRef(c, |layer, c| layer.base().children(c).is_empty()) {
            return;
        }
        let last_index = current.WithSourceRef(c, |layer, c| layer.base().children(c).len()) - 1;
        self.current_layer_ = Some(
            current.WithSourceRef(c, |layer, c| layer.base().children(c)[last_index].borrow()),
        );
        self.list_indices_.push(last_index);
        self.DescendToRightmostInSubtree(c);
    }
    // Source: cc/layers/layer_list_iterator.h:91-124. Native postfix++ and raw pointer accessors.
    pub fn PostIncrement(&mut self, c: &LayerContext<'_, B>) -> Self {
        let that = self.clone();
        self.Increment(c);
        that
    }
    pub fn Arrow(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
    pub fn Dereference(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
}
// Source: cc/layers/layer_list_iterator.cc:144-144. Default member destruction
// releases only raw-pointer allocation borrows and the native index vector.

// Source: cc/layers/layer_list_iterator.h:126-159.
pub struct LayerListReverseConstIterator<B: MainLayerTypes> {
    current_layer_: Option<LayerBorrow<B>>,
    list_indices_: Vec<usize>,
}
// Source: cc/layers/layer_list_iterator.cc:146-146. Defaulted source constructor.
impl<B: MainLayerTypes> Default for LayerListReverseConstIterator<B> {
    fn default() -> Self {
        Self {
            current_layer_: None,
            list_indices_: Vec::new(),
        }
    }
}
// Source: cc/layers/layer_list_iterator.cc:148-149. Defaulted source copy constructor.
impl<B: MainLayerTypes> Clone for LayerListReverseConstIterator<B> {
    fn clone(&self) -> Self {
        Self {
            current_layer_: self.current_layer_.clone(),
            list_indices_: self.list_indices_.clone(),
        }
    }
}
impl<B: MainLayerTypes> PartialEq for LayerListReverseConstIterator<B> {
    fn eq(&self, other: &Self) -> bool {
        self.current_layer_ == other.current_layer_
    }
}
impl<B: MainLayerTypes> Eq for LayerListReverseConstIterator<B> {}
impl<B: MainLayerTypes> LayerListReverseConstIterator<B> {
    // Source: cc/layers/layer_list_iterator.cc:151-157.
    pub fn new(root_layer: Option<LayerBorrow<B>>, c: &LayerContext<'_, B>) -> Self {
        let mut result = Self {
            current_layer_: root_layer,
            list_indices_: Vec::new(),
        };
        debug_assert!(result
            .current_layer_
            .as_ref()
            .is_none_or(|root| root.WithSourceRef(c, |layer, c| layer.base().parent(c).is_none())));
        result.list_indices_.push(0);
        result.DescendToRightmostInSubtree(c);
        result
    }
    // Source: cc/layers/layer_list_iterator.cc:159-179.
    pub fn Increment(&mut self, c: &LayerContext<'_, B>) -> &mut Self {
        if self.current_layer_.is_none() {
            return self;
        }
        if *self.list_indices_.last().unwrap() == 0 {
            self.list_indices_.pop();
            self.current_layer_ = self
                .current_layer_
                .as_ref()
                .unwrap()
                .WithSourceRef(c, |layer, c| layer.base().parent(c));
            return self;
        }
        assert!(self
            .current_layer_
            .as_ref()
            .unwrap()
            .WithSourceRef(c, |layer, c| layer.base().parent(c).is_some()));
        *self.list_indices_.last_mut().unwrap() -= 1;
        self.current_layer_ = Some(
            self.current_layer_
                .as_ref()
                .unwrap()
                .WithSourceRef(c, |layer, c| layer.base().parent(c))
                .unwrap()
                .WithSourceRef(c, |layer, c| {
                    layer.base().children(c)[*self.list_indices_.last().unwrap()].borrow()
                }),
        );
        self.DescendToRightmostInSubtree(c);
        self
    }
    // Source: cc/layers/layer_list_iterator.cc:181-192.
    fn DescendToRightmostInSubtree(&mut self, c: &LayerContext<'_, B>) {
        let Some(current) = self.current_layer_.as_ref() else {
            return;
        };
        if current.WithSourceRef(c, |layer, c| layer.base().children(c).is_empty()) {
            return;
        }
        let last_index = current.WithSourceRef(c, |layer, c| layer.base().children(c).len()) - 1;
        self.current_layer_ = Some(
            current.WithSourceRef(c, |layer, c| layer.base().children(c)[last_index].borrow()),
        );
        self.list_indices_.push(last_index);
        self.DescendToRightmostInSubtree(c);
    }
    // Source: cc/layers/layer_list_iterator.h:126-159. Native postfix++ and raw pointer accessors.
    pub fn PostIncrement(&mut self, c: &LayerContext<'_, B>) -> Self {
        let that = self.clone();
        self.Increment(c);
        that
    }
    pub fn Arrow(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
    pub fn Dereference(&self) -> Option<&LayerBorrow<B>> {
        self.current_layer_.as_ref()
    }
}
// Source: cc/layers/layer_list_iterator.cc:194-194. Default member destruction
// releases only raw-pointer allocation borrows and the native index vector.
// Source: cc/layers/layer_list_iterator.h:160-162; cc/layers/layer_list_iterator.cc:196. Namespace/header closure.
