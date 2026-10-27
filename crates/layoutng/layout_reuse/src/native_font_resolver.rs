//! Native font resolver construction owned by `layout_reuse`.
//!
//! Deferred, shared-engine, and newly-created engine paths all live beside
//! the reusable environment which controls their lifetime.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:541-559
struct DeferredFontResolver {
    font_: Persistent<Font>,
}

impl DeferredFontResolver {
    fn new() -> Self {
        Self {
            font_: Persistent::from_ptr(MakeGarbageCollected(Font::default())),
        }
    }
}

impl NativeFontResolver for DeferredFontResolver {
    fn Resolve(&mut self, _request: &NativeFontRequest<'_>) -> &mut Font {
        unsafe { &mut *self.font_.Get() }
    }

    fn FaceIndex(&self, _font_data: *const SimpleFontData) -> usize {
        unreachable!("DeferredFontResolver cannot select a face")
    }
}

#[inline(never)]
pub(super) fn EngineFontResolver(space: &ConstraintSpace) -> Box<dyn NativeFontResolver> {
    if let Some(engine) = &space.font_engine {
        return Box::new(SharedEngineFontResolver(engine.clone()));
    }
    let mut engine =
        font_engine::FontEngine::FromFaces(&space.fonts, space.font_backend_factory.clone());
    engine.SetQueryCollector(space.font_queries.clone());
    Box::new(engine)
}

struct SharedEngineFontResolver(font_engine::FontEngineHandle);

impl NativeFontResolver for SharedEngineFontResolver {
    #[inline(never)]
    fn Resolve(&mut self, request: &NativeFontRequest<'_>) -> &mut Font {
        let engine = unsafe { self.0.get_mut_for_exclusive_scope() };
        font_engine::FontEngine::ResolveFont(
            engine,
            &font_engine::resolver::FontRequest {
                base_description: request.base_description,
                size: request.size,
                specified_size: request.specified_size,
                letter_spacing: request.letter_spacing,
                word_spacing: request.word_spacing,
                language: request.language,
                families: request.families,
                weight: request.weight,
                italic: request.italic,
                orientation: request.orientation,
                smoothing: request.smoothing,
            },
        )
    }

    fn FaceIndex(&self, font_data: *const SimpleFontData) -> usize {
        let engine = unsafe { self.0.get_mut_for_exclusive_scope() };
        engine.FaceIndex(font_data)
    }

    fn Variations(&self, font_data: *const SimpleFontData) -> &[FontVariation] {
        let engine = unsafe { self.0.get_mut_for_exclusive_scope() };
        engine.Variations(font_data)
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:560-565
#[inline(never)]
pub(crate) fn ConstructionFontResolver(space: &ConstraintSpace) -> Box<dyn NativeFontResolver> {
    if space.fonts.is_empty() && space.font_engine.is_none() {
        return Box::new(DeferredFontResolver::new());
    }
    EngineFontResolver(space)
}
