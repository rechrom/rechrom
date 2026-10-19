//! Rechrom-owned typed resource boundary. No GPU, tiles, or Viz implementation
//! is owned here; these values preserve the source fields used by cc core.

// Source vocabulary: gpu/command_buffer/common/gpu_memory_allocation.h:15-48
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum PriorityCutoff {
    CUTOFF_ALLOW_NOTHING,
    CUTOFF_ALLOW_REQUIRED_ONLY,
    CUTOFF_ALLOW_NICE_TO_HAVE,
    CUTOFF_ALLOW_EVERYTHING,
}
impl PriorityCutoff {
    pub const CUTOFF_LAST: Self = Self::CUTOFF_ALLOW_EVERYTHING;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryAllocation {
    pub bytes_limit_when_visible: u64,
    pub priority_cutoff_when_visible: PriorityCutoff,
}
impl Default for MemoryAllocation {
    fn default() -> Self {
        Self {
            bytes_limit_when_visible: 0,
            priority_cutoff_when_visible: PriorityCutoff::CUTOFF_ALLOW_NOTHING,
        }
    }
}
impl MemoryAllocation {
    pub fn new(bytes_limit_when_visible: u64) -> Self {
        Self {
            bytes_limit_when_visible,
            priority_cutoff_when_visible: PriorityCutoff::CUTOFF_ALLOW_EVERYTHING,
        }
    }
    pub fn Equals(&self, other: &Self) -> bool {
        self == other
    }
}

// Source vocabulary: cc/tiles/tile_priority.h:66-80
pub use crate::cc::tiles::tile_priority::TileMemoryLimitPolicy;

// Source vocabulary: services/viz/public/mojom/compositing/internal/singleplanar_format.mojom
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SingleplanarFormat {
    RGBA_8888,
    RGBA_4444,
    BGRA_8888,
    ALPHA_8,
    BGR_565,
    ETC1,
    R_8,
    RG_88,
    LUMINANCE_F16,
    RGBA_F16,
    R_16,
    RG_1616,
    RGBX_8888,
    BGRX_8888,
    RGBA_1010102,
    BGRA_1010102,
    R_F16,
}

// Source vocabulary: components/viz/common/resources/shared_image_format.h:39-72
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlaneConfig {
    kY_U_V,
    kY_V_U,
    kY_UV,
    kY_UV_A,
    kY_U_V_A,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subsampling {
    k420,
    k422,
    k444,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChannelFormat {
    k8,
    k10,
    k16,
    k16F,
}

/// Retains all format states. External sampling is carried independently:
/// upstream intentionally excludes that field from equality and ordering.
#[derive(Clone, Copy, Debug, Default)]
pub struct SharedImageFormat {
    format: FormatIdentity,
    prefers_external_sampler: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FormatIdentity {
    Unknown,
    SinglePlane(SingleplanarFormat),
    MultiPlane(PlaneConfig, Subsampling, ChannelFormat),
}
impl Default for FormatIdentity {
    fn default() -> Self {
        Self::Unknown
    }
}
impl SharedImageFormat {
    pub const fn SinglePlane(format: SingleplanarFormat) -> Self {
        Self {
            format: FormatIdentity::SinglePlane(format),
            prefers_external_sampler: false,
        }
    }
    pub const fn MultiPlane(
        planes: PlaneConfig,
        subsampling: Subsampling,
        channel: ChannelFormat,
    ) -> Self {
        Self {
            format: FormatIdentity::MultiPlane(planes, subsampling, channel),
            prefers_external_sampler: false,
        }
    }
    pub fn is_single_plane(&self) -> bool {
        matches!(self.format, FormatIdentity::SinglePlane(_))
    }
    pub fn is_multi_plane(&self) -> bool {
        matches!(self.format, FormatIdentity::MultiPlane(..))
    }
    pub fn singleplanar_format(&self) -> SingleplanarFormat {
        match self.format {
            FormatIdentity::SinglePlane(format) => format,
            _ => panic!("not single-plane"),
        }
    }
    pub fn plane_config(&self) -> PlaneConfig {
        match self.format {
            FormatIdentity::MultiPlane(planes, _, _) => planes,
            _ => panic!("not multi-plane"),
        }
    }
    pub fn subsampling(&self) -> Subsampling {
        match self.format {
            FormatIdentity::MultiPlane(_, subsampling, _) => subsampling,
            _ => panic!("not multi-plane"),
        }
    }
    pub fn channel_format(&self) -> ChannelFormat {
        match self.format {
            FormatIdentity::MultiPlane(_, _, channel) => channel,
            _ => panic!("not multi-plane"),
        }
    }
    pub fn PrefersExternalSampler(&self) -> bool {
        self.is_multi_plane() && self.prefers_external_sampler
    }
    pub fn SetPrefersExternalSampler(&mut self) {
        assert!(self.is_multi_plane());
        self.prefers_external_sampler = true;
    }
    pub fn ClearPrefersExternalSampler(&mut self) {
        assert!(self.is_multi_plane() && self.prefers_external_sampler);
        self.prefers_external_sampler = false;
    }

    // Source: components/viz/common/resources/shared_image_format.cc:128-238.
    fn number_of_planes(&self) -> Option<usize> {
        match self.format {
            FormatIdentity::Unknown => None,
            FormatIdentity::SinglePlane(_) => Some(1),
            FormatIdentity::MultiPlane(PlaneConfig::kY_U_V | PlaneConfig::kY_V_U, _, _) => Some(3),
            FormatIdentity::MultiPlane(PlaneConfig::kY_UV, _, _) => Some(2),
            FormatIdentity::MultiPlane(PlaneConfig::kY_UV_A, _, _) => Some(3),
            FormatIdentity::MultiPlane(PlaneConfig::kY_U_V_A, _, _) => Some(4),
        }
    }

    // Source: components/viz/common/resources/shared_image_format.cc:239-274.
    fn is_uv_plane(&self, plane: usize) -> bool {
        match self.plane_config() {
            PlaneConfig::kY_U_V | PlaneConfig::kY_V_U => plane != 0,
            PlaneConfig::kY_UV | PlaneConfig::kY_UV_A => plane == 1,
            PlaneConfig::kY_U_V_A => plane == 1 || plane == 2,
        }
    }
    fn subsampling_scale(&self) -> (usize, usize) {
        match self.subsampling() {
            Subsampling::k420 => (2, 2),
            Subsampling::k422 => (2, 1),
            Subsampling::k444 => (1, 1),
        }
    }
    fn channels_in_plane(&self, plane: usize) -> usize {
        match self.plane_config() {
            PlaneConfig::kY_U_V | PlaneConfig::kY_V_U | PlaneConfig::kY_U_V_A => 1,
            PlaneConfig::kY_UV | PlaneConfig::kY_UV_A => {
                if plane == 1 {
                    2
                } else {
                    1
                }
            }
        }
    }
    fn storage_bytes_per_channel(&self) -> usize {
        match self.channel_format() {
            ChannelFormat::k8 => 1,
            ChannelFormat::k10 | ChannelFormat::k16 | ChannelFormat::k16F => 2,
        }
    }
    fn bytes_per_pixel(&self) -> Option<usize> {
        use SingleplanarFormat::*;
        match self.singleplanar_format() {
            RGBA_F16 => Some(8),
            BGRA_8888 | RGBA_8888 | RGBX_8888 | BGRX_8888 | RGBA_1010102 | BGRA_1010102
            | RG_1616 => Some(4),
            RGBA_4444 | LUMINANCE_F16 | R_F16 | R_16 | BGR_565 | RG_88 => Some(2),
            ALPHA_8 | R_8 => Some(1),
            ETC1 => None,
        }
    }

    // Source: components/viz/common/resources/shared_image_format.cc:157-238.
    pub fn MaybeEstimatedSizeInBytes(&self, size: &super::geometry::Size) -> Option<usize> {
        let width = usize::try_from(size.width())
            .ok()
            .filter(|value| *value != 0)?;
        let height = usize::try_from(size.height())
            .ok()
            .filter(|value| *value != 0)?;
        if self.is_single_plane() {
            if self.singleplanar_format() == SingleplanarFormat::ETC1 {
                let blocks_wide = width.checked_add(3)? >> 2;
                let blocks_high = height.checked_add(3)? >> 2;
                return 8usize.checked_mul(blocks_wide)?.checked_mul(blocks_high);
            }
            return self
                .bytes_per_pixel()?
                .checked_mul(width)?
                .checked_mul(height);
        }
        let planes = self.number_of_planes()?;
        let bytes_per_channel = self.storage_bytes_per_channel();
        let (width_scale, height_scale) = self.subsampling_scale();
        (0..planes).try_fold(0usize, |total, plane| {
            let (plane_width, plane_height) = if self.is_uv_plane(plane) {
                (
                    width.checked_add(width_scale - 1)? / width_scale,
                    height.checked_add(height_scale - 1)? / height_scale,
                )
            } else {
                (width, height)
            };
            let bytes = bytes_per_channel
                .checked_mul(self.channels_in_plane(plane))?
                .checked_mul(plane_width)?
                .checked_mul(plane_height)?;
            total.checked_add(bytes)
        })
    }
    pub fn EstimatedSizeInBytes(&self, size: &super::geometry::Size) -> usize {
        self.MaybeEstimatedSizeInBytes(size).unwrap_or(0)
    }
    pub fn VerifySizeInBytes(&self, size: &super::geometry::Size) -> bool {
        self.MaybeEstimatedSizeInBytes(size).is_some()
    }
}
impl PartialEq for SharedImageFormat {
    fn eq(&self, other: &Self) -> bool {
        self.format == other.format
    }
}
impl Eq for SharedImageFormat {}
impl PartialOrd for SharedImageFormat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SharedImageFormat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.format.cmp(&other.format)
    }
}

pub struct SinglePlaneFormat;
macro_rules! single_plane_constants {
    ($($name:ident),* $(,)?) => { $(pub const $name: SharedImageFormat = SharedImageFormat::SinglePlane(SingleplanarFormat::$name);)* };
}
impl SinglePlaneFormat {
    single_plane_constants!(
        RGBA_8888,
        RGBA_4444,
        BGRA_8888,
        ALPHA_8,
        BGR_565,
        ETC1,
        R_8,
        RG_88,
        LUMINANCE_F16,
        RGBA_F16,
        R_16,
        RG_1616,
        RGBX_8888,
        BGRX_8888,
        RGBA_1010102,
        BGRA_1010102,
        R_F16
    );
    pub const kRGBA_8888: SharedImageFormat = Self::RGBA_8888;
}

/// Live move-only lease supplied by the raster resource pool owner. Default
/// constructs an empty lease; moving it must leave the old slot empty. The
/// owner implements pool release/export and no platform resource lives here.
/// Source dependency: cc/resources/resource_pool.h:171-205.
pub trait TileDrawResource: Default {
    /// Pure format/size accounting owned by the external resource format type.
    /// Source dependency: components/viz/common/resources/shared_image_format.h:159.
    fn EstimatedSizeInBytes(
        &self,
        format: &SharedImageFormat,
        size: &super::geometry::Size,
    ) -> usize;
    fn IsValid(&self) -> bool;
    fn resource_id_for_export(&self) -> super::layer_tree_impl::ResourceId;
    fn size(&self) -> &super::geometry::Size;
    fn format(&self) -> &SharedImageFormat;
}
