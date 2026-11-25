// Translation status: complete
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/effect_tree_layer_list_iterator.h:1-131.
// Source: cc/layers/effect_tree_layer_list_iterator.cc:1-125.
use crate::{
    adapter::layer_tree_impl::{LayerTreeId, LayerTreeLayer, LayerTreeTypes},
    cc::trees::{
        layer_tree_impl::LayerTreeImpl,
        property_ids::{kContentsRootPropertyNodeId, kInvalidPropertyNodeId},
    },
};
// Live borrow interface for the original raw LayerTreeImpl/EffectTree pointers.
// The drawing split supplies the same resident layer list/property trees.
pub trait IteratorContext {
    type Types: LayerTreeTypes;
    fn id(&self) -> LayerTreeId;
    fn num_layers(&self) -> usize;
    fn HasRenderSurface(&self, effect_id: i32) -> bool {
        self.property_trees()
            .effect_tree()
            .GetRenderSurface(effect_id)
            .is_some()
    }
    fn LayerAtReverseIndex(&self, index: usize) -> Option<&<Self::Types as LayerTreeTypes>::Layer>;
    fn property_trees(&self) -> &crate::adapter::layer_tree_impl::PropertyTreesFor<Self::Types>;
}
impl<B: LayerTreeTypes> IteratorContext for LayerTreeImpl<B> {
    type Types = B;
    fn id(&self) -> LayerTreeId {
        self.id
    }
    fn num_layers(&self) -> usize {
        LayerTreeImpl::num_layers(self)
    }
    fn LayerAtReverseIndex(&self, index: usize) -> Option<&B::Layer> {
        LayerTreeImpl::LayerAtReverseIndex(self, index)
    }
    fn property_trees(&self) -> &crate::adapter::layer_tree_impl::PropertyTreesFor<B> {
        LayerTreeImpl::property_trees(self)
    }
}
// Source: cc/layers/effect_tree_layer_list_iterator.h:39.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    kLayer,
    kTargetSurface,
    kContributingSurface,
    kEnd,
}
// Source: cc/layers/effect_tree_layer_list_iterator.h:74-84.
// Native non-owning pointers become resident identities. Layer IDs identify
// layers; effect IDs identify render surfaces in the caller-supplied live tree.
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub state: State,
    pub current_layer: Option<i32>,
    pub current_render_surface: Option<i32>,
    pub target_render_surface: Option<i32>,
}
impl Default for Position {
    fn default() -> Self {
        Self {
            state: State::kEnd,
            current_layer: None,
            current_render_surface: None,
            target_render_surface: None,
        }
    }
}
// Source: cc/layers/effect_tree_layer_list_iterator.h:102-127.
// The source reverse iterator is a stable offset into the resident layer list.
// Every operation reborrows that list through its O(1) LayerAt accessor. Neither
// the layer list nor the effect tree is copied or retained by this cursor.
// Source: cc/layers/effect_tree_layer_list_iterator.cc:42-45.
#[derive(Clone, Copy, Debug)]
pub struct EffectTreeLayerListIterator {
    state_: State,
    layer_list_iterator_: usize,
    current_effect_tree_index_: i32,
    next_effect_tree_index_: i32,
    lowest_common_effect_tree_ancestor_index_: i32,
    layer_tree_impl_: LayerTreeId,
    effect_tree_: LayerTreeId,
}
impl EffectTreeLayerListIterator {
    // Source: cc/layers/effect_tree_layer_list_iterator.cc:11-40.
    pub fn new<C: IteratorContext>(tree: &C) -> Self {
        let mut iterator = Self {
            state_: State::kEnd,
            layer_list_iterator_: 0,
            current_effect_tree_index_: kInvalidPropertyNodeId,
            next_effect_tree_index_: kInvalidPropertyNodeId,
            lowest_common_effect_tree_ancestor_index_: kInvalidPropertyNodeId,
            layer_tree_impl_: tree.id(),
            effect_tree_: tree.id(),
        };
        while iterator.layer_list_iterator_ != tree.num_layers()
            && !tree
                .LayerAtReverseIndex(iterator.layer_list_iterator_)
                .unwrap()
                .contributes_to_drawn_render_surface()
        {
            iterator.layer_list_iterator_ += 1;
        }
        if iterator.layer_list_iterator_ == tree.num_layers() {
            debug_assert!(
                tree.property_trees().effect_tree().size() > kContentsRootPropertyNodeId as usize
            );
            iterator.state_ = State::kTargetSurface;
            iterator.current_effect_tree_index_ = kContentsRootPropertyNodeId;
        } else {
            iterator.state_ = State::kLayer;
            iterator.current_effect_tree_index_ = tree
                .LayerAtReverseIndex(iterator.layer_list_iterator_)
                .unwrap()
                .render_target_effect_tree_index(tree.property_trees());
            iterator.next_effect_tree_index_ = iterator.current_effect_tree_index_;
            iterator.lowest_common_effect_tree_ancestor_index_ =
                iterator.current_effect_tree_index_;
        }
        iterator
    }
    // Rust borrow guard for both source non-owning pointers. No lookup/cache or
    // ownership is introduced; caller must supply the same resident allocation.
    fn CheckOwner<C: IteratorContext>(&self, tree: &C) {
        assert_eq!(self.layer_tree_impl_, tree.id());
        assert_eq!(self.effect_tree_, tree.id());
    }
    // Source: cc/layers/effect_tree_layer_list_iterator.h:47.
    pub fn state(&self) -> State {
        self.state_
    }
    // Source: cc/layers/effect_tree_layer_list_iterator.h:49-52.
    pub fn current_layer<C: IteratorContext>(&self, tree: &C) -> i32 {
        self.CheckOwner(tree);
        debug_assert_eq!(self.state_, State::kLayer);
        tree.LayerAtReverseIndex(self.layer_list_iterator_)
            .unwrap()
            .id()
    }
    // Source: cc/layers/effect_tree_layer_list_iterator.h:54-57.
    pub fn current_render_surface<C: IteratorContext>(&self, tree: &C) -> i32 {
        self.CheckOwner(tree);
        debug_assert_eq!(self.state_, State::kContributingSurface);
        debug_assert!(tree.HasRenderSurface(self.current_effect_tree_index_));
        self.current_effect_tree_index_
    }
    // Source: cc/layers/effect_tree_layer_list_iterator.h:59-72.
    pub fn target_render_surface<C: IteratorContext>(&self, tree: &C) -> i32 {
        self.CheckOwner(tree);
        match self.state_ {
            State::kLayer | State::kTargetSurface => self.current_effect_tree_index_,
            State::kContributingSurface => {
                tree.property_trees()
                    .effect_tree()
                    .Node(self.current_effect_tree_index_)
                    .target_id
            }
            State::kEnd => panic!("NOTREACHED"),
        }
    }
    // Source: cc/layers/effect_tree_layer_list_iterator.h:86-100.
    pub fn position<C: IteratorContext>(&self, tree: &C) -> Position {
        let mut position = Position::default();
        if self.state_ == State::kEnd {
            return position;
        }
        position.state = self.state_;
        position.target_render_surface = Some(self.target_render_surface(tree));
        if self.state_ == State::kLayer {
            position.current_layer = Some(self.current_layer(tree));
        } else if self.state_ == State::kContributingSurface {
            position.current_render_surface = Some(self.current_render_surface(tree));
        }
        position
    }
    // Source: cc/layers/effect_tree_layer_list_iterator.cc:47-123.
    pub fn Increment<C: IteratorContext>(&mut self, tree: &C) {
        self.CheckOwner(tree);
        match self.state_ {
            State::kLayer => {
                self.layer_list_iterator_ += 1;
                while self.layer_list_iterator_ != tree.num_layers()
                    && !tree
                        .LayerAtReverseIndex(self.layer_list_iterator_)
                        .unwrap()
                        .contributes_to_drawn_render_surface()
                {
                    self.layer_list_iterator_ += 1;
                }
                if self.layer_list_iterator_ == tree.num_layers() {
                    self.next_effect_tree_index_ = kInvalidPropertyNodeId;
                    self.lowest_common_effect_tree_ancestor_index_ = kInvalidPropertyNodeId;
                    self.state_ = State::kTargetSurface;
                    return;
                }
                self.next_effect_tree_index_ = tree
                    .LayerAtReverseIndex(self.layer_list_iterator_)
                    .unwrap()
                    .render_target_effect_tree_index(tree.property_trees());
                if self.next_effect_tree_index_ != self.current_effect_tree_index_ {
                    self.lowest_common_effect_tree_ancestor_index_ = tree
                        .property_trees()
                        .effect_tree()
                        .LowestCommonAncestorWithRenderSurface(
                            self.current_effect_tree_index_,
                            self.next_effect_tree_index_,
                        );
                    if self.current_effect_tree_index_
                        == self.lowest_common_effect_tree_ancestor_index_
                    {
                        self.current_effect_tree_index_ = self.next_effect_tree_index_;
                        self.lowest_common_effect_tree_ancestor_index_ =
                            self.next_effect_tree_index_;
                    } else {
                        self.state_ = State::kTargetSurface;
                    }
                }
            }
            State::kTargetSurface => {
                if self.current_effect_tree_index_ == kContentsRootPropertyNodeId {
                    self.current_effect_tree_index_ = kInvalidPropertyNodeId;
                    self.state_ = State::kEnd;
                    debug_assert_eq!(self.next_effect_tree_index_, kInvalidPropertyNodeId);
                    debug_assert_eq!(self.layer_list_iterator_, tree.num_layers());
                } else {
                    self.state_ = State::kContributingSurface;
                }
            }
            State::kContributingSurface => {
                debug_assert_ne!(
                    self.current_effect_tree_index_,
                    self.lowest_common_effect_tree_ancestor_index_
                );
                self.current_effect_tree_index_ = tree
                    .property_trees()
                    .effect_tree()
                    .Node(self.current_effect_tree_index_)
                    .target_id;
                if self.current_effect_tree_index_ == self.next_effect_tree_index_ {
                    self.state_ = State::kLayer;
                } else if self.current_effect_tree_index_
                    == self.lowest_common_effect_tree_ancestor_index_
                {
                    self.state_ = State::kLayer;
                    self.current_effect_tree_index_ = self.next_effect_tree_index_;
                    self.lowest_common_effect_tree_ancestor_index_ = self.next_effect_tree_index_;
                } else {
                    self.state_ = State::kTargetSurface;
                }
            }
            State::kEnd => panic!("NOTREACHED"),
        }
    }
}
