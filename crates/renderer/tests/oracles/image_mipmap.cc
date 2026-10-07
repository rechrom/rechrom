// Test oracle only. Reproduce skia_renderer.cc:646-669 against the source
// project's Skia archive; no C++ runtime is used by the Rust implementation.
#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <vector>
#include "include/core/SkColor.h"
#include "include/core/SkColorSpace.h"
#include "include/core/SkImageInfo.h"
#include "include/core/SkPixmap.h"
#include "include/core/SkSamplingOptions.h"

void Word(uint32_t value) {
  for (int shift = 0; shift < 32; shift += 8) std::putchar(value >> shift);
}

int main() {
  for (uint32_t width = 1; width <= 19; ++width) {
    for (uint32_t height = 1; height <= 19; ++height) {
      for (uint32_t alpha = 0; alpha < 2; ++alpha) {
        std::vector<SkPMColor> pixels(width * height);
        for (uint32_t y = 0; y < height; ++y) {
          for (uint32_t x = 0; x < width; ++x) {
            pixels[y * width + x] = SkPreMultiplyARGB(
                alpha ? (x * 43 + y * 61 + 29) % 256 : 255,
                (x * 37 + y * 17 + 11) % 256,
                (x * 13 + y * 53 + 71) % 256,
                (x * 73 + y * 29 + 19) % 256);
          }
        }
        const auto info = SkImageInfo::Make(width, height, kN32_SkColorType,
                                           kPremul_SkAlphaType,
                                           SkColorSpace::MakeSRGB());
        const SkPixmap source(info, pixels.data(), width * 4);
        for (uint32_t level = 1; level <= 5; ++level) {
          const uint32_t divisor = 1u << level;
          const uint32_t w = std::max(1u, (width + divisor - 1) / divisor);
          const uint32_t h = std::max(1u, (height + divisor - 1) / divisor);
          std::vector<SkPMColor> output(w * h);
          const SkPixmap target(info.makeWH(w, h), output.data(), w * 4);
          if (!source.scalePixels(target, SkSamplingOptions(
                  SkFilterMode::kLinear, SkMipmapMode::kNearest))) return 1;
          Word(width); Word(height); Word(alpha); Word(level); Word(w); Word(h);
          for (auto pixel : output) {
            std::putchar(pixel >> SK_R32_SHIFT);
            std::putchar(pixel >> SK_G32_SHIFT);
            std::putchar(pixel >> SK_B32_SHIFT);
            std::putchar(pixel >> SK_A32_SHIFT);
          }
        }
      }
    }
  }
}
