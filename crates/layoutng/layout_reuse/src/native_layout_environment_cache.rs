//! Reusable native font environment and per-pass resolver binding.
//!
//! Included by the reciprocal native boundary. Snapshot admission, cache
//! transfer, and resolver lifetime remain assembly-aware while trace and
//! sampling ownership resolve to `layout_reuse`.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:76-111
pub(crate) struct ReusableLayoutEnvironment {
    snapshot: ConstraintSpace,
    // The native cache is shared with its root but only one layout pass uses
    // its mutable resolver at a time, as in the source shared_ptr boundary.
    fonts: UnsafeCell<Box<dyn NativeFontResolver>>,
    font_query_revision: u64,
}

impl ReusableLayoutEnvironment {
    fn new(input: &ConstraintSpace, resolver: Box<dyn NativeFontResolver>) -> Self {
        Self {
            snapshot: ConstraintSpace {
                available_size: input.available_size,
                fragmentainer_block_size: input.fragmentainer_block_size,
                compatibility_mode: input.compatibility_mode,
                printing: input.printing,
                vertical_scroll_enforced: input.vertical_scroll_enforced,
                scrollbar_theme: input.scrollbar_theme,
                minimum_font_physical_size: input.minimum_font_physical_size,
                device_pixel_ratio: input.device_pixel_ratio,
                layout_zoom_factor: input.layout_zoom_factor,
                canvas_draw_element_enabled: input.canvas_draw_element_enabled,
                prefer_non_composited_scrolling_enabled: input
                    .prefer_non_composited_scrolling_enabled,
                viewport: input.viewport,
                control_theme: input.control_theme,
                hyphenation: input.hyphenation.clone(),
                phrase_break: input.phrase_break.clone(),
                text_transform: input.text_transform.clone(),
                font_backend_factory: input.font_backend_factory.clone(),
                font_engine: input.font_engine.clone(),
                fonts: input.fonts.clone(),
                font_queries: input.font_queries.clone(),
                ..ConstraintSpace::default()
            },
            fonts: UnsafeCell::new(resolver),
            font_query_revision: input
                .font_queries
                .as_ref()
                .map_or(0, font_engine::FontQueryCollector::Revision),
        }
    }

    fn Eligible(_space: &ConstraintSpace) -> bool {
        // Images and host callbacks have their own per-pass scopes; only fonts are cached.
        true
    }

    #[inline(never)]
    fn Matches(&self, space: &ConstraintSpace) -> bool {
        fn same_provider<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
        }
        Self::Eligible(space)
            && self.font_query_revision
                == space.font_queries.as_ref().map_or(0, font_engine::FontQueryCollector::Revision)
            && same_provider(&self.snapshot.hyphenation, &space.hyphenation)
            && same_provider(&self.snapshot.phrase_break, &space.phrase_break)
            && same_provider(&self.snapshot.text_transform, &space.text_transform)
            // Width/height change constraints and line breaking, not the
            // OpenType faces or resolved font requests cached here. Styles
            // computed from viewport units/media queries retain their normal
            // per-node font binding invalidation in the resident tree.
            && self.snapshot.fragmentainer_block_size == space.fragmentainer_block_size
            && self.snapshot.compatibility_mode == space.compatibility_mode
            && self.snapshot.printing == space.printing
            && self.snapshot.vertical_scroll_enforced == space.vertical_scroll_enforced
            && self.snapshot.scrollbar_theme == space.scrollbar_theme
            && self.snapshot.minimum_font_physical_size == space.minimum_font_physical_size
            && self.snapshot.device_pixel_ratio == space.device_pixel_ratio
            && self.snapshot.layout_zoom_factor == space.layout_zoom_factor
            && self.snapshot.canvas_draw_element_enabled == space.canvas_draw_element_enabled
            && self.snapshot.prefer_non_composited_scrolling_enabled == space.prefer_non_composited_scrolling_enabled
            && self.snapshot.control_theme == space.control_theme
            && match (
                &self.snapshot.font_backend_factory,
                &space.font_backend_factory,
            ) {
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
            && match (&self.snapshot.font_engine, &space.font_engine) {
                (Some(left), Some(right)) => left.ptr_eq(right),
                (None, None) => true,
                _ => false,
            }
            && match (&self.snapshot.font_queries, &space.font_queries) {
                (Some(left), Some(right)) => left.ptr_eq(right),
                (None, None) => true,
                _ => false,
            }
            && self.snapshot.fonts == space.fonts
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2463-2494
pub(super) struct FontBinding {
    cache: Option<Arc<ReusableLayoutEnvironment>>,
    transient: Option<Box<dyn NativeFontResolver>>,
    scope: Option<NativeFontResolverScope<'static>>,
    pub(super) reused: bool,
}

impl FontBinding {
    #[inline(never)]
    pub(super) fn new(root: &mut LayoutObject, space: &ConstraintSpace) -> Self {
        let mut trace = browser_tracing::span("layout", "layout_reuse.FontEnvironment");
        let root_node = root.GetNode();
        if root_node.is_null() {
            panic!("Layout root requires standalone metadata");
        }
        let previous = unsafe { &mut *root_node }.TakeReusableLayoutEnvironment();
        let mut cache = None;
        let mut transient: Option<Box<dyn NativeFontResolver>> = None;
        let mut reused = false;
        if let Some(previous) = previous {
            if previous.Matches(space) {
                cache = Some(previous);
                // Provider identity is part of Matches(). As in Blink, a
                // stable layout input cannot silently change its shaping
                // semantics between lifecycle updates; hosts publish a new
                // provider (or another explicit layout mutation) when those
                // semantics change. Rebinding every node merely because a
                // stable provider exists turns every otherwise clean pass into
                // a full-font layout and defeats LayoutNG's result cache.
                reused = true;
            }
        }
        if cache.is_none() {
            let eligible = ReusableLayoutEnvironment::Eligible(space);
            let resolver = EngineFontResolver(space);
            if eligible {
                cache = Some(Arc::new(ReusableLayoutEnvironment::new(space, resolver)));
            } else {
                transient = Some(resolver);
            }
        }
        let resolver: *mut dyn NativeFontResolver = if let Some(cache) = cache.as_ref() {
            // SAFETY: the layout environment owns this active pass; its scope
            // is dropped before the cache or root may release the resolver.
            unsafe { (&mut *cache.fonts.get()).as_mut() }
        } else {
            transient
                .as_mut()
                .expect("transient font resolver")
                .as_mut()
        };
        let scope = Some(unsafe { NativeFontResolverScope::new_from_raw(resolver) });
        trace.set("reused", reused as u8 as f64);
        trace.set("cached", cache.is_some() as u8 as f64);
        Self {
            cache,
            transient,
            scope,
            reused,
        }
    }

    pub(super) fn Commit(&mut self, root: &mut LayoutObject) {
        if let Some(cache) = self.cache.take() {
            let root_node = root.GetNode();
            unsafe { &mut *root_node }.SetReusableLayoutEnvironment(Some(cache));
        }
    }
}

impl Drop for FontBinding {
    fn drop(&mut self) {
        // The TLS pointer must be restored before either resolver is released.
        self.scope.take();
    }
}
