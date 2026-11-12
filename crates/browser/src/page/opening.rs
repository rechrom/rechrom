use super::*;
use document_loader::ResourceFetcher;
use open::{
    dynamic_scripts::DynamicScriptTasks, script_scheduler::ScriptLoadClient, OpenDocumentAdapter,
    OpenDocumentPipeline, OpenDocumentServices,
};
use std::{cell::RefCell, io, rc::Rc};

pub(super) fn ResetRootDocumentInputForCommit(state: &Rc<PageState>) {
    // Commit owns the document-token transition. Cancel the old detection
    // and ACK/timer state on the same source queue; a provisional request
    // must leave the current document's active gesture intact.
    let input_generation = state
        .document_input_queue
        .get()
        .map(|inputs| inputs.ResetGestureDetectionForDocumentCommit());
    // Preserve already completed compositor acknowledgements in FIFO
    // order. Unsent gestures still belong to the detached document and
    // must never be forwarded through the replacement client's binder.
    {
        let mut pending = state.pending_compositor_mutations.borrow_mut();
        pending.retain(|input| matches!(input, super::PageBrowserInput::Compositor(_)));
        if let Some(generation) =
            input_generation.filter(|_| state.rendering.borrow().external_presentation)
        {
            pending.push(super::PageBrowserInput::DocumentDetached { generation });
        }
    }
}

pub(super) fn CreateRootDocumentResources(
    state: &Rc<PageState>,
    resource_engine: Rc<ResourceEngine>,
) -> Rc<ResourceFetcher> {
    let resources = Rc::new(ResourceFetcher::WithEngine(
        resource_engine.clone(),
        state.document.Handle(),
        state.constraints.clone(),
    ));
    let image_security_document = state.document.Handle();
    resources.SetHTMLImageFetchSecurity(Rc::new(move || {
        let owner = image_security_document.borrow();
        FrameFetchSecurity(owner.GetDocument())
    }));
    resources.SetFontFetchSecuritySource(Rc::new(FrameFetchSecurity));
    let responsive_state = Rc::downgrade(&state);
    resources.SetResponsiveImageSelector(Rc::new(move |document, node, constraints, is_loaded| {
        let state = responsive_state.upgrade()?;
        crate::style_services::SelectResponsiveImage(
            document,
            node,
            constraints,
            state.preferred_color_scheme.get(),
            state.forced_colors.get(),
            is_loaded,
        )
    }));
    let responsive_state = Rc::downgrade(&state);
    resources.SetPreloadImageSelector(Rc::new(move |input, constraints, is_loaded| {
        let state = responsive_state.upgrade()?;
        crate::style_services::SelectResponsiveImageFromAttributes(
            input,
            constraints,
            state.preferred_color_scheme.get(),
            state.forced_colors.get(),
            is_loaded,
        )
    }));
    let responsive_state = Rc::downgrade(&state);
    resources.SetPreloadMediaMatcher(Rc::new(move |query, constraints| {
        let Some(state) = responsive_state.upgrade() else {
            return false;
        };
        let media = crate::style_services::MediaValuesForLayout(
            constraints,
            state.preferred_color_scheme.get(),
            state.forced_colors.get(),
        );
        let values = style::media_queries::MediaValuesCached::new(&media);
        crate::style_services::ResponsiveMediaMatches(query, &values)
    }));
    state.ConnectResources(resources.clone());
    resources
}

pub(super) fn InstallRootDocumentInteraction(
    state: &Rc<PageState>,
    scripts: &Option<Box<PageScripts>>,
) -> (
    interaction::Interaction<'static>,
    Option<Rc<interaction::adapter::page_form_controller::PageFormController>>,
) {
    let interaction = if let Some(scripts) = &scripts {
        scripts.Engine()
    } else {
        let mutate = state.clone();
        let interaction = ::interaction::Interaction::WithSelectionState(
            Rc::new(move |output| mutate.ApplyInteractionOutput(output)),
            None,
            state.layout_editing.selections.clone(),
            state.InteractionEventClock(),
        );
        interaction
            .FormControlState()
            .SetValidationPatternEngine(webapi::form_validation::CreateValidationPatternEngine());
        interaction.BindDocumentInputLifecycle(&state.document.Handle());
        interaction
    };
    interaction.BindWindowPerformance(state.window_performance_timing.EventTimingSource());
    let form_controller = if scripts.is_none() {
        let controller = interaction::adapter::page_form_controller::PageFormController::Install(
            state.document.Handle(),
            interaction.clone(),
        );
        let page = Rc::downgrade(&state);
        let identity = state.document.borrow().GetDocument().RootHandle();
        controller.SetTaskRunner(Rc::new(move |task| {
            if let Some(page) = page.upgrade() {
                page.popup_tasks.Post(
                    identity,
                    0.0,
                    popup_tasks::PageInteractionTask::FormRestore(Rc::new(RefCell::new(Some(
                        task,
                    )))),
                );
            }
        }));
        Some(controller)
    } else {
        None
    };
    interaction
        .FormControlState()
        .SetValidationMessageClient(validation_message::PageValidationMessageClient::new(&state));
    let interaction_geometry = state.InteractionMouseFrameGeometry();
    interaction.SetMouseFrameGeometry(Some(interaction_geometry.clone()));
    interaction.BindPointerInput(
        Rc::new(pointer_geometry::PagePointerGeometryProvider(
            Rc::downgrade(&state),
        )),
        Rc::new(
            pointer_scroll_controls::PagePointerScrollControlsProvider::new(
                Rc::downgrade(&state),
                interaction.SourceInputOwners().mouse.clone(),
            ),
        ),
    );
    select_control_host::ResidentSelectControlHost::Root(&state, &interaction);
    let focus_effect_state = Rc::downgrade(&state);
    let focus_effects: interaction::default::core::page::focus_controller::FocusEffectEmitter =
        Rc::new(move |_effect| {
            if let Some(state) = focus_effect_state.upgrade() {
                // FocusController's current emitted effect is active-state
                // custom-scrollbar invalidation. Until Paint exports that
                // narrower mutation, retain correctness with one paint frame.
                state.dirty.set(true);
            }
        });
    assert!(interaction.BindFocusNavigation(
        interaction_geometry.clone(),
        interaction_geometry.clone(),
        interaction_geometry,
        interaction::adapter::page_focus_navigation::FocusNavigationPolicy {
            tabs_to_links: true,
            spatial_navigation_enabled: false,
        },
        focus_effects,
    ));
    interaction.SetFormNavigationPreparation(Some(Rc::new(
        local_frame_navigation::FormNavigationPreparation::new(
            state.local_frames.get().unwrap(),
            state.local_frames.get().unwrap().root,
        ),
    )));
    *state
        .local_frames
        .get()
        .expect("native FrameTree")
        .focus_controller
        .borrow_mut() = interaction.FocusControllerOwner();
    if let Some(controller) = interaction.FocusControllerOwner() {
        let domain = state.local_frames.get().expect("native FrameTree");
        controller.SetFocusedFrame(domain.FocusFrame(domain.root), false);
    }
    *state.resident_frame_selection.borrow_mut() = Rc::downgrade(
        &interaction
            .FrameSelectionOwner()
            .expect("canonical frame selection owner"),
    );
    *state.popup_source_interaction.borrow_mut() = Some(interaction.clone());
    {
        let domain = state.local_frames.get().expect("native FrameTree");
        interaction.SetEventHandlerEmbedding(
            &state.document.Handle(),
            native_event_handler_host::NativeEventHandlerEmbedding::new(domain, domain.root),
        );
        interaction.SetAnchorNavigationHost(domain.AnchorNavigationHost(domain.root).map(|host| {
            host as Rc<
                dyn interaction::default::core::html::html_anchor_element::AnchorNavigationHost,
            >
        }));
    }
    interaction
        .SetGestureTimerServices(Rc::new(popup_tasks::PageGestureTimerServices::new(&state)));
    (interaction, form_controller)
}

pub(super) fn ResolvePageStylesAndLoadResources(
    state: &Rc<PageState>,
    resources: &Rc<ResourceFetcher>,
    scripts: &mut Option<Box<PageScripts>>,
    allowance: usize,
    rendering_opportunity: bool,
    drain_render_blocking_styles: bool,
) -> io::Result<usize> {
    let mut trace = browser_tracing::span("lifecycle", "Page.ResolveStylesAndLoadResources");
    trace.set(
        "drain_render_blocking_styles",
        drain_render_blocking_styles as u8 as f64,
    );
    // Synchronous JS geometry queries may resolve styles before this pass;
    // resource discovery has its own dirty state and cannot use style-cache
    // validity as a proxy. Clear before callbacks so reentrant mutations
    // remain dirty for the next discovery pass.
    let profile = std::env::var_os("BROWSER_PROFILE_RESOURCES").map(|_| std::time::Instant::now());
    let elapsed = || profile.map(|start| start.elapsed()).unwrap_or_default();
    // Network/parser task turns may complete resources, but they are not a
    // rendering opportunity.  Blink batches parser mutations until the next
    // lifecycle update; resolving the whole growing document after every
    // bounded parser pump makes streaming input quadratic and publishes
    // arbitrary intermediate frames.  Keep discovery dirty until Page enters
    // UpdateFrameIfNeeded from BeginFrame.
    let discovery = rendering_opportunity
        .then(|| {
            state
                .resource_references_dirty
                .replace(ResourceReferenceDirty::default())
        })
        .unwrap_or_default();
    let discover = !discovery.IsEmpty();
    trace.set("discover", discover as u8 as f64);
    trace.set("discovery_flags", discovery.0 as f64);
    if rendering_opportunity {
        state.ResolveStyles();
    }
    let style_before_done = elapsed();
    let mut images_discovered_done = style_before_done;
    let mut fonts_discovered_done = style_before_done;
    if discovery.Contains(ResourceReferenceDirty::STYLE_SHEETS) {
        // Scroll/caret/unchanged hover updates leave all resource references
        // intact. DOM and stylesheet changes still discover resources;
        // image completion only refreshes consumers. Completions are polled on
        // every pass with the existing event and admission budget.
        let style_discovery = browser_tracing::span("lifecycle", "Page.DiscoverStyleSheets");
        if let Err(error) = resources.QueueReferencedStyleSheets() {
            state.MarkResourceReferencesDirty(discovery);
            return Err(error);
        }
        drop(style_discovery);
    }
    if discovery.Contains(ResourceReferenceDirty::IMAGES) {
        let image_discovery = browser_tracing::span("lifecycle", "Page.DiscoverImages");
        if let Err(error) = resources.QueueReferencedImages() {
            state.MarkResourceReferencesDirty(ResourceReferenceDirty(
                discovery.0 & (ResourceReferenceDirty::IMAGES | ResourceReferenceDirty::FONTS),
            ));
            return Err(error);
        }
        drop(image_discovery);
        images_discovered_done = elapsed();
    }
    if discovery.Contains(ResourceReferenceDirty::FONTS) {
        let font_discovery = browser_tracing::span("lifecycle", "Page.DiscoverFonts");
        resources.PrepareFontFaces();
        resources.StartPendingFonts();
        drop(font_discovery);
        fonts_discovered_done = elapsed();
    }
    // Raw keepalive fetches share the general resource completion turn without
    // entering image discovery, decoding, events or load-event accounting.
    let raw_resources = resources.PollPendingRawResources();
    trace.set("raw_resource_completions", raw_resources as f64);
    let mut client = ResourceClient {
        state: state.clone(),
        scripts: scripts.as_deref_mut(),
    };
    let style_allowance = if drain_render_blocking_styles {
        usize::MAX
    } else {
        allowance
    };
    let style_sheets = {
        let _trace = browser_tracing::span("lifecycle", "Page.PollStyleSheets");
        resources.PollPendingStyleSheetsWithBudget(&mut client, style_allowance)
    };
    // A render-blocking stylesheet drain consumes this task's ordinary
    // completion opportunity, but never borrows image/font budget from a
    // future task turn.
    let remaining_allowance = allowance.saturating_sub(style_sheets.min(allowance));
    let images = {
        let _trace = browser_tracing::span("lifecycle", "Page.PollImages");
        resources.PollPendingImagesWithBudget(&mut client, remaining_allowance)
    };
    let images_polled_done = elapsed();
    let fonts = {
        let _trace = browser_tracing::span("lifecycle", "Page.PollFonts");
        resources
            .PollPendingFontsWithBudget(&mut client, remaining_allowance.saturating_sub(images))
    };
    trace.set("image_completions", images as f64);
    trace.set("font_completions", fonts as f64);
    trace.set("stylesheet_completions", style_sheets as f64);
    let fonts_polled_done = elapsed();
    drop(client);
    if rendering_opportunity {
        state.ResolveStyles();
    }
    let style_after_done = elapsed();
    if let Some(scripts) = scripts.as_deref() {
        let _trace = browser_tracing::span("lifecycle", "Page.FlushScriptTasks");
        scripts.FlushTasks();
    }
    if let Some(start) = profile {
        let total = start.elapsed();
        if total.as_millis() >= 16 {
            eprintln!(
                "page-resources-profile discover={discover} style_before_ms={:.3} image_discovery_ms={:.3} font_discovery_ms={:.3} image_poll_ms={:.3} font_poll_ms={:.3} style_after_ms={:.3} flush_tasks_ms={:.3} total_ms={:.3}",
                style_before_done.as_secs_f64() * 1000.0,
                (images_discovered_done - style_before_done).as_secs_f64() * 1000.0,
                (fonts_discovered_done - images_discovered_done).as_secs_f64() * 1000.0,
                (images_polled_done - fonts_discovered_done).as_secs_f64() * 1000.0,
                (fonts_polled_done - images_polled_done).as_secs_f64() * 1000.0,
                (style_after_done - fonts_polled_done).as_secs_f64() * 1000.0,
                (total - style_after_done).as_secs_f64() * 1000.0,
                total.as_secs_f64() * 1000.0
            );
        }
    }
    Ok(style_sheets + images + fonts)
}

/// Browser-facing wiring for `OpenEngine`.
///
/// Navigation order and load completion live in `OpenEngine`; this adapter only
/// routes its typed operations to the concrete document, resource and script
/// services assembled by `Page`.
pub(super) struct PageOpenAdapter<'a> {
    state: &'a mut Rc<PageState>,
    resources: &'a mut Rc<ResourceFetcher>,
    interaction: &'a mut interaction::Interaction<'static>,
    form_controller:
        &'a mut Option<Rc<interaction::adapter::page_form_controller::PageFormController>>,
    scripts: &'a mut Option<Box<PageScripts>>,
    script_client: &'a mut Rc<RefCell<dyn ScriptLoadClient>>,
}

impl<'a> PageOpenAdapter<'a> {
    pub(super) fn new(
        state: &'a mut Rc<PageState>,
        resources: &'a mut Rc<ResourceFetcher>,
        interaction: &'a mut interaction::Interaction<'static>,
        form_controller: &'a mut Option<
            Rc<interaction::adapter::page_form_controller::PageFormController>,
        >,
        scripts: &'a mut Option<Box<PageScripts>>,
        script_client: &'a mut Rc<RefCell<dyn ScriptLoadClient>>,
    ) -> Self {
        Self {
            state,
            resources,
            interaction,
            form_controller,
            scripts,
            script_client,
        }
    }
}

impl OpenDocumentAdapter for PageOpenAdapter<'_> {
    fn InitializeDocument(
        &mut self,
        document_url: &str,
        resource_engine: Rc<ResourceEngine>,
        owners: open::OpenDocumentOwners,
    ) -> io::Result<Option<open::OpenDocumentWindowInstallation>> {
        let previous = self.state.clone();
        let document = Rc::new(DocumentEngine::new());
        {
            let mut owner = document.borrow_mut();
            let next = owner.GetDocumentMut();
            next.InheritBlobURLRegistryFrom(previous.document.borrow().GetDocument());
            let origin = dom::persistent_document::SecurityOrigin::CreateForNavigation(
                document_url,
                None,
                Rc::new(dom::persistent_document::SecurityOriginSchemeRegistry::default()),
            );
            next.InitializeSecurityContextForNavigation(origin, false);
            next.SetDocumentURL(document_url.to_owned());
        }
        let choice = document_loader::StaticDocumentWindowChoice::ForNavigation(
            previous.document.borrow().GetDocument(),
            document.borrow().GetDocument().SecurityContext().Origin(),
            // The root has no anonymous owner and both actual isolation
            // owners currently use the non-isolated AgentCluster branch.
            (previous
                .document
                .borrow()
                .GetDocument()
                .ExecutionContextIsolation()
                .CrossOriginIsolationKeyKnownAbsent()
                && document
                    .borrow()
                    .GetDocument()
                    .ExecutionContextIsolation()
                    .CrossOriginIsolationKeyKnownAbsent())
            .then_some(document_loader::StaticDocumentWindowReuseFacts {
                window_anonymous_matching: true,
                cross_origin_isolation_keys_match: true,
            }),
        );
        let timing = if choice.Disposition()
            == document_loader::StaticDocumentWindowDisposition::ReuseInitialWindow
        {
            document
                .borrow_mut()
                .GetDocumentMut()
                .ReuseWindowExecutionContext(previous.document.borrow().GetDocument());
            previous.window_performance_timing.clone()
        } else {
            Rc::new(webapi::window_bindings::WindowPerformanceTiming::new())
        };
        let state = PageState::ForNewDocument(&previous, document, timing);
        state.InstallDocumentLoaderOwners(owners);
        previous.HideValidationMessage();
        let previous_resources = previous.resources.borrow().clone();
        if let Some(resources) = previous_resources {
            resources.DetachDocumentFetchContext()?;
        }
        previous.CancelImageEvents();
        previous.local_frames.get().unwrap().DetachChildren();
        let previous_interaction = previous.popup_source_interaction.borrow().clone();
        if let Some(interaction) = previous_interaction {
            interaction.CaptureMouseEventsToWidget(false);
        }
        // Document::Shutdown detaches the resident LayoutObject tree before
        // disposing StyleEngine's resolver and selector owners. The Page
        // domain survives navigation, but these document engines do not.
        previous.ShutdownDocumentEngines();
        if choice.Disposition()
            == document_loader::StaticDocumentWindowDisposition::ReuseInitialWindow
        {
            previous
                .document
                .borrow()
                .GetDocument()
                .ShutdownForWindowReuse();
        } else {
            previous
                .document
                .borrow()
                .GetDocument()
                .SetLifecycleActive(false);
        }
        state.pending_compositor_mutations.borrow_mut().extend(
            previous
                .pending_compositor_mutations
                .borrow_mut()
                .drain(..)
                .filter(|input| matches!(input, super::PageBrowserInput::Compositor(_))),
        );
        let resources = CreateRootDocumentResources(&state, resource_engine);
        ResetRootDocumentInputForCommit(&state);
        let script_client: Rc<RefCell<dyn ScriptLoadClient>> = Rc::new(RefCell::new(ScriptClient(
            state.client.clone(),
            Rc::downgrade(&state),
        )));
        if let Some(scripts) = self.scripts.as_deref_mut() {
            let installed = if choice.Disposition()
                == document_loader::StaticDocumentWindowDisposition::ReuseInitialWindow
            {
                scripts.ReuseInitialWindow(state.clone(), &choice)
            } else {
                scripts
                    .PrepareNewWindow(state.clone())
                    .and_then(|prepared| scripts.CreateNewWindowRealm(prepared, &choice))
            }
            .map_err(|error| io::Error::other(format!("Window installation failed: {error:?}")))?;
            for error in installed.BootstrapErrors() {
                state.client.borrow_mut().DidReportScriptError(error);
            }
            scripts.SetScriptLoadClient(script_client.clone());
            // InstallNewDocument publishes only after all new native services
            // exist; no parser or author script can observe partial owners.
            scripts.PublishNewWindow(installed).map_err(|error| {
                io::Error::other(format!("Window publication failed: {error:?}"))
            })?;
        } else {
            state.PublishPreparedDocumentInputs();
        }
        state
            .local_frames
            .get()
            .unwrap()
            .page
            .Rebind(Rc::downgrade(&state));
        let (interaction, form_controller) = InstallRootDocumentInteraction(&state, self.scripts);
        *self.state = state;
        *self.resources = resources;
        *self.script_client = script_client;
        *self.interaction = interaction;
        *self.form_controller = form_controller;
        Ok(Some(open::OpenDocumentWindowInstallation {
            script_executor: self.scripts.as_ref().map(|scripts| scripts.Executor()),
        }))
    }
    fn InitializeWindowNavigation(
        &mut self,
        loader: Rc<document_loader::DocumentLoaderNavigationState>,
    ) {
        loader.EnsureHistoryItemForWindow(self.state.document.borrow().GetDocument());
        if let Some(domain) = self.state.local_frames.get() {
            domain.InstallHistoryRestore(domain.root, &loader);
        }
        if let Some(scripts) = self.scripts.as_ref() {
            scripts.InitializeWindowNavigation(loader);
        } else if let Some(item) = loader.CurrentHistoryItem() {
            let snapshot = self.state.SnapshotCurrentHistoryItem(&item);
            self.state.client.borrow_mut().DidCommitHistory(
                snapshot,
                loader
                    .LoadType()
                    .unwrap_or(navigation_history::FrameLoadType::Standard),
            );
        }
    }
    fn CommitURL(&mut self, url: &str) {
        self.state.HideValidationMessage();
        self.state
            .document
            .borrow()
            .GetDocument()
            .CommitNavigationDocumentToken();
        // InitializeDocument selected the immutable origin before bindings
        // and fetch contexts retained it. URL publication keeps that owner.
        self.resources.SetDocumentURL(url.to_owned());
        self.state
            .document
            .borrow_mut()
            .GetDocumentMut()
            .SetDocumentURL(url.to_owned());
        if let Some(scripts) = self.scripts.as_ref() {
            scripts.SetURL(url.to_owned());
        }
        self.state.client.borrow_mut().DidCommit(url);
    }

    fn PrepareDocumentServices(&mut self, document_url: &str) -> OpenDocumentServices {
        self.state.InstallPopupTaskServices();
        super::select_control_host::ResidentSelectControlHost::Root(&self.state, &self.interaction);
        if !self
            .state
            .document
            .borrow()
            .GetDocument()
            .HasNavigationSecurityContext()
        {
            let origin = dom::persistent_document::SecurityOrigin::CreateForNavigation(
                document_url,
                None,
                Rc::new(dom::persistent_document::SecurityOriginSchemeRegistry::default()),
            );
            self.state
                .document
                .borrow_mut()
                .GetDocumentMut()
                .InitializeSecurityContextForNavigation(origin, false);
        }
        self.state.CancelImageEvents();
        self.state.InvalidateLayoutOutput();
        let media_state = self.state.clone();
        OpenDocumentServices {
            document: self.state.document.Handle(),
            resources: self.resources.clone(),
            scripts: self.scripts.as_deref().map(PageScripts::OpenServices),
            render_blocking_media_matches: Rc::new(move |condition| {
                let constraints = media_state.constraints.borrow();
                let values = crate::style_services::MediaValuesForLayout(
                    &constraints,
                    media_state.preferred_color_scheme.get(),
                    media_state.forced_colors.get(),
                );
                let evaluator =
                    style::production_style_sheet::ProductionMediaQueryEvaluator::FromCachedData(
                        &values,
                    );
                let queries =
                    style::media_queries::media_query_backend::ParseMediaQuerySet(condition);
                evaluator.EvalSet(&queries)
            }),
        }
    }

    fn InstallDynamicScriptHooks(&mut self, dynamic: &Rc<DynamicScriptTasks>) {
        self.scripts
            .as_deref_mut()
            .expect("dynamic scripts require a Page script environment")
            .InstallDynamicScriptHooks(dynamic);
    }

    fn WithScriptLoadClient(&mut self, callback: &mut dyn FnMut(&mut dyn ScriptLoadClient)) {
        callback(&mut *self.script_client.borrow_mut());
    }

    fn UpdateDocumentResources(
        &mut self,
        allowance: usize,
        drain_render_blocking_styles: bool,
    ) -> io::Result<usize> {
        ResolvePageStylesAndLoadResources(
            &self.state,
            &self.resources,
            self.scripts,
            allowance,
            false,
            drain_render_blocking_styles,
        )
    }

    fn HasPendingImageEvents(&self) -> bool {
        !self.state.pending_image_events.borrow().is_empty()
    }

    fn HasPendingLoadBlockingImages(&self) -> bool {
        self.resources.HasPendingLoadBlockingImages()
    }

    fn LoadEventDelayCount(&self) -> u32 {
        self.state
            .document
            .borrow()
            .GetDocument()
            .LoadEventDelayCount()
    }

    fn FinishLoad(&mut self, pipeline: &mut OpenDocumentPipeline) {
        if let OpenDocumentPipeline::Script {
            scheduler,
            executor,
        } = pipeline
        {
            self.scripts
                .as_deref_mut()
                .expect("script pipeline requires Page scripts")
                .FinishLoad(Some(scheduler), executor.as_ref());
        }
        {
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            document.SetReadyState(dom::document_parser_lifecycle::DocumentReadyState::Complete);
            document
                .SetParsingState(dom::document_parser_lifecycle::DocumentParsingState::Finished);
            document.SetLoadEventProgress(
                dom::document_parser_lifecycle::DocumentLoadEventProgress::Completed,
            );
        }
        self.state.client.borrow_mut().DidFinishLoad();
    }

    fn FailLoad(&mut self, message: &str) {
        self.state.client.borrow_mut().DidFail(message);
    }
}
