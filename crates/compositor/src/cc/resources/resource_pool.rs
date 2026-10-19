// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/resources/resource_pool.h:5-52
// Source: cc/resources/resource_pool.cc:5-44
// Includes, namespaces, export/feature declarations map to modules and the
// typed live feature query (InvalidateResourcesOnSizeChange defaults externally
// to the source disabled-by-default feature). No GPU implementation lives here.
use crate::adapter::{
    geometry::{Rect, Size},
    layer_tree_impl::ResourceId,
    resource_pool::*,
    resource_types::{SharedImageFormat, TileDrawResource},
    time::{TimeDelta, TimeTicks},
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

// Source: cc/resources/resource_pool.h:55-62
pub struct SizeComparator;
impl SizeComparator {
    pub fn Compare(a: &Size, b: &Size) -> bool {
        if a.width() != b.width() {
            return a.width() < b.width();
        }
        a.height() < b.height()
    }
}
fn size_key(size: &Size) -> (i32, i32) {
    (size.width(), size.height())
}

// Source: cc/resources/resource_pool.h:69-157
// These access cells protect one resident backing, not a pool/host snapshot.
// SharedImage construction happens outside the cell, so worker creation does
// not block compositor metadata/flag queries. Access follows the source's
// can_access_shared_image_on_compositor_thread protocol.
pub struct Backing<B: ResourcePoolTypes> {
    shared_image_: Mutex<Option<B::SharedImage>>,
    size_: Size,
    format_: SharedImageFormat,
    color_space_: B::ColorSpace,
    can_access_shared_image_on_compositor_thread_: AtomicBool,
    mailbox_sync_token_: Mutex<B::SyncToken>,
    wait_on_fence_required_: AtomicBool,
    returned_sync_token_: Mutex<B::SyncToken>,
    is_using_raw_draw_: AtomicBool,
}
impl<B: ResourcePoolTypes> Backing<B> {
    // Source: cc/resources/resource_pool.h:72-75,125-156
    // Source: cc/resources/resource_pool.cc:46-49
    pub fn new(size: &Size, format: SharedImageFormat, color_space: &B::ColorSpace) -> Self {
        Self {
            shared_image_: Mutex::new(None),
            size_: *size,
            format_: format,
            color_space_: color_space.clone(),
            can_access_shared_image_on_compositor_thread_: AtomicBool::new(true),
            mailbox_sync_token_: Mutex::new(B::SyncToken::default()),
            wait_on_fence_required_: AtomicBool::new(false),
            returned_sync_token_: Mutex::new(B::SyncToken::default()),
            is_using_raw_draw_: AtomicBool::new(false),
        }
    }
    // Source: cc/resources/resource_pool.h:77-81
    // Source: cc/resources/resource_pool.cc:64-72
    pub fn CreateSharedImage(
        &self,
        sii: &dyn PoolSharedImageInterface<B>,
        usage: SharedImageUsageSet,
        debug_label: &str,
    ) {
        let image = sii.CreateSharedImage(
            SharedImageDescriptor {
                format: self.format(),
                size: self.size(),
                color_space: self.color_space(),
                usage,
                debug_label,
            },
            None,
        );
        *self.shared_image_.lock().unwrap() = image;
        assert!(self.shared_image().is_some());
    }
    // Source: cc/resources/resource_pool.h:83-86
    // Source: cc/resources/resource_pool.cc:91-98
    pub fn CreateSharedImageForSoftwareCompositor(
        &self,
        sii: &dyn PoolSharedImageInterface<B>,
        debug_label: &str,
    ) {
        let image = sii.CreateSharedImageForSoftwareCompositor(SharedImageDescriptor {
            format: self.format(),
            size: self.size(),
            color_space: self.color_space(),
            usage: SharedImageUsageSet::CPU_WRITE_ONLY,
            debug_label,
        });
        *self.shared_image_.lock().unwrap() = image;
        assert!(self.shared_image().is_some());
    }
    // Source: cc/resources/resource_pool.h:88-104
    // Source: cc/resources/resource_pool.cc:100-109
    pub fn CreateMappableSharedImage(
        &self,
        sii: &dyn PoolSharedImageInterface<B>,
        usage: SharedImageUsageSet,
        debug_label: &str,
        buffer_usage: B::BufferUsage,
    ) -> bool {
        let image = sii.CreateSharedImage(
            SharedImageDescriptor {
                format: self.format(),
                size: self.size(),
                color_space: self.color_space(),
                usage,
                debug_label,
            },
            Some(buffer_usage),
        );
        *self.shared_image_.lock().unwrap() = image;
        self.shared_image().is_some()
    }
    // Source: cc/resources/resource_pool.h:106
    // Source: cc/resources/resource_pool.cc:74-76
    pub fn CreateSharedImageForTesting(&self, sii: &dyn PoolSharedImageInterface<B>) {
        self.CreateSharedImageForTestingWithTarget(sii, 0x0de1); // GL_TEXTURE_2D.
    }
    // Source: cc/resources/resource_pool.h:107
    // Source: cc/resources/resource_pool.cc:78-89
    pub fn CreateSharedImageForTestingWithTarget(
        &self,
        sii: &dyn PoolSharedImageInterface<B>,
        texture_target: u32,
    ) {
        let image = sii.CreateSharedImageForTesting(
            SharedImageTestDescriptor {
                format: self.format(),
                size: self.size(),
                color_space: self.color_space(),
                surface_origin: TestSurfaceOrigin::TopLeft,
                alpha_type: TestAlphaType::Opaque,
                usage: SharedImageUsageSet::SCANOUT,
            },
            texture_target,
        );
        *self.shared_image_.lock().unwrap() = image;
    }
    // Source: cc/resources/resource_pool.h:109-123
    pub fn clear_shared_image(&self) {
        *self.shared_image_.lock().unwrap() = None;
    }
    pub fn shared_image(&self) -> Option<B::SharedImage> {
        self.shared_image_.lock().unwrap().clone()
    }
    // Source: cc/resources/resource_pool.h:125-145
    pub fn can_access_shared_image_on_compositor_thread(&self) -> bool {
        self.can_access_shared_image_on_compositor_thread_
            .load(Ordering::Relaxed)
    }
    pub fn set_can_access_shared_image_on_compositor_thread(&self, value: bool) {
        self.can_access_shared_image_on_compositor_thread_
            .store(value, Ordering::Relaxed);
    }
    pub fn mailbox_sync_token(&self) -> B::SyncToken {
        self.mailbox_sync_token_.lock().unwrap().clone()
    }
    pub fn set_mailbox_sync_token(&self, value: B::SyncToken) {
        *self.mailbox_sync_token_.lock().unwrap() = value;
    }
    pub fn wait_on_fence_required(&self) -> bool {
        self.wait_on_fence_required_.load(Ordering::Relaxed)
    }
    pub fn set_wait_on_fence_required(&self, value: bool) {
        self.wait_on_fence_required_.store(value, Ordering::Relaxed);
    }
    pub fn returned_sync_token(&self) -> B::SyncToken {
        self.returned_sync_token_.lock().unwrap().clone()
    }
    pub fn set_returned_sync_token(&self, value: B::SyncToken) {
        *self.returned_sync_token_.lock().unwrap() = value;
    }
    pub fn is_using_raw_draw(&self) -> bool {
        self.is_using_raw_draw_.load(Ordering::Relaxed)
    }
    pub fn set_is_using_raw_draw(&self, value: bool) {
        self.is_using_raw_draw_.store(value, Ordering::Relaxed);
    }
    // Source: cc/resources/resource_pool.h:147-156
    pub fn size(&self) -> &Size {
        &self.size_
    }
    pub fn format(&self) -> &SharedImageFormat {
        &self.format_
    }
    pub fn color_space(&self) -> &B::ColorSpace {
        &self.color_space_
    }
}
impl<B: ResourcePoolTypes> Drop for Backing<B> {
    // Source: cc/resources/resource_pool.cc:51-62
    fn drop(&mut self) {
        let Some(image) = self.shared_image_.get_mut().unwrap().as_ref() else {
            return;
        };
        let returned = self.returned_sync_token_.get_mut().unwrap();
        if returned.HasData() {
            image.UpdateDestructionSyncToken(returned);
        } else {
            let mailbox = self.mailbox_sync_token_.get_mut().unwrap();
            if mailbox.HasData() {
                image.UpdateDestructionSyncToken(mailbox);
            }
        }
        *self.shared_image_.get_mut().unwrap() = None;
    }
}

// Source: cc/resources/resource_pool.h:376-393
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    kUnused,
    kInUse,
    kBusy,
}
// Source: cc/resources/resource_pool.h:326-441
// One resident allocation owns the immutable configuration and mutable fields.
// The pool's lists are authoritative membership; a lease's Arc is only a safe
// lifetime anchor for the source non-owning pointer, never a copied resource.
struct PoolResource<B: ResourcePoolTypes> {
    mutable_: Mutex<PoolResourceMutable<B>>,
    color_space_: B::ColorSpace,
    format_: SharedImageFormat,
    size_: Size,
    unique_id_: usize,
    resource_pool_: ResourcePoolId,
}
struct PoolResourceMutable<B: ResourcePoolTypes> {
    // Native destruction releases debug-name storage before the backing.
    debug_name_: String,
    backing_: Option<Arc<Backing<B>>>,
    content_id_: u64,
    last_usage_: TimeTicks,
    invalidated_rect_: Rect,
    avoid_reuse_: bool,
    resource_id_: ResourceId,
    state_: State,
}
impl<B: ResourcePoolTypes> PoolResource<B> {
    // Source: cc/resources/resource_pool.h:328-333,413-440
    // Source: cc/resources/resource_pool.cc:779-790
    fn new(
        pool: ResourcePoolId,
        unique_id: usize,
        size: &Size,
        format: SharedImageFormat,
        color_space: &B::ColorSpace,
    ) -> Self {
        Self {
            mutable_: Mutex::new(PoolResourceMutable {
                backing_: None,
                content_id_: 0,
                last_usage_: TimeTicks::default(),
                invalidated_rect_: Rect::default(),
                avoid_reuse_: false,
                resource_id_: ResourceId(0),
                debug_name_: String::new(),
                state_: State::kUnused,
            }),
            resource_pool_: pool,
            unique_id_: unique_id,
            size_: *size,
            format_: format,
            color_space_: color_space.clone(),
        }
    }
    // Source: cc/resources/resource_pool.h:335-341,351-363,371-374,392-393
    fn unique_id(&self) -> usize {
        self.unique_id_
    }
    fn size(&self) -> &Size {
        &self.size_
    }
    fn format(&self) -> &SharedImageFormat {
        &self.format_
    }
    fn color_space(&self) -> &B::ColorSpace {
        &self.color_space_
    }
    fn resource_id(&self) -> ResourceId {
        self.mutable_.lock().unwrap().resource_id_
    }
    fn set_resource_id(&self, id: ResourceId) {
        self.mutable_.lock().unwrap().resource_id_ = id;
    }
    fn content_id(&self) -> u64 {
        self.mutable_.lock().unwrap().content_id_
    }
    fn set_content_id(&self, id: u64) {
        self.mutable_.lock().unwrap().content_id_ = id;
    }
    fn last_usage(&self) -> TimeTicks {
        self.mutable_.lock().unwrap().last_usage_
    }
    fn set_last_usage(&self, time: TimeTicks) {
        self.mutable_.lock().unwrap().last_usage_ = time;
    }
    fn invalidated_rect(&self) -> Rect {
        self.mutable_.lock().unwrap().invalidated_rect_
    }
    fn set_invalidated_rect(&self, rect: Rect) {
        self.mutable_.lock().unwrap().invalidated_rect_ = rect;
    }
    fn avoid_reuse(&self) -> bool {
        self.mutable_.lock().unwrap().avoid_reuse_
    }
    fn mark_avoid_reuse(&self) {
        self.mutable_.lock().unwrap().avoid_reuse_ = true;
    }
    fn set_debug_name(&self, name: &str) {
        self.mutable_.lock().unwrap().debug_name_ = name.to_owned();
    }
    // Source: cc/resources/resource_pool.h:372,374
    fn WithDebugName<R>(&self, callback: impl FnOnce(&str) -> R) -> R {
        let resource = self.mutable_.lock().unwrap();
        callback(&resource.debug_name_)
    }
    fn resource_pool(&self) -> ResourcePoolId {
        self.resource_pool_
    }
    fn state(&self) -> State {
        self.mutable_.lock().unwrap().state_
    }
    fn set_state(&self, state: State) {
        self.mutable_.lock().unwrap().state_ = state;
    }
    // Source: cc/resources/resource_pool.h:343-349
    // The short Arc clone is an identity/lifetime borrow of this same backing,
    // not a state snapshot. No external callback is called while a cell is held.
    fn backing(&self) -> Option<Arc<Backing<B>>> {
        self.mutable_.lock().unwrap().backing_.clone()
    }
    fn set_backing(&self, backing: Box<Backing<B>>, pool: &mut ResourcePool<B>) {
        debug_assert!(self.backing().is_none());
        pool.ValidateResident(self);
        self.mutable_.lock().unwrap().backing_ = Some(Arc::from(backing));
        pool.OnBackingAllocated(self);
    }
    // Source: cc/resources/resource_pool.h:395-411
    fn memory_usage(&self) -> usize {
        let Some(backing) = self.backing() else {
            return 0;
        };
        let mut memory_usage = B::EstimatedSizeInBytes(self.format(), self.size());
        if backing.is_using_raw_draw() {
            memory_usage /= 2;
        }
        memory_usage
    }
    // Source: cc/resources/resource_pool.cc:792-824
    fn OnMemoryDump(
        &self,
        pmd: &mut dyn ResourcePoolMemoryDump<B::DumpGuid>,
        tracing_id: i32,
        is_free: bool,
        is_busy: bool,
    ) {
        let dump_name = self.WithDebugName(|debug_name| {
            format!(
                "cc/tile_memory/provider_{}/{}{}resource_{}",
                tracing_id,
                debug_name,
                if debug_name.is_empty() { "" } else { "/" },
                self.unique_id()
            )
        });
        let dump = pmd.CreateAllocatorDump(&dump_name);
        let importance = SharedImageTracingImportance::ClientOwner;
        if let Some(backing) = self.backing() {
            if backing.can_access_shared_image_on_compositor_thread() {
                if let Some(image) = backing.shared_image() {
                    image.OnMemoryDump(pmd, &dump, importance);
                }
            }
        }
        let total_bytes = self.memory_usage() as u64;
        pmd.AddScalar(&dump, "size", "bytes", total_bytes);
        let free_size = if is_free { total_bytes } else { 0 };
        pmd.AddScalar(&dump, "free_size", "bytes", free_size);
        let busy_size = if is_busy { total_bytes } else { 0 };
        pmd.AddScalar(&dump, "busy_size", "bytes", busy_size);
    }
}

// Source: cc/resources/resource_pool.h:159-232
// No Clone: a live lease must be consumed by its original pool. Ordinary Rust
// moves replace the native move constructor; explicit MoveFrom/MoveAssign also
// expose the native moved-from-empty behavior without releasing the resource.
pub struct InUsePoolResource<B: ResourcePoolTypes> {
    resource_: Option<Arc<PoolResource<B>>>,
}
impl<B: ResourcePoolTypes> Default for InUsePoolResource<B> {
    // Source: cc/resources/resource_pool.h:163
    fn default() -> Self {
        Self { resource_: None }
    }
}
impl<B: ResourcePoolTypes> Drop for InUsePoolResource<B> {
    // Source: cc/resources/resource_pool.h:164-166
    fn drop(&mut self) {
        debug_assert!(
            self.resource_.is_none(),
            "Must be returned to ResourcePool to be freed."
        );
    }
}
impl<B: ResourcePoolTypes> InUsePoolResource<B> {
    // Source: cc/resources/resource_pool.h:168-179
    pub fn MoveFrom(other: &mut Self) -> Self {
        Self {
            resource_: other.resource_.take(),
        }
    }
    pub fn MoveAssign(&mut self, other: &mut Self) {
        self.resource_ = other.resource_.take();
    }
    // Source: cc/resources/resource_pool.h:181-194
    pub fn IsValid(&self) -> bool {
        self.resource_.is_some()
    }
    pub fn size(&self) -> &Size {
        self.resource_
            .as_ref()
            .expect("nonempty resource lease")
            .size()
    }
    pub fn format(&self) -> &SharedImageFormat {
        self.resource_
            .as_ref()
            .expect("nonempty resource lease")
            .format()
    }
    pub fn color_space(&self) -> &B::ColorSpace {
        self.resource_
            .as_ref()
            .expect("nonempty resource lease")
            .color_space()
    }
    pub fn resource_id_for_export(&self) -> ResourceId {
        let id = self
            .resource_
            .as_ref()
            .expect("nonempty resource lease")
            .resource_id();
        debug_assert!(id.0 != 0);
        id
    }
    // Source: cc/resources/resource_pool.h:196-199
    pub fn WithBacking<R>(&self, callback: impl FnOnce(Option<&Backing<B>>) -> R) -> R {
        let backing = self
            .resource_
            .as_ref()
            .expect("nonempty resource lease")
            .backing();
        callback(backing.as_deref())
    }
    pub fn set_backing(&self, backing: Box<Backing<B>>, pool: &mut ResourcePool<B>) {
        self.resource_
            .as_ref()
            .expect("nonempty resource lease")
            .set_backing(backing, pool);
    }
    // Source: cc/resources/resource_pool.h:201-205
    // Source: cc/resources/resource_pool.cc:111-125
    pub fn InstallGpuBacking(
        &self,
        sii: &dyn PoolSharedImageInterface<B>,
        is_overlay_candidate: bool,
        debug_label: &str,
        pool: &mut ResourcePool<B>,
    ) {
        let backing = Box::new(Backing::new(
            self.size(),
            *self.format(),
            self.color_space(),
        ));
        let mut flags = SharedImageUsageSet::DISPLAY_READ | SharedImageUsageSet::RASTER_WRITE;
        if is_overlay_candidate {
            flags |= SharedImageUsageSet::SCANOUT;
        }
        backing.CreateSharedImage(sii, flags, debug_label);
        self.set_backing(backing, pool);
    }
    // Source: cc/resources/resource_pool.h:207-211
    // Source: cc/resources/resource_pool.cc:127-135
    pub fn InstallSoftwareBacking(
        &self,
        sii: &dyn PoolSharedImageInterface<B>,
        debug_label: &str,
        pool: &mut ResourcePool<B>,
    ) {
        assert!(self.WithBacking(|backing| backing.is_none()));
        let backing = Box::new(Backing::new(
            self.size(),
            *self.format(),
            self.color_space(),
        ));
        backing.CreateSharedImageForSoftwareCompositor(sii, debug_label);
        self.set_backing(backing, pool);
    }
    // Source: cc/resources/resource_pool.h:213-220
    pub fn memory_usage(&self) -> usize {
        self.resource_
            .as_ref()
            .expect("nonempty resource lease")
            .memory_usage()
    }
    pub fn unique_id_for_testing(&self) -> usize {
        self.resource_
            .as_ref()
            .expect("nonempty resource lease")
            .unique_id()
    }
    // Source: cc/resources/resource_pool.h:222-231
    fn new(resource: Arc<PoolResource<B>>) -> Self {
        debug_assert_eq!(resource.state(), State::kInUse);
        Self {
            resource_: Some(resource),
        }
    }
    fn SetWasFreedByResourcePool(&mut self) {
        self.resource_ = None;
    }
}
// The existing draw-info contract now accepts this canonical move-only lease;
// it retains direct resident metadata access instead of a resource projection.
impl<B: ResourcePoolTypes> TileDrawResource for InUsePoolResource<B> {
    fn EstimatedSizeInBytes(&self, format: &SharedImageFormat, size: &Size) -> usize {
        B::EstimatedSizeInBytes(format, size)
    }
    fn IsValid(&self) -> bool {
        self.IsValid()
    }
    fn resource_id_for_export(&self) -> ResourceId {
        self.resource_id_for_export()
    }
    fn size(&self) -> &Size {
        self.size()
    }
    fn format(&self) -> &SharedImageFormat {
        self.format()
    }
}

// Source: cc/resources/resource_pool.cc:137-167
fn ResourceMeetsSizeRequirements(
    requested_size: &Size,
    actual_size: &Size,
    disallow_non_exact_reuse: bool,
) -> bool {
    const kReuseThreshold: f32 = 2.0;
    if disallow_non_exact_reuse {
        return requested_size == actual_size;
    }
    if actual_size.width() < requested_size.width()
        || actual_size.height() < requested_size.height()
    {
        return false;
    }
    // Source GetArea CHECKs overflow; tile sizes are bounded by max texture size.
    let actual_area = actual_size
        .width()
        .checked_mul(actual_size.height())
        .expect("tile size area overflow") as f32;
    let requested_area = requested_size
        .width()
        .checked_mul(requested_size.height())
        .expect("tile size area overflow") as f32;
    if actual_area / requested_area > kReuseThreshold {
        return false;
    }
    true
}

// Source: cc/resources/resource_pool.h:234-246,482-515
// External provider/context/task/clock/diagnostic capabilities are independent
// services, never Page/Host/Pool backreferences. PoolResource keeps only PoolId;
// its backing-allocation notification explicitly receives this live owner.
// Owning fields are in reverse native order for C++ member destruction.
pub struct ResourcePool<B: ResourcePoolTypes> {
    lifetime_: Arc<()>,
    clock_: PoolClockId,
    flush_evicted_resources_deadline_: TimeTicks,
    // Option preserves a source unique_ptr's transient moved-from null slot;
    // its original container erase still occurs after canonical transfer/delete.
    in_use_resources_: BTreeMap<usize, Option<Arc<PoolResource<B>>>>,
    unused_resources_by_size_: BTreeMap<(i32, i32), usize>,
    busy_resources_: VecDeque<Option<Arc<PoolResource<B>>>>,
    unused_resources_: VecDeque<Arc<PoolResource<B>>>,
    evict_busy_resources_when_unused_: bool,
    evict_expired_resources_pending_: bool,
    peak_total_resource_count_: usize,
    peak_total_memory_usage_bytes_: usize,
    total_resource_count_: usize,
    total_memory_usage_bytes_: usize,
    unused_memory_usage_bytes_: usize,
    max_resource_count_: usize,
    max_memory_usage_bytes_: usize,
    next_resource_unique_id_: usize,
    tracing_track_: Box<dyn PoolTracingTrack>,
    tracing_id_: i32,
    disallow_non_exact_reuse_: bool,
    resource_expiration_delay_: TimeDelta,
    has_context_provider_: bool,
    services_: Arc<dyn ResourcePoolServices<B>>,
    identity_: ResourcePoolId,
}
impl<B: ResourcePoolTypes> ResourcePool<B> {
    // Source: cc/resources/resource_pool.h:64-67
    // Source: cc/resources/resource_pool.cc:179-180
    pub const kDefaultExpirationDelay: TimeDelta = TimeDelta::Seconds(5);
    pub const kDefaultMaxFlushDelay: TimeDelta = TimeDelta::Seconds(1);

    // Source: cc/resources/resource_pool.h:234-246,482-515
    // Source: cc/resources/resource_pool.cc:182-200
    pub fn new(
        identity: ResourcePoolId,
        services: Arc<dyn ResourcePoolServices<B>>,
        has_context_provider: bool,
        expiration_delay: TimeDelta,
        disallow_non_exact_reuse: bool,
    ) -> Self {
        let tracing_id = services.GetNextGlobalTraceId() as i32;
        let tracing_track = services.RegisterTracingTrack("ResourcePool", tracing_id, true);
        let clock = services.DefaultTickClock();
        let result = Self {
            lifetime_: Arc::new(()),
            clock_: clock,
            flush_evicted_resources_deadline_: TimeTicks::Max(),
            // Option preserves a source unique_ptr's transient moved-from null slot;
            // its original container erase still occurs after canonical transfer/delete.
            in_use_resources_: BTreeMap::new(),
            unused_resources_by_size_: BTreeMap::new(),
            busy_resources_: VecDeque::new(),
            unused_resources_: VecDeque::new(),
            evict_busy_resources_when_unused_: false,
            evict_expired_resources_pending_: false,
            peak_total_resource_count_: 0,
            peak_total_memory_usage_bytes_: 0,
            total_resource_count_: 0,
            total_memory_usage_bytes_: 0,
            unused_memory_usage_bytes_: 0,
            max_resource_count_: 0,
            max_memory_usage_bytes_: 0,
            next_resource_unique_id_: 1,
            tracing_track_: tracing_track,
            tracing_id_: tracing_id,
            disallow_non_exact_reuse_: disallow_non_exact_reuse,
            resource_expiration_delay_: expiration_delay,
            has_context_provider_: has_context_provider,
            services_: services,
            identity_: identity,
        };
        result
            .services_
            .RegisterDumpProvider(result.WeakIdentity(), "cc::ResourcePool");
        result
    }
    // Source: cc/resources/resource_pool.h:513-515
    // Safe identity mapping of WeakPtrFactory: tasks contain no owner pointer.
    fn WeakIdentity(&self) -> WeakResourcePoolId {
        WeakResourcePoolId {
            id: self.identity_,
            lifetime: Arc::downgrade(&self.lifetime_),
        }
    }
    // Source: cc/resources/resource_pool.h:224-225,374
    // Borrow-only resident validation replaces native raw owner/pointer checks.
    fn ValidateResident(&self, resource: &PoolResource<B>) {
        assert_eq!(resource.resource_pool(), self.identity_);
        let present = self
            .in_use_resources_
            .get(&resource.unique_id())
            .and_then(Option::as_ref)
            .is_some_and(|resident| std::ptr::eq(resident.as_ref(), resource))
            || self
                .busy_resources_
                .iter()
                .flatten()
                .any(|resident| std::ptr::eq(resident.as_ref(), resource))
            || self
                .unused_resources_
                .iter()
                .any(|resident| std::ptr::eq(resident.as_ref(), resource));
        assert!(present, "resource must be resident in its original pool");
    }
    // Source: cc/resources/resource_pool.h:462-464
    // Source: cc/resources/resource_pool.cc:169-177
    fn DecrementSizeCount(unused_resources_by_size: &mut BTreeMap<(i32, i32), usize>, size: &Size) {
        let key = size_key(size);
        let count = unused_resources_by_size
            .get_mut(&key)
            .expect("unused size has a count");
        *count -= 1;
        if *count == 0 {
            unused_resources_by_size.remove(&key);
        }
    }
    // Source: cc/resources/resource_pool.h:452-455
    // Source: cc/resources/resource_pool.cc:230-294
    fn ReuseResource(
        &mut self,
        size: &Size,
        format: SharedImageFormat,
        color_space: &B::ColorSpace,
    ) -> Option<Arc<PoolResource<B>>> {
        let could_have_exact_match = self.unused_resources_by_size_.contains_key(&size_key(size));
        let prefer_exact_reuse = self.services_.ResourcePoolPreferExactSizeReuse();
        if self.disallow_non_exact_reuse_ && !could_have_exact_match {
            return None;
        }
        let mut first_fit_it = None;
        let mut exact_fit_it = None;
        for (index, resource) in self.unused_resources_.iter().enumerate() {
            debug_assert_eq!(resource.resource_id().0, 0);
            if *resource.format() != format || resource.color_space() != color_space {
                continue;
            }
            if resource.size() == size {
                exact_fit_it = Some(index);
                if prefer_exact_reuse {
                    break;
                }
            }
            if first_fit_it.is_none()
                && ResourceMeetsSizeRequirements(
                    size,
                    resource.size(),
                    self.disallow_non_exact_reuse_,
                )
            {
                first_fit_it = Some(index);
                if !prefer_exact_reuse || !could_have_exact_match {
                    break;
                }
            }
        }
        let index = exact_fit_it.or(first_fit_it)?;
        // The same resident allocation moves from unused membership to in-use.
        // Arc clones here are short lifetime anchors, never metadata copies.
        let resource = Arc::clone(&self.unused_resources_[index]);
        self.in_use_resources_
            .insert(resource.unique_id(), Some(Arc::clone(&resource)));
        self.unused_resources_.remove(index);
        Self::DecrementSizeCount(&mut self.unused_resources_by_size_, resource.size());
        debug_assert!(self.unused_memory_usage_bytes_ >= resource.memory_usage());
        self.unused_memory_usage_bytes_ -= resource.memory_usage();
        debug_assert_eq!(resource.state(), State::kUnused);
        resource.set_state(State::kInUse);
        Some(resource)
    }
    // Source: cc/resources/resource_pool.h:457-460
    // Source: cc/resources/resource_pool.cc:296-316
    fn CreateResource(
        &mut self,
        size: &Size,
        format: SharedImageFormat,
        color_space: &B::ColorSpace,
    ) -> Arc<PoolResource<B>> {
        debug_assert!(B::VerifySizeInBytes(&format, size));
        let unique_id = self.next_resource_unique_id_;
        self.next_resource_unique_id_ += 1;
        let pool_resource = Arc::new(PoolResource::new(
            self.identity_,
            unique_id,
            size,
            format,
            color_space,
        ));
        debug_assert_eq!(pool_resource.memory_usage(), 0);
        self.total_resource_count_ += 1;
        self.peak_total_resource_count_ = self
            .peak_total_resource_count_
            .max(self.total_resource_count_);
        self.in_use_resources_
            .insert(pool_resource.unique_id(), Some(Arc::clone(&pool_resource)));
        pool_resource.set_state(State::kInUse);
        pool_resource
    }
    // Source: cc/resources/resource_pool.h:248-253
    // Source: cc/resources/resource_pool.cc:318-329
    pub fn AcquireResource(
        &mut self,
        size: &Size,
        format: SharedImageFormat,
        color_space: &B::ColorSpace,
        debug_name: &str,
    ) -> InUsePoolResource<B> {
        let resource = match self.ReuseResource(size, format, color_space) {
            Some(resource) => resource,
            None => self.CreateResource(size, format, color_space),
        };
        resource.set_debug_name(debug_name);
        self.UpdateTracingCounters();
        InUsePoolResource::new(resource)
    }
    // Source: cc/resources/resource_pool.h:255-264
    // Source: cc/resources/resource_pool.cc:331-423
    pub fn TryAcquireResourceForPartialRaster(
        &mut self,
        new_content_id: u64,
        new_invalidated_rect: &Rect,
        previous_content_id: u64,
        total_invalidated_rect: &mut Rect,
        raster_color_space: &B::ColorSpace,
        debug_name: &str,
    ) -> InUsePoolResource<B> {
        debug_assert!(new_content_id != 0);
        debug_assert!(previous_content_id != 0);
        *total_invalidated_rect = Rect::default();
        let mut iter_resource_to_return = None;
        let mut minimum_area = 0;
        for (index, resource) in self.unused_resources_.iter().enumerate() {
            if resource.content_id() == previous_content_id {
                if resource.color_space() != raster_color_space {
                    continue;
                }
                Self::UpdateResourceContentIdAndInvalidation(
                    resource,
                    new_content_id,
                    new_invalidated_rect,
                );
                let invalidated = resource.invalidated_rect();
                let area = invalidated
                    .width
                    .checked_mul(invalidated.height)
                    .unwrap_or(i32::MAX);
                if iter_resource_to_return.is_none() || area < minimum_area {
                    iter_resource_to_return = Some(index);
                    minimum_area = area;
                }
            }
        }
        for resource in self.busy_resources_.iter().flatten() {
            if resource.content_id() == previous_content_id {
                Self::UpdateResourceContentIdAndInvalidation(
                    resource,
                    new_content_id,
                    new_invalidated_rect,
                );
            }
        }
        for resource in self.in_use_resources_.values().flatten() {
            if resource.content_id() == previous_content_id {
                Self::UpdateResourceContentIdAndInvalidation(
                    resource,
                    new_content_id,
                    new_invalidated_rect,
                );
            }
        }
        if let Some(index) = iter_resource_to_return {
            let resource = Arc::clone(&self.unused_resources_[index]);
            debug_assert_eq!(resource.resource_id().0, 0);
            resource.set_state(State::kInUse);
            self.in_use_resources_
                .insert(resource.unique_id(), Some(Arc::clone(&resource)));
            self.unused_resources_.remove(index);
            Self::DecrementSizeCount(&mut self.unused_resources_by_size_, resource.size());
            debug_assert!(self.unused_memory_usage_bytes_ >= resource.memory_usage());
            self.unused_memory_usage_bytes_ -= resource.memory_usage();
            *total_invalidated_rect = resource.invalidated_rect();
            resource.set_invalidated_rect(Rect::default());
            resource.set_content_id(0);
            resource.set_debug_name(debug_name);
            self.UpdateTracingCounters();
            return InUsePoolResource::new(resource);
        }
        InUsePoolResource::default()
    }
    // Source: cc/resources/resource_pool.h:443-444
    // Source: cc/resources/resource_pool.cc:425-433
    fn OnBackingAllocated(&mut self, resource: &PoolResource<B>) {
        let size = resource.memory_usage();
        self.total_memory_usage_bytes_ += size;
        self.peak_total_memory_usage_bytes_ = self
            .peak_total_memory_usage_bytes_
            .max(self.total_memory_usage_bytes_);
        if resource.state() == State::kUnused {
            self.unused_memory_usage_bytes_ += size;
        }
        self.UpdateTracingCounters();
    }
    // Source: cc/resources/resource_pool.h:480
    // Source: cc/resources/resource_pool.cc:435-440
    fn UpdateTracingCounters(&self) {
        self.services_.TraceCounter(
            self.tracing_id_,
            "TotalMemory",
            self.total_memory_usage_bytes_,
        );
        self.services_
            .TraceCounter(self.tracing_id_, "InUseMemory", self.memory_usage_bytes());
    }
    // Source: cc/resources/resource_pool.h:446-450
    // Source: cc/resources/resource_pool.cc:442-475
    /// Source: cc/resources/resource_pool.cc:499-501. Rust routing of the
    /// source weak-bound release callback, using the actual live pool owner.
    pub(crate) fn RunReleaseCallback(
        &mut self,
        callback: ResourcePoolReleaseCallback,
        sync_token: &B::SyncToken,
        lost: bool,
    ) {
        if !callback.owner.IsValid() { return; }
        assert_eq!(callback.owner.id, self.identity_, "pool callback owner mismatch");
        assert!(callback.owner.lifetime.ptr_eq(&Arc::downgrade(&self.lifetime_)), "pool callback generation mismatch");
        self.OnResourceReleased(callback.unique_id, sync_token, lost);
    }
    pub fn OnResourceReleased(&mut self, unique_id: usize, sync_token: &B::SyncToken, lost: bool) {
        let services = Arc::clone(&self.services_);
        let _trace = services.Trace(ResourcePoolTraceEvent::OnResourceReleased);
        debug_assert!(!self.in_use_resources_.contains_key(&unique_id));
        let busy_it = self
            .busy_resources_
            .iter()
            .position(|resource| {
                resource.as_ref().expect("resident busy owner").unique_id() == unique_id
            })
            .expect("returned resource must still be busy");
        let resource = Arc::clone(self.busy_resources_[busy_it].as_ref().unwrap());
        resource.set_state(State::kUnused);
        if lost || self.evict_busy_resources_when_unused_ || resource.avoid_reuse() {
            // Move the container owner, leaving only this short local anchor.
            let moved = self.busy_resources_[busy_it].take().unwrap();
            drop(resource);
            self.DeleteResource(moved);
            self.busy_resources_.remove(busy_it);
            return;
        }
        resource.set_resource_id(ResourceId(0));
        if self.has_context_provider_ {
            resource
                .backing()
                .expect("exported resource has backing")
                .set_returned_sync_token(sync_token.clone());
        }
        let moved = self.busy_resources_[busy_it].take().unwrap();
        self.DidFinishUsingResource(moved);
        self.busy_resources_.remove(busy_it);
        self.UpdateTracingCounters();
    }
    // Source: cc/resources/resource_pool.h:266-277
    // Source: cc/resources/resource_pool.cc:477-503
    pub fn PrepareForExport(
        &mut self,
        in_use_resource: &InUsePoolResource<B>,
        resource_source: B::ResourceSource,
    ) -> bool {
        let resource = in_use_resource
            .resource_
            .as_ref()
            .expect("nonempty resource lease");
        self.ValidateResident(resource);
        let backing = resource.backing().expect("export requires backing");
        let Some(image) = backing.shared_image() else {
            resource.set_resource_id(ResourceId(0));
            resource.mark_avoid_reuse();
            return false;
        };
        let mut transferable = self.services_.MakeTransferableResource(
            image,
            resource_source,
            backing.mailbox_sync_token(),
        );
        if backing.wait_on_fence_required() {
            transferable.SetGpuCommandsCompletedSynchronization();
        }
        let resource_id = self.services_.ImportResource(
            transferable,
            ResourcePoolReleaseCallback {
                owner: self.WeakIdentity(),
                unique_id: resource.unique_id(),
            },
        );
        resource.set_resource_id(resource_id);
        true
    }
    // Source: cc/resources/resource_pool.h:317
    // Source: cc/resources/resource_pool.cc:505-518
    pub fn NotifyOfViewportSizeChange(&mut self, old_size: Size, new_size: Size) {
        let services = Arc::clone(&self.services_);
        let _trace = services
            .Trace(ResourcePoolTraceEvent::NotifyOfViewportSizeChange { old_size, new_size });
        if self.services_.InvalidateResourcesOnSizeChange() {
            self.InvalidateResources();
        }
    }
    // Source: cc/resources/resource_pool.h:279-282
    // Source: cc/resources/resource_pool.cc:520-535
    pub fn InvalidateResources(&mut self) {
        let services = Arc::clone(&self.services_);
        let _trace = services.Trace(ResourcePoolTraceEvent::InvalidateResources);
        self.unused_resources_by_size_.clear();
        while !self.unused_resources_.is_empty() {
            debug_assert!(
                self.unused_memory_usage_bytes_
                    >= self.unused_resources_.back().unwrap().memory_usage()
            );
            self.unused_memory_usage_bytes_ -=
                self.unused_resources_.back().unwrap().memory_usage();
            let resource = self.unused_resources_.pop_back().unwrap();
            self.DeleteResource(resource);
        }
        debug_assert_eq!(self.unused_memory_usage_bytes_, 0);
        for pool_resource in self.busy_resources_.iter().flatten() {
            pool_resource.mark_avoid_reuse();
        }
        for resource in self.in_use_resources_.values().flatten() {
            resource.mark_avoid_reuse();
        }
    }
    // Source: cc/resources/resource_pool.h:288
    // Source: cc/resources/resource_pool.cc:537-603
    pub fn ReleaseResource(&mut self, mut in_use_resource: InUsePoolResource<B>) {
        let pool_resource = in_use_resource
            .resource_
            .as_ref()
            .expect("nonempty resource lease")
            .clone();
        in_use_resource.SetWasFreedByResourcePool();
        debug_assert_eq!(pool_resource.state(), State::kInUse);
        // The safe identity validation adds no resource-policy decision.
        assert_eq!(pool_resource.resource_pool(), self.identity_);
        let id = pool_resource.unique_id();
        let Some(resident_slot) = self.in_use_resources_.get(&id) else {
            assert!(
                !self
                    .busy_resources_
                    .iter()
                    .flatten()
                    .any(|resource| resource.unique_id() == id)
            );
            assert!(
                !self
                    .unused_resources_
                    .iter()
                    .any(|resource| resource.unique_id() == id)
            );
            unreachable!("released resource is not in any original pool list");
        };
        let resident = resident_slot
            .as_ref()
            .expect("non-null in-use resource owner");
        assert!(
            Arc::ptr_eq(resident, &pool_resource),
            "lease must refer to the original resident allocation"
        );
        pool_resource.set_last_usage(self.services_.NowTicks(self.clock_));
        let resource_id = pool_resource.resource_id();
        // Consume the only lease lifetime anchor before deleting or moving the
        // pool's membership owner; resource destruction stays at source timing.
        drop(pool_resource);
        let resource = self.in_use_resources_.get_mut(&id).unwrap().take().unwrap();
        if resource_id.0 != 0 {
            resource.set_state(State::kBusy);
            self.busy_resources_.push_front(Some(resource));
        } else if resource.avoid_reuse() {
            resource.set_state(State::kUnused);
            self.DeleteResource(resource);
        } else {
            resource.set_state(State::kUnused);
            self.DidFinishUsingResource(resource);
        }
        self.in_use_resources_.remove(&id);
        // Both membership moves and removal of the in-use entry precede this
        // call: RemoveImportedResource may synchronously call OnResourceReleased.
        if resource_id.0 != 0 {
            let services = Arc::clone(&self.services_);
            services.RemoveImportedResource(resource_id, self);
        }
        self.ScheduleEvictExpiredResourcesIn(self.resource_expiration_delay_);
        self.UpdateTracingCounters();
    }
    // Source: cc/resources/resource_pool.h:284-287
    // Source: cc/resources/resource_pool.cc:605-611
    pub fn OnContentReplaced(&mut self, in_use_resource: &InUsePoolResource<B>, content_id: u64) {
        let resource = in_use_resource
            .resource_
            .as_ref()
            .expect("nonempty resource lease");
        self.ValidateResident(resource);
        resource.set_content_id(content_id);
        resource.set_invalidated_rect(Rect::default());
    }
    // Source: cc/resources/resource_pool.h:290-291
    // Source: cc/resources/resource_pool.cc:613-619
    pub fn SetResourceUsageLimits(
        &mut self,
        max_memory_usage_bytes: usize,
        max_resource_count: usize,
    ) {
        self.max_memory_usage_bytes_ = max_memory_usage_bytes;
        self.max_resource_count_ = max_resource_count;
        self.ReduceResourceUsage();
    }
    // Source: cc/resources/resource_pool.h:292
    // Source: cc/resources/resource_pool.cc:621-639
    pub fn ReduceResourceUsage(&mut self) {
        while !self.unused_resources_.is_empty() {
            if !self.ResourceUsageTooHigh() {
                break;
            }
            let resource = self.unused_resources_.back().unwrap();
            Self::DecrementSizeCount(&mut self.unused_resources_by_size_, resource.size());
            debug_assert!(self.unused_memory_usage_bytes_ >= resource.memory_usage());
            self.unused_memory_usage_bytes_ -= resource.memory_usage();
            let resource = self.unused_resources_.pop_back().unwrap();
            self.DeleteResource(resource);
        }
    }
    // Source: cc/resources/resource_pool.h:293
    // Source: cc/resources/resource_pool.cc:641-647
    pub fn ResourceUsageTooHigh(&self) -> bool {
        if self.total_resource_count_ > self.max_resource_count_ {
            return true;
        }
        if self.total_memory_usage_bytes_ > self.max_memory_usage_bytes_ {
            return true;
        }
        false
    }
    // Source: cc/resources/resource_pool.h:467
    // Source: cc/resources/resource_pool.cc:649-658
    fn DeleteResource(&mut self, resource: Arc<PoolResource<B>>) {
        debug_assert!(self.total_memory_usage_bytes_ >= resource.memory_usage());
        self.total_memory_usage_bytes_ -= resource.memory_usage();
        self.total_resource_count_ -= 1;
        if self.flush_evicted_resources_deadline_ == TimeTicks::Max() {
            self.flush_evicted_resources_deadline_ =
                self.services_.NowTicks(self.clock_) + Self::kDefaultMaxFlushDelay;
        }
        self.UpdateTracingCounters();
        // Parameter owner destruction occurs after counters, as in native.
        drop(resource);
    }
    // Source: cc/resources/resource_pool.h:468-471
    // Source: cc/resources/resource_pool.cc:660-670
    fn UpdateResourceContentIdAndInvalidation(
        resource: &PoolResource<B>,
        new_content_id: u64,
        new_invalidated_rect: &Rect,
    ) {
        let mut updated_invalidated_rect = *new_invalidated_rect;
        if !resource.invalidated_rect().IsEmpty() {
            updated_invalidated_rect.Union(resource.invalidated_rect());
        }
        resource.set_content_id(new_content_id);
        resource.set_invalidated_rect(updated_invalidated_rect);
    }
    // Source: cc/resources/resource_pool.h:466
    // Source: cc/resources/resource_pool.cc:672-677
    fn DidFinishUsingResource(&mut self, resource: Arc<PoolResource<B>>) {
        self.unused_memory_usage_bytes_ += resource.memory_usage();
        *self
            .unused_resources_by_size_
            .entry(size_key(resource.size()))
            .or_default() += 1;
        self.unused_resources_.push_front(resource);
    }
    // Source: cc/resources/resource_pool.h:473-474
    // Source: cc/resources/resource_pool.cc:679-691
    fn ScheduleEvictExpiredResourcesIn(&mut self, time_from_now: TimeDelta) {
        if self.evict_expired_resources_pending_ {
            return;
        }
        self.evict_expired_resources_pending_ = true;
        self.services_
            .PostDelayedEviction(self.WeakIdentity(), time_from_now);
    }
    // Source: cc/resources/resource_pool.h:475
    // Source: cc/resources/resource_pool.cc:693-716
    pub fn EvictExpiredResources(&mut self) {
        self.evict_expired_resources_pending_ = false;
        let current_time = self.services_.NowTicks(self.clock_);
        self.EvictResourcesNotUsedSince(current_time - self.resource_expiration_delay_);
        if self.unused_resources_.is_empty()
            || self.flush_evicted_resources_deadline_ <= current_time
        {
            self.FlushEvictedResources();
        }
        if !self.unused_resources_.is_empty() {
            self.ScheduleEvictExpiredResourcesIn(
                (self.GetUsageTimeForLRUResource() + self.resource_expiration_delay_)
                    .min(self.flush_evicted_resources_deadline_)
                    - current_time,
            );
        }
    }
    // Source: cc/resources/resource_pool.h:476
    // Source: cc/resources/resource_pool.cc:718-733
    fn EvictResourcesNotUsedSince(&mut self, time_limit: TimeTicks) {
        while !self.unused_resources_.is_empty() {
            if self.unused_resources_.back().unwrap().last_usage() > time_limit {
                return;
            }
            let resource = self.unused_resources_.back().unwrap();
            Self::DecrementSizeCount(&mut self.unused_resources_by_size_, resource.size());
            debug_assert!(self.unused_memory_usage_bytes_ >= resource.memory_usage());
            self.unused_memory_usage_bytes_ -= resource.memory_usage();
            let resource = self.unused_resources_.pop_back().unwrap();
            self.DeleteResource(resource);
        }
    }
    // Source: cc/resources/resource_pool.h:478
    // Source: cc/resources/resource_pool.cc:735-743
    fn GetUsageTimeForLRUResource(&self) -> TimeTicks {
        if !self.unused_resources_.is_empty() {
            return self.unused_resources_.back().unwrap().last_usage();
        }
        debug_assert!(!self.busy_resources_.is_empty());
        self.busy_resources_
            .back()
            .unwrap()
            .as_ref()
            .unwrap()
            .last_usage()
    }
    // Source: cc/resources/resource_pool.h:479
    // Source: cc/resources/resource_pool.cc:745-751
    fn FlushEvictedResources(&mut self) {
        self.flush_evicted_resources_deadline_ = TimeTicks::Max();
        if self.has_context_provider_ {
            self.services_.FlushPendingWork();
        }
    }
    // Source: cc/resources/resource_pool.h:300-302
    // Source: cc/resources/resource_pool.cc:753-777
    pub fn OnMemoryDump(
        &self,
        level_of_detail: MemoryDumpLevelOfDetail,
        pmd: &mut dyn ResourcePoolMemoryDump<B::DumpGuid>,
    ) -> bool {
        if level_of_detail == MemoryDumpLevelOfDetail::Background {
            let dump_name = format!("cc/tile_memory/provider_0x{:x}", self.tracing_id_ as u32);
            let dump = pmd.CreateAllocatorDump(&dump_name);
            pmd.AddScalar(
                &dump,
                "size",
                "bytes",
                self.total_memory_usage_bytes_ as u64,
            );
        } else {
            for resource in &self.unused_resources_ {
                resource.OnMemoryDump(pmd, self.tracing_id_, true, false);
            }
            for resource in self.busy_resources_.iter().flatten() {
                resource.OnMemoryDump(pmd, self.tracing_id_, false, true);
            }
            for resource in self.in_use_resources_.values().flatten() {
                resource.OnMemoryDump(pmd, self.tracing_id_, false, false);
            }
        }
        true
    }
    // Source: cc/resources/resource_pool.h:295-298,304-321
    pub fn memory_usage_bytes(&self) -> usize {
        self.total_memory_usage_bytes_ - self.unused_memory_usage_bytes_
    }
    pub fn resource_count(&self) -> usize {
        self.in_use_resources_.len()
    }
    pub fn GetTotalMemoryUsageForTesting(&self) -> usize {
        self.total_memory_usage_bytes_
    }
    pub fn GetTotalResourceCountForTesting(&self) -> usize {
        self.total_resource_count_
    }
    pub fn GetBusyResourceCountForTesting(&self) -> usize {
        self.busy_resources_.len()
    }
    pub fn AllowsNonExactReUseForTesting(&self) -> bool {
        !self.disallow_non_exact_reuse_
    }
    pub fn SetClockForTesting(&mut self, clock: PoolClockId) {
        self.clock_ = clock;
    }
    pub fn tracing_id(&self) -> i32 {
        self.tracing_id_
    }
}
// Source: cc/resources/resource_pool.h:446-450
// The synchronous callback receives only a call-scoped live client. No service
// retains a host/pool reference, and the canonical callback body exists once.
impl<B: ResourcePoolTypes> PoolReleaseClient<B> for ResourcePool<B> {
    fn OnResourceReleased(&mut self, unique_id: usize, sync_token: &B::SyncToken, lost: bool) {
        ResourcePool::OnResourceReleased(self, unique_id, sync_token, lost);
    }
}
impl<B: ResourcePoolTypes> Drop for ResourcePool<B> {
    // Source: cc/resources/resource_pool.h:244
    // Source: cc/resources/resource_pool.cc:202-228
    fn drop(&mut self) {
        self.services_.UnregisterDumpProvider(self.identity_);
        self.services_.HistogramMemoryMediumMb(
            "Compositing.ResourcePool.PeakMemoryUsage",
            self.peak_total_memory_usage_bytes_ / (1024 * 1024),
        );
        if self.peak_total_memory_usage_bytes_ > 0 {
            self.services_.HistogramMemoryMediumMb(
                "Compositing.ResourcePool.PeakMemoryUsage.NonZero",
                self.peak_total_memory_usage_bytes_ / (1024 * 1024),
            );
        }
        self.services_.HistogramCounts10000(
            "Compositing.ResourcePool.PeakResourceCount",
            self.peak_total_resource_count_,
        );
        debug_assert_eq!(self.in_use_resources_.len(), 0);
        while !self.busy_resources_.is_empty() {
            let resource = self.busy_resources_.pop_back().unwrap().unwrap();
            self.DidFinishUsingResource(resource);
        }
        self.SetResourceUsageLimits(0, 0);
        debug_assert_eq!(self.unused_resources_.len(), 0);
        debug_assert_eq!(self.unused_memory_usage_bytes_, 0);
        debug_assert_eq!(self.total_memory_usage_bytes_, 0);
        debug_assert_eq!(self.total_resource_count_, 0);
        // Native WeakPtrFactory is the first member destroyed after this body;
        // lifetime_ is likewise declared first and invalidates pending routes.
    }
}
// Source: cc/resources/resource_pool.h:477
// This pinned revision only declares this unused private method: resource_pool.cc
// has neither definition nor caller. Preserve its signature without inventing an
// algorithm or providing a callable stand-in on the canonical ResourcePool.
trait NativeHasEvictableResourcesDeclaration {
    fn HasEvictableResources(&self) -> bool;
}
// Source: cc/resources/resource_pool.h:323-325
// Native FRIEND_TEST declarations grant no production capability; Rust module
// privacy replaces that declaration without adding repository tests.
// Source: cc/resources/resource_pool.h:516-520
// Source: cc/resources/resource_pool.cc:826
// Namespace and include guard closures map to this Rust module boundary.

impl<B:ResourcePoolTypes> ResourcePool<B>{
    /// The same pool's non-owning identity, used to validate a live release borrow.
    pub(crate) fn identity(&self)->ResourcePoolId{self.identity_}
}

impl<B:ResourcePoolTypes> ResourcePool<B>{
    /// Source: cc/resources/resource_pool.cc:686-690. Resolve the original
    /// weak-bound eviction task to this same resident pool generation.
    pub(crate) fn RunEvictionTask(&mut self,owner:WeakResourcePoolId){
        if !owner.IsValid(){return;}
        assert_eq!(owner.id,self.identity_);
        assert!(owner.lifetime.ptr_eq(&Arc::downgrade(&self.lifetime_)));
        self.EvictExpiredResources();
    }
}
