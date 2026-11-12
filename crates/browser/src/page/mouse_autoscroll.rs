//! Embedding effects for the single EventHandler AutoscrollController.
use super::*;
use interaction::adapter::page_mouse_autoscroll::AutoscrollHostEffect;
use interaction::default::core::page::autoscroll_controller::AutoscrollStopServices;

pub(super) struct PageAutoscrollStop(pub(super) Weak<PageState>);
impl AutoscrollStopServices for PageAutoscrollStop {
    // WebFrameWidgetImpl.cc:4549-4566. Performance belongs to this Window;
    // the input host receives the same document's ordered routing effect.
    fn NotifyAutoscrollForSelectionInMainFrame(&self, active: bool) {
        let Some(state) = self.0.upgrade() else {
            return;
        };
        state
            .client
            .borrow_mut()
            .DidAutoscroll(AutoscrollHostEffect::AutoscrollSelectionActive(active));
        if !active {
            if let Some(performance) = state.window_performance_timing.ExistingEventTimingSource() {
                performance.ResetAutoscroll();
            }
        }
    }
    fn SystemAutoscrollEnd(&self) {
        if let Some(state) = self.0.upgrade() {
            state
                .client
                .borrow_mut()
                .DidAutoscroll(AutoscrollHostEffect::AutoscrollEnd);
        }
    }
    fn SetCursorOverridden(&self, overridden: bool) {
        if let Some(state) = self.0.upgrade() {
            state.autoscroll_cursor_overridden.set(overridden);
        }
    }
    fn UpdateCursor(&self) {
        if let Some(state) = self.0.upgrade() {
            // Consumed by Page's existing post-input/lifecycle cursor refresh.
            state.scroll_finished_hover_dirty.set(true);
        }
    }
}

use super::local_frame_scroll::InteractionScrollOwner;
use interaction::default::core::page::autoscroll_controller::AutoscrollLayout;
use interaction::input::public::common::input::web_pointer_properties::PointF;
use layoutng_assembly::native_autoscroll_source::{NativeAutoscrollLayout, NativeAutoscrollSource};

/// Roots the original LayoutObject. All geometry and scrolling enters the
/// owning LayoutEngine for one call, exactly like the logical-scroll adapter.
struct PageAutoscrollLayout {
    owner: InteractionScrollOwner,
    native: NativeAutoscrollLayout,
}
impl PageAutoscrollLayout {
    fn Wrap(&self, native: NativeAutoscrollLayout) -> Rc<dyn AutoscrollLayout> {
        Rc::new(Self {
            owner: self.owner.clone(),
            native,
        })
    }
    fn Query<T>(&self, query: impl FnOnce(&NativeAutoscrollSource<'_>) -> T) -> Option<T> {
        self.owner.WithLayout(|engine| {
            let scroll = engine.MouseLayoutViewportScrollOffset().unwrap_or_default();
            let frame = layoutng_assembly::native_scroll_source::NativeScrollFrameGeometry {
                document_to_root_frame: Offset {
                    x: -(scroll.0 as f64),
                    y: -(scroll.1 as f64),
                },
            };
            let source = engine.NativeAutoscrollSource(&self.native, frame)?;
            Some(query(&source))
        })?
    }
    fn Document(&self) -> Option<Rc<DocumentEngine>> {
        match &self.owner {
            InteractionScrollOwner::Root(page) => Some(page.upgrade()?.document.clone()),
            InteractionScrollOwner::Child { .. } => {
                Some(self.owner.ChildOwners()?.1.document.clone())
            }
        }
    }
    fn LocalFramePoint(&self, p: (f64, f64)) -> Option<(f64, f64)> {
        if let Some((frame, _)) = self.owner.ChildOwners() {
            let point = frame.domain.upgrade()?.RootPointToFrameFloat(
                frame.state.Identity(),
                foundation::gfx::PointF::new(p.0 as f32, p.1 as f32),
            )?;
            Some((point.x() as f64, point.y() as f64))
        } else {
            Some(p)
        }
    }
    fn OwnerLayoutInParentFrame(&self) -> Option<Rc<dyn AutoscrollLayout>> {
        let (frame, _) = self.owner.ChildOwners()?;
        let domain = frame.domain.upgrade()?;
        let node = frame.state.Owner()?.Id();
        if frame.parent == domain.root {
            return LayoutForNode(&domain.page.upgrade()?, node);
        }
        let parent = domain.Frame(frame.parent)?;
        let runtime = parent.runtime.borrow().clone()?;
        LayoutForChildNode(&parent, &runtime, node)
    }
}
pub(super) fn LayoutForNode(state: &Rc<PageState>, node: u64) -> Option<Rc<dyn AutoscrollLayout>> {
    let native = state.layout_engine.borrow().AutoscrollLayoutFor(node)?;
    Some(Rc::new(PageAutoscrollLayout {
        owner: InteractionScrollOwner::Root(Rc::downgrade(state)),
        native,
    }))
}
pub(super) fn LayoutForChildNode(
    frame: &Rc<local_frame::LocalFrame>,
    runtime: &Rc<local_frame::LocalFrameRuntime>,
    node: u64,
) -> Option<Rc<dyn AutoscrollLayout>> {
    if !frame.IsCurrentRuntime(runtime) {
        return None;
    }
    let native = runtime.layout.borrow().AutoscrollLayoutFor(node)?;
    Some(Rc::new(PageAutoscrollLayout {
        owner: InteractionScrollOwner::Child {
            frame: Rc::downgrade(frame),
            runtime: Rc::downgrade(runtime),
        },
        native,
    }))
}
impl AutoscrollLayout for PageAutoscrollLayout {
    fn Identity(&self) -> usize {
        self.native.Identity()
    }
    fn FrameIdentity(&self) -> Option<usize> {
        self.Query(|_| match &self.owner {
            InteractionScrollOwner::Root(page) => page.as_ptr() as usize,
            InteractionScrollOwner::Child { frame, .. } => frame.as_ptr() as usize,
        })
    }
    fn HasPage(&self) -> bool {
        self.Document()
            .is_some_and(|document| document.borrow().GetDocument().IsActive())
    }
    fn IsBox(&self) -> bool {
        self.Query(|s| s.IsBox()).unwrap_or(false)
    }
    fn HasNode(&self) -> bool {
        self.Query(|s| s.NodeId().is_some()).unwrap_or(false)
    }
    fn NodeId(&self) -> Option<u64> {
        self.Query(|s| s.NodeId()).flatten()
    }
    fn IsAttachedOutermostMainFrame(&self) -> bool {
        matches!(self.owner, InteractionScrollOwner::Root(_)) && self.HasPage()
    }
    fn FindAutoscrollable(&self, middle: bool) -> Option<Rc<dyn AutoscrollLayout>> {
        use layoutng_assembly::native_autoscroll_source::NativeAutoscrollSearch;
        match self.Query(|source| source.FindAutoscrollableInFrame(middle))? {
            NativeAutoscrollSearch::Found(layout) => Some(self.Wrap(layout)),
            NativeAutoscrollSearch::ParentFrame => {
                self.OwnerLayoutInParentFrame()?.FindAutoscrollable(middle)
            }
            NativeAutoscrollSearch::NotFound => None,
        }
    }
    fn NodeAutoscrollBox(&self) -> Option<Rc<dyn AutoscrollLayout>> {
        let document = self.Document()?;
        let node = self.Query(|s| s.NodeId())??;
        let owner = document.borrow();
        let d = owner.GetDocument();
        let i = d.FindNodeById(node)?;
        // Node's default is null; HTMLSelectElement.cc:1371-1373 is the only
        // desktop override, returning its own box only for listbox selects.
        if !d.Node(i).IsHTMLElement("select") || d.SelectUsesMenuList(i) {
            return None;
        }
        drop(owner);
        self.Query(|s| s.IsBox())
            .unwrap_or(false)
            .then(|| self.Wrap(self.native.clone()))
    }
    fn StopNodeAutoscroll(&self) {
        // Ordinary Node::StopAutoscroll is empty. Select's source release is
        // executed by the call-scoped interaction transaction, never retained
        // as a dispatcher callback on Page.
    }
    fn CalculateAutoscrollDirection(&self, p: PointF) -> (f64, f64) {
        if let Some((frame, runtime)) = self.owner.ChildOwners() {
            let Some(mut rect) = self.Query(|source| source.ScrollingBoxRect()).flatten() else {
                return (0.0, 0.0);
            };
            let scroll = runtime
                .layout
                .borrow()
                .MouseLayoutViewportScrollOffset()
                .unwrap_or_default();
            rect = foundation::gfx::RectF::new(
                foundation::gfx::PointF::new(rect.x() - scroll.0, rect.y() - scroll.1),
                rect.size(),
            );
            let Some(domain) = frame.domain.upgrade() else {
                return (0.0, 0.0);
            };
            let rect = domain.MapChildRect(&frame, rect);
            return self
                .Query(|source| {
                    source
                        .CalculateAutoscrollDirectionForRootRect((p.x() as f64, p.y() as f64), rect)
                })
                .unwrap_or_default();
        }
        self.Query(|s| s.CalculateAutoscrollDirection((p.x() as f64, p.y() as f64)))
            .unwrap_or_default()
    }
    fn Autoscroll(&self, p: (f64, f64)) -> bool {
        let Some(p) = self.LocalFramePoint(p) else {
            return false;
        };
        let Some(result) = self.Query(|s| s.Autoscroll(p)) else {
            return false;
        };
        let did_scroll = result.did_scroll;
        match &self.owner {
            InteractionScrollOwner::Root(page) => {
                if let Some(page) = page.upgrade() {
                    page.ApplyNativeScrollResult(result);
                }
            }
            InteractionScrollOwner::Child { .. } => {
                if let Some((frame, runtime)) = self.owner.ChildOwners() {
                    frame.ApplyNativeScrollResult(&runtime, result);
                }
            }
        }
        did_scroll
    }
    fn ParentCrossingDocumentBoundary(&self) -> Option<Rc<dyn AutoscrollLayout>> {
        if let Some(parent) = self.Query(|s| s.Parent())? {
            return Some(self.Wrap(parent));
        }
        self.OwnerLayoutInParentFrame()
    }
    fn HasScrollableOverflow(&self, horizontal: bool) -> bool {
        self.Query(|s| s.HasScrollableOverflow(horizontal))
            .unwrap_or(false)
    }
    fn CanPropagate(&self, horizontal: bool) -> bool {
        self.Query(|s| {
            let (x, y, bx, by) = s.PropagationFacts();
            if horizontal {
                !x || bx == foundation::style_constants::EOverscrollBehavior::kAuto
            } else {
                !y || by == foundation::style_constants::EOverscrollBehavior::kAuto
            }
        })
        .unwrap_or(false)
    }
    fn IsUserScrollable(&self) -> bool {
        self.Query(|s| s.IsUserScrollable()).unwrap_or(false)
    }
}

impl local_frame::LocalFrameDomain {
    pub(super) fn AnimateChildAutoscroll(&self, frame_time: std::time::Instant) -> io::Result<()> {
        let Some(page) = self.page.upgrade() else {
            return Ok(());
        };
        let time = page.rendering.borrow().executor.TimeTicks(frame_time);
        let time = interaction::input::base::time::time::TimeTicks::FromInternalValue(
            time.since_origin().InMicroseconds(),
        );
        let frames = self.frames.borrow().values().cloned().collect::<Vec<_>>();
        for frame in frames {
            let Some(runtime) = frame.runtime.borrow().clone() else {
                continue;
            };
            let Some(engine) = self.FocusInteraction(frame.state.Identity()) else {
                continue;
            };
            let active = engine
                .SourceInputOwners()
                .autoscroll_services
                .borrow()
                .as_ref()
                .is_some_and(|owner| owner.NeedsAnimation());
            if !active || !frame.IsCurrentRuntime(&runtime) {
                continue;
            }
            let window = frame
                .navigation
                .borrow()
                .window
                .clone()
                .filter(|window| Rc::ptr_eq(&window.document, &runtime.document));
            if let Some(window) = window {
                let mut result = Ok(());
                window.executor.WithRuntimeAndRealm(&mut |script, realm| {
                    let script = RefCell::new(script);
                    let listeners = Rc::new(|invocation: &EventListenerInvocation<'_>, document: &interaction::InteractionDocument, _: &interaction::ownership::InteractionDOMMutationEmitter| {
                        window.interaction.DispatchListeners(document, invocation, &mut **script.borrow_mut(), realm, &mut |error| page.client.borrow_mut().DidReportScriptError(error))
                    });
                    let scoped = engine.WithScopedListeners(listeners);
                    result = scoped.AnimateSelectionAutoscroll(&runtime.document.Handle(), time);
                    window.interaction.CompleteFormSubmissions(&scoped);
                });
                result.map_err(io::Error::other)?;
            } else {
                engine
                    .AnimateSelectionAutoscroll(&runtime.document.Handle(), time)
                    .map_err(io::Error::other)?;
                runtime.CompleteNoScriptFormSubmissions(&frame, &engine);
            }
        }
        Ok(())
    }
}
