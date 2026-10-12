#include <iostream>

#include "blink_geometry/geometry/margin_strut.h"

int main() {
  blink::MarginStrut strut;
  auto emit = [&](int step) {
    std::cout << step << '\t' << strut.positive_margin.RawValue() << '\t'
              << strut.negative_margin.RawValue() << '\t'
              << strut.quirky_positive_margin.RawValue() << '\t'
              << strut.is_quirky_container_start << '\t' << strut.discard_margins
              << '\t' << strut.trim_leading_margins << '\t'
              << strut.Sum().RawValue() << '\t'
              << strut.QuirkyContainerSum().RawValue() << '\t'
              << strut.IsEmpty() << '\n';
  };

  emit(0);
  strut.Append(blink::LayoutUnit::FromRawValue(128), false);
  emit(1);
  strut.Append(blink::LayoutUnit::FromRawValue(-64), false);
  emit(2);
  strut.Append(blink::LayoutUnit::FromRawValue(256), true);
  emit(3);
  strut.is_quirky_container_start = true;
  strut.Append(blink::LayoutUnit::FromRawValue(512), true);
  emit(4);
  strut.trim_leading_margins = true;
  strut.Append(blink::LayoutUnit::FromRawValue(-256), false);
  emit(5);
  strut.discard_margins = true;
  emit(6);
  blink::MarginStrut copy = strut;
  std::cout << "equal\t" << (copy == strut) << '\n';
  copy.trim_leading_margins = false;
  std::cout << "different\t" << (copy == strut) << '\n';
}
