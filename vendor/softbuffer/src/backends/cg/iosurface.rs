//! Opt-in macOS IOSurface storage for the existing Core Animation backend.
//! Ownership follows Chromium SoftwareOutputDeviceMac: at most four retained
//! buffers, server-in-use check, lock -> full CPU paint -> unlock -> contents.
use crate::SoftBufferError;
use objc2_core_foundation::{CFDictionary, CFNumber, CFRetained};
use objc2_core_graphics::kCGColorSpaceSRGB;
use objc2_io_surface::{
    kIOSurfaceAllocSize, kIOSurfaceBytesPerElement, kIOSurfaceBytesPerRow, kIOSurfaceColorSpace,
    kIOSurfaceHeight, kIOSurfacePixelFormat, kIOSurfaceWidth, IOSurfaceLockOptions, IOSurfaceRef,
};
use std::{ptr::NonNull, slice, sync::Arc};

#[derive(Debug, Clone)]
pub(super) struct SendSurface(pub CFRetained<IOSurfaceRef>);
// SAFETY: IOSurface lifetime, queries and lock/unlock are thread-safe. Mutable
// CPU access is only exposed by the unique Surface buffer borrow after checking
// WindowServer use and retaining queued mappings through their native commit;
// no writable aliases exist.
unsafe impl Send for SendSurface {}

#[derive(Debug)]
struct Slot {
    surface: SendSurface,
    lease: Arc<()>,
    /// Damage produced since this IOSurface last became the submitted buffer.
    /// Chromium keeps this as an SkRegion on each SoftwareOutputDeviceMac
    /// buffer. The queue is at most four entries, so retaining the source
    /// rectangles directly keeps the same semantics without another graphics
    /// type at this platform boundary.
    accumulated_damage: Vec<DamageRect>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DamageRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl DamageRect {
    fn full(width: usize, height: usize) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    fn right(self) -> usize {
        self.x.saturating_add(self.width)
    }

    fn bottom(self) -> usize {
        self.y.saturating_add(self.height)
    }

    fn clipped(self, width: usize, height: usize) -> Self {
        let right = self.right().min(width);
        let bottom = self.bottom().min(height);
        let x = self.x.min(right);
        let y = self.y.min(bottom);
        Self {
            x,
            y,
            width: right - x,
            height: bottom - y,
        }
    }

    fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// Return `self - cut` as the same at-most-four rectangles used by an
    /// SkRegion difference for one rectangular damage input.
    fn subtract(self, cut: Self) -> Vec<Self> {
        let left = self.x.max(cut.x);
        let top = self.y.max(cut.y);
        let right = self.right().min(cut.right());
        let bottom = self.bottom().min(cut.bottom());
        if left >= right || top >= bottom {
            return (!self.is_empty()).then_some(self).into_iter().collect();
        }
        let mut result = Vec::with_capacity(4);
        let mut push = |x, y, width, height| {
            if width != 0 && height != 0 {
                result.push(Self {
                    x,
                    y,
                    width,
                    height,
                });
            }
        };
        push(self.x, self.y, self.width, top - self.y);
        push(self.x, bottom, self.width, self.bottom() - bottom);
        push(self.x, top, left - self.x, bottom - top);
        push(right, top, self.right() - right, bottom - top);
        result
    }
}

#[derive(Debug, Default)]
pub(super) struct TargetPool {
    buffers: Vec<Slot>,
    last_submitted: Option<u32>,
}
fn error(message: &str) -> SoftBufferError {
    SoftBufferError::PlatformError(Some(message.into()), None)
}
impl TargetPool {
    pub fn acquire(&mut self, width: usize, height: usize) -> Result<Mapping, SoftBufferError> {
        self.acquire_with_damage_rects(width, height, &[DamageRect::full(width, height)])
    }

    pub fn acquire_with_damage(
        &mut self,
        width: usize,
        height: usize,
        damage: DamageRect,
    ) -> Result<Mapping, SoftBufferError> {
        self.acquire_with_damage_rects(width, height, &[damage])
    }

    pub fn acquire_with_damage_rects(
        &mut self,
        width: usize,
        height: usize,
        damage: &[DamageRect],
    ) -> Result<Mapping, SoftBufferError> {
        // Chromium gfx::CreateIOSurface reserves even width/height, then asks
        // IOSurface to align row pitch and allocation size. Visible size stays
        // unchanged. CPU mappings expose only the visible rows plus padding.
        let allocation_width = width
            .checked_add(1)
            .ok_or_else(|| error("IOSurface width overflow"))?
            & !1;
        let allocation_height = height
            .checked_add(1)
            .ok_or_else(|| error("IOSurface height overflow"))?
            & !1;
        let minimum_stride = allocation_width
            .checked_mul(4)
            .ok_or_else(|| error("IOSurface stride overflow"))?;
        let stride = IOSurfaceRef::align_property(unsafe { kIOSurfaceBytesPerRow }, minimum_stride);
        let len = (stride / 4)
            .checked_mul(height)
            .filter(|&n| n > 0 && n <= isize::MAX as usize / 4)
            .ok_or_else(|| error("IOSurface dimensions overflow"))?;
        if width == 0 || stride < minimum_stride || stride % 4 != 0 {
            return Err(error("invalid IOSurface row alignment"));
        }
        let minimum_alloc_size = stride
            .checked_mul(allocation_height)
            .filter(|&n| n <= isize::MAX as usize)
            .ok_or_else(|| error("IOSurface allocation overflow"))?;
        let alloc_size =
            IOSurfaceRef::align_property(unsafe { kIOSurfaceAllocSize }, minimum_alloc_size);
        if alloc_size < minimum_alloc_size || alloc_size > isize::MAX as usize {
            return Err(error("IOSurface allocation overflow"));
        }
        self.buffers
            .retain(|s| s.surface.0.width() == width && s.surface.0.height() == height);
        let previous = self.buffers.last().map(|slot| slot.surface.clone());
        let reusable = self
            .buffers
            .iter()
            .position(|s| Arc::strong_count(&s.lease) == 1 && !s.surface.0.is_in_use());
        if let Some(index) = reusable {
            let slot = self.buffers.remove(index);
            self.buffers.push(slot);
        } else {
            let values = [
                width as i64,
                height as i64,
                4,
                stride as i64,
                alloc_size as i64,
                u32::from_be_bytes(*b"BGRA") as i64,
            ];
            let numbers = values.map(CFNumber::new_i64);
            let keys = unsafe {
                [
                    kIOSurfaceWidth,
                    kIOSurfaceHeight,
                    kIOSurfaceBytesPerElement,
                    kIOSurfaceBytesPerRow,
                    kIOSurfaceAllocSize,
                    kIOSurfacePixelFormat,
                ]
            };
            let refs = numbers.each_ref().map(|n| &**n);
            let props = CFDictionary::from_slices(&keys, &refs);
            // SAFETY: dictionary keys match initialized numeric property types.
            let surface = SendSurface(
                unsafe { IOSurfaceRef::new(props.as_opaque()) }
                    .ok_or_else(|| error("IOSurfaceCreate failed"))?,
            );
            // Match Chromium's gfx::IOSurfaceSetColorSpace(CreateSRGB()).
            // Untagged bytes are otherwise treated as display-native RGB, so
            // CSS sRGB colors bypass the display transform on wide-gamut Macs.
            unsafe {
                surface
                    .0
                    .set_value(kIOSurfaceColorSpace, kCGColorSpaceSRGB.as_ref());
            }
            if surface.0.width() != width
                || surface.0.height() != height
                || surface.0.bytes_per_row() != stride
                || surface.0.alloc_size() < minimum_alloc_size
            {
                return Err(error("unsupported IOSurface pitch"));
            }
            if self.buffers.len() >= 4 {
                self.buffers.remove(0);
            }
            let lease = Arc::new(());
            self.buffers.push(Slot {
                surface: surface.clone(),
                lease: lease.clone(),
                accumulated_damage: vec![DamageRect::full(width, height)],
            });
        }
        let damage: Vec<_> = damage
            .iter()
            .map(|damage| damage.clipped(width, height))
            .filter(|damage| !damage.is_empty())
            .collect();
        if !damage.is_empty() {
            for slot in &mut self.buffers {
                slot.accumulated_damage.extend(damage.iter().copied());
            }
        }
        let current = self.buffers.last_mut().expect("acquired IOSurface slot");
        let accumulated = std::mem::take(&mut current.accumulated_damage);
        let surface = current.surface.clone();
        let lease = current.lease.clone();
        let mut copy_rects = accumulated;
        for damage in &damage {
            copy_rects = copy_rects
                .into_iter()
                .flat_map(|rect| rect.subtract(*damage))
                .collect();
        }
        // SAFETY: no external CPU/GPU accesses to this not-in-use mapping. A
        // queued target retains an extra lease until its CA transaction commits.
        if unsafe {
            surface
                .0
                .lock(IOSurfaceLockOptions::AvoidSync, std::ptr::null_mut())
        } != 0
        {
            return Err(error("IOSurfaceLock failed"));
        }
        let ptr = surface.0.base_address().cast::<u32>();
        if ptr.as_ptr() as usize % std::mem::align_of::<u32>() != 0 {
            unsafe {
                surface
                    .0
                    .unlock(IOSurfaceLockOptions::AvoidSync, std::ptr::null_mut());
            }
            return Err(error("unaligned IOSurface mapping"));
        }
        let mut mapping = Mapping {
            surface,
            _lease: lease,
            ptr,
            len,
            row_stride: stride / 4,
            locked: true,
            retained: previous.is_some(),
        };
        if !copy_rects.is_empty() && previous.is_some() {
            let previous = previous.expect("checked previous IOSurface");
            if previous.0.id() == mapping.surface.0.id() {
                return Err(error(
                    "reused IOSurface unexpectedly requires a damage copy",
                ));
            }
            mapping.copy_from(&previous, &copy_rects)?;
        }
        Ok(mapping)
    }
    /// Record the surface installed in the CALayer and report whether the
    /// layer already names this IOSurface. Chromium uses this distinction to
    /// call `setContentsChanged` when reusing the same software buffer.
    pub fn submitted(&mut self, id: u32) -> bool {
        let unchanged = self.last_submitted == Some(id);
        self.last_submitted = Some(id);
        unchanged
    }
}
#[derive(Debug)]
pub(super) struct Mapping {
    surface: SendSurface,
    _lease: Arc<()>,
    ptr: NonNull<u32>,
    len: usize,
    row_stride: usize,
    locked: bool,
    retained: bool,
}
// SAFETY: same exclusive mapping contract as SendSurface, retained through Drop.
unsafe impl Send for Mapping {}
impl Mapping {
    pub fn row_stride(&self) -> usize {
        self.row_stride
    }
    pub fn pixels(&self) -> &[u32] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
    pub fn retained(&self) -> bool {
        self.retained
    }
    fn copy_from(
        &mut self,
        previous: &SendSurface,
        rects: &[DamageRect],
    ) -> Result<(), SoftBufferError> {
        if unsafe {
            previous.0.lock(
                IOSurfaceLockOptions::ReadOnly | IOSurfaceLockOptions::AvoidSync,
                std::ptr::null_mut(),
            )
        } != 0
        {
            return Err(error("previous IOSurface read lock failed"));
        }
        let source = previous.0.base_address().cast::<u32>();
        let source_stride = previous.0.bytes_per_row() / 4;
        for rect in rects {
            for row in 0..rect.height {
                let source =
                    unsafe { source.as_ptr().add((rect.y + row) * source_stride + rect.x) };
                let target = unsafe {
                    self.ptr
                        .as_ptr()
                        .add((rect.y + row) * self.row_stride + rect.x)
                };
                unsafe { std::ptr::copy_nonoverlapping(source, target, rect.width) };
            }
        }
        if unsafe {
            previous.0.unlock(
                IOSurfaceLockOptions::ReadOnly | IOSurfaceLockOptions::AvoidSync,
                std::ptr::null_mut(),
            )
        } != 0
        {
            return Err(error("previous IOSurface read unlock failed"));
        }
        Ok(())
    }
    pub fn finish(mut self) -> Result<SendSurface, SoftBufferError> {
        self.unlock()?;
        Ok(self.surface.clone())
    }
    /// Keep CPU ownership until the queued window-thread transaction consumes
    /// the buffer. WindowServer's in-use flag alone cannot protect a buffer
    /// that has not reached Core Animation yet.
    pub fn finish_with_lease(mut self) -> Result<(SendSurface, Arc<()>), SoftBufferError> {
        self.unlock()?;
        Ok((self.surface.clone(), self._lease.clone()))
    }
    fn unlock(&mut self) -> Result<(), SoftBufferError> {
        if self.locked {
            if unsafe {
                self.surface
                    .0
                    .unlock(IOSurfaceLockOptions::AvoidSync, std::ptr::null_mut())
            } != 0
            {
                return Err(error("IOSurfaceUnlock failed"));
            }
            self.locked = false;
        }
        Ok(())
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        let _ = self.unlock();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn odd_pitch_and_simultaneous_mapping_ownership() {
        for (width, height) in [(1, 1), (319, 17), (1281, 9), (1602, 3), (1654, 5)] {
            let mut pool = TargetPool::default();
            let mut first = pool.acquire(width, height).unwrap();
            let mut second = pool.acquire(width, height).unwrap();
            assert_ne!(first.surface.0.id(), second.surface.0.id());
            assert_eq!(first.pixels().len(), first.row_stride() * height);
            assert_eq!(
                first.row_stride() * 4,
                IOSurfaceRef::align_property(
                    unsafe { kIOSurfaceBytesPerRow },
                    ((width + 1) & !1) * 4
                )
            );
            assert!(first.surface.0.alloc_size() >= first.row_stride() * 4 * ((height + 1) & !1));
            assert!(first.row_stride() >= width);
            first.pixels_mut().fill(0xff112233);
            second.pixels_mut().fill(0xffabcdef);
            assert!(first.pixels().iter().all(|&p| p == 0xff112233));
            let first_id = first.surface.0.id();
            drop(first);
            let reused = pool.acquire(width, height).unwrap();
            assert_eq!(reused.surface.0.id(), first_id);
            assert!(reused.pixels().iter().all(|&p| p == 0xff112233));
        }
    }
    #[test]
    fn queued_surface_stays_unavailable_until_native_commit() {
        let mut pool = TargetPool::default();
        let (queued, lease) = pool.acquire(319, 7).unwrap().finish_with_lease().unwrap();
        let id = queued.0.id();
        // A queued surface is not yet in WindowServer use. Its lease must
        // prevent reuse even if newer frames have been submitted already.
        let other = pool.acquire(319, 7).unwrap().finish().unwrap();
        pool.submitted(other.0.id());
        assert_ne!(other.0.id(), id);
        let third = pool.acquire(319, 7).unwrap().finish().unwrap();
        assert_ne!(third.0.id(), id);
        drop(lease);
        let reusable = pool.acquire(319, 7).unwrap();
        assert_eq!(reusable.surface.0.id(), id);
    }

    #[test]
    fn rotating_target_preserves_pixels_outside_current_damage() {
        let mut pool = TargetPool::default();
        let mut first = pool.acquire(8, 6).unwrap();
        first.pixels_mut().fill(0xff11_2233);
        let (_submitted, queued_lease) = first.finish_with_lease().unwrap();

        let damage = DamageRect {
            x: 2,
            y: 1,
            width: 3,
            height: 2,
        };
        let second = pool.acquire_with_damage(8, 6, damage).unwrap();
        assert!(second.retained());
        for y in 0..6 {
            for x in 0..8 {
                if x < 2 || x >= 5 || y < 1 || y >= 3 {
                    assert_eq!(second.pixels()[y * second.row_stride() + x], 0xff11_2233);
                }
            }
        }
        drop(queued_lease);
    }

    #[test]
    fn first_partial_frame_after_resize_is_unretained() {
        let mut pool = TargetPool::default();
        let old = pool.acquire(8, 6).unwrap().finish().unwrap();
        pool.submitted(old.0.id());
        let resized = pool
            .acquire_with_damage(
                10,
                7,
                DamageRect {
                    x: 2,
                    y: 1,
                    width: 3,
                    height: 2,
                },
            )
            .unwrap();
        assert!(!resized.retained());
    }

    #[test]
    fn submitted_target_can_be_reused_after_native_release() {
        let mut pool = TargetPool::default();
        let submitted = pool.acquire(319, 7).unwrap().finish().unwrap();
        assert!(!pool.submitted(submitted.0.id()));
        let next = pool.acquire(319, 7).unwrap();
        assert_eq!(next.surface.0.id(), submitted.0.id());
        drop(next);
        assert!(pool.submitted(submitted.0.id()));
        let mappings: Vec<_> = (0..6).map(|_| pool.acquire(319, 7).unwrap()).collect();
        assert_eq!(pool.buffers.len(), 4);
        let mut ids: Vec<_> = mappings.iter().map(|m| m.surface.0.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 6);
        drop(mappings);
        let resized = pool.acquire(321, 9).unwrap();
        assert_eq!(resized.pixels().len(), resized.row_stride() * 9);
        assert_eq!(pool.buffers.len(), 1);
        assert_eq!(
            submitted.0.width(),
            319,
            "retained old resource survives resize"
        );
    }
    #[test]
    fn invalid_dimensions_are_rejected_before_allocation() {
        let mut pool = TargetPool::default();
        for (w, h) in [(0, 1), (1, 0), (usize::MAX, 2), (1, usize::MAX)] {
            assert!(pool.acquire(w, h).is_err());
        }
    }
}
