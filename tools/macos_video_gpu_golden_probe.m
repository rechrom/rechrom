// Copyright 2026 The Rechrom Authors
// Controlled macOS video-plane generator for tools/video_gpu_golden.mjs.
// This is a test tool, not a production decoder or renderer.

#import <AVFoundation/AVFoundation.h>
#import <CoreImage/CoreImage.h>
#import <CoreVideo/CoreVideo.h>
#import <Foundation/Foundation.h>

#include <math.h>

typedef enum RechromSampling {
  RechromSamplingNearest,
  RechromSamplingCenteredLinear,
} RechromSampling;

typedef enum RechromPrecision {
  RechromPrecisionFloat,
  RechromPrecisionHalfStages,
} RechromPrecision;

typedef enum RechromQuantization {
  RechromQuantizationRound,
  RechromQuantizationFloor,
} RechromQuantization;

static inline float Clamp01(float value) {
  return fminf(1.0f, fmaxf(0.0f, value));
}

static inline float Half(float value) {
  return (float)(_Float16)value;
}

static inline uint8_t Quantize(float value, RechromQuantization quantization) {
  float scaled = Clamp01(value) * 255.0f;
  long result = quantization == RechromQuantizationFloor
                    ? (long)floorf(scaled)
                    : lrintf(scaled);
  return (uint8_t)MAX(0, MIN(255, result));
}

static float PlaneSample(const uint8_t* plane,
                         size_t stride,
                         size_t width,
                         size_t height,
                         int x,
                         int y,
                         size_t component) {
  x = MAX(0, MIN((int)width - 1, x));
  y = MAX(0, MIN((int)height - 1, y));
  return plane[(size_t)y * stride + (size_t)x * 2 + component] / 255.0f;
}

static float CenteredLinearSample(const uint8_t* plane,
                                  size_t stride,
                                  size_t width,
                                  size_t height,
                                  float x,
                                  float y,
                                  size_t component,
                                  RechromPrecision precision) {
  int x0 = (int)floorf(x);
  int y0 = (int)floorf(y);
  float tx = x - x0;
  float ty = y - y0;
  float a = PlaneSample(plane, stride, width, height, x0, y0, component);
  float b = PlaneSample(plane, stride, width, height, x0 + 1, y0, component);
  float c = PlaneSample(plane, stride, width, height, x0, y0 + 1, component);
  float d = PlaneSample(plane, stride, width, height, x0 + 1, y0 + 1, component);
  if (precision == RechromPrecisionHalfStages) {
    tx = Half(tx);
    ty = Half(ty);
    a = Half(a);
    b = Half(b);
    c = Half(c);
    d = Half(d);
    float top = Half(Half(a * Half(1.0f - tx)) + Half(b * tx));
    float bottom = Half(Half(c * Half(1.0f - tx)) + Half(d * tx));
    return Half(Half(top * Half(1.0f - ty)) + Half(bottom * ty));
  }
  return (1.0f - ty) * ((1.0f - tx) * a + tx * b) +
         ty * ((1.0f - tx) * c + tx * d);
}

static float ToSrgb(float encoded, RechromPrecision precision) {
  encoded = Clamp01(encoded);
  if (precision == RechromPrecisionHalfStages) {
    encoded = Half(encoded);
    float linear = Half(powf(encoded, Half(1.961f)));
    if (linear <= Half(0.0031308f))
      return Half(Half(12.92f) * linear);
    return Half(Half(Half(1.055f) * Half(powf(linear, Half(1.0f / 2.4f)))) -
                Half(0.055f));
  }
  float linear = powf(encoded, 1.961f);
  return linear <= 0.0031308f
             ? 12.92f * linear
             : 1.055f * powf(linear, 1.0f / 2.4f) - 0.055f;
}

static NSData* ConvertNV12(CVPixelBufferRef pixel_buffer,
                           RechromSampling sampling,
                           RechromPrecision precision,
                           BOOL apply_transfer,
                           RechromQuantization quantization) {
  const size_t width = CVPixelBufferGetWidth(pixel_buffer);
  const size_t height = CVPixelBufferGetHeight(pixel_buffer);
  const size_t uv_width = CVPixelBufferGetWidthOfPlane(pixel_buffer, 1);
  const size_t uv_height = CVPixelBufferGetHeightOfPlane(pixel_buffer, 1);
  const size_t y_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 0);
  const size_t uv_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 1);
  const uint8_t* y_plane = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0);
  const uint8_t* uv_plane = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 1);
  NSMutableData* output = [NSMutableData dataWithLength:width * height * 4];
  uint8_t* rgba = output.mutableBytes;

  // Source: third_party/skia/src/core/SkYUVMath.cpp,
  // Rec709_limited_yuv_to_rgb with SK_YUV_COLOR_SPACE_HIGH_PRECISION.
  const float y_r = 0x1.2a150cp+0f;
  const float v_r = 0x1.caf116p+0f;
  const float o_r = -0x1.f225dep-1f;
  const float y_g = 0x1.2a150ap+0f;
  const float u_g = -0x1.b4bbaep-3f;
  const float v_g = -0x1.10d97ep-1f;
  const float o_g = 0x1.34b7ep-2f;
  const float y_b = 0x1.2a150ap+0f;
  const float u_b = 0x1.0e632ep+1f;
  const float o_b = -0x1.2226a6p+0f;

  for (size_t y = 0; y < height; ++y) {
    for (size_t x = 0; x < width; ++x) {
      float luma = y_plane[y * y_stride + x] / 255.0f;
      float u;
      float v;
      if (sampling == RechromSamplingNearest) {
        u = PlaneSample(uv_plane, uv_stride, uv_width, uv_height, (int)x / 2,
                        (int)y / 2, 0);
        v = PlaneSample(uv_plane, uv_stride, uv_width, uv_height, (int)x / 2,
                        (int)y / 2, 1);
      } else {
        // Sampling both planes at one normalized coordinate maps the UV plane
        // to ((x - .5) / 2, (y - .5) / 2) in texel-center coordinates.
        const float uv_x = ((float)x - 0.5f) * 0.5f;
        const float uv_y = ((float)y - 0.5f) * 0.5f;
        u = CenteredLinearSample(uv_plane, uv_stride, uv_width, uv_height,
                                 uv_x, uv_y, 0, precision);
        v = CenteredLinearSample(uv_plane, uv_stride, uv_width, uv_height,
                                 uv_x, uv_y, 1, precision);
      }
      float r;
      float g;
      float b;
      if (precision == RechromPrecisionHalfStages) {
        luma = Half(luma);
        u = Half(u);
        v = Half(v);
        r = Half(Half(Half(luma * Half(y_r)) + Half(v * Half(v_r))) + Half(o_r));
        g = Half(Half(Half(Half(luma * Half(y_g)) + Half(u * Half(u_g))) +
                            Half(v * Half(v_g))) +
                 Half(o_g));
        b = Half(Half(Half(luma * Half(y_b)) + Half(u * Half(u_b))) + Half(o_b));
      } else {
        r = y_r * luma + v_r * v + o_r;
        g = y_g * luma + u_g * u + v_g * v + o_g;
        b = y_b * luma + u_b * u + o_b;
      }
      if (apply_transfer) {
        r = ToSrgb(r, precision);
        g = ToSrgb(g, precision);
        b = ToSrgb(b, precision);
      }
      const size_t offset = (y * width + x) * 4;
      rgba[offset] = Quantize(r, quantization);
      rgba[offset + 1] = Quantize(g, quantization);
      rgba[offset + 2] = Quantize(b, quantization);
      rgba[offset + 3] = 255;
    }
  }
  return output;
}

static CVPixelBufferRef DecodeFirstFrame(NSString* path, NSError** error) {
  AVURLAsset* asset = [AVURLAsset URLAssetWithURL:[NSURL fileURLWithPath:path]
                                          options:nil];
  AVAssetReader* reader = [[AVAssetReader alloc] initWithAsset:asset error:error];
  if (!reader)
    return nil;
  AVAssetTrack* track = [[asset tracksWithMediaType:AVMediaTypeVideo] firstObject];
  NSDictionary* settings = @{
    (id)kCVPixelBufferPixelFormatTypeKey :
        @(kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange),
  };
  AVAssetReaderTrackOutput* output =
      [[AVAssetReaderTrackOutput alloc] initWithTrack:track outputSettings:settings];
  [reader addOutput:output];
  if (![reader startReading]) {
    if (error)
      *error = reader.error;
    return nil;
  }
  CMSampleBufferRef sample = [output copyNextSampleBuffer];
  if (!sample) {
    if (error)
      *error = reader.error;
    return nil;
  }
  CVPixelBufferRef pixel_buffer =
      CVPixelBufferRetain(CMSampleBufferGetImageBuffer(sample));
  CFRelease(sample);
  return pixel_buffer;
}

static BOOL WriteCI(CVPixelBufferRef pixel_buffer,
                    BOOL managed,
                    NSString* path) {
  const size_t width = CVPixelBufferGetWidth(pixel_buffer);
  const size_t height = CVPixelBufferGetHeight(pixel_buffer);
  NSMutableData* output = [NSMutableData dataWithLength:width * height * 4];
  CIImage* image = [CIImage imageWithCVPixelBuffer:pixel_buffer];
  CGColorSpaceRef srgb = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  NSDictionary* options = managed
                              ? @{}
                              : @{
                                  kCIContextWorkingColorSpace : [NSNull null],
                                  kCIContextOutputColorSpace : [NSNull null],
                                };
  CIContext* context = [CIContext contextWithOptions:options];
  [context render:image
          toBitmap:output.mutableBytes
          rowBytes:width * 4
            bounds:CGRectMake(0, 0, width, height)
            format:kCIFormatRGBA8
        colorSpace:managed ? srgb : nil];
  CGColorSpaceRelease(srgb);
  return [output writeToFile:path atomically:YES];
}

static NSString* Attachment(CVPixelBufferRef pixel_buffer, CFStringRef key) {
  CFTypeRef value = CVBufferCopyAttachment(pixel_buffer, key, NULL);
  if (!value)
    return @"missing";
  NSString* description = [(__bridge id)value description];
  CFRelease(value);
  return description;
}

static BOOL WritePackedPlanes(CVPixelBufferRef pixel_buffer,
                              NSString* directory) {
  const size_t width = CVPixelBufferGetWidth(pixel_buffer);
  const size_t height = CVPixelBufferGetHeight(pixel_buffer);
  const size_t y_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 0);
  const size_t uv_width = CVPixelBufferGetWidthOfPlane(pixel_buffer, 1);
  const size_t uv_height = CVPixelBufferGetHeightOfPlane(pixel_buffer, 1);
  const size_t uv_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 1);
  const uint8_t* y_plane = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0);
  const uint8_t* uv_plane = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 1);
  NSMutableData* packed_y = [NSMutableData dataWithLength:width * height];
  NSMutableData* packed_uv =
      [NSMutableData dataWithLength:uv_width * uv_height * 2];
  uint8_t* y_output = packed_y.mutableBytes;
  uint8_t* uv_output = packed_uv.mutableBytes;
  for (size_t row = 0; row < height; ++row)
    memcpy(y_output + row * width, y_plane + row * y_stride, width);
  for (size_t row = 0; row < uv_height; ++row)
    memcpy(uv_output + row * uv_width * 2, uv_plane + row * uv_stride,
           uv_width * 2);
  return [packed_y writeToFile:[directory stringByAppendingPathComponent:@"av_y.raw"]
                     atomically:YES] &&
         [packed_uv writeToFile:[directory stringByAppendingPathComponent:@"av_uv.raw"]
                      atomically:YES];
}

int main(int argc, const char* argv[]) {
  @autoreleasepool {
    if (argc != 3) {
      fprintf(stderr, "usage: %s VIDEO OUTPUT_DIRECTORY\n", argv[0]);
      return 2;
    }
    NSString* video = @(argv[1]);
    NSString* directory = @(argv[2]);
    NSError* error = nil;
    CVPixelBufferRef pixel_buffer = DecodeFirstFrame(video, &error);
    if (!pixel_buffer) {
      fprintf(stderr, "decode failed: %s\n", error.description.UTF8String);
      return 1;
    }
    CVPixelBufferLockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
    if (!WritePackedPlanes(pixel_buffer, directory)) {
      fprintf(stderr, "failed to write packed AVFoundation planes\n");
      CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
      CFRelease(pixel_buffer);
      return 1;
    }
    NSDictionary<NSString*, NSData*>* candidates = @{
      @"center_raw_fp32_round.rgba" :
          ConvertNV12(pixel_buffer, RechromSamplingCenteredLinear,
                      RechromPrecisionFloat, NO, RechromQuantizationRound),
      @"center_gamma_fp32_round.rgba" :
          ConvertNV12(pixel_buffer, RechromSamplingCenteredLinear,
                      RechromPrecisionFloat, YES, RechromQuantizationRound),
      @"center_gamma_fp32_floor.rgba" :
          ConvertNV12(pixel_buffer, RechromSamplingCenteredLinear,
                      RechromPrecisionFloat, YES, RechromQuantizationFloor),
      @"center_raw_fp16_round.rgba" :
          ConvertNV12(pixel_buffer, RechromSamplingCenteredLinear,
                      RechromPrecisionHalfStages, NO, RechromQuantizationRound),
      @"center_gamma_fp16_round.rgba" :
          ConvertNV12(pixel_buffer, RechromSamplingCenteredLinear,
                      RechromPrecisionHalfStages, YES, RechromQuantizationRound),
      @"nearest_gamma_fp32_round.rgba" :
          ConvertNV12(pixel_buffer, RechromSamplingNearest,
                      RechromPrecisionFloat, YES, RechromQuantizationRound),
    };
    for (NSString* name in candidates) {
      if (![candidates[name] writeToFile:[directory stringByAppendingPathComponent:name]
                              atomically:YES]) {
        fprintf(stderr, "failed to write %s\n", name.UTF8String);
        CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
        CFRelease(pixel_buffer);
        return 1;
      }
    }
    if (!WriteCI(pixel_buffer, NO,
                 [directory stringByAppendingPathComponent:@"ci_raw.rgba"]) ||
        !WriteCI(pixel_buffer, YES,
                 [directory stringByAppendingPathComponent:@"ci_srgb.rgba"])) {
      fprintf(stderr, "failed to write CoreImage candidates\n");
      CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
      CFRelease(pixel_buffer);
      return 1;
    }
    NSDictionary* metadata = @{
      @"width" : @(CVPixelBufferGetWidth(pixel_buffer)),
      @"height" : @(CVPixelBufferGetHeight(pixel_buffer)),
      @"pixel_format" : @"420v",
      @"primaries" : Attachment(pixel_buffer, kCVImageBufferColorPrimariesKey),
      @"transfer" : Attachment(pixel_buffer, kCVImageBufferTransferFunctionKey),
      @"matrix" : Attachment(pixel_buffer, kCVImageBufferYCbCrMatrixKey),
      @"chroma_location" :
          Attachment(pixel_buffer, kCVImageBufferChromaLocationTopFieldKey),
    };
    NSData* json = [NSJSONSerialization dataWithJSONObject:metadata options:0 error:&error];
    fwrite(json.bytes, 1, json.length, stdout);
    fputc('\n', stdout);
    CVPixelBufferUnlockBaseAddress(pixel_buffer, kCVPixelBufferLock_ReadOnly);
    CFRelease(pixel_buffer);
  }
  return 0;
}
