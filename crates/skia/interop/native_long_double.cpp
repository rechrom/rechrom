// External compiler/standard-library primitive binding; no Skia or Time algorithm.
#include <cfloat>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
extern "C" {
struct SkNativeLongDoubleFormat {
    std::size_t size;
    std::size_t alignment;
    std::int32_t radix;
    std::int32_t mantissa_digits;
    std::int32_t minimum_exponent;
    std::int32_t maximum_exponent;
};
void sk_native_ld_format(SkNativeLongDoubleFormat* out) {
    *out = {sizeof(long double), alignof(long double), FLT_RADIX,
            LDBL_MANT_DIG, LDBL_MIN_EXP, LDBL_MAX_EXP};
}
// All four actual complete primitive objects live through the synchronous call.
// Their bytes and alignment never become a guessed Rust storage declaration.
using SkNativeLongDoubleSlots = void (*)(void*, void*, void*, void*, void*);
void sk_native_ld_with_slots(void* context, SkNativeLongDoubleSlots callback) {
    long double a{}, b{}, c{}, d{};
    callback(context, &a, &b, &c, &d);
}
void sk_native_ld_copy(void* out, const void* x) {
    *static_cast<long double*>(out) = *static_cast<const long double*>(x);
}
#define FROM(name, type) \
void sk_native_ld_from_##name(void* out, type x) { \
    *static_cast<long double*>(out) = static_cast<long double>(x); \
}
FROM(f32, float)
FROM(f64, double)
FROM(i64, std::int64_t)
FROM(u64, std::uint64_t)
#undef FROM
#define TO(name, type) \
type sk_native_ld_to_##name(const void* x) { \
    return static_cast<type>(*static_cast<const long double*>(x)); \
}
TO(f32, float)
TO(f64, double)
TO(i32, std::int32_t)
TO(i64, std::int64_t)
TO(u64, std::uint64_t)
#undef TO
#define BINARY(name, op) \
void sk_native_ld_##name(void* out, const void* x, const void* y) { \
    *static_cast<long double*>(out) = *static_cast<const long double*>(x) op \
                                    *static_cast<const long double*>(y); \
}
BINARY(add, +)
BINARY(sub, -)
BINARY(mul, *)
BINARY(div, /)
#undef BINARY
#define COMPARE(name, op) \
std::uint8_t sk_native_ld_##name(const void* x, const void* y) { \
    return *static_cast<const long double*>(x) op *static_cast<const long double*>(y); \
}
COMPARE(eq, ==)
COMPARE(ne, !=)
COMPARE(lt, <)
COMPARE(le, <=)
COMPARE(gt, >)
COMPARE(ge, >=)
#undef COMPARE
#define UNARY(name) \
void sk_native_ld_##name(void* out, const void* x) { \
    *static_cast<long double*>(out) = std::name(*static_cast<const long double*>(x)); \
}
UNARY(floor)
UNARY(ceil)
UNARY(round)
UNARY(trunc)
#undef UNARY
// Genuine C vararg promotion/long-double operand remains inside its native TU.
int sk_native_ld_snprintf(char* out, std::size_t size, const char* format, const void* x) {
    return std::snprintf(out, size, format, *static_cast<const long double*>(x));
}
}
