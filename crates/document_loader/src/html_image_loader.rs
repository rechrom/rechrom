// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Concrete ImageLoader fetch admission, sharing the ResourceFetcher's existing
//! transport/decode/pixel commit path. No URL-based element content lookup.
use super::*;
use dom::persistent_document::{
    DOMImageResourceContent, DOMInputType, HTMLImageLoaderTask, ImageLoaderTaskKind,
    ImageLoaderUpdate,
};
#[derive(Clone, PartialEq, Eq, Hash)]
struct ImageRequestKey {
    url: String,
    referrer: String,
    policy: Option<ReferrerPolicy>,
    cors: Option<(
        url_loader::FetchMode,
        url_loader::FetchCredentialsMode,
        url_loader::FetchOrigin,
    )>,
    headers: Vec<(String, String)>,
}
impl ImageRequestKey {
    fn FromRequest(request: &URLRequest) -> Self {
        Self {
            url: request.url.clone(),
            referrer: request.referrer.clone(),
            policy: request.referrer_policy,
            headers: request.headers.clone(),
            cors: request.fetch.as_ref().map(|fetch| {
                (
                    fetch.mode,
                    fetch.credentials,
                    fetch.requestor_origin.clone(),
                )
            }),
        }
    }
}
#[derive(Default)]
pub(super) struct HTMLImageFetchContext {
    contents: RefCell<HashMap<ImageRequestKey, (String, Rc<DOMImageResourceContent>)>>,
    pub(super) tasks: RefCell<Vec<HTMLImageLoaderTask>>,
    visible_contents: RefCell<Vec<Rc<DOMImageResourceContent>>>,
    pub(super) ready_events: RefCell<Vec<dom::persistent_document::HTMLImageLoaderEventTicket>>,
    pub(super) security:
        RefCell<Option<Rc<dyn Fn() -> Arc<fetch_policy::RawFetchSecurityContext>>>>,
    runner: RefCell<Option<Rc<dyn Fn(HTMLImageLoaderTask)>>>,
    lazy_monitoring: RefCell<Option<Rc<dyn Fn(u64, bool)>>>,
    platform_publisher: RefCell<Option<Rc<dyn Fn(ResourceMutation)>>>,
    platform_effects: RefCell<Vec<ResourceMutation>>,
    layout_update: RefCell<Option<Rc<dyn Fn(u64, image_resource::ImageResourceContentUpdate)>>>,
    finished: RefCell<Option<Rc<dyn Fn(Rc<dom::persistent_document::HTMLImageLoader>, bool)>>>,
}
impl ResourceFetcher {
    pub fn SetHTMLImageFetchSecurity(
        &self,
        security: Rc<dyn Fn() -> Arc<fetch_policy::RawFetchSecurityContext>>,
    ) {
        *self.html_images.security.borrow_mut() = Some(security);
    }
    /// LazyImageHelper::Start/StopMonitoring route to the Document's one
    /// LazyLoadMediaObserver, after releasing all document/content borrows.
    pub fn SetHTMLImageLazyMonitoring(&self, route: Rc<dyn Fn(u64, bool)>) {
        *self.html_images.lazy_monitoring.borrow_mut() = Some(route);
    }
    fn SetHTMLImageLazyMonitored(&self, node: u64, monitored: bool) {
        let route = self.html_images.lazy_monitoring.borrow().clone();
        if let Some(route) = route {
            route(node, monitored);
        }
    }
    pub fn SetHTMLImageLoaderTaskRunner(&self, runner: Rc<dyn Fn(HTMLImageLoaderTask)>) {
        *self.html_images.runner.borrow_mut() = Some(runner);
    }
    pub fn SetPlatformImageResourcePublisher(&self, publisher: Rc<dyn Fn(ResourceMutation)>) {
        *self.html_images.platform_publisher.borrow_mut() = Some(publisher);
    }
    pub fn TakePlatformImageResourceEffects(&self) -> Vec<ResourceMutation> {
        std::mem::take(&mut *self.html_images.platform_effects.borrow_mut())
    }
    pub fn EnsureBrokenImageResourcesPublished(&self) -> io::Result<()> {
        let effects = self.PrepareBrokenImageResources()?;
        let publisher = self.html_images.platform_publisher.borrow().clone();
        if let Some(publisher) = publisher {
            for effect in effects {
                publisher(effect);
            }
        } else {
            self.html_images
                .platform_effects
                .borrow_mut()
                .extend(effects);
        }
        Ok(())
    }
    /// ImageLoader::OnAttachLayoutTree reuses the same resource lease even
    /// when the request completed before the LayoutObject was constructed.
    pub fn HTMLImageLoaderAttached(&self, node: usize) {
        let loader = self
            .document
            .borrow()
            .GetDocument()
            .ExistingHTMLImageLoader(node);
        if let Some(loader) = loader {
            self.NotifyHTMLImageLoaderLayout(&loader);
        }
    }
    pub fn SetHTMLImageLoaderLayoutUpdate(
        &self,
        update: Rc<dyn Fn(u64, image_resource::ImageResourceContentUpdate)>,
    ) {
        *self.html_images.layout_update.borrow_mut() = Some(update);
    }
    fn NotifyHTMLImageLoaderLayout(&self, loader: &Rc<dom::persistent_document::HTMLImageLoader>) {
        let update = self.html_images.layout_update.borrow().clone();
        if let Some(update) = update {
            update(loader.Element().Id(), loader.LayoutUpdate());
        }
    }
    pub fn SetHTMLImageLoaderCompletion(
        &self,
        completion: Rc<dyn Fn(Rc<dom::persistent_document::HTMLImageLoader>, bool)>,
    ) {
        *self.html_images.finished.borrow_mut() = Some(completion);
    }
    fn ImageRequestKey(&self, request: &URLRequest) -> ImageRequestKey {
        let mut key = ImageRequestKey::FromRequest(request);
        if key.cors.is_none() {
            let owner = self.document.borrow();
            let origin = owner.GetDocument().SecurityContext().Origin();
            let origin = match origin.Tuple() {
                Some(t) => url_loader::FetchOrigin::Tuple {
                    scheme: t.scheme.clone(),
                    host: t.host.clone(),
                    port: t.port,
                },
                None => url_loader::FetchOrigin::Opaque(
                    origin.OpaqueNonce().expect("opaque origin nonce"),
                ),
            };
            key.cors = Some((
                url_loader::FetchMode::NoCors,
                url_loader::FetchCredentialsMode::Include,
                origin,
            ));
        }
        key
    }
    pub(super) fn ImageContentForRequest(
        &self,
        source: &str,
        request: &URLRequest,
    ) -> Rc<DOMImageResourceContent> {
        self.html_images
            .contents
            .borrow_mut()
            .entry(self.ImageRequestKey(request))
            .or_insert_with(|| {
                let content = DOMImageResourceContent::Pending();
                content.SetRequestURL(request.url.clone());
                content.SetCORSEnabled(
                    request
                        .fetch
                        .as_ref()
                        .is_some_and(|fetch| fetch.mode == url_loader::FetchMode::Cors),
                );
                (source.into(), content)
            })
            .1
            .clone()
    }
    fn HTMLImageContentsForSource(&self, source: &str) -> Vec<Rc<DOMImageResourceContent>> {
        self.html_images
            .contents
            .borrow()
            .values()
            .filter(|(key, _)| key == source)
            .map(|(_, content)| content.clone())
            .collect()
    }
    pub(super) fn CompleteHTMLImageContent(
        &self,
        content: &Rc<DOMImageResourceContent>,
        metadata: Option<dom::ImageResourceMetadata>,
        access_check_failed: bool,
    ) {
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .CompleteHTMLImageResource(content, metadata, access_check_failed);
        let completion = self.html_images.finished.borrow().clone();
        for loader in content.Observers() {
            self.NotifyHTMLImageLoaderLayout(&loader);
        }
        for loader in content.Observers() {
            self.DeliverHTMLImageLoaderCompletion(loader, false, completion.as_ref());
        }
    }
    fn DeliverHTMLImageLoaderCompletion(
        &self,
        loader: Rc<dom::persistent_document::HTMLImageLoader>,
        no_resource: bool,
        completion: Option<&Rc<dyn Fn(Rc<dom::persistent_document::HTMLImageLoader>, bool)>>,
    ) {
        if let Some(completion) = completion {
            completion(loader, no_resource);
        } else if let Some(ticket) = loader.EventTicket() {
            self.html_images.ready_events.borrow_mut().push(ticket);
        }
    }
    pub fn ImageReferrerPolicyChangeRequiresUpdate(
        &self,
        node: usize,
        previous: Option<&str>,
    ) -> bool {
        let owner = self.document.borrow();
        let d = owner.GetDocument();
        let current = d
            .Node(node)
            .FindAttribute("referrerpolicy")
            .and_then(|attribute| ReferrerPolicy::FromLegacyAttribute(&attribute.value));
        current != previous.and_then(ReferrerPolicy::FromLegacyAttribute)
    }
    pub fn UpdateHTMLImageLoader(
        &self,
        node: usize,
        update: ImageLoaderUpdate,
        force_blocking: bool,
    ) -> io::Result<()> {
        if update != ImageLoaderUpdate::Normal && update != ImageLoaderUpdate::FromMicrotask {
            self.UpdateResponsiveImageSelection(node);
        }
        let (loader, source, active) = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let n = d.Node(node);
            if !n.IsHTMLElement("img")
                && d.InputType(node) != Some(DOMInputType::kImage)
                && !d.IsVideoElement(node)
            {
                return Ok(());
            }
            let source = if d.IsVideoElement(node) {
                Some(d.VideoImageSourceURL(node))
            } else {
                d.ResponsiveImageSelectionFor(node)
                    .map(|selection| selection.source.clone())
                    .or_else(|| n.FindAttribute("src").map(|a| a.value.clone()))
            };
            (
                d.EnsureHTMLImageLoader(node),
                source,
                d.IsActive() && n.OwnerDocumentNode() == Some(d.RootHandle()),
            )
        };
        if update == ImageLoaderUpdate::Normal
            && loader.HasStarted()
            && loader.CurrentSource() == source
        {
            return Ok(());
        }
        let available = source.as_ref().and_then(|source| {
            let url = ResolveUrl(&self.BaseURL(), source).ok()?;
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let n = d.Node(node);
            let crossorigin = n
                .FindAttribute("crossorigin")
                .map(|a| a.value.eq_ignore_ascii_case("use-credentials"));
            let request = URLRequest {
                url: url.clone(),
                destination: RequestDestination::kImage,
                ..Default::default()
            };
            let key = self.ImageRequestKey(&request);
            let origin = key.cors.as_ref().map(|(_, _, origin)| origin);
            let available = self
                .html_images
                .contents
                .borrow()
                .iter()
                .find(|(key, (_, content))| {
                    key.url == url
                        && content.IsLoaded()
                        && !content.ErrorOccurred()
                        && key
                            .cors
                            .as_ref()
                            .is_some_and(|(_, credentials, request_origin)| {
                                (crossorigin.is_none() || Some(request_origin) == origin)
                                    && (*credentials
                                        != url_loader::FetchCredentialsMode::SameOrigin
                                        || crossorigin == Some(false))
                            })
                })
                .map(|(_, (_, content))| content.clone());
            available
        });
        let immediate = available.is_some();
        if update == ImageLoaderUpdate::IgnorePreviousError {
            let mut contents = self.html_images.contents.borrow_mut();
            contents.retain(|_, (old, content)| {
                Some(old.as_str()) != source.as_deref() || !content.ErrorOccurred()
            });
            if let Some(source) = &source {
                self.loaded_image_sources.borrow_mut().remove(source);
                self.document
                    .borrow_mut()
                    .GetDocumentMut()
                    .ResetFailedImageResource(source);
            }
        }
        let Some(task) =
            loader.UpdateFromElement(source, active, update, immediate, force_blocking)
        else {
            return Ok(());
        };
        self.document
            .borrow_mut()
            .GetDocumentMut()
            .ResetHTMLImageLayoutDisposition(node);
        let needs_broken = {
            let owner = self.document.borrow();
            owner.GetDocument().NeedsBrokenImageResources(node)
        };
        if needs_broken {
            self.EnsureBrokenImageResourcesPublished()?;
        }
        if task.synchronous {
            return self.RunHTMLImageLoaderTask(task);
        }
        let runner = self.html_images.runner.borrow().clone();
        if let Some(runner) = runner {
            runner(task);
        } else {
            self.html_images.tasks.borrow_mut().push(task);
        }
        Ok(())
    }
    pub(super) fn RunPendingHTMLImageLoaderTasks(&self) {
        let tasks = std::mem::take(&mut *self.html_images.tasks.borrow_mut());
        for task in tasks {
            if let Err(error) = self.RunHTMLImageLoaderTask(task) {
                std::panic::panic_any(error);
            }
        }
    }
    pub fn RunHTMLImageLoaderTask(&self, task: HTMLImageLoaderTask) -> io::Result<()> {
        let loader = task.loader.clone();
        let (active, node, source, policy, lazy, cross_origin, request_context, origin) = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            let Some(node) = d.FindNodeById(loader.Element().Id()) else {
                return Ok(());
            };
            if d.NodeHandle(node) != loader.Element() {
                return Ok(());
            }
            let n = d.Node(node);
            (
                d.IsActive() && n.OwnerDocumentNode() == Some(d.RootHandle()),
                node,
                loader.CurrentSource(),
                n.FindAttribute("referrerpolicy")
                    .and_then(|a| ReferrerPolicy::FromLegacyAttribute(&a.value)),
                n.IsHTMLElement("img")
                    && n.FindAttribute("loading")
                        .is_some_and(|a| a.value.eq_ignore_ascii_case("lazy")),
                n.FindAttribute("crossorigin")
                    .map(|a| a.value.eq_ignore_ascii_case("use-credentials")),
                if n.FindAttribute("srcset").is_some()
                    || n.Parent()
                        .is_some_and(|parent| d.Node(parent).IsHTMLElement("picture"))
                {
                    foundation::content_security_policy::CSPRequestContext::ImageSet
                } else {
                    foundation::content_security_policy::CSPRequestContext::Image
                },
                match d.SecurityContext().Origin().Tuple() {
                    Some(origin) => url_loader::FetchOrigin::Tuple {
                        scheme: origin.scheme.clone(),
                        host: origin.host.clone(),
                        port: origin.port,
                    },
                    None => url_loader::FetchOrigin::Opaque(
                        d.SecurityContext()
                            .Origin()
                            .OpaqueNonce()
                            .expect("opaque origin nonce"),
                    ),
                },
            )
        };
        if let ImageLoaderTaskKind::Decode(id) = task.kind {
            loader.ProcessDecodeTask(id, active);
            return Ok(());
        }
        if !loader.TaskIsCurrent(&task) {
            return Ok(());
        }
        if !active {
            loader.ClearImage();
            self.NotifyHTMLImageLoaderLayout(&loader);
            return Ok(());
        }

        if matches!(task.kind, ImageLoaderTaskKind::ResumeDeferred) {
            let content = loader.GetContent();
            let already_loaded = content.as_ref().is_some_and(|content| content.IsLoaded());
            loader.DoUpdateFromElement(&task, content, active);
            self.LoadDeferredHTMLImageWithBlocking(loader.Element().Id(), task.force_blocking);
            // A deferred element can share a resource already completed by
            // another element. The original update still completes this
            // loader and queues its own load/error event after resumption.
            if already_loaded {
                self.NotifyHTMLImageLoaderLayout(&loader);
                let completion = self.html_images.finished.borrow().clone();
                self.DeliverHTMLImageLoaderCompletion(loader, false, completion.as_ref());
            }
            return Ok(());
        }

        let source_present = source.is_some();
        let source = source.filter(|source| {
            !source
                .trim_matches(|c| matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' '))
                .is_empty()
        });
        let Some(source) = source else {
            loader.DoUpdateFromElement(&task, None, active);
            self.SetHTMLImageLazyMonitored(loader.Element().Id(), false);
            self.NotifyHTMLImageLoaderLayout(&loader);
            if source_present {
                loader.QueueError();
            }
            let completion = self.html_images.finished.borrow().clone();
            self.DeliverHTMLImageLoaderCompletion(loader, true, completion.as_ref());
            return Ok(());
        };
        let url = match ResolveUrl(
            &self.BaseURL(),
            source.trim_matches(|c| matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')),
        ) {
            Ok(url) => url,
            Err(error) => {
                loader.DoUpdateFromElement(&task, None, active);
                self.NotifyHTMLImageLoaderLayout(&loader);
                loader.QueueError();
                let completion = self.html_images.finished.borrow().clone();
                self.DeliverHTMLImageLoaderCompletion(loader, true, completion.as_ref());
                let _ = error;
                return Ok(());
            }
        };
        let mut request = URLRequest {
            url: url.clone(),
            referrer_policy: policy,
            destination: RequestDestination::kImage,
            ..Default::default()
        };
        let security = self.html_images.security.borrow().clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "image fetch requires the canonical frame security owner",
            )
        })?;
        if self
            .ConfigureImageNetworkRequest(
                &mut request,
                security(),
                if cross_origin.is_some() {
                    url_loader::FetchMode::Cors
                } else {
                    url_loader::FetchMode::NoCors
                },
                if cross_origin == Some(false) {
                    url_loader::FetchCredentialsMode::SameOrigin
                } else {
                    url_loader::FetchCredentialsMode::Include
                },
                request_context,
                origin,
            )
            .is_err()
        {
            let content = DOMImageResourceContent::Pending();
            loader.DoUpdateFromElement(&task, Some(content.clone()), active);
            self.CompleteHTMLImageContent(&content, None, true);
            return Ok(());
        }
        let key = self.ImageRequestKey(&request);
        if task.update == ImageLoaderUpdate::ForcedReload {
            self.html_images.contents.borrow_mut().remove(&key);
        }
        let content = self
            .html_images
            .contents
            .borrow()
            .get(&key)
            .map(|(_, content)| content.clone());
        let content = if let Some(content) = content {
            content
        } else {
            if lazy && !task.force_blocking && task.update != ImageLoaderUpdate::ForcedReload {
                let content = self.ImageContentForRequest(&source, &request);
                content.SetDeferred();
                loader.SetLazyDeferred();
                self.deferred_images.borrow_mut().push(DeferredImage {
                    source: source.clone(),
                    url,
                    referrer_policy: policy,
                    request: Some(request),
                });
                loader.DoUpdateFromElement(&task, Some(content), active);
                self.NotifyHTMLImageLoaderLayout(&loader);
                self.SetHTMLImageLazyMonitored(loader.Element().Id(), true);
                return Ok(());
            }
            let (resource, content) = self.StartImageRequestWithReloadAndPolicy(
                &source,
                request,
                task.update == ImageLoaderUpdate::ForcedReload,
                if !lazy || task.force_blocking {
                    resource::ResourceLoadEventPolicy::Regular
                } else {
                    resource::ResourceLoadEventPolicy::ForceNonBlockingImage
                },
            );
            if cross_origin.is_none() {
                self.loaded_image_sources
                    .borrow_mut()
                    .insert(source.clone());
            }
            self.pending_images.borrow_mut().push(PendingImage {
                source: source.clone(),
                url,
                resource,
                content: content.clone(),
                decode: None,
                is_multipart_response: None,
                is_cors_same_origin: None,
                content_dpr: None,
                blocks_load: !lazy || task.force_blocking,
            });
            content
        };
        if lazy && !content.IsLoaded() {
            loader.SetLazyDeferred();
        }
        let promote = !lazy && !content.IsLoading() && !content.IsLoaded();
        loader.DoUpdateFromElement(&task, Some(content.clone()), active);
        if promote {
            self.LoadDeferredHTMLImage(loader.Element().Id());
        }

        self.NotifyHTMLImageLoaderLayout(&loader);
        if content.IsLoaded() {
            let completion = self.html_images.finished.borrow().clone();
            self.DeliverHTMLImageLoaderCompletion(loader.clone(), false, completion.as_ref());
        }
        self.SetHTMLImageLazyMonitored(loader.Element().Id(), loader.IsLazyDeferred());
        let _ = node;
        Ok(())
    }
}

impl ResourceFetcher {
    /// LazyImageHelper's callback identifies the actual element/loader. A
    /// second lazy element sharing this resource remains independently deferred.
    /// HTMLImageElement's two source resume paths share the loader, original
    /// pending request and Agent task runner. No URL re-admission occurs.
    pub fn ResumeDeferredHTMLImage(
        &self,
        node_id: u64,
        force_blocking: bool,
        from_microtask: bool,
    ) -> io::Result<()> {
        let (loader, active) = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            (
                d.FindNodeById(node_id)
                    .and_then(|n| d.ExistingHTMLImageLoader(n)),
                d.IsActive(),
            )
        };
        let Some(loader) = loader else {
            return Ok(());
        };
        let Some(task) = loader.ResumeDeferredImage(active, force_blocking, from_microtask) else {
            return Ok(());
        };
        self.SetHTMLImageLazyMonitored(node_id, false);
        if task.synchronous {
            return self.RunHTMLImageLoaderTask(task);
        }
        let runner = self.html_images.runner.borrow().clone();
        if let Some(runner) = runner {
            runner(task);
        } else {
            self.html_images.tasks.borrow_mut().push(task);
        }
        Ok(())
    }
    pub fn LoadDeferredHTMLImage(&self, node_id: u64) -> (usize, usize) {
        self.LoadDeferredHTMLImageWithBlocking(node_id, false)
    }
    fn LoadDeferredHTMLImageWithBlocking(
        &self,
        node_id: u64,
        force_blocking: bool,
    ) -> (usize, usize) {
        let loader = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            d.FindNodeById(node_id)
                .and_then(|node| d.ExistingHTMLImageLoader(node))
        };
        let Some(loader) = loader else {
            return (0, 0);
        };
        let Some(content) = loader.GetContent() else {
            return (0, 0);
        };
        if !loader.IsLazyDeferred() && content.IsLoaded() {
            return (0, 0);
        }
        if loader.IsLazyDeferred() {
            loader.LoadDeferredImage();
        }
        let index = {
            let deferred = self.deferred_images.borrow();
            let contents = self.html_images.contents.borrow();
            deferred.iter().position(|image| {
                let request = image.request.clone().unwrap_or_else(|| URLRequest {
                    url: image.url.clone(),
                    referrer_policy: image.referrer_policy,
                    destination: RequestDestination::kImage,
                    ..Default::default()
                });
                contents
                    .get(&self.ImageRequestKey(&request))
                    .is_some_and(|(_, candidate)| Rc::ptr_eq(candidate, &content))
            })
        };
        let mut started = 0;
        if let Some(index) = index {
            let image = self.deferred_images.borrow_mut().remove(index);
            let request = image.request.unwrap_or_else(|| URLRequest {
                url: image.url.clone(),
                referrer_policy: image.referrer_policy,
                destination: RequestDestination::kImage,
                ..Default::default()
            });
            let (mut resource, content) = self.StartImageRequestWithReloadAndPolicy(
                &image.source,
                request,
                false,
                if force_blocking {
                    resource::ResourceLoadEventPolicy::Regular
                } else {
                    resource::ResourceLoadEventPolicy::ForceNonBlockingImage
                },
            );
            self.pending_images.borrow_mut().push(PendingImage {
                source: image.source,
                url: image.url,
                resource,
                content,
                decode: None,
                is_multipart_response: None,
                is_cors_same_origin: None,
                content_dpr: None,
                blocks_load: force_blocking,
            });
            started = 1;
        }
        if content.IsLoaded() {
            self.NotifyHTMLImageLoaderLayout(&loader);
            let completion = self.html_images.finished.borrow().clone();
            self.DeliverHTMLImageLoaderCompletion(loader, false, completion.as_ref());
        }
        (started, 0)
    }
    pub fn StartVisibleHTMLImages(&self, nodes: &[u64]) -> (usize, usize) {
        let visible = {
            let owner = self.document.borrow();
            let d = owner.GetDocument();
            nodes
                .iter()
                .filter_map(|node| {
                    d.FindNodeById(*node)
                        .and_then(|node| d.ExistingHTMLImageLoader(node))
                        .and_then(|loader| loader.GetContent())
                })
                .collect::<Vec<_>>()
        };
        let next = visible.iter().map(Rc::as_ptr).collect::<HashSet<_>>();
        let previous = self
            .html_images
            .visible_contents
            .replace(visible)
            .iter()
            .map(Rc::as_ptr)
            .collect::<HashSet<_>>();
        let started = nodes
            .iter()
            .map(|node| self.LoadDeferredHTMLImage(*node).0)
            .sum();
        let mut prioritized = 0;
        for image in self.pending_images.borrow_mut().iter_mut() {
            let identity = Rc::as_ptr(&image.content);
            if next.contains(&identity) {
                image
                    .resource
                    .SetPriority(url_loader::ResourceLoadPriority::High);
                prioritized += 1;
            } else if previous.contains(&identity) {
                image
                    .resource
                    .SetPriority(url_loader::ResourceLoadPriority::Normal);
            }
        }
        (started, prioritized)
    }
}

impl ResourceFetcher {
    /// Decode the original immutable Blink platform PNGs through the existing
    /// bitmap owner. Both typed content variants are installed before fallback
    /// layout; pixel publication is returned to Page as ordinary typed effects.
    pub fn PrepareBrokenImageResources(&self) -> io::Result<Vec<ResourceMutation>> {
        use image_resource::platform_resources::{
            BrokenImageResourceVariants, PlatformImageResource, PlatformImageVariant,
            PlatformResourceScale,
        };
        if self
            .document
            .borrow()
            .GetDocument()
            .HasBrokenImageResources()
        {
            return Ok(vec![]);
        }
        let mut variants = Vec::with_capacity(2);
        let mut effects = Vec::with_capacity(2);
        for scale in [
            PlatformResourceScale::Scale100,
            PlatformResourceScale::Scale200,
        ] {
            let asset = self.engine.LoadPlatformImage(PlatformImageVariant {
                resource: PlatformImageResource::BrokenImage,
                scale,
            })?;
            let image = &asset.image;
            let metadata = dom::ImageResourceMetadata {
                id: image.id,
                natural_width: image.width as f64,
                natural_height: image.height as f64,
                resolution_scale: 1.0,
                orientation: image.orientation,
                known_to_be_opaque: image
                    .BitmapPixels()
                    .is_some_and(|pixels| pixels.chunks_exact(4).all(|pixel| pixel[3] == 255)),
                is_cors_same_origin: true,
            };
            let content = DOMImageResourceContent::Pending();
            self.document
                .borrow_mut()
                .GetDocumentMut()
                .CompleteHTMLImageResource(&content, Some(metadata), false);
            content.SetIsBroken();
            variants.push(content as Rc<dyn image_resource::ImageResourceContentLease>);
            effects.push(ResourceMutation::PlatformImageResourceReady(image.clone()));
        }
        let high = variants.pop().unwrap();
        let low = variants.pop().unwrap();
        self.document
            .borrow()
            .GetDocument()
            .InstallBrokenImageResources(BrokenImageResourceVariants {
                scale100: low,
                scale200: high,
            });
        Ok(effects)
    }
}
