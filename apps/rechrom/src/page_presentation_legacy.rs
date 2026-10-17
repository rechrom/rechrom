//! Test-only coordinator used by existing compositor/display fixtures.
use super::*;
#[cfg(test)]
pub(crate) struct WindowCompositor {
    pages: BTreeMap<Document, PagePresentation>,
    scene: Option<Scene>,
    reply: mpsc::Sender<Command>,
    main_documents: BTreeMap<(u64, u64), Document>,
}
#[cfg(test)]
impl WindowCompositor {
    pub fn new(reply: mpsc::Sender<Command>) -> Self {
        Self {
            pages: BTreeMap::new(),
            scene: None,
            reply,
            main_documents: BTreeMap::new(),
        }
    }
    fn route(&mut self, document: Document, command: PageCommand) -> io::Result<Vec<Effect>> {
        let Some(page) = self.pages.get_mut(&document) else {
            return Ok(vec![]);
        };
        let effects = page.compositor.Handle(command)?;
        self.map_effects(document, effects)
    }
    fn map_effects(
        &mut self,
        document: Document,
        effects: Vec<PageEffect>,
    ) -> io::Result<Vec<Effect>> {
        let Some(page) = self.pages.get_mut(&document) else {
            return Ok(vec![]);
        };
        let mut pending: VecDeque<_> = effects.into();
        let mut output = vec![];
        while let Some(effect) = pending.pop_front() {
            match effect {
                PageEffect::Raster(job) => output.push(Effect::Raster {
                    sender: page.raster.clone(),
                    job,
                }),
                PageEffect::InstallRasterBundle(bundle) => {
                    if let Some(previous) = page.active.replace(bundle) {
                        pending.extend(page.compositor.Handle(PageCommand::SpareBundle(previous))?);
                    }
                }
                PageEffect::FrameProduced {
                    frame,
                    submitted,
                    scroll_active,
                } => {
                    let active = page.active.as_ref().ok_or_else(|| {
                        io::Error::other("Page frame has no raster resource owner")
                    })?;
                    let resources =
                        active
                            .raster
                            .ExportResources(submitted.content.iter().flat_map(|frame| {
                                frame.resource_list.iter().map(|resource| resource.tile_id)
                            }));
                    browser_tracing::instant(
                        "frame",
                        "PageCompositorFrameProduced",
                        &[
                            ("view_id", document.0 as f64),
                            ("document_generation", document.1 as f64),
                            (
                                "sequence",
                                submitted.provenance.content.map_or(0, |p| p.sequence) as f64,
                            ),
                        ],
                    );
                    output.push(Effect::FrameProduced {
                        frame,
                        submitted,
                        resources,
                        scroll_active,
                    });
                }
                PageEffect::Submit {
                    frame,
                    scroll_active,
                } => output.push(Effect::Submit {
                    frame,
                    scroll_active,
                }),
                PageEffect::MainThread(mut event) => {
                    if let MainThreadEvent::Wheel { event, .. } = &mut event {
                        if let Some(placement) = self.scene.as_ref().and_then(|scene| {
                            scene.placements.iter().find(|p| p.document == document)
                        }) {
                            let scale = self.scene.as_ref().unwrap().viewport.scale;
                            event.position.x += f64::from(placement.destination.x) / scale;
                            event.position.y += f64::from(placement.destination.y) / scale;
                        }
                    }
                    output.push(Effect::MainThread(event));
                }
                PageEffect::RequestBeginFrame => output.push(Effect::RequestBeginFrame),
                PageEffect::VisualDamage => output.push(Effect::VisualDamage),
            }
        }
        Ok(output)
    }
    pub fn Handle(&mut self, command: Command) -> io::Result<Vec<Effect>> {
        match command {
            Command::Snapshot(snapshot) => {
                let mut effects = Vec::new();
                let changed = self.scene.as_ref() != Some(&snapshot.scene);
                self.pages
                    .retain(|document, _| snapshot.scene.live_documents.contains(document));
                self.main_documents
                    .retain(|_, document| snapshot.scene.live_documents.contains(document));
                self.scene = Some(snapshot.scene.clone());
                if changed {
                    effects.extend([
                        Effect::Scene(snapshot.scene),
                        Effect::VisualDamage,
                        Effect::RequestBeginFrame,
                    ]);
                }
                for commit in snapshot.pages {
                    let document = commit.document;
                    if !self.pages.contains_key(&document) {
                        self.pages.insert(
                            document,
                            PagePresentation::new(document, {
                                let reply = self.reply.clone();
                                std::sync::Arc::new(move |ready| {
                                    let _ = reply.send(Command::RasterReady(document, ready));
                                })
                            })?,
                        );
                    }
                    effects.extend(self.route(document, PageCommand::Snapshot(commit))?);
                }
                Ok(effects)
            }
            Command::RasterReady(document, ready) => {
                if self.pages.contains_key(&document) {
                    self.route(document, PageCommand::RasterReady(ready.take()))
                } else {
                    Ok(vec![])
                } // envelope Drop returns to its own executor
            }
            Command::BeginFrame(frame) => {
                // The document host currently runs one lifecycle turn for its
                // Pages. Only its selected content compositor requests it; all
                // other compositors still own independent commit/raster state.
                let document = self
                    .scene
                    .as_ref()
                    .and_then(|scene| {
                        scene
                            .placements
                            .iter()
                            .find(|p| p.document.0 != 0 && p.document.0 != u64::MAX)
                            .or_else(|| scene.placements.first())
                    })
                    .map(|p| p.document);
                let document = document
                    .filter(|document| self.pages.contains_key(document))
                    .or_else(|| self.pages.contains_key(&(0, 0)).then_some((0, 0)));
                if let Some(document) = document {
                    let effects = self.route(document, PageCommand::BeginFrame(frame))?;
                    if effects.iter().any(|effect| {
                        matches!(
                            effect,
                            Effect::MainThread(MainThreadEvent::BeginFrame { .. })
                        )
                    }) {
                        self.main_documents
                            .insert((frame.source_id, frame.sequence_number), document);
                    }
                    Ok(effects)
                } else {
                    Ok(vec![Effect::MainThread(MainThreadEvent::BeginFrame {
                        frame,
                        document: None,
                        scroll: None,
                    })])
                }
            }
            Command::MainFrameComplete {
                source_id,
                sequence_number,
            } => {
                if let Some(document) = self.main_documents.remove(&(source_id, sequence_number)) {
                    self.route(
                        document,
                        PageCommand::MainFrameComplete {
                            source_id,
                            sequence_number,
                        },
                    )
                } else {
                    Ok(vec![])
                }
            }
            Command::Wheel {
                mut event,
                queued_at,
                samples,
            } => {
                let placement = self.scene.as_ref().and_then(|scene| {
                    let x = event.position.x * scene.viewport.scale;
                    let y = event.position.y * scene.viewport.scale;
                    scene
                        .placements
                        .iter()
                        .rev()
                        .find(|p| {
                            let r = p.destination;
                            x >= f64::from(r.x)
                                && y >= f64::from(r.y)
                                && x < f64::from(r.x) + f64::from(r.width)
                                && y < f64::from(r.y) + f64::from(r.height)
                        })
                        .cloned()
                });
                if let Some(placement) = placement.filter(|p| {
                    p.document.0 != 0
                        && p.document.0 != u64::MAX
                        && self.pages.contains_key(&p.document)
                }) {
                    let scale = self.scene.as_ref().unwrap().viewport.scale;
                    event.position.x -= f64::from(placement.destination.x) / scale;
                    event.position.y -= f64::from(placement.destination.y) / scale;
                    self.route(
                        placement.document,
                        PageCommand::Wheel {
                            event,
                            queued_at,
                            samples,
                        },
                    )
                } else {
                    Ok(vec![Effect::MainThread(MainThreadEvent::Wheel {
                        event,
                        queued_at,
                        samples,
                    })])
                }
            }
            Command::Redraw => {
                let mut effects = vec![Effect::RequestBeginFrame];
                if let Some(scene) = &self.scene {
                    // Redraw may need the retained Surface even while its new
                    // producer has not committed a frame yet.
                    effects.extend([Effect::Scene(scene.clone()), Effect::VisualDamage]);
                }
                Ok(effects)
            }
            Command::Resize(viewport) => {
                let Some(mut scene) = self.scene.clone() else {
                    return Ok(vec![]);
                };
                scene.viewport = viewport;
                scene.metadata.clear(); // regenerated by the chrome Page commit
                let toolbar_height = viewport.toolbar_pixels();
                let mut effects = vec![];
                for placement in &mut scene.placements {
                    // Window layout policy stays in the embedding. Only each
                    // visible Page's local viewport enters its compositor.
                    placement.destination = if placement.document.0 == 0 {
                        compositor::DeviceRect::new(0, 0, viewport.width, toolbar_height)
                    } else {
                        compositor::DeviceRect::new(
                            0,
                            toolbar_height as i32,
                            viewport.width,
                            viewport.height.saturating_sub(toolbar_height).max(1),
                        )
                    };
                    let local = Viewport {
                        width: placement.destination.width,
                        height: placement.destination.height,
                        scale: viewport.scale,
                    };
                    effects.extend(self.route(placement.document, PageCommand::Resize(local))?);
                }
                self.scene = Some(scene.clone());
                effects.extend([
                    Effect::Scene(scene),
                    Effect::VisualDamage,
                    Effect::RequestBeginFrame,
                ]);
                Ok(effects)
            }
            Command::InputBeginFrame(frame) => {
                browser_tracing::instant(
                    "input",
                    "InputBeginFrame",
                    &[
                        ("source_id", frame.source_id as f64),
                        ("sequence", frame.sequence_number as f64),
                    ],
                );
                Ok(vec![])
            }
            Command::DisplayBeginFrame(_) | Command::Stop => Ok(vec![]),
        }
    }

    pub fn Prepare(&mut self, frame: NativeBeginFrame) -> io::Result<Vec<Effect>> {
        let mut effects = vec![];
        let documents: Vec<_> = self.pages.keys().copied().collect();
        for document in documents {
            let page = self.pages.get_mut(&document).unwrap();
            if let Some(active) = &mut page.active {
                let ready = active.raster.PollCompletedRaster()?;
                active.tiles.ApplyRasterResults(&ready);
                if !ready.is_empty() {
                    let output = page
                        .compositor
                        .Handle(PageCommand::ActiveRasterReady(ready))?;
                    effects.extend(self.map_effects(document, output)?);
                }
            }
            let output = self
                .pages
                .get_mut(&document)
                .unwrap()
                .compositor
                .Prepare(frame)?;
            effects.extend(self.map_effects(document, output)?);
        }
        Ok(effects)
    }
    pub fn Draw(&mut self, frame: NativeBeginFrame) -> Vec<Effect> {
        let mut scroll_active = false;
        if let Some(scene) = &self.scene {
            for placement in &scene.placements {
                if let Some(page) = self.pages.get_mut(&placement.document) {
                    for effect in page.compositor.Draw(frame) {
                        if let PageEffect::Submit {
                            scroll_active: active,
                            ..
                        } = effect
                        {
                            scroll_active |= active;
                        }
                    }
                }
            }
        }
        vec![Effect::Submit {
            frame,
            scroll_active,
        }]
    }
}
