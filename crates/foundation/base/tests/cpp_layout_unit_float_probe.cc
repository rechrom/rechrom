// Reference output for floating-point conversion in layout_unit.h.
#include "blink_geometry/geometry/layout_unit.h"

#include <bit>
#include <cstdint>
#include <iostream>

int main() {
  for (uint32_t bits : {0u, 0x80000000u, 0x3dcccccdu, 0xbdcccccdu,
                        0x3fc00000u, 0xbfc00000u, 0x7f800000u,
                        0xff800000u, 0x7fc00000u, 0x4bffffffu}) {
    const float value = std::bit_cast<float>(bits);
    std::cout << "f32\t" << bits << '\t'
              << blink::LayoutUnit(value).RawValue() << '\t'
              << blink::LayoutUnit::FromFloatCeil(value).RawValue() << '\t'
              << blink::LayoutUnit::FromFloatFloor(value).RawValue() << '\t'
              << blink::LayoutUnit::FromFloatRound(value).RawValue() << '\n';
  }
  for (uint64_t bits : {0ull, 0x8000000000000000ull, 0x3fb999999999999aull,
                        0xbfb999999999999aull, 0x3ff8000000000000ull,
                        0xbff8000000000000ull, 0x7ff0000000000000ull,
                        0xfff0000000000000ull, 0x7ff8000000000000ull,
                        0x417fffffffc00000ull}) {
    const double value = std::bit_cast<double>(bits);
    std::cout << "f64\t" << bits << '\t'
              << blink::LayoutUnit(value).RawValue() << '\t'
              << blink::LayoutUnit::FromDoubleRound(value).RawValue() << '\n';
  }
}

