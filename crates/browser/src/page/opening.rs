use super::{PageScripts, PageState, ResourceClient};
use document_loader::ResourceFetcher;
use open::{
    dynamic_scripts::DynamicScriptTasks, script_scheduler::ScriptLoadClient, OpenDocumentAdapter,
    OpenDocumentPipeline, OpenDocumentServices,
};
use std::{cell::RefCell, io, rc::Rc};

pub(super) fn ResolvePageStylesAndLoadResources(
    state: &Rc<PageState>,
    resources: &Rc<ResourceFetcher>,
    scripts: &mut Option<Box<PageScripts>>,
    allowance: usize,
) -> io::Result<usize> {
    let mut trace = browser_tracing::span("lifecycle", "Page.ResolveStylesAndLoadResources");
    // Synchronous JS geometry queries may resolve styles before this pass;
    // resource discovery has its own dirty state and cannot use style-cache
    // validity as a proxy. Clear before callbacks so reentrant mutations
    // remain dirty for the next discovery pass.
    let profile = std::env::var_os("BROWSER_PROFILE_RESOURCES").map(|_| std::time::Instant::now());
    let elapsed = || profile.map(|start| start.elapsed()).unwrap_or_default();
    let discover = state.resource_references_dirty.replace(false);
    trace.set("discover", discover as u8 as f64);
    state.ResolveStyles();
    let style_before_done = elapsed();
    let mut images_discovered_done = style_before_done;
    let mut fonts_discovered_done = style_before_done;
    if discover {
        // Scroll/caret/unchanged hover updates leave all resource references
        // intact. DOM and stylesheet changes still discover resources;
        // image completion only refreshes consumers. Completions are polled on
        // every pass with the existing event and admission budget.
        let image_discovery = browser_tracing::span("lifecycle", "Page.DiscoverImages");
        if let Err(error) = resources.QueueReferencedImages() {
            state.resource_references_dirty.set(true);
            return Err(error);
        }
        drop(image_discovery);
        images_discovered_done = elapsed();
        let font_discovery = browser_tracing::span("lifecycle", "Page.DiscoverFonts");
        resources.SelectUsedFontFaces();
        resources.StartPendingFonts();
        drop(font_discovery);
        fonts_discovered_done = elapsed();
    }
    let mut client = ResourceClient {
        state: state.clone(),
        scripts: scripts.as_deref_mut(),
    };
    let images = {
        let _trace = browser_tracing::span("lifecycle", "Page.PollImages");
        resources.PollPendingImagesWithBudget(&mut client, allowance)
    };
    let images_polled_done = elapsed();
    let fonts = {
        let _trace = browser_tracing::span("lifecycle", "Page.PollFonts");
        resources.PollPendingFontsWithBudget(&mut client, allowance.saturating_sub(images))
    };
    trace.set("image_completions", images as f64);
    trace.set("font_completions", fonts as f64);
    let fonts_polled_done = elapsed();
    drop(client);
    state.ResolveStyles();
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
    Ok(images + fonts)
}

/// Browser-facing wiring for `OpenEngine`.
///
/// Navigation order and load completion live in `OpenEngine`; this adapter only
/// routes its typed operations to the concrete document, resource and script
/// services assembled by `Page`.
pub(super) struct PageOpenAdapter<'a> {
    state: Rc<PageState>,
    resources: Rc<ResourceFetcher>,
    scripts: &'a mut Option<Box<PageScripts>>,
    script_client: Rc<RefCell<dyn ScriptLoadClient>>,
}

impl<'a> PageOpenAdapter<'a> {
    pub(super) fn new(
        state: Rc<PageState>,
        resources: Rc<ResourceFetcher>,
        scripts: &'a mut Option<Box<PageScripts>>,
        script_client: Rc<RefCell<dyn ScriptLoadClient>>,
    ) -> Self {
        Self {
            state,
            resources,
            scripts,
            script_client,
        }
    }
}

impl OpenDocumentAdapter for PageOpenAdapter<'_> {
    fn CommitURL(&mut self, url: &str) {
        self.resources.SetDocumentURL(url.to_owned());
        if let Some(scripts) = self.scripts.as_ref() {
            scripts.SetURL(url.to_owned());
        }
        self.state.client.borrow_mut().DidCommit(url);
    }

    fn PrepareDocumentServices(&mut self) -> OpenDocumentServices {
        self.state.CancelImageEvents();
        self.state.measurement.borrow_mut().take();
        OpenDocumentServices {
            document: self.state.document.Handle(),
            resources: self.resources.clone(),
            scripts: self.scripts.as_deref().map(PageScripts::OpenServices),
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

    fn UpdateDocumentResources(&mut self, allowance: usize) -> io::Result<usize> {
        ResolvePageStylesAndLoadResources(&self.state, &self.resources, self.scripts, allowance)
    }

    fn HasPendingImageEvents(&self) -> bool {
        !self.state.pending_image_events.borrow().is_empty()
    }

    fn HasPendingLoadBlockingImages(&self) -> bool {
        self.resources.HasPendingLoadBlockingImages()
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
        self.state.client.borrow_mut().DidFinishLoad();
        self.resources.StartDeferredImages(4);
    }

    fn FailLoad(&mut self, message: &str) {
        self.state.client.borrow_mut().DidFail(message);
    }
}
