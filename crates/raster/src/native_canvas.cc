// Skia C ABI for the Rust translation of skia_renderer.cc's replay sequence.
// Branching over DisplayItemType and paint-state decisions stays in Rust.
#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <map>
#include <memory>
#include <unordered_map>
#include <utility>
#include <vector>

#include "include/core/SkCanvas.h"
#include "include/core/SkBlurTypes.h"
#include "include/core/SkColor.h"
#include "include/core/SkColorFilter.h"
#include "include/core/SkColorSpace.h"
#include "include/core/SkData.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontArguments.h"
#include "include/core/SkFontParameters.h"
#include "include/core/SkFontMgr.h"
#include "include/core/SkFontMetrics.h"
#include "include/core/SkFontStyle.h"
#include "include/core/SkGraphics.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkM44.h"
#include "include/core/SkImage.h"
#include "include/core/SkMaskFilter.h"
#include "include/core/SkPaint.h"
#include "include/core/SkPath.h"
#include "include/core/SkPathBuilder.h"
#include "include/core/SkPixmap.h"
#include "include/core/SkRRect.h"
#include "include/core/SkSamplingOptions.h"
#include "include/core/SkSurface.h"
#include "include/core/SkSurfaceProps.h"
#include "include/core/SkTextBlob.h"
#include "include/core/SkTypeface.h"
#include "include/effects/SkDashPathEffect.h"
#include "include/effects/SkGradient.h"
#include "include/effects/SkImageFilters.h"
#if defined(__APPLE__)
#include "include/ports/SkFontMgr_mac_ct.h"
#endif

namespace {
struct NativeRect {
  double x, y, width, height;
};
struct NativeRadii {
  double values[8];
};
struct NativeColor {
  float red, green, blue, alpha;
};
struct NativeGradientStop {
  NativeColor color;
  float offset;
};
struct NativeVariation {
  uint32_t tag;
  float value;
};
struct NativePathCommand {
  uint32_t verb;
  float x, y, c1x, c1y, c2x, c2y, weight;
};
struct NativeFontMetrics {
  float ascent, descent, leading, x_height, cap_height;
  float underline_position, underline_thickness;
  bool has_underline_position, has_underline_thickness;
};
struct NativeGlyphMetrics {
  float x, y, width, height, advance;
};
struct NativeFontBackendState {
  sk_sp<SkTypeface> typeface;
  sk_sp<SkTypeface> metrics_typeface;
};
struct NativeCanvasState {
  sk_sp<SkSurface> surface;
  sk_sp<SkFontMgr> font_manager;
  std::unordered_map<uint64_t, sk_sp<SkImage>> images;
  std::map<std::pair<uint64_t, int>, sk_sp<SkImage>> mip_images;
  std::vector<sk_sp<SkTypeface>> typefaces;
  std::vector<std::vector<NativeVariation>> font_variations;
  std::vector<uint32_t> font_face_indices;
  std::vector<bool> font_supports_optical_size;
};

sk_sp<SkTypeface> CloneWithVariations(sk_sp<SkTypeface> typeface,
                                      uint32_t face_index,
                                      const std::vector<NativeVariation>& variations) {
  if (!typeface || variations.empty()) return typeface;
  std::vector<SkFontArguments::VariationPosition::Coordinate> coordinates;
  coordinates.reserve(variations.size());
  for (const auto& variation : variations)
    coordinates.push_back({variation.tag, variation.value});
  SkFontArguments arguments;
  arguments.setCollectionIndex(static_cast<int>(face_index & 0xffffu));
  arguments.setVariationDesignPosition(
      {coordinates.data(), static_cast<int>(coordinates.size())});
  sk_sp<SkTypeface> varied = typeface->makeClone(arguments);
  return varied ? varied : typeface;
}

SkRect ToRect(NativeRect rect) {
  return SkRect::MakeXYWH(static_cast<float>(rect.x),
                          static_cast<float>(rect.y),
                          static_cast<float>(rect.width),
                          static_cast<float>(rect.height));
}

SkRRect ToRRect(NativeRect rect, NativeRadii radii) {
  const SkRect bounds = ToRect(rect);
  const double* r = radii.values;
  if (r[0] == r[2] && r[1] == r[3] &&
      r[0] == r[4] && r[1] == r[5] &&
      r[0] == r[6] && r[1] == r[7] && r[0] == r[1]) {
    return SkRRect::MakeRectXY(bounds, static_cast<float>(r[0]),
                               static_cast<float>(r[1]));
  }
  SkVector corners[4] = {
      {static_cast<float>(r[0]), static_cast<float>(r[1])},
      {static_cast<float>(r[2]), static_cast<float>(r[3])},
      {static_cast<float>(r[4]), static_cast<float>(r[5])},
      {static_cast<float>(r[6]), static_cast<float>(r[7])},
  };
  SkRRect result;
  result.setRectRadii(bounds, corners);
  return result;
}

SkPaint ToPaint(NativeColor color, bool antialias) {
  SkPaint paint;
  paint.setAntiAlias(antialias);
  paint.setColor4f({std::clamp(color.red, 0.0f, 1.0f),
                    std::clamp(color.green, 0.0f, 1.0f),
                    std::clamp(color.blue, 0.0f, 1.0f),
                    std::clamp(color.alpha, 0.0f, 1.0f)});
  return paint;
}
}  // namespace

extern "C" {
// cpp: skia_renderer/skia_renderer.cc:374-410,779-826
void LayoutngCanvasDrawLinearGradient(void* canvas, NativeRect destination,
                                      float start_x, float start_y,
                                      float end_x, float end_y,
                                      const NativeGradientStop* stops,
                                      size_t stop_count, int spread,
                                      const float* matrix,
                                      bool premultiplied, bool antialias,
                                      bool dither) {
  if (!canvas || !stops || stop_count == 0 || !matrix) return;
  auto* state = static_cast<NativeCanvasState*>(canvas);
  std::vector<SkColor4f> colors;
  std::vector<float> positions;
  colors.reserve(stop_count + 2);
  positions.reserve(stop_count + 2);
  if (stops[0].offset > 0) {
    colors.push_back({stops[0].color.red, stops[0].color.green,
                      stops[0].color.blue, stops[0].color.alpha});
    positions.push_back(0);
  }
  for (size_t i = 0; i < stop_count; ++i) {
    colors.push_back({stops[i].color.red, stops[i].color.green,
                      stops[i].color.blue, stops[i].color.alpha});
    positions.push_back(stops[i].offset);
  }
  if (positions.back() < 1) {
    colors.push_back(colors.back());
    positions.push_back(1);
  }
  SkGradient::Interpolation interpolation;
  interpolation.fInPremul = premultiplied
      ? SkGradient::Interpolation::InPremul::kYes
      : SkGradient::Interpolation::InPremul::kNo;
  interpolation.fColorSpace = SkGradient::Interpolation::ColorSpace::kSRGB;
  SkTileMode mode = spread == 1 ? SkTileMode::kMirror
                  : spread == 2 ? SkTileMode::kRepeat
                                : SkTileMode::kClamp;
  SkGradient gradient(
      SkGradient::Colors(SkSpan<const SkColor4f>(colors),
                         SkSpan<const float>(positions), mode,
                         SkColorSpace::MakeSRGB()),
      interpolation);
  SkMatrix local;
  local.setAll(matrix[0], matrix[4], matrix[12], matrix[1], matrix[5],
               matrix[13], matrix[3], matrix[7], matrix[15]);
  const SkPoint points[2] = {{start_x, start_y}, {end_x, end_y}};
  SkPaint paint;
  paint.setColor4f({0, 0, 0, 1});
  paint.setDither(dither);
  paint.setAntiAlias(antialias &&
                     !state->surface->getCanvas()->getLocalToDevice()
                          .asM33().rectStaysRect());
  paint.setShader(SkShaders::LinearGradient(points, gradient, &local));
  state->surface->getCanvas()->drawRect(ToRect(destination), paint);
}
void* LayoutngFontBackendCreate(const uint8_t* bytes, size_t byte_count,
                                uint32_t face_index,
                                const NativeVariation* variations,
                                size_t variation_count,
                                const char* native_family,
                                const char* metrics_family,
                                double weight, bool italic) {
  sk_sp<SkFontMgr> manager;
#if defined(__APPLE__)
  manager = SkFontMgr_New_CoreText(nullptr);
#endif
  if (!manager) return nullptr;
  const SkFontStyle style(static_cast<int>(weight), SkFontStyle::kNormal_Width,
                          italic ? SkFontStyle::kItalic_Slant : SkFontStyle::kUpright_Slant);
  sk_sp<SkTypeface> typeface;
  if (native_family && *native_family)
    typeface = manager->matchFamilyStyle(native_family, style);
  if (!typeface && bytes && byte_count)
    typeface = manager->makeFromData(SkData::MakeWithCopy(bytes, byte_count),
                                     static_cast<int>(face_index & 0xffffu));
  if (!typeface) return nullptr;
  sk_sp<SkTypeface> metrics_typeface = typeface;
  if (metrics_family && *metrics_family) {
    if (auto matched = manager->matchFamilyStyle(metrics_family, style))
      metrics_typeface = std::move(matched);
  }
  std::vector<NativeVariation> axes;
  if (variations && variation_count)
    axes.assign(variations, variations + variation_count);
  typeface = CloneWithVariations(std::move(typeface), face_index, axes);
  return new NativeFontBackendState{std::move(typeface), std::move(metrics_typeface)};
}

void LayoutngFontBackendDestroy(void* backend) {
  delete static_cast<NativeFontBackendState*>(backend);
}

void* LayoutngFontBackendWithVariations(void* backend,
                                       const NativeVariation* variations,
                                       size_t variation_count) {
  auto* source = static_cast<NativeFontBackendState*>(backend);
  if (!source) return nullptr;
  std::vector<NativeVariation> axes;
  if (variations && variation_count)
    axes.assign(variations, variations + variation_count);
  return new NativeFontBackendState{
      CloneWithVariations(source->typeface, 0, axes),
      CloneWithVariations(source->metrics_typeface, 0, axes)};
}

NativeFontMetrics LayoutngFontBackendMetrics(void* backend, float size,
                                              bool synthetic_bold,
                                              bool synthetic_italic) {
  auto* state = static_cast<NativeFontBackendState*>(backend);
  SkFont font(state->metrics_typeface, size);
  font.setEdging(SkFont::Edging::kSubpixelAntiAlias);
  font.setEmbeddedBitmaps(false);
  font.setSubpixel(true);
  font.setLinearMetrics(true);
  font.setEmbolden(synthetic_bold);
  font.setSkewX(synthetic_italic ? -0.25f : 0);
  SkFontMetrics metrics;
  font.getMetrics(&metrics);
  NativeFontMetrics result{-metrics.fAscent, metrics.fDescent, metrics.fLeading,
                            metrics.fXHeight, metrics.fCapHeight, 0, 0, false, false};
  result.has_underline_position = metrics.hasUnderlinePosition(&result.underline_position);
  result.has_underline_thickness = metrics.hasUnderlineThickness(&result.underline_thickness);
  return result;
}

NativeGlyphMetrics LayoutngFontBackendGlyphMetrics(void* backend, uint16_t glyph,
                                                    float size, bool synthetic_bold,
                                                    bool synthetic_italic) {
  auto* state = static_cast<NativeFontBackendState*>(backend);
  SkFont font(state->typeface, size);
  font.setEdging(SkFont::Edging::kSubpixelAntiAlias);
  font.setEmbeddedBitmaps(false);
  font.setSubpixel(true);
  font.setLinearMetrics(true);
  font.setEmbolden(synthetic_bold);
  font.setSkewX(synthetic_italic ? -0.25f : 0);
  SkRect bounds;
#if defined(__APPLE__)
  if (const auto path = font.getPath(glyph))
    bounds = path->getBounds();
  else
#endif
    bounds = font.getBounds(glyph, nullptr);
  return {bounds.x(), bounds.y(), bounds.width(), bounds.height(),
          font.getWidth(glyph)};
}

void* LayoutngCanvasCreate(int width, int height) {
  if (width <= 0 || height <= 0) return nullptr;
  SkGraphics::Init();
  const SkImageInfo info =
      SkImageInfo::MakeN32(width, height, kOpaque_SkAlphaType);
  const SkSurfaceProps props(0, kUnknown_SkPixelGeometry);
  auto surface = SkSurfaces::Raster(info, &props);
  if (!surface) return nullptr;
  surface->getCanvas()->clear(SK_ColorWHITE);
  auto result = std::make_unique<NativeCanvasState>();
  result->surface = std::move(surface);
#if defined(__APPLE__)
  result->font_manager = SkFontMgr_New_CoreText(nullptr);
#endif
  return result.release();
}

void LayoutngCanvasDestroy(void* canvas) { delete static_cast<NativeCanvasState*>(canvas); }

void LayoutngCanvasTranslate(void* canvas, float x, float y) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->translate(x, y);
}

void LayoutngCanvasConcat(void* canvas, const float* matrix) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->concat(
      SkM44::ColMajor(matrix));
}

void LayoutngCanvasSave(void* canvas) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->save();
}

void LayoutngCanvasRestore(void* canvas) {
  auto* target = static_cast<NativeCanvasState*>(canvas)->surface->getCanvas();
  if (target->getSaveCount() > 1) target->restore();
}

int LayoutngCanvasSaveCount(void* canvas) {
  return static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->getSaveCount();
}

void LayoutngCanvasClipRect(void* canvas, NativeRect rect, bool antialias) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->clipRect(
      ToRect(rect), SkClipOp::kIntersect, antialias);
}

void LayoutngCanvasClipRRect(void* canvas, NativeRect rect, NativeRadii radii,
                             bool antialias) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->clipRRect(
      ToRRect(rect, radii), SkClipOp::kIntersect, antialias);
}

void LayoutngCanvasClipOutRect(void* canvas, NativeRect rect, bool antialias) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->clipRect(
      ToRect(rect), SkClipOp::kDifference, antialias);
}

void LayoutngCanvasClipOutRRect(void* canvas, NativeRect rect, NativeRadii radii,
                                bool antialias) {
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->clipRRect(
      ToRRect(rect, radii), SkClipOp::kDifference, antialias);
}

void LayoutngCanvasSaveLayer(void* canvas, NativeRect rect) {
  const SkRect bounds = ToRect(rect);
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->saveLayer(
      rect.width > 0 && rect.height > 0 ? &bounds : nullptr, nullptr);
}

// Test-only oracle for the admitted blur-filter layer subset.
void LayoutngCanvasSaveLayerBlurFilters(void* canvas, const float* sigmas, size_t count) {
  sk_sp<SkImageFilter> filter;
  for (size_t i = 0; i < count; ++i) {
    filter = SkImageFilters::Blur(sigmas[i], sigmas[i], SkTileMode::kDecal, std::move(filter));
  }
  SkPaint paint;
  paint.setImageFilter(std::move(filter));
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->saveLayer(nullptr, &paint);
}

// Test oracle for skia_renderer.cc's mask-surface restore.
void LayoutngCanvasSaveLayerDstIn(void* canvas) {
  SkPaint paint;
  paint.setBlendMode(SkBlendMode::kDstIn);
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->saveLayer(nullptr, &paint);
}

// skia_renderer.cc:151-171,1035-1049. Keep the original normal/opaque
// branch's unbounded saveLayer and the native mode/alpha/bounds for all others.
void LayoutngCanvasSaveLayerBlend(void* canvas, NativeRect rect, int mode, float opacity) {
  SkCanvas* target = static_cast<NativeCanvasState*>(canvas)->surface->getCanvas();
  if (mode == 0 && opacity == 1.0f) {
    target->saveLayer(nullptr, nullptr);
    return;
  }
  static constexpr SkBlendMode modes[] = {
      SkBlendMode::kSrcOver, SkBlendMode::kMultiply, SkBlendMode::kScreen,
      SkBlendMode::kOverlay, SkBlendMode::kDarken, SkBlendMode::kLighten,
      SkBlendMode::kColorDodge, SkBlendMode::kColorBurn, SkBlendMode::kHardLight,
      SkBlendMode::kSoftLight, SkBlendMode::kDifference, SkBlendMode::kExclusion,
      SkBlendMode::kHue, SkBlendMode::kSaturation, SkBlendMode::kColor,
      SkBlendMode::kLuminosity, SkBlendMode::kPlus};
  SkPaint paint;
  paint.setBlendMode(modes[mode]);
  paint.setAlphaf(std::clamp(opacity, 0.0f, 1.0f));
  const SkRect bounds = ToRect(rect);
  target->saveLayer(rect.width > 0 && rect.height > 0 ? &bounds : nullptr, &paint);
}

void LayoutngCanvasSaveLayerAlpha(void* canvas, NativeRect rect, float opacity) {
  SkPaint paint;
  paint.setAlphaf(std::clamp(opacity, 0.0f, 1.0f));
  const SkRect bounds = ToRect(rect);
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->saveLayer(
      SkCanvas::SaveLayerRec(rect.width > 0 && rect.height > 0 ? &bounds : nullptr,
                             &paint, SkCanvas::kF16ColorType));
}

void LayoutngCanvasDrawRect(void* canvas, NativeRect rect, NativeColor color,
                            bool antialias) {
  SkCanvas* target = static_cast<NativeCanvasState*>(canvas)->surface->getCanvas();
  SkPaint paint = ToPaint(color, antialias &&
                         !target->getLocalToDevice().asM33().rectStaysRect());
  target->drawRect(ToRect(rect), paint);
}

void LayoutngCanvasStrokeLine(void* canvas, NativeRect rect, NativeColor color,
                              bool antialias, float stroke_width,
                              bool round_cap, const double* dashes,
                              size_t dash_count, float dash_offset) {
  SkPaint paint = ToPaint(color, antialias);
  paint.setStyle(SkPaint::kStroke_Style);
  paint.setStrokeWidth(stroke_width);
  paint.setStrokeCap(round_cap ? SkPaint::kRound_Cap : SkPaint::kButt_Cap);
  if (dashes && dash_count) {
    std::vector<SkScalar> intervals(dashes, dashes + dash_count);
    paint.setPathEffect(SkDashPathEffect::Make(
        SkSpan<const SkScalar>(intervals.data(), intervals.size()), dash_offset));
  }
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->drawLine(
      static_cast<float>(rect.x), static_cast<float>(rect.y),
      static_cast<float>(rect.x + rect.width),
      static_cast<float>(rect.y + rect.height), paint);
}

void LayoutngCanvasDrawOval(void* canvas, NativeRect rect, NativeColor color,
                            bool antialias) {
  SkPaint paint = ToPaint(color, antialias);
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->drawOval(
      ToRect(rect), paint);
}

// Reference-only clip adapter: skia_renderer.cc:1010-1012. Build the same
// path independently of the Rust rasterizer, then call the original SkCanvas.
void LayoutngCanvasClipPath(void* canvas, const NativePathCommand* commands,
                           size_t count, bool even_odd, bool antialias) {
  SkPathBuilder builder;
  builder.setFillType(even_odd ? SkPathFillType::kEvenOdd : SkPathFillType::kWinding);
  for (size_t i = 0; i < count; ++i) {
    const auto& c = commands[i];
    switch (c.verb) {
      case 0: builder.moveTo(c.x, c.y); break;
      case 1: builder.lineTo(c.x, c.y); break;
      case 2: builder.quadTo(c.c1x, c.c1y, c.x, c.y); break;
      case 3: builder.conicTo(c.c1x, c.c1y, c.x, c.y, c.weight); break;
      case 4: builder.cubicTo(c.c1x, c.c1y, c.c2x, c.c2y, c.x, c.y); break;
      case 5: builder.close(); break;
    }
  }
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->clipPath(
      builder.detach(), SkClipOp::kIntersect, antialias);
}

// Reference-only canvas adapter. Match skia_renderer.cc::Path / PaintFor;
// pure_replay does not link or call this path.
void LayoutngCanvasDrawPath(void* canvas, const NativePathCommand* commands,
                            size_t count, bool even_odd, bool inverse,
                            NativeColor color, bool antialias, bool stroke,
                            float width, float miter, int cap, int join,
                            const double* dashes, size_t dash_count,
                            float dash_offset, float blur_radius) {
  SkPathBuilder builder;
  builder.setFillType(inverse ? SkPathFillType::kInverseWinding
                              : even_odd ? SkPathFillType::kEvenOdd
                                         : SkPathFillType::kWinding);
  for (size_t i = 0; i < count; ++i) {
    const auto& c = commands[i];
    switch (c.verb) {
      case 0: builder.moveTo(c.x, c.y); break;
      case 1: builder.lineTo(c.x, c.y); break;
      case 2: builder.quadTo(c.c1x, c.c1y, c.x, c.y); break;
      case 3: builder.conicTo(c.c1x, c.c1y, c.x, c.y, c.weight); break;
      case 4: builder.cubicTo(c.c1x, c.c1y, c.c2x, c.c2y, c.x, c.y); break;
      case 5: builder.close(); break;
    }
  }
  SkPaint paint = ToPaint(color, antialias);
  if (stroke) {
    paint.setStyle(SkPaint::kStroke_Style);
    paint.setStrokeWidth(width);
    paint.setStrokeMiter(miter);
    paint.setStrokeCap(cap == 1 ? SkPaint::kRound_Cap
                               : cap == 2 ? SkPaint::kSquare_Cap : SkPaint::kButt_Cap);
    paint.setStrokeJoin(join == 1 ? SkPaint::kRound_Join
                                 : join == 2 ? SkPaint::kBevel_Join : SkPaint::kMiter_Join);
    if (dashes && dash_count) {
      std::vector<SkScalar> intervals(dashes, dashes + dash_count);
      paint.setPathEffect(SkDashPathEffect::Make(
          SkSpan<const SkScalar>(intervals.data(), intervals.size()), dash_offset));
    }
  }
  if (blur_radius > 0) {
    paint.setMaskFilter(SkMaskFilter::MakeBlur(kNormal_SkBlurStyle, blur_radius * 0.5f));
  }
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->drawPath(builder.detach(), paint);
}

void LayoutngCanvasDrawRRect(void* canvas, NativeRect rect, NativeRadii radii,
                             NativeColor color, bool antialias,
                             bool has_radius) {
  SkPaint paint = ToPaint(color, antialias && has_radius);
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()->drawRRect(
      ToRRect(rect, radii), paint);
}

void LayoutngCanvasDrawDRRect(void* canvas, NativeRect outer, NativeRadii outer_radii,
                              NativeRect inner, NativeRadii inner_radii, NativeColor color,
                              bool antialias, bool has_radius,
                              float stroke_width) {
  SkCanvas* target = static_cast<NativeCanvasState*>(canvas)->surface->getCanvas();
  SkRRect outer_rrect = ToRRect(outer, outer_radii);
  if (stroke_width >= 0) {
    outer_rrect.inset(stroke_width * 0.5f, stroke_width * 0.5f);
    SkPaint paint = ToPaint(color, antialias);
    paint.setStyle(SkPaint::kStroke_Style);
    paint.setStrokeWidth(stroke_width);
    target->drawRRect(outer_rrect, paint);
  } else {
    SkPaint paint = ToPaint(color, antialias && has_radius);
    target->drawDRRect(outer_rrect, ToRRect(inner, inner_radii), paint);
  }
}

void LayoutngCanvasDrawBoxShadow(void* canvas, NativeRect box,
                                 NativeRadii radii, NativeColor color,
                                 double offset_x, double offset_y,
                                 double blur_radius, double spread,
                                 bool inset, bool opaque_background,
                                 bool antialias) {
  SkCanvas* target = static_cast<NativeCanvasState*>(canvas)->surface->getCanvas();
  SkPaint paint;
  paint.setAntiAlias(antialias);
  paint.setColor(SK_ColorBLACK);
  if (blur_radius > 0)
    paint.setMaskFilter(SkMaskFilter::MakeBlur(
        kNormal_SkBlurStyle, static_cast<float>(blur_radius * 0.5), true));
  paint.setColorFilter(SkColorFilters::Blend(
      {std::clamp(color.red, 0.0f, 1.0f), std::clamp(color.green, 0.0f, 1.0f),
       std::clamp(color.blue, 0.0f, 1.0f), std::clamp(color.alpha, 0.0f, 1.0f)},
      SkColorSpace::MakeSRGB(), SkBlendMode::kSrcIn));
  NativeRect shadow{box.x - spread, box.y - spread,
                    box.width + spread * 2, box.height + spread * 2};
  if (!inset) {
    target->save();
    NativeRect hole = box;
    NativeRadii hole_radii = radii;
    if (opaque_background) {
      hole.x += 1;
      hole.y += 1;
      hole.width = std::max(0.0, hole.width - 2);
      hole.height = std::max(0.0, hole.height - 2);
      for (double& radius : hole_radii.values)
        radius = std::max(0.0, radius - 1);
    }
    target->clipRRect(ToRRect(hole, hole_radii), SkClipOp::kDifference, true);
    target->save();
    target->translate(static_cast<float>(offset_x), static_cast<float>(offset_y));
    target->drawRRect(ToRRect(shadow, radii), paint);
    target->restore();
    target->restore();
  } else {
    shadow.x += offset_x;
    shadow.y += offset_y;
    target->drawRRect(ToRRect(shadow, radii), paint);
  }
}

int LayoutngCanvasRegisterImage(void* canvas, uint64_t id, uint32_t width,
                                uint32_t height, const uint8_t* rgba,
                                size_t byte_count) {
  auto* target = static_cast<NativeCanvasState*>(canvas);
  if (!target || !id || !width || !height || !rgba ||
      width > static_cast<uint32_t>(std::numeric_limits<int>::max()) ||
      height > static_cast<uint32_t>(std::numeric_limits<int>::max()) ||
      static_cast<size_t>(width) >
          std::numeric_limits<size_t>::max() / height / 4 ||
      byte_count != static_cast<size_t>(width) * height * 4 ||
      target->images.contains(id))
    return 0;
  std::vector<SkPMColor> pixels(static_cast<size_t>(width) * height);
  for (size_t i = 0; i < pixels.size(); ++i) {
    pixels[i] = SkPreMultiplyARGB(rgba[i * 4 + 3], rgba[i * 4],
                                  rgba[i * 4 + 1], rgba[i * 4 + 2]);
  }
  auto data = SkData::MakeWithCopy(pixels.data(), pixels.size() * sizeof(SkPMColor));
  const auto info = SkImageInfo::Make(
      static_cast<int>(width), static_cast<int>(height), kN32_SkColorType,
      kPremul_SkAlphaType, SkColorSpace::MakeSRGB());
  auto image = SkImages::RasterFromData(info, std::move(data),
                                       static_cast<size_t>(width) * 4);
  if (!image) return 0;
  target->images.emplace(id, std::move(image));
  return 1;
}

void LayoutngCanvasImageDeviceSize(void* canvas, NativeRect rect, int* width,
                                   int* height) {
  SkVector axes[] = {{static_cast<float>(rect.width), 0.f},
                     {0.f, static_cast<float>(rect.height)}};
  static_cast<NativeCanvasState*>(canvas)->surface->getCanvas()
      ->getTotalMatrix().mapVectors(axes);
  *width = std::max(1, static_cast<int>(std::round(axes[0].length())));
  *height = std::max(1, static_cast<int>(std::round(axes[1].length())));
}

int LayoutngCanvasImageDimensions(void* canvas, uint64_t id, int* width,
                                  int* height) {
  auto* state = static_cast<NativeCanvasState*>(canvas);
  const auto found = state->images.find(id);
  if (found == state->images.end() || !found->second) return 0;
  *width = found->second->width();
  *height = found->second->height();
  return 1;
}

int LayoutngCanvasBuildMipImage(void* canvas, uint64_t id, int level) {
  auto* state = static_cast<NativeCanvasState*>(canvas);
  const auto original = state->images.find(id);
  if (original == state->images.end() || !original->second ||
      level <= 0 || level >= 31)
    return 0;
  const auto key = std::pair(id, level);
  if (const auto found = state->mip_images.find(key);
      found != state->mip_images.end())
    return found->second != nullptr;
  SkPixmap source_pixels;
  if (!original->second->peekPixels(&source_pixels)) return 0;
  const int divisor = 1 << level;
  const int mip_width =
      std::max(1, (original->second->width() + divisor - 1) / divisor);
  const int mip_height =
      std::max(1, (original->second->height() + divisor - 1) / divisor);
  const SkImageInfo info =
      source_pixels.info().makeDimensions({mip_width, mip_height});
  auto data = SkData::MakeUninitialized(info.computeMinByteSize());
  if (!data) return 0;
  SkPixmap target_pixels(info, data->writable_data(), info.minRowBytes());
  if (!source_pixels.scalePixels(
          target_pixels,
          SkSamplingOptions(SkFilterMode::kLinear, SkMipmapMode::kNearest)))
    return 0;
  auto image = SkImages::RasterFromData(info, std::move(data),
                                       info.minRowBytes());
  if (!image) return 0;
  state->mip_images.emplace(key, std::move(image));
  return 1;
}

void LayoutngCanvasDrawImageRect(void* canvas, uint64_t id, int mip_level,
                                 NativeRect source, NativeRect destination) {
  auto* state = static_cast<NativeCanvasState*>(canvas);
  sk_sp<SkImage> image;
  if (mip_level > 0) {
    const auto found = state->mip_images.find(std::pair(id, mip_level));
    if (found != state->mip_images.end()) image = found->second;
  }
  if (!image) {
    const auto found = state->images.find(id);
    if (found == state->images.end()) return;
    image = found->second;
  }
  state->surface->getCanvas()->drawImageRect(
      image, ToRect(source), ToRect(destination),
      SkSamplingOptions(SkFilterMode::kLinear, SkMipmapMode::kNone), nullptr,
      SkCanvas::kStrict_SrcRectConstraint);
}

int LayoutngCanvasRegisterFont(void* canvas, const uint8_t* bytes,
                               size_t byte_count, uint32_t face_index,
                               const char* native_family, double weight,
                               bool italic, const NativeVariation* variations,
                               size_t variation_count) {
  NativeCanvasState* target = static_cast<NativeCanvasState*>(canvas);
  sk_sp<SkTypeface> typeface;
  if (target->font_manager && native_family && *native_family) {
    typeface = target->font_manager->matchFamilyStyle(
        native_family,
        SkFontStyle(static_cast<int>(weight), SkFontStyle::kNormal_Width,
                    italic ? SkFontStyle::kItalic_Slant
                           : SkFontStyle::kUpright_Slant));
  }
  if (!typeface && target->font_manager && bytes && byte_count) {
    typeface = target->font_manager->makeFromData(
        SkData::MakeWithCopy(bytes, byte_count),
        static_cast<int>(face_index & 0xffffu));
  }
  std::vector<NativeVariation> face_variations;
  if (variations && variation_count)
    face_variations.assign(variations, variations + variation_count);
  typeface = CloneWithVariations(std::move(typeface), face_index, face_variations);
  bool supports_optical_size = false;
  if (typeface) {
    const int axis_count = typeface->getVariationDesignParameters({});
    if (axis_count > 0) {
      std::vector<SkFontParameters::Variation::Axis> axes(axis_count);
      if (typeface->getVariationDesignParameters(SkSpan(axes.data(), axes.size())) == axis_count) {
        supports_optical_size = std::any_of(axes.begin(), axes.end(), [](const auto& axis) {
          return axis.tag == SkSetFourByteTag('o', 'p', 's', 'z');
        });
      }
    }
  }
  target->font_variations.push_back(std::move(face_variations));
  target->font_face_indices.push_back(face_index);
  target->font_supports_optical_size.push_back(supports_optical_size);
  target->typefaces.push_back(std::move(typeface));
  return target->typefaces.back() ? 1 : 0;
}

void LayoutngCanvasDrawGlyphRun(void* canvas, size_t face_index,
                                float font_size, const uint16_t* ids,
                                const float* xy, size_t glyph_count,
                                float origin_x, float origin_y, NativeColor color,
                                bool synthetic_bold, bool synthetic_italic,
                                uint8_t smoothing, bool stroke,
                                float stroke_width,
                                const NativeVariation* variations,
                                size_t variation_count) {
  NativeCanvasState* target = static_cast<NativeCanvasState*>(canvas);
  if (face_index >= target->typefaces.size() ||
      !target->typefaces[face_index] || !ids || !xy || !glyph_count) return;
  std::vector<NativeVariation> run_variations;
  if (variations && variation_count)
    run_variations.assign(variations, variations + variation_count);
  if (run_variations.empty())
    run_variations = target->font_variations[face_index];
  if (target->font_supports_optical_size[face_index]) {
    constexpr uint32_t kOpsz = SkSetFourByteTag('o', 'p', 's', 'z');
    auto optical = std::find_if(run_variations.begin(), run_variations.end(),
                                [](const auto& axis) { return axis.tag == kOpsz; });
    if (optical == run_variations.end())
      run_variations.push_back({kOpsz, font_size});
    else
      optical->value = font_size;
  }
  sk_sp<SkTypeface> typeface = CloneWithVariations(
      target->typefaces[face_index], target->font_face_indices[face_index], run_variations);
  SkFont font(typeface, font_size);
  font.setEdging(smoothing == 1 ? SkFont::Edging::kAlias
                 : smoothing == 2 ? SkFont::Edging::kAntiAlias
                                  : SkFont::Edging::kSubpixelAntiAlias);
  font.setEmbeddedBitmaps(false);
  font.setSubpixel(true);
  font.setLinearMetrics(true);
  if (smoothing == 2) font.setHinting(SkFontHinting::kNone);
  font.setEmbolden(synthetic_bold);
  if (synthetic_italic) font.setSkewX(-0.25f);
  SkTextBlobBuilder builder;
  bool horizontal = true;
  for (size_t i = 0; i < glyph_count; ++i) horizontal &= xy[i * 2 + 1] == 0;
  if (horizontal) {
    const auto buffer = builder.allocRunPosH(font, static_cast<int>(glyph_count), 0);
    for (size_t i = 0; i < glyph_count; ++i) {
      buffer.glyphs[i] = ids[i];
      buffer.pos[i] = xy[i * 2];
    }
  } else {
    const auto buffer = builder.allocRunPos(font, static_cast<int>(glyph_count));
    for (size_t i = 0; i < glyph_count; ++i) {
      buffer.glyphs[i] = ids[i];
      buffer.points()[i] = {xy[i * 2], xy[i * 2 + 1]};
    }
  }
  const auto blob = builder.make();
  if (!blob) return;
  SkPaint paint = ToPaint(color, true);
  if (stroke) {
    paint.setStyle(SkPaint::kStroke_Style);
    paint.setStrokeWidth(stroke_width);
  }
  target->surface->getCanvas()->drawTextBlob(blob, origin_x, origin_y, paint);
}

void LayoutngCanvasClipOutGlyphRunIntercepts(
    void* canvas, size_t face_index, float font_size, const uint16_t* ids,
    const float* xy, size_t glyph_count, float origin_x, float origin_y,
    bool synthetic_bold, bool synthetic_italic,
    const NativeVariation* variations, size_t variation_count,
    float decoration_top, float decoration_height, float dilation) {
  NativeCanvasState* target = static_cast<NativeCanvasState*>(canvas);
  if (face_index >= target->typefaces.size() ||
      !target->typefaces[face_index] || !ids || !xy || !glyph_count ||
      decoration_height <= 0) {
    return;
  }
  std::vector<NativeVariation> run_variations;
  if (variations && variation_count)
    run_variations.assign(variations, variations + variation_count);
  if (run_variations.empty())
    run_variations = target->font_variations[face_index];
  if (target->font_supports_optical_size[face_index]) {
    constexpr uint32_t kOpsz = SkSetFourByteTag('o', 'p', 's', 'z');
    auto optical = std::find_if(run_variations.begin(), run_variations.end(),
                                [](const auto& axis) { return axis.tag == kOpsz; });
    if (optical == run_variations.end())
      run_variations.push_back({kOpsz, font_size});
    else
      optical->value = font_size;
  }
  sk_sp<SkTypeface> typeface = CloneWithVariations(
      target->typefaces[face_index], target->font_face_indices[face_index],
      run_variations);
  SkFont font(typeface, font_size);
  font.setEmbeddedBitmaps(false);
  font.setSubpixel(true);
  font.setLinearMetrics(true);
  font.setEmbolden(synthetic_bold);
  if (synthetic_italic) font.setSkewX(-0.25f);

  SkTextBlobBuilder builder;
  bool horizontal = true;
  for (size_t i = 0; i < glyph_count; ++i) horizontal &= xy[i * 2 + 1] == 0;
  if (horizontal) {
    const auto buffer =
        builder.allocRunPosH(font, static_cast<int>(glyph_count), 0);
    for (size_t i = 0; i < glyph_count; ++i) {
      buffer.glyphs[i] = ids[i];
      buffer.pos[i] = xy[i * 2];
    }
  } else {
    const auto buffer = builder.allocRunPos(font, static_cast<int>(glyph_count));
    for (size_t i = 0; i < glyph_count; ++i) {
      buffer.glyphs[i] = ids[i];
      buffer.points()[i] = {xy[i * 2], xy[i * 2 + 1]};
    }
  }
  const auto blob = builder.make();
  if (!blob) return;

  // Match TextPainter::ClipDecorationLine: ignore intersections smaller than
  // half a CSS pixel, then dilate horizontally by the decoration thickness and
  // vertically by one pixel before clipping them out.
  const SkScalar bounds[2] = {
      decoration_top + 0.5f - origin_y,
      decoration_top + decoration_height - 0.5f - origin_y,
  };
  const int count = blob->getIntercepts(bounds, nullptr, nullptr);
  if (count <= 0) return;
  std::vector<SkScalar> intervals(static_cast<size_t>(count));
  blob->getIntercepts(bounds, intervals.data(), nullptr);
  SkCanvas* sk_canvas = target->surface->getCanvas();
  for (int i = 0; i + 1 < count; i += 2) {
    const float left = origin_x + intervals[i] - dilation;
    const float right = origin_x + intervals[i + 1] + dilation;
    sk_canvas->clipRect(
        SkRect::MakeLTRB(left, decoration_top - 1.0f, right,
                         decoration_top + decoration_height + 1.0f),
        SkClipOp::kDifference, false);
  }
}

int LayoutngCanvasReadRgba(void* canvas, uint8_t* rgba, size_t length) {
  NativeCanvasState* target = static_cast<NativeCanvasState*>(canvas);
  const int width = target->surface->width();
  const int height = target->surface->height();
  if (!rgba || length != static_cast<size_t>(width) * height * 4) return 0;
  const SkImageInfo info = SkImageInfo::Make(
      width, height, kRGBA_8888_SkColorType, kUnpremul_SkAlphaType,
      SkColorSpace::MakeSRGB());
  return target->surface->readPixels(info, rgba,
                                     static_cast<size_t>(width) * 4, 0, 0) ? 1 : 0;
}
}
