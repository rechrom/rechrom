//! Runtime lifetime for one native layout pass environment.
//!
//! The concrete resolvers retain their reuse/algorithm owners. This module
//! owns their ordered installation and teardown. These are explicit shared
//! sampling boundaries because no lifecycle or algorithm package owns the
//! whole cross-provider transaction.

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:2502-2514
struct InputTextAndImageScopes {
    text_transform: Box<InputTextTransformResolver>,
    text_transform_scope: Option<NativeTextTransformResolverScope<'static>>,
    shape_images: Box<InputShapeImageResolver>,
    shape_image_scope: Option<NativeShapeImageResolverScope<'static>>,
}

impl InputTextAndImageScopes {
    fn new(space: &ConstraintSpace) -> Self {
        let mut text_transform = Box::new(InputTextTransformResolver::new(
            space.text_transform.clone(),
        ));
        let text_transform_ptr = text_transform.as_mut() as *mut dyn NativeTextTransformResolver;
        let text_transform_scope =
            Some(unsafe { NativeTextTransformResolverScope::new_from_raw(text_transform_ptr) });
        let shape_images = Box::new(InputShapeImageResolver::new(&space.images));
        let shape_image_ptr = shape_images.as_ref() as *const dyn NativeShapeImageResolver;
        let shape_image_scope =
            Some(unsafe { NativeShapeImageResolverScope::new_from_raw(shape_image_ptr) });
        Self {
            text_transform,
            text_transform_scope,
            shape_images,
            shape_image_scope,
        }
    }
}

impl Drop for InputTextAndImageScopes {
    fn drop(&mut self) {
        // C++ destroys fields in reverse construction order. Restore both
        // TLS slots before dropping their boxed resolvers.
        self.shape_image_scope.take();
        self.text_transform_scope.take();
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.h:20-33
// cpp: layoutng/internal/boundary/layout_boundary.cc:2516-2521
pub(crate) struct NativeLayoutEnvironment {
    storage_: Box<NativeLayoutEnvironmentStorage>,
}

// cpp: layoutng/internal/boundary/layout_boundary.h:30-32
// cpp: layoutng/internal/boundary/layout_boundary.cc:2495-2514
struct NativeLayoutEnvironmentStorage {
    // C++ constructs font, hyphenation, phrase-break, text and image scopes
    // in that order. Rust drops fields top to bottom, so list them in reverse
    // order and retain stable boxes under each external scope.
    text_and_images: InputTextAndImageScopes,
    phrase_break_scope: NativePhraseBreakResolverScope<'static>,
    phrase_break: Box<InputPhraseBreakResolver>,
    hyphenation_scope: NativeHyphenationResolverScope<'static>,
    hyphenation: Box<InputHyphenationResolver>,
    font_binding: FontBinding,
}

impl NativeLayoutEnvironmentStorage {
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2496-2505
    fn new(root: &mut LayoutObject, space: &ConstraintSpace) -> Self {
        let font_binding = FontBinding::new(root, space);
        let mut hyphenation = Box::new(InputHyphenationResolver::new(space.hyphenation.clone()));
        let hyphenation_ptr = hyphenation.as_mut() as *mut dyn NativeHyphenationResolver;
        let hyphenation_scope =
            unsafe { NativeHyphenationResolverScope::new_from_raw(hyphenation_ptr) };
        let mut phrase_break = Box::new(InputPhraseBreakResolver::new(space.phrase_break.clone()));
        let phrase_break_ptr = phrase_break.as_mut() as *mut dyn NativePhraseBreakResolver;
        let phrase_break_scope =
            unsafe { NativePhraseBreakResolverScope::new_from_raw(phrase_break_ptr) };
        let text_and_images = InputTextAndImageScopes::new(space);
        Self {
            text_and_images,
            phrase_break_scope,
            phrase_break,
            hyphenation_scope,
            hyphenation,
            font_binding,
        }
    }
}

impl NativeLayoutEnvironment {
    // cpp: layoutng/internal/boundary/layout_boundary.h:22-22
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2516-2518
    #[inline(never)]
    pub(crate) fn new(root: &mut LayoutObject, space: &ConstraintSpace) -> Self {
        Self {
            storage_: Box::new(NativeLayoutEnvironmentStorage::new(root, space)),
        }
    }
}

impl LayoutBoundaryEnvironment for NativeLayoutEnvironment {
    // cpp: layoutng/internal/boundary/layout_boundary.h:26-27
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2522-2524
    fn ReusesPreparedFonts(&self) -> bool {
        self.storage_.font_binding.reused
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:2526-2528
    #[inline(never)]
    fn Commit(&mut self, root: &mut LayoutObject) {
        self.storage_.font_binding.Commit(root)
    }
}
