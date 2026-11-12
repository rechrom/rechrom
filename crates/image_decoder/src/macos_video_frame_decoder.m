#import <AVFoundation/AVFoundation.h>
#import <CoreMedia/CoreMedia.h>
#import <CoreVideo/CoreVideo.h>
#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
  int ub;
  int ug;
  int vg;
  int vr;
  int yg;
  int yb;
} RechromYuvConstants;

typedef struct {
  uint32_t width;
  uint32_t height;
  uint8_t orientation;
  uint8_t matrix;
  uint8_t primaries;
  uint8_t transfer;
  uint8_t range;
  size_t y_stride;
  size_t uv_stride;
  uint8_t *rgba;
  size_t rgba_length;
  uint8_t *nv12;
  size_t nv12_length;
} RechromDecodedVideoFrame;

// Chromium's software VideoFrame path converts NV12 through libyuv. Keep the
// same 6-bit integer constants and rounding as row_common.cc::YuvPixel instead
// of asking AVFoundation to manufacture an already-converted BGRA buffer.
// Source: third_party/libyuv/source/row_common.cc:1588-1662,1748-1830.
static const RechromYuvConstants kRechromRec601Limited = {
  128, 25, 52, 102, 18997, -1160
};
static const RechromYuvConstants kRechromRec709Limited = {
  128, 14, 34, 115, 18997, -1160
};

static uint8_t RechromClampByte(int32_t value) {
  if (value < 0)
    return 0;
  if (value > 255)
    return 255;
  return (uint8_t)value;
}

static void RechromYuvPixel(uint8_t y,
                            uint8_t u,
                            uint8_t v,
                            const RechromYuvConstants *constants,
                            uint8_t *rgba) {
  uint32_t y32 = (uint32_t)y * 0x0101u;
  int32_t y1 = (int32_t)((y32 * (uint32_t)constants->yg) >> 16);
  int32_t b16 = y1 + (int32_t)u * constants->ub -
                (constants->ub * 128 - constants->yb);
  int32_t g16 = y1 +
                (constants->ug * 128 + constants->vg * 128 + constants->yb) -
                ((int32_t)u * constants->ug + (int32_t)v * constants->vg);
  int32_t r16 = y1 + (int32_t)v * constants->vr -
                (constants->vr * 128 - constants->yb);
  rgba[0] = RechromClampByte(r16 >> 6);
  rgba[1] = RechromClampByte(g16 >> 6);
  rgba[2] = RechromClampByte(b16 >> 6);
  rgba[3] = 255;
}

static CFTypeRef RechromCopyPixelBufferAttachment(CVPixelBufferRef pixel_buffer,
                                                   CFStringRef key) {
  return CVBufferCopyAttachment(pixel_buffer, key, NULL);
}

static uint8_t RechromTransferForPixelBuffer(CVPixelBufferRef pixel_buffer) {
  CFTypeRef value = RechromCopyPixelBufferAttachment(
      pixel_buffer, kCVImageBufferTransferFunctionKey);
  // Source: ui/gfx/mac/color_space_util.mm. The first CoreVideo mapping for
  // ITU-R 709 is gfx::ColorSpace::TransferID::BT709_APPLE.
  uint8_t transfer = !value ||
                             CFEqual(value,
                                     kCVImageBufferTransferFunction_ITU_R_709_2)
                         ? 1
                         : 0;
  if (value)
    CFRelease(value);
  return transfer;
}

static uint8_t RechromPrimariesForPixelBuffer(CVPixelBufferRef pixel_buffer) {
  CFTypeRef value = RechromCopyPixelBufferAttachment(
      pixel_buffer, kCVImageBufferColorPrimariesKey);
  uint8_t primaries = 0;
  if (!value || CFEqual(value, kCVImageBufferColorPrimaries_ITU_R_709_2))
    primaries = 1;
  else if (CFEqual(value, kCVImageBufferColorPrimaries_SMPTE_C))
    primaries = 2;
  else if (CFEqual(value, kCVImageBufferColorPrimaries_EBU_3213))
    primaries = 3;
  if (value)
    CFRelease(value);
  return primaries;
}

static const RechromYuvConstants *RechromYuvConstantsForPixelBuffer(
    CVPixelBufferRef pixel_buffer,
    uint8_t *out_matrix) {
  CFTypeRef matrix = RechromCopyPixelBufferAttachment(
      pixel_buffer, kCVImageBufferYCbCrMatrixKey);
  const RechromYuvConstants *constants = NULL;
  if (!matrix) {
    // VideoToolboxVideoDecoder defaults absent macOS color metadata to
    // BT709/BT709_APPLE/BT709/LIMITED. Software Paint keeps libyuv's historic
    // height-based matrix fallback independently below.
    constants = CVPixelBufferGetHeight(pixel_buffer) < 720
                    ? &kRechromRec601Limited
                    : &kRechromRec709Limited;
    *out_matrix = 1;
  } else if (CFEqual(matrix, kCVImageBufferYCbCrMatrix_ITU_R_709_2)) {
    constants = &kRechromRec709Limited;
    *out_matrix = 1;
  } else if (CFEqual(matrix, kCVImageBufferYCbCrMatrix_ITU_R_601_4)) {
    constants = &kRechromRec601Limited;
    *out_matrix = 2;
  }
  if (matrix)
    CFRelease(matrix);
  return constants;
}

static uint8_t RechromOrientationForTransform(CGAffineTransform transform) {
  const CGFloat epsilon = 0.0001;
  BOOL a0 = fabs(transform.a) < epsilon;
  BOOL b0 = fabs(transform.b) < epsilon;
  BOOL c0 = fabs(transform.c) < epsilon;
  BOOL d0 = fabs(transform.d) < epsilon;
  BOOL a1 = fabs(transform.a - 1.0) < epsilon;
  BOOL an1 = fabs(transform.a + 1.0) < epsilon;
  BOOL b1 = fabs(transform.b - 1.0) < epsilon;
  BOOL bn1 = fabs(transform.b + 1.0) < epsilon;
  BOOL c1 = fabs(transform.c - 1.0) < epsilon;
  BOOL cn1 = fabs(transform.c + 1.0) < epsilon;
  BOOL d1 = fabs(transform.d - 1.0) < epsilon;
  BOOL dn1 = fabs(transform.d + 1.0) < epsilon;
  if (a1 && b0 && c0 && d1)
    return 1;
  if (an1 && b0 && c0 && dn1)
    return 3;
  if (a0 && b1 && cn1 && d0)
    return 6;
  if (a0 && bn1 && c1 && d0)
    return 8;
  return 0;
}

// AVFoundation supplies container demux and VideoToolbox-backed decode. The
// returned storage contains immutable NV12 planes plus the independent RGBA
// software-paint fallback, both owned by Rust after this call. No player, run
// loop, visible window, or external process is created.
int32_t RechromDecodeVideoFirstFrameFile(const char *path,
                                         RechromDecodedVideoFrame *out_frame) {
  @autoreleasepool {
    if (!path || !out_frame)
      return 1;
    *out_frame = (RechromDecodedVideoFrame){0};

    NSError *error = nil;
    NSDictionary *options = @{
      AVURLAssetReferenceRestrictionsKey : @(AVAssetReferenceRestrictionForbidAll)
    };
    NSString *path_string = [NSString stringWithUTF8String:path];
    if (!path_string)
      return 2;
    NSURL *url = [NSURL fileURLWithPath:path_string isDirectory:NO];
    AVURLAsset *asset = [AVURLAsset URLAssetWithURL:url options:options];
    if (!asset)
      return 2;
    __block NSArray<AVAssetTrack *> *tracks = nil;
    __block NSError *track_error = nil;
    dispatch_semaphore_t tracks_ready = dispatch_semaphore_create(0);
    [asset loadTracksWithMediaType:AVMediaTypeVideo
                completionHandler:^(NSArray<AVAssetTrack *> *loaded_tracks,
                                    NSError *load_error) {
                  tracks = loaded_tracks;
                  track_error = load_error;
                  dispatch_semaphore_signal(tracks_ready);
                }];
    dispatch_time_t deadline = dispatch_time(DISPATCH_TIME_NOW, 30 * NSEC_PER_SEC);
    if (dispatch_semaphore_wait(tracks_ready, deadline) != 0 || track_error)
      return 3;
    AVAssetTrack *track = tracks.firstObject;
    if (!track)
      return 3;
    uint8_t orientation = RechromOrientationForTransform(track.preferredTransform);
    if (!orientation)
      return 4;

    AVAssetReader *reader = [[AVAssetReader alloc] initWithAsset:asset error:&error];
    if (!reader || error)
      return 5;
    NSDictionary *settings = @{
      (id)kCVPixelBufferPixelFormatTypeKey :
          @(kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange)
    };
    AVAssetReaderTrackOutput *output =
        [[AVAssetReaderTrackOutput alloc] initWithTrack:track
                                        outputSettings:settings];
    output.alwaysCopiesSampleData = NO;
    if (![reader canAddOutput:output])
      return 6;
    [reader addOutput:output];
    if (![reader startReading])
      return 7;

    CMSampleBufferRef sample = [output copyNextSampleBuffer];
    if (!sample)
      return 8;
    CVPixelBufferRef pixel_buffer = CMSampleBufferGetImageBuffer(sample);
    if (!pixel_buffer) {
      CFRelease(sample);
      return 9;
    }
    size_t width = CVPixelBufferGetWidth(pixel_buffer);
    size_t height = CVPixelBufferGetHeight(pixel_buffer);
    OSType pixel_format = CVPixelBufferGetPixelFormatType(pixel_buffer);
    size_t plane_count = CVPixelBufferGetPlaneCount(pixel_buffer);
    if (pixel_format != kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange ||
        plane_count < 2) {
      CFRelease(sample);
      return 11;
    }
    CVReturn lock = CVPixelBufferLockBaseAddress(pixel_buffer,
                                                 kCVPixelBufferLock_ReadOnly);
    if (lock != kCVReturnSuccess) {
      CFRelease(sample);
      return 10;
    }
    const uint8_t *plane_y = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0);
    const uint8_t *plane_uv = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 1);
    size_t stride_y = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 0);
    size_t stride_uv = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 1);
    size_t height_y = CVPixelBufferGetHeightOfPlane(pixel_buffer, 0);
    size_t height_uv = CVPixelBufferGetHeightOfPlane(pixel_buffer, 1);
    uint8_t matrix = 0;
    const RechromYuvConstants *yuv_constants =
        RechromYuvConstantsForPixelBuffer(pixel_buffer, &matrix);
    uint8_t primaries = RechromPrimariesForPixelBuffer(pixel_buffer);
    uint8_t transfer = RechromTransferForPixelBuffer(pixel_buffer);
    size_t tight_y_stride = width;
    size_t tight_uv_stride = ((width + 1) / 2) * 2;
    size_t tight_uv_height = (height + 1) / 2;
    if (pixel_format != kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange ||
        !plane_y || !plane_uv || !yuv_constants || !matrix || !primaries ||
        !transfer ||
        !width || !height ||
        height_y < height || height_uv < (height + 1) / 2 ||
        stride_y < width || stride_uv < ((width + 1) / 2) * 2 ||
        width > UINT32_MAX || height > UINT32_MAX ||
        width > SIZE_MAX / 4 || height > SIZE_MAX / (width * 4) ||
        height > SIZE_MAX / tight_y_stride ||
        tight_uv_height > SIZE_MAX / tight_uv_stride ||
        tight_y_stride * height >
            SIZE_MAX - tight_uv_stride * tight_uv_height) {
      CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
      CFRelease(sample);
      return 11;
    }
    size_t rgba_length = width * height * 4;
    size_t y_length = tight_y_stride * height;
    size_t uv_length = tight_uv_stride * tight_uv_height;
    size_t nv12_length = y_length + uv_length;
    uint8_t *rgba = malloc(rgba_length);
    uint8_t *nv12 = malloc(nv12_length);
    if (!rgba || !nv12) {
      free(rgba);
      free(nv12);
      CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
      CFRelease(sample);
      return 12;
    }
    for (size_t y = 0; y < height; ++y) {
      uint8_t *destination = rgba + y * width * 4;
      const uint8_t *source_y = plane_y + y * stride_y;
      const uint8_t *source_uv = plane_uv + (y / 2) * stride_uv;
      for (size_t x = 0; x < width; ++x) {
        size_t chroma = (x / 2) * 2;
        RechromYuvPixel(source_y[x], source_uv[chroma], source_uv[chroma + 1],
                        yuv_constants, destination + x * 4);
      }
      memcpy(nv12 + y * tight_y_stride, source_y, tight_y_stride);
    }
    for (size_t y = 0; y < tight_uv_height; ++y) {
      memcpy(nv12 + y_length + y * tight_uv_stride,
             plane_uv + y * stride_uv, tight_uv_stride);
    }
    CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
    CFRelease(sample);
    out_frame->width = (uint32_t)width;
    out_frame->height = (uint32_t)height;
    out_frame->orientation = orientation;
    out_frame->matrix = matrix;
    out_frame->primaries = primaries;
    out_frame->transfer = transfer;
    out_frame->range = 1;
    out_frame->y_stride = tight_y_stride;
    out_frame->uv_stride = tight_uv_stride;
    out_frame->rgba = rgba;
    out_frame->rgba_length = rgba_length;
    out_frame->nv12 = nv12;
    out_frame->nv12_length = nv12_length;
    return 0;
  }
}

void RechromFreeVideoFrame(uint8_t *rgba, uint8_t *nv12) {
  free(rgba);
  free(nv12);
}
