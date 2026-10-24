//! Pure, retained font computation. Platform discovery and byte acquisition are
//! explicit effects, completed by the embedding through `ApplyMutation`.
use crate::resolver::{
    EqualFamilyName, FontFace, FontFaceCatalog, FontRequest, FontResolver, FontSmoothing,
};
use crate::system_fonts::*;
use crate::{
    Font, FontBackendFactory, FontDescription, FontMetrics, FontVariation, HarfBuzzShaper,
    RunSegmenterRange, ShapeOptions, ShapeResult, SimpleFontData,
};
use foundation::{Persistent, String as BlinkString, TextDirection as RunDirection};
use std::{
    cell::RefCell,
    ops::{Deref, Range},
    rc::Rc,
    sync::Arc,
};

#[derive(Clone, Debug, PartialEq)]
pub enum FontFaceState {
    Loading { resource_id: u64 },
    Ready(FontFace),
    Failed,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FontRule {
    pub id: u64,
    pub family: String,
    pub state: FontFaceState,
}

pub enum FontMutation {
    /// Decoded immutable faces supplied by the embedding, never encoded WOFF.
    SetFaces(Vec<FontFace>),
    SetRules(Vec<FontRule>),
    SetFaceState {
        rule_id: u64,
        state: FontFaceState,
    },
    SystemFontResponse(SystemFontResponse),
    EnvironmentChanged(FontContextVersion),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontError {
    StaleContext,
    InvalidRange,
    NoUsableFont,
    InvalidResponse,
    UnknownRule,
    ProviderFailure(String),
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontPending {
    pub system_queries: Vec<SystemFontRequest>,
    /// ResourceEngine owns downloads and DecodeEngine owns decode. These IDs
    /// identify CSS face loads already registered by the document embedding.
    pub resource_ids: Vec<u64>,
}
pub enum FontCompute<T> {
    Ready(T),
    Pending(FontPending),
    Error(FontError),
}

/// UTF-16 offsets use the complete text as shaping context. Inline owns bidi
/// and may supply analyzed script/orientation/presentation ranges; an empty
/// ranges slice asks FontEngine's existing RunSegmenter to analyze the range.
pub struct FontShapeRequest<'a> {
    pub text: &'a [u16],
    pub range: Range<u32>,
    pub description: &'a FontDescription,
    pub direction: RunDirection,
    pub segments: &'a [RunSegmenterRange],
    pub context_version: FontContextVersion,
}
#[derive(Clone)]
pub struct RetainedShapeResult {
    result: Persistent<ShapeResult>,
    pub context_version: FontContextVersion,
}
impl Deref for RetainedShapeResult {
    type Target = ShapeResult;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.result.Get() }
    }
}

pub struct FontEngine {
    context_version: FontContextVersion,
    faces: Vec<FontFace>,
    rules: Vec<FontRule>,
    answers: Vec<SystemFontResponse>,
    pending: Vec<SystemFontRequest>,
    next_request: u64,
    backend: Option<Arc<dyn FontBackendFactory>>,
    resolver: Option<FontResolver>,
    /// Canonical face ordering published to Layout/Paint/Raster. Rebuilding
    /// this catalog is tied to font mutations, never to an ordinary layout
    /// pass. Clones share the same immutable allocation.
    resolved_faces: Option<FontFaceCatalog>,
    discover_system: bool,
    queries: Option<FontQueryCollector>,
    /// cpp: FontCache / FontFallbackList. Style repeatedly requests metrics
    /// for the same immutable FontDescription while cascading a large
    /// inherited subtree. Cache the resolved primary metrics at the retained
    /// font-engine boundary; every font mutation below revokes these entries.
    metrics_cache: Vec<(FontDescription, FontMetrics)>,
}

/// Page-assembled handle to one retained font computation engine.
///
/// Style and Layout use this through their narrow adapter interfaces. The
/// handle carries no Document knowledge and is deliberately !Send: Chromium's
/// FontCache/HarfBuzz caches are likewise used on one renderer thread. Page's
/// lifecycle guarantees that Style and Layout scopes never overlap.
#[derive(Clone)]
pub struct FontEngineHandle {
    engine: Rc<RefCell<FontEngine>>,
}

impl FontEngineHandle {
    pub fn new(engine: FontEngine) -> Self {
        Self {
            engine: Rc::new(RefCell::new(engine)),
        }
    }

    pub fn with_mut<T>(&self, callback: impl FnOnce(&mut FontEngine) -> T) -> T {
        callback(&mut self.engine.borrow_mut())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.engine, &other.engine)
    }

    /// Layout's native resolver returns references retained by FontEngine, so
    /// its existing TLS scope cannot use a short `RefMut`. The caller must own
    /// the exclusive Style/Layout lifecycle scope for the returned reference.
    pub unsafe fn get_mut_for_exclusive_scope(&self) -> &mut FontEngine {
        unsafe { &mut *self.engine.as_ptr() }
    }
}
impl FontEngine {
    pub fn new(
        context_version: FontContextVersion,
        backend: Option<Arc<dyn FontBackendFactory>>,
    ) -> Self {
        Self {
            context_version,
            faces: vec![],
            rules: vec![],
            answers: vec![],
            pending: vec![],
            next_request: 1,
            backend,
            resolver: None,
            resolved_faces: None,
            discover_system: true,
            queries: Some(FontQueryCollector::default()),
            metrics_cache: Vec::new(),
        }
    }
    /// Compatibility for hosts supplying an already resolved, immutable face
    /// snapshot. Uses the same resolver as the request/resume entry points.
    pub fn FromFaces(faces: &[FontFace], backend: Option<Arc<dyn FontBackendFactory>>) -> Self {
        let mut engine = Self::new(FontContextVersion::default(), backend);
        engine.faces = faces.to_vec();
        engine.resolved_faces = Some(faces.to_vec().into());
        engine.discover_system = false;
        engine.queries = None;
        engine
    }
    pub fn SetQueryCollector(&mut self, collector: Option<FontQueryCollector>) {
        let changed = match (&self.queries, &collector) {
            (Some(current), Some(next)) => !current.ptr_eq(next),
            (None, None) => false,
            _ => true,
        };
        self.queries = collector.clone();
        // Building the canonical catalog also publishes the cached host
        // match/face associations into a newly installed collector. Preserve
        // that behavior without disturbing the steady-state snapshot when
        // Page supplies the same collector on every layout transaction.
        if changed {
            self.resolved_faces = None;
        }
        if let Some(resolver) = &mut self.resolver {
            resolver.SetQueryCollector(collector);
        }
    }
    /// Resolve an effect emitted by the synchronous layout adapter. This only
    /// consults versioned answers and produces additional host requests.
    pub fn PrepareSystemQuery(&mut self, query: SystemFontQuery) -> FontPending {
        let mut pending = FontPending::default();
        self.OpenCandidates(query, &mut pending);
        pending
    }
    pub fn ContextVersion(&self) -> FontContextVersion {
        self.context_version
    }
    pub fn ApplyMutation(&mut self, mutation: FontMutation) -> Result<(), FontError> {
        match mutation {
            FontMutation::SetFaces(faces) => {
                self.faces = faces;
                self.resolver = None;
                self.resolved_faces = None;
            }
            FontMutation::SetRules(rules) => {
                self.rules = rules;
                self.resolver = None;
                self.resolved_faces = None;
            }
            FontMutation::SetFaceState { rule_id, state } => {
                self.rules
                    .iter_mut()
                    .find(|r| r.id == rule_id)
                    .ok_or(FontError::UnknownRule)?
                    .state = state;
                self.resolver = None;
                self.resolved_faces = None;
            }
            FontMutation::EnvironmentChanged(version) => {
                self.context_version = version;
                self.answers.clear();
                self.pending.clear();
                if let Some(collector) = &self.queries {
                    collector.Clear();
                }
                self.resolver = None;
                self.resolved_faces = None;
            }
            FontMutation::SystemFontResponse(response) => {
                if response.request.context_version != self.context_version {
                    return Err(FontError::StaleContext);
                }
                let Some(index) = self.pending.iter().position(|r| *r == response.request) else {
                    return Err(FontError::InvalidResponse);
                };
                if let SystemFontAnswer::Failed(message) = &response.answer {
                    // Leave this exact pending request retryable. A transport
                    // failure is not evidence that a font does not exist.
                    return Err(FontError::ProviderFailure(message.clone()));
                }
                let valid =
                    matches!(
                        (&response.request.query, &response.answer),
                        (_, SystemFontAnswer::Unavailable)
                            | (SystemFontQuery::OpenFace { .. }, SystemFontAnswer::Face(_))
                    ) || (!matches!(response.request.query, SystemFontQuery::OpenFace { .. })
                        && matches!(response.answer, SystemFontAnswer::Faces(_)));
                if !valid
                    || match &response.answer {
                        SystemFontAnswer::Faces(faces) => {
                            faces.iter().any(|f| !f.face.bytes.is_empty())
                        }
                        SystemFontAnswer::Face(face) => face.bytes.is_empty(),
                        SystemFontAnswer::Unavailable => false,
                        SystemFontAnswer::Failed(_) => unreachable!("failure handled above"),
                    }
                {
                    return Err(FontError::InvalidResponse);
                }
                if let (
                    Some(collector),
                    SystemFontQuery::OpenFace { id },
                    SystemFontAnswer::Face(face),
                ) = (&self.queries, &response.request.query, &response.answer)
                {
                    collector.RegisterFace(face, id.clone());
                }
                if let Some(collector) = &self.queries {
                    match (&response.request.query, &response.answer) {
                        (query, SystemFontAnswer::Faces(faces)) => {
                            collector.RegisterMatches(query, faces)
                        }
                        (SystemFontQuery::OpenFace { id }, SystemFontAnswer::Unavailable) => {
                            collector.RegisterUnavailableFace(id)
                        }
                        (query, SystemFontAnswer::Unavailable) => {
                            collector.RegisterMatches(query, &[])
                        }
                        _ => {}
                    }
                }
                self.pending.remove(index);
                self.answers.push(response);
                self.resolver = None;
                self.resolved_faces = None;
            }
        }
        self.metrics_cache.clear();
        Ok(())
    }
    fn Query(
        &mut self,
        query: SystemFontQuery,
        pending: &mut FontPending,
    ) -> Option<SystemFontAnswer> {
        if let Some(answer) = self.answers.iter().find(|a| a.request.query == query) {
            return Some(answer.answer.clone());
        }
        let request = if let Some(request) = self.pending.iter().find(|r| r.query == query) {
            request.clone()
        } else {
            let request = SystemFontRequest {
                id: self.next_request,
                context_version: self.context_version,
                query,
            };
            self.next_request += 1;
            self.pending.push(request.clone());
            request
        };
        if !pending.system_queries.contains(&request) {
            pending.system_queries.push(request);
        }
        None
    }
    fn OpenCandidates(&mut self, query: SystemFontQuery, pending: &mut FontPending) {
        if let Some(SystemFontAnswer::Faces(candidates)) = self.Query(query.clone(), pending) {
            // Match results are ordered by the platform. Admit one exact face
            // at a time. Another fallback query is emitted only after the real
            // shaper still has a missing cluster with the admitted face.
            let fallback = matches!(query, SystemFontQuery::Fallback { .. });
            let failed_prefix = self
                .queries
                .as_ref()
                .map_or(0, |collector| collector.FallbackFailedPrefix(&query));
            for (index, candidate) in candidates.into_iter().enumerate() {
                let open = SystemFontQuery::OpenFace { id: candidate.id };
                match self.answers.iter().find(|a| a.request.query == open) {
                    Some(answer)
                        if (!fallback || index >= failed_prefix)
                            && matches!(answer.answer, SystemFontAnswer::Face(_)) =>
                    {
                        break
                    }
                    Some(_) => continue,
                    None => {
                        self.Query(open, pending);
                        break;
                    }
                }
            }
        }
    }
    fn Families(description: &FontDescription) -> Vec<(String, bool)> {
        let mut families = vec![];
        let mut current = description.Family() as *const crate::FontFamily;
        while !current.is_null() {
            let family = unsafe { &*current };
            if !family.FamilyName().empty() {
                families.push((family.FamilyName().Utf8(), family.FamilyIsGeneric()));
            }
            current = family.Next();
        }
        families
    }
    fn Prepare(
        &mut self,
        description: &FontDescription,
        version: FontContextVersion,
    ) -> Result<FontPending, FontError> {
        if version != self.context_version {
            return Err(FontError::StaleContext);
        }
        let families = Self::Families(description);
        let mut pending = FontPending::default();
        for rule in &self.rules {
            if families
                .iter()
                .any(|(name, _)| EqualFamilyName(name, &rule.family))
            {
                if let FontFaceState::Loading { resource_id } = rule.state {
                    pending.resource_ids.push(resource_id);
                }
            }
        }
        if self.discover_system {
            let locale = description.LocaleOrDefault().Ascii();
            let weight = description.Weight().ToFloat() as f64;
            let italic = description.Style().ToFloat() != 0.;
            for (family, generic) in &families {
                if self
                    .faces
                    .iter()
                    .any(|f| EqualFamilyName(&f.family, family))
                    || self.rules.iter().any(|r| {
                        EqualFamilyName(&r.family, family)
                            && matches!(r.state, FontFaceState::Ready(_))
                    })
                {
                    continue;
                }
                self.OpenCandidates(
                    if *generic {
                        SystemFontQuery::MatchGeneric {
                            generic: family.clone(),
                            weight,
                            italic,
                            locale: locale.clone(),
                        }
                    } else {
                        SystemFontQuery::MatchFamily {
                            family: family.clone(),
                            weight,
                            italic,
                            locale: locale.clone(),
                        }
                    },
                    &mut pending,
                );
            }
            // Last-resort primary metrics also require a concrete base face.
            if families.is_empty()
                || (pending.system_queries.is_empty() && !self.HasAvailableFaces())
            {
                self.OpenCandidates(
                    SystemFontQuery::MatchGeneric {
                        generic: "sans-serif".into(),
                        weight,
                        italic,
                        locale,
                    },
                    &mut pending,
                );
            }
        }
        Ok(pending)
    }
    pub fn ResolvedFaces(&mut self) -> FontFaceCatalog {
        if self.resolved_faces.is_none() {
            self.resolved_faces = Some(self.BuildAvailableFaces().into());
        }
        self.resolved_faces
            .as_ref()
            .expect("resolved face catalog")
            .clone()
    }
    fn HasAvailableFaces(&self) -> bool {
        if self.resolver.is_some()
            || !self.faces.is_empty()
            || self
                .rules
                .iter()
                .any(|rule| matches!(rule.state, FontFaceState::Ready(_)))
        {
            return true;
        }
        self.answers.iter().any(|response| {
            let SystemFontAnswer::Faces(descriptors) = &response.answer else {
                return false;
            };
            descriptors.iter().any(|descriptor| {
                self.answers.iter().any(|answer| {
                    matches!(
                        (&answer.request.query, &answer.answer),
                        (SystemFontQuery::OpenFace { id }, SystemFontAnswer::Face(_))
                            if *id == descriptor.id
                    )
                })
            })
        })
    }
    fn BuildAvailableFaces(&self) -> Vec<FontFace> {
        let mut faces = self.faces.clone();
        for rule in &self.rules {
            if let FontFaceState::Ready(face) = &rule.state {
                faces.push(face.clone());
            }
        }
        for response in &self.answers {
            if let Some(collector) = &self.queries {
                if let SystemFontAnswer::Unavailable = response.answer {
                    if let SystemFontQuery::OpenFace { id } = &response.request.query {
                        collector.RegisterUnavailableFace(id);
                    } else {
                        collector.RegisterMatches(&response.request.query, &[]);
                    }
                }
            }
            if let SystemFontAnswer::Faces(descriptors) = &response.answer {
                if let Some(collector) = &self.queries {
                    collector.RegisterMatches(&response.request.query, descriptors);
                }
                for descriptor in descriptors {
                    if let Some(opened) = self.answers.iter().find_map(|answer| {
                        match (&answer.request.query, &answer.answer) {
                            (SystemFontQuery::OpenFace { id }, SystemFontAnswer::Face(face))
                                if *id == descriptor.id =>
                            {
                                Some(face)
                            }
                            _ => None,
                        }
                    }) {
                        if let Some(collector) = &self.queries {
                            collector.RegisterFace(opened, descriptor.id.clone());
                        }
                        let mut face = descriptor.face.clone();
                        face.bytes = opened.bytes.clone();
                        face.face_index = opened.face_index;
                        face.variations = opened.variations.clone();
                        if !faces.contains(&face) {
                            faces.push(face);
                        }
                    }
                }
            }
        }
        faces
    }
    fn Resolver(&mut self) -> &mut FontResolver {
        if self.resolver.is_none() {
            let faces = self.ResolvedFaces();
            let mut resolver = FontResolver::new(&faces, self.backend.clone());
            resolver.SetQueryCollector(self.queries.clone());
            self.resolver = Some(resolver);
        }
        self.resolver.as_mut().unwrap()
    }
    pub fn ResolveFont(&mut self, request: &FontRequest<'_>) -> &mut Font {
        let _scope = foundation::LayoutHeapScope::new();
        self.Resolver().Resolve(request)
    }
    pub fn FaceIndex(&self, font: *const SimpleFontData) -> usize {
        self.resolver
            .as_ref()
            .expect("resolved font")
            .FaceIndex(font)
    }
    pub fn Variations(&self, font: *const SimpleFontData) -> &[FontVariation] {
        self.resolver
            .as_ref()
            .expect("resolved font")
            .Variations(font)
    }
    fn ResolveDescription(&mut self, description: &FontDescription) -> *const Font {
        let families: Vec<String> = Self::Families(description)
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        let language = description.LocaleOrDefault().Ascii();
        self.ResolveFont(&FontRequest {
            base_description: Some(description),
            size: description.ComputedSize() as f64,
            specified_size: description.SpecifiedSize() as f64,
            letter_spacing: description.LetterSpacing() as f64,
            word_spacing: description.WordSpacing() as f64,
            language: &language,
            families: &families,
            weight: description.Weight().ToFloat() as f64,
            italic: description.Style().ToFloat() != 0.,
            orientation: description.Orientation(),
            smoothing: FontSmoothing::kAuto,
        }) as *const Font
    }
    pub fn GetMetrics(
        &mut self,
        description: &FontDescription,
        version: FontContextVersion,
    ) -> FontCompute<FontMetrics> {
        if version != self.context_version {
            return FontCompute::Error(FontError::StaleContext);
        }
        if let Some((_, metrics)) = self
            .metrics_cache
            .iter()
            .find(|(cached, _)| cached == description)
        {
            return FontCompute::Ready(metrics.clone());
        }
        match self.Prepare(description, version) {
            Err(error) => return FontCompute::Error(error),
            Ok(pending)
                if !pending.system_queries.is_empty() || !pending.resource_ids.is_empty() =>
            {
                return FontCompute::Pending(pending)
            }
            _ => {}
        }
        if !self.HasAvailableFaces() {
            return FontCompute::Error(FontError::NoUsableFont);
        }
        let font = self.ResolveDescription(description);
        let primary = unsafe { &*font }.PrimaryFont();
        if primary.is_null() {
            return FontCompute::Error(FontError::NoUsableFont);
        }
        let metrics = unsafe { &*primary }.GetFontMetrics().clone();
        self.metrics_cache
            .push((description.clone(), metrics.clone()));
        FontCompute::Ready(metrics)
    }
    pub fn Shape(&mut self, request: &FontShapeRequest<'_>) -> FontCompute<RetainedShapeResult> {
        let _scope = foundation::LayoutHeapScope::new();
        if request.range.start > request.range.end
            || request.range.end as usize > request.text.len()
            || [request.range.start, request.range.end]
                .iter()
                .any(|&offset| {
                    offset > 0
                        && (offset as usize) < request.text.len()
                        && (0xdc00..=0xdfff).contains(&request.text[offset as usize])
                        && (0xd800..=0xdbff).contains(&request.text[offset as usize - 1])
                })
        {
            return FontCompute::Error(FontError::InvalidRange);
        }
        if !request.segments.is_empty() {
            let mut covered = request.range.start;
            let splits_pair = |offset: u32| {
                offset > 0
                    && (offset as usize) < request.text.len()
                    && (0xdc00..=0xdfff).contains(&request.text[offset as usize])
                    && (0xd800..=0xdbff).contains(&request.text[offset as usize - 1])
            };
            for segment in request.segments {
                if segment.start > segment.end
                    || segment.end as usize > request.text.len()
                    || splits_pair(segment.start)
                    || splits_pair(segment.end)
                {
                    return FontCompute::Error(FontError::InvalidRange);
                }
                let start = segment.start.max(request.range.start);
                let end = segment.end.min(request.range.end);
                if start >= end {
                    continue;
                }
                if start != covered {
                    return FontCompute::Error(FontError::InvalidRange);
                }
                covered = end;
            }
            if covered != request.range.end {
                return FontCompute::Error(FontError::InvalidRange);
            }
        }
        match self.Prepare(request.description, request.context_version) {
            Err(error) => return FontCompute::Error(error),
            Ok(pending)
                if !pending.system_queries.is_empty() || !pending.resource_ids.is_empty() =>
            {
                return FontCompute::Pending(pending)
            }
            _ => {}
        }
        if !self.HasAvailableFaces() {
            return FontCompute::Error(FontError::NoUsableFont);
        }
        if self.discover_system {
            if let Some(collector) = &self.queries {
                collector.Drain();
            }
        }
        let font = self.ResolveDescription(request.description);
        let shaper = HarfBuzzShaper::new(BlinkString::from_utf16(request.text));
        let result = if request.segments.is_empty() {
            shaper.ShapeRangeWithSegmentation(
                font,
                request.direction,
                request.range.start,
                request.range.end,
            )
        } else {
            shaper.ShapeWithRanges(
                font,
                request.direction,
                request.range.start,
                request.range.end,
                request.segments,
                ShapeOptions::default(),
            )
        };
        let retained = RetainedShapeResult {
            result: Persistent::from_ptr(result),
            context_version: self.context_version,
        };
        if self.discover_system {
            let mut pending = FontPending::default();
            // Use the same miss channel as production inline shaping. It
            // retains VS/presentation failures even when the final fallback
            // pass produces a nonzero base glyph, and applies the iterator's
            // PUA/noncharacter/default-ignorable suppression exactly once.
            let queries = self
                .queries
                .as_ref()
                .map(|collector| collector.Drain())
                .unwrap_or_default();
            for query in queries {
                self.OpenCandidates(query, &mut pending);
            }
            if !pending.system_queries.is_empty() {
                return FontCompute::Pending(pending);
            }
        }
        FontCompute::Ready(retained)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::AtomicString;
    fn description() -> FontDescription {
        let mut description = FontDescription::default();
        description.SetComputedSize(16.);
        description.SetSpecifiedSize(16.);
        description.SetAdjustedSize(16.);
        description.SetWeight(crate::FontSelectionValue::from_int(400));
        description.SetFamily(&crate::FontFamily::new(
            AtomicString::FromUtf8(b"test-face"),
            crate::FontFamilyType::kFamilyName,
            None,
        ));
        description
    }
    fn face() -> FontFace {
        FontFace {
            family: "test-face".into(),
            bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
                .as_slice()
                .into(),
            ..Default::default()
        }
    }
    fn pending<T>(result: FontCompute<T>) -> FontPending {
        match result {
            FontCompute::Pending(pending) => pending,
            _ => panic!("expected pending"),
        }
    }
    #[test]
    fn unicode_family_matching_is_shared_by_loading_and_discovery() {
        let version = FontContextVersion(0);
        let mut engine = FontEngine::new(version, None);
        let mut native = face();
        native.family = "Straße Σ".into();
        engine
            .ApplyMutation(FontMutation::SetFaces(vec![native]))
            .unwrap();
        let mut description = description();
        description.SetFamily(&crate::FontFamily::new(
            AtomicString::FromUtf8("STRASSE ς".as_bytes()),
            crate::FontFamilyType::kFamilyName,
            None,
        ));
        let ready = engine.Prepare(&description, version).unwrap();
        assert!(ready.system_queries.is_empty());
        assert!(ready.resource_ids.is_empty());
        engine
            .ApplyMutation(FontMutation::SetRules(vec![FontRule {
                id: 1,
                family: "Straße Σ".into(),
                state: FontFaceState::Loading { resource_id: 91 },
            }]))
            .unwrap();
        let waiting = engine.Prepare(&description, version).unwrap();
        assert_eq!(waiting.resource_ids, vec![91]);
        assert!(waiting.system_queries.is_empty());
    }

    #[test]
    fn repeated_metrics_reuse_the_retained_description_entry() {
        let version = FontContextVersion(0);
        let mut engine = FontEngine::FromFaces(&[face()], None);
        let description = description();
        assert!(matches!(
            engine.GetMetrics(&description, version),
            FontCompute::Ready(_)
        ));
        assert_eq!(engine.metrics_cache.len(), 1);
        assert!(matches!(
            engine.GetMetrics(&description, version),
            FontCompute::Ready(_)
        ));
        assert_eq!(engine.metrics_cache.len(), 1);

        engine
            .ApplyMutation(FontMutation::SetFaces(vec![face()]))
            .unwrap();
        assert!(engine.metrics_cache.is_empty());
    }

    fn cjk_face(family: &str) -> FontFace {
        let mut bytes = crate::minimal_font::Build(false, 0, 0x6f22, false, false);
        let parsed = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let cmap = parsed
            .raw_face()
            .table(ttf_parser::Tag::from_bytes(b"cmap"))
            .unwrap();
        let offset = cmap.as_ptr() as usize - bytes.as_ptr() as usize;
        let mut groups: Vec<_> = bytes[offset + 28..offset + cmap.len()]
            .chunks_exact(12)
            .map(|group| group.to_vec())
            .collect();
        groups.sort();
        for (index, group) in groups.iter().enumerate() {
            bytes[offset + 28 + index * 12..offset + 40 + index * 12].copy_from_slice(group);
        }
        FontFace {
            family: family.into(),
            bytes: bytes.into(),
            fallback_role: crate::FontFallbackRole::kText,
            ..Default::default()
        }
    }
    fn reply(engine: &mut FontEngine, request: SystemFontRequest, answer: SystemFontAnswer) {
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request,
                answer,
            }))
            .unwrap();
    }
    fn descriptor(id: &str, face: &FontFace) -> SystemFontFace {
        let mut metadata = face.clone();
        metadata.bytes = Default::default();
        SystemFontFace {
            id: SystemFontFaceId(id.into()),
            face: metadata,
        }
    }
    fn shape_request<'a>(
        description: &'a FontDescription,
        text: &'a [u16],
    ) -> FontShapeRequest<'a> {
        FontShapeRequest {
            text,
            range: 0..text.len() as u32,
            description,
            direction: RunDirection::kLtr,
            segments: &[],
            context_version: FontContextVersion(0),
        }
    }
    fn selected_id(engine: &mut FontEngine, request: &FontShapeRequest<'_>) -> SystemFontFaceId {
        let FontCompute::Ready(result) = engine.Shape(request) else {
            panic!("expected ready shape")
        };
        assert_eq!(result.NumGlyphs(), 1);
        let font = unsafe { &*result.runs_[0].Get() }.font_data_.Get();
        let face = &engine.ResolvedFaces()[engine.FaceIndex(font)];
        engine.queries.as_ref().unwrap().FaceId(face).unwrap()
    }

    #[test]
    fn contextual_fallback_does_not_reuse_another_locales_nominal_coverage() {
        let mut engine = FontEngine::new(FontContextVersion(0), None);
        engine
            .ApplyMutation(FontMutation::SetFaces(vec![face()]))
            .unwrap();
        let japanese = cjk_face("japanese");
        let chinese = cjk_face("chinese");
        let descriptors = vec![
            descriptor("ja-face", &japanese),
            descriptor("zh-face", &chinese),
        ];
        let text = [0x6f22];
        let mut description = description();
        description.SetLocale(crate::LayoutLocale::GetShared(&AtomicString::FromUtf8(
            b"ja",
        )));
        let request = shape_request(&description, &text);
        let query = pending(engine.Shape(&request)).system_queries.remove(0);
        assert!(
            matches!(&query.query, SystemFontQuery::Fallback { text, locale, weight, italic, presentation: FontPresentation::Text, base_face: None } if text == &[0x6f22] && locale == "ja" && *weight == 400. && !italic)
        );
        reply(
            &mut engine,
            query,
            SystemFontAnswer::Faces(descriptors.clone()),
        );
        let open = pending(engine.Shape(&request)).system_queries.remove(0);
        assert!(matches!(&open.query, SystemFontQuery::OpenFace { id } if id.0 == "ja-face"));
        reply(&mut engine, open, SystemFontAnswer::Face(japanese));
        assert_eq!(selected_id(&mut engine, &request).0, "ja-face");
        assert_eq!(selected_id(&mut engine, &request).0, "ja-face");
        assert!(
            engine.pending.is_empty(),
            "success must not open the remaining candidate"
        );

        description.SetLocale(crate::LayoutLocale::GetShared(&AtomicString::FromUtf8(
            b"ja-JP",
        )));
        let request = shape_request(&description, &text);
        let query = pending(engine.Shape(&request)).system_queries.remove(0);
        reply(
            &mut engine,
            query,
            SystemFontAnswer::Faces(descriptors.clone()),
        );
        assert_eq!(selected_id(&mut engine, &request).0, "ja-face");
        assert!(engine.pending.is_empty(), "new query reusing the first opened candidate must not open the next face before shaping");

        description.SetLocale(crate::LayoutLocale::GetShared(&AtomicString::FromUtf8(
            b"zh-CN",
        )));
        let request = shape_request(&description, &text);
        let query = pending(engine.Shape(&request)).system_queries.remove(0);
        assert!(
            matches!(&query.query, SystemFontQuery::Fallback { locale, text, .. } if locale == "zh-CN" && text == &[0x6f22])
        );
        let reversed = vec![descriptors[1].clone(), descriptors[0].clone()];
        reply(
            &mut engine,
            query,
            SystemFontAnswer::Faces(reversed.clone()),
        );
        let open = pending(engine.Shape(&request)).system_queries.remove(0);
        assert!(
            matches!(&open.query, SystemFontQuery::OpenFace { id } if id.0 == "zh-face"),
            "already opened Japanese face must not skip the provider's first Chinese candidate"
        );
        reply(&mut engine, open, SystemFontAnswer::Face(chinese));
        assert_eq!(selected_id(&mut engine, &request).0, "zh-face");
        assert_eq!(selected_id(&mut engine, &request).0, "zh-face");
        assert!(engine.pending.is_empty());

        description.SetLocale(crate::LayoutLocale::GetShared(&AtomicString::FromUtf8(
            b"ko",
        )));
        let request = shape_request(&description, &text);
        let query = pending(engine.Shape(&request)).system_queries.remove(0);
        reply(&mut engine, query, SystemFontAnswer::Faces(reversed));
        assert_eq!(
            selected_id(&mut engine, &request).0,
            "zh-face",
            "provider order wins over shared catalog insertion order even when both faces are open"
        );
        assert!(engine.pending.is_empty());
    }

    #[test]
    fn successful_explicit_family_does_not_request_system_fallback() {
        let mut engine = FontEngine::new(FontContextVersion(0), None);
        engine
            .ApplyMutation(FontMutation::SetFaces(vec![face()]))
            .unwrap();
        let description = description();
        let text: Vec<_> = "office".encode_utf16().collect();
        assert!(matches!(
            engine.Shape(&shape_request(&description, &text)),
            FontCompute::Ready(_)
        ));
        assert!(engine.pending.is_empty());
        assert!(engine.queries.as_ref().unwrap().Drain().is_empty());
    }

    #[test]
    fn fallback_only_opens_next_candidate_after_actual_shaping_failure() {
        let mut engine = FontEngine::new(FontContextVersion(0), None);
        engine
            .ApplyMutation(FontMutation::SetFaces(vec![face()]))
            .unwrap();
        let description = description();
        let text = [0x6f22];
        let request = shape_request(&description, &text);
        let mut wrong = face();
        wrong.family = "wrong-coverage".into();
        wrong.fallback_role = crate::FontFallbackRole::kText;
        let correct = cjk_face("correct");
        let query = pending(engine.Shape(&request)).system_queries.remove(0);
        reply(
            &mut engine,
            query,
            SystemFontAnswer::Faces(vec![
                descriptor("wrong", &wrong),
                descriptor("correct", &correct),
            ]),
        );
        let open = pending(engine.Shape(&request)).system_queries.remove(0);
        assert!(matches!(&open.query, SystemFontQuery::OpenFace { id } if id.0 == "wrong"));
        reply(&mut engine, open, SystemFontAnswer::Face(wrong));
        let open = pending(engine.Shape(&request)).system_queries.remove(0);
        assert!(matches!(&open.query, SystemFontQuery::OpenFace { id } if id.0 == "correct"));
        reply(&mut engine, open, SystemFontAnswer::Face(correct));
        assert_eq!(selected_id(&mut engine, &request).0, "correct");
        assert!(engine.pending.is_empty());
    }

    #[test]
    fn discovery_is_two_stage_pure_and_resumes_real_shaping() {
        let mut engine = FontEngine::new(FontContextVersion(7), None);
        let description = description();
        let text: Vec<u16> = "office".encode_utf16().collect();
        let request = FontShapeRequest {
            text: &text,
            range: 0..6,
            description: &description,
            direction: RunDirection::kLtr,
            segments: &[],
            context_version: FontContextVersion(7),
        };
        let first = pending(engine.Shape(&request));
        assert_eq!(first.system_queries.len(), 1);
        assert_eq!(
            pending(engine.Shape(&request)),
            first,
            "retry deduplicates cache misses"
        );
        let query = first.system_queries[0].clone();
        let mut descriptor = face();
        descriptor.bytes = Default::default();
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request: query,
                answer: SystemFontAnswer::Faces(vec![SystemFontFace {
                    id: SystemFontFaceId("exact-file#0".into()),
                    face: descriptor,
                }]),
            }))
            .unwrap();
        let second = pending(engine.Shape(&request));
        assert!(matches!(
            second.system_queries[0].query,
            SystemFontQuery::OpenFace { .. }
        ));
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request: second.system_queries[0].clone(),
                answer: SystemFontAnswer::Face(face()),
            }))
            .unwrap();
        let retained = match engine.Shape(&request) {
            FontCompute::Ready(shape) => shape,
            _ => panic!("expected shaped font"),
        };
        assert_eq!(retained.NumCharacters(), 6);
        assert!(retained.NumGlyphs() > 0);
        assert!(matches!(
            engine.GetMetrics(&description, FontContextVersion(7)),
            FontCompute::Ready(_)
        ));
        engine
            .ApplyMutation(FontMutation::EnvironmentChanged(FontContextVersion(8)))
            .unwrap();
        assert_eq!(
            retained.NumCharacters(),
            6,
            "retained result outlives resolver invalidation"
        );
        assert!(matches!(
            engine.Shape(&request),
            FontCompute::Error(FontError::StaleContext)
        ));
    }
    #[test]
    fn stale_and_unsolicited_answers_cannot_poison_context() {
        let mut engine = FontEngine::new(FontContextVersion(1), None);
        let description = description();
        let request = pending(engine.GetMetrics(&description, FontContextVersion(1)))
            .system_queries
            .remove(0);
        engine
            .ApplyMutation(FontMutation::EnvironmentChanged(FontContextVersion(2)))
            .unwrap();
        assert_eq!(
            engine.ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request,
                answer: SystemFontAnswer::Unavailable
            })),
            Err(FontError::StaleContext)
        );
        let unknown = SystemFontRequest {
            id: 99,
            context_version: FontContextVersion(2),
            query: SystemFontQuery::MatchUniqueName {
                name: "unknown".into(),
            },
        };
        assert_eq!(
            engine.ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request: unknown,
                answer: SystemFontAnswer::Unavailable
            })),
            Err(FontError::InvalidResponse)
        );
    }
    #[test]
    fn analyzed_segments_must_cover_the_requested_utf16_range_once() {
        let mut engine = FontEngine::FromFaces(&[face()], None);
        let description = description();
        let text: Vec<_> = "office".encode_utf16().collect();
        let segment = |start, end| RunSegmenterRange {
            start,
            end,
            ..Default::default()
        };
        for segments in [
            vec![segment(0, 2), segment(3, 6)],
            vec![segment(0, 4), segment(3, 6)],
            vec![segment(0, 7)],
            vec![segment(3, 2)],
        ] {
            let mut request = shape_request(&description, &text);
            request.segments = &segments;
            assert!(matches!(
                engine.Shape(&request),
                FontCompute::Error(FontError::InvalidRange)
            ));
        }
        let segments = [segment(0, 3), segment(3, 6)];
        let mut request = shape_request(&description, &text);
        request.range = 1..5;
        request.segments = &segments;
        let FontCompute::Ready(result) = engine.Shape(&request) else {
            panic!("valid clipped coverage")
        };
        assert_eq!(result.NumCharacters(), 4);
        let text: Vec<_> = "😀".encode_utf16().collect();
        let segments = [segment(0, 1), segment(1, 2)];
        let mut request = shape_request(&description, &text);
        request.segments = &segments;
        assert!(matches!(
            engine.Shape(&request),
            FontCompute::Error(FontError::InvalidRange)
        ));
    }

    #[test]
    fn loaded_rule_and_metrics_share_resolver_and_utf16_bounds_are_checked() {
        let mut engine = FontEngine::FromFaces(&[face()], None);
        let description = description();
        engine
            .ApplyMutation(FontMutation::SetRules(vec![FontRule {
                id: 4,
                family: "test-face".into(),
                state: FontFaceState::Loading { resource_id: 9 },
            }]))
            .unwrap();
        assert_eq!(
            pending(engine.GetMetrics(&description, FontContextVersion(0))).resource_ids,
            vec![9]
        );
        engine
            .ApplyMutation(FontMutation::SetFaceState {
                rule_id: 4,
                state: FontFaceState::Ready(face()),
            })
            .unwrap();
        assert!(matches!(
            engine.GetMetrics(&description, FontContextVersion(0)),
            FontCompute::Ready(_)
        ));
        let text: Vec<u16> = "😀".encode_utf16().collect();
        let request = FontShapeRequest {
            text: &text,
            range: 1..2,
            description: &description,
            direction: RunDirection::kLtr,
            segments: &[],
            context_version: FontContextVersion(0),
        };
        assert!(matches!(
            engine.Shape(&request),
            FontCompute::Error(FontError::InvalidRange)
        ));
    }
    #[test]
    fn facade_uses_same_variation_miss_channel_as_inline() {
        let mut engine = FontEngine::new(FontContextVersion(0), None);
        engine
            .ApplyMutation(FontMutation::SetFaces(vec![face()]))
            .unwrap();
        let description = description();
        let text = vec![0x30, 0xfe00];
        let request = FontShapeRequest {
            text: &text,
            range: 0..2,
            description: &description,
            direction: RunDirection::kLtr,
            segments: &[],
            context_version: FontContextVersion(0),
        };
        let pending = pending(engine.Shape(&request));
        assert!(pending.system_queries.iter().any(|q| matches!(&q.query, SystemFontQuery::Fallback { text, .. } if text == &[0x30, 0xfe00])));
        for request in pending.system_queries {
            engine
                .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                    request,
                    answer: SystemFontAnswer::Unavailable,
                }))
                .unwrap();
        }
        assert!(
            matches!(engine.Shape(&request), FontCompute::Ready(_)),
            "a terminal negative answer permits base-glyph second pass"
        );
    }
    #[test]
    fn cached_match_associations_survive_collector_replacement() {
        let mut engine = FontEngine::new(FontContextVersion(0), None);
        let query = SystemFontQuery::MatchGeneric {
            generic: "sans-serif".into(),
            weight: 400.,
            italic: false,
            locale: "ja".into(),
        };
        let id = SystemFontFaceId("japanese-native-face".into());
        let request = engine
            .PrepareSystemQuery(query.clone())
            .system_queries
            .remove(0);
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request,
                answer: SystemFontAnswer::Faces(vec![SystemFontFace {
                    id: id.clone(),
                    face: FontFace {
                        family: "sans-serif".into(),
                        ..Default::default()
                    },
                }]),
            }))
            .unwrap();
        let request = engine
            .PrepareSystemQuery(query.clone())
            .system_queries
            .remove(0);
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request,
                answer: SystemFontAnswer::Face(face()),
            }))
            .unwrap();
        let replacement = FontQueryCollector::default();
        engine.SetQueryCollector(Some(replacement.clone()));
        let resolved = engine.ResolvedFaces();
        assert_eq!(resolved[0].family, "sans-serif");
        assert_eq!(replacement.FaceId(&resolved[0]), Some(id));
        assert_eq!(replacement.MatchRank(&query, &resolved[0]), Some(0));
    }

    #[test]
    fn canonical_face_catalog_is_retained_until_a_font_mutation() {
        let mut engine = FontEngine::FromFaces(&[face()], None);
        let first = engine.ResolvedFaces();
        let second = engine.ResolvedFaces();
        assert!(first.ptr_eq(&second));

        let mut replacement = face();
        replacement.family = "replacement".into();
        engine
            .ApplyMutation(FontMutation::SetFaces(vec![replacement]))
            .unwrap();
        let third = engine.ResolvedFaces();
        assert!(!first.ptr_eq(&third));
        assert_eq!(third[0].family, "replacement");
    }

    #[test]
    fn transient_provider_failure_keeps_exact_request_retryable() {
        let mut engine = FontEngine::new(FontContextVersion(5), None);
        let query = SystemFontQuery::MatchUniqueName {
            name: "retry-face".into(),
        };
        let request = engine
            .PrepareSystemQuery(query.clone())
            .system_queries
            .remove(0);
        assert_eq!(
            engine.ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request: request.clone(),
                answer: SystemFontAnswer::Failed("temporary read failure".into()),
            })),
            Err(FontError::ProviderFailure("temporary read failure".into()))
        );
        assert_eq!(
            engine.PrepareSystemQuery(query.clone()).system_queries,
            vec![request.clone()]
        );
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request,
                answer: SystemFontAnswer::Faces(vec![]),
            }))
            .unwrap();
        assert!(engine.PrepareSystemQuery(query).system_queries.is_empty());
    }

    #[test]
    fn unavailable_open_advances_to_next_metadata_candidate() {
        let mut engine = FontEngine::new(FontContextVersion(0), None);
        let query = SystemFontQuery::MatchUniqueName {
            name: "local-face".into(),
        };
        let request = engine
            .PrepareSystemQuery(query.clone())
            .system_queries
            .remove(0);
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request,
                answer: SystemFontAnswer::Faces(vec![
                    SystemFontFace {
                        id: SystemFontFaceId("gone".into()),
                        face: FontFace::default(),
                    },
                    SystemFontFace {
                        id: SystemFontFaceId("present".into()),
                        face: FontFace::default(),
                    },
                ]),
            }))
            .unwrap();
        let first = engine
            .PrepareSystemQuery(query.clone())
            .system_queries
            .remove(0);
        assert!(matches!(&first.query, SystemFontQuery::OpenFace { id } if id.0 == "gone"));
        engine
            .ApplyMutation(FontMutation::SystemFontResponse(SystemFontResponse {
                request: first,
                answer: SystemFontAnswer::Unavailable,
            }))
            .unwrap();
        let second = engine.PrepareSystemQuery(query).system_queries.remove(0);
        assert!(matches!(&second.query, SystemFontQuery::OpenFace { id } if id.0 == "present"));
    }
}
