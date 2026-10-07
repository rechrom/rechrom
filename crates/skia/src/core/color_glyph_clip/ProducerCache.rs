//! Owned pristine path products. Never cache an intersected clip or pixels.
use super::{produce_path, PathProduct};
use crate::compat::commands::PaintPathCommand;
use crate::raster::{FillRule, Path, Transform};
use crate::src::core::SkColorGlyphClip::{Bounds, CanvasClipOwner, ReceiverKind};
#[path = "ProducerKey.rs"]
mod key;

const SLOTS: usize = 64;
// Admit large pristine rounded clips while retaining the total 8 MiB budget.
const ENTRY_LIMIT: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct RasterClipProductCacheStats {
    pub requests: u64,
    pub original_producer_calls: u64,
    pub hits: u64,
    pub misses: u64,
    pub disabled: u64,
    pub key_failures: u64,
    pub copy_failures: u64,
    pub unpristine: u64,
    pub oversize_or_overflow: u64,
    pub slot_failures: u64,
    pub evictions: u64,
    pub hit_mask_copy_bytes: u64,
    pub insertion_mask_copy_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct RasterClipProductCacheAccounting {
    pub configured_byte_limit: usize,
    pub configured_entry_limit: usize,
    pub cache_inline_bytes: usize,
    pub fixed_slots_len: usize,
    pub fixed_slots_capacity: usize,
    pub fixed_slots_bytes: usize,
    pub occupied_slots: usize,
    pub entry_key_capacity_bytes: usize,
    pub entry_mask_capacity_bytes: usize,
    pub entry_encoded_owner_heap_bytes: usize,
    pub resident_requested_bytes: usize,
    pub allocator_overhead_unknown: bool,
}

struct Entry {
    key: Vec<u64>,
    product: PathProduct,
    age: u64,
    key_bytes: usize,
    mask_bytes: usize,
    owner_bytes: usize,
    heap_bytes: usize,
}

/// Exact-key owned cache, moved through each actual canvas. The 64 slot array
/// is allocated lazily and never grows; its actual capacity including EMPTY
/// slots and the entire cache inline layout count against the requested-byte
/// budget. Unknown allocator rounding/control metadata is not hidden as zero.
/// A zero limit disables products; its metadata-only inline owner still exists.
pub struct RasterClipProductCache {
    slots: Vec<Option<Entry>>,
    byte_limit: usize,
    heap_bytes: usize,
    clock: u64,
    stats: RasterClipProductCacheStats,
}

impl Default for RasterClipProductCache {
    fn default() -> Self {
        Self::with_byte_limit(8 * 1024 * 1024)
    }
}

impl RasterClipProductCache {
    pub fn with_byte_limit(byte_limit: usize) -> Self {
        Self {
            slots: Vec::new(),
            byte_limit,
            heap_bytes: 0,
            clock: 0,
            stats: Default::default(),
        }
    }
    pub fn stats(&self) -> RasterClipProductCacheStats {
        self.stats
    }
    fn container_bytes(&self) -> Option<usize> {
        self.slots
            .capacity()
            .checked_mul(std::mem::size_of::<Option<Entry>>())?
            .checked_add(std::mem::size_of::<Self>())
    }
    pub fn accounting(&self) -> RasterClipProductCacheAccounting {
        let mut a = RasterClipProductCacheAccounting {
            configured_byte_limit: self.byte_limit,
            configured_entry_limit: ENTRY_LIMIT,
            cache_inline_bytes: std::mem::size_of::<Self>(),
            fixed_slots_len: self.slots.len(),
            fixed_slots_capacity: self.slots.capacity(),
            fixed_slots_bytes: self.slots.capacity() * std::mem::size_of::<Option<Entry>>(),
            occupied_slots: 0,
            entry_key_capacity_bytes: 0,
            entry_mask_capacity_bytes: 0,
            entry_encoded_owner_heap_bytes: 0,
            resident_requested_bytes: 0,
            allocator_overhead_unknown: true,
        };
        for e in self.slots.iter().flatten() {
            a.occupied_slots += 1;
            a.entry_key_capacity_bytes += e.key_bytes;
            a.entry_mask_capacity_bytes += e.mask_bytes;
            a.entry_encoded_owner_heap_bytes += e.owner_bytes;
        }
        a.resident_requested_bytes = a.cache_inline_bytes
            + a.fixed_slots_bytes
            + a.entry_key_capacity_bytes
            + a.entry_mask_capacity_bytes
            + a.entry_encoded_owner_heap_bytes;
        debug_assert_eq!(
            self.heap_bytes,
            a.entry_key_capacity_bytes
                + a.entry_mask_capacity_bytes
                + a.entry_encoded_owner_heap_bytes
        );
        a
    }
    fn next_age(&mut self) -> u64 {
        if self.clock == u64::MAX {
            // Safe loss of reusable products rather than wrapped LRU ages.
            for e in &mut self.slots {
                *e = None;
            }
            self.heap_bytes = 0;
            self.clock = 0;
        }
        self.clock += 1;
        self.clock
    }
    fn allocate_slots(&mut self, entry_heap: usize) -> bool {
        if !self.slots.is_empty() {
            return true;
        }
        let mut slots = Vec::new();
        if slots.try_reserve_exact(SLOTS).is_err() {
            return false;
        }
        let charge = slots
            .capacity()
            .checked_mul(std::mem::size_of::<Option<Entry>>())
            .and_then(|b| b.checked_add(std::mem::size_of::<Self>()))
            .and_then(|b| b.checked_add(entry_heap));
        if charge.is_none_or(|b| b > self.byte_limit) {
            return false;
        }
        slots.resize_with(SLOTS, || None);
        self.slots = slots;
        true
    }
    fn evict_one(&mut self) -> bool {
        let Some(index) = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(i, e)| e.as_ref().map(|e| (i, e.age)))
            .min_by_key(|(_, age)| *age)
            .map(|(i, _)| i)
        else {
            return false;
        };
        let e = self.slots[index].take().unwrap();
        self.heap_bytes -= e.heap_bytes;
        self.stats.evictions = self.stats.evictions.saturating_add(1);
        true
    }
    fn produce(
        &mut self,
        path: &Path,
        commands: Option<&[PaintPathCommand]>,
        rule: FillRule,
        aa: bool,
        t: Transform,
        width: u32,
        height: u32,
        bounds: Bounds,
        kind: ReceiverKind,
    ) -> PathProduct {
        self.stats.requests = self.stats.requests.saturating_add(1);
        let original = || produce_path(path, commands, rule, aa, t, width, height, bounds, kind);
        if self.byte_limit == 0 {
            self.stats.disabled = self.stats.disabled.saturating_add(1);
            self.stats.original_producer_calls =
                self.stats.original_producer_calls.saturating_add(1);
            return original();
        }
        let Some(key) = key::exact_key(path, commands, rule, aa, t, width, height, bounds, kind)
        else {
            self.stats.key_failures = self.stats.key_failures.saturating_add(1);
            self.stats.original_producer_calls =
                self.stats.original_producer_calls.saturating_add(1);
            return original();
        };
        // Full integer-vector equality. No digest, pointer/NodeId, coordinates
        // modulo a tile, or normalized transform/scroll phase decides a hit.
        if let Some(index) = self
            .slots
            .iter()
            .position(|e| e.as_ref().is_some_and(|e| e.key == key))
        {
            let age = self.next_age();
            // next_age may have cleared the entries on clock overflow.
            if let Some(entry) = self.slots[index].as_mut() {
                if let Some(mask) = entry.product.mask.try_clone_pristine_clip_product() {
                    self.stats.hits = self.stats.hits.saturating_add(1);
                    self.stats.hit_mask_copy_bytes = self.stats.hit_mask_copy_bytes.saturating_add(
                        entry
                            .product
                            .mask
                            .pristine_clip_product_allocation()
                            .unwrap()
                            .1 as u64,
                    );
                    entry.age = age;
                    return PathProduct {
                        mask,
                        owner: entry.product.owner.clone(),
                    };
                }
                self.stats.copy_failures = self.stats.copy_failures.saturating_add(1);
                // Preserve the existing pristine entry; retrying insertion on
                // an allocation failure would create a duplicate key/slot.
                self.stats.misses = self.stats.misses.saturating_add(1);
                self.stats.original_producer_calls =
                    self.stats.original_producer_calls.saturating_add(1);
                return original();
            }
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        self.stats.original_producer_calls = self.stats.original_producer_calls.saturating_add(1);
        let product = original();
        let Some((mask_bytes, mask_len)) = product.mask.pristine_clip_product_allocation() else {
            self.stats.unpristine = self.stats.unpristine.saturating_add(1);
            return product;
        };
        let Some(owner_bytes) = product.owner.clip_product_owned_heap_bytes() else {
            self.stats.oversize_or_overflow = self.stats.oversize_or_overflow.saturating_add(1);
            return product;
        };
        let Some(key_bytes) = key.capacity().checked_mul(std::mem::size_of::<u64>()) else {
            return product;
        };
        let Some(heap_bytes) = key_bytes
            .checked_add(mask_bytes)
            .and_then(|b| b.checked_add(owner_bytes))
        else {
            return product;
        };
        let entry_total = heap_bytes.checked_add(std::mem::size_of::<Option<Entry>>());
        if entry_total.is_none_or(|b| b > ENTRY_LIMIT) {
            self.stats.oversize_or_overflow = self.stats.oversize_or_overflow.saturating_add(1);
            return product;
        }
        if !self.allocate_slots(heap_bytes) {
            self.stats.slot_failures = self.stats.slot_failures.saturating_add(1);
            return product;
        }
        let base = self.container_bytes().unwrap();
        if base
            .checked_add(heap_bytes)
            .is_none_or(|b| b > self.byte_limit)
        {
            self.stats.oversize_or_overflow = self.stats.oversize_or_overflow.saturating_add(1);
            return product;
        }
        // The original product itself is moved into the cache; its capacities
        // are measured already. The fallible deep copy is the active CALLER's
        // independently mutable mask, not unaccounted cached storage.
        let Some(mask) = product.mask.try_clone_pristine_clip_product() else {
            self.stats.copy_failures = self.stats.copy_failures.saturating_add(1);
            return product;
        };
        while self.slots.iter().all(Option::is_some)
            || base
                .checked_add(self.heap_bytes)
                .and_then(|b| b.checked_add(heap_bytes))
                .is_none_or(|b| b > self.byte_limit)
        {
            if !self.evict_one() {
                return product;
            }
        }
        let age = self.next_age();
        let index = self.slots.iter().position(Option::is_none).unwrap();
        let owner = product.owner.clone();
        self.heap_bytes += heap_bytes;
        self.slots[index] = Some(Entry {
            key,
            product,
            age,
            key_bytes,
            mask_bytes,
            owner_bytes,
            heap_bytes,
        });
        self.stats.insertion_mask_copy_bytes = self
            .stats
            .insertion_mask_copy_bytes
            .saturating_add(mask_len as u64);
        debug_assert!(self.accounting().resident_requested_bytes <= self.byte_limit);
        PathProduct { mask, owner }
    }
}

pub(crate) fn produce_path_cached(
    cache: &mut Option<RasterClipProductCache>,
    path: &Path,
    commands: Option<&[PaintPathCommand]>,
    rule: FillRule,
    aa: bool,
    t: Transform,
    width: u32,
    height: u32,
    bounds: Bounds,
    kind: ReceiverKind,
) -> PathProduct {
    match cache {
        Some(cache) => cache.produce(path, commands, rule, aa, t, width, height, bounds, kind),
        None => produce_path(path, commands, rule, aa, t, width, height, bounds, kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn geometry(x: f64) -> (Path, Vec<PaintPathCommand>) {
        let c = crate::compat::geometry::rect_commands(crate::compat::commands::PaintRect {
            x,
            y: 2.375,
            width: 31.5,
            height: 19.75,
        })
        .unwrap();
        (crate::compat::geometry::path_from_commands(&c).unwrap(), c)
    }
    fn assert_products(actual: &PathProduct, original: &PathProduct) {
        assert_eq!(actual.mask.data(), original.mask.data());
        assert_eq!(
            format!("{:?}", actual.owner),
            format!("{:?}", original.owner)
        );
        assert_eq!(
            actual.owner.row_run_starts(96),
            original.owner.row_run_starts(96)
        );
        for glyph in [
            Bounds([0, 0, 1, 1]),
            Bounds([3, 4, 8, 9]),
            Bounds([31, 19, 45, 31]),
        ] {
            for f16 in [false, true] {
                assert_eq!(
                    actual.owner.composite_mode(f16, glyph),
                    original.owner.composite_mode(f16, glyph)
                );
            }
        }
    }
    #[test]
    fn original_raw_products_match_after_real_hits_and_caller_mutation() {
        let bounds = Bounds([0, 0, 96, 80]);
        let callers = [
            (CanvasClipOwner::device(bounds), false),
            (
                CanvasClipOwner::Aa(
                    crate::src::core::SkColorGlyphClip::dispatch::Encoding::set_rect(bounds),
                ),
                false,
            ),
            (CanvasClipOwner::device(bounds), true),
        ];
        let mut pairs = 0;
        for (caller, aa) in callers {
            let kind = caller.path_receiver_kind(aa);
            for rule in [FillRule::Winding, FillRule::EvenOdd] {
                for supplied in [false, true] {
                    for t in [
                        Transform::identity(),
                        Transform::from_row(1.5, 0.0, 0.25, 1.5, 5.125, -7.75),
                        Transform::from_row(1.0, 0.125, -0.25, 1.0, -3.5, 7.125),
                    ] {
                        let mut cache = RasterClipProductCache::default();
                        let (path, commands) = geometry(1.125);
                        let source = supplied.then_some(commands.as_slice());
                        let original =
                            produce_path(&path, source, rule, aa, t, 96, 80, bounds, kind);
                        let mut first =
                            cache.produce(&path, source, rule, aa, t, 96, 80, bounds, kind);
                        assert_products(&first, &original);
                        first.mask.data_mut().fill(37); // caller intersection must not poison entry
                        let hit = cache.produce(&path, source, rule, aa, t, 96, 80, bounds, kind);
                        assert_products(&hit, &original);
                        assert_eq!(cache.stats().hits, 1);
                        assert_eq!(cache.stats().original_producer_calls, 1);
                        assert!(cache.accounting().resident_requested_bytes <= 8 * 1024 * 1024);
                        assert_eq!(cache.accounting().fixed_slots_len, 64);
                        assert_eq!(
                            cache.accounting().fixed_slots_bytes,
                            cache.accounting().fixed_slots_capacity
                                * std::mem::size_of::<Option<Entry>>()
                        );
                        pairs += 1;
                    }
                }
            }
        }
        assert_eq!(pairs, 36);
    }
    #[test]
    fn disabled_limits_fixed_slot_eviction_and_clock_overflow_preserve_original() {
        let bounds = Bounds([0, 0, 96, 80]);
        for limit in [0, 1, std::mem::size_of::<RasterClipProductCache>()] {
            let mut cache = RasterClipProductCache::with_byte_limit(limit);
            let (p, c) = geometry(1.125);
            for _ in 0..2 {
                let actual = cache.produce(
                    &p,
                    Some(&c),
                    FillRule::Winding,
                    true,
                    Transform::identity(),
                    96,
                    80,
                    bounds,
                    ReceiverKind::AaClip,
                );
                assert_products(
                    &actual,
                    &produce_path(
                        &p,
                        Some(&c),
                        FillRule::Winding,
                        true,
                        Transform::identity(),
                        96,
                        80,
                        bounds,
                        ReceiverKind::AaClip,
                    ),
                );
            }
            assert_eq!(cache.stats().hits, 0);
            assert_eq!(cache.accounting().occupied_slots, 0);
        }
        let mut cache = RasterClipProductCache::default();
        for n in 0..70 {
            let (p, c) = geometry(1.125 + n as f64 / 128.0);
            let actual = cache.produce(
                &p,
                Some(&c),
                FillRule::Winding,
                true,
                Transform::identity(),
                96,
                80,
                bounds,
                ReceiverKind::AaClip,
            );
            assert_products(
                &actual,
                &produce_path(
                    &p,
                    Some(&c),
                    FillRule::Winding,
                    true,
                    Transform::identity(),
                    96,
                    80,
                    bounds,
                    ReceiverKind::AaClip,
                ),
            );
            assert!(cache.accounting().resident_requested_bytes <= 8 * 1024 * 1024);
        }
        assert_eq!(cache.accounting().occupied_slots, 64);
        assert_eq!(cache.stats().evictions, 6);
        let (p, c) = geometry(1.125 + 69.0 / 128.0);
        cache.clock = u64::MAX;
        let actual = cache.produce(
            &p,
            Some(&c),
            FillRule::Winding,
            true,
            Transform::identity(),
            96,
            80,
            bounds,
            ReceiverKind::AaClip,
        );
        assert_products(
            &actual,
            &produce_path(
                &p,
                Some(&c),
                FillRule::Winding,
                true,
                Transform::identity(),
                96,
                80,
                bounds,
                ReceiverKind::AaClip,
            ),
        );
        assert_eq!(cache.accounting().occupied_slots, 1);
    }
    #[test]
    fn cache_keys_retain_complete_source_bits_not_phase_or_partial_path_identity() {
        let (p, c) = geometry(1.125);
        let make =
            |c: Option<&[PaintPathCommand]>,
             t: Transform,
             b: Bounds,
             r: FillRule,
             aa: bool,
             k: ReceiverKind| key::exact_key(&p, c, r, aa, t, 96, 80, b, k).unwrap();
        let t = Transform::identity();
        let b = Bounds([0, 0, 96, 80]);
        let base = make(
            Some(&c),
            t,
            b,
            FillRule::Winding,
            true,
            ReceiverKind::AaClip,
        );
        assert_ne!(
            base,
            make(None, t, b, FillRule::Winding, true, ReceiverKind::AaClip)
        );
        assert_ne!(
            base,
            make(
                Some(&[]),
                t,
                b,
                FillRule::Winding,
                true,
                ReceiverKind::AaClip
            )
        );
        let mut changed = c.clone();
        changed[0].control2.y = f64::from_bits(1);
        assert_ne!(
            base,
            make(
                Some(&changed),
                t,
                b,
                FillRule::Winding,
                true,
                ReceiverKind::AaClip
            )
        );
        assert_ne!(
            base,
            make(
                Some(&c),
                Transform::from_row(1.0, -0.0, 0.0, 1.0, 0.0, 0.0),
                b,
                FillRule::Winding,
                true,
                ReceiverKind::AaClip
            )
        );
        assert_ne!(
            base,
            make(
                Some(&c),
                t,
                Bounds([0, 1, 96, 80]),
                FillRule::Winding,
                true,
                ReceiverKind::AaClip
            )
        );
        assert_ne!(
            base,
            make(
                Some(&c),
                t,
                b,
                FillRule::EvenOdd,
                true,
                ReceiverKind::AaClip
            )
        );
        assert_ne!(
            base,
            make(
                Some(&c),
                t,
                b,
                FillRule::Winding,
                false,
                ReceiverKind::AaClip
            )
        );
        assert_ne!(
            base,
            make(
                Some(&c),
                t,
                b,
                FillRule::Winding,
                false,
                ReceiverKind::BwRegion
            )
        );
    }
    #[test]
    fn owned_canvas_transfers_match_uncached_actual_intersections_layers_and_all_target_bytes() {
        use crate::compat::commands::{
            Color, CommandKind, DrawCommand, PaintCornerRadii, PaintCornerRadius, PaintRect,
            ResourceContext,
        };
        use crate::src::core::SkCanvas::SkCanvas;
        use crate::{PixelFormat, PixelStorage};
        let resources = ResourceContext::default();
        let (width, height) = (131u32, 112u32); // actual visible viewport128, 3 padding columns
        let rect = |x: f64, y: f64, width: f64, height: f64| PaintRect {
            x,
            y,
            width,
            height,
        };
        let radii = |r: f64| PaintCornerRadii {
            top_left: PaintCornerRadius { x: r, y: r },
            top_right: PaintCornerRadius { x: r, y: r },
            bottom_right: PaintCornerRadius { x: r, y: r },
            bottom_left: PaintCornerRadius { x: r, y: r },
        };
        let run = |cache: Option<RasterClipProductCache>,
                   format: PixelFormat,
                   scale: f64,
                   f16: bool,
                   prior: u8,
                   frame: u8| {
            let initial: Vec<u8> = (0..width as usize * height as usize)
                .flat_map(|i| format.encode([(i * 17) as u8, (i * 31) as u8, (i * 7) as u8, 255]))
                .collect();
            let mut canvas = SkCanvas::make_raster_direct_with_format_preserving(
                &resources,
                width,
                height,
                width as usize * 4,
                PixelStorage::owned(initial.clone()),
                format,
                true,
            )
            .unwrap();
            if let Some(cache) = cache {
                canvas.install_clip_product_cache(cache);
            }
            canvas.replay_item(
                &DrawCommand {
                    r#type: CommandKind::kClipRect,
                    rect: rect(0., 0., 128., 112.),
                    antialias: false,
                    ..Default::default()
                },
                &resources,
            );
            canvas.set_scale(scale);
            if prior != 0 {
                canvas.replay_item(
                    &DrawCommand {
                        r#type: if prior == 1 {
                            CommandKind::kClipRect
                        } else {
                            CommandKind::kClipRoundedRect
                        },
                        rect: rect(1.25, 2.375, 61.5, 50.25),
                        corner_radii: radii(3.5),
                        antialias: true,
                        ..Default::default()
                    },
                    &resources,
                );
            }
            for _ in 0..3 {
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kSave,
                        ..Default::default()
                    },
                    &resources,
                );
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kClipRoundedRect,
                        rect: rect(4.125, 5.375, 50.5, 39.25),
                        corner_radii: radii(5.25),
                        antialias: true,
                        ..Default::default()
                    },
                    &resources,
                );
                canvas.internalSaveLayer(
                    &DrawCommand {
                        rect: rect(2.5, 3.25, 56.75, 44.5),
                        opacity: 0.5,
                        ..Default::default()
                    },
                    f16,
                );
                // Child original allocation/local F32 mapping supplies its own
                // complete key. Cache hits must still run original AA owner /
                // dense intersection, F16/N32 restore and actual blending.
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kSave,
                        ..Default::default()
                    },
                    &resources,
                );
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kClipRoundedRect,
                        rect: rect(6.125, 7.375, 43.5, 31.25),
                        corner_radii: radii(4.125),
                        antialias: true,
                        ..Default::default()
                    },
                    &resources,
                );
                canvas.replay_item(
                    &DrawCommand {
                        r#type: CommandKind::kDrawRect,
                        rect: rect(0., 0., 65., 55.),
                        color: Color {
                            red: if frame == 0 { 0.25 } else { 0.75 },
                            green: 0.4,
                            blue: 0.7,
                            alpha: 0.65,
                        },
                        antialias: true,
                        ..Default::default()
                    },
                    &resources,
                );
                for _ in 0..3 {
                    canvas.replay_item(
                        &DrawCommand {
                            r#type: CommandKind::kRestore,
                            ..Default::default()
                        },
                        &resources,
                    );
                }
            }
            let owner = format!("{:?}", canvas.state.clip_encoding);
            let row_starts = canvas.state.clip_encoding.row_run_starts(width as usize);
            let modes = [Bounds([3, 4, 8, 9]), Bounds([23, 29, 45, 57])].map(|glyph| {
                [
                    canvas.state.clip_encoding.composite_mode(false, glyph),
                    canvas.state.clip_encoding.composite_mode(true, glyph),
                ]
            });
            let mask = canvas.state.clip.as_ref().map(|m| m.data().to_vec());
            let cache = canvas.take_clip_product_cache();
            let pixels = canvas.finish_direct().into_vec();
            for row in 0..height as usize {
                let padding = (row * width as usize + 128) * 4..((row + 1) * width as usize) * 4;
                assert_eq!(pixels[padding.clone()], initial[padding]);
            }
            (pixels, owner, row_starts, modes, mask, cache)
        };
        let mut frames = 0;
        for format in [
            PixelFormat::Rgba8888,
            PixelFormat::Bgra8888,
            PixelFormat::Bgrx8888,
        ] {
            for scale in [1.0, 1.5, 2.0] {
                for f16 in [false, true] {
                    for prior in [0, 1, 2] {
                        let mut cache = RasterClipProductCache::default();
                        for frame in 0..2 {
                            let original = run(None, format, scale, f16, prior, frame);
                            let actual = run(Some(cache), format, scale, f16, prior, frame);
                            assert_eq!(actual.0, original.0);
                            assert_eq!(actual.1, original.1);
                            assert_eq!(actual.2, original.2);
                            assert_eq!(actual.3, original.3);
                            assert_eq!(actual.4, original.4);
                            cache = actual.5;
                            assert!(
                                cache.stats().hits > 0,
                                "fixture must exercise actual cache hits"
                            );
                            assert!(cache.accounting().resident_requested_bytes <= 8 * 1024 * 1024);
                            frames += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(frames, 108);
    }
}
