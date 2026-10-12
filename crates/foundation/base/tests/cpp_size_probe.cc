#include <bit>
#include <cstdint>
#include <iostream>

#include "blink_geometry/geometry/physical_size.h"
#include "gfx_geometry/size_conversions.h"

void PrintSize(const char* name, const gfx::Size& value) {
  std::cout << name << '\t' << value.width() << '\t' << value.height() << '\t'
            << value.IsEmpty() << '\t' << value.IsZero() << '\t'
            << value.Area64() << '\t' << value.ToString() << '\n';
}

void PrintSizeF(const char* name, const gfx::SizeF& value) {
  std::cout << name << '\t' << value.width() << '\t' << value.height() << '\t'
            << value.IsEmpty() << '\t' << value.IsZero() << '\t'
            << value.GetArea() << '\t' << value.ToString() << '\n';
}

void PrintPhysical(const char* name, const blink::PhysicalSize& value) {
  std::cout << name << '\t' << value.width.RawValue() << '\t'
            << value.height.RawValue() << '\t' << value.IsEmpty() << '\t'
            << value.IsZero() << '\t' << value.HasFraction() << '\n';
}

int main() {
  PrintSize("size0", gfx::Size(-3, 4));
  gfx::Size size(7, 11);
  PrintSize("size1", size);
  size.Enlarge(3, -2);
  PrintSize("size2", size);
  PrintSize("size3", gfx::ScaleToCeiledSize(size, 1.25f, 0.5f));
  PrintSize("size4", gfx::ScaleToFlooredSize(size, 1.25f, 0.5f));
  PrintSize("size5", gfx::ScaleToRoundedSize(size, 1.25f, 0.5f));
  size.SetToMax(gfx::Size(100, 2));
  PrintSize("size6", size);
  size.Transpose();
  PrintSize("size7", size);

  PrintSizeF("sizef0", gfx::SizeF(-1.f, 0.0000001f));
  gfx::SizeF size_f(2.5f, 3.75f);
  PrintSizeF("sizef1", size_f);
  size_f.Enlarge(1.25f, -0.5f);
  PrintSizeF("sizef2", size_f);
  size_f.Scale(2.f, 0.5f);
  PrintSizeF("sizef3", size_f);
  PrintSize("sizef_round", gfx::ToRoundedSize(size_f));

  PrintPhysical("physical0", blink::PhysicalSize());
  blink::PhysicalSize physical(blink::LayoutUnit(300), blink::LayoutUnit(200));
  blink::PhysicalSize aspect(blink::LayoutUnit(16), blink::LayoutUnit(9));
  PrintPhysical("physical1", physical);
  PrintPhysical("physical_shrink",
                physical.FitToAspectRatio(aspect, blink::kAspectRatioFitShrink));
  PrintPhysical("physical_grow",
                physical.FitToAspectRatio(aspect, blink::kAspectRatioFitGrow));
  for (const auto& ratio : {gfx::SizeF(16.f, 9.f), gfx::SizeF(1.6f, 1.f),
                            gfx::SizeF(355.f, 113.f), gfx::SizeF(0.f, 0.f),
                            gfx::SizeF(1.4142135f, 1.f)}) {
    PrintPhysical("ratio", blink::LayoutRatioFromSizeF(ratio));
  }
  uint32_t state = 0x97e15a3bu;
  for (int i = 0; i < 256; ++i) {
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    const float width = static_cast<float>((state & 0xffff) + 1) / 37.f;
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    const float height = static_cast<float>((state & 0xffff) + 1) / 53.f;
    const auto converted = blink::LayoutRatioFromSizeF(gfx::SizeF(width, height));
    std::cout << "ratio_case\t" << std::bit_cast<uint32_t>(width) << '\t'
              << std::bit_cast<uint32_t>(height) << '\t'
              << converted.width.RawValue() << '\t'
              << converted.height.RawValue() << '\n';
  }
}
