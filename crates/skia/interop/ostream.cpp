// External C++ standard-library binding. Compile with the exact same C++ SDK,
// ABI flags, allocator and runtime as Chromium base. No compiler probe is run by
// this source delivery; sizeof/alignof are evaluated by the real configured TU.
#include <cstddef>
#include <cstdint>
#include <iomanip>
#include <ios>
#include <locale>
#include <new>
#include <ostream>
#include <sstream>
#include <string>
#include <string_view>
#include "base/base_export.h"

// Generated ONLY from the supplied genuine SDK ABI metadata. The root build
// must compile this exact TU successfully before publishing the Rust cfg.
#include "rechrom_native_ostream_abi.h"
#ifndef RECHROM_NATIVE_OSTREAM_ABI_METADATA_AVAILABLE
#error "No supplied genuine native ostream ABI metadata"
#endif
static_assert(sizeof(std::ostringstream) == RECHROM_NATIVE_OSTRINGSTREAM_SIZE,
              "supplied ostringstream size differs from this actual native TU");
static_assert(alignof(std::ostringstream) == RECHROM_NATIVE_OSTRINGSTREAM_ALIGN,
              "supplied ostringstream alignment differs from this actual native TU");
static_assert(sizeof(std::string) == RECHROM_NATIVE_STD_STRING_SIZE,
              "supplied std::string size differs from this actual native TU");
static_assert(alignof(std::string) == RECHROM_NATIVE_STD_STRING_ALIGN,
              "supplied std::string alignment differs from this actual native TU");

#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
#error "This C boundary requires the actual nonthrowing Chromium no-exceptions native domain; exception-enabled SDK transport is not implemented."
#endif

namespace {
std::ostringstream& Stream(void* p) { return *static_cast<std::ostringstream*>(p); }
const std::ostringstream& Stream(const void* p) { return *static_cast<const std::ostringstream*>(p); }
std::ostream& Out(void* p) { return *static_cast<std::ostream*>(p); }
}

// Source: base/logging.h:553-554 (actual native storage type dependency)
extern "C" BASE_EXPORT size_t rechrom_ostringstream_size() { return sizeof(std::ostringstream); }
extern "C" BASE_EXPORT size_t rechrom_ostringstream_align() { return alignof(std::ostringstream); }
extern "C" BASE_EXPORT void rechrom_ostringstream_construct(void* p) { ::new (p) std::ostringstream(); }
extern "C" BASE_EXPORT void rechrom_ostringstream_destroy(void* p) { Stream(p).~basic_ostringstream(); }
// Source: base/logging.h:534
// C++ performs the actual base-subobject adjustment. Never reinterpret a Rust
// storage pointer as ostream, and never guess the native inheritance layout.
extern "C" BASE_EXPORT void* rechrom_ostringstream_ostream(void* p) {
  return static_cast<std::ostream*>(&Stream(p));
}
// Source: base/logging.cc:732; base/logging.cc:1000
extern "C" BASE_EXPORT void rechrom_ostringstream_view(const void* p,const uint8_t** data,size_t* len) {
  auto v=Stream(p).view();
  *data=reinterpret_cast<const uint8_t*>(v.data()); *len=v.size();
}
// Source: base/logging.h:537; base/logging.cc:764
extern "C" BASE_EXPORT size_t rechrom_std_string_size() { return sizeof(std::string); }
extern "C" BASE_EXPORT size_t rechrom_std_string_align() { return alignof(std::string); }
extern "C" BASE_EXPORT void rechrom_ostringstream_str_at(const void* p,void* dst) {
  ::new(dst) std::string(Stream(p).str());
}
extern "C" BASE_EXPORT void rechrom_std_string_destroy(void* p) {
  static_cast<std::string*>(p)->~basic_string();
}
extern "C" BASE_EXPORT void rechrom_std_string_view(const void* p,const uint8_t** data,size_t* len) {
  const auto& s=*static_cast<const std::string*>(p);
  *data=reinterpret_cast<const uint8_t*>(s.data()); *len=s.size();
}
// Source: base/logging.cc:765 (actual std::string::substr value)
extern "C" BASE_EXPORT void rechrom_std_string_substr_at(const void* p,size_t start,void* dst) {
  ::new(dst) std::string(static_cast<const std::string*>(p)->substr(start));
}

// Concrete standard operator domains. Every operation executes on the original
// referenced ostream, preserving its locale, flags, width, fill and error state.
// Source: base/logging.cc:955-998; base/check.cc:255-336
extern "C" BASE_EXPORT void rechrom_ostream_char(void* p,char x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_cstr(void* p,const char* x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_bytes(void* p,const uint8_t* x,size_t n) {
  Out(p)<<std::string_view(reinterpret_cast<const char*>(x),n);
}
extern "C" BASE_EXPORT void rechrom_ostream_string(void* p,const void* x) {
  Out(p)<<*static_cast<const std::string*>(x);
}
extern "C" BASE_EXPORT void rechrom_ostream_bool(void* p,bool x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_short(void* p,short x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_ushort(void* p,unsigned short x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_int(void* p,int x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_uint(void* p,unsigned int x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_long(void* p,long x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_ulong(void* p,unsigned long x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_llong(void* p,long long x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_ullong(void* p,unsigned long long x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_float(void* p,float x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_double(void* p,double x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_pointer(void* p,const void* x) { Out(p)<<x; }
extern "C" BASE_EXPORT void rechrom_ostream_size(void* p,size_t x) { Out(p)<<x; }
// Source: base/logging.cc:969-987
extern "C" BASE_EXPORT void rechrom_ostream_setfill(void* p,char x) { Out(p)<<std::setfill(x); }
extern "C" BASE_EXPORT void rechrom_ostream_setw(void* p,int x) { Out(p)<<std::setw(x); }
// Source: base/logging.cc:737; base/logging.cc:763
extern "C" BASE_EXPORT void rechrom_ostream_endl(void* p) { Out(p)<<std::endl; }

// Genuine standard-library primitive binding, not a StackTrace implementation.
// No native string layout, buffer, owning callback or symbolization replacement.
#include <ostream>
#include <stddef.h>
#include <utility>
#include <string>
#include "base/base_export.h"
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
#error "Actual Chromium no-exceptions stdlib domain required; cross-language exception transport is not supplied."
#endif
extern "C" BASE_EXPORT void rechrom_stack_message_delete(void* message) {
  delete static_cast<std::string*>(message);
}
extern "C" BASE_EXPORT void* rechrom_stack_message_move_new(void* message) {
  return new std::string(std::move(*static_cast<std::string*>(message)));
}
#include <new>
#include <string_view>
extern "C" BASE_EXPORT void rechrom_debug_string_cstr_construct(void* dst,const char* s) {new(dst) std::string(s);}
extern "C" BASE_EXPORT void rechrom_debug_string_move_construct(void* dst,void* s) {new(dst) std::string(std::move(*static_cast<std::string*>(s)));}
extern "C" BASE_EXPORT const char* rechrom_debug_string_cstr(const void* s) {return static_cast<const std::string*>(s)->c_str();}
extern "C" BASE_EXPORT bool rechrom_debug_string_find(const void* s,const unsigned char* n,size_t length,size_t start,size_t* out) {
  size_t result=static_cast<const std::string*>(s)->find(reinterpret_cast<const char*>(n),start,length);
  if(result==std::string::npos)return false;*out=result;return true;
}
extern "C" BASE_EXPORT bool rechrom_debug_string_find_first_not_of(const void* s,const unsigned char* n,size_t length,size_t start,size_t* out) {
  size_t result=static_cast<const std::string*>(s)->find_first_not_of(reinterpret_cast<const char*>(n),start,length);
  if(result==std::string::npos)return false;*out=result;return true;
}
extern "C" BASE_EXPORT void rechrom_debug_string_erase(void* s,size_t start,size_t count) {static_cast<std::string*>(s)->erase(start,count);}
extern "C" BASE_EXPORT void rechrom_debug_string_insert(void* s,size_t start,const char* value) {static_cast<std::string*>(s)->insert(start,value);}
extern "C" BASE_EXPORT void rechrom_debug_string_substr_construct(const void* s,size_t start,size_t count,void* dst) {
  new(dst) std::string(static_cast<const std::string*>(s)->substr(start,count));
}
#include <sstream>
#include <iostream>
#include "build/build_config.h"
#if BUILDFLAG(IS_ANDROID)
#include <android/log.h>
#include "base/no_destructor.h"
#endif
extern "C" BASE_EXPORT void rechrom_debug_stringstream_construct(void* dst) {new(dst) std::stringstream();}
extern "C" BASE_EXPORT void* rechrom_debug_stringstream_ostream(void* p) {return static_cast<std::ostream*>(static_cast<std::stringstream*>(p));}
extern "C" BASE_EXPORT void rechrom_debug_stringstream_take_str(void* p,void* dst) {new(dst) std::string(std::move(*static_cast<std::stringstream*>(p)).str());}
extern "C" BASE_EXPORT void rechrom_debug_stringstream_destroy(void* p) {static_cast<std::stringstream*>(p)->~basic_stringstream();}
#if BUILDFLAG(IS_ANDROID)
// Source: base/debug/task_trace.cc:26-38. Actual std::stringbuf virtual ABI
// remains owned by the C++ standard library, not a guessed Rust vtable.
namespace {
class AndroidErrBuffer:public std::stringbuf{
 protected:
  int sync()override{__android_log_write(ANDROID_LOG_ERROR,"chromium",str().c_str());return 0;}
};
}
#endif
extern "C" BASE_EXPORT void* rechrom_debug_task_default_ostream(){
#if BUILDFLAG(IS_ANDROID)
 static base::NoDestructor<AndroidErrBuffer> buf;
 static base::NoDestructor<std::ostream> out(buf.get());
 return out.get();
#else
 return &std::cerr;
#endif
}
extern "C" BASE_EXPORT size_t rechrom_debug_stringstream_size(){return sizeof(std::stringstream);}
extern "C" BASE_EXPORT size_t rechrom_debug_stringstream_align(){return alignof(std::stringstream);}

// Additional same-record assertions: stringstream is a distinct SDK object.
static_assert(sizeof(std::stringstream)==RECHROM_NATIVE_STRINGSTREAM_SIZE,
              "supplied stringstream size differs from actual native SDK");
static_assert(alignof(std::stringstream)==RECHROM_NATIVE_STRINGSTREAM_ALIGN,
              "supplied stringstream alignment differs from actual native SDK");
extern "C" BASE_EXPORT void rechrom_std_string_default_construct(void* dst) {new(dst) std::string();}
extern "C" BASE_EXPORT void rechrom_std_string_append(void* dst,const uint8_t* data,size_t len) {
 static_cast<std::string*>(dst)->append(reinterpret_cast<const char*>(data),len);
}
extern "C" BASE_EXPORT void rechrom_std_string_clear(void* dst) {static_cast<std::string*>(dst)->clear();}
extern "C" BASE_EXPORT void rechrom_ostringstream_take_str(void* src,void* dst) {
 new(dst) std::string(std::move(*static_cast<std::ostringstream*>(src)).str());
}
#include <stdarg.h>
#include <errno.h>
#include "base/strings/string_util.h"
// Source: base/strings/span_printf.h:26-33
// This retains actual Windows vsnprintf_s/_vscprintf versus POSIX forwarding.
extern "C" BASE_EXPORT int rechrom_native_vspanprintf(uint8_t* dst,size_t len,
                                                       const uint8_t* format,va_list ap) {
 return base::vsnprintf(reinterpret_cast<char*>(dst),len,reinterpret_cast<const char*>(format),ap);
}
extern "C" BASE_EXPORT void rechrom_native_va_end_slot(void* slot) {
 va_end(*static_cast<va_list*>(slot));
}
extern "C" BASE_EXPORT int* rechrom_native_errno_address() {return &errno;}
#if BUILDFLAG(IS_WIN)
#include <windows.h>
extern "C" BASE_EXPORT unsigned long rechrom_native_get_last_error() {return GetLastError();}
extern "C" BASE_EXPORT void rechrom_native_set_last_error(unsigned long error) {SetLastError(error);}
#endif

// Only the compiler's variadic entry/va_start/va_end protocol is native here.
// All original severity/filter/formatting/log owner/abort algorithms remain Rust.
extern "C" bool rechrom_google_sklog_prepare(int priority,int* severity);
extern "C" bool rechrom_google_debug_prepare(int* severity);
extern "C" void rechrom_google_log_body(const uint8_t*,int,int,const uint8_t*,va_list,void*);
extern "C" int rechrom_google_abort_prepare();
extern "C" void rechrom_google_abort_body(const uint8_t*,int,int,const uint8_t*,va_list,void*);
// Source: skia/ext/google_logging.cc:13-49
extern "C" BASE_EXPORT void rechrom_google_sklog_fileline(const uint8_t* file,int line,
                                                          int priority,const uint8_t* format,...) {
 int severity;
 if(!rechrom_google_sklog_prepare(priority,&severity)) return;
 va_list ap;va_start(ap,format);
 rechrom_google_log_body(file,line,severity,format,ap,&ap);
 // Rust body called actual va_end on this same slot at the original source
 // position before LogMessage construction. Never va_end this cursor twice.
}
// Source: skia/ext/google_logging.cc:51-68
extern "C" BASE_EXPORT void rechrom_google_debug_fileline(const uint8_t* file,int line,
                                                          const uint8_t* format,...) {
 int severity;
 if(!rechrom_google_debug_prepare(&severity)) return;
 va_list ap;va_start(ap,format);
 rechrom_google_log_body(file,line,severity,format,ap,&ap);
}
// Source: skia/ext/google_logging.cc:70-84
extern "C" BASE_EXPORT void rechrom_google_abort_fileline(const uint8_t* file,int line,
                                                          const uint8_t* format,...) {
 int severity=rechrom_google_abort_prepare();
 va_list ap;va_start(ap,format);
 rechrom_google_abort_body(file,line,severity,format,ap,&ap);
}

// Exact native stdlib object operations; append to the existing interop TU.
// Source expression: base/logging.cc:798-799,1064-1087.
extern "C" void rechrom_logging_std_string_empty_at(void* dst) {
  new (dst) std::string("");
}
extern "C" void rechrom_logging_std_string_concat_at(void* dst,const void* a,const void* b) {
  new (dst) std::string(*static_cast<const std::string*>(a) +
                       *static_cast<const std::string*>(b));
}

// Actual standard-library return-slot binding to the original called function.
// Root must link the actual base/strings/strcat.cc producer and genuine native
// dependencies. This is not a Rust Vec stand-in, extra heap string owner or
// replacement StrCat algorithm. No native ABI number is inferred here.
#include <new>
#include <string>
#include "base/base_export.h"
#include "base/strings/strcat.h"
#include "rechrom_native_ostream_abi.h"
static_assert(sizeof(std::string)==RECHROM_NATIVE_STD_STRING_SIZE, "actual StrCat SDK std::string size differs from supplied metadata");
static_assert(alignof(std::string)==RECHROM_NATIVE_STD_STRING_ALIGN, "actual StrCat SDK std::string alignment differs from supplied metadata");
#if defined(__cpp_exceptions) || defined(_CPPUNWIND)
#error "Actual native no-exceptions domain required; exception transport not supplied"
#endif
// Source: base/check_op.cc:122-127
extern "C" BASE_EXPORT void rechrom_checkop_strcat_at(void* dst,const char* expr,
                                                     const char* v1,const char* v2) {
 new(dst) std::string(base::StrCat({expr," (",v1," vs. ",v2,")"}));
}

// Actual compile-time FormatSpec SDK instantiations for already selected
// logging call sites; no runtime-format approximation of the Absl template.
#include "base/strings/stringprintf.h"
// Source: base/logging.cc:756-756
extern "C" BASE_EXPORT void rechrom_absl_hex8_at(void* out,uint32_t value) {
 new(out) std::string(base::StringPrintf("0x%08X",value));
}
// Source: base/logging.cc:1077-1077
extern "C" BASE_EXPORT void rechrom_absl_system_hex_at(void* out,unsigned long value) {
 new(out) std::string(base::StringPrintf(" (0x%lX)",value));
}
// Source: base/logging.cc:1079-1080
extern "C" BASE_EXPORT void rechrom_absl_system_error_at(void* out,unsigned long last,unsigned long value) {
 new(out) std::string(base::StringPrintf("Error (0x%lX) while retrieving error. (0x%lX)",last,value));
}
// Source: base/logging.cc:1083-1083
extern "C" BASE_EXPORT void rechrom_absl_system_decimal_at(void* out,int value) {
 new(out) std::string(base::StringPrintf(" (%d)",value));
}
// Source: base/logging.cc:489-489
extern "C" BASE_EXPORT void rechrom_absl_crash_string_at(void* out,const uint8_t* file,
                                                        int line,const uint8_t* data,size_t len) {
 new(out) std::string(base::StringPrintf("%s:%d: %s",reinterpret_cast<const char*>(file),line,
                                       std::string_view(reinterpret_cast<const char*>(data),len)));
}

// APPEND to actual configured interop TU (steward owns file).
// Source: base/logging.cc:431-441. No local static here; the Rust native
// declaration's unique initialization guard owns publication and caching.
#if BUILDFLAG(IS_WIN)
#include <windows.h>
using RechromLogFatalCrashKeyFunc=void (*)(const char*,int,const char*);
extern "C" RechromLogFatalCrashKeyFunc rechrom_logging_resolve_bootstrap_fatal_key(){
 return reinterpret_cast<RechromLogFatalCrashKeyFunc>(
   GetProcAddress(GetModuleHandle(NULL),"SetLogFatalCrashKey"));
}
#endif
