//! Exact font identity adapter around SkStrike's persistent outline records.
//! Filled and derived stroke paths are replayed separately, preserving existing paint order.
use crate::raster::Path;
use crate::src::core::SkStrikeCache::SkStrikeCache;
use std::cell::RefCell;

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
    fonts: Vec<(u64, Vec<u8>)>,
    next_id: u64,
    bytes: usize,
    strikes: SkStrikeCache<Descriptor, u16, (), GlyphPaths>,
}
thread_local! {static CACHE:RefCell<Cache> = RefCell::new(Cache::default());}
pub(crate) fn with_paths<R>(
    bytes: &[u8],
    mut descriptor: Descriptor,
    f: impl FnOnce(&mut crate::src::core::SkStrike::SkStrike<u16, (), GlyphPaths>) -> R,
) -> R {
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let id = cache
            .fonts
            .iter()
            .find(|(_, stored)| stored.as_slice() == bytes)
            .map(|(id, _)| *id)
            .unwrap_or_else(|| {
                // Font ownership is separately bounded; purge paths before releasing an identity.
                if cache.fonts.len() >= 32
                    || cache.bytes.saturating_add(bytes.len()) > 2 * 1024 * 1024
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
                cache.fonts.push((id, bytes.to_vec()));
                cache.bytes += bytes.len();
                id
            });
        descriptor.font = id;
        cache.strikes.with_strike(descriptor, f)
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
    fn font_identity_uses_bytes_not_storage_address() {
        purge();
        let calls = std::cell::Cell::new(0);
        let mut bytes = vec![1, 2, 3];
        let run = |bytes: &[u8]| {
            with_paths(bytes, descriptor(), |s| {
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
            with_paths(&[1], d, |s| {
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
