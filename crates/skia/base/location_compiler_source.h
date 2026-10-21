// Actual native compiler source defaults. Include at the native invocation site;
// never substitute the bridge TU's own file/function/line for a Rust call site.
#ifndef CHROMIUM_LOCATION_COMPILER_SOURCE_BINDING_H_
#define CHROMIUM_LOCATION_COMPILER_SOURCE_BINDING_H_
struct ChromiumNativeSourceInput {
    const char* function;
    const char* file;
    int line;
};
constexpr ChromiumNativeSourceInput ChromiumNativeSourceDefaults(
    const char* function = __builtin_FUNCTION(),
    const char* file = __builtin_FILE(),
    int line = __builtin_LINE()) {
    return {function, file, line};
}
#endif
