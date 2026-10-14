//! Exact font identity adapter around SkStrike's persistent outline records.
//! Filled and derived stroke paths are replayed separately, preserving existing paint order.
use crate::compat::glyph_bitmaps::GlyphBitmap;
use crate::raster::Path;
use crate::src::core::SkStrikeCache::SkStrikeCache;
use std::{cell::RefCell, sync::Arc};
type SharedBytes = Arc<dyn AsRef<[u8]> + Send + Sync>;
enum FontBytes {
    Shared(SharedBytes),
    Copied(Vec<u8>),
}
impl FontBytes {
    fn as_slice(&self) -> &[u8] {
        match self {
            Self::Shared(b) => b.as_ref().as_ref(),
            Self::Copied(b) => b,
        }
    }
    fn copied_capacity(&self) -> usize {
        match self {
            Self::Shared(_) => 0,
            Self::Copied(b) => b.capacity(),
        }
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub(crate) struct Descriptor {
    pub font: u64,
    pub face_index: u32,
    pub size: u32,
    pub variations: Vec<(u32, u32)>,
    pub italic: bool,
    pub stroke_width: u32,
    pub resolution: u32,
    pub expand_stroke: bool,
}
pub(crate) struct GlyphPaths {
    pub fill: Path,
    pub stroke: Option<Path>,
}
impl GlyphPaths {
    pub fn heap_bytes(&self) -> usize {
        fn size(p: &Path) -> usize {
            p.verbs.capacity() * std::mem::size_of::<crate::path::PathVerb>()
                + p.points.capacity() * std::mem::size_of::<crate::raster::Point>()
        }
        size(&self.fill) + self.stroke.as_ref().map_or(0, size)
    }
}
#[derive(Default)]
struct Cache {
    fonts: Vec<(u64, FontBytes)>,
    next_id: u64,
    bytes: usize,
    strikes: SkStrikeCache<Descriptor, u16, GlyphBitmap, GlyphPaths>,
}
thread_local! {static CACHE:RefCell<Cache> = RefCell::new(Cache::default());}
pub(crate) fn with_paths<R>(
    bytes: &[u8],
    owner: Option<&SharedBytes>,
    mut descriptor: Descriptor,
    f: impl FnOnce(&mut crate::src::core::SkStrike::SkStrike<u16, GlyphBitmap, GlyphPaths>) -> R,
) -> R {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let id = cache
            .fonts
            .iter()
            .find(|(_, stored)| {
                std::ptr::eq(stored.as_slice(), bytes) || stored.as_slice() == bytes
            })
            .map(|(id, _)| *id)
            .unwrap_or_else(|| {
                // Validate the full immutable view before retaining its owner.
                let owner = owner.filter(|owner| {
                    let view: &[u8] = owner.as_ref().as_ref();
                    std::ptr::eq(view, bytes)
                });
                let copied_bytes = if owner.is_some() { 0 } else { bytes.len() };
                // Shared SkData-style owners are bounded by count; only new
                // copies consume this adapter's byte budget. A large shared CJK
                // font must coexist with Latin faces without purging all paths.
                if cache.fonts.len() >= 32
                    || cache.bytes.saturating_add(copied_bytes) > 2 * 1024 * 1024
                {
                    cache.strikes.purge_all();
                    cache.fonts.clear();
                    cache.bytes = 0;
                }
                cache.next_id = cache
                    .next_id
                    .checked_add(1)
                    .expect("font identity overflow");
                let id = cache.next_id;
                // Like SkData/SkTypeface, retain immutable data through its owner.
                // A caller with only borrowed bytes preserves the old exact copy.
                let data = owner.map_or_else(
                    || FontBytes::Copied(bytes.to_vec()),
                    |owner| FontBytes::Shared(owner.clone()),
                );
                cache.bytes += data.copied_capacity();
                cache.fonts.push((id, data));
                id
            });
        descriptor.font = id;
        cache.strikes.with_strike(descriptor, f)
    })
}

pub(crate) fn memory_diagnostics() -> (usize, usize, usize, usize, usize, usize) {
    CACHE.with(|cache| {
        let cache = cache.borrow();
        (
            cache.fonts.len(),
            cache.fonts.capacity() * std::mem::size_of::<(u64, FontBytes)>(),
            cache
                .fonts
                .iter()
                .map(|(_, bytes)| bytes.copied_capacity())
                .sum(),
            cache.strikes.count(),
            cache.strikes.memory_used(),
            cache
                .fonts
                .iter()
                .filter_map(|(_, bytes)| match bytes {
                    FontBytes::Shared(b) => Some(b.as_ref().as_ref().len()),
                    _ => None,
                })
                .sum(),
        )
    })
}

#[cfg(test)]
pub(crate) fn purge() {
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        c.strikes.purge_all();
        c.fonts.clear();
        c.bytes = 0;
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    fn descriptor() -> Descriptor {
        Descriptor {
            font: 0,
            face_index: 0,
            size: 16f32.to_bits(),
            variations: vec![],
            italic: false,
            stroke_width: 0,
            resolution: 1f32.to_bits(),
            expand_stroke: false,
        }
    }
    #[test]
    fn identity_memory_snapshot_preserves_oversized_font_and_strikes() {
        purge();
        let bytes = vec![7; 2 * 1024 * 1024 + 1];
        with_paths(&bytes, None, descriptor(), |strike| {
            strike.prepare_path(7, || None, GlyphPaths::heap_bytes);
        });
        let before = memory_diagnostics();
        assert_eq!(before.3, 1);
        assert_eq!(before.0, 1);
        assert!(before.1 >= std::mem::size_of::<(u64, FontBytes)>());
        assert_eq!(before.2, bytes.len());
        for _ in 0..4 {
            assert_eq!(before, memory_diagnostics());
        }
        with_paths(&bytes, None, descriptor(), |_| {});
        assert_eq!(before, memory_diagnostics());
        purge();
        assert_eq!(memory_diagnostics().2, 0);
    }
    #[test]
    fn shared_identity_retains_backing_without_copy_across_threads() {
        let bytes = Arc::new(vec![7; 2 * 1024 * 1024 + 1]);
        let weak = Arc::downgrade(&bytes);
        let owner: SharedBytes = bytes.clone();
        drop(bytes);
        std::thread::spawn(move || {
            purge();
            let view: &[u8] = owner.as_ref().as_ref();
            with_paths(view, Some(&owner), descriptor(), |_| {});
            assert_eq!(memory_diagnostics().2, 0);
            assert_eq!(memory_diagnostics().5, view.len());
            with_paths(&[1, 2, 3], None, descriptor(), |_| {});
            with_paths(view, Some(&owner), descriptor(), |_| {});
            assert_eq!(
                memory_diagnostics().0,
                2,
                "large shared and small faces coexist"
            );
            assert_eq!(memory_diagnostics().2, 3);
            drop(owner);
            assert!(
                weak.upgrade().is_some(),
                "cache must own the only remaining backing"
            );
            purge();
            assert!(
                weak.upgrade().is_none(),
                "purge must release the last owner"
            );
        })
        .join()
        .unwrap();
    }
    #[test]
    fn shared_identity_checks_complete_view_and_exact_equal_bytes() {
        purge();
        let owner: SharedBytes = Arc::new(vec![1u8, 2, 3]);
        let view: &[u8] = owner.as_ref().as_ref();
        with_paths(view, Some(&owner), descriptor(), |_| {});
        with_paths(&[1, 2, 3], None, descriptor(), |_| {});
        assert_eq!(memory_diagnostics().0, 1);
        assert_eq!(memory_diagnostics().2, 0);
        // A reassigned public view must not retain an unrelated backing.
        with_paths(&[4, 5], Some(&owner), descriptor(), |_| {});
        assert_eq!(memory_diagnostics().0, 2);
        assert_eq!(memory_diagnostics().2, 2);
        with_paths(&view[..2], Some(&owner), descriptor(), |_| {});
        assert_eq!(memory_diagnostics().2, 4);
        purge();
    }
    #[test]
    fn font_identity_uses_bytes_not_storage_address() {
        purge();
        let calls = std::cell::Cell::new(0);
        let mut bytes = vec![1, 2, 3];
        let run = |bytes: &[u8]| {
            with_paths(bytes, None, descriptor(), |s| {
                s.prepare_path(
                    7,
                    || {
                        calls.set(calls.get() + 1);
                        None
                    },
                    GlyphPaths::heap_bytes,
                );
            })
        };
        run(&bytes);
        run(&bytes.clone());
        assert_eq!(calls.get(), 1);
        bytes[0] = 4;
        run(&bytes);
        assert_eq!(calls.get(), 2);
    }
    #[test]
    fn path_parameters_partition_strikes_and_share_identical_records() {
        purge();
        let calls = std::cell::Cell::new(0);
        let d = descriptor();
        let run = |d: Descriptor| {
            with_paths(&[1], None, d, |s| {
                s.prepare_path(
                    4,
                    || {
                        calls.set(calls.get() + 1);
                        None
                    },
                    GlyphPaths::heap_bytes,
                );
            })
        };
        run(d.clone());
        run(d.clone());
        assert_eq!(calls.get(), 1);
        let mut changed = d.clone();
        changed.face_index = 1;
        run(changed);
        let mut changed = d.clone();
        changed.variations.push((0x77676874, 500f32.to_bits()));
        run(changed);
        let mut changed = d;
        changed.italic = true;
        run(changed);
        assert_eq!(calls.get(), 4);
    }
}
