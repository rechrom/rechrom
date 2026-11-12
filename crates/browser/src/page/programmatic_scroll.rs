//! Frame assembly for the source ProgrammaticScrollAnimator. Every Frame has
//! its own retained animator registry and borrows only its resident Layout/DOM.
use super::*;
use interaction::scroll::core::scroll::programmatic_scroll_animator::{
    ProgrammaticCompositorAnimation, ProgrammaticScrollAnimationToken,
    ProgrammaticScrollAnimatorRegistry, ProgrammaticScrollAnimatorSink, ProgrammaticScrollOffset,
    ProgrammaticScrollRequest,
};
struct RootScrollSink(Weak<PageState>);
impl ProgrammaticScrollAnimatorSink for RootScrollSink {
    fn ScheduleAnimation(&self) -> bool {
        let Some(page) = self
            .0
            .upgrade()
            .filter(|page| page.has_begin_frame_source.get())
        else {
            return false;
        };
        page.animation_begin_frame_requested.set(true);
        true
    }
    fn AddCompositorAnimation(&self, _: ProgrammaticCompositorAnimation) -> bool {
        // This embedding currently schedules programmatic scroll on its
        // document sequence; the source animator owns the main-thread fallback.
        false
    }
    fn RemoveCompositorAnimation(
        &self,
        _: foundation::CompositorElementId,
        _: ProgrammaticScrollAnimationToken,
    ) {
    }
    fn ApplyOffset(&self, offset: ProgrammaticScrollOffset) {
        let Some(page) = self.0.upgrade() else {
            return;
        };
        if page
            .document
            .borrow()
            .GetDocument()
            .FindNodeById(offset.node_id)
            .is_none()
        {
            return;
        }
        page.ApplyMutation(PageMutation::ScrollMutation(
            page_mutation::ScrollMutation {
                target_node_id: offset.node_id,
                offset: Offset {
                    x: offset.offset.x() as f64,
                    y: offset.offset.y() as f64,
                },
                scroll_type:
                    layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic,
            },
        ));
    }
}
impl PageState {
    pub(super) fn InstallProgrammaticScroll(self: &Rc<Self>) {
        assert!(self
            .programmatic_scroll_animators
            .set(ProgrammaticScrollAnimatorRegistry::new(Rc::new(
                RootScrollSink(Rc::downgrade(self))
            )))
            .is_ok());
    }
    pub(super) fn ProgrammaticAnimations(&self) -> &ProgrammaticScrollAnimatorRegistry {
        self.programmatic_scroll_animators
            .get()
            .expect("Frame installed ProgrammaticScrollAnimator")
    }
    pub(super) fn SetProgrammaticScrollOffset(
        &self,
        node: u64,
        offset: Offset,
        behavior: ViewportScrollBehavior,
        source_type: compositor::ScrollSourceType,
    ) {
        let element = self
            .layout_engine
            .borrow_mut()
            .NativeScrollSource(node)
            .and_then(|mut source| source.ScrollElementIdentity());
        let Some(element) = element else {
            return;
        };
        let current = {
            let document = self.document.borrow();
            let doc = document.GetDocument();
            let Some(node) = doc.FindNodeById(node) else {
                return;
            };
            doc.ScrollOffsetFor(node)
        };
        let request = ProgrammaticScrollRequest {
            node_id: node,
            element,
            current_offset: foundation::gfx::Vector2dF::new(current.x as f32, current.y as f32),
            target_offset: foundation::gfx::Vector2dF::new(offset.x as f32, offset.y as f32),
            source_type,
        };
        let animator = self.ProgrammaticAnimations().GetOrCreate(element);
        if behavior == ViewportScrollBehavior::Smooth {
            animator.AnimateToOffset(request, None);
        } else {
            animator.ScrollToOffsetWithoutAnimation(request);
        }
    }
    pub(super) fn TickProgrammaticScroll(&self, frame_time: std::time::Instant) {
        let now = self.rendering.borrow().executor.TimeTicks(frame_time);
        if self.ProgrammaticAnimations().TickAnimations(now) {
            self.animation_begin_frame_requested.set(true);
        }
        if let Some(domain) = self.local_frames.get() {
            domain.TickProgrammaticScroll(now);
        }
    }
}
struct ChildScrollSink {
    frame: Weak<local_frame::LocalFrame>,
    runtime: Weak<local_frame::LocalFrameRuntime>,
}
impl ProgrammaticScrollAnimatorSink for ChildScrollSink {
    fn ScheduleAnimation(&self) -> bool {
        let (Some(frame), Some(runtime)) = (self.frame.upgrade(), self.runtime.upgrade()) else {
            return false;
        };
        if !frame.IsCurrentRuntime(&runtime) {
            return false;
        }
        let Some(page) = frame
            .domain
            .upgrade()
            .and_then(|domain| domain.page.upgrade())
            .filter(|page| page.has_begin_frame_source.get())
        else {
            return false;
        };
        page.animation_begin_frame_requested.set(true);
        true
    }
    fn AddCompositorAnimation(&self, _: ProgrammaticCompositorAnimation) -> bool {
        false
    }
    fn RemoveCompositorAnimation(
        &self,
        _: foundation::CompositorElementId,
        _: ProgrammaticScrollAnimationToken,
    ) {
    }
    fn ApplyOffset(&self, offset: ProgrammaticScrollOffset) {
        let (Some(frame), Some(runtime)) = (self.frame.upgrade(), self.runtime.upgrade()) else {
            return;
        };
        if !frame.IsCurrentRuntime(&runtime) {
            return;
        }
        let effect = runtime.document.ApplyScrollOffset(
            offset.node_id,
            Offset {
                x: offset.offset.x() as f64,
                y: offset.offset.y() as f64,
            },
        );
        if effect.is_some() {
            runtime.document.borrow_mut().InvalidatePaint();
            frame.MarkNavigationDirty();
        }
    }
}
impl local_frame::LocalFrameRuntime {
    pub(super) fn InstallProgrammaticScroll(self: &Rc<Self>, frame: &Rc<local_frame::LocalFrame>) {
        assert!(self
            .programmatic_scroll_animators
            .set(ProgrammaticScrollAnimatorRegistry::new(Rc::new(
                ChildScrollSink {
                    frame: Rc::downgrade(frame),
                    runtime: Rc::downgrade(self)
                }
            )))
            .is_ok());
    }
}
impl local_frame::LocalFrame {
    pub(super) fn SetProgrammaticScrollOffset(
        &self,
        runtime: &Rc<local_frame::LocalFrameRuntime>,
        node: u64,
        offset: Offset,
        behavior: ViewportScrollBehavior,
        source_type: compositor::ScrollSourceType,
    ) {
        if !self.IsCurrentRuntime(runtime) {
            return;
        }
        let element = runtime
            .layout
            .borrow_mut()
            .NativeScrollSource(node)
            .and_then(|mut source| source.ScrollElementIdentity());
        let Some(element) = element else {
            return;
        };
        let current = {
            let document = runtime.document.borrow();
            let doc = document.GetDocument();
            let Some(node) = doc.FindNodeById(node) else {
                return;
            };
            doc.ScrollOffsetFor(node)
        };
        let request = ProgrammaticScrollRequest {
            node_id: node,
            element,
            current_offset: foundation::gfx::Vector2dF::new(current.x as f32, current.y as f32),
            target_offset: foundation::gfx::Vector2dF::new(offset.x as f32, offset.y as f32),
            source_type,
        };
        let animator = runtime
            .programmatic_scroll_animators
            .get()
            .expect("child Frame animator owner")
            .GetOrCreate(element);
        if behavior == ViewportScrollBehavior::Smooth {
            animator.AnimateToOffset(request, None);
        } else {
            animator.ScrollToOffsetWithoutAnimation(request);
        }
    }
}
impl local_frame::LocalFrameDomain {
    fn TickProgrammaticScroll(&self, now: scroll_animation::TimeTicks) {
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            if let Some(animators) = runtime.programmatic_scroll_animators.get() {
                if animators.TickAnimations(now) {
                    if let Some(page) = self.page.upgrade() {
                        page.animation_begin_frame_requested.set(true);
                    }
                }
            }
        }
    }
}
