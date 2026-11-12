//! Frame-owned ScrollManager assembly. Layout queries borrow the original
//! LocalFrameView/LayoutObject owners; input and animator state stays attached
//! to the corresponding document runtime (scroll_manager.cc:201-342).
use super::local_frame::{LocalFrame, LocalFrameDomain, LocalFrameRuntime};
use super::*;
use interaction::adapter::page_scroll_manager::PageScrollManagerOwners;
use interaction::scroll::core::input::scroll_manager::{
    ScrollAutoscrollController, ScrollDirection, ScrollEffect, ScrollEffectEmitter,
    ScrollGranularity,
};
use interaction::scroll::core::scroll::programmatic_scroll_animator::ProgrammaticScrollAnimatorRegistry;

#[derive(Clone)]
pub(super) enum InteractionScrollOwner {
    Root(Weak<PageState>),
    Child {
        frame: Weak<LocalFrame>,
        runtime: Weak<LocalFrameRuntime>,
    },
}
impl InteractionScrollOwner {
    pub(super) fn BlockingWheelTargets(&self) -> Arc<[u64]> {
        let (mut targets, document) = match self {
            Self::Root(page) => {
                let Some(page) = page.upgrade() else {
                    return Arc::from([]);
                };
                let bindings = page.bindings.borrow().upgrade();
                let targets = bindings.map_or_else(Vec::new, |bindings| {
                    bindings.borrow().BlockingListenerTargets("wheel")
                });
                (targets, page.document.clone())
            }
            Self::Child { .. } => {
                let Some((frame, runtime)) = self.ChildOwners() else {
                    return Arc::from([]);
                };
                let window = frame
                    .navigation
                    .borrow()
                    .window
                    .clone()
                    .filter(|window| Rc::ptr_eq(&window.document, &runtime.document));
                let targets = window.map_or_else(Vec::new, |window| {
                    window
                        .web_api
                        .DOMBindings()
                        .borrow()
                        .BlockingListenerTargets("wheel")
                });
                (targets, runtime.document.clone())
            }
        };
        let document_id = document.borrow().GetDocument().RootHandle().Id();
        for target in &mut targets {
            if *target == document_id {
                *target = 0;
            }
        }
        targets.sort_unstable();
        targets.dedup();
        targets.into()
    }
    pub(super) fn BindBlockingWheelInvalidation(&self, bindings: &DOMJavaScriptBindings) {
        let document_root = match self {
            Self::Root(page) => page
                .upgrade()
                .map(|page| page.document.borrow().GetDocument().RootHandle()),
            Self::Child { runtime, .. } => runtime
                .upgrade()
                .map(|runtime| runtime.document.borrow().GetDocument().RootHandle()),
        };
        let owner = self.clone();
        bindings.SetBlockingWheelInvalidation(Rc::new(move |node| {
            if let Self::Root(page) = &owner {
                if page.upgrade().is_none_or(|page| {
                    Some(page.document.borrow().GetDocument().RootHandle()) != document_root
                }) {
                    return;
                }
            }
            let node = node.filter(|node| Some(*node) != document_root.map(|root| root.Id()));
            owner.WithLayout(|layout| {
                if let Some(target) = layout.EventRegistryLayout(node) {
                    layout.InvalidateEventRegistryLayout(&target, true);
                }
            });
            match &owner {
                Self::Root(page) => {
                    if let Some(page) = page.upgrade() {
                        page.blocking_wheel_registry_dirty.set(true);
                        page.document.borrow_mut().InvalidatePaint();
                        page.dirty.set(true);
                        page.scroll_only.set(false);
                    }
                }
                Self::Child { .. } => {
                    if let Some((frame, runtime)) = owner.ChildOwners() {
                        runtime.blocking_wheel_registry_dirty.set(true);
                        runtime.document.borrow_mut().InvalidatePaint();
                        frame.MarkNavigationDirty();
                    }
                }
            }
        }));
    }
    pub(super) fn ChildOwners(&self) -> Option<(Rc<LocalFrame>, Rc<LocalFrameRuntime>)> {
        let Self::Child { frame, runtime } = self else {
            return None;
        };
        let (frame, runtime) = (frame.upgrade()?, runtime.upgrade()?);
        frame.IsCurrentRuntime(&runtime).then_some((frame, runtime))
    }
    pub(super) fn Page(&self) -> Option<Rc<PageState>> {
        match self {
            Self::Root(page) => page.upgrade(),
            Self::Child { .. } => self.ChildOwners()?.0.domain.upgrade()?.page.upgrade(),
        }
    }
    pub(super) fn WithLayout<T>(
        &self,
        query: impl FnOnce(&mut layoutng_assembly::layout_engine::LayoutEngine) -> T,
    ) -> Option<T> {
        match self {
            Self::Root(page) => Some(query(&mut page.upgrade()?.layout_engine.borrow_mut())),
            Self::Child { .. } => Some(query(&mut self.ChildOwners()?.1.layout.borrow_mut())),
        }
    }
    pub(super) fn FocusedNodeId(&self) -> Option<u64> {
        match self {
            Self::Root(page) => page.upgrade()?.interaction.borrow().focused_node_id,
            Self::Child { .. } => {
                self.ChildOwners()?
                    .1
                    .interaction_state
                    .borrow()
                    .focused_node_id
            }
        }
    }
    pub(super) fn WithProgrammaticAnimations<T>(
        &self,
        query: impl FnOnce(&ProgrammaticScrollAnimatorRegistry) -> T,
    ) -> Option<T> {
        match self {
            Self::Root(page) => Some(query(page.upgrade()?.ProgrammaticAnimations())),
            Self::Child { .. } => Some(query(
                self.ChildOwners()?.1.programmatic_scroll_animators.get()?,
            )),
        }
    }
    pub(super) fn SetProgrammaticScrollOffset(
        &self,
        node: u64,
        offset: Offset,
        behavior: ViewportScrollBehavior,
        source_type: compositor::ScrollSourceType,
    ) {
        match self {
            Self::Root(page) => {
                if let Some(page) = page.upgrade() {
                    page.SetProgrammaticScrollOffset(node, offset, behavior, source_type);
                }
            }
            Self::Child { .. } => {
                if let Some((frame, runtime)) = self.ChildOwners() {
                    frame.SetProgrammaticScrollOffset(
                        &runtime,
                        node,
                        offset,
                        behavior,
                        source_type,
                    );
                }
            }
        }
    }
    pub(super) fn ApplyScroll(&self, mutation: page_mutation::ScrollMutation) {
        match self {
            Self::Root(page) => {
                if let Some(page) = page.upgrade() {
                    page.ApplyMutation(PageMutation::ScrollMutation(mutation));
                }
            }
            Self::Child { .. } => {
                let Some((frame, runtime)) = self.ChildOwners() else {
                    return;
                };
                if runtime
                    .document
                    .ApplyScrollOffset(mutation.target_node_id, mutation.offset)
                    .is_some()
                {
                    runtime.document.borrow_mut().InvalidatePaint();
                    frame.MarkNavigationDirty();
                }
            }
        }
    }
    pub(super) fn AutoscrollController(&self) -> Option<Rc<dyn ScrollAutoscrollController>> {
        match self {
            Self::Root(page) => page
                .upgrade()?
                .mouse_autoscroll
                .borrow()
                .upgrade()
                .map(|owner| owner as Rc<dyn ScrollAutoscrollController>),
            Self::Child { .. } => {
                let (frame, _) = self.ChildOwners()?;
                let domain = frame.domain.upgrade()?;
                if let Some(root) = domain.FocusInteraction(domain.root) {
                    if root
                        .SourceInputOwners()
                        .autoscroll
                        .MiddleClickAutoscrollInProgress()
                    {
                        return root
                            .SourceInputOwners()
                            .autoscroll_services
                            .borrow()
                            .as_ref()
                            .cloned()
                            .map(|owner| owner as Rc<dyn ScrollAutoscrollController>);
                    }
                }
                let engine = domain.FocusInteraction(frame.state.Identity())?;
                let result = engine
                    .SourceInputOwners()
                    .autoscroll_services
                    .borrow()
                    .as_ref()
                    .cloned();
                result.map(|owner| owner as Rc<dyn ScrollAutoscrollController>)
            }
        }
    }
    pub(super) fn BubbleLogicalScrollInParentFrame(
        &self,
        direction: ScrollDirection,
        granularity: ScrollGranularity,
    ) -> bool {
        let Some((frame, _)) = self.ChildOwners() else {
            return false;
        };
        let Some(domain) = frame.domain.upgrade() else {
            return false;
        };
        let Some(parent) = domain.FocusInteraction(frame.parent) else {
            return false;
        };
        // EventHandler::BubblingScroll starts in the owner element in the
        // parent frame. The parent's retained manager owns further bubbling.
        parent.BubbleLogicalScroll(
            direction,
            granularity,
            frame.state.Owner().map(|owner| owner.Id()),
        )
    }
    pub(super) fn Assemble(
        &self,
        engine: &interaction::Interaction<'_>,
    ) -> (
        Rc<PageScrollManagerOwners>,
        Rc<PageInteractionScrollRuntime>,
    ) {
        let layout: Rc<dyn interaction::adapter::page_scroll_manager::PageLogicalScrollLayout> =
            Rc::new(PageInteractionScrollLayout(self.clone()));
        let runtime = Rc::new(PageInteractionScrollRuntime::new(
            self.clone(),
            layout.clone(),
        ));
        let frame = Rc::new(PageInteractionScrollFrameState {
            owner: self.clone(),
            mouse: engine.SourceInputOwners().mouse.clone(),
            keyboard: engine.SourceInputOwners().keyboard.clone(),
        });
        let owner = self.clone();
        let effects: ScrollEffectEmitter = Rc::new(move |effect| match effect {
            ScrollEffect::SetOffset(mutation) => owner.ApplyScroll(page_mutation::ScrollMutation {
                target_node_id: mutation.node_id,
                offset: Offset {
                    x: f64::from(mutation.offset.x()),
                    y: f64::from(mutation.offset.y()),
                },
                scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kUser,
            }),
            ScrollEffect::ScrollFinished { .. } => {
                if let Some(page) = owner.Page() {
                    page.scroll_finished_hover_dirty.set(true);
                }
            }
            ScrollEffect::AnimationCompleted { .. } => {}
        });
        let owners = Rc::new(PageScrollManagerOwners::new(
            layout,
            runtime.clone(),
            frame,
            effects,
        ));
        assert!(engine.BindNativeWheelEventHandler(
            owners.clone(),
            interaction::scroll::core::input::mouse_wheel_event_manager::MouseWheelFeatures {
                fade_in_scrollbar_when_mouse_wheel_may_begin: cfg!(target_os = "macos"),
                defer_fade_out_scrollbar_until_mouse_wheel_ended: true,
            }
        ));
        (owners, runtime)
    }
}

pub(super) struct ChildInteractionScroll {
    owners: Rc<PageScrollManagerOwners>,
    runtime: Rc<PageInteractionScrollRuntime>,
    mouse: Weak<interaction::core::input::mouse_event_manager::MouseEventManagerState>,
}
impl LocalFrameRuntime {
    pub(super) fn ScrollableContainerForNode(
        &self,
        node_id: u64,
    ) -> Option<foundation::CompositorElementId> {
        self.interaction_scroll
            .borrow()
            .as_ref()?
            .owners
            .ScrollableContainerForNode(node_id)
    }
    pub(super) fn UserScrollAnimationTarget(
        &self,
        area_identity: usize,
    ) -> Option<foundation::gfx::Vector2dF> {
        self.interaction_scroll
            .borrow()
            .as_ref()?
            .runtime
            .animators
            .Get(area_identity)?
            .Snapshot()
            .desired_target_offset
    }
    pub(super) fn EnsureInteractionScrollOwners(
        self: &Rc<Self>,
        frame: &Rc<LocalFrame>,
        engine: &interaction::Interaction<'_>,
    ) {
        if !frame.IsCurrentRuntime(self) {
            return;
        }
        if let Some(domain) = frame.domain.upgrade() {
            if let Some(parent) = domain.Frame(frame.parent) {
                let parent_runtime = parent.runtime.borrow().clone();
                let parent_input = domain.FocusInteraction(parent.state.Identity());
                if let (Some(runtime), Some(engine)) = (parent_runtime, parent_input) {
                    runtime.EnsureInteractionScrollOwners(&parent, &engine);
                }
            }
        }
        let Some(identity) = self
            .layout
            .borrow()
            .LayoutViewportLogicalScrollArea()
            .map(|area| area.Identity())
        else {
            return;
        };
        if self
            .interaction_scroll
            .borrow()
            .as_ref()
            .is_some_and(|scroll| {
                scroll.owners.LayoutViewportIdentity() == identity
                    && scroll
                        .mouse
                        .upgrade()
                        .is_some_and(|mouse| Rc::ptr_eq(&mouse, &engine.SourceInputOwners().mouse))
            })
        {
            return;
        }
        let owner = InteractionScrollOwner::Child {
            frame: Rc::downgrade(frame),
            runtime: Rc::downgrade(self),
        };
        if let Some(page) = owner.Page() {
            let input = engine.SourceInputOwners();
            input
                .autoscroll_services
                .borrow_mut()
                .get_or_insert_with(|| {
                    interaction::adapter::page_mouse_autoscroll::PageMouseAutoscroll::new(
                        input.autoscroll.clone(),
                        Rc::new(super::mouse_autoscroll::PageAutoscrollStop(Rc::downgrade(
                            &page,
                        ))),
                    )
                });
        }
        let (owners, runtime) = owner.Assemble(engine);
        *self.interaction_scroll.borrow_mut() = Some(ChildInteractionScroll {
            owners,
            runtime,
            mouse: Rc::downgrade(&engine.SourceInputOwners().mouse),
        });
    }
}
impl LocalFrameDomain {
    pub(super) fn HasBlockingWheelListener(&self) -> bool {
        self.frames.borrow().values().any(|frame| {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                return false;
            };
            frame
                .navigation
                .borrow()
                .window
                .as_ref()
                .filter(|window| Rc::ptr_eq(&window.document, &runtime.document))
                .is_some_and(|window| {
                    window
                        .web_api
                        .DOMBindings()
                        .borrow()
                        .HasBlockingListener("wheel")
                })
        })
    }
    pub(super) fn BlockingWheelListenerTargets(&self) -> (Vec<(u64, Vec<u64>)>, bool) {
        let mut selected = Vec::new();
        let mut pending = false;
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            let window = frame
                .navigation
                .borrow()
                .window
                .clone()
                .filter(|window| Rc::ptr_eq(&window.document, &runtime.document));
            let Some(window) = window else { continue };
            let targets = window
                .web_api
                .DOMBindings()
                .borrow()
                .BlockingListenerTargets("wheel");
            if targets.is_empty() {
                continue;
            }
            pending |= runtime.blocking_wheel_registry_dirty.get();
            selected.push((frame.state.Identity(), targets));
        }
        (selected, pending)
    }
    pub(super) fn ReflectCompositorScrollElement(
        &self,
        element: foundation::CompositorElementId,
        offset: Offset,
        source_type: interaction::scroll::core::input::scroll_manager::ScrollSourceType,
    ) -> Option<u64> {
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            // The compositor offset is CSS/DIP. Each child Document owns its
            // own ConstraintSpace, so convert only after the element resolves
            // to that frame rather than applying the root document's zoom.
            let layout_zoom = runtime.constraints.borrow().layout_zoom_factor;
            let layout_zoom = if layout_zoom.is_finite() && layout_zoom > 0.0 {
                layout_zoom
            } else {
                1.0
            };
            let layout_offset = Offset {
                x: offset.x * layout_zoom,
                y: offset.y * layout_zoom,
            };
            let update = runtime.layout.borrow_mut().DidCompositorScroll(
                element,
                layout_offset,
                source_type,
            );
            if let Some(update) = update {
                let target = update.target_node_id;
                if runtime
                    .document
                    .ApplyScrollOffset(target, update.offset)
                    .is_some()
                {
                    runtime.compositor_scroll_pending.set(true);
                }
                return Some(target);
            }
        }
        None
    }
    pub(super) fn CommitCompositorScrollOffsets(&self) {
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            if runtime.compositor_scroll_pending.replace(false) {
                runtime.document.borrow_mut().InvalidatePaint();
                frame.MarkNavigationDirty();
            }
        }
    }
    pub(super) fn UpdateCompositorScrollbarsHiddenElement(
        &self,
        element: foundation::CompositorElementId,
        hidden: bool,
    ) -> bool {
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            let target = runtime
                .layout
                .borrow()
                .CompositorScrollArea(element)
                .map(|area| area.target_node_id);
            let Some(target) = target else {
                continue;
            };
            let changed = runtime
                .layout
                .borrow_mut()
                .UpdateScrollbarsHiddenIfOverlay(target, hidden);
            if changed {
                runtime.document.borrow_mut().InvalidatePaint();
                frame.MarkNavigationDirty();
            }
            return changed;
        }
        false
    }
    pub(super) fn TickInteractionScroll(&self, frame_time: std::time::Instant) {
        // Snapshot Frame identities, never their Layout/DOM trees. Animator
        // callbacks may navigate or detach a frame during offset application.
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            let animator = runtime
                .interaction_scroll
                .borrow()
                .as_ref()
                .map(|scroll| scroll.runtime.clone());
            if let Some(animator) = animator {
                animator.Tick(frame_time);
            }
        }
    }
}
