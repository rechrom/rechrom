// Reference output for FixedPoint::ToString in layout_unit.cc.
#include "blink_geometry/geometry/layout_unit.h"
#include "blink_base/wtf/text/string_statics.h"

#include <cstdint>
#include <iostream>

template <typename Unit, typename Storage>
void Print(const char* name, Storage raw) {
  const Unit unit = Unit::FromRawValue(raw);
  std::cout << name << '\t' << raw << '\t' << unit << '\n';
}

int main() {
  blink::InitStringStatics();
  for (int32_t raw : {INT32_MIN, -123456789, -65537, -64, -1,
                      0, 1, 32, 63, 64, 65, 65537, 123456789, INT32_MAX}) {
    Print<blink::LayoutUnit>("layout", raw);
    Print<blink::TextRunLayoutUnit>("text", raw);
  }
  for (int64_t raw : {INT64_MIN, -12345678901234567ll, -65537ll, -1ll,
                      0ll, 1ll, 32768ll, 65535ll, 65536ll,
                      12345678901234567ll, INT64_MAX}) {
    Print<blink::InlineLayoutUnit>("inline", raw);
  }
  uint64_t state = 0x9e3779b97f4a7c15ull;
  for (int i = 0; i < 256; ++i) {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    const int32_t raw = static_cast<int32_t>(state);
    Print<blink::LayoutUnit>("layout", raw);
    Print<blink::TextRunLayoutUnit>("text", raw);
    Print<blink::InlineLayoutUnit>("inline", static_cast<int64_t>(state));
  }
}
