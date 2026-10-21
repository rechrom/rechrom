// Compiler primitive only. No Chromium Location algorithm is implemented here.
#include "build/build_config.h"
#if defined(COMPILER_GCC)
extern "C" const void* chromium_location_extract_return_address(const void* address) {
    return __builtin_extract_return_addr(const_cast<void*>(address));
}
#endif
