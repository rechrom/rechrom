//! Host-only font discovery protocol. FontEngine never invokes this service:
//! its compute methods return requests, and the embedding completes them.
use crate::resolver::FontFace;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FontContextVersion(pub u64);

/// Opaque provider identity, valid only within its context version.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SystemFontFaceId(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontPresentation {
    Text,
    Emoji,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SystemFontQuery {
    MatchFamily {
        family: String,
        weight: f64,
        italic: bool,
        locale: String,
    },
    MatchUniqueName {
        name: String,
    },
    MatchGeneric {
        generic: String,
        weight: f64,
        italic: bool,
        locale: String,
    },
    Fallback {
        text: Vec<u16>,
        weight: f64,
        italic: bool,
        locale: String,
        presentation: FontPresentation,
        base_face: Option<SystemFontFaceId>,
    },
    OpenFace {
        id: SystemFontFaceId,
    },
}

/// Metadata only: `face.bytes` must be empty until OpenFace is completed.
#[derive(Clone, Debug, PartialEq)]
pub struct SystemFontFace {
    pub id: SystemFontFaceId,
    pub face: FontFace,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemFontRequest {
    pub id: u64,
    pub context_version: FontContextVersion,
    pub query: SystemFontQuery,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SystemFontAnswer {
    Faces(Vec<SystemFontFace>),
    Face(FontFace),
    Unavailable,
    /// A transient host/I/O failure. Unlike an unavailable match, this must
    /// not poison the context's negative cache.
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemFontResponse {
    pub request: SystemFontRequest,
    pub answer: SystemFontAnswer,
}

/// Inject into the embedding, outside the pure compute engine. Implementations
/// may perform platform/file I/O here; callers feed responses via ApplyMutation.
pub trait SystemFontProvider {
    fn Resolve(&mut self, request: &SystemFontRequest) -> SystemFontAnswer;
    /// The embedding advances the provider and engine together. Stateless
    /// providers may keep the default implementation.
    fn SetContextVersion(&mut self, _version: FontContextVersion) {}
}

/// Per-layout-pass outbox. This is a pure cache-miss channel, never an OS
/// callback. The host drains it before publishing a provisional layout.
#[derive(Clone, Default)]
pub struct FontQueryCollector(std::sync::Arc<std::sync::Mutex<FontQueryState>>);
#[derive(Default)]
struct FontQueryState {
    revision: u64,
    queries: Vec<SystemFontQuery>,
    matches: Vec<(SystemFontQuery, Vec<SystemFontFaceId>)>,
    unavailable_faces: Vec<SystemFontFaceId>,
    fallback_failures: Vec<(SystemFontQuery, usize)>,
    faces: Vec<(
        crate::SharedFontBytes,
        u32,
        Vec<crate::FontVariation>,
        SystemFontFaceId,
    )>,
}
impl FontQueryCollector {
    pub fn Push(&self, query: SystemFontQuery) {
        let mut queries = self.0.lock().expect("font query outbox poisoned");
        if !queries.queries.contains(&query) {
            queries.queries.push(query);
        }
    }
    pub fn Drain(&self) -> Vec<SystemFontQuery> {
        std::mem::take(&mut self.0.lock().expect("font query outbox poisoned").queries)
    }
    pub fn Clear(&self) {
        let mut state = self.0.lock().expect("font query outbox poisoned");
        let revision = state.revision.wrapping_add(1);
        *state = FontQueryState {
            revision,
            ..Default::default()
        };
    }
    /// Changes only when matching/identity semantics change, never when the
    /// pure request outbox is pushed or drained.
    pub fn Revision(&self) -> u64 {
        self.0.lock().expect("font query outbox poisoned").revision
    }
    pub fn RegisterFace(&self, face: &FontFace, id: SystemFontFaceId) {
        let mut state = self.0.lock().expect("font query outbox poisoned");
        if !state.faces.iter().any(|(bytes, index, variations, _)| {
            *index == face.face_index
                && *variations == face.variations
                && crate::SharedFontBytes::ptr_eq(bytes, &face.bytes)
        }) {
            state.faces.push((
                face.bytes.clone(),
                face.face_index,
                face.variations.clone(),
                id,
            ));
            state.revision = state.revision.wrapping_add(1);
        }
    }
    pub(crate) fn RegisterMatches(&self, query: &SystemFontQuery, faces: &[SystemFontFace]) {
        if !matches!(
            query,
            SystemFontQuery::MatchFamily { .. }
                | SystemFontQuery::MatchGeneric { .. }
                | SystemFontQuery::Fallback { .. }
        ) || (faces.is_empty() && !matches!(query, SystemFontQuery::Fallback { .. }))
        {
            return;
        }
        let mut state = self.0.lock().expect("font query outbox poisoned");
        if !state
            .matches
            .iter()
            .any(|(registered, _)| registered == query)
        {
            state.matches.push((
                query.clone(),
                faces.iter().map(|face| face.id.clone()).collect(),
            ));
            if !faces.is_empty() {
                state.revision = state.revision.wrapping_add(1);
            }
        }
    }
    pub(crate) fn RegisterUnavailableFace(&self, id: &SystemFontFaceId) {
        let mut state = self.0.lock().expect("font query outbox poisoned");
        if !state.unavailable_faces.contains(id) {
            state.unavailable_faces.push(id.clone());
            state.revision = state.revision.wrapping_add(1);
        }
    }
    /// Only the opened prefix of the provider's ordered answer is eligible.
    /// A face admitted by another query must not jump ahead of an unopened
    /// candidate. Entry requests the first face; a shaping miss requests more.
    pub(crate) fn FallbackReadyLimit(&self, query: &SystemFontQuery, missing: bool) -> usize {
        let mut state = self.0.lock().expect("font query outbox poisoned");
        let mut ready = false;
        let (limit, needs_request) = match state.matches.iter().find(|(key, _)| key == query) {
            Some((_, matches)) => {
                let limit = matches
                    .iter()
                    .position(|id| {
                        if state.faces.iter().any(|(_, _, _, face)| face == id) {
                            ready = true;
                            false
                        } else {
                            !state.unavailable_faces.contains(id)
                        }
                    })
                    .unwrap_or(matches.len());
                (limit, limit < matches.len() && (missing || !ready))
            }
            None => (0, true),
        };
        if missing && limit != 0 {
            if let Some((_, previous)) = state
                .fallback_failures
                .iter_mut()
                .find(|(key, _)| key == query)
            {
                *previous = (*previous).max(limit);
            } else {
                state.fallback_failures.push((query.clone(), limit));
            }
        }
        if needs_request && !state.queries.contains(query) {
            state.queries.push(query.clone());
        }
        limit
    }
    pub(crate) fn FallbackFailedPrefix(&self, query: &SystemFontQuery) -> usize {
        self.0
            .lock()
            .expect("font query outbox poisoned")
            .fallback_failures
            .iter()
            .find(|(key, _)| key == query)
            .map_or(0, |(_, count)| *count)
    }
    /// Preserve the platform's exact family/style/locale answer after its
    /// immutable faces enter a shared catalog. Unrelated admitted aliases must
    /// not replace the face selected for this request.
    pub(crate) fn MatchRank(&self, query: &SystemFontQuery, face: &FontFace) -> Option<usize> {
        let state = self.0.lock().expect("font query outbox poisoned");
        let (_, _, _, id) = state.faces.iter().find(|(bytes, index, variations, _)| {
            *index == face.face_index
                && *variations == face.variations
                && crate::SharedFontBytes::ptr_eq(bytes, &face.bytes)
        })?;
        let (_, matches) = state
            .matches
            .iter()
            .find(|(registered, _)| registered == query)?;
        matches.iter().position(|candidate| candidate == id)
    }
    pub fn FaceId(&self, face: &FontFace) -> Option<SystemFontFaceId> {
        self.0
            .lock()
            .expect("font query outbox poisoned")
            .faces
            .iter()
            .find(|(bytes, index, variations, _)| {
                *index == face.face_index
                    && *variations == face.variations
                    && crate::SharedFontBytes::ptr_eq(bytes, &face.bytes)
            })
            .map(|(_, _, _, id)| id.clone())
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}
impl PartialEq for FontQueryCollector {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}
impl Eq for FontQueryCollector {}
impl std::fmt::Debug for FontQueryCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontQueryCollector").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_identity_distinguishes_bytes_collection_face_and_variations() {
        let collector = FontQueryCollector::default();
        let regular = FontFace {
            bytes: vec![1, 2, 3].into(),
            variations: vec![crate::FontVariation {
                tag: u32::from_be_bytes(*b"wght"),
                value: 400.,
            }],
            ..Default::default()
        };
        let mut bold = regular.clone();
        bold.variations[0].value = 700.;
        let mut collection = regular.clone();
        collection.face_index = 1;
        for (face, id) in [
            (&regular, "regular"),
            (&bold, "bold"),
            (&collection, "ttc-1"),
        ] {
            collector.RegisterFace(face, SystemFontFaceId(id.into()));
            assert_eq!(collector.FaceId(face), Some(SystemFontFaceId(id.into())));
        }
        assert_eq!(
            collector.FaceId(&regular),
            Some(SystemFontFaceId("regular".into()))
        );
        let mut alias = bold.clone();
        alias.family = "system-ui".into();
        assert_eq!(
            collector.FaceId(&alias),
            Some(SystemFontFaceId("bold".into()))
        );
        let mut copied = regular.clone();
        copied.bytes = regular.bytes.to_vec().into();
        assert_eq!(
            collector.FaceId(&copied),
            None,
            "equal bytes are not the same provider-owned instance"
        );
    }

    #[test]
    fn matching_revision_changes_without_new_font_bytes_but_not_for_repeated_queries() {
        let collector = FontQueryCollector::default();
        let face = FontFace {
            bytes: vec![1, 2, 3].into(),
            ..Default::default()
        };
        let id = SystemFontFaceId("shared-face".into());
        collector.RegisterFace(&face, id.clone());
        let before = collector.Revision();
        let query = SystemFontQuery::MatchGeneric {
            generic: "sans-serif".into(),
            weight: 400.,
            italic: false,
            locale: "ja".into(),
        };
        let descriptors = [SystemFontFace {
            id: id.clone(),
            face: FontFace::default(),
        }];
        collector.RegisterMatches(&query, &descriptors);
        assert!(
            collector.Revision() > before,
            "a new query association invalidates selection even for existing bytes"
        );
        let matched = collector.Revision();
        collector.RegisterMatches(&query, &descriptors);
        collector.RegisterFace(&face, id);
        collector.Push(query);
        collector.Drain();
        collector.RegisterMatches(
            &SystemFontQuery::MatchFamily {
                family: "not-installed".into(),
                weight: 400.,
                italic: false,
                locale: "ja".into(),
            },
            &[],
        );
        assert_eq!(
            collector.Revision(),
            matched,
            "stable cache queries do not keep relayout spinning"
        );
        collector.Clear();
        assert!(collector.Revision() > matched);
    }

    #[test]
    fn contextual_answers_change_revision_once_and_negative_queries_settle() {
        let collector = FontQueryCollector::default();
        let face = FontFace {
            bytes: vec![1, 2, 3].into(),
            ..Default::default()
        };
        let id = SystemFontFaceId("shared".into());
        collector.RegisterFace(&face, id.clone());
        let query = SystemFontQuery::Fallback {
            text: vec![0x6f22],
            weight: 400.,
            italic: false,
            locale: "ja".into(),
            presentation: FontPresentation::Text,
            base_face: None,
        };
        assert_eq!(collector.FallbackReadyLimit(&query, false), 0);
        assert_eq!(collector.Drain(), vec![query.clone()]);
        let before = collector.Revision();
        let descriptors = [SystemFontFace {
            id,
            face: FontFace::default(),
        }];
        collector.RegisterMatches(&query, &descriptors);
        assert_eq!(collector.Revision(), before.wrapping_add(1));
        assert_eq!(collector.FallbackReadyLimit(&query, false), 1);
        assert!(collector.Drain().is_empty());
        collector.RegisterMatches(&query, &descriptors);
        assert_eq!(collector.Revision(), before.wrapping_add(1));
        let mut negative = query.clone();
        if let SystemFontQuery::Fallback { locale, .. } = &mut negative {
            *locale = "zh".into();
        }
        assert_eq!(collector.FallbackReadyLimit(&negative, false), 0);
        assert_eq!(collector.Drain(), vec![negative.clone()]);
        collector.RegisterMatches(&negative, &[]);
        let settled = collector.Revision();
        assert_eq!(collector.FallbackReadyLimit(&negative, false), 0);
        assert_eq!(collector.FallbackReadyLimit(&negative, true), 0);
        assert!(collector.Drain().is_empty());
        assert_eq!(collector.Revision(), settled);
        assert_eq!(collector.MatchRank(&negative, &face), None);
        assert_eq!(collector.MatchRank(&query, &face), Some(0));
    }

    #[test]
    fn environment_change_clears_shared_queries_and_native_identities() {
        let collector = FontQueryCollector::default();
        let face = FontFace {
            bytes: vec![1, 2, 3].into(),
            ..Default::default()
        };
        collector.RegisterFace(&face, SystemFontFaceId("old-context".into()));
        collector.Push(SystemFontQuery::MatchUniqueName {
            name: "old-query".into(),
        });
        let mut engine = crate::FontEngine::new(FontContextVersion(7), None);
        engine.SetQueryCollector(Some(collector.clone()));
        engine
            .ApplyMutation(crate::FontMutation::EnvironmentChanged(FontContextVersion(
                8,
            )))
            .unwrap();
        assert!(collector.Drain().is_empty());
        assert_eq!(collector.FaceId(&face), None);
        collector.RegisterFace(&face, SystemFontFaceId("new-context".into()));
        assert_eq!(
            collector.FaceId(&face),
            Some(SystemFontFaceId("new-context".into()))
        );
    }
}
