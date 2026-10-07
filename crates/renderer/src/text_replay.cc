// Narrow ABI between Rust display items and the existing Skia raster library.
// Glyph IDs and positions are already shaped by LayoutNG; this only replays
// the source skia_renderer.cc DrawGlyphs operation on an RGBA surface.
#include <cstddef>
#include <cstdint>
#include <vector>

#include "include/core/SkCanvas.h"
#include "include/core/SkData.h"
#include "include/core/SkFont.h"
#include "include/core/SkFontMgr.h"
#include "include/core/SkGraphics.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkPaint.h"
#include "include/core/SkSurface.h"
#include "include/core/SkSurfaceProps.h"
#include "include/core/SkTextBlob.h"
#include "include/ports/SkFontMgr_mac_ct.h"

extern "C" int LayoutngDrawGlyphRun(
    uint8_t* rgba, int width, int height, const uint8_t* font_bytes,
    size_t font_length, int face_index, float font_size,
    const uint16_t* glyph_ids, const float* glyph_xy, size_t glyph_count,
    float origin_x, float origin_y, float red, float green, float blue,
    float alpha, uint8_t synthetic_bold, uint8_t synthetic_italic,
    uint8_t smoothing) {
  if (!rgba || width <= 0 || height <= 0 || !font_bytes || !font_length ||
      !glyph_ids || !glyph_xy || !glyph_count) return 0;
  SkGraphics::Init();
  const SkImageInfo info = SkImageInfo::Make(
      width, height, kRGBA_8888_SkColorType, kOpaque_SkAlphaType);
  const SkSurfaceProps props(0, kUnknown_SkPixelGeometry);
  const auto surface = SkSurfaces::WrapPixels(
      info, rgba, static_cast<size_t>(width) * 4, &props);
  if (!surface) return 0;
  const auto manager = SkFontMgr_New_CoreText(nullptr);
  if (!manager) return 0;
  const auto typeface = manager->makeFromData(
      SkData::MakeWithCopy(font_bytes, font_length), face_index);
  if (!typeface) return 0;
  SkFont font(typeface, font_size);
  font.setEdging(smoothing == 1 ? SkFont::Edging::kAlias
                 : smoothing == 2 ? SkFont::Edging::kAntiAlias
                                  : SkFont::Edging::kSubpixelAntiAlias);
  font.setEmbeddedBitmaps(false);
  font.setSubpixel(true);
  font.setLinearMetrics(true);
  if (smoothing == 2) font.setHinting(SkFontHinting::kNone);
  font.setEmbolden(synthetic_bold != 0);
  if (synthetic_italic) font.setSkewX(-0.25f);
  SkTextBlobBuilder builder;
  bool horizontal = true;
  for (size_t index = 0; index < glyph_count; ++index)
    horizontal &= glyph_xy[index * 2 + 1] == 0;
  if (horizontal) {
    const auto buffer = builder.allocRunPosH(font, static_cast<int>(glyph_count), 0);
    for (size_t index = 0; index < glyph_count; ++index) {
      buffer.glyphs[index] = glyph_ids[index];
      buffer.pos[index] = glyph_xy[index * 2];
    }
  } else {
    const auto buffer = builder.allocRunPos(font, static_cast<int>(glyph_count));
    for (size_t index = 0; index < glyph_count; ++index) {
      buffer.glyphs[index] = glyph_ids[index];
      buffer.points()[index] = {glyph_xy[index * 2], glyph_xy[index * 2 + 1]};
    }
  }
  const auto blob = builder.make();
  if (!blob) return 0;
  SkPaint paint;
  paint.setAntiAlias(true);
  paint.setColor4f({red, green, blue, alpha});
  surface->getCanvas()->drawTextBlob(blob, origin_x, origin_y, paint);
  return 1;
}
