// Reference output for the mechanical Rust port of layout_unit.h.
// Compile against the copied C++ foundation to refresh layout_unit_raw_reference.tsv.
#include "blink_geometry/geometry/layout_unit.h"

#include <cstdint>
#include <iostream>

int main() {
  const auto print = [](int32_t raw) {
    const blink::LayoutUnit unit = blink::LayoutUnit::FromRawValue(raw);
    std::cout << raw << '\t' << unit.ToInt() << '\t' << unit.Ceil()
              << '\t' << unit.Round() << '\t' << unit.Floor()
              << '\t' << unit.Fraction().RawValue()
              << '\t' << unit.HasFraction()
              << '\t' << unit.ClampNegativeToZero().RawValue()
              << '\t' << unit.ClampPositiveToZero().RawValue()
              << '\t' << unit.AddEpsilon().RawValue() << '\n';
  };
  for (int32_t raw : {INT32_MIN, -100001, -65, -64, -63, -33, -32,
                      -31, -1, 0, 1, 31, 32, 33, 63, 64, 65,
                      100001, INT32_MAX}) {
    print(raw);
  }
  for (int32_t raw = -100000; raw <= 100000; raw += 997) {
    print(raw);
  }
}

