// Copyright 2006 The Android Open Source Project
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Restricted image/path preparation subsets of SkStrike.cpp::prepareImages,
//! preparePaths, prepareForImage and prepareForPath, including SkGlyph.cpp's
//! setImage/setPath once-only creation and remembered absence of a glyph path.
//! Rust HashMap/Arc storage adapts upstream glyph records and arena ownership;
//! this does not implement metrics, path generation, drawables or SkScalerContext.
//! A caller may include a derived stroked path in P; that compound payload is a
//! local adapter, not a translation of an upstream synthetic-bold path record.

use std::collections::HashMap;
use std::hash::Hash;
use std::mem::size_of;
use std::sync::Arc;

/// A resident strike, borrowed for a text run through SkStrikeCache::with_strike.
/// Individual glyph operations hash only glyph keys, not the strike descriptor.
pub struct SkStrike<G, I, P = I> {
    images: HashMap<G, Option<Arc<I>>>,
    paths: HashMap<G, Option<Arc<P>>>,
    pub(crate) memory_used: usize,
    pub(crate) last_used: u64,
    hits: u64,
    misses: u64,
}

impl<G: Eq + Hash, I, P> SkStrike<G, I, P> {
    pub(crate) fn new(memory_used: usize, last_used: u64) -> Self {
        Self {
            images: HashMap::new(),
            paths: HashMap::new(),
            memory_used,
            last_used,
            hits: 0,
            misses: 0,
        }
    }

    /// The outer Option distinguishes unprepared from prepared-but-empty.
    /// Miss probes do not increment misses or admit an offscreen glyph.
    pub fn find_image(&mut self, glyph: &G) -> Option<Option<Arc<I>>> {
        let image = self.images.get(glyph)?.clone();
        self.hits = self.hits.saturating_add(1);
        Some(image)
    }

    /// Lazy once per resident glyph, including negative records. image_size
    /// reports separately allocated payload; I and Arc metadata are counted here.
    /// Budget enforcement happens when the enclosing text-run borrow completes.
    pub fn prepare_image(
        &mut self,
        glyph: G,
        generate: impl FnOnce() -> Option<I>,
        image_size: impl Fn(&I) -> usize,
    ) -> Option<Arc<I>> {
        if let Some(image) = self.find_image(&glyph) {
            return image;
        }
        self.misses = self.misses.saturating_add(1);
        let image = generate().map(Arc::new);
        let record_bytes = size_of::<G>() + size_of::<Option<Arc<I>>>();
        let image_bytes = image.as_ref().map_or(Some(0), |value| {
            image_size(value)
                .checked_add(size_of::<I>())
                .and_then(|bytes| bytes.checked_add(2 * size_of::<usize>()))
        });
        let Some(entry_bytes) = image_bytes.and_then(|bytes| record_bytes.checked_add(bytes))
        else {
            return image;
        };
        let Some(memory_used) = self.memory_used.checked_add(entry_bytes) else {
            return image;
        };
        self.images.insert(glyph, image.clone());
        self.memory_used = memory_used;
        image
    }

    /// Path preparation is independent of image preparation for the same key,
    /// like SkGlyph's separate setPath/setImage flags. None records a glyph
    /// without an outline; a lookup miss does not admit an offscreen glyph.
    pub fn find_path(&mut self, glyph: &G) -> Option<Option<Arc<P>>> {
        let path = self.paths.get(glyph)?.clone();
        self.hits = self.hits.saturating_add(1);
        Some(path)
    }

    /// Translates prepareForPath/setPath's resident-once behavior. The caller
    /// generates the outline and reports its separately allocated storage;
    /// size_of::<P>() and Arc metadata are counted here. A compound P may also
    /// retain a caller-derived stroke, whose allocation must be included in size.
    pub fn prepare_path(
        &mut self,
        glyph: G,
        generate: impl FnOnce() -> Option<P>,
        path_size: impl Fn(&P) -> usize,
    ) -> Option<Arc<P>> {
        if let Some(path) = self.find_path(&glyph) {
            return path;
        }
        self.misses = self.misses.saturating_add(1);
        let path = generate().map(Arc::new);
        let record_bytes = size_of::<G>() + size_of::<Option<Arc<P>>>();
        let path_bytes = path.as_ref().map_or(Some(0), |value| {
            path_size(value)
                .checked_add(size_of::<P>())
                .and_then(|bytes| bytes.checked_add(2 * size_of::<usize>()))
        });
        let Some(entry_bytes) = path_bytes.and_then(|bytes| record_bytes.checked_add(bytes)) else {
            return path;
        };
        let Some(memory_used) = self.memory_used.checked_add(entry_bytes) else {
            return path;
        };
        self.paths.insert(glyph, path.clone());
        self.memory_used = memory_used;
        path
    }

    pub fn hits(&self) -> u64 {
        self.hits
    }

    pub fn misses(&self) -> u64 {
        self.misses
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.images.is_empty() && self.paths.is_empty()
    }
}
