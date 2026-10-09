// Copyright 2021 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/post_style_update_scope.h
// cpp: third_party/blink/renderer/core/css/post_style_update_scope.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   post_style_update_scope.h: 139 / 62 / 34 / 28 / 0.
//   post_style_update_scope.cc: 147 / 110 / 95 / 15 / 0.
// Effective excludes comments/blanks; braces retained. Every production
// declaration/body is mapped, including explicit Apply and member disposal.
// h mapped: 26,30-31,33,41,56,61,63,70-72,74,86,91-92,94-95,97-99,101,106,114,117,120-121,125,127-128,130-132,134-135.
// h omitted: 5-6,8-13,15,17-20,27,29,34,36,65-68,75,77,88-89,116,137,139.
// cc mapped: 16,18-21,23-25,27-32,34-37,42,44-45,48-49,51-56,58-59,61-63,65-66,68-70,72-73,75-77,79-80,82-88,95-98,102,104-109,111-117,119-123,125-132,134-137,139-145.
// cc omitted: 5,7-12,14,38-41,100-101,147.
// Omissions: preprocessing/includes/namespace/forward/access/stack-allocation
// scaffolding, friend declarations and DCHECK Missing Apply/reentrancy checks.
// The current_ singleton is one thread-local weak slot, not per-backend state.
// AnimationData preserves absent versus stored-null old styles and uses the
// real ComputedStyle::NullifyEnsured. Animation/style-engine owners are typed.
// Apply is explicit in this source version. Its destructor clears current_,
// with debug-only Missing Apply checks; it does not automatically apply work.
// There is no scroll-snapshot or Dispose operation in this source pair.

use crate::style_engine::{InApplyAnimationUpdateScope, StyleEngine, StyleEngineBackend};
use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// Real Document/Element/animation/frame owners. Update recording, current
/// scope selection, snapshots, callback order and Apply control stay below.
pub trait PostStyleUpdateScopeBackend: 'static {
    type Document: 'static;
    type Element: 'static;
    type CSSAnimationUpdate;
    type ElementAnimations;
    type CSSAnimations;
    type LocalFrameView;
    type DocumentAnimations;
    type EngineBackend: StyleEngineBackend<Document = Self::Document>;

    fn DocumentStyleEngine(document: &Self::Document) -> Rc<StyleEngine<Self::EngineBackend>>;
    fn EnsureElementAnimations(element: &Rc<Self::Element>) -> Rc<Self::ElementAnimations>;
    fn ElementAnimations(element: &Self::Element) -> Option<Rc<Self::ElementAnimations>>;
    fn CssAnimations(animations: &Self::ElementAnimations) -> &Self::CSSAnimations;
    fn CssAnimationsSetPendingUpdate(
        animations: &Self::CSSAnimations,
        update: &Self::CSSAnimationUpdate,
    );
    fn CssAnimationsMaybeApplyPendingUpdate(
        animations: &Self::CSSAnimations,
        element: &Rc<Self::Element>,
    );
    fn ElementComputedStyle(element: &Self::Element) -> Option<Persistent<ComputedStyle>>;
    fn ApplyPendingBackdropPseudoElementUpdate(element: &Rc<Self::Element>);
    fn AnimationTriggerEnabled() -> bool;
    fn DocumentView(document: &Self::Document) -> &Self::LocalFrameView;
    fn ViewNeedsLayout(view: &Self::LocalFrameView) -> bool;
    fn DocumentAnimations(document: &Self::Document) -> &Self::DocumentAnimations;
    fn UpdateAnimationTriggerAttachments(animations: &Self::DocumentAnimations);
    fn RemoveFinishedTopLayerElements(document: &Self::Document) -> bool;
}

type OldStyle<B> = (
    Rc<<B as PostStyleUpdateScopeBackend>::Element>,
    Option<Persistent<ComputedStyle>>,
);

// cpp: post_style_update_scope.h:33-72
pub struct AnimationData<B: PostStyleUpdateScopeBackend> {
    // Map keys are addresses of canonical retained Element objects. Values
    // keep those objects alive; map iteration has the source set's unspecified
    // order, and storing None differs from absence of an old-style entry.
    elements_with_pending_updates_: RefCell<HashMap<usize, Rc<B::Element>>>,
    old_styles_: RefCell<HashMap<usize, OldStyle<B>>>,
}
impl<B: PostStyleUpdateScopeBackend> AnimationData<B> {
    fn new() -> Self {
        Self {
            elements_with_pending_updates_: RefCell::new(HashMap::new()),
            old_styles_: RefCell::new(HashMap::new()),
        }
    }
    fn NullifyEnsured(
        style: Option<Persistent<ComputedStyle>>,
    ) -> Option<Persistent<ComputedStyle>> {
        style.filter(|style| !ComputedStyle::NullifyEnsured(style.Get()).is_null())
    }
    // cpp: post_style_update_scope.cc:104-109
    pub fn SetPendingUpdate(&self, element: Rc<B::Element>, update: &B::CSSAnimationUpdate) {
        let animations = B::EnsureElementAnimations(&element);
        B::CssAnimationsSetPendingUpdate(B::CssAnimations(&animations), update);
        self.elements_with_pending_updates_
            .borrow_mut()
            .insert(Rc::as_ptr(&element) as usize, element);
    }
    // cpp: post_style_update_scope.cc:119-123
    pub fn StoreOldStyleIfNeeded(&self, element: Rc<B::Element>) {
        let style = Self::NullifyEnsured(B::ElementComputedStyle(&element));
        self.old_styles_
            .borrow_mut()
            .entry(Rc::as_ptr(&element) as usize)
            .or_insert((element, style));
    }
    // cpp: post_style_update_scope.cc:125-132
    pub fn GetOldStyle(&self, element: &B::Element) -> Option<Persistent<ComputedStyle>> {
        let stored = self
            .old_styles_
            .borrow()
            .get(&(element as *const B::Element as usize))
            .map(|(_, style)| style.clone());
        match stored {
            Some(style) => style,
            None => Self::NullifyEnsured(B::ElementComputedStyle(element)),
        }
    }
    // cpp: post_style_update_scope.h:63
    pub fn HasOldStyles(&self) -> bool {
        !self.old_styles_.borrow().is_empty()
    }
}

// cpp: post_style_update_scope.h:75-95
pub struct PseudoData<B: PostStyleUpdateScopeBackend> {
    pending_backdrops_: RefCell<Vec<Rc<B::Element>>>,
}
impl<B: PostStyleUpdateScopeBackend> PseudoData<B> {
    fn new() -> Self {
        Self {
            pending_backdrops_: RefCell::new(Vec::new()),
        }
    }
    // cpp: post_style_update_scope.cc:134-137
    pub fn AddPendingBackdrop(&self, originating_element: Rc<B::Element>) {
        self.pending_backdrops_
            .borrow_mut()
            .push(originating_element);
    }
}

struct ScopeState<B: PostStyleUpdateScopeBackend> {
    animation_data_: Rc<AnimationData<B>>,
    pseudo_data_: Rc<PseudoData<B>>,
    nullify_pseudo_data_: Cell<bool>,
}

// cpp: post_style_update_scope.h:134; post_style_update_scope.cc:16
// A single current_ slot per rendering thread, across backend instantiations.
// The weak reference does not extend the RAII scope's lifetime. Type erasure
// only stores this explicitly source-owned singleton, not an owner registry.
thread_local! {
    static CURRENT: RefCell<Option<Weak<dyn Any>>> = RefCell::new(None);
}

// cpp: post_style_update_scope.h:116-128
pub struct PostStyleUpdateScope<B: PostStyleUpdateScopeBackend> {
    document_: Rc<B::Document>,
    state_: Rc<ScopeState<B>>,
}
impl<B: PostStyleUpdateScopeBackend> PostStyleUpdateScope<B> {
    // cpp: post_style_update_scope.cc:27-32
    pub fn new(document: Rc<B::Document>) -> Self {
        let state = Rc::new(ScopeState {
            animation_data_: Rc::new(AnimationData::new()),
            pseudo_data_: Rc::new(PseudoData::new()),
            nullify_pseudo_data_: Cell::new(false),
        });
        CURRENT.with(|current| {
            let mut current = current.borrow_mut();
            if current.as_ref().and_then(Weak::upgrade).is_none() {
                let erased: Rc<dyn Any> = state.clone();
                *current = Some(Rc::downgrade(&erased));
            }
        });
        Self {
            document_: document,
            state_: state,
        }
    }
    fn CurrentState() -> Option<Rc<ScopeState<B>>> {
        CURRENT.with(|current| {
            current
                .borrow()
                .as_ref()
                .and_then(Weak::upgrade)
                .map(|state| {
                    state.downcast::<ScopeState<B>>().unwrap_or_else(|_| {
                        panic!("current style scope belongs to a different DOM backend")
                    })
                })
        })
    }
    fn IsCurrent(&self) -> bool {
        let state: Rc<dyn Any> = self.state_.clone();
        CURRENT.with(|current| {
            current
                .borrow()
                .as_ref()
                .is_some_and(|current| Weak::ptr_eq(current, &Rc::downgrade(&state)))
        })
    }
    // cpp: post_style_update_scope.cc:18-21
    pub fn CurrentAnimationData() -> Option<Rc<AnimationData<B>>> {
        Self::CurrentState().map(|state| state.animation_data_.clone())
    }
    // cpp: post_style_update_scope.cc:23-25
    pub fn CurrentPseudoData() -> Option<Rc<PseudoData<B>>> {
        Self::CurrentState().and_then(|state| {
            if state.nullify_pseudo_data_.get() {
                None
            } else {
                Some(state.pseudo_data_.clone())
            }
        })
    }
    // cpp: post_style_update_scope.h:100-102
    pub fn InPendingPseudoUpdate() -> bool {
        Self::CurrentState().is_some_and(|state| state.nullify_pseudo_data_.get())
    }
    // cpp: post_style_update_scope.cc:111-117
    pub fn SetPendingUpdateForTesting(element: Rc<B::Element>, update: &B::CSSAnimationUpdate) {
        if let Some(data) = Self::CurrentAnimationData() {
            data.SetPendingUpdate(element, update);
        }
    }
    // cpp: post_style_update_scope.cc:139-145
    pub fn GetOldStyle(element: &B::Element) -> Option<Persistent<ComputedStyle>> {
        if let Some(data) = Self::CurrentAnimationData() {
            return data.GetOldStyle(element);
        }
        AnimationData::<B>::NullifyEnsured(B::ElementComputedStyle(element))
    }
    // cpp: post_style_update_scope.cc:44-56
    pub fn Apply(&self) -> bool {
        if !self.IsCurrent() {
            return false;
        }
        if self.ApplyPseudo() {
            return true;
        }
        self.ApplyAnimations();
        B::RemoveFinishedTopLayerElements(&self.document_)
    }
    // cpp: post_style_update_scope.cc:58-73
    fn ApplyPseudo(&self) -> bool {
        self.state_.nullify_pseudo_data_.set(true);
        if self
            .state_
            .pseudo_data_
            .pending_backdrops_
            .borrow()
            .is_empty()
        {
            return false;
        }
        let pending =
            std::mem::take(&mut *self.state_.pseudo_data_.pending_backdrops_.borrow_mut());
        for element in pending {
            B::ApplyPendingBackdropPseudoElementUpdate(&element);
        }
        true
    }
    // cpp: post_style_update_scope.cc:75-102
    fn ApplyAnimations(&self) {
        let engine = B::DocumentStyleEngine(&self.document_);
        let _in_apply = InApplyAnimationUpdateScope::new(&engine);
        let pending = std::mem::take(
            &mut *self
                .state_
                .animation_data_
                .elements_with_pending_updates_
                .borrow_mut(),
        );
        for element in pending.into_values() {
            let Some(animations) = B::ElementAnimations(&element) else {
                continue;
            };
            B::CssAnimationsMaybeApplyPendingUpdate(B::CssAnimations(&animations), &element);
        }
        if B::AnimationTriggerEnabled() && !B::ViewNeedsLayout(B::DocumentView(&self.document_)) {
            B::UpdateAnimationTriggerAttachments(B::DocumentAnimations(&self.document_));
        }
    }
}
impl<B: PostStyleUpdateScopeBackend> Drop for PostStyleUpdateScope<B> {
    // cpp: post_style_update_scope.cc:34-42
    fn drop(&mut self) {
        let state: Rc<dyn Any> = self.state_.clone();
        CURRENT.with(|current| {
            let mut current = current.borrow_mut();
            if current
                .as_ref()
                .is_some_and(|current| Weak::ptr_eq(current, &Rc::downgrade(&state)))
            {
                *current = None;
            }
        });
        // C++ destroys members after the destructor body: PseudoData first,
        // then AnimationData's old-style map and pending-element set. Clear
        // them in that order even if a safe accessor handle remains retained.
        self.state_
            .pseudo_data_
            .pending_backdrops_
            .borrow_mut()
            .clear();
        self.state_.animation_data_.old_styles_.borrow_mut().clear();
        self.state_
            .animation_data_
            .elements_with_pending_updates_
            .borrow_mut()
            .clear();
    }
}
