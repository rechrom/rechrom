// Reference output for the operators in layout_unit.h.
#include "blink_geometry/geometry/layout_unit.h"

#include <cstdint>
#include <iostream>

int main() {
  const auto print = [](int32_t a_raw, int32_t b_raw) {
    const blink::LayoutUnit a = blink::LayoutUnit::FromRawValue(a_raw);
    const blink::LayoutUnit b = blink::LayoutUnit::FromRawValue(b_raw);
    const blink::LayoutUnit divisor(2);
    std::cout << a_raw << '\t' << b_raw
              << '\t' << (a + b).RawValue()
              << '\t' << (a - b).RawValue()
              << '\t' << (-a).RawValue()
              << '\t' << (a * b).RawValue()
              << '\t' << (a / b).RawValue()
              << '\t' << (a % b).RawValue()
              << '\t' << a.MulDiv(b, divisor).RawValue()
              << '\t' << blink::SnapSizeToPixel(a, b)
              << '\t' << blink::SnapSizeToPixelAllowingZero(a, b)
              << '\n';
  };
  print(INT32_MIN, 1);
  print(INT32_MAX, -1);
  print(64, 64);
  print(-64, 64);
  print(33, 32);
  print(-33, 32);
  print(1, 1);
  print(-1, -1);
  uint32_t state = 0x12345678u;
  for (int i = 0; i < 200; ++i) {
    state = state * 1664525u + 1013904223u;
    const int32_t a_raw = static_cast<int32_t>(state);
    state = state * 1664525u + 1013904223u;
    const int32_t b_raw = static_cast<int32_t>(state | 1u);
    print(a_raw, b_raw);
  }
}

