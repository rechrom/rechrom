// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/trees/layer_tree_mutator.h (class LayerTreeMutator). Blink
// creates the animation-worklet mutator on the main thread;
// ProxyMain::SetMutator (cc/trees/proxy_main.cc) posts it to
// ProxyImpl::InitializeMutatorOnImpl, which hands it to the impl
// AnimationHost (AnimationHost::SetLayerTreeMutator).

/// The native animation-worklet mutator. The AnimationHost owns and
/// transfers it; no animation state or mutation algorithm is copied.
/// `Send`: the owner moves from the main thread to the impl thread.
pub trait LayerTreeMutatorBackend: Send {
    // Source: cc/trees/layer_tree_mutator.h HasMutators.
    fn HasMutators(&mut self) -> bool;
}
pub struct LayerTreeMutator(pub Box<dyn LayerTreeMutatorBackend>);
