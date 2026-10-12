#include <bit>
#include <cstdint>
#include <iostream>

#include "blink_geometry/geometry/physical_offset.h"
#include "gfx_geometry/point_conversions.h"

void PrintPoint(const char* name, gfx::Point value) {
  std::cout << name << '\t' << value.x() << '\t' << value.y() << '\t'
            << value.IsOrigin() << '\t' << value.ToString() << '\n';
}

void PrintVector(const char* name, gfx::Vector2d value) {
  std::cout << name << '\t' << value.x() << '\t' << value.y() << '\t'
            << value.IsZero() << '\t' << value.LengthSquared() << '\t'
            << std::bit_cast<uint32_t>(value.Length()) << '\t' << value.ToString()
            << '\n';
}

void PrintOffset(const char* name, blink::PhysicalOffset value) {
  std::cout << name << '\t' << value.left.RawValue() << '\t'
            << value.top.RawValue() << '\t' << value.IsZero() << '\t'
            << value.HasFraction() << '\n';
}

int main() {
  gfx::Point point(10, -20);
  PrintPoint("point0", point);
  point.Offset(3, 4);
  PrintPoint("point1", point);
  point += gfx::Vector2d(5, -2);
  PrintPoint("point2", point);
  PrintPoint("point3", gfx::ScaleToRoundedPoint(point, 1.25f, -0.5f));
  PrintPoint("point4", gfx::ToFlooredPoint(gfx::PointF(-1.25f, 2.75f)));
  PrintPoint("point5", gfx::ToCeiledPoint(gfx::PointF(-1.25f, 2.75f)));

  gfx::Vector2d vector(3, 4);
  PrintVector("vector0", vector);
  vector.Add(gfx::Vector2d(-10, 2));
  PrintVector("vector1", vector);
  PrintVector("vector2", -vector);

  const gfx::Vector2dF vector_f(3.f, 4.f);
  std::cout << "vector_f\t" << std::bit_cast<uint32_t>(vector_f.Length())
            << '\t' << std::bit_cast<uint32_t>(vector_f.SlopeAngleRadians())
            << '\t' << gfx::CrossProduct(vector_f, gfx::Vector2dF(2.f, 5.f))
            << '\t' << gfx::DotProduct(vector_f, gfx::Vector2dF(2.f, 5.f))
            << '\t' << vector_f.ToString() << '\n';

  PrintOffset("offset0", blink::PhysicalOffset());
  auto offset = blink::PhysicalOffset::FromPointFRound(gfx::PointF(1.25f, -2.75f));
  PrintOffset("offset1", offset);
  PrintPoint("offset_point", gfx::ToRoundedPoint(gfx::PointF(offset)));
  offset.Scale(2.f);
  PrintOffset("offset2", offset);
  PrintOffset("offset3", -offset);
}
