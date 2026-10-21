// Source: base/numerics/safe_conversions_impl.h:83-94, MSVC branch.
// Language primitive binding only: the original resumable compiler intrinsic.
#if !defined(_MSC_VER)
#error "native __debugbreak bridge requires the actual _MSC_VER compiler domain"
#endif
#include <intrin.h>
extern "C" void rechrom_numeric_debugbreak() {
  __debugbreak();
}
