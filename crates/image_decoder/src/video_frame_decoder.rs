//! Platform video current-frame extraction used by the software VideoPainter.
//!
//! Chromium owns demux and decode in media/, then publishes the first frame
//! through VideoRendererImpl -> VideoFrameCompositor before VideoPainter asks
//! HTMLVideoElement to paint it. Rechrom keeps that ownership boundary: this
//! module decodes bytes, while DOM/media state owns publication and timing.

#![allow(non_snake_case)]

use std::io;

use crate::image_decoder::DecodedImage;
use image_resource::{
    VideoColorMatrix, VideoColorPrimaries, VideoColorRange, VideoColorSpace, VideoFrameResource,
    VideoPixelFormat, VideoTransferFunction,
};

const MAX_ENCODED_BYTES: usize = 256 * 1024 * 1024;
const MAX_DIMENSION: u32 = 16_384;
const MAX_DECODED_BYTES: usize = 512 * 1024 * 1024;

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Default)]
struct PlatformVideoFrame {
    width: u32,
    height: u32,
    orientation: u8,
    matrix: u8,
    primaries: u8,
    transfer: u8,
    range: u8,
    y_stride: usize,
    uv_stride: usize,
    rgba: *mut u8,
    rgba_length: usize,
    nv12: *mut u8,
    nv12_length: usize,
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn RechromDecodeVideoFirstFrameFile(
        path: *const std::ffi::c_char,
        frame: *mut PlatformVideoFrame,
    ) -> i32;
    fn RechromFreeVideoFrame(rgba: *mut u8, nv12: *mut u8);
}

#[derive(Debug)]
pub struct DecodedVideoFrame {
    /// Exact Chromium/libyuv conversion used only by software VideoPainter.
    pub software_image: DecodedImage,
    /// Original decoded planes and gfx::ColorSpace producer facts used by the
    /// compositor TextureDrawQuad path.
    pub resource: VideoFrameResource,
}

/// Decodes one immutable current frame without creating an HTML media timeline.
/// The caller remains responsible for readyState/currentTime and frame revision.
// cpp: media/renderers/video_renderer_impl.cc:584-736
// cpp: media/renderers/paint_canvas_video_renderer.cc:256-341,517-523
pub fn DecodeVideoFirstFrame(bytes: &[u8], mime_type: &str) -> io::Result<DecodedVideoFrame> {
    if bytes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot decode an empty video",
        ));
    }
    if bytes.len() > MAX_ENCODED_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "encoded video exceeds the current-frame byte budget",
        ));
    }
    if !matches!(mime_type, "video/mp4" | "video/quicktime" | "") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("video current-frame decoder does not support {mime_type}"),
        ));
    }
    DecodePlatformVideoFirstFrame(bytes, mime_type)
}

#[cfg(target_os = "macos")]
fn DecodePlatformVideoFirstFrame(bytes: &[u8], mime_type: &str) -> io::Result<DecodedVideoFrame> {
    use std::io::Write as _;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMPORARY_VIDEO: AtomicU64 = AtomicU64::new(1);
    struct TemporaryVideoFile(std::path::PathBuf);
    impl Drop for TemporaryVideoFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let extension = if mime_type == "video/quicktime" {
        "mov"
    } else {
        "mp4"
    };
    let sequence = NEXT_TEMPORARY_VIDEO.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "rechrom-video-frame-{}-{sequence}.{extension}",
        std::process::id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)?;
    file.write_all(bytes)?;
    drop(file);
    let temporary = TemporaryVideoFile(path);
    let path = std::ffi::CString::new(temporary.0.as_os_str().as_bytes())
        .map_err(|_| io::Error::other("temporary video path contains NUL"))?;
    let mut platform = PlatformVideoFrame::default();
    let status = unsafe { RechromDecodeVideoFirstFrameFile(path.as_ptr(), &mut platform) };
    if status != 0 {
        return Err(io::Error::other(format!(
            "platform video first-frame decode failed ({status})"
        )));
    }
    struct PlatformAllocations(*mut u8, *mut u8);
    impl Drop for PlatformAllocations {
        fn drop(&mut self) {
            unsafe { RechromFreeVideoFrame(self.0, self.1) };
        }
    }
    let allocations = PlatformAllocations(platform.rgba, platform.nv12);
    let expected_rgba = (platform.width as usize)
        .checked_mul(platform.height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .filter(|length| *length <= MAX_DECODED_BYTES);
    let y_length = platform.y_stride.checked_mul(platform.height as usize);
    let uv_length = platform
        .uv_stride
        .checked_mul((platform.height as usize).div_ceil(2));
    let expected_nv12 = y_length.and_then(|y| uv_length.and_then(|uv| y.checked_add(uv)));
    if platform.width == 0
        || platform.height == 0
        || platform.width > MAX_DIMENSION
        || platform.height > MAX_DIMENSION
        || expected_rgba != Some(platform.rgba_length)
        || expected_nv12 != Some(platform.nv12_length)
        || platform.nv12_length > MAX_DECODED_BYTES
        || platform.y_stride < platform.width as usize
        || platform.uv_stride < (platform.width as usize).div_ceil(2) * 2
        || platform.rgba.is_null()
        || platform.nv12.is_null()
    {
        return Err(io::Error::other(
            "platform video frame violates the decoded pixel budget",
        ));
    }
    let color_space = VideoColorSpace {
        matrix: match platform.matrix {
            1 => VideoColorMatrix::Bt709,
            2 => VideoColorMatrix::Smpte170M,
            _ => return Err(io::Error::other("unsupported video YUV matrix")),
        },
        primaries: match platform.primaries {
            1 => VideoColorPrimaries::Bt709,
            2 => VideoColorPrimaries::Smpte170M,
            3 => VideoColorPrimaries::Ebu3213E,
            _ => return Err(io::Error::other("unsupported video color primaries")),
        },
        transfer: match platform.transfer {
            1 => VideoTransferFunction::Bt709Apple,
            _ => return Err(io::Error::other("unsupported video transfer function")),
        },
        range: match platform.range {
            1 => VideoColorRange::Limited,
            _ => return Err(io::Error::other("unsupported video color range")),
        },
    };
    let orientation = image_resource::ImageOrientation::from_exif(platform.orientation);
    let rgba8 = unsafe { std::slice::from_raw_parts(platform.rgba, platform.rgba_length).to_vec() };
    let nv12 = unsafe { std::slice::from_raw_parts(platform.nv12, platform.nv12_length) };
    let y_length = y_length.expect("validated NV12 Y length");
    let resource = VideoFrameResource {
        width: platform.width,
        height: platform.height,
        orientation,
        format: VideoPixelFormat::Nv12,
        color_space,
        y_stride: platform.y_stride,
        uv_stride: platform.uv_stride,
        y_plane: std::sync::Arc::new(nv12[..y_length].to_vec()),
        uv_plane: std::sync::Arc::new(nv12[y_length..].to_vec()),
    };
    debug_assert!(resource.Validate());
    drop(allocations);
    Ok(DecodedVideoFrame {
        software_image: DecodedImage {
            filename_extension: None,
            width: platform.width,
            height: platform.height,
            orientation,
            hot_spot: None,
            alpha_type: image_resource::PaintImageAlphaType::Premultiplied,
            animation: image_resource::PaintImageAnimationMetadata::Unavailable(
                image_resource::PaintImageMetadataUnavailable::EncodedAnimationTimeline {
                    format: "MP4 video",
                },
            ),
            animation_frames: Vec::new(),
            rgba8,
        },
        resource,
    })
}

#[cfg(not(target_os = "macos"))]
fn DecodePlatformVideoFirstFrame(_bytes: &[u8], _mime_type: &str) -> io::Result<DecodedVideoFrame> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "no platform video current-frame decoder is available",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_video_is_rejected_before_platform_decode() {
        let error = DecodeVideoFirstFrame(&[], "video/mp4").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn unsupported_container_is_explicit() {
        let error = DecodeVideoFirstFrame(&[1], "video/webm").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}
