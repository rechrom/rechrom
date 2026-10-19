//! UI-resource external raster primitives. No UI resource selection, sizing,
//! lifetime or source host policy is implemented here; those remain in HostImpl.
use super::{
    host_resources::{SourceBitmap, UIImageInfo, UISurface},
    resources::{CpuColorSpace, CpuSharedImage},
};
use crate::adapter::{
    geometry::Size, layer_tree_host_impl::HostUISharedImageUsage, resource_types::SharedImageFormat,
};
/// The CPU raster/Skia allocation owner implements this capability. Borrowed
/// surfaces must retain the exact caller pixel slice and its stride; the trait
/// does not permit a compositor-side pixel copy or fabricated image/surface.
pub trait NativeUiRasterServices: Send + Sync {
    fn MakeN32Premul(&self, size: Size) -> UIImageInfo;
    fn NewSourceBitmap<'a>(&self) -> SourceBitmap<'a>;
    fn WrapPixels<'a>(
        &self,
        info: UIImageInfo,
        pixels: &'a mut [u8],
        row_bytes: usize,
    ) -> Option<UISurface<'a>>;
    fn RasterSurface<'a>(&self, info: UIImageInfo) -> Option<UISurface<'a>>;
    fn CreateSoftwareSharedImage(
        &self,
        format: SharedImageFormat,
        size: Size,
        color: &CpuColorSpace,
        usage: HostUISharedImageUsage,
        label: &'static str,
    ) -> Option<CpuSharedImage>;
    fn CreateSharedImageWithPixels(
        &self,
        format: SharedImageFormat,
        size: Size,
        color: &CpuColorSpace,
        usage: HostUISharedImageUsage,
        label: &'static str,
        pixels: &[u8],
    ) -> Option<CpuSharedImage>;
}
use super::{
    engine_factory::NativeFactoryCapabilities,
    host_resources::SharedMapping,
    resources::{CpuResourceSource, CpuSyncToken, CpuSynchronizationType, CpuTransferableResource},
};
use crate::{
    adapter::{geometry::SizeF, layer_tree_host_impl::*, layer_tree_impl::ResourceId},
    NativeCompositorTypes,
};
use std::sync::Arc;
pub(crate) struct NativeUiServices {
    native: Arc<NativeFactoryCapabilities>,
}
impl NativeUiServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self { native }
    }
    fn CheckSink(&self, sink: HostImplFrameSinkId) {
        assert_eq!(
            self.native.frame_sink.lock().expect("CPU frame sink").id(),
            sink
        )
    }
}
impl HostUIResourceServices<NativeCompositorTypes> for NativeUiServices {
    fn TraceUIResourceEvent(&mut self, name: &'static str) {
        self.native.events.Emit("cc", name.into())
    }
    // This endpoint is specifically the CPU FrameSink; its context-provider and
    // GPU context-provider/scanout capabilities are absent, as in Core's CPU path.
    fn UIResourceContextProviderExists(&self, sink: HostImplFrameSinkId) -> bool {
        self.CheckSink(sink);
        false
    }
    // Source: cc/trees/layer_tree_host_impl.cc:5680-5690. This requires the CPU SharedImageInterface too. The
    // injected raster capability is that actual allocator, even without GPU.
    fn UIResourceSharedImageInterfaceExists(&self, sink: HostImplFrameSinkId) -> bool {
        self.CheckSink(sink);
        true
    }
    fn UIResourceSupportsScanoutSharedImages(&self, sink: HostImplFrameSinkId) -> bool {
        self.CheckSink(sink);
        false
    }
    fn CanCreateNativeBufferForFormat(&self, _format: SharedImageFormat) -> bool {
        false
    }
    // Source dependency: ui/gfx/geometry/size.cc:83-86; size_conversions.cc:15-17.
    fn ScaleToCeiledSize(&self, size: Size, scale: f32) -> Size {
        {
            if scale == 1.0 {
                return size;
            }
            Size::new(
                ((size.width() as f32) * scale).ceil() as i32,
                ((size.height() as f32) * scale).ceil() as i32,
            )
        }
    }
    fn SRGBColorSpace(&self) -> CpuColorSpace {
        CpuColorSpace::CreateSRGB()
    }
    fn CreateUIResourceSharedImageForSoftwareCompositor(
        &mut self,
        sink: HostImplFrameSinkId,
        format: SharedImageFormat,
        size: Size,
        color: &CpuColorSpace,
        usage: HostUISharedImageUsage,
        label: &'static str,
    ) -> Option<CpuSharedImage> {
        self.CheckSink(sink);
        self.native
            .ui_raster
            .CreateSoftwareSharedImage(format, size, color, usage, label)
    }
    fn MapUIResourceSharedImage(&mut self, image: &CpuSharedImage) -> SharedMapping {
        SharedMapping::FromCpuWriteMapping(
            image
                .MapCpuWrite()
                .expect("UI shared-image map must succeed"),
        )
    }
    fn CreateUIResourceSharedImage(
        &mut self,
        sink: HostImplFrameSinkId,
        format: SharedImageFormat,
        size: Size,
        color: &CpuColorSpace,
        usage: HostUISharedImageUsage,
        label: &'static str,
        pixels: &[u8],
    ) -> Option<CpuSharedImage> {
        self.CheckSink(sink);
        self.native
            .ui_raster
            .CreateSharedImageWithPixels(format, size, color, usage, label, pixels)
    }
    fn MakeN32Premul(&self, size: Size) -> UIImageInfo {
        self.native.ui_raster.MakeN32Premul(size)
    }
    fn NewUIResourceSourceBitmap<'a>(&mut self) -> SourceBitmap<'a> {
        self.native.ui_raster.NewSourceBitmap()
    }
    fn WrapUIResourcePixels<'a>(
        &mut self,
        info: UIImageInfo,
        pixels: &'a mut [u8],
        row_bytes: usize,
    ) -> Option<UISurface<'a>> {
        self.native.ui_raster.WrapPixels(info, pixels, row_bytes)
    }
    fn RasterUIResourceSurface<'a>(&mut self, info: UIImageInfo) -> Option<UISurface<'a>> {
        self.native.ui_raster.RasterSurface(info)
    }
    // Source: components/viz/common/resources/transferable_resource.cc:28-41.
    fn MakeUITransferableResource(
        &mut self,
        image: CpuSharedImage,
        sync_token: CpuSyncToken,
    ) -> CpuTransferableResource {
        CpuTransferableResource {
            id: ResourceId::default(),
            shared_image: image,
            source: CpuResourceSource::Ui,
            sync_token,
            synchronization_type: CpuSynchronizationType::SyncToken,
        }
    }
    fn OnCanDrawStateChanged(&mut self, delegate: HostImplDelegateId, can_draw: bool) {
        self.native.resource_effects.Push(
            super::native_resource_events::NativeResourceEffect::CanDrawStateChanged {
                delegate,
                can_draw,
            },
        )
    }
    fn NotifyLatencyInfoSwapPromiseMonitor(
        &mut self,
        monitor: HostImplLatencyMonitorId,
    ) -> Option<Box<super::host_owners::SwapPromise<super::skia::SkRegion>>> {
        self.native.latency_monitors.Notify(monitor)
    }
    // Source: cc/trees/task_runner_provider.cc:14-19. A single-threaded
    // renderer has no independent impl runner; do not fabricate Some(main).
    fn OptionalImplThreadTaskRunner(
        &self,
        provider: HostImplTaskRunnerProviderId,
    ) -> Option<HostImplTaskRunnerId> {
        use crate::adapter::layer_tree_host_impl::HostImplTaskRunnerServices;
        self.native
            .runners
            .HasImplThread(provider)
            .then(|| self.native.runners.ImplThreadTaskRunner(provider))
    }
    fn RunsTasksInCurrentSequence(&self, runner: HostImplTaskRunnerId) -> bool {
        self.native
            .runners
            .Resolve(runner)
            .expect("known native runner")
            .RunsTasksInCurrentSequence()
    }
}
