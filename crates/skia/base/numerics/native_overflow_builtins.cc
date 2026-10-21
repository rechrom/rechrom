// Genuine compiler intrinsic SDK binding; no substitute arithmetic.
// Source: base/numerics/safe_math_clang_gcc_impl.h:72-111
#include <stdint.h>
#include "base/base_export.h"
#if !defined(__GNUC__) && !defined(__clang__)
#error "Actual GCC/Clang overflow builtin domain required"
#endif
#if defined(__asmjs__) || defined(__wasm__)
#error "Native shared header selects unavailable optimized paths in this domain"
#endif
#if INTPTR_MAX != INT32_MAX && INTPTR_MAX != INT64_MAX
#error "Native integer pointer width outside this explicit binding domain"
#endif
#define RECHROM_ADD_SUB(T,U,V,ADD,SUB) \
 extern "C" BASE_EXPORT bool ADD(T x,U y,V* result) {return !__builtin_add_overflow(x,y,result);} \
 extern "C" BASE_EXPORT bool SUB(T x,U y,V* result) {return !__builtin_sub_overflow(x,y,result);}
#define RECHROM_MUL(T,U,V,MUL) \
 extern "C" BASE_EXPORT bool MUL(T x,U y,V* result) {return !__builtin_mul_overflow(x,y,result);}
RECHROM_ADD_SUB(int8_t,int8_t,int8_t,rechrom_builtin_add_i8_i8_i8,rechrom_builtin_sub_i8_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,int8_t,rechrom_builtin_mul_i8_i8_i8)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,uint8_t,rechrom_builtin_add_i8_i8_u8,rechrom_builtin_sub_i8_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,uint8_t,rechrom_builtin_mul_i8_i8_u8)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,int16_t,rechrom_builtin_add_i8_i8_i16,rechrom_builtin_sub_i8_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,int16_t,rechrom_builtin_mul_i8_i8_i16)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,uint16_t,rechrom_builtin_add_i8_i8_u16,rechrom_builtin_sub_i8_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,uint16_t,rechrom_builtin_mul_i8_i8_u16)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,int32_t,rechrom_builtin_add_i8_i8_i32,rechrom_builtin_sub_i8_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,int32_t,rechrom_builtin_mul_i8_i8_i32)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,uint32_t,rechrom_builtin_add_i8_i8_u32,rechrom_builtin_sub_i8_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,uint32_t,rechrom_builtin_mul_i8_i8_u32)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,int64_t,rechrom_builtin_add_i8_i8_i64,rechrom_builtin_sub_i8_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,int64_t,rechrom_builtin_mul_i8_i8_i64)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,uint64_t,rechrom_builtin_add_i8_i8_u64,rechrom_builtin_sub_i8_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,uint64_t,rechrom_builtin_mul_i8_i8_u64)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,intptr_t,rechrom_builtin_add_i8_i8_isize,rechrom_builtin_sub_i8_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,intptr_t,rechrom_builtin_mul_i8_i8_isize)
#endif
RECHROM_ADD_SUB(int8_t,int8_t,uintptr_t,rechrom_builtin_add_i8_i8_usize,rechrom_builtin_sub_i8_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int8_t,uintptr_t,rechrom_builtin_mul_i8_i8_usize)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,int8_t,rechrom_builtin_add_i8_u8_i8,rechrom_builtin_sub_i8_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,int8_t,rechrom_builtin_mul_i8_u8_i8)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,uint8_t,rechrom_builtin_add_i8_u8_u8,rechrom_builtin_sub_i8_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,uint8_t,rechrom_builtin_mul_i8_u8_u8)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,int16_t,rechrom_builtin_add_i8_u8_i16,rechrom_builtin_sub_i8_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,int16_t,rechrom_builtin_mul_i8_u8_i16)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,uint16_t,rechrom_builtin_add_i8_u8_u16,rechrom_builtin_sub_i8_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,uint16_t,rechrom_builtin_mul_i8_u8_u16)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,int32_t,rechrom_builtin_add_i8_u8_i32,rechrom_builtin_sub_i8_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,int32_t,rechrom_builtin_mul_i8_u8_i32)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,uint32_t,rechrom_builtin_add_i8_u8_u32,rechrom_builtin_sub_i8_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,uint32_t,rechrom_builtin_mul_i8_u8_u32)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,int64_t,rechrom_builtin_add_i8_u8_i64,rechrom_builtin_sub_i8_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,int64_t,rechrom_builtin_mul_i8_u8_i64)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,uint64_t,rechrom_builtin_add_i8_u8_u64,rechrom_builtin_sub_i8_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,uint64_t,rechrom_builtin_mul_i8_u8_u64)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,intptr_t,rechrom_builtin_add_i8_u8_isize,rechrom_builtin_sub_i8_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,intptr_t,rechrom_builtin_mul_i8_u8_isize)
#endif
RECHROM_ADD_SUB(int8_t,uint8_t,uintptr_t,rechrom_builtin_add_i8_u8_usize,rechrom_builtin_sub_i8_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint8_t,uintptr_t,rechrom_builtin_mul_i8_u8_usize)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,int8_t,rechrom_builtin_add_i8_i16_i8,rechrom_builtin_sub_i8_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,int8_t,rechrom_builtin_mul_i8_i16_i8)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,uint8_t,rechrom_builtin_add_i8_i16_u8,rechrom_builtin_sub_i8_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,uint8_t,rechrom_builtin_mul_i8_i16_u8)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,int16_t,rechrom_builtin_add_i8_i16_i16,rechrom_builtin_sub_i8_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,int16_t,rechrom_builtin_mul_i8_i16_i16)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,uint16_t,rechrom_builtin_add_i8_i16_u16,rechrom_builtin_sub_i8_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,uint16_t,rechrom_builtin_mul_i8_i16_u16)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,int32_t,rechrom_builtin_add_i8_i16_i32,rechrom_builtin_sub_i8_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,int32_t,rechrom_builtin_mul_i8_i16_i32)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,uint32_t,rechrom_builtin_add_i8_i16_u32,rechrom_builtin_sub_i8_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,uint32_t,rechrom_builtin_mul_i8_i16_u32)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,int64_t,rechrom_builtin_add_i8_i16_i64,rechrom_builtin_sub_i8_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,int64_t,rechrom_builtin_mul_i8_i16_i64)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,uint64_t,rechrom_builtin_add_i8_i16_u64,rechrom_builtin_sub_i8_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,uint64_t,rechrom_builtin_mul_i8_i16_u64)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,intptr_t,rechrom_builtin_add_i8_i16_isize,rechrom_builtin_sub_i8_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,intptr_t,rechrom_builtin_mul_i8_i16_isize)
#endif
RECHROM_ADD_SUB(int8_t,int16_t,uintptr_t,rechrom_builtin_add_i8_i16_usize,rechrom_builtin_sub_i8_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int16_t,uintptr_t,rechrom_builtin_mul_i8_i16_usize)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,int8_t,rechrom_builtin_add_i8_u16_i8,rechrom_builtin_sub_i8_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,int8_t,rechrom_builtin_mul_i8_u16_i8)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,uint8_t,rechrom_builtin_add_i8_u16_u8,rechrom_builtin_sub_i8_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,uint8_t,rechrom_builtin_mul_i8_u16_u8)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,int16_t,rechrom_builtin_add_i8_u16_i16,rechrom_builtin_sub_i8_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,int16_t,rechrom_builtin_mul_i8_u16_i16)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,uint16_t,rechrom_builtin_add_i8_u16_u16,rechrom_builtin_sub_i8_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,uint16_t,rechrom_builtin_mul_i8_u16_u16)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,int32_t,rechrom_builtin_add_i8_u16_i32,rechrom_builtin_sub_i8_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,int32_t,rechrom_builtin_mul_i8_u16_i32)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,uint32_t,rechrom_builtin_add_i8_u16_u32,rechrom_builtin_sub_i8_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,uint32_t,rechrom_builtin_mul_i8_u16_u32)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,int64_t,rechrom_builtin_add_i8_u16_i64,rechrom_builtin_sub_i8_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,int64_t,rechrom_builtin_mul_i8_u16_i64)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,uint64_t,rechrom_builtin_add_i8_u16_u64,rechrom_builtin_sub_i8_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,uint64_t,rechrom_builtin_mul_i8_u16_u64)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,intptr_t,rechrom_builtin_add_i8_u16_isize,rechrom_builtin_sub_i8_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,intptr_t,rechrom_builtin_mul_i8_u16_isize)
#endif
RECHROM_ADD_SUB(int8_t,uint16_t,uintptr_t,rechrom_builtin_add_i8_u16_usize,rechrom_builtin_sub_i8_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint16_t,uintptr_t,rechrom_builtin_mul_i8_u16_usize)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,int8_t,rechrom_builtin_add_i8_i32_i8,rechrom_builtin_sub_i8_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,int8_t,rechrom_builtin_mul_i8_i32_i8)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,uint8_t,rechrom_builtin_add_i8_i32_u8,rechrom_builtin_sub_i8_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,uint8_t,rechrom_builtin_mul_i8_i32_u8)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,int16_t,rechrom_builtin_add_i8_i32_i16,rechrom_builtin_sub_i8_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,int16_t,rechrom_builtin_mul_i8_i32_i16)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,uint16_t,rechrom_builtin_add_i8_i32_u16,rechrom_builtin_sub_i8_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,uint16_t,rechrom_builtin_mul_i8_i32_u16)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,int32_t,rechrom_builtin_add_i8_i32_i32,rechrom_builtin_sub_i8_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,int32_t,rechrom_builtin_mul_i8_i32_i32)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,uint32_t,rechrom_builtin_add_i8_i32_u32,rechrom_builtin_sub_i8_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,uint32_t,rechrom_builtin_mul_i8_i32_u32)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,int64_t,rechrom_builtin_add_i8_i32_i64,rechrom_builtin_sub_i8_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,int64_t,rechrom_builtin_mul_i8_i32_i64)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,uint64_t,rechrom_builtin_add_i8_i32_u64,rechrom_builtin_sub_i8_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,uint64_t,rechrom_builtin_mul_i8_i32_u64)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,intptr_t,rechrom_builtin_add_i8_i32_isize,rechrom_builtin_sub_i8_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,intptr_t,rechrom_builtin_mul_i8_i32_isize)
#endif
RECHROM_ADD_SUB(int8_t,int32_t,uintptr_t,rechrom_builtin_add_i8_i32_usize,rechrom_builtin_sub_i8_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int32_t,uintptr_t,rechrom_builtin_mul_i8_i32_usize)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,int8_t,rechrom_builtin_add_i8_u32_i8,rechrom_builtin_sub_i8_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,int8_t,rechrom_builtin_mul_i8_u32_i8)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,uint8_t,rechrom_builtin_add_i8_u32_u8,rechrom_builtin_sub_i8_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,uint8_t,rechrom_builtin_mul_i8_u32_u8)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,int16_t,rechrom_builtin_add_i8_u32_i16,rechrom_builtin_sub_i8_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,int16_t,rechrom_builtin_mul_i8_u32_i16)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,uint16_t,rechrom_builtin_add_i8_u32_u16,rechrom_builtin_sub_i8_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,uint16_t,rechrom_builtin_mul_i8_u32_u16)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,int32_t,rechrom_builtin_add_i8_u32_i32,rechrom_builtin_sub_i8_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,int32_t,rechrom_builtin_mul_i8_u32_i32)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,uint32_t,rechrom_builtin_add_i8_u32_u32,rechrom_builtin_sub_i8_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,uint32_t,rechrom_builtin_mul_i8_u32_u32)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,int64_t,rechrom_builtin_add_i8_u32_i64,rechrom_builtin_sub_i8_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,int64_t,rechrom_builtin_mul_i8_u32_i64)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,uint64_t,rechrom_builtin_add_i8_u32_u64,rechrom_builtin_sub_i8_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,uint64_t,rechrom_builtin_mul_i8_u32_u64)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,intptr_t,rechrom_builtin_add_i8_u32_isize,rechrom_builtin_sub_i8_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,intptr_t,rechrom_builtin_mul_i8_u32_isize)
#endif
RECHROM_ADD_SUB(int8_t,uint32_t,uintptr_t,rechrom_builtin_add_i8_u32_usize,rechrom_builtin_sub_i8_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,uint32_t,uintptr_t,rechrom_builtin_mul_i8_u32_usize)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,int8_t,rechrom_builtin_add_i8_i64_i8,rechrom_builtin_sub_i8_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,int8_t,rechrom_builtin_mul_i8_i64_i8)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,uint8_t,rechrom_builtin_add_i8_i64_u8,rechrom_builtin_sub_i8_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,uint8_t,rechrom_builtin_mul_i8_i64_u8)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,int16_t,rechrom_builtin_add_i8_i64_i16,rechrom_builtin_sub_i8_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,int16_t,rechrom_builtin_mul_i8_i64_i16)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,uint16_t,rechrom_builtin_add_i8_i64_u16,rechrom_builtin_sub_i8_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,uint16_t,rechrom_builtin_mul_i8_i64_u16)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,int32_t,rechrom_builtin_add_i8_i64_i32,rechrom_builtin_sub_i8_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,int32_t,rechrom_builtin_mul_i8_i64_i32)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,uint32_t,rechrom_builtin_add_i8_i64_u32,rechrom_builtin_sub_i8_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,uint32_t,rechrom_builtin_mul_i8_i64_u32)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,int64_t,rechrom_builtin_add_i8_i64_i64,rechrom_builtin_sub_i8_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,int64_t,rechrom_builtin_mul_i8_i64_i64)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,uint64_t,rechrom_builtin_add_i8_i64_u64,rechrom_builtin_sub_i8_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,uint64_t,rechrom_builtin_mul_i8_i64_u64)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,intptr_t,rechrom_builtin_add_i8_i64_isize,rechrom_builtin_sub_i8_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,intptr_t,rechrom_builtin_mul_i8_i64_isize)
#endif
RECHROM_ADD_SUB(int8_t,int64_t,uintptr_t,rechrom_builtin_add_i8_i64_usize,rechrom_builtin_sub_i8_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,int64_t,uintptr_t,rechrom_builtin_mul_i8_i64_usize)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,int8_t,rechrom_builtin_add_i8_u64_i8,rechrom_builtin_sub_i8_u64_i8)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,int8_t,rechrom_builtin_mul_i8_u64_i8)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,uint8_t,rechrom_builtin_add_i8_u64_u8,rechrom_builtin_sub_i8_u64_u8)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,uint8_t,rechrom_builtin_mul_i8_u64_u8)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,int16_t,rechrom_builtin_add_i8_u64_i16,rechrom_builtin_sub_i8_u64_i16)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,int16_t,rechrom_builtin_mul_i8_u64_i16)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,uint16_t,rechrom_builtin_add_i8_u64_u16,rechrom_builtin_sub_i8_u64_u16)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,uint16_t,rechrom_builtin_mul_i8_u64_u16)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,int32_t,rechrom_builtin_add_i8_u64_i32,rechrom_builtin_sub_i8_u64_i32)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,int32_t,rechrom_builtin_mul_i8_u64_i32)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,uint32_t,rechrom_builtin_add_i8_u64_u32,rechrom_builtin_sub_i8_u64_u32)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,uint32_t,rechrom_builtin_mul_i8_u64_u32)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,int64_t,rechrom_builtin_add_i8_u64_i64,rechrom_builtin_sub_i8_u64_i64)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,int64_t,rechrom_builtin_mul_i8_u64_i64)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,uint64_t,rechrom_builtin_add_i8_u64_u64,rechrom_builtin_sub_i8_u64_u64)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,uint64_t,rechrom_builtin_mul_i8_u64_u64)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,intptr_t,rechrom_builtin_add_i8_u64_isize,rechrom_builtin_sub_i8_u64_isize)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,intptr_t,rechrom_builtin_mul_i8_u64_isize)
#endif
RECHROM_ADD_SUB(int8_t,uint64_t,uintptr_t,rechrom_builtin_add_i8_u64_usize,rechrom_builtin_sub_i8_u64_usize)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uint64_t,uintptr_t,rechrom_builtin_mul_i8_u64_usize)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,int8_t,rechrom_builtin_add_i8_isize_i8,rechrom_builtin_sub_i8_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,int8_t,rechrom_builtin_mul_i8_isize_i8)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,uint8_t,rechrom_builtin_add_i8_isize_u8,rechrom_builtin_sub_i8_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,uint8_t,rechrom_builtin_mul_i8_isize_u8)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,int16_t,rechrom_builtin_add_i8_isize_i16,rechrom_builtin_sub_i8_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,int16_t,rechrom_builtin_mul_i8_isize_i16)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,uint16_t,rechrom_builtin_add_i8_isize_u16,rechrom_builtin_sub_i8_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,uint16_t,rechrom_builtin_mul_i8_isize_u16)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,int32_t,rechrom_builtin_add_i8_isize_i32,rechrom_builtin_sub_i8_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,int32_t,rechrom_builtin_mul_i8_isize_i32)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,uint32_t,rechrom_builtin_add_i8_isize_u32,rechrom_builtin_sub_i8_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,uint32_t,rechrom_builtin_mul_i8_isize_u32)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,int64_t,rechrom_builtin_add_i8_isize_i64,rechrom_builtin_sub_i8_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,int64_t,rechrom_builtin_mul_i8_isize_i64)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,uint64_t,rechrom_builtin_add_i8_isize_u64,rechrom_builtin_sub_i8_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,uint64_t,rechrom_builtin_mul_i8_isize_u64)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,intptr_t,rechrom_builtin_add_i8_isize_isize,rechrom_builtin_sub_i8_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,intptr_t,rechrom_builtin_mul_i8_isize_isize)
#endif
RECHROM_ADD_SUB(int8_t,intptr_t,uintptr_t,rechrom_builtin_add_i8_isize_usize,rechrom_builtin_sub_i8_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int8_t,intptr_t,uintptr_t,rechrom_builtin_mul_i8_isize_usize)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,int8_t,rechrom_builtin_add_i8_usize_i8,rechrom_builtin_sub_i8_usize_i8)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,int8_t,rechrom_builtin_mul_i8_usize_i8)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,uint8_t,rechrom_builtin_add_i8_usize_u8,rechrom_builtin_sub_i8_usize_u8)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,uint8_t,rechrom_builtin_mul_i8_usize_u8)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,int16_t,rechrom_builtin_add_i8_usize_i16,rechrom_builtin_sub_i8_usize_i16)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,int16_t,rechrom_builtin_mul_i8_usize_i16)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,uint16_t,rechrom_builtin_add_i8_usize_u16,rechrom_builtin_sub_i8_usize_u16)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,uint16_t,rechrom_builtin_mul_i8_usize_u16)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,int32_t,rechrom_builtin_add_i8_usize_i32,rechrom_builtin_sub_i8_usize_i32)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,int32_t,rechrom_builtin_mul_i8_usize_i32)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,uint32_t,rechrom_builtin_add_i8_usize_u32,rechrom_builtin_sub_i8_usize_u32)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,uint32_t,rechrom_builtin_mul_i8_usize_u32)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,int64_t,rechrom_builtin_add_i8_usize_i64,rechrom_builtin_sub_i8_usize_i64)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,int64_t,rechrom_builtin_mul_i8_usize_i64)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,uint64_t,rechrom_builtin_add_i8_usize_u64,rechrom_builtin_sub_i8_usize_u64)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,uint64_t,rechrom_builtin_mul_i8_usize_u64)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,intptr_t,rechrom_builtin_add_i8_usize_isize,rechrom_builtin_sub_i8_usize_isize)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,intptr_t,rechrom_builtin_mul_i8_usize_isize)
#endif
RECHROM_ADD_SUB(int8_t,uintptr_t,uintptr_t,rechrom_builtin_add_i8_usize_usize,rechrom_builtin_sub_i8_usize_usize)
#if !defined(__clang__)
RECHROM_MUL(int8_t,uintptr_t,uintptr_t,rechrom_builtin_mul_i8_usize_usize)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,int8_t,rechrom_builtin_add_u8_i8_i8,rechrom_builtin_sub_u8_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,int8_t,rechrom_builtin_mul_u8_i8_i8)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,uint8_t,rechrom_builtin_add_u8_i8_u8,rechrom_builtin_sub_u8_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,uint8_t,rechrom_builtin_mul_u8_i8_u8)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,int16_t,rechrom_builtin_add_u8_i8_i16,rechrom_builtin_sub_u8_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,int16_t,rechrom_builtin_mul_u8_i8_i16)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,uint16_t,rechrom_builtin_add_u8_i8_u16,rechrom_builtin_sub_u8_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,uint16_t,rechrom_builtin_mul_u8_i8_u16)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,int32_t,rechrom_builtin_add_u8_i8_i32,rechrom_builtin_sub_u8_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,int32_t,rechrom_builtin_mul_u8_i8_i32)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,uint32_t,rechrom_builtin_add_u8_i8_u32,rechrom_builtin_sub_u8_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,uint32_t,rechrom_builtin_mul_u8_i8_u32)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,int64_t,rechrom_builtin_add_u8_i8_i64,rechrom_builtin_sub_u8_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,int64_t,rechrom_builtin_mul_u8_i8_i64)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,uint64_t,rechrom_builtin_add_u8_i8_u64,rechrom_builtin_sub_u8_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,uint64_t,rechrom_builtin_mul_u8_i8_u64)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,intptr_t,rechrom_builtin_add_u8_i8_isize,rechrom_builtin_sub_u8_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,intptr_t,rechrom_builtin_mul_u8_i8_isize)
#endif
RECHROM_ADD_SUB(uint8_t,int8_t,uintptr_t,rechrom_builtin_add_u8_i8_usize,rechrom_builtin_sub_u8_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int8_t,uintptr_t,rechrom_builtin_mul_u8_i8_usize)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,int8_t,rechrom_builtin_add_u8_u8_i8,rechrom_builtin_sub_u8_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,int8_t,rechrom_builtin_mul_u8_u8_i8)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,uint8_t,rechrom_builtin_add_u8_u8_u8,rechrom_builtin_sub_u8_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,uint8_t,rechrom_builtin_mul_u8_u8_u8)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,int16_t,rechrom_builtin_add_u8_u8_i16,rechrom_builtin_sub_u8_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,int16_t,rechrom_builtin_mul_u8_u8_i16)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,uint16_t,rechrom_builtin_add_u8_u8_u16,rechrom_builtin_sub_u8_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,uint16_t,rechrom_builtin_mul_u8_u8_u16)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,int32_t,rechrom_builtin_add_u8_u8_i32,rechrom_builtin_sub_u8_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,int32_t,rechrom_builtin_mul_u8_u8_i32)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,uint32_t,rechrom_builtin_add_u8_u8_u32,rechrom_builtin_sub_u8_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,uint32_t,rechrom_builtin_mul_u8_u8_u32)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,int64_t,rechrom_builtin_add_u8_u8_i64,rechrom_builtin_sub_u8_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,int64_t,rechrom_builtin_mul_u8_u8_i64)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,uint64_t,rechrom_builtin_add_u8_u8_u64,rechrom_builtin_sub_u8_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,uint64_t,rechrom_builtin_mul_u8_u8_u64)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,intptr_t,rechrom_builtin_add_u8_u8_isize,rechrom_builtin_sub_u8_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,intptr_t,rechrom_builtin_mul_u8_u8_isize)
#endif
RECHROM_ADD_SUB(uint8_t,uint8_t,uintptr_t,rechrom_builtin_add_u8_u8_usize,rechrom_builtin_sub_u8_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint8_t,uintptr_t,rechrom_builtin_mul_u8_u8_usize)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,int8_t,rechrom_builtin_add_u8_i16_i8,rechrom_builtin_sub_u8_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,int8_t,rechrom_builtin_mul_u8_i16_i8)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,uint8_t,rechrom_builtin_add_u8_i16_u8,rechrom_builtin_sub_u8_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,uint8_t,rechrom_builtin_mul_u8_i16_u8)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,int16_t,rechrom_builtin_add_u8_i16_i16,rechrom_builtin_sub_u8_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,int16_t,rechrom_builtin_mul_u8_i16_i16)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,uint16_t,rechrom_builtin_add_u8_i16_u16,rechrom_builtin_sub_u8_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,uint16_t,rechrom_builtin_mul_u8_i16_u16)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,int32_t,rechrom_builtin_add_u8_i16_i32,rechrom_builtin_sub_u8_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,int32_t,rechrom_builtin_mul_u8_i16_i32)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,uint32_t,rechrom_builtin_add_u8_i16_u32,rechrom_builtin_sub_u8_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,uint32_t,rechrom_builtin_mul_u8_i16_u32)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,int64_t,rechrom_builtin_add_u8_i16_i64,rechrom_builtin_sub_u8_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,int64_t,rechrom_builtin_mul_u8_i16_i64)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,uint64_t,rechrom_builtin_add_u8_i16_u64,rechrom_builtin_sub_u8_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,uint64_t,rechrom_builtin_mul_u8_i16_u64)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,intptr_t,rechrom_builtin_add_u8_i16_isize,rechrom_builtin_sub_u8_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,intptr_t,rechrom_builtin_mul_u8_i16_isize)
#endif
RECHROM_ADD_SUB(uint8_t,int16_t,uintptr_t,rechrom_builtin_add_u8_i16_usize,rechrom_builtin_sub_u8_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int16_t,uintptr_t,rechrom_builtin_mul_u8_i16_usize)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,int8_t,rechrom_builtin_add_u8_u16_i8,rechrom_builtin_sub_u8_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,int8_t,rechrom_builtin_mul_u8_u16_i8)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,uint8_t,rechrom_builtin_add_u8_u16_u8,rechrom_builtin_sub_u8_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,uint8_t,rechrom_builtin_mul_u8_u16_u8)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,int16_t,rechrom_builtin_add_u8_u16_i16,rechrom_builtin_sub_u8_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,int16_t,rechrom_builtin_mul_u8_u16_i16)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,uint16_t,rechrom_builtin_add_u8_u16_u16,rechrom_builtin_sub_u8_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,uint16_t,rechrom_builtin_mul_u8_u16_u16)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,int32_t,rechrom_builtin_add_u8_u16_i32,rechrom_builtin_sub_u8_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,int32_t,rechrom_builtin_mul_u8_u16_i32)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,uint32_t,rechrom_builtin_add_u8_u16_u32,rechrom_builtin_sub_u8_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,uint32_t,rechrom_builtin_mul_u8_u16_u32)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,int64_t,rechrom_builtin_add_u8_u16_i64,rechrom_builtin_sub_u8_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,int64_t,rechrom_builtin_mul_u8_u16_i64)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,uint64_t,rechrom_builtin_add_u8_u16_u64,rechrom_builtin_sub_u8_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,uint64_t,rechrom_builtin_mul_u8_u16_u64)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,intptr_t,rechrom_builtin_add_u8_u16_isize,rechrom_builtin_sub_u8_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,intptr_t,rechrom_builtin_mul_u8_u16_isize)
#endif
RECHROM_ADD_SUB(uint8_t,uint16_t,uintptr_t,rechrom_builtin_add_u8_u16_usize,rechrom_builtin_sub_u8_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint16_t,uintptr_t,rechrom_builtin_mul_u8_u16_usize)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,int8_t,rechrom_builtin_add_u8_i32_i8,rechrom_builtin_sub_u8_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,int8_t,rechrom_builtin_mul_u8_i32_i8)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,uint8_t,rechrom_builtin_add_u8_i32_u8,rechrom_builtin_sub_u8_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,uint8_t,rechrom_builtin_mul_u8_i32_u8)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,int16_t,rechrom_builtin_add_u8_i32_i16,rechrom_builtin_sub_u8_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,int16_t,rechrom_builtin_mul_u8_i32_i16)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,uint16_t,rechrom_builtin_add_u8_i32_u16,rechrom_builtin_sub_u8_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,uint16_t,rechrom_builtin_mul_u8_i32_u16)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,int32_t,rechrom_builtin_add_u8_i32_i32,rechrom_builtin_sub_u8_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,int32_t,rechrom_builtin_mul_u8_i32_i32)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,uint32_t,rechrom_builtin_add_u8_i32_u32,rechrom_builtin_sub_u8_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,uint32_t,rechrom_builtin_mul_u8_i32_u32)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,int64_t,rechrom_builtin_add_u8_i32_i64,rechrom_builtin_sub_u8_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,int64_t,rechrom_builtin_mul_u8_i32_i64)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,uint64_t,rechrom_builtin_add_u8_i32_u64,rechrom_builtin_sub_u8_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,uint64_t,rechrom_builtin_mul_u8_i32_u64)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,intptr_t,rechrom_builtin_add_u8_i32_isize,rechrom_builtin_sub_u8_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,intptr_t,rechrom_builtin_mul_u8_i32_isize)
#endif
RECHROM_ADD_SUB(uint8_t,int32_t,uintptr_t,rechrom_builtin_add_u8_i32_usize,rechrom_builtin_sub_u8_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int32_t,uintptr_t,rechrom_builtin_mul_u8_i32_usize)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,int8_t,rechrom_builtin_add_u8_u32_i8,rechrom_builtin_sub_u8_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,int8_t,rechrom_builtin_mul_u8_u32_i8)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,uint8_t,rechrom_builtin_add_u8_u32_u8,rechrom_builtin_sub_u8_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,uint8_t,rechrom_builtin_mul_u8_u32_u8)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,int16_t,rechrom_builtin_add_u8_u32_i16,rechrom_builtin_sub_u8_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,int16_t,rechrom_builtin_mul_u8_u32_i16)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,uint16_t,rechrom_builtin_add_u8_u32_u16,rechrom_builtin_sub_u8_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,uint16_t,rechrom_builtin_mul_u8_u32_u16)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,int32_t,rechrom_builtin_add_u8_u32_i32,rechrom_builtin_sub_u8_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,int32_t,rechrom_builtin_mul_u8_u32_i32)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,uint32_t,rechrom_builtin_add_u8_u32_u32,rechrom_builtin_sub_u8_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,uint32_t,rechrom_builtin_mul_u8_u32_u32)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,int64_t,rechrom_builtin_add_u8_u32_i64,rechrom_builtin_sub_u8_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,int64_t,rechrom_builtin_mul_u8_u32_i64)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,uint64_t,rechrom_builtin_add_u8_u32_u64,rechrom_builtin_sub_u8_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,uint64_t,rechrom_builtin_mul_u8_u32_u64)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,intptr_t,rechrom_builtin_add_u8_u32_isize,rechrom_builtin_sub_u8_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,intptr_t,rechrom_builtin_mul_u8_u32_isize)
#endif
RECHROM_ADD_SUB(uint8_t,uint32_t,uintptr_t,rechrom_builtin_add_u8_u32_usize,rechrom_builtin_sub_u8_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint32_t,uintptr_t,rechrom_builtin_mul_u8_u32_usize)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,int8_t,rechrom_builtin_add_u8_i64_i8,rechrom_builtin_sub_u8_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,int8_t,rechrom_builtin_mul_u8_i64_i8)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,uint8_t,rechrom_builtin_add_u8_i64_u8,rechrom_builtin_sub_u8_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,uint8_t,rechrom_builtin_mul_u8_i64_u8)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,int16_t,rechrom_builtin_add_u8_i64_i16,rechrom_builtin_sub_u8_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,int16_t,rechrom_builtin_mul_u8_i64_i16)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,uint16_t,rechrom_builtin_add_u8_i64_u16,rechrom_builtin_sub_u8_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,uint16_t,rechrom_builtin_mul_u8_i64_u16)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,int32_t,rechrom_builtin_add_u8_i64_i32,rechrom_builtin_sub_u8_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,int32_t,rechrom_builtin_mul_u8_i64_i32)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,uint32_t,rechrom_builtin_add_u8_i64_u32,rechrom_builtin_sub_u8_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,uint32_t,rechrom_builtin_mul_u8_i64_u32)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,int64_t,rechrom_builtin_add_u8_i64_i64,rechrom_builtin_sub_u8_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,int64_t,rechrom_builtin_mul_u8_i64_i64)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,uint64_t,rechrom_builtin_add_u8_i64_u64,rechrom_builtin_sub_u8_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,uint64_t,rechrom_builtin_mul_u8_i64_u64)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,intptr_t,rechrom_builtin_add_u8_i64_isize,rechrom_builtin_sub_u8_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,intptr_t,rechrom_builtin_mul_u8_i64_isize)
#endif
RECHROM_ADD_SUB(uint8_t,int64_t,uintptr_t,rechrom_builtin_add_u8_i64_usize,rechrom_builtin_sub_u8_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,int64_t,uintptr_t,rechrom_builtin_mul_u8_i64_usize)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,int8_t,rechrom_builtin_add_u8_u64_i8,rechrom_builtin_sub_u8_u64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,int8_t,rechrom_builtin_mul_u8_u64_i8)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,uint8_t,rechrom_builtin_add_u8_u64_u8,rechrom_builtin_sub_u8_u64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,uint8_t,rechrom_builtin_mul_u8_u64_u8)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,int16_t,rechrom_builtin_add_u8_u64_i16,rechrom_builtin_sub_u8_u64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,int16_t,rechrom_builtin_mul_u8_u64_i16)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,uint16_t,rechrom_builtin_add_u8_u64_u16,rechrom_builtin_sub_u8_u64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,uint16_t,rechrom_builtin_mul_u8_u64_u16)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,int32_t,rechrom_builtin_add_u8_u64_i32,rechrom_builtin_sub_u8_u64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,int32_t,rechrom_builtin_mul_u8_u64_i32)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,uint32_t,rechrom_builtin_add_u8_u64_u32,rechrom_builtin_sub_u8_u64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,uint32_t,rechrom_builtin_mul_u8_u64_u32)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,int64_t,rechrom_builtin_add_u8_u64_i64,rechrom_builtin_sub_u8_u64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,int64_t,rechrom_builtin_mul_u8_u64_i64)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,uint64_t,rechrom_builtin_add_u8_u64_u64,rechrom_builtin_sub_u8_u64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,uint64_t,rechrom_builtin_mul_u8_u64_u64)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,intptr_t,rechrom_builtin_add_u8_u64_isize,rechrom_builtin_sub_u8_u64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,intptr_t,rechrom_builtin_mul_u8_u64_isize)
#endif
RECHROM_ADD_SUB(uint8_t,uint64_t,uintptr_t,rechrom_builtin_add_u8_u64_usize,rechrom_builtin_sub_u8_u64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uint64_t,uintptr_t,rechrom_builtin_mul_u8_u64_usize)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,int8_t,rechrom_builtin_add_u8_isize_i8,rechrom_builtin_sub_u8_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,int8_t,rechrom_builtin_mul_u8_isize_i8)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,uint8_t,rechrom_builtin_add_u8_isize_u8,rechrom_builtin_sub_u8_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,uint8_t,rechrom_builtin_mul_u8_isize_u8)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,int16_t,rechrom_builtin_add_u8_isize_i16,rechrom_builtin_sub_u8_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,int16_t,rechrom_builtin_mul_u8_isize_i16)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,uint16_t,rechrom_builtin_add_u8_isize_u16,rechrom_builtin_sub_u8_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,uint16_t,rechrom_builtin_mul_u8_isize_u16)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,int32_t,rechrom_builtin_add_u8_isize_i32,rechrom_builtin_sub_u8_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,int32_t,rechrom_builtin_mul_u8_isize_i32)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,uint32_t,rechrom_builtin_add_u8_isize_u32,rechrom_builtin_sub_u8_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,uint32_t,rechrom_builtin_mul_u8_isize_u32)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,int64_t,rechrom_builtin_add_u8_isize_i64,rechrom_builtin_sub_u8_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,int64_t,rechrom_builtin_mul_u8_isize_i64)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,uint64_t,rechrom_builtin_add_u8_isize_u64,rechrom_builtin_sub_u8_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,uint64_t,rechrom_builtin_mul_u8_isize_u64)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,intptr_t,rechrom_builtin_add_u8_isize_isize,rechrom_builtin_sub_u8_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,intptr_t,rechrom_builtin_mul_u8_isize_isize)
#endif
RECHROM_ADD_SUB(uint8_t,intptr_t,uintptr_t,rechrom_builtin_add_u8_isize_usize,rechrom_builtin_sub_u8_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,intptr_t,uintptr_t,rechrom_builtin_mul_u8_isize_usize)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,int8_t,rechrom_builtin_add_u8_usize_i8,rechrom_builtin_sub_u8_usize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,int8_t,rechrom_builtin_mul_u8_usize_i8)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,uint8_t,rechrom_builtin_add_u8_usize_u8,rechrom_builtin_sub_u8_usize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,uint8_t,rechrom_builtin_mul_u8_usize_u8)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,int16_t,rechrom_builtin_add_u8_usize_i16,rechrom_builtin_sub_u8_usize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,int16_t,rechrom_builtin_mul_u8_usize_i16)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,uint16_t,rechrom_builtin_add_u8_usize_u16,rechrom_builtin_sub_u8_usize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,uint16_t,rechrom_builtin_mul_u8_usize_u16)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,int32_t,rechrom_builtin_add_u8_usize_i32,rechrom_builtin_sub_u8_usize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,int32_t,rechrom_builtin_mul_u8_usize_i32)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,uint32_t,rechrom_builtin_add_u8_usize_u32,rechrom_builtin_sub_u8_usize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,uint32_t,rechrom_builtin_mul_u8_usize_u32)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,int64_t,rechrom_builtin_add_u8_usize_i64,rechrom_builtin_sub_u8_usize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,int64_t,rechrom_builtin_mul_u8_usize_i64)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,uint64_t,rechrom_builtin_add_u8_usize_u64,rechrom_builtin_sub_u8_usize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,uint64_t,rechrom_builtin_mul_u8_usize_u64)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,intptr_t,rechrom_builtin_add_u8_usize_isize,rechrom_builtin_sub_u8_usize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,intptr_t,rechrom_builtin_mul_u8_usize_isize)
#endif
RECHROM_ADD_SUB(uint8_t,uintptr_t,uintptr_t,rechrom_builtin_add_u8_usize_usize,rechrom_builtin_sub_u8_usize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint8_t,uintptr_t,uintptr_t,rechrom_builtin_mul_u8_usize_usize)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,int8_t,rechrom_builtin_add_i16_i8_i8,rechrom_builtin_sub_i16_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,int8_t,rechrom_builtin_mul_i16_i8_i8)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,uint8_t,rechrom_builtin_add_i16_i8_u8,rechrom_builtin_sub_i16_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,uint8_t,rechrom_builtin_mul_i16_i8_u8)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,int16_t,rechrom_builtin_add_i16_i8_i16,rechrom_builtin_sub_i16_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,int16_t,rechrom_builtin_mul_i16_i8_i16)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,uint16_t,rechrom_builtin_add_i16_i8_u16,rechrom_builtin_sub_i16_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,uint16_t,rechrom_builtin_mul_i16_i8_u16)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,int32_t,rechrom_builtin_add_i16_i8_i32,rechrom_builtin_sub_i16_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,int32_t,rechrom_builtin_mul_i16_i8_i32)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,uint32_t,rechrom_builtin_add_i16_i8_u32,rechrom_builtin_sub_i16_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,uint32_t,rechrom_builtin_mul_i16_i8_u32)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,int64_t,rechrom_builtin_add_i16_i8_i64,rechrom_builtin_sub_i16_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,int64_t,rechrom_builtin_mul_i16_i8_i64)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,uint64_t,rechrom_builtin_add_i16_i8_u64,rechrom_builtin_sub_i16_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,uint64_t,rechrom_builtin_mul_i16_i8_u64)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,intptr_t,rechrom_builtin_add_i16_i8_isize,rechrom_builtin_sub_i16_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,intptr_t,rechrom_builtin_mul_i16_i8_isize)
#endif
RECHROM_ADD_SUB(int16_t,int8_t,uintptr_t,rechrom_builtin_add_i16_i8_usize,rechrom_builtin_sub_i16_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int8_t,uintptr_t,rechrom_builtin_mul_i16_i8_usize)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,int8_t,rechrom_builtin_add_i16_u8_i8,rechrom_builtin_sub_i16_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,int8_t,rechrom_builtin_mul_i16_u8_i8)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,uint8_t,rechrom_builtin_add_i16_u8_u8,rechrom_builtin_sub_i16_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,uint8_t,rechrom_builtin_mul_i16_u8_u8)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,int16_t,rechrom_builtin_add_i16_u8_i16,rechrom_builtin_sub_i16_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,int16_t,rechrom_builtin_mul_i16_u8_i16)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,uint16_t,rechrom_builtin_add_i16_u8_u16,rechrom_builtin_sub_i16_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,uint16_t,rechrom_builtin_mul_i16_u8_u16)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,int32_t,rechrom_builtin_add_i16_u8_i32,rechrom_builtin_sub_i16_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,int32_t,rechrom_builtin_mul_i16_u8_i32)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,uint32_t,rechrom_builtin_add_i16_u8_u32,rechrom_builtin_sub_i16_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,uint32_t,rechrom_builtin_mul_i16_u8_u32)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,int64_t,rechrom_builtin_add_i16_u8_i64,rechrom_builtin_sub_i16_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,int64_t,rechrom_builtin_mul_i16_u8_i64)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,uint64_t,rechrom_builtin_add_i16_u8_u64,rechrom_builtin_sub_i16_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,uint64_t,rechrom_builtin_mul_i16_u8_u64)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,intptr_t,rechrom_builtin_add_i16_u8_isize,rechrom_builtin_sub_i16_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,intptr_t,rechrom_builtin_mul_i16_u8_isize)
#endif
RECHROM_ADD_SUB(int16_t,uint8_t,uintptr_t,rechrom_builtin_add_i16_u8_usize,rechrom_builtin_sub_i16_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint8_t,uintptr_t,rechrom_builtin_mul_i16_u8_usize)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,int8_t,rechrom_builtin_add_i16_i16_i8,rechrom_builtin_sub_i16_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,int8_t,rechrom_builtin_mul_i16_i16_i8)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,uint8_t,rechrom_builtin_add_i16_i16_u8,rechrom_builtin_sub_i16_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,uint8_t,rechrom_builtin_mul_i16_i16_u8)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,int16_t,rechrom_builtin_add_i16_i16_i16,rechrom_builtin_sub_i16_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,int16_t,rechrom_builtin_mul_i16_i16_i16)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,uint16_t,rechrom_builtin_add_i16_i16_u16,rechrom_builtin_sub_i16_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,uint16_t,rechrom_builtin_mul_i16_i16_u16)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,int32_t,rechrom_builtin_add_i16_i16_i32,rechrom_builtin_sub_i16_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,int32_t,rechrom_builtin_mul_i16_i16_i32)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,uint32_t,rechrom_builtin_add_i16_i16_u32,rechrom_builtin_sub_i16_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,uint32_t,rechrom_builtin_mul_i16_i16_u32)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,int64_t,rechrom_builtin_add_i16_i16_i64,rechrom_builtin_sub_i16_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,int64_t,rechrom_builtin_mul_i16_i16_i64)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,uint64_t,rechrom_builtin_add_i16_i16_u64,rechrom_builtin_sub_i16_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,uint64_t,rechrom_builtin_mul_i16_i16_u64)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,intptr_t,rechrom_builtin_add_i16_i16_isize,rechrom_builtin_sub_i16_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,intptr_t,rechrom_builtin_mul_i16_i16_isize)
#endif
RECHROM_ADD_SUB(int16_t,int16_t,uintptr_t,rechrom_builtin_add_i16_i16_usize,rechrom_builtin_sub_i16_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int16_t,uintptr_t,rechrom_builtin_mul_i16_i16_usize)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,int8_t,rechrom_builtin_add_i16_u16_i8,rechrom_builtin_sub_i16_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,int8_t,rechrom_builtin_mul_i16_u16_i8)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,uint8_t,rechrom_builtin_add_i16_u16_u8,rechrom_builtin_sub_i16_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,uint8_t,rechrom_builtin_mul_i16_u16_u8)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,int16_t,rechrom_builtin_add_i16_u16_i16,rechrom_builtin_sub_i16_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,int16_t,rechrom_builtin_mul_i16_u16_i16)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,uint16_t,rechrom_builtin_add_i16_u16_u16,rechrom_builtin_sub_i16_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,uint16_t,rechrom_builtin_mul_i16_u16_u16)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,int32_t,rechrom_builtin_add_i16_u16_i32,rechrom_builtin_sub_i16_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,int32_t,rechrom_builtin_mul_i16_u16_i32)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,uint32_t,rechrom_builtin_add_i16_u16_u32,rechrom_builtin_sub_i16_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,uint32_t,rechrom_builtin_mul_i16_u16_u32)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,int64_t,rechrom_builtin_add_i16_u16_i64,rechrom_builtin_sub_i16_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,int64_t,rechrom_builtin_mul_i16_u16_i64)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,uint64_t,rechrom_builtin_add_i16_u16_u64,rechrom_builtin_sub_i16_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,uint64_t,rechrom_builtin_mul_i16_u16_u64)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,intptr_t,rechrom_builtin_add_i16_u16_isize,rechrom_builtin_sub_i16_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,intptr_t,rechrom_builtin_mul_i16_u16_isize)
#endif
RECHROM_ADD_SUB(int16_t,uint16_t,uintptr_t,rechrom_builtin_add_i16_u16_usize,rechrom_builtin_sub_i16_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint16_t,uintptr_t,rechrom_builtin_mul_i16_u16_usize)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,int8_t,rechrom_builtin_add_i16_i32_i8,rechrom_builtin_sub_i16_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,int8_t,rechrom_builtin_mul_i16_i32_i8)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,uint8_t,rechrom_builtin_add_i16_i32_u8,rechrom_builtin_sub_i16_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,uint8_t,rechrom_builtin_mul_i16_i32_u8)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,int16_t,rechrom_builtin_add_i16_i32_i16,rechrom_builtin_sub_i16_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,int16_t,rechrom_builtin_mul_i16_i32_i16)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,uint16_t,rechrom_builtin_add_i16_i32_u16,rechrom_builtin_sub_i16_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,uint16_t,rechrom_builtin_mul_i16_i32_u16)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,int32_t,rechrom_builtin_add_i16_i32_i32,rechrom_builtin_sub_i16_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,int32_t,rechrom_builtin_mul_i16_i32_i32)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,uint32_t,rechrom_builtin_add_i16_i32_u32,rechrom_builtin_sub_i16_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,uint32_t,rechrom_builtin_mul_i16_i32_u32)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,int64_t,rechrom_builtin_add_i16_i32_i64,rechrom_builtin_sub_i16_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,int64_t,rechrom_builtin_mul_i16_i32_i64)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,uint64_t,rechrom_builtin_add_i16_i32_u64,rechrom_builtin_sub_i16_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,uint64_t,rechrom_builtin_mul_i16_i32_u64)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,intptr_t,rechrom_builtin_add_i16_i32_isize,rechrom_builtin_sub_i16_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,intptr_t,rechrom_builtin_mul_i16_i32_isize)
#endif
RECHROM_ADD_SUB(int16_t,int32_t,uintptr_t,rechrom_builtin_add_i16_i32_usize,rechrom_builtin_sub_i16_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int32_t,uintptr_t,rechrom_builtin_mul_i16_i32_usize)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,int8_t,rechrom_builtin_add_i16_u32_i8,rechrom_builtin_sub_i16_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,int8_t,rechrom_builtin_mul_i16_u32_i8)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,uint8_t,rechrom_builtin_add_i16_u32_u8,rechrom_builtin_sub_i16_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,uint8_t,rechrom_builtin_mul_i16_u32_u8)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,int16_t,rechrom_builtin_add_i16_u32_i16,rechrom_builtin_sub_i16_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,int16_t,rechrom_builtin_mul_i16_u32_i16)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,uint16_t,rechrom_builtin_add_i16_u32_u16,rechrom_builtin_sub_i16_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,uint16_t,rechrom_builtin_mul_i16_u32_u16)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,int32_t,rechrom_builtin_add_i16_u32_i32,rechrom_builtin_sub_i16_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,int32_t,rechrom_builtin_mul_i16_u32_i32)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,uint32_t,rechrom_builtin_add_i16_u32_u32,rechrom_builtin_sub_i16_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,uint32_t,rechrom_builtin_mul_i16_u32_u32)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,int64_t,rechrom_builtin_add_i16_u32_i64,rechrom_builtin_sub_i16_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,int64_t,rechrom_builtin_mul_i16_u32_i64)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,uint64_t,rechrom_builtin_add_i16_u32_u64,rechrom_builtin_sub_i16_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,uint64_t,rechrom_builtin_mul_i16_u32_u64)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,intptr_t,rechrom_builtin_add_i16_u32_isize,rechrom_builtin_sub_i16_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,intptr_t,rechrom_builtin_mul_i16_u32_isize)
#endif
RECHROM_ADD_SUB(int16_t,uint32_t,uintptr_t,rechrom_builtin_add_i16_u32_usize,rechrom_builtin_sub_i16_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,uint32_t,uintptr_t,rechrom_builtin_mul_i16_u32_usize)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,int8_t,rechrom_builtin_add_i16_i64_i8,rechrom_builtin_sub_i16_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,int8_t,rechrom_builtin_mul_i16_i64_i8)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,uint8_t,rechrom_builtin_add_i16_i64_u8,rechrom_builtin_sub_i16_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,uint8_t,rechrom_builtin_mul_i16_i64_u8)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,int16_t,rechrom_builtin_add_i16_i64_i16,rechrom_builtin_sub_i16_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,int16_t,rechrom_builtin_mul_i16_i64_i16)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,uint16_t,rechrom_builtin_add_i16_i64_u16,rechrom_builtin_sub_i16_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,uint16_t,rechrom_builtin_mul_i16_i64_u16)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,int32_t,rechrom_builtin_add_i16_i64_i32,rechrom_builtin_sub_i16_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,int32_t,rechrom_builtin_mul_i16_i64_i32)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,uint32_t,rechrom_builtin_add_i16_i64_u32,rechrom_builtin_sub_i16_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,uint32_t,rechrom_builtin_mul_i16_i64_u32)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,int64_t,rechrom_builtin_add_i16_i64_i64,rechrom_builtin_sub_i16_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,int64_t,rechrom_builtin_mul_i16_i64_i64)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,uint64_t,rechrom_builtin_add_i16_i64_u64,rechrom_builtin_sub_i16_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,uint64_t,rechrom_builtin_mul_i16_i64_u64)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,intptr_t,rechrom_builtin_add_i16_i64_isize,rechrom_builtin_sub_i16_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,intptr_t,rechrom_builtin_mul_i16_i64_isize)
#endif
RECHROM_ADD_SUB(int16_t,int64_t,uintptr_t,rechrom_builtin_add_i16_i64_usize,rechrom_builtin_sub_i16_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,int64_t,uintptr_t,rechrom_builtin_mul_i16_i64_usize)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,int8_t,rechrom_builtin_add_i16_u64_i8,rechrom_builtin_sub_i16_u64_i8)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,int8_t,rechrom_builtin_mul_i16_u64_i8)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,uint8_t,rechrom_builtin_add_i16_u64_u8,rechrom_builtin_sub_i16_u64_u8)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,uint8_t,rechrom_builtin_mul_i16_u64_u8)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,int16_t,rechrom_builtin_add_i16_u64_i16,rechrom_builtin_sub_i16_u64_i16)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,int16_t,rechrom_builtin_mul_i16_u64_i16)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,uint16_t,rechrom_builtin_add_i16_u64_u16,rechrom_builtin_sub_i16_u64_u16)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,uint16_t,rechrom_builtin_mul_i16_u64_u16)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,int32_t,rechrom_builtin_add_i16_u64_i32,rechrom_builtin_sub_i16_u64_i32)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,int32_t,rechrom_builtin_mul_i16_u64_i32)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,uint32_t,rechrom_builtin_add_i16_u64_u32,rechrom_builtin_sub_i16_u64_u32)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,uint32_t,rechrom_builtin_mul_i16_u64_u32)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,int64_t,rechrom_builtin_add_i16_u64_i64,rechrom_builtin_sub_i16_u64_i64)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,int64_t,rechrom_builtin_mul_i16_u64_i64)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,uint64_t,rechrom_builtin_add_i16_u64_u64,rechrom_builtin_sub_i16_u64_u64)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,uint64_t,rechrom_builtin_mul_i16_u64_u64)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,intptr_t,rechrom_builtin_add_i16_u64_isize,rechrom_builtin_sub_i16_u64_isize)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,intptr_t,rechrom_builtin_mul_i16_u64_isize)
#endif
RECHROM_ADD_SUB(int16_t,uint64_t,uintptr_t,rechrom_builtin_add_i16_u64_usize,rechrom_builtin_sub_i16_u64_usize)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uint64_t,uintptr_t,rechrom_builtin_mul_i16_u64_usize)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,int8_t,rechrom_builtin_add_i16_isize_i8,rechrom_builtin_sub_i16_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,int8_t,rechrom_builtin_mul_i16_isize_i8)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,uint8_t,rechrom_builtin_add_i16_isize_u8,rechrom_builtin_sub_i16_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,uint8_t,rechrom_builtin_mul_i16_isize_u8)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,int16_t,rechrom_builtin_add_i16_isize_i16,rechrom_builtin_sub_i16_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,int16_t,rechrom_builtin_mul_i16_isize_i16)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,uint16_t,rechrom_builtin_add_i16_isize_u16,rechrom_builtin_sub_i16_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,uint16_t,rechrom_builtin_mul_i16_isize_u16)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,int32_t,rechrom_builtin_add_i16_isize_i32,rechrom_builtin_sub_i16_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,int32_t,rechrom_builtin_mul_i16_isize_i32)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,uint32_t,rechrom_builtin_add_i16_isize_u32,rechrom_builtin_sub_i16_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,uint32_t,rechrom_builtin_mul_i16_isize_u32)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,int64_t,rechrom_builtin_add_i16_isize_i64,rechrom_builtin_sub_i16_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,int64_t,rechrom_builtin_mul_i16_isize_i64)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,uint64_t,rechrom_builtin_add_i16_isize_u64,rechrom_builtin_sub_i16_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,uint64_t,rechrom_builtin_mul_i16_isize_u64)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,intptr_t,rechrom_builtin_add_i16_isize_isize,rechrom_builtin_sub_i16_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,intptr_t,rechrom_builtin_mul_i16_isize_isize)
#endif
RECHROM_ADD_SUB(int16_t,intptr_t,uintptr_t,rechrom_builtin_add_i16_isize_usize,rechrom_builtin_sub_i16_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int16_t,intptr_t,uintptr_t,rechrom_builtin_mul_i16_isize_usize)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,int8_t,rechrom_builtin_add_i16_usize_i8,rechrom_builtin_sub_i16_usize_i8)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,int8_t,rechrom_builtin_mul_i16_usize_i8)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,uint8_t,rechrom_builtin_add_i16_usize_u8,rechrom_builtin_sub_i16_usize_u8)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,uint8_t,rechrom_builtin_mul_i16_usize_u8)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,int16_t,rechrom_builtin_add_i16_usize_i16,rechrom_builtin_sub_i16_usize_i16)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,int16_t,rechrom_builtin_mul_i16_usize_i16)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,uint16_t,rechrom_builtin_add_i16_usize_u16,rechrom_builtin_sub_i16_usize_u16)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,uint16_t,rechrom_builtin_mul_i16_usize_u16)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,int32_t,rechrom_builtin_add_i16_usize_i32,rechrom_builtin_sub_i16_usize_i32)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,int32_t,rechrom_builtin_mul_i16_usize_i32)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,uint32_t,rechrom_builtin_add_i16_usize_u32,rechrom_builtin_sub_i16_usize_u32)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,uint32_t,rechrom_builtin_mul_i16_usize_u32)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,int64_t,rechrom_builtin_add_i16_usize_i64,rechrom_builtin_sub_i16_usize_i64)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,int64_t,rechrom_builtin_mul_i16_usize_i64)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,uint64_t,rechrom_builtin_add_i16_usize_u64,rechrom_builtin_sub_i16_usize_u64)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,uint64_t,rechrom_builtin_mul_i16_usize_u64)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,intptr_t,rechrom_builtin_add_i16_usize_isize,rechrom_builtin_sub_i16_usize_isize)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,intptr_t,rechrom_builtin_mul_i16_usize_isize)
#endif
RECHROM_ADD_SUB(int16_t,uintptr_t,uintptr_t,rechrom_builtin_add_i16_usize_usize,rechrom_builtin_sub_i16_usize_usize)
#if !defined(__clang__)
RECHROM_MUL(int16_t,uintptr_t,uintptr_t,rechrom_builtin_mul_i16_usize_usize)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,int8_t,rechrom_builtin_add_u16_i8_i8,rechrom_builtin_sub_u16_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,int8_t,rechrom_builtin_mul_u16_i8_i8)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,uint8_t,rechrom_builtin_add_u16_i8_u8,rechrom_builtin_sub_u16_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,uint8_t,rechrom_builtin_mul_u16_i8_u8)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,int16_t,rechrom_builtin_add_u16_i8_i16,rechrom_builtin_sub_u16_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,int16_t,rechrom_builtin_mul_u16_i8_i16)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,uint16_t,rechrom_builtin_add_u16_i8_u16,rechrom_builtin_sub_u16_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,uint16_t,rechrom_builtin_mul_u16_i8_u16)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,int32_t,rechrom_builtin_add_u16_i8_i32,rechrom_builtin_sub_u16_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,int32_t,rechrom_builtin_mul_u16_i8_i32)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,uint32_t,rechrom_builtin_add_u16_i8_u32,rechrom_builtin_sub_u16_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,uint32_t,rechrom_builtin_mul_u16_i8_u32)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,int64_t,rechrom_builtin_add_u16_i8_i64,rechrom_builtin_sub_u16_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,int64_t,rechrom_builtin_mul_u16_i8_i64)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,uint64_t,rechrom_builtin_add_u16_i8_u64,rechrom_builtin_sub_u16_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,uint64_t,rechrom_builtin_mul_u16_i8_u64)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,intptr_t,rechrom_builtin_add_u16_i8_isize,rechrom_builtin_sub_u16_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,intptr_t,rechrom_builtin_mul_u16_i8_isize)
#endif
RECHROM_ADD_SUB(uint16_t,int8_t,uintptr_t,rechrom_builtin_add_u16_i8_usize,rechrom_builtin_sub_u16_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int8_t,uintptr_t,rechrom_builtin_mul_u16_i8_usize)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,int8_t,rechrom_builtin_add_u16_u8_i8,rechrom_builtin_sub_u16_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,int8_t,rechrom_builtin_mul_u16_u8_i8)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,uint8_t,rechrom_builtin_add_u16_u8_u8,rechrom_builtin_sub_u16_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,uint8_t,rechrom_builtin_mul_u16_u8_u8)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,int16_t,rechrom_builtin_add_u16_u8_i16,rechrom_builtin_sub_u16_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,int16_t,rechrom_builtin_mul_u16_u8_i16)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,uint16_t,rechrom_builtin_add_u16_u8_u16,rechrom_builtin_sub_u16_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,uint16_t,rechrom_builtin_mul_u16_u8_u16)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,int32_t,rechrom_builtin_add_u16_u8_i32,rechrom_builtin_sub_u16_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,int32_t,rechrom_builtin_mul_u16_u8_i32)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,uint32_t,rechrom_builtin_add_u16_u8_u32,rechrom_builtin_sub_u16_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,uint32_t,rechrom_builtin_mul_u16_u8_u32)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,int64_t,rechrom_builtin_add_u16_u8_i64,rechrom_builtin_sub_u16_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,int64_t,rechrom_builtin_mul_u16_u8_i64)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,uint64_t,rechrom_builtin_add_u16_u8_u64,rechrom_builtin_sub_u16_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,uint64_t,rechrom_builtin_mul_u16_u8_u64)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,intptr_t,rechrom_builtin_add_u16_u8_isize,rechrom_builtin_sub_u16_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,intptr_t,rechrom_builtin_mul_u16_u8_isize)
#endif
RECHROM_ADD_SUB(uint16_t,uint8_t,uintptr_t,rechrom_builtin_add_u16_u8_usize,rechrom_builtin_sub_u16_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint8_t,uintptr_t,rechrom_builtin_mul_u16_u8_usize)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,int8_t,rechrom_builtin_add_u16_i16_i8,rechrom_builtin_sub_u16_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,int8_t,rechrom_builtin_mul_u16_i16_i8)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,uint8_t,rechrom_builtin_add_u16_i16_u8,rechrom_builtin_sub_u16_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,uint8_t,rechrom_builtin_mul_u16_i16_u8)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,int16_t,rechrom_builtin_add_u16_i16_i16,rechrom_builtin_sub_u16_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,int16_t,rechrom_builtin_mul_u16_i16_i16)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,uint16_t,rechrom_builtin_add_u16_i16_u16,rechrom_builtin_sub_u16_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,uint16_t,rechrom_builtin_mul_u16_i16_u16)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,int32_t,rechrom_builtin_add_u16_i16_i32,rechrom_builtin_sub_u16_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,int32_t,rechrom_builtin_mul_u16_i16_i32)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,uint32_t,rechrom_builtin_add_u16_i16_u32,rechrom_builtin_sub_u16_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,uint32_t,rechrom_builtin_mul_u16_i16_u32)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,int64_t,rechrom_builtin_add_u16_i16_i64,rechrom_builtin_sub_u16_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,int64_t,rechrom_builtin_mul_u16_i16_i64)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,uint64_t,rechrom_builtin_add_u16_i16_u64,rechrom_builtin_sub_u16_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,uint64_t,rechrom_builtin_mul_u16_i16_u64)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,intptr_t,rechrom_builtin_add_u16_i16_isize,rechrom_builtin_sub_u16_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,intptr_t,rechrom_builtin_mul_u16_i16_isize)
#endif
RECHROM_ADD_SUB(uint16_t,int16_t,uintptr_t,rechrom_builtin_add_u16_i16_usize,rechrom_builtin_sub_u16_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int16_t,uintptr_t,rechrom_builtin_mul_u16_i16_usize)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,int8_t,rechrom_builtin_add_u16_u16_i8,rechrom_builtin_sub_u16_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,int8_t,rechrom_builtin_mul_u16_u16_i8)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,uint8_t,rechrom_builtin_add_u16_u16_u8,rechrom_builtin_sub_u16_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,uint8_t,rechrom_builtin_mul_u16_u16_u8)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,int16_t,rechrom_builtin_add_u16_u16_i16,rechrom_builtin_sub_u16_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,int16_t,rechrom_builtin_mul_u16_u16_i16)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,uint16_t,rechrom_builtin_add_u16_u16_u16,rechrom_builtin_sub_u16_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,uint16_t,rechrom_builtin_mul_u16_u16_u16)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,int32_t,rechrom_builtin_add_u16_u16_i32,rechrom_builtin_sub_u16_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,int32_t,rechrom_builtin_mul_u16_u16_i32)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,uint32_t,rechrom_builtin_add_u16_u16_u32,rechrom_builtin_sub_u16_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,uint32_t,rechrom_builtin_mul_u16_u16_u32)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,int64_t,rechrom_builtin_add_u16_u16_i64,rechrom_builtin_sub_u16_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,int64_t,rechrom_builtin_mul_u16_u16_i64)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,uint64_t,rechrom_builtin_add_u16_u16_u64,rechrom_builtin_sub_u16_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,uint64_t,rechrom_builtin_mul_u16_u16_u64)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,intptr_t,rechrom_builtin_add_u16_u16_isize,rechrom_builtin_sub_u16_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,intptr_t,rechrom_builtin_mul_u16_u16_isize)
#endif
RECHROM_ADD_SUB(uint16_t,uint16_t,uintptr_t,rechrom_builtin_add_u16_u16_usize,rechrom_builtin_sub_u16_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint16_t,uintptr_t,rechrom_builtin_mul_u16_u16_usize)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,int8_t,rechrom_builtin_add_u16_i32_i8,rechrom_builtin_sub_u16_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,int8_t,rechrom_builtin_mul_u16_i32_i8)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,uint8_t,rechrom_builtin_add_u16_i32_u8,rechrom_builtin_sub_u16_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,uint8_t,rechrom_builtin_mul_u16_i32_u8)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,int16_t,rechrom_builtin_add_u16_i32_i16,rechrom_builtin_sub_u16_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,int16_t,rechrom_builtin_mul_u16_i32_i16)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,uint16_t,rechrom_builtin_add_u16_i32_u16,rechrom_builtin_sub_u16_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,uint16_t,rechrom_builtin_mul_u16_i32_u16)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,int32_t,rechrom_builtin_add_u16_i32_i32,rechrom_builtin_sub_u16_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,int32_t,rechrom_builtin_mul_u16_i32_i32)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,uint32_t,rechrom_builtin_add_u16_i32_u32,rechrom_builtin_sub_u16_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,uint32_t,rechrom_builtin_mul_u16_i32_u32)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,int64_t,rechrom_builtin_add_u16_i32_i64,rechrom_builtin_sub_u16_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,int64_t,rechrom_builtin_mul_u16_i32_i64)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,uint64_t,rechrom_builtin_add_u16_i32_u64,rechrom_builtin_sub_u16_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,uint64_t,rechrom_builtin_mul_u16_i32_u64)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,intptr_t,rechrom_builtin_add_u16_i32_isize,rechrom_builtin_sub_u16_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,intptr_t,rechrom_builtin_mul_u16_i32_isize)
#endif
RECHROM_ADD_SUB(uint16_t,int32_t,uintptr_t,rechrom_builtin_add_u16_i32_usize,rechrom_builtin_sub_u16_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int32_t,uintptr_t,rechrom_builtin_mul_u16_i32_usize)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,int8_t,rechrom_builtin_add_u16_u32_i8,rechrom_builtin_sub_u16_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,int8_t,rechrom_builtin_mul_u16_u32_i8)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,uint8_t,rechrom_builtin_add_u16_u32_u8,rechrom_builtin_sub_u16_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,uint8_t,rechrom_builtin_mul_u16_u32_u8)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,int16_t,rechrom_builtin_add_u16_u32_i16,rechrom_builtin_sub_u16_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,int16_t,rechrom_builtin_mul_u16_u32_i16)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,uint16_t,rechrom_builtin_add_u16_u32_u16,rechrom_builtin_sub_u16_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,uint16_t,rechrom_builtin_mul_u16_u32_u16)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,int32_t,rechrom_builtin_add_u16_u32_i32,rechrom_builtin_sub_u16_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,int32_t,rechrom_builtin_mul_u16_u32_i32)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,uint32_t,rechrom_builtin_add_u16_u32_u32,rechrom_builtin_sub_u16_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,uint32_t,rechrom_builtin_mul_u16_u32_u32)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,int64_t,rechrom_builtin_add_u16_u32_i64,rechrom_builtin_sub_u16_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,int64_t,rechrom_builtin_mul_u16_u32_i64)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,uint64_t,rechrom_builtin_add_u16_u32_u64,rechrom_builtin_sub_u16_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,uint64_t,rechrom_builtin_mul_u16_u32_u64)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,intptr_t,rechrom_builtin_add_u16_u32_isize,rechrom_builtin_sub_u16_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,intptr_t,rechrom_builtin_mul_u16_u32_isize)
#endif
RECHROM_ADD_SUB(uint16_t,uint32_t,uintptr_t,rechrom_builtin_add_u16_u32_usize,rechrom_builtin_sub_u16_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint32_t,uintptr_t,rechrom_builtin_mul_u16_u32_usize)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,int8_t,rechrom_builtin_add_u16_i64_i8,rechrom_builtin_sub_u16_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,int8_t,rechrom_builtin_mul_u16_i64_i8)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,uint8_t,rechrom_builtin_add_u16_i64_u8,rechrom_builtin_sub_u16_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,uint8_t,rechrom_builtin_mul_u16_i64_u8)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,int16_t,rechrom_builtin_add_u16_i64_i16,rechrom_builtin_sub_u16_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,int16_t,rechrom_builtin_mul_u16_i64_i16)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,uint16_t,rechrom_builtin_add_u16_i64_u16,rechrom_builtin_sub_u16_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,uint16_t,rechrom_builtin_mul_u16_i64_u16)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,int32_t,rechrom_builtin_add_u16_i64_i32,rechrom_builtin_sub_u16_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,int32_t,rechrom_builtin_mul_u16_i64_i32)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,uint32_t,rechrom_builtin_add_u16_i64_u32,rechrom_builtin_sub_u16_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,uint32_t,rechrom_builtin_mul_u16_i64_u32)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,int64_t,rechrom_builtin_add_u16_i64_i64,rechrom_builtin_sub_u16_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,int64_t,rechrom_builtin_mul_u16_i64_i64)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,uint64_t,rechrom_builtin_add_u16_i64_u64,rechrom_builtin_sub_u16_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,uint64_t,rechrom_builtin_mul_u16_i64_u64)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,intptr_t,rechrom_builtin_add_u16_i64_isize,rechrom_builtin_sub_u16_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,intptr_t,rechrom_builtin_mul_u16_i64_isize)
#endif
RECHROM_ADD_SUB(uint16_t,int64_t,uintptr_t,rechrom_builtin_add_u16_i64_usize,rechrom_builtin_sub_u16_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,int64_t,uintptr_t,rechrom_builtin_mul_u16_i64_usize)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,int8_t,rechrom_builtin_add_u16_u64_i8,rechrom_builtin_sub_u16_u64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,int8_t,rechrom_builtin_mul_u16_u64_i8)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,uint8_t,rechrom_builtin_add_u16_u64_u8,rechrom_builtin_sub_u16_u64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,uint8_t,rechrom_builtin_mul_u16_u64_u8)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,int16_t,rechrom_builtin_add_u16_u64_i16,rechrom_builtin_sub_u16_u64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,int16_t,rechrom_builtin_mul_u16_u64_i16)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,uint16_t,rechrom_builtin_add_u16_u64_u16,rechrom_builtin_sub_u16_u64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,uint16_t,rechrom_builtin_mul_u16_u64_u16)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,int32_t,rechrom_builtin_add_u16_u64_i32,rechrom_builtin_sub_u16_u64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,int32_t,rechrom_builtin_mul_u16_u64_i32)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,uint32_t,rechrom_builtin_add_u16_u64_u32,rechrom_builtin_sub_u16_u64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,uint32_t,rechrom_builtin_mul_u16_u64_u32)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,int64_t,rechrom_builtin_add_u16_u64_i64,rechrom_builtin_sub_u16_u64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,int64_t,rechrom_builtin_mul_u16_u64_i64)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,uint64_t,rechrom_builtin_add_u16_u64_u64,rechrom_builtin_sub_u16_u64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,uint64_t,rechrom_builtin_mul_u16_u64_u64)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,intptr_t,rechrom_builtin_add_u16_u64_isize,rechrom_builtin_sub_u16_u64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,intptr_t,rechrom_builtin_mul_u16_u64_isize)
#endif
RECHROM_ADD_SUB(uint16_t,uint64_t,uintptr_t,rechrom_builtin_add_u16_u64_usize,rechrom_builtin_sub_u16_u64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uint64_t,uintptr_t,rechrom_builtin_mul_u16_u64_usize)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,int8_t,rechrom_builtin_add_u16_isize_i8,rechrom_builtin_sub_u16_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,int8_t,rechrom_builtin_mul_u16_isize_i8)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,uint8_t,rechrom_builtin_add_u16_isize_u8,rechrom_builtin_sub_u16_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,uint8_t,rechrom_builtin_mul_u16_isize_u8)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,int16_t,rechrom_builtin_add_u16_isize_i16,rechrom_builtin_sub_u16_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,int16_t,rechrom_builtin_mul_u16_isize_i16)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,uint16_t,rechrom_builtin_add_u16_isize_u16,rechrom_builtin_sub_u16_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,uint16_t,rechrom_builtin_mul_u16_isize_u16)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,int32_t,rechrom_builtin_add_u16_isize_i32,rechrom_builtin_sub_u16_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,int32_t,rechrom_builtin_mul_u16_isize_i32)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,uint32_t,rechrom_builtin_add_u16_isize_u32,rechrom_builtin_sub_u16_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,uint32_t,rechrom_builtin_mul_u16_isize_u32)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,int64_t,rechrom_builtin_add_u16_isize_i64,rechrom_builtin_sub_u16_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,int64_t,rechrom_builtin_mul_u16_isize_i64)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,uint64_t,rechrom_builtin_add_u16_isize_u64,rechrom_builtin_sub_u16_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,uint64_t,rechrom_builtin_mul_u16_isize_u64)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,intptr_t,rechrom_builtin_add_u16_isize_isize,rechrom_builtin_sub_u16_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,intptr_t,rechrom_builtin_mul_u16_isize_isize)
#endif
RECHROM_ADD_SUB(uint16_t,intptr_t,uintptr_t,rechrom_builtin_add_u16_isize_usize,rechrom_builtin_sub_u16_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,intptr_t,uintptr_t,rechrom_builtin_mul_u16_isize_usize)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,int8_t,rechrom_builtin_add_u16_usize_i8,rechrom_builtin_sub_u16_usize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,int8_t,rechrom_builtin_mul_u16_usize_i8)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,uint8_t,rechrom_builtin_add_u16_usize_u8,rechrom_builtin_sub_u16_usize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,uint8_t,rechrom_builtin_mul_u16_usize_u8)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,int16_t,rechrom_builtin_add_u16_usize_i16,rechrom_builtin_sub_u16_usize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,int16_t,rechrom_builtin_mul_u16_usize_i16)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,uint16_t,rechrom_builtin_add_u16_usize_u16,rechrom_builtin_sub_u16_usize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,uint16_t,rechrom_builtin_mul_u16_usize_u16)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,int32_t,rechrom_builtin_add_u16_usize_i32,rechrom_builtin_sub_u16_usize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,int32_t,rechrom_builtin_mul_u16_usize_i32)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,uint32_t,rechrom_builtin_add_u16_usize_u32,rechrom_builtin_sub_u16_usize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,uint32_t,rechrom_builtin_mul_u16_usize_u32)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,int64_t,rechrom_builtin_add_u16_usize_i64,rechrom_builtin_sub_u16_usize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,int64_t,rechrom_builtin_mul_u16_usize_i64)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,uint64_t,rechrom_builtin_add_u16_usize_u64,rechrom_builtin_sub_u16_usize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,uint64_t,rechrom_builtin_mul_u16_usize_u64)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,intptr_t,rechrom_builtin_add_u16_usize_isize,rechrom_builtin_sub_u16_usize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,intptr_t,rechrom_builtin_mul_u16_usize_isize)
#endif
RECHROM_ADD_SUB(uint16_t,uintptr_t,uintptr_t,rechrom_builtin_add_u16_usize_usize,rechrom_builtin_sub_u16_usize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint16_t,uintptr_t,uintptr_t,rechrom_builtin_mul_u16_usize_usize)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,int8_t,rechrom_builtin_add_i32_i8_i8,rechrom_builtin_sub_i32_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,int8_t,rechrom_builtin_mul_i32_i8_i8)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,uint8_t,rechrom_builtin_add_i32_i8_u8,rechrom_builtin_sub_i32_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,uint8_t,rechrom_builtin_mul_i32_i8_u8)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,int16_t,rechrom_builtin_add_i32_i8_i16,rechrom_builtin_sub_i32_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,int16_t,rechrom_builtin_mul_i32_i8_i16)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,uint16_t,rechrom_builtin_add_i32_i8_u16,rechrom_builtin_sub_i32_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,uint16_t,rechrom_builtin_mul_i32_i8_u16)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,int32_t,rechrom_builtin_add_i32_i8_i32,rechrom_builtin_sub_i32_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,int32_t,rechrom_builtin_mul_i32_i8_i32)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,uint32_t,rechrom_builtin_add_i32_i8_u32,rechrom_builtin_sub_i32_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,uint32_t,rechrom_builtin_mul_i32_i8_u32)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,int64_t,rechrom_builtin_add_i32_i8_i64,rechrom_builtin_sub_i32_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,int64_t,rechrom_builtin_mul_i32_i8_i64)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,uint64_t,rechrom_builtin_add_i32_i8_u64,rechrom_builtin_sub_i32_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,uint64_t,rechrom_builtin_mul_i32_i8_u64)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,intptr_t,rechrom_builtin_add_i32_i8_isize,rechrom_builtin_sub_i32_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,intptr_t,rechrom_builtin_mul_i32_i8_isize)
#endif
RECHROM_ADD_SUB(int32_t,int8_t,uintptr_t,rechrom_builtin_add_i32_i8_usize,rechrom_builtin_sub_i32_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int8_t,uintptr_t,rechrom_builtin_mul_i32_i8_usize)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,int8_t,rechrom_builtin_add_i32_u8_i8,rechrom_builtin_sub_i32_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,int8_t,rechrom_builtin_mul_i32_u8_i8)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,uint8_t,rechrom_builtin_add_i32_u8_u8,rechrom_builtin_sub_i32_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,uint8_t,rechrom_builtin_mul_i32_u8_u8)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,int16_t,rechrom_builtin_add_i32_u8_i16,rechrom_builtin_sub_i32_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,int16_t,rechrom_builtin_mul_i32_u8_i16)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,uint16_t,rechrom_builtin_add_i32_u8_u16,rechrom_builtin_sub_i32_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,uint16_t,rechrom_builtin_mul_i32_u8_u16)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,int32_t,rechrom_builtin_add_i32_u8_i32,rechrom_builtin_sub_i32_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,int32_t,rechrom_builtin_mul_i32_u8_i32)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,uint32_t,rechrom_builtin_add_i32_u8_u32,rechrom_builtin_sub_i32_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,uint32_t,rechrom_builtin_mul_i32_u8_u32)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,int64_t,rechrom_builtin_add_i32_u8_i64,rechrom_builtin_sub_i32_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,int64_t,rechrom_builtin_mul_i32_u8_i64)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,uint64_t,rechrom_builtin_add_i32_u8_u64,rechrom_builtin_sub_i32_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,uint64_t,rechrom_builtin_mul_i32_u8_u64)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,intptr_t,rechrom_builtin_add_i32_u8_isize,rechrom_builtin_sub_i32_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,intptr_t,rechrom_builtin_mul_i32_u8_isize)
#endif
RECHROM_ADD_SUB(int32_t,uint8_t,uintptr_t,rechrom_builtin_add_i32_u8_usize,rechrom_builtin_sub_i32_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint8_t,uintptr_t,rechrom_builtin_mul_i32_u8_usize)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,int8_t,rechrom_builtin_add_i32_i16_i8,rechrom_builtin_sub_i32_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,int8_t,rechrom_builtin_mul_i32_i16_i8)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,uint8_t,rechrom_builtin_add_i32_i16_u8,rechrom_builtin_sub_i32_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,uint8_t,rechrom_builtin_mul_i32_i16_u8)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,int16_t,rechrom_builtin_add_i32_i16_i16,rechrom_builtin_sub_i32_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,int16_t,rechrom_builtin_mul_i32_i16_i16)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,uint16_t,rechrom_builtin_add_i32_i16_u16,rechrom_builtin_sub_i32_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,uint16_t,rechrom_builtin_mul_i32_i16_u16)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,int32_t,rechrom_builtin_add_i32_i16_i32,rechrom_builtin_sub_i32_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,int32_t,rechrom_builtin_mul_i32_i16_i32)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,uint32_t,rechrom_builtin_add_i32_i16_u32,rechrom_builtin_sub_i32_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,uint32_t,rechrom_builtin_mul_i32_i16_u32)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,int64_t,rechrom_builtin_add_i32_i16_i64,rechrom_builtin_sub_i32_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,int64_t,rechrom_builtin_mul_i32_i16_i64)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,uint64_t,rechrom_builtin_add_i32_i16_u64,rechrom_builtin_sub_i32_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,uint64_t,rechrom_builtin_mul_i32_i16_u64)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,intptr_t,rechrom_builtin_add_i32_i16_isize,rechrom_builtin_sub_i32_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,intptr_t,rechrom_builtin_mul_i32_i16_isize)
#endif
RECHROM_ADD_SUB(int32_t,int16_t,uintptr_t,rechrom_builtin_add_i32_i16_usize,rechrom_builtin_sub_i32_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int16_t,uintptr_t,rechrom_builtin_mul_i32_i16_usize)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,int8_t,rechrom_builtin_add_i32_u16_i8,rechrom_builtin_sub_i32_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,int8_t,rechrom_builtin_mul_i32_u16_i8)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,uint8_t,rechrom_builtin_add_i32_u16_u8,rechrom_builtin_sub_i32_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,uint8_t,rechrom_builtin_mul_i32_u16_u8)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,int16_t,rechrom_builtin_add_i32_u16_i16,rechrom_builtin_sub_i32_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,int16_t,rechrom_builtin_mul_i32_u16_i16)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,uint16_t,rechrom_builtin_add_i32_u16_u16,rechrom_builtin_sub_i32_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,uint16_t,rechrom_builtin_mul_i32_u16_u16)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,int32_t,rechrom_builtin_add_i32_u16_i32,rechrom_builtin_sub_i32_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,int32_t,rechrom_builtin_mul_i32_u16_i32)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,uint32_t,rechrom_builtin_add_i32_u16_u32,rechrom_builtin_sub_i32_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,uint32_t,rechrom_builtin_mul_i32_u16_u32)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,int64_t,rechrom_builtin_add_i32_u16_i64,rechrom_builtin_sub_i32_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,int64_t,rechrom_builtin_mul_i32_u16_i64)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,uint64_t,rechrom_builtin_add_i32_u16_u64,rechrom_builtin_sub_i32_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,uint64_t,rechrom_builtin_mul_i32_u16_u64)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,intptr_t,rechrom_builtin_add_i32_u16_isize,rechrom_builtin_sub_i32_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,intptr_t,rechrom_builtin_mul_i32_u16_isize)
#endif
RECHROM_ADD_SUB(int32_t,uint16_t,uintptr_t,rechrom_builtin_add_i32_u16_usize,rechrom_builtin_sub_i32_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint16_t,uintptr_t,rechrom_builtin_mul_i32_u16_usize)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,int8_t,rechrom_builtin_add_i32_i32_i8,rechrom_builtin_sub_i32_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,int8_t,rechrom_builtin_mul_i32_i32_i8)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,uint8_t,rechrom_builtin_add_i32_i32_u8,rechrom_builtin_sub_i32_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,uint8_t,rechrom_builtin_mul_i32_i32_u8)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,int16_t,rechrom_builtin_add_i32_i32_i16,rechrom_builtin_sub_i32_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,int16_t,rechrom_builtin_mul_i32_i32_i16)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,uint16_t,rechrom_builtin_add_i32_i32_u16,rechrom_builtin_sub_i32_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,uint16_t,rechrom_builtin_mul_i32_i32_u16)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,int32_t,rechrom_builtin_add_i32_i32_i32,rechrom_builtin_sub_i32_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,int32_t,rechrom_builtin_mul_i32_i32_i32)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,uint32_t,rechrom_builtin_add_i32_i32_u32,rechrom_builtin_sub_i32_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,uint32_t,rechrom_builtin_mul_i32_i32_u32)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,int64_t,rechrom_builtin_add_i32_i32_i64,rechrom_builtin_sub_i32_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,int64_t,rechrom_builtin_mul_i32_i32_i64)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,uint64_t,rechrom_builtin_add_i32_i32_u64,rechrom_builtin_sub_i32_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,uint64_t,rechrom_builtin_mul_i32_i32_u64)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,intptr_t,rechrom_builtin_add_i32_i32_isize,rechrom_builtin_sub_i32_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,intptr_t,rechrom_builtin_mul_i32_i32_isize)
#endif
RECHROM_ADD_SUB(int32_t,int32_t,uintptr_t,rechrom_builtin_add_i32_i32_usize,rechrom_builtin_sub_i32_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int32_t,uintptr_t,rechrom_builtin_mul_i32_i32_usize)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,int8_t,rechrom_builtin_add_i32_u32_i8,rechrom_builtin_sub_i32_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,int8_t,rechrom_builtin_mul_i32_u32_i8)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,uint8_t,rechrom_builtin_add_i32_u32_u8,rechrom_builtin_sub_i32_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,uint8_t,rechrom_builtin_mul_i32_u32_u8)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,int16_t,rechrom_builtin_add_i32_u32_i16,rechrom_builtin_sub_i32_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,int16_t,rechrom_builtin_mul_i32_u32_i16)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,uint16_t,rechrom_builtin_add_i32_u32_u16,rechrom_builtin_sub_i32_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,uint16_t,rechrom_builtin_mul_i32_u32_u16)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,int32_t,rechrom_builtin_add_i32_u32_i32,rechrom_builtin_sub_i32_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,int32_t,rechrom_builtin_mul_i32_u32_i32)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,uint32_t,rechrom_builtin_add_i32_u32_u32,rechrom_builtin_sub_i32_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,uint32_t,rechrom_builtin_mul_i32_u32_u32)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,int64_t,rechrom_builtin_add_i32_u32_i64,rechrom_builtin_sub_i32_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,int64_t,rechrom_builtin_mul_i32_u32_i64)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,uint64_t,rechrom_builtin_add_i32_u32_u64,rechrom_builtin_sub_i32_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,uint64_t,rechrom_builtin_mul_i32_u32_u64)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,intptr_t,rechrom_builtin_add_i32_u32_isize,rechrom_builtin_sub_i32_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,intptr_t,rechrom_builtin_mul_i32_u32_isize)
#endif
RECHROM_ADD_SUB(int32_t,uint32_t,uintptr_t,rechrom_builtin_add_i32_u32_usize,rechrom_builtin_sub_i32_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,uint32_t,uintptr_t,rechrom_builtin_mul_i32_u32_usize)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,int8_t,rechrom_builtin_add_i32_i64_i8,rechrom_builtin_sub_i32_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,int8_t,rechrom_builtin_mul_i32_i64_i8)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,uint8_t,rechrom_builtin_add_i32_i64_u8,rechrom_builtin_sub_i32_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,uint8_t,rechrom_builtin_mul_i32_i64_u8)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,int16_t,rechrom_builtin_add_i32_i64_i16,rechrom_builtin_sub_i32_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,int16_t,rechrom_builtin_mul_i32_i64_i16)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,uint16_t,rechrom_builtin_add_i32_i64_u16,rechrom_builtin_sub_i32_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,uint16_t,rechrom_builtin_mul_i32_i64_u16)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,int32_t,rechrom_builtin_add_i32_i64_i32,rechrom_builtin_sub_i32_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,int32_t,rechrom_builtin_mul_i32_i64_i32)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,uint32_t,rechrom_builtin_add_i32_i64_u32,rechrom_builtin_sub_i32_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,uint32_t,rechrom_builtin_mul_i32_i64_u32)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,int64_t,rechrom_builtin_add_i32_i64_i64,rechrom_builtin_sub_i32_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,int64_t,rechrom_builtin_mul_i32_i64_i64)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,uint64_t,rechrom_builtin_add_i32_i64_u64,rechrom_builtin_sub_i32_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,uint64_t,rechrom_builtin_mul_i32_i64_u64)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,intptr_t,rechrom_builtin_add_i32_i64_isize,rechrom_builtin_sub_i32_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,intptr_t,rechrom_builtin_mul_i32_i64_isize)
#endif
RECHROM_ADD_SUB(int32_t,int64_t,uintptr_t,rechrom_builtin_add_i32_i64_usize,rechrom_builtin_sub_i32_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,int64_t,uintptr_t,rechrom_builtin_mul_i32_i64_usize)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,int8_t,rechrom_builtin_add_i32_u64_i8,rechrom_builtin_sub_i32_u64_i8)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,int8_t,rechrom_builtin_mul_i32_u64_i8)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,uint8_t,rechrom_builtin_add_i32_u64_u8,rechrom_builtin_sub_i32_u64_u8)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,uint8_t,rechrom_builtin_mul_i32_u64_u8)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,int16_t,rechrom_builtin_add_i32_u64_i16,rechrom_builtin_sub_i32_u64_i16)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,int16_t,rechrom_builtin_mul_i32_u64_i16)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,uint16_t,rechrom_builtin_add_i32_u64_u16,rechrom_builtin_sub_i32_u64_u16)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,uint16_t,rechrom_builtin_mul_i32_u64_u16)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,int32_t,rechrom_builtin_add_i32_u64_i32,rechrom_builtin_sub_i32_u64_i32)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,int32_t,rechrom_builtin_mul_i32_u64_i32)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,uint32_t,rechrom_builtin_add_i32_u64_u32,rechrom_builtin_sub_i32_u64_u32)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,uint32_t,rechrom_builtin_mul_i32_u64_u32)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,int64_t,rechrom_builtin_add_i32_u64_i64,rechrom_builtin_sub_i32_u64_i64)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,int64_t,rechrom_builtin_mul_i32_u64_i64)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,uint64_t,rechrom_builtin_add_i32_u64_u64,rechrom_builtin_sub_i32_u64_u64)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,uint64_t,rechrom_builtin_mul_i32_u64_u64)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,intptr_t,rechrom_builtin_add_i32_u64_isize,rechrom_builtin_sub_i32_u64_isize)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,intptr_t,rechrom_builtin_mul_i32_u64_isize)
#endif
RECHROM_ADD_SUB(int32_t,uint64_t,uintptr_t,rechrom_builtin_add_i32_u64_usize,rechrom_builtin_sub_i32_u64_usize)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uint64_t,uintptr_t,rechrom_builtin_mul_i32_u64_usize)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,int8_t,rechrom_builtin_add_i32_isize_i8,rechrom_builtin_sub_i32_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,int8_t,rechrom_builtin_mul_i32_isize_i8)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,uint8_t,rechrom_builtin_add_i32_isize_u8,rechrom_builtin_sub_i32_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,uint8_t,rechrom_builtin_mul_i32_isize_u8)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,int16_t,rechrom_builtin_add_i32_isize_i16,rechrom_builtin_sub_i32_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,int16_t,rechrom_builtin_mul_i32_isize_i16)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,uint16_t,rechrom_builtin_add_i32_isize_u16,rechrom_builtin_sub_i32_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,uint16_t,rechrom_builtin_mul_i32_isize_u16)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,int32_t,rechrom_builtin_add_i32_isize_i32,rechrom_builtin_sub_i32_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,int32_t,rechrom_builtin_mul_i32_isize_i32)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,uint32_t,rechrom_builtin_add_i32_isize_u32,rechrom_builtin_sub_i32_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,uint32_t,rechrom_builtin_mul_i32_isize_u32)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,int64_t,rechrom_builtin_add_i32_isize_i64,rechrom_builtin_sub_i32_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,int64_t,rechrom_builtin_mul_i32_isize_i64)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,uint64_t,rechrom_builtin_add_i32_isize_u64,rechrom_builtin_sub_i32_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,uint64_t,rechrom_builtin_mul_i32_isize_u64)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,intptr_t,rechrom_builtin_add_i32_isize_isize,rechrom_builtin_sub_i32_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,intptr_t,rechrom_builtin_mul_i32_isize_isize)
#endif
RECHROM_ADD_SUB(int32_t,intptr_t,uintptr_t,rechrom_builtin_add_i32_isize_usize,rechrom_builtin_sub_i32_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int32_t,intptr_t,uintptr_t,rechrom_builtin_mul_i32_isize_usize)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,int8_t,rechrom_builtin_add_i32_usize_i8,rechrom_builtin_sub_i32_usize_i8)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,int8_t,rechrom_builtin_mul_i32_usize_i8)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,uint8_t,rechrom_builtin_add_i32_usize_u8,rechrom_builtin_sub_i32_usize_u8)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,uint8_t,rechrom_builtin_mul_i32_usize_u8)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,int16_t,rechrom_builtin_add_i32_usize_i16,rechrom_builtin_sub_i32_usize_i16)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,int16_t,rechrom_builtin_mul_i32_usize_i16)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,uint16_t,rechrom_builtin_add_i32_usize_u16,rechrom_builtin_sub_i32_usize_u16)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,uint16_t,rechrom_builtin_mul_i32_usize_u16)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,int32_t,rechrom_builtin_add_i32_usize_i32,rechrom_builtin_sub_i32_usize_i32)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,int32_t,rechrom_builtin_mul_i32_usize_i32)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,uint32_t,rechrom_builtin_add_i32_usize_u32,rechrom_builtin_sub_i32_usize_u32)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,uint32_t,rechrom_builtin_mul_i32_usize_u32)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,int64_t,rechrom_builtin_add_i32_usize_i64,rechrom_builtin_sub_i32_usize_i64)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,int64_t,rechrom_builtin_mul_i32_usize_i64)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,uint64_t,rechrom_builtin_add_i32_usize_u64,rechrom_builtin_sub_i32_usize_u64)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,uint64_t,rechrom_builtin_mul_i32_usize_u64)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,intptr_t,rechrom_builtin_add_i32_usize_isize,rechrom_builtin_sub_i32_usize_isize)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,intptr_t,rechrom_builtin_mul_i32_usize_isize)
#endif
RECHROM_ADD_SUB(int32_t,uintptr_t,uintptr_t,rechrom_builtin_add_i32_usize_usize,rechrom_builtin_sub_i32_usize_usize)
#if !defined(__clang__)
RECHROM_MUL(int32_t,uintptr_t,uintptr_t,rechrom_builtin_mul_i32_usize_usize)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,int8_t,rechrom_builtin_add_u32_i8_i8,rechrom_builtin_sub_u32_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,int8_t,rechrom_builtin_mul_u32_i8_i8)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,uint8_t,rechrom_builtin_add_u32_i8_u8,rechrom_builtin_sub_u32_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,uint8_t,rechrom_builtin_mul_u32_i8_u8)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,int16_t,rechrom_builtin_add_u32_i8_i16,rechrom_builtin_sub_u32_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,int16_t,rechrom_builtin_mul_u32_i8_i16)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,uint16_t,rechrom_builtin_add_u32_i8_u16,rechrom_builtin_sub_u32_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,uint16_t,rechrom_builtin_mul_u32_i8_u16)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,int32_t,rechrom_builtin_add_u32_i8_i32,rechrom_builtin_sub_u32_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,int32_t,rechrom_builtin_mul_u32_i8_i32)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,uint32_t,rechrom_builtin_add_u32_i8_u32,rechrom_builtin_sub_u32_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,uint32_t,rechrom_builtin_mul_u32_i8_u32)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,int64_t,rechrom_builtin_add_u32_i8_i64,rechrom_builtin_sub_u32_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,int64_t,rechrom_builtin_mul_u32_i8_i64)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,uint64_t,rechrom_builtin_add_u32_i8_u64,rechrom_builtin_sub_u32_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,uint64_t,rechrom_builtin_mul_u32_i8_u64)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,intptr_t,rechrom_builtin_add_u32_i8_isize,rechrom_builtin_sub_u32_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,intptr_t,rechrom_builtin_mul_u32_i8_isize)
#endif
RECHROM_ADD_SUB(uint32_t,int8_t,uintptr_t,rechrom_builtin_add_u32_i8_usize,rechrom_builtin_sub_u32_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int8_t,uintptr_t,rechrom_builtin_mul_u32_i8_usize)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,int8_t,rechrom_builtin_add_u32_u8_i8,rechrom_builtin_sub_u32_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,int8_t,rechrom_builtin_mul_u32_u8_i8)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,uint8_t,rechrom_builtin_add_u32_u8_u8,rechrom_builtin_sub_u32_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,uint8_t,rechrom_builtin_mul_u32_u8_u8)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,int16_t,rechrom_builtin_add_u32_u8_i16,rechrom_builtin_sub_u32_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,int16_t,rechrom_builtin_mul_u32_u8_i16)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,uint16_t,rechrom_builtin_add_u32_u8_u16,rechrom_builtin_sub_u32_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,uint16_t,rechrom_builtin_mul_u32_u8_u16)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,int32_t,rechrom_builtin_add_u32_u8_i32,rechrom_builtin_sub_u32_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,int32_t,rechrom_builtin_mul_u32_u8_i32)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,uint32_t,rechrom_builtin_add_u32_u8_u32,rechrom_builtin_sub_u32_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,uint32_t,rechrom_builtin_mul_u32_u8_u32)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,int64_t,rechrom_builtin_add_u32_u8_i64,rechrom_builtin_sub_u32_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,int64_t,rechrom_builtin_mul_u32_u8_i64)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,uint64_t,rechrom_builtin_add_u32_u8_u64,rechrom_builtin_sub_u32_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,uint64_t,rechrom_builtin_mul_u32_u8_u64)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,intptr_t,rechrom_builtin_add_u32_u8_isize,rechrom_builtin_sub_u32_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,intptr_t,rechrom_builtin_mul_u32_u8_isize)
#endif
RECHROM_ADD_SUB(uint32_t,uint8_t,uintptr_t,rechrom_builtin_add_u32_u8_usize,rechrom_builtin_sub_u32_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint8_t,uintptr_t,rechrom_builtin_mul_u32_u8_usize)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,int8_t,rechrom_builtin_add_u32_i16_i8,rechrom_builtin_sub_u32_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,int8_t,rechrom_builtin_mul_u32_i16_i8)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,uint8_t,rechrom_builtin_add_u32_i16_u8,rechrom_builtin_sub_u32_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,uint8_t,rechrom_builtin_mul_u32_i16_u8)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,int16_t,rechrom_builtin_add_u32_i16_i16,rechrom_builtin_sub_u32_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,int16_t,rechrom_builtin_mul_u32_i16_i16)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,uint16_t,rechrom_builtin_add_u32_i16_u16,rechrom_builtin_sub_u32_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,uint16_t,rechrom_builtin_mul_u32_i16_u16)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,int32_t,rechrom_builtin_add_u32_i16_i32,rechrom_builtin_sub_u32_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,int32_t,rechrom_builtin_mul_u32_i16_i32)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,uint32_t,rechrom_builtin_add_u32_i16_u32,rechrom_builtin_sub_u32_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,uint32_t,rechrom_builtin_mul_u32_i16_u32)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,int64_t,rechrom_builtin_add_u32_i16_i64,rechrom_builtin_sub_u32_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,int64_t,rechrom_builtin_mul_u32_i16_i64)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,uint64_t,rechrom_builtin_add_u32_i16_u64,rechrom_builtin_sub_u32_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,uint64_t,rechrom_builtin_mul_u32_i16_u64)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,intptr_t,rechrom_builtin_add_u32_i16_isize,rechrom_builtin_sub_u32_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,intptr_t,rechrom_builtin_mul_u32_i16_isize)
#endif
RECHROM_ADD_SUB(uint32_t,int16_t,uintptr_t,rechrom_builtin_add_u32_i16_usize,rechrom_builtin_sub_u32_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int16_t,uintptr_t,rechrom_builtin_mul_u32_i16_usize)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,int8_t,rechrom_builtin_add_u32_u16_i8,rechrom_builtin_sub_u32_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,int8_t,rechrom_builtin_mul_u32_u16_i8)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,uint8_t,rechrom_builtin_add_u32_u16_u8,rechrom_builtin_sub_u32_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,uint8_t,rechrom_builtin_mul_u32_u16_u8)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,int16_t,rechrom_builtin_add_u32_u16_i16,rechrom_builtin_sub_u32_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,int16_t,rechrom_builtin_mul_u32_u16_i16)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,uint16_t,rechrom_builtin_add_u32_u16_u16,rechrom_builtin_sub_u32_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,uint16_t,rechrom_builtin_mul_u32_u16_u16)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,int32_t,rechrom_builtin_add_u32_u16_i32,rechrom_builtin_sub_u32_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,int32_t,rechrom_builtin_mul_u32_u16_i32)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,uint32_t,rechrom_builtin_add_u32_u16_u32,rechrom_builtin_sub_u32_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,uint32_t,rechrom_builtin_mul_u32_u16_u32)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,int64_t,rechrom_builtin_add_u32_u16_i64,rechrom_builtin_sub_u32_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,int64_t,rechrom_builtin_mul_u32_u16_i64)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,uint64_t,rechrom_builtin_add_u32_u16_u64,rechrom_builtin_sub_u32_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,uint64_t,rechrom_builtin_mul_u32_u16_u64)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,intptr_t,rechrom_builtin_add_u32_u16_isize,rechrom_builtin_sub_u32_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,intptr_t,rechrom_builtin_mul_u32_u16_isize)
#endif
RECHROM_ADD_SUB(uint32_t,uint16_t,uintptr_t,rechrom_builtin_add_u32_u16_usize,rechrom_builtin_sub_u32_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint16_t,uintptr_t,rechrom_builtin_mul_u32_u16_usize)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,int8_t,rechrom_builtin_add_u32_i32_i8,rechrom_builtin_sub_u32_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,int8_t,rechrom_builtin_mul_u32_i32_i8)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,uint8_t,rechrom_builtin_add_u32_i32_u8,rechrom_builtin_sub_u32_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,uint8_t,rechrom_builtin_mul_u32_i32_u8)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,int16_t,rechrom_builtin_add_u32_i32_i16,rechrom_builtin_sub_u32_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,int16_t,rechrom_builtin_mul_u32_i32_i16)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,uint16_t,rechrom_builtin_add_u32_i32_u16,rechrom_builtin_sub_u32_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,uint16_t,rechrom_builtin_mul_u32_i32_u16)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,int32_t,rechrom_builtin_add_u32_i32_i32,rechrom_builtin_sub_u32_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,int32_t,rechrom_builtin_mul_u32_i32_i32)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,uint32_t,rechrom_builtin_add_u32_i32_u32,rechrom_builtin_sub_u32_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,uint32_t,rechrom_builtin_mul_u32_i32_u32)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,int64_t,rechrom_builtin_add_u32_i32_i64,rechrom_builtin_sub_u32_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,int64_t,rechrom_builtin_mul_u32_i32_i64)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,uint64_t,rechrom_builtin_add_u32_i32_u64,rechrom_builtin_sub_u32_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,uint64_t,rechrom_builtin_mul_u32_i32_u64)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,intptr_t,rechrom_builtin_add_u32_i32_isize,rechrom_builtin_sub_u32_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,intptr_t,rechrom_builtin_mul_u32_i32_isize)
#endif
RECHROM_ADD_SUB(uint32_t,int32_t,uintptr_t,rechrom_builtin_add_u32_i32_usize,rechrom_builtin_sub_u32_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int32_t,uintptr_t,rechrom_builtin_mul_u32_i32_usize)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,int8_t,rechrom_builtin_add_u32_u32_i8,rechrom_builtin_sub_u32_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,int8_t,rechrom_builtin_mul_u32_u32_i8)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,uint8_t,rechrom_builtin_add_u32_u32_u8,rechrom_builtin_sub_u32_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,uint8_t,rechrom_builtin_mul_u32_u32_u8)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,int16_t,rechrom_builtin_add_u32_u32_i16,rechrom_builtin_sub_u32_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,int16_t,rechrom_builtin_mul_u32_u32_i16)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,uint16_t,rechrom_builtin_add_u32_u32_u16,rechrom_builtin_sub_u32_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,uint16_t,rechrom_builtin_mul_u32_u32_u16)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,int32_t,rechrom_builtin_add_u32_u32_i32,rechrom_builtin_sub_u32_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,int32_t,rechrom_builtin_mul_u32_u32_i32)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,uint32_t,rechrom_builtin_add_u32_u32_u32,rechrom_builtin_sub_u32_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,uint32_t,rechrom_builtin_mul_u32_u32_u32)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,int64_t,rechrom_builtin_add_u32_u32_i64,rechrom_builtin_sub_u32_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,int64_t,rechrom_builtin_mul_u32_u32_i64)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,uint64_t,rechrom_builtin_add_u32_u32_u64,rechrom_builtin_sub_u32_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,uint64_t,rechrom_builtin_mul_u32_u32_u64)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,intptr_t,rechrom_builtin_add_u32_u32_isize,rechrom_builtin_sub_u32_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,intptr_t,rechrom_builtin_mul_u32_u32_isize)
#endif
RECHROM_ADD_SUB(uint32_t,uint32_t,uintptr_t,rechrom_builtin_add_u32_u32_usize,rechrom_builtin_sub_u32_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint32_t,uintptr_t,rechrom_builtin_mul_u32_u32_usize)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,int8_t,rechrom_builtin_add_u32_i64_i8,rechrom_builtin_sub_u32_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,int8_t,rechrom_builtin_mul_u32_i64_i8)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,uint8_t,rechrom_builtin_add_u32_i64_u8,rechrom_builtin_sub_u32_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,uint8_t,rechrom_builtin_mul_u32_i64_u8)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,int16_t,rechrom_builtin_add_u32_i64_i16,rechrom_builtin_sub_u32_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,int16_t,rechrom_builtin_mul_u32_i64_i16)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,uint16_t,rechrom_builtin_add_u32_i64_u16,rechrom_builtin_sub_u32_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,uint16_t,rechrom_builtin_mul_u32_i64_u16)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,int32_t,rechrom_builtin_add_u32_i64_i32,rechrom_builtin_sub_u32_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,int32_t,rechrom_builtin_mul_u32_i64_i32)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,uint32_t,rechrom_builtin_add_u32_i64_u32,rechrom_builtin_sub_u32_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,uint32_t,rechrom_builtin_mul_u32_i64_u32)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,int64_t,rechrom_builtin_add_u32_i64_i64,rechrom_builtin_sub_u32_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,int64_t,rechrom_builtin_mul_u32_i64_i64)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,uint64_t,rechrom_builtin_add_u32_i64_u64,rechrom_builtin_sub_u32_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,uint64_t,rechrom_builtin_mul_u32_i64_u64)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,intptr_t,rechrom_builtin_add_u32_i64_isize,rechrom_builtin_sub_u32_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,intptr_t,rechrom_builtin_mul_u32_i64_isize)
#endif
RECHROM_ADD_SUB(uint32_t,int64_t,uintptr_t,rechrom_builtin_add_u32_i64_usize,rechrom_builtin_sub_u32_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,int64_t,uintptr_t,rechrom_builtin_mul_u32_i64_usize)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,int8_t,rechrom_builtin_add_u32_u64_i8,rechrom_builtin_sub_u32_u64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,int8_t,rechrom_builtin_mul_u32_u64_i8)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,uint8_t,rechrom_builtin_add_u32_u64_u8,rechrom_builtin_sub_u32_u64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,uint8_t,rechrom_builtin_mul_u32_u64_u8)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,int16_t,rechrom_builtin_add_u32_u64_i16,rechrom_builtin_sub_u32_u64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,int16_t,rechrom_builtin_mul_u32_u64_i16)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,uint16_t,rechrom_builtin_add_u32_u64_u16,rechrom_builtin_sub_u32_u64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,uint16_t,rechrom_builtin_mul_u32_u64_u16)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,int32_t,rechrom_builtin_add_u32_u64_i32,rechrom_builtin_sub_u32_u64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,int32_t,rechrom_builtin_mul_u32_u64_i32)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,uint32_t,rechrom_builtin_add_u32_u64_u32,rechrom_builtin_sub_u32_u64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,uint32_t,rechrom_builtin_mul_u32_u64_u32)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,int64_t,rechrom_builtin_add_u32_u64_i64,rechrom_builtin_sub_u32_u64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,int64_t,rechrom_builtin_mul_u32_u64_i64)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,uint64_t,rechrom_builtin_add_u32_u64_u64,rechrom_builtin_sub_u32_u64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,uint64_t,rechrom_builtin_mul_u32_u64_u64)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,intptr_t,rechrom_builtin_add_u32_u64_isize,rechrom_builtin_sub_u32_u64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,intptr_t,rechrom_builtin_mul_u32_u64_isize)
#endif
RECHROM_ADD_SUB(uint32_t,uint64_t,uintptr_t,rechrom_builtin_add_u32_u64_usize,rechrom_builtin_sub_u32_u64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uint64_t,uintptr_t,rechrom_builtin_mul_u32_u64_usize)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,int8_t,rechrom_builtin_add_u32_isize_i8,rechrom_builtin_sub_u32_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,int8_t,rechrom_builtin_mul_u32_isize_i8)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,uint8_t,rechrom_builtin_add_u32_isize_u8,rechrom_builtin_sub_u32_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,uint8_t,rechrom_builtin_mul_u32_isize_u8)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,int16_t,rechrom_builtin_add_u32_isize_i16,rechrom_builtin_sub_u32_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,int16_t,rechrom_builtin_mul_u32_isize_i16)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,uint16_t,rechrom_builtin_add_u32_isize_u16,rechrom_builtin_sub_u32_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,uint16_t,rechrom_builtin_mul_u32_isize_u16)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,int32_t,rechrom_builtin_add_u32_isize_i32,rechrom_builtin_sub_u32_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,int32_t,rechrom_builtin_mul_u32_isize_i32)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,uint32_t,rechrom_builtin_add_u32_isize_u32,rechrom_builtin_sub_u32_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,uint32_t,rechrom_builtin_mul_u32_isize_u32)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,int64_t,rechrom_builtin_add_u32_isize_i64,rechrom_builtin_sub_u32_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,int64_t,rechrom_builtin_mul_u32_isize_i64)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,uint64_t,rechrom_builtin_add_u32_isize_u64,rechrom_builtin_sub_u32_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,uint64_t,rechrom_builtin_mul_u32_isize_u64)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,intptr_t,rechrom_builtin_add_u32_isize_isize,rechrom_builtin_sub_u32_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,intptr_t,rechrom_builtin_mul_u32_isize_isize)
#endif
RECHROM_ADD_SUB(uint32_t,intptr_t,uintptr_t,rechrom_builtin_add_u32_isize_usize,rechrom_builtin_sub_u32_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,intptr_t,uintptr_t,rechrom_builtin_mul_u32_isize_usize)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,int8_t,rechrom_builtin_add_u32_usize_i8,rechrom_builtin_sub_u32_usize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,int8_t,rechrom_builtin_mul_u32_usize_i8)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,uint8_t,rechrom_builtin_add_u32_usize_u8,rechrom_builtin_sub_u32_usize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,uint8_t,rechrom_builtin_mul_u32_usize_u8)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,int16_t,rechrom_builtin_add_u32_usize_i16,rechrom_builtin_sub_u32_usize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,int16_t,rechrom_builtin_mul_u32_usize_i16)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,uint16_t,rechrom_builtin_add_u32_usize_u16,rechrom_builtin_sub_u32_usize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,uint16_t,rechrom_builtin_mul_u32_usize_u16)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,int32_t,rechrom_builtin_add_u32_usize_i32,rechrom_builtin_sub_u32_usize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,int32_t,rechrom_builtin_mul_u32_usize_i32)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,uint32_t,rechrom_builtin_add_u32_usize_u32,rechrom_builtin_sub_u32_usize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,uint32_t,rechrom_builtin_mul_u32_usize_u32)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,int64_t,rechrom_builtin_add_u32_usize_i64,rechrom_builtin_sub_u32_usize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,int64_t,rechrom_builtin_mul_u32_usize_i64)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,uint64_t,rechrom_builtin_add_u32_usize_u64,rechrom_builtin_sub_u32_usize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,uint64_t,rechrom_builtin_mul_u32_usize_u64)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,intptr_t,rechrom_builtin_add_u32_usize_isize,rechrom_builtin_sub_u32_usize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,intptr_t,rechrom_builtin_mul_u32_usize_isize)
#endif
RECHROM_ADD_SUB(uint32_t,uintptr_t,uintptr_t,rechrom_builtin_add_u32_usize_usize,rechrom_builtin_sub_u32_usize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint32_t,uintptr_t,uintptr_t,rechrom_builtin_mul_u32_usize_usize)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,int8_t,rechrom_builtin_add_i64_i8_i8,rechrom_builtin_sub_i64_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,int8_t,rechrom_builtin_mul_i64_i8_i8)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,uint8_t,rechrom_builtin_add_i64_i8_u8,rechrom_builtin_sub_i64_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,uint8_t,rechrom_builtin_mul_i64_i8_u8)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,int16_t,rechrom_builtin_add_i64_i8_i16,rechrom_builtin_sub_i64_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,int16_t,rechrom_builtin_mul_i64_i8_i16)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,uint16_t,rechrom_builtin_add_i64_i8_u16,rechrom_builtin_sub_i64_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,uint16_t,rechrom_builtin_mul_i64_i8_u16)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,int32_t,rechrom_builtin_add_i64_i8_i32,rechrom_builtin_sub_i64_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,int32_t,rechrom_builtin_mul_i64_i8_i32)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,uint32_t,rechrom_builtin_add_i64_i8_u32,rechrom_builtin_sub_i64_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,uint32_t,rechrom_builtin_mul_i64_i8_u32)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,int64_t,rechrom_builtin_add_i64_i8_i64,rechrom_builtin_sub_i64_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,int64_t,rechrom_builtin_mul_i64_i8_i64)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,uint64_t,rechrom_builtin_add_i64_i8_u64,rechrom_builtin_sub_i64_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,uint64_t,rechrom_builtin_mul_i64_i8_u64)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,intptr_t,rechrom_builtin_add_i64_i8_isize,rechrom_builtin_sub_i64_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,intptr_t,rechrom_builtin_mul_i64_i8_isize)
#endif
RECHROM_ADD_SUB(int64_t,int8_t,uintptr_t,rechrom_builtin_add_i64_i8_usize,rechrom_builtin_sub_i64_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int8_t,uintptr_t,rechrom_builtin_mul_i64_i8_usize)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,int8_t,rechrom_builtin_add_i64_u8_i8,rechrom_builtin_sub_i64_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,int8_t,rechrom_builtin_mul_i64_u8_i8)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,uint8_t,rechrom_builtin_add_i64_u8_u8,rechrom_builtin_sub_i64_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,uint8_t,rechrom_builtin_mul_i64_u8_u8)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,int16_t,rechrom_builtin_add_i64_u8_i16,rechrom_builtin_sub_i64_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,int16_t,rechrom_builtin_mul_i64_u8_i16)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,uint16_t,rechrom_builtin_add_i64_u8_u16,rechrom_builtin_sub_i64_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,uint16_t,rechrom_builtin_mul_i64_u8_u16)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,int32_t,rechrom_builtin_add_i64_u8_i32,rechrom_builtin_sub_i64_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,int32_t,rechrom_builtin_mul_i64_u8_i32)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,uint32_t,rechrom_builtin_add_i64_u8_u32,rechrom_builtin_sub_i64_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,uint32_t,rechrom_builtin_mul_i64_u8_u32)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,int64_t,rechrom_builtin_add_i64_u8_i64,rechrom_builtin_sub_i64_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,int64_t,rechrom_builtin_mul_i64_u8_i64)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,uint64_t,rechrom_builtin_add_i64_u8_u64,rechrom_builtin_sub_i64_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,uint64_t,rechrom_builtin_mul_i64_u8_u64)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,intptr_t,rechrom_builtin_add_i64_u8_isize,rechrom_builtin_sub_i64_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,intptr_t,rechrom_builtin_mul_i64_u8_isize)
#endif
RECHROM_ADD_SUB(int64_t,uint8_t,uintptr_t,rechrom_builtin_add_i64_u8_usize,rechrom_builtin_sub_i64_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint8_t,uintptr_t,rechrom_builtin_mul_i64_u8_usize)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,int8_t,rechrom_builtin_add_i64_i16_i8,rechrom_builtin_sub_i64_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,int8_t,rechrom_builtin_mul_i64_i16_i8)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,uint8_t,rechrom_builtin_add_i64_i16_u8,rechrom_builtin_sub_i64_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,uint8_t,rechrom_builtin_mul_i64_i16_u8)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,int16_t,rechrom_builtin_add_i64_i16_i16,rechrom_builtin_sub_i64_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,int16_t,rechrom_builtin_mul_i64_i16_i16)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,uint16_t,rechrom_builtin_add_i64_i16_u16,rechrom_builtin_sub_i64_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,uint16_t,rechrom_builtin_mul_i64_i16_u16)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,int32_t,rechrom_builtin_add_i64_i16_i32,rechrom_builtin_sub_i64_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,int32_t,rechrom_builtin_mul_i64_i16_i32)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,uint32_t,rechrom_builtin_add_i64_i16_u32,rechrom_builtin_sub_i64_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,uint32_t,rechrom_builtin_mul_i64_i16_u32)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,int64_t,rechrom_builtin_add_i64_i16_i64,rechrom_builtin_sub_i64_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,int64_t,rechrom_builtin_mul_i64_i16_i64)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,uint64_t,rechrom_builtin_add_i64_i16_u64,rechrom_builtin_sub_i64_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,uint64_t,rechrom_builtin_mul_i64_i16_u64)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,intptr_t,rechrom_builtin_add_i64_i16_isize,rechrom_builtin_sub_i64_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,intptr_t,rechrom_builtin_mul_i64_i16_isize)
#endif
RECHROM_ADD_SUB(int64_t,int16_t,uintptr_t,rechrom_builtin_add_i64_i16_usize,rechrom_builtin_sub_i64_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int16_t,uintptr_t,rechrom_builtin_mul_i64_i16_usize)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,int8_t,rechrom_builtin_add_i64_u16_i8,rechrom_builtin_sub_i64_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,int8_t,rechrom_builtin_mul_i64_u16_i8)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,uint8_t,rechrom_builtin_add_i64_u16_u8,rechrom_builtin_sub_i64_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,uint8_t,rechrom_builtin_mul_i64_u16_u8)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,int16_t,rechrom_builtin_add_i64_u16_i16,rechrom_builtin_sub_i64_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,int16_t,rechrom_builtin_mul_i64_u16_i16)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,uint16_t,rechrom_builtin_add_i64_u16_u16,rechrom_builtin_sub_i64_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,uint16_t,rechrom_builtin_mul_i64_u16_u16)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,int32_t,rechrom_builtin_add_i64_u16_i32,rechrom_builtin_sub_i64_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,int32_t,rechrom_builtin_mul_i64_u16_i32)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,uint32_t,rechrom_builtin_add_i64_u16_u32,rechrom_builtin_sub_i64_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,uint32_t,rechrom_builtin_mul_i64_u16_u32)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,int64_t,rechrom_builtin_add_i64_u16_i64,rechrom_builtin_sub_i64_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,int64_t,rechrom_builtin_mul_i64_u16_i64)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,uint64_t,rechrom_builtin_add_i64_u16_u64,rechrom_builtin_sub_i64_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,uint64_t,rechrom_builtin_mul_i64_u16_u64)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,intptr_t,rechrom_builtin_add_i64_u16_isize,rechrom_builtin_sub_i64_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,intptr_t,rechrom_builtin_mul_i64_u16_isize)
#endif
RECHROM_ADD_SUB(int64_t,uint16_t,uintptr_t,rechrom_builtin_add_i64_u16_usize,rechrom_builtin_sub_i64_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint16_t,uintptr_t,rechrom_builtin_mul_i64_u16_usize)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,int8_t,rechrom_builtin_add_i64_i32_i8,rechrom_builtin_sub_i64_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,int8_t,rechrom_builtin_mul_i64_i32_i8)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,uint8_t,rechrom_builtin_add_i64_i32_u8,rechrom_builtin_sub_i64_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,uint8_t,rechrom_builtin_mul_i64_i32_u8)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,int16_t,rechrom_builtin_add_i64_i32_i16,rechrom_builtin_sub_i64_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,int16_t,rechrom_builtin_mul_i64_i32_i16)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,uint16_t,rechrom_builtin_add_i64_i32_u16,rechrom_builtin_sub_i64_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,uint16_t,rechrom_builtin_mul_i64_i32_u16)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,int32_t,rechrom_builtin_add_i64_i32_i32,rechrom_builtin_sub_i64_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,int32_t,rechrom_builtin_mul_i64_i32_i32)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,uint32_t,rechrom_builtin_add_i64_i32_u32,rechrom_builtin_sub_i64_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,uint32_t,rechrom_builtin_mul_i64_i32_u32)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,int64_t,rechrom_builtin_add_i64_i32_i64,rechrom_builtin_sub_i64_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,int64_t,rechrom_builtin_mul_i64_i32_i64)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,uint64_t,rechrom_builtin_add_i64_i32_u64,rechrom_builtin_sub_i64_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,uint64_t,rechrom_builtin_mul_i64_i32_u64)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,intptr_t,rechrom_builtin_add_i64_i32_isize,rechrom_builtin_sub_i64_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,intptr_t,rechrom_builtin_mul_i64_i32_isize)
#endif
RECHROM_ADD_SUB(int64_t,int32_t,uintptr_t,rechrom_builtin_add_i64_i32_usize,rechrom_builtin_sub_i64_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int32_t,uintptr_t,rechrom_builtin_mul_i64_i32_usize)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,int8_t,rechrom_builtin_add_i64_u32_i8,rechrom_builtin_sub_i64_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,int8_t,rechrom_builtin_mul_i64_u32_i8)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,uint8_t,rechrom_builtin_add_i64_u32_u8,rechrom_builtin_sub_i64_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,uint8_t,rechrom_builtin_mul_i64_u32_u8)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,int16_t,rechrom_builtin_add_i64_u32_i16,rechrom_builtin_sub_i64_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,int16_t,rechrom_builtin_mul_i64_u32_i16)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,uint16_t,rechrom_builtin_add_i64_u32_u16,rechrom_builtin_sub_i64_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,uint16_t,rechrom_builtin_mul_i64_u32_u16)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,int32_t,rechrom_builtin_add_i64_u32_i32,rechrom_builtin_sub_i64_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,int32_t,rechrom_builtin_mul_i64_u32_i32)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,uint32_t,rechrom_builtin_add_i64_u32_u32,rechrom_builtin_sub_i64_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,uint32_t,rechrom_builtin_mul_i64_u32_u32)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,int64_t,rechrom_builtin_add_i64_u32_i64,rechrom_builtin_sub_i64_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,int64_t,rechrom_builtin_mul_i64_u32_i64)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,uint64_t,rechrom_builtin_add_i64_u32_u64,rechrom_builtin_sub_i64_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,uint64_t,rechrom_builtin_mul_i64_u32_u64)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,intptr_t,rechrom_builtin_add_i64_u32_isize,rechrom_builtin_sub_i64_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,intptr_t,rechrom_builtin_mul_i64_u32_isize)
#endif
RECHROM_ADD_SUB(int64_t,uint32_t,uintptr_t,rechrom_builtin_add_i64_u32_usize,rechrom_builtin_sub_i64_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,uint32_t,uintptr_t,rechrom_builtin_mul_i64_u32_usize)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,int8_t,rechrom_builtin_add_i64_i64_i8,rechrom_builtin_sub_i64_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,int8_t,rechrom_builtin_mul_i64_i64_i8)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,uint8_t,rechrom_builtin_add_i64_i64_u8,rechrom_builtin_sub_i64_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,uint8_t,rechrom_builtin_mul_i64_i64_u8)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,int16_t,rechrom_builtin_add_i64_i64_i16,rechrom_builtin_sub_i64_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,int16_t,rechrom_builtin_mul_i64_i64_i16)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,uint16_t,rechrom_builtin_add_i64_i64_u16,rechrom_builtin_sub_i64_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,uint16_t,rechrom_builtin_mul_i64_i64_u16)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,int32_t,rechrom_builtin_add_i64_i64_i32,rechrom_builtin_sub_i64_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,int32_t,rechrom_builtin_mul_i64_i64_i32)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,uint32_t,rechrom_builtin_add_i64_i64_u32,rechrom_builtin_sub_i64_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,uint32_t,rechrom_builtin_mul_i64_i64_u32)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,int64_t,rechrom_builtin_add_i64_i64_i64,rechrom_builtin_sub_i64_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,int64_t,rechrom_builtin_mul_i64_i64_i64)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,uint64_t,rechrom_builtin_add_i64_i64_u64,rechrom_builtin_sub_i64_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,uint64_t,rechrom_builtin_mul_i64_i64_u64)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,intptr_t,rechrom_builtin_add_i64_i64_isize,rechrom_builtin_sub_i64_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,intptr_t,rechrom_builtin_mul_i64_i64_isize)
#endif
RECHROM_ADD_SUB(int64_t,int64_t,uintptr_t,rechrom_builtin_add_i64_i64_usize,rechrom_builtin_sub_i64_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,int64_t,uintptr_t,rechrom_builtin_mul_i64_i64_usize)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,int8_t,rechrom_builtin_add_i64_u64_i8,rechrom_builtin_sub_i64_u64_i8)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,int8_t,rechrom_builtin_mul_i64_u64_i8)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,uint8_t,rechrom_builtin_add_i64_u64_u8,rechrom_builtin_sub_i64_u64_u8)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,uint8_t,rechrom_builtin_mul_i64_u64_u8)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,int16_t,rechrom_builtin_add_i64_u64_i16,rechrom_builtin_sub_i64_u64_i16)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,int16_t,rechrom_builtin_mul_i64_u64_i16)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,uint16_t,rechrom_builtin_add_i64_u64_u16,rechrom_builtin_sub_i64_u64_u16)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,uint16_t,rechrom_builtin_mul_i64_u64_u16)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,int32_t,rechrom_builtin_add_i64_u64_i32,rechrom_builtin_sub_i64_u64_i32)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,int32_t,rechrom_builtin_mul_i64_u64_i32)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,uint32_t,rechrom_builtin_add_i64_u64_u32,rechrom_builtin_sub_i64_u64_u32)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,uint32_t,rechrom_builtin_mul_i64_u64_u32)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,int64_t,rechrom_builtin_add_i64_u64_i64,rechrom_builtin_sub_i64_u64_i64)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,int64_t,rechrom_builtin_mul_i64_u64_i64)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,uint64_t,rechrom_builtin_add_i64_u64_u64,rechrom_builtin_sub_i64_u64_u64)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,uint64_t,rechrom_builtin_mul_i64_u64_u64)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,intptr_t,rechrom_builtin_add_i64_u64_isize,rechrom_builtin_sub_i64_u64_isize)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,intptr_t,rechrom_builtin_mul_i64_u64_isize)
#endif
RECHROM_ADD_SUB(int64_t,uint64_t,uintptr_t,rechrom_builtin_add_i64_u64_usize,rechrom_builtin_sub_i64_u64_usize)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uint64_t,uintptr_t,rechrom_builtin_mul_i64_u64_usize)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,int8_t,rechrom_builtin_add_i64_isize_i8,rechrom_builtin_sub_i64_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,int8_t,rechrom_builtin_mul_i64_isize_i8)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,uint8_t,rechrom_builtin_add_i64_isize_u8,rechrom_builtin_sub_i64_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,uint8_t,rechrom_builtin_mul_i64_isize_u8)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,int16_t,rechrom_builtin_add_i64_isize_i16,rechrom_builtin_sub_i64_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,int16_t,rechrom_builtin_mul_i64_isize_i16)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,uint16_t,rechrom_builtin_add_i64_isize_u16,rechrom_builtin_sub_i64_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,uint16_t,rechrom_builtin_mul_i64_isize_u16)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,int32_t,rechrom_builtin_add_i64_isize_i32,rechrom_builtin_sub_i64_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,int32_t,rechrom_builtin_mul_i64_isize_i32)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,uint32_t,rechrom_builtin_add_i64_isize_u32,rechrom_builtin_sub_i64_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,uint32_t,rechrom_builtin_mul_i64_isize_u32)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,int64_t,rechrom_builtin_add_i64_isize_i64,rechrom_builtin_sub_i64_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,int64_t,rechrom_builtin_mul_i64_isize_i64)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,uint64_t,rechrom_builtin_add_i64_isize_u64,rechrom_builtin_sub_i64_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,uint64_t,rechrom_builtin_mul_i64_isize_u64)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,intptr_t,rechrom_builtin_add_i64_isize_isize,rechrom_builtin_sub_i64_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,intptr_t,rechrom_builtin_mul_i64_isize_isize)
#endif
RECHROM_ADD_SUB(int64_t,intptr_t,uintptr_t,rechrom_builtin_add_i64_isize_usize,rechrom_builtin_sub_i64_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(int64_t,intptr_t,uintptr_t,rechrom_builtin_mul_i64_isize_usize)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,int8_t,rechrom_builtin_add_i64_usize_i8,rechrom_builtin_sub_i64_usize_i8)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,int8_t,rechrom_builtin_mul_i64_usize_i8)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,uint8_t,rechrom_builtin_add_i64_usize_u8,rechrom_builtin_sub_i64_usize_u8)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,uint8_t,rechrom_builtin_mul_i64_usize_u8)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,int16_t,rechrom_builtin_add_i64_usize_i16,rechrom_builtin_sub_i64_usize_i16)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,int16_t,rechrom_builtin_mul_i64_usize_i16)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,uint16_t,rechrom_builtin_add_i64_usize_u16,rechrom_builtin_sub_i64_usize_u16)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,uint16_t,rechrom_builtin_mul_i64_usize_u16)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,int32_t,rechrom_builtin_add_i64_usize_i32,rechrom_builtin_sub_i64_usize_i32)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,int32_t,rechrom_builtin_mul_i64_usize_i32)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,uint32_t,rechrom_builtin_add_i64_usize_u32,rechrom_builtin_sub_i64_usize_u32)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,uint32_t,rechrom_builtin_mul_i64_usize_u32)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,int64_t,rechrom_builtin_add_i64_usize_i64,rechrom_builtin_sub_i64_usize_i64)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,int64_t,rechrom_builtin_mul_i64_usize_i64)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,uint64_t,rechrom_builtin_add_i64_usize_u64,rechrom_builtin_sub_i64_usize_u64)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,uint64_t,rechrom_builtin_mul_i64_usize_u64)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,intptr_t,rechrom_builtin_add_i64_usize_isize,rechrom_builtin_sub_i64_usize_isize)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,intptr_t,rechrom_builtin_mul_i64_usize_isize)
#endif
RECHROM_ADD_SUB(int64_t,uintptr_t,uintptr_t,rechrom_builtin_add_i64_usize_usize,rechrom_builtin_sub_i64_usize_usize)
#if !defined(__clang__)
RECHROM_MUL(int64_t,uintptr_t,uintptr_t,rechrom_builtin_mul_i64_usize_usize)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,int8_t,rechrom_builtin_add_u64_i8_i8,rechrom_builtin_sub_u64_i8_i8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,int8_t,rechrom_builtin_mul_u64_i8_i8)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,uint8_t,rechrom_builtin_add_u64_i8_u8,rechrom_builtin_sub_u64_i8_u8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,uint8_t,rechrom_builtin_mul_u64_i8_u8)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,int16_t,rechrom_builtin_add_u64_i8_i16,rechrom_builtin_sub_u64_i8_i16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,int16_t,rechrom_builtin_mul_u64_i8_i16)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,uint16_t,rechrom_builtin_add_u64_i8_u16,rechrom_builtin_sub_u64_i8_u16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,uint16_t,rechrom_builtin_mul_u64_i8_u16)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,int32_t,rechrom_builtin_add_u64_i8_i32,rechrom_builtin_sub_u64_i8_i32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,int32_t,rechrom_builtin_mul_u64_i8_i32)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,uint32_t,rechrom_builtin_add_u64_i8_u32,rechrom_builtin_sub_u64_i8_u32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,uint32_t,rechrom_builtin_mul_u64_i8_u32)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,int64_t,rechrom_builtin_add_u64_i8_i64,rechrom_builtin_sub_u64_i8_i64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,int64_t,rechrom_builtin_mul_u64_i8_i64)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,uint64_t,rechrom_builtin_add_u64_i8_u64,rechrom_builtin_sub_u64_i8_u64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,uint64_t,rechrom_builtin_mul_u64_i8_u64)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,intptr_t,rechrom_builtin_add_u64_i8_isize,rechrom_builtin_sub_u64_i8_isize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,intptr_t,rechrom_builtin_mul_u64_i8_isize)
#endif
RECHROM_ADD_SUB(uint64_t,int8_t,uintptr_t,rechrom_builtin_add_u64_i8_usize,rechrom_builtin_sub_u64_i8_usize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int8_t,uintptr_t,rechrom_builtin_mul_u64_i8_usize)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,int8_t,rechrom_builtin_add_u64_u8_i8,rechrom_builtin_sub_u64_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,int8_t,rechrom_builtin_mul_u64_u8_i8)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,uint8_t,rechrom_builtin_add_u64_u8_u8,rechrom_builtin_sub_u64_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,uint8_t,rechrom_builtin_mul_u64_u8_u8)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,int16_t,rechrom_builtin_add_u64_u8_i16,rechrom_builtin_sub_u64_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,int16_t,rechrom_builtin_mul_u64_u8_i16)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,uint16_t,rechrom_builtin_add_u64_u8_u16,rechrom_builtin_sub_u64_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,uint16_t,rechrom_builtin_mul_u64_u8_u16)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,int32_t,rechrom_builtin_add_u64_u8_i32,rechrom_builtin_sub_u64_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,int32_t,rechrom_builtin_mul_u64_u8_i32)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,uint32_t,rechrom_builtin_add_u64_u8_u32,rechrom_builtin_sub_u64_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,uint32_t,rechrom_builtin_mul_u64_u8_u32)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,int64_t,rechrom_builtin_add_u64_u8_i64,rechrom_builtin_sub_u64_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,int64_t,rechrom_builtin_mul_u64_u8_i64)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,uint64_t,rechrom_builtin_add_u64_u8_u64,rechrom_builtin_sub_u64_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,uint64_t,rechrom_builtin_mul_u64_u8_u64)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,intptr_t,rechrom_builtin_add_u64_u8_isize,rechrom_builtin_sub_u64_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,intptr_t,rechrom_builtin_mul_u64_u8_isize)
#endif
RECHROM_ADD_SUB(uint64_t,uint8_t,uintptr_t,rechrom_builtin_add_u64_u8_usize,rechrom_builtin_sub_u64_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint8_t,uintptr_t,rechrom_builtin_mul_u64_u8_usize)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,int8_t,rechrom_builtin_add_u64_i16_i8,rechrom_builtin_sub_u64_i16_i8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,int8_t,rechrom_builtin_mul_u64_i16_i8)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,uint8_t,rechrom_builtin_add_u64_i16_u8,rechrom_builtin_sub_u64_i16_u8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,uint8_t,rechrom_builtin_mul_u64_i16_u8)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,int16_t,rechrom_builtin_add_u64_i16_i16,rechrom_builtin_sub_u64_i16_i16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,int16_t,rechrom_builtin_mul_u64_i16_i16)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,uint16_t,rechrom_builtin_add_u64_i16_u16,rechrom_builtin_sub_u64_i16_u16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,uint16_t,rechrom_builtin_mul_u64_i16_u16)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,int32_t,rechrom_builtin_add_u64_i16_i32,rechrom_builtin_sub_u64_i16_i32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,int32_t,rechrom_builtin_mul_u64_i16_i32)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,uint32_t,rechrom_builtin_add_u64_i16_u32,rechrom_builtin_sub_u64_i16_u32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,uint32_t,rechrom_builtin_mul_u64_i16_u32)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,int64_t,rechrom_builtin_add_u64_i16_i64,rechrom_builtin_sub_u64_i16_i64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,int64_t,rechrom_builtin_mul_u64_i16_i64)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,uint64_t,rechrom_builtin_add_u64_i16_u64,rechrom_builtin_sub_u64_i16_u64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,uint64_t,rechrom_builtin_mul_u64_i16_u64)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,intptr_t,rechrom_builtin_add_u64_i16_isize,rechrom_builtin_sub_u64_i16_isize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,intptr_t,rechrom_builtin_mul_u64_i16_isize)
#endif
RECHROM_ADD_SUB(uint64_t,int16_t,uintptr_t,rechrom_builtin_add_u64_i16_usize,rechrom_builtin_sub_u64_i16_usize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int16_t,uintptr_t,rechrom_builtin_mul_u64_i16_usize)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,int8_t,rechrom_builtin_add_u64_u16_i8,rechrom_builtin_sub_u64_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,int8_t,rechrom_builtin_mul_u64_u16_i8)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,uint8_t,rechrom_builtin_add_u64_u16_u8,rechrom_builtin_sub_u64_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,uint8_t,rechrom_builtin_mul_u64_u16_u8)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,int16_t,rechrom_builtin_add_u64_u16_i16,rechrom_builtin_sub_u64_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,int16_t,rechrom_builtin_mul_u64_u16_i16)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,uint16_t,rechrom_builtin_add_u64_u16_u16,rechrom_builtin_sub_u64_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,uint16_t,rechrom_builtin_mul_u64_u16_u16)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,int32_t,rechrom_builtin_add_u64_u16_i32,rechrom_builtin_sub_u64_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,int32_t,rechrom_builtin_mul_u64_u16_i32)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,uint32_t,rechrom_builtin_add_u64_u16_u32,rechrom_builtin_sub_u64_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,uint32_t,rechrom_builtin_mul_u64_u16_u32)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,int64_t,rechrom_builtin_add_u64_u16_i64,rechrom_builtin_sub_u64_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,int64_t,rechrom_builtin_mul_u64_u16_i64)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,uint64_t,rechrom_builtin_add_u64_u16_u64,rechrom_builtin_sub_u64_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,uint64_t,rechrom_builtin_mul_u64_u16_u64)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,intptr_t,rechrom_builtin_add_u64_u16_isize,rechrom_builtin_sub_u64_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,intptr_t,rechrom_builtin_mul_u64_u16_isize)
#endif
RECHROM_ADD_SUB(uint64_t,uint16_t,uintptr_t,rechrom_builtin_add_u64_u16_usize,rechrom_builtin_sub_u64_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint16_t,uintptr_t,rechrom_builtin_mul_u64_u16_usize)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,int8_t,rechrom_builtin_add_u64_i32_i8,rechrom_builtin_sub_u64_i32_i8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,int8_t,rechrom_builtin_mul_u64_i32_i8)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,uint8_t,rechrom_builtin_add_u64_i32_u8,rechrom_builtin_sub_u64_i32_u8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,uint8_t,rechrom_builtin_mul_u64_i32_u8)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,int16_t,rechrom_builtin_add_u64_i32_i16,rechrom_builtin_sub_u64_i32_i16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,int16_t,rechrom_builtin_mul_u64_i32_i16)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,uint16_t,rechrom_builtin_add_u64_i32_u16,rechrom_builtin_sub_u64_i32_u16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,uint16_t,rechrom_builtin_mul_u64_i32_u16)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,int32_t,rechrom_builtin_add_u64_i32_i32,rechrom_builtin_sub_u64_i32_i32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,int32_t,rechrom_builtin_mul_u64_i32_i32)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,uint32_t,rechrom_builtin_add_u64_i32_u32,rechrom_builtin_sub_u64_i32_u32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,uint32_t,rechrom_builtin_mul_u64_i32_u32)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,int64_t,rechrom_builtin_add_u64_i32_i64,rechrom_builtin_sub_u64_i32_i64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,int64_t,rechrom_builtin_mul_u64_i32_i64)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,uint64_t,rechrom_builtin_add_u64_i32_u64,rechrom_builtin_sub_u64_i32_u64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,uint64_t,rechrom_builtin_mul_u64_i32_u64)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,intptr_t,rechrom_builtin_add_u64_i32_isize,rechrom_builtin_sub_u64_i32_isize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,intptr_t,rechrom_builtin_mul_u64_i32_isize)
#endif
RECHROM_ADD_SUB(uint64_t,int32_t,uintptr_t,rechrom_builtin_add_u64_i32_usize,rechrom_builtin_sub_u64_i32_usize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int32_t,uintptr_t,rechrom_builtin_mul_u64_i32_usize)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,int8_t,rechrom_builtin_add_u64_u32_i8,rechrom_builtin_sub_u64_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,int8_t,rechrom_builtin_mul_u64_u32_i8)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,uint8_t,rechrom_builtin_add_u64_u32_u8,rechrom_builtin_sub_u64_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,uint8_t,rechrom_builtin_mul_u64_u32_u8)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,int16_t,rechrom_builtin_add_u64_u32_i16,rechrom_builtin_sub_u64_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,int16_t,rechrom_builtin_mul_u64_u32_i16)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,uint16_t,rechrom_builtin_add_u64_u32_u16,rechrom_builtin_sub_u64_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,uint16_t,rechrom_builtin_mul_u64_u32_u16)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,int32_t,rechrom_builtin_add_u64_u32_i32,rechrom_builtin_sub_u64_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,int32_t,rechrom_builtin_mul_u64_u32_i32)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,uint32_t,rechrom_builtin_add_u64_u32_u32,rechrom_builtin_sub_u64_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,uint32_t,rechrom_builtin_mul_u64_u32_u32)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,int64_t,rechrom_builtin_add_u64_u32_i64,rechrom_builtin_sub_u64_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,int64_t,rechrom_builtin_mul_u64_u32_i64)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,uint64_t,rechrom_builtin_add_u64_u32_u64,rechrom_builtin_sub_u64_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,uint64_t,rechrom_builtin_mul_u64_u32_u64)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,intptr_t,rechrom_builtin_add_u64_u32_isize,rechrom_builtin_sub_u64_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,intptr_t,rechrom_builtin_mul_u64_u32_isize)
#endif
RECHROM_ADD_SUB(uint64_t,uint32_t,uintptr_t,rechrom_builtin_add_u64_u32_usize,rechrom_builtin_sub_u64_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint32_t,uintptr_t,rechrom_builtin_mul_u64_u32_usize)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,int8_t,rechrom_builtin_add_u64_i64_i8,rechrom_builtin_sub_u64_i64_i8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,int8_t,rechrom_builtin_mul_u64_i64_i8)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,uint8_t,rechrom_builtin_add_u64_i64_u8,rechrom_builtin_sub_u64_i64_u8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,uint8_t,rechrom_builtin_mul_u64_i64_u8)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,int16_t,rechrom_builtin_add_u64_i64_i16,rechrom_builtin_sub_u64_i64_i16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,int16_t,rechrom_builtin_mul_u64_i64_i16)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,uint16_t,rechrom_builtin_add_u64_i64_u16,rechrom_builtin_sub_u64_i64_u16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,uint16_t,rechrom_builtin_mul_u64_i64_u16)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,int32_t,rechrom_builtin_add_u64_i64_i32,rechrom_builtin_sub_u64_i64_i32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,int32_t,rechrom_builtin_mul_u64_i64_i32)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,uint32_t,rechrom_builtin_add_u64_i64_u32,rechrom_builtin_sub_u64_i64_u32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,uint32_t,rechrom_builtin_mul_u64_i64_u32)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,int64_t,rechrom_builtin_add_u64_i64_i64,rechrom_builtin_sub_u64_i64_i64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,int64_t,rechrom_builtin_mul_u64_i64_i64)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,uint64_t,rechrom_builtin_add_u64_i64_u64,rechrom_builtin_sub_u64_i64_u64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,uint64_t,rechrom_builtin_mul_u64_i64_u64)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,intptr_t,rechrom_builtin_add_u64_i64_isize,rechrom_builtin_sub_u64_i64_isize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,intptr_t,rechrom_builtin_mul_u64_i64_isize)
#endif
RECHROM_ADD_SUB(uint64_t,int64_t,uintptr_t,rechrom_builtin_add_u64_i64_usize,rechrom_builtin_sub_u64_i64_usize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,int64_t,uintptr_t,rechrom_builtin_mul_u64_i64_usize)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,int8_t,rechrom_builtin_add_u64_u64_i8,rechrom_builtin_sub_u64_u64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,int8_t,rechrom_builtin_mul_u64_u64_i8)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,uint8_t,rechrom_builtin_add_u64_u64_u8,rechrom_builtin_sub_u64_u64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,uint8_t,rechrom_builtin_mul_u64_u64_u8)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,int16_t,rechrom_builtin_add_u64_u64_i16,rechrom_builtin_sub_u64_u64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,int16_t,rechrom_builtin_mul_u64_u64_i16)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,uint16_t,rechrom_builtin_add_u64_u64_u16,rechrom_builtin_sub_u64_u64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,uint16_t,rechrom_builtin_mul_u64_u64_u16)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,int32_t,rechrom_builtin_add_u64_u64_i32,rechrom_builtin_sub_u64_u64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,int32_t,rechrom_builtin_mul_u64_u64_i32)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,uint32_t,rechrom_builtin_add_u64_u64_u32,rechrom_builtin_sub_u64_u64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,uint32_t,rechrom_builtin_mul_u64_u64_u32)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,int64_t,rechrom_builtin_add_u64_u64_i64,rechrom_builtin_sub_u64_u64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,int64_t,rechrom_builtin_mul_u64_u64_i64)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,uint64_t,rechrom_builtin_add_u64_u64_u64,rechrom_builtin_sub_u64_u64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,uint64_t,rechrom_builtin_mul_u64_u64_u64)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,intptr_t,rechrom_builtin_add_u64_u64_isize,rechrom_builtin_sub_u64_u64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,intptr_t,rechrom_builtin_mul_u64_u64_isize)
#endif
RECHROM_ADD_SUB(uint64_t,uint64_t,uintptr_t,rechrom_builtin_add_u64_u64_usize,rechrom_builtin_sub_u64_u64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uint64_t,uintptr_t,rechrom_builtin_mul_u64_u64_usize)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,int8_t,rechrom_builtin_add_u64_isize_i8,rechrom_builtin_sub_u64_isize_i8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,int8_t,rechrom_builtin_mul_u64_isize_i8)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,uint8_t,rechrom_builtin_add_u64_isize_u8,rechrom_builtin_sub_u64_isize_u8)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,uint8_t,rechrom_builtin_mul_u64_isize_u8)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,int16_t,rechrom_builtin_add_u64_isize_i16,rechrom_builtin_sub_u64_isize_i16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,int16_t,rechrom_builtin_mul_u64_isize_i16)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,uint16_t,rechrom_builtin_add_u64_isize_u16,rechrom_builtin_sub_u64_isize_u16)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,uint16_t,rechrom_builtin_mul_u64_isize_u16)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,int32_t,rechrom_builtin_add_u64_isize_i32,rechrom_builtin_sub_u64_isize_i32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,int32_t,rechrom_builtin_mul_u64_isize_i32)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,uint32_t,rechrom_builtin_add_u64_isize_u32,rechrom_builtin_sub_u64_isize_u32)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,uint32_t,rechrom_builtin_mul_u64_isize_u32)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,int64_t,rechrom_builtin_add_u64_isize_i64,rechrom_builtin_sub_u64_isize_i64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,int64_t,rechrom_builtin_mul_u64_isize_i64)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,uint64_t,rechrom_builtin_add_u64_isize_u64,rechrom_builtin_sub_u64_isize_u64)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,uint64_t,rechrom_builtin_mul_u64_isize_u64)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,intptr_t,rechrom_builtin_add_u64_isize_isize,rechrom_builtin_sub_u64_isize_isize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,intptr_t,rechrom_builtin_mul_u64_isize_isize)
#endif
RECHROM_ADD_SUB(uint64_t,intptr_t,uintptr_t,rechrom_builtin_add_u64_isize_usize,rechrom_builtin_sub_u64_isize_usize)
#if !defined(__clang__)
RECHROM_MUL(uint64_t,intptr_t,uintptr_t,rechrom_builtin_mul_u64_isize_usize)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,int8_t,rechrom_builtin_add_u64_usize_i8,rechrom_builtin_sub_u64_usize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,int8_t,rechrom_builtin_mul_u64_usize_i8)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,uint8_t,rechrom_builtin_add_u64_usize_u8,rechrom_builtin_sub_u64_usize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,uint8_t,rechrom_builtin_mul_u64_usize_u8)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,int16_t,rechrom_builtin_add_u64_usize_i16,rechrom_builtin_sub_u64_usize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,int16_t,rechrom_builtin_mul_u64_usize_i16)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,uint16_t,rechrom_builtin_add_u64_usize_u16,rechrom_builtin_sub_u64_usize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,uint16_t,rechrom_builtin_mul_u64_usize_u16)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,int32_t,rechrom_builtin_add_u64_usize_i32,rechrom_builtin_sub_u64_usize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,int32_t,rechrom_builtin_mul_u64_usize_i32)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,uint32_t,rechrom_builtin_add_u64_usize_u32,rechrom_builtin_sub_u64_usize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,uint32_t,rechrom_builtin_mul_u64_usize_u32)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,int64_t,rechrom_builtin_add_u64_usize_i64,rechrom_builtin_sub_u64_usize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,int64_t,rechrom_builtin_mul_u64_usize_i64)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,uint64_t,rechrom_builtin_add_u64_usize_u64,rechrom_builtin_sub_u64_usize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,uint64_t,rechrom_builtin_mul_u64_usize_u64)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,intptr_t,rechrom_builtin_add_u64_usize_isize,rechrom_builtin_sub_u64_usize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,intptr_t,rechrom_builtin_mul_u64_usize_isize)
#endif
RECHROM_ADD_SUB(uint64_t,uintptr_t,uintptr_t,rechrom_builtin_add_u64_usize_usize,rechrom_builtin_sub_u64_usize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uint64_t,uintptr_t,uintptr_t,rechrom_builtin_mul_u64_usize_usize)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,int8_t,rechrom_builtin_add_isize_i8_i8,rechrom_builtin_sub_isize_i8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,int8_t,rechrom_builtin_mul_isize_i8_i8)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,uint8_t,rechrom_builtin_add_isize_i8_u8,rechrom_builtin_sub_isize_i8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,uint8_t,rechrom_builtin_mul_isize_i8_u8)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,int16_t,rechrom_builtin_add_isize_i8_i16,rechrom_builtin_sub_isize_i8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,int16_t,rechrom_builtin_mul_isize_i8_i16)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,uint16_t,rechrom_builtin_add_isize_i8_u16,rechrom_builtin_sub_isize_i8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,uint16_t,rechrom_builtin_mul_isize_i8_u16)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,int32_t,rechrom_builtin_add_isize_i8_i32,rechrom_builtin_sub_isize_i8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,int32_t,rechrom_builtin_mul_isize_i8_i32)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,uint32_t,rechrom_builtin_add_isize_i8_u32,rechrom_builtin_sub_isize_i8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,uint32_t,rechrom_builtin_mul_isize_i8_u32)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,int64_t,rechrom_builtin_add_isize_i8_i64,rechrom_builtin_sub_isize_i8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,int64_t,rechrom_builtin_mul_isize_i8_i64)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,uint64_t,rechrom_builtin_add_isize_i8_u64,rechrom_builtin_sub_isize_i8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,uint64_t,rechrom_builtin_mul_isize_i8_u64)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,intptr_t,rechrom_builtin_add_isize_i8_isize,rechrom_builtin_sub_isize_i8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,intptr_t,rechrom_builtin_mul_isize_i8_isize)
#endif
RECHROM_ADD_SUB(intptr_t,int8_t,uintptr_t,rechrom_builtin_add_isize_i8_usize,rechrom_builtin_sub_isize_i8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int8_t,uintptr_t,rechrom_builtin_mul_isize_i8_usize)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,int8_t,rechrom_builtin_add_isize_u8_i8,rechrom_builtin_sub_isize_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,int8_t,rechrom_builtin_mul_isize_u8_i8)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,uint8_t,rechrom_builtin_add_isize_u8_u8,rechrom_builtin_sub_isize_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,uint8_t,rechrom_builtin_mul_isize_u8_u8)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,int16_t,rechrom_builtin_add_isize_u8_i16,rechrom_builtin_sub_isize_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,int16_t,rechrom_builtin_mul_isize_u8_i16)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,uint16_t,rechrom_builtin_add_isize_u8_u16,rechrom_builtin_sub_isize_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,uint16_t,rechrom_builtin_mul_isize_u8_u16)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,int32_t,rechrom_builtin_add_isize_u8_i32,rechrom_builtin_sub_isize_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,int32_t,rechrom_builtin_mul_isize_u8_i32)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,uint32_t,rechrom_builtin_add_isize_u8_u32,rechrom_builtin_sub_isize_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,uint32_t,rechrom_builtin_mul_isize_u8_u32)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,int64_t,rechrom_builtin_add_isize_u8_i64,rechrom_builtin_sub_isize_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,int64_t,rechrom_builtin_mul_isize_u8_i64)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,uint64_t,rechrom_builtin_add_isize_u8_u64,rechrom_builtin_sub_isize_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,uint64_t,rechrom_builtin_mul_isize_u8_u64)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,intptr_t,rechrom_builtin_add_isize_u8_isize,rechrom_builtin_sub_isize_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,intptr_t,rechrom_builtin_mul_isize_u8_isize)
#endif
RECHROM_ADD_SUB(intptr_t,uint8_t,uintptr_t,rechrom_builtin_add_isize_u8_usize,rechrom_builtin_sub_isize_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint8_t,uintptr_t,rechrom_builtin_mul_isize_u8_usize)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,int8_t,rechrom_builtin_add_isize_i16_i8,rechrom_builtin_sub_isize_i16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,int8_t,rechrom_builtin_mul_isize_i16_i8)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,uint8_t,rechrom_builtin_add_isize_i16_u8,rechrom_builtin_sub_isize_i16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,uint8_t,rechrom_builtin_mul_isize_i16_u8)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,int16_t,rechrom_builtin_add_isize_i16_i16,rechrom_builtin_sub_isize_i16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,int16_t,rechrom_builtin_mul_isize_i16_i16)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,uint16_t,rechrom_builtin_add_isize_i16_u16,rechrom_builtin_sub_isize_i16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,uint16_t,rechrom_builtin_mul_isize_i16_u16)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,int32_t,rechrom_builtin_add_isize_i16_i32,rechrom_builtin_sub_isize_i16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,int32_t,rechrom_builtin_mul_isize_i16_i32)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,uint32_t,rechrom_builtin_add_isize_i16_u32,rechrom_builtin_sub_isize_i16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,uint32_t,rechrom_builtin_mul_isize_i16_u32)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,int64_t,rechrom_builtin_add_isize_i16_i64,rechrom_builtin_sub_isize_i16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,int64_t,rechrom_builtin_mul_isize_i16_i64)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,uint64_t,rechrom_builtin_add_isize_i16_u64,rechrom_builtin_sub_isize_i16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,uint64_t,rechrom_builtin_mul_isize_i16_u64)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,intptr_t,rechrom_builtin_add_isize_i16_isize,rechrom_builtin_sub_isize_i16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,intptr_t,rechrom_builtin_mul_isize_i16_isize)
#endif
RECHROM_ADD_SUB(intptr_t,int16_t,uintptr_t,rechrom_builtin_add_isize_i16_usize,rechrom_builtin_sub_isize_i16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int16_t,uintptr_t,rechrom_builtin_mul_isize_i16_usize)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,int8_t,rechrom_builtin_add_isize_u16_i8,rechrom_builtin_sub_isize_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,int8_t,rechrom_builtin_mul_isize_u16_i8)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,uint8_t,rechrom_builtin_add_isize_u16_u8,rechrom_builtin_sub_isize_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,uint8_t,rechrom_builtin_mul_isize_u16_u8)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,int16_t,rechrom_builtin_add_isize_u16_i16,rechrom_builtin_sub_isize_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,int16_t,rechrom_builtin_mul_isize_u16_i16)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,uint16_t,rechrom_builtin_add_isize_u16_u16,rechrom_builtin_sub_isize_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,uint16_t,rechrom_builtin_mul_isize_u16_u16)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,int32_t,rechrom_builtin_add_isize_u16_i32,rechrom_builtin_sub_isize_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,int32_t,rechrom_builtin_mul_isize_u16_i32)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,uint32_t,rechrom_builtin_add_isize_u16_u32,rechrom_builtin_sub_isize_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,uint32_t,rechrom_builtin_mul_isize_u16_u32)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,int64_t,rechrom_builtin_add_isize_u16_i64,rechrom_builtin_sub_isize_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,int64_t,rechrom_builtin_mul_isize_u16_i64)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,uint64_t,rechrom_builtin_add_isize_u16_u64,rechrom_builtin_sub_isize_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,uint64_t,rechrom_builtin_mul_isize_u16_u64)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,intptr_t,rechrom_builtin_add_isize_u16_isize,rechrom_builtin_sub_isize_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,intptr_t,rechrom_builtin_mul_isize_u16_isize)
#endif
RECHROM_ADD_SUB(intptr_t,uint16_t,uintptr_t,rechrom_builtin_add_isize_u16_usize,rechrom_builtin_sub_isize_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint16_t,uintptr_t,rechrom_builtin_mul_isize_u16_usize)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,int8_t,rechrom_builtin_add_isize_i32_i8,rechrom_builtin_sub_isize_i32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,int8_t,rechrom_builtin_mul_isize_i32_i8)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,uint8_t,rechrom_builtin_add_isize_i32_u8,rechrom_builtin_sub_isize_i32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,uint8_t,rechrom_builtin_mul_isize_i32_u8)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,int16_t,rechrom_builtin_add_isize_i32_i16,rechrom_builtin_sub_isize_i32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,int16_t,rechrom_builtin_mul_isize_i32_i16)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,uint16_t,rechrom_builtin_add_isize_i32_u16,rechrom_builtin_sub_isize_i32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,uint16_t,rechrom_builtin_mul_isize_i32_u16)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,int32_t,rechrom_builtin_add_isize_i32_i32,rechrom_builtin_sub_isize_i32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,int32_t,rechrom_builtin_mul_isize_i32_i32)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,uint32_t,rechrom_builtin_add_isize_i32_u32,rechrom_builtin_sub_isize_i32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,uint32_t,rechrom_builtin_mul_isize_i32_u32)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,int64_t,rechrom_builtin_add_isize_i32_i64,rechrom_builtin_sub_isize_i32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,int64_t,rechrom_builtin_mul_isize_i32_i64)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,uint64_t,rechrom_builtin_add_isize_i32_u64,rechrom_builtin_sub_isize_i32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,uint64_t,rechrom_builtin_mul_isize_i32_u64)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,intptr_t,rechrom_builtin_add_isize_i32_isize,rechrom_builtin_sub_isize_i32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,intptr_t,rechrom_builtin_mul_isize_i32_isize)
#endif
RECHROM_ADD_SUB(intptr_t,int32_t,uintptr_t,rechrom_builtin_add_isize_i32_usize,rechrom_builtin_sub_isize_i32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int32_t,uintptr_t,rechrom_builtin_mul_isize_i32_usize)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,int8_t,rechrom_builtin_add_isize_u32_i8,rechrom_builtin_sub_isize_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,int8_t,rechrom_builtin_mul_isize_u32_i8)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,uint8_t,rechrom_builtin_add_isize_u32_u8,rechrom_builtin_sub_isize_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,uint8_t,rechrom_builtin_mul_isize_u32_u8)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,int16_t,rechrom_builtin_add_isize_u32_i16,rechrom_builtin_sub_isize_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,int16_t,rechrom_builtin_mul_isize_u32_i16)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,uint16_t,rechrom_builtin_add_isize_u32_u16,rechrom_builtin_sub_isize_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,uint16_t,rechrom_builtin_mul_isize_u32_u16)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,int32_t,rechrom_builtin_add_isize_u32_i32,rechrom_builtin_sub_isize_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,int32_t,rechrom_builtin_mul_isize_u32_i32)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,uint32_t,rechrom_builtin_add_isize_u32_u32,rechrom_builtin_sub_isize_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,uint32_t,rechrom_builtin_mul_isize_u32_u32)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,int64_t,rechrom_builtin_add_isize_u32_i64,rechrom_builtin_sub_isize_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,int64_t,rechrom_builtin_mul_isize_u32_i64)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,uint64_t,rechrom_builtin_add_isize_u32_u64,rechrom_builtin_sub_isize_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,uint64_t,rechrom_builtin_mul_isize_u32_u64)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,intptr_t,rechrom_builtin_add_isize_u32_isize,rechrom_builtin_sub_isize_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,intptr_t,rechrom_builtin_mul_isize_u32_isize)
#endif
RECHROM_ADD_SUB(intptr_t,uint32_t,uintptr_t,rechrom_builtin_add_isize_u32_usize,rechrom_builtin_sub_isize_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,uint32_t,uintptr_t,rechrom_builtin_mul_isize_u32_usize)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,int8_t,rechrom_builtin_add_isize_i64_i8,rechrom_builtin_sub_isize_i64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,int8_t,rechrom_builtin_mul_isize_i64_i8)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,uint8_t,rechrom_builtin_add_isize_i64_u8,rechrom_builtin_sub_isize_i64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,uint8_t,rechrom_builtin_mul_isize_i64_u8)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,int16_t,rechrom_builtin_add_isize_i64_i16,rechrom_builtin_sub_isize_i64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,int16_t,rechrom_builtin_mul_isize_i64_i16)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,uint16_t,rechrom_builtin_add_isize_i64_u16,rechrom_builtin_sub_isize_i64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,uint16_t,rechrom_builtin_mul_isize_i64_u16)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,int32_t,rechrom_builtin_add_isize_i64_i32,rechrom_builtin_sub_isize_i64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,int32_t,rechrom_builtin_mul_isize_i64_i32)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,uint32_t,rechrom_builtin_add_isize_i64_u32,rechrom_builtin_sub_isize_i64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,uint32_t,rechrom_builtin_mul_isize_i64_u32)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,int64_t,rechrom_builtin_add_isize_i64_i64,rechrom_builtin_sub_isize_i64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,int64_t,rechrom_builtin_mul_isize_i64_i64)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,uint64_t,rechrom_builtin_add_isize_i64_u64,rechrom_builtin_sub_isize_i64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,uint64_t,rechrom_builtin_mul_isize_i64_u64)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,intptr_t,rechrom_builtin_add_isize_i64_isize,rechrom_builtin_sub_isize_i64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,intptr_t,rechrom_builtin_mul_isize_i64_isize)
#endif
RECHROM_ADD_SUB(intptr_t,int64_t,uintptr_t,rechrom_builtin_add_isize_i64_usize,rechrom_builtin_sub_isize_i64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,int64_t,uintptr_t,rechrom_builtin_mul_isize_i64_usize)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,int8_t,rechrom_builtin_add_isize_u64_i8,rechrom_builtin_sub_isize_u64_i8)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,int8_t,rechrom_builtin_mul_isize_u64_i8)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,uint8_t,rechrom_builtin_add_isize_u64_u8,rechrom_builtin_sub_isize_u64_u8)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,uint8_t,rechrom_builtin_mul_isize_u64_u8)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,int16_t,rechrom_builtin_add_isize_u64_i16,rechrom_builtin_sub_isize_u64_i16)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,int16_t,rechrom_builtin_mul_isize_u64_i16)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,uint16_t,rechrom_builtin_add_isize_u64_u16,rechrom_builtin_sub_isize_u64_u16)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,uint16_t,rechrom_builtin_mul_isize_u64_u16)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,int32_t,rechrom_builtin_add_isize_u64_i32,rechrom_builtin_sub_isize_u64_i32)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,int32_t,rechrom_builtin_mul_isize_u64_i32)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,uint32_t,rechrom_builtin_add_isize_u64_u32,rechrom_builtin_sub_isize_u64_u32)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,uint32_t,rechrom_builtin_mul_isize_u64_u32)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,int64_t,rechrom_builtin_add_isize_u64_i64,rechrom_builtin_sub_isize_u64_i64)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,int64_t,rechrom_builtin_mul_isize_u64_i64)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,uint64_t,rechrom_builtin_add_isize_u64_u64,rechrom_builtin_sub_isize_u64_u64)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,uint64_t,rechrom_builtin_mul_isize_u64_u64)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,intptr_t,rechrom_builtin_add_isize_u64_isize,rechrom_builtin_sub_isize_u64_isize)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,intptr_t,rechrom_builtin_mul_isize_u64_isize)
#endif
RECHROM_ADD_SUB(intptr_t,uint64_t,uintptr_t,rechrom_builtin_add_isize_u64_usize,rechrom_builtin_sub_isize_u64_usize)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uint64_t,uintptr_t,rechrom_builtin_mul_isize_u64_usize)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,int8_t,rechrom_builtin_add_isize_isize_i8,rechrom_builtin_sub_isize_isize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,int8_t,rechrom_builtin_mul_isize_isize_i8)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,uint8_t,rechrom_builtin_add_isize_isize_u8,rechrom_builtin_sub_isize_isize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,uint8_t,rechrom_builtin_mul_isize_isize_u8)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,int16_t,rechrom_builtin_add_isize_isize_i16,rechrom_builtin_sub_isize_isize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,int16_t,rechrom_builtin_mul_isize_isize_i16)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,uint16_t,rechrom_builtin_add_isize_isize_u16,rechrom_builtin_sub_isize_isize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,uint16_t,rechrom_builtin_mul_isize_isize_u16)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,int32_t,rechrom_builtin_add_isize_isize_i32,rechrom_builtin_sub_isize_isize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,int32_t,rechrom_builtin_mul_isize_isize_i32)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,uint32_t,rechrom_builtin_add_isize_isize_u32,rechrom_builtin_sub_isize_isize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,uint32_t,rechrom_builtin_mul_isize_isize_u32)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,int64_t,rechrom_builtin_add_isize_isize_i64,rechrom_builtin_sub_isize_isize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,int64_t,rechrom_builtin_mul_isize_isize_i64)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,uint64_t,rechrom_builtin_add_isize_isize_u64,rechrom_builtin_sub_isize_isize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,uint64_t,rechrom_builtin_mul_isize_isize_u64)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,intptr_t,rechrom_builtin_add_isize_isize_isize,rechrom_builtin_sub_isize_isize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,intptr_t,rechrom_builtin_mul_isize_isize_isize)
#endif
RECHROM_ADD_SUB(intptr_t,intptr_t,uintptr_t,rechrom_builtin_add_isize_isize_usize,rechrom_builtin_sub_isize_isize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(intptr_t,intptr_t,uintptr_t,rechrom_builtin_mul_isize_isize_usize)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,int8_t,rechrom_builtin_add_isize_usize_i8,rechrom_builtin_sub_isize_usize_i8)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,int8_t,rechrom_builtin_mul_isize_usize_i8)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,uint8_t,rechrom_builtin_add_isize_usize_u8,rechrom_builtin_sub_isize_usize_u8)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,uint8_t,rechrom_builtin_mul_isize_usize_u8)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,int16_t,rechrom_builtin_add_isize_usize_i16,rechrom_builtin_sub_isize_usize_i16)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,int16_t,rechrom_builtin_mul_isize_usize_i16)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,uint16_t,rechrom_builtin_add_isize_usize_u16,rechrom_builtin_sub_isize_usize_u16)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,uint16_t,rechrom_builtin_mul_isize_usize_u16)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,int32_t,rechrom_builtin_add_isize_usize_i32,rechrom_builtin_sub_isize_usize_i32)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,int32_t,rechrom_builtin_mul_isize_usize_i32)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,uint32_t,rechrom_builtin_add_isize_usize_u32,rechrom_builtin_sub_isize_usize_u32)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,uint32_t,rechrom_builtin_mul_isize_usize_u32)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,int64_t,rechrom_builtin_add_isize_usize_i64,rechrom_builtin_sub_isize_usize_i64)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,int64_t,rechrom_builtin_mul_isize_usize_i64)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,uint64_t,rechrom_builtin_add_isize_usize_u64,rechrom_builtin_sub_isize_usize_u64)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,uint64_t,rechrom_builtin_mul_isize_usize_u64)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,intptr_t,rechrom_builtin_add_isize_usize_isize,rechrom_builtin_sub_isize_usize_isize)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,intptr_t,rechrom_builtin_mul_isize_usize_isize)
#endif
RECHROM_ADD_SUB(intptr_t,uintptr_t,uintptr_t,rechrom_builtin_add_isize_usize_usize,rechrom_builtin_sub_isize_usize_usize)
#if !defined(__clang__)
RECHROM_MUL(intptr_t,uintptr_t,uintptr_t,rechrom_builtin_mul_isize_usize_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,int8_t,rechrom_builtin_add_usize_i8_i8,rechrom_builtin_sub_usize_i8_i8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,int8_t,rechrom_builtin_mul_usize_i8_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,uint8_t,rechrom_builtin_add_usize_i8_u8,rechrom_builtin_sub_usize_i8_u8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,uint8_t,rechrom_builtin_mul_usize_i8_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,int16_t,rechrom_builtin_add_usize_i8_i16,rechrom_builtin_sub_usize_i8_i16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,int16_t,rechrom_builtin_mul_usize_i8_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,uint16_t,rechrom_builtin_add_usize_i8_u16,rechrom_builtin_sub_usize_i8_u16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,uint16_t,rechrom_builtin_mul_usize_i8_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,int32_t,rechrom_builtin_add_usize_i8_i32,rechrom_builtin_sub_usize_i8_i32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,int32_t,rechrom_builtin_mul_usize_i8_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,uint32_t,rechrom_builtin_add_usize_i8_u32,rechrom_builtin_sub_usize_i8_u32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,uint32_t,rechrom_builtin_mul_usize_i8_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,int64_t,rechrom_builtin_add_usize_i8_i64,rechrom_builtin_sub_usize_i8_i64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,int64_t,rechrom_builtin_mul_usize_i8_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,uint64_t,rechrom_builtin_add_usize_i8_u64,rechrom_builtin_sub_usize_i8_u64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,uint64_t,rechrom_builtin_mul_usize_i8_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,intptr_t,rechrom_builtin_add_usize_i8_isize,rechrom_builtin_sub_usize_i8_isize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,intptr_t,rechrom_builtin_mul_usize_i8_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,int8_t,uintptr_t,rechrom_builtin_add_usize_i8_usize,rechrom_builtin_sub_usize_i8_usize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int8_t,uintptr_t,rechrom_builtin_mul_usize_i8_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,int8_t,rechrom_builtin_add_usize_u8_i8,rechrom_builtin_sub_usize_u8_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,int8_t,rechrom_builtin_mul_usize_u8_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,uint8_t,rechrom_builtin_add_usize_u8_u8,rechrom_builtin_sub_usize_u8_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,uint8_t,rechrom_builtin_mul_usize_u8_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,int16_t,rechrom_builtin_add_usize_u8_i16,rechrom_builtin_sub_usize_u8_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,int16_t,rechrom_builtin_mul_usize_u8_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,uint16_t,rechrom_builtin_add_usize_u8_u16,rechrom_builtin_sub_usize_u8_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,uint16_t,rechrom_builtin_mul_usize_u8_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,int32_t,rechrom_builtin_add_usize_u8_i32,rechrom_builtin_sub_usize_u8_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,int32_t,rechrom_builtin_mul_usize_u8_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,uint32_t,rechrom_builtin_add_usize_u8_u32,rechrom_builtin_sub_usize_u8_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,uint32_t,rechrom_builtin_mul_usize_u8_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,int64_t,rechrom_builtin_add_usize_u8_i64,rechrom_builtin_sub_usize_u8_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,int64_t,rechrom_builtin_mul_usize_u8_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,uint64_t,rechrom_builtin_add_usize_u8_u64,rechrom_builtin_sub_usize_u8_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,uint64_t,rechrom_builtin_mul_usize_u8_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,intptr_t,rechrom_builtin_add_usize_u8_isize,rechrom_builtin_sub_usize_u8_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,intptr_t,rechrom_builtin_mul_usize_u8_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint8_t,uintptr_t,rechrom_builtin_add_usize_u8_usize,rechrom_builtin_sub_usize_u8_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint8_t,uintptr_t,rechrom_builtin_mul_usize_u8_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,int8_t,rechrom_builtin_add_usize_i16_i8,rechrom_builtin_sub_usize_i16_i8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,int8_t,rechrom_builtin_mul_usize_i16_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,uint8_t,rechrom_builtin_add_usize_i16_u8,rechrom_builtin_sub_usize_i16_u8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,uint8_t,rechrom_builtin_mul_usize_i16_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,int16_t,rechrom_builtin_add_usize_i16_i16,rechrom_builtin_sub_usize_i16_i16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,int16_t,rechrom_builtin_mul_usize_i16_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,uint16_t,rechrom_builtin_add_usize_i16_u16,rechrom_builtin_sub_usize_i16_u16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,uint16_t,rechrom_builtin_mul_usize_i16_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,int32_t,rechrom_builtin_add_usize_i16_i32,rechrom_builtin_sub_usize_i16_i32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,int32_t,rechrom_builtin_mul_usize_i16_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,uint32_t,rechrom_builtin_add_usize_i16_u32,rechrom_builtin_sub_usize_i16_u32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,uint32_t,rechrom_builtin_mul_usize_i16_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,int64_t,rechrom_builtin_add_usize_i16_i64,rechrom_builtin_sub_usize_i16_i64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,int64_t,rechrom_builtin_mul_usize_i16_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,uint64_t,rechrom_builtin_add_usize_i16_u64,rechrom_builtin_sub_usize_i16_u64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,uint64_t,rechrom_builtin_mul_usize_i16_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,intptr_t,rechrom_builtin_add_usize_i16_isize,rechrom_builtin_sub_usize_i16_isize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,intptr_t,rechrom_builtin_mul_usize_i16_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,int16_t,uintptr_t,rechrom_builtin_add_usize_i16_usize,rechrom_builtin_sub_usize_i16_usize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int16_t,uintptr_t,rechrom_builtin_mul_usize_i16_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,int8_t,rechrom_builtin_add_usize_u16_i8,rechrom_builtin_sub_usize_u16_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,int8_t,rechrom_builtin_mul_usize_u16_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,uint8_t,rechrom_builtin_add_usize_u16_u8,rechrom_builtin_sub_usize_u16_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,uint8_t,rechrom_builtin_mul_usize_u16_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,int16_t,rechrom_builtin_add_usize_u16_i16,rechrom_builtin_sub_usize_u16_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,int16_t,rechrom_builtin_mul_usize_u16_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,uint16_t,rechrom_builtin_add_usize_u16_u16,rechrom_builtin_sub_usize_u16_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,uint16_t,rechrom_builtin_mul_usize_u16_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,int32_t,rechrom_builtin_add_usize_u16_i32,rechrom_builtin_sub_usize_u16_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,int32_t,rechrom_builtin_mul_usize_u16_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,uint32_t,rechrom_builtin_add_usize_u16_u32,rechrom_builtin_sub_usize_u16_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,uint32_t,rechrom_builtin_mul_usize_u16_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,int64_t,rechrom_builtin_add_usize_u16_i64,rechrom_builtin_sub_usize_u16_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,int64_t,rechrom_builtin_mul_usize_u16_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,uint64_t,rechrom_builtin_add_usize_u16_u64,rechrom_builtin_sub_usize_u16_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,uint64_t,rechrom_builtin_mul_usize_u16_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,intptr_t,rechrom_builtin_add_usize_u16_isize,rechrom_builtin_sub_usize_u16_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,intptr_t,rechrom_builtin_mul_usize_u16_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint16_t,uintptr_t,rechrom_builtin_add_usize_u16_usize,rechrom_builtin_sub_usize_u16_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint16_t,uintptr_t,rechrom_builtin_mul_usize_u16_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,int8_t,rechrom_builtin_add_usize_i32_i8,rechrom_builtin_sub_usize_i32_i8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,int8_t,rechrom_builtin_mul_usize_i32_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,uint8_t,rechrom_builtin_add_usize_i32_u8,rechrom_builtin_sub_usize_i32_u8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,uint8_t,rechrom_builtin_mul_usize_i32_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,int16_t,rechrom_builtin_add_usize_i32_i16,rechrom_builtin_sub_usize_i32_i16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,int16_t,rechrom_builtin_mul_usize_i32_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,uint16_t,rechrom_builtin_add_usize_i32_u16,rechrom_builtin_sub_usize_i32_u16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,uint16_t,rechrom_builtin_mul_usize_i32_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,int32_t,rechrom_builtin_add_usize_i32_i32,rechrom_builtin_sub_usize_i32_i32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,int32_t,rechrom_builtin_mul_usize_i32_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,uint32_t,rechrom_builtin_add_usize_i32_u32,rechrom_builtin_sub_usize_i32_u32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,uint32_t,rechrom_builtin_mul_usize_i32_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,int64_t,rechrom_builtin_add_usize_i32_i64,rechrom_builtin_sub_usize_i32_i64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,int64_t,rechrom_builtin_mul_usize_i32_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,uint64_t,rechrom_builtin_add_usize_i32_u64,rechrom_builtin_sub_usize_i32_u64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,uint64_t,rechrom_builtin_mul_usize_i32_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,intptr_t,rechrom_builtin_add_usize_i32_isize,rechrom_builtin_sub_usize_i32_isize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,intptr_t,rechrom_builtin_mul_usize_i32_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,int32_t,uintptr_t,rechrom_builtin_add_usize_i32_usize,rechrom_builtin_sub_usize_i32_usize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int32_t,uintptr_t,rechrom_builtin_mul_usize_i32_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,int8_t,rechrom_builtin_add_usize_u32_i8,rechrom_builtin_sub_usize_u32_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,int8_t,rechrom_builtin_mul_usize_u32_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,uint8_t,rechrom_builtin_add_usize_u32_u8,rechrom_builtin_sub_usize_u32_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,uint8_t,rechrom_builtin_mul_usize_u32_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,int16_t,rechrom_builtin_add_usize_u32_i16,rechrom_builtin_sub_usize_u32_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,int16_t,rechrom_builtin_mul_usize_u32_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,uint16_t,rechrom_builtin_add_usize_u32_u16,rechrom_builtin_sub_usize_u32_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,uint16_t,rechrom_builtin_mul_usize_u32_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,int32_t,rechrom_builtin_add_usize_u32_i32,rechrom_builtin_sub_usize_u32_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,int32_t,rechrom_builtin_mul_usize_u32_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,uint32_t,rechrom_builtin_add_usize_u32_u32,rechrom_builtin_sub_usize_u32_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,uint32_t,rechrom_builtin_mul_usize_u32_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,int64_t,rechrom_builtin_add_usize_u32_i64,rechrom_builtin_sub_usize_u32_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,int64_t,rechrom_builtin_mul_usize_u32_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,uint64_t,rechrom_builtin_add_usize_u32_u64,rechrom_builtin_sub_usize_u32_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,uint64_t,rechrom_builtin_mul_usize_u32_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,intptr_t,rechrom_builtin_add_usize_u32_isize,rechrom_builtin_sub_usize_u32_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,intptr_t,rechrom_builtin_mul_usize_u32_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint32_t,uintptr_t,rechrom_builtin_add_usize_u32_usize,rechrom_builtin_sub_usize_u32_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint32_t,uintptr_t,rechrom_builtin_mul_usize_u32_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,int8_t,rechrom_builtin_add_usize_i64_i8,rechrom_builtin_sub_usize_i64_i8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,int8_t,rechrom_builtin_mul_usize_i64_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,uint8_t,rechrom_builtin_add_usize_i64_u8,rechrom_builtin_sub_usize_i64_u8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,uint8_t,rechrom_builtin_mul_usize_i64_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,int16_t,rechrom_builtin_add_usize_i64_i16,rechrom_builtin_sub_usize_i64_i16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,int16_t,rechrom_builtin_mul_usize_i64_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,uint16_t,rechrom_builtin_add_usize_i64_u16,rechrom_builtin_sub_usize_i64_u16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,uint16_t,rechrom_builtin_mul_usize_i64_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,int32_t,rechrom_builtin_add_usize_i64_i32,rechrom_builtin_sub_usize_i64_i32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,int32_t,rechrom_builtin_mul_usize_i64_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,uint32_t,rechrom_builtin_add_usize_i64_u32,rechrom_builtin_sub_usize_i64_u32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,uint32_t,rechrom_builtin_mul_usize_i64_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,int64_t,rechrom_builtin_add_usize_i64_i64,rechrom_builtin_sub_usize_i64_i64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,int64_t,rechrom_builtin_mul_usize_i64_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,uint64_t,rechrom_builtin_add_usize_i64_u64,rechrom_builtin_sub_usize_i64_u64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,uint64_t,rechrom_builtin_mul_usize_i64_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,intptr_t,rechrom_builtin_add_usize_i64_isize,rechrom_builtin_sub_usize_i64_isize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,intptr_t,rechrom_builtin_mul_usize_i64_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,int64_t,uintptr_t,rechrom_builtin_add_usize_i64_usize,rechrom_builtin_sub_usize_i64_usize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,int64_t,uintptr_t,rechrom_builtin_mul_usize_i64_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,int8_t,rechrom_builtin_add_usize_u64_i8,rechrom_builtin_sub_usize_u64_i8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,int8_t,rechrom_builtin_mul_usize_u64_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,uint8_t,rechrom_builtin_add_usize_u64_u8,rechrom_builtin_sub_usize_u64_u8)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,uint8_t,rechrom_builtin_mul_usize_u64_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,int16_t,rechrom_builtin_add_usize_u64_i16,rechrom_builtin_sub_usize_u64_i16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,int16_t,rechrom_builtin_mul_usize_u64_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,uint16_t,rechrom_builtin_add_usize_u64_u16,rechrom_builtin_sub_usize_u64_u16)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,uint16_t,rechrom_builtin_mul_usize_u64_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,int32_t,rechrom_builtin_add_usize_u64_i32,rechrom_builtin_sub_usize_u64_i32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,int32_t,rechrom_builtin_mul_usize_u64_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,uint32_t,rechrom_builtin_add_usize_u64_u32,rechrom_builtin_sub_usize_u64_u32)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,uint32_t,rechrom_builtin_mul_usize_u64_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,int64_t,rechrom_builtin_add_usize_u64_i64,rechrom_builtin_sub_usize_u64_i64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,int64_t,rechrom_builtin_mul_usize_u64_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,uint64_t,rechrom_builtin_add_usize_u64_u64,rechrom_builtin_sub_usize_u64_u64)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,uint64_t,rechrom_builtin_mul_usize_u64_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,intptr_t,rechrom_builtin_add_usize_u64_isize,rechrom_builtin_sub_usize_u64_isize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,intptr_t,rechrom_builtin_mul_usize_u64_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,uint64_t,uintptr_t,rechrom_builtin_add_usize_u64_usize,rechrom_builtin_sub_usize_u64_usize)
#if !defined(__clang__) || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uint64_t,uintptr_t,rechrom_builtin_mul_usize_u64_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,int8_t,rechrom_builtin_add_usize_isize_i8,rechrom_builtin_sub_usize_isize_i8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,int8_t,rechrom_builtin_mul_usize_isize_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,uint8_t,rechrom_builtin_add_usize_isize_u8,rechrom_builtin_sub_usize_isize_u8)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,uint8_t,rechrom_builtin_mul_usize_isize_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,int16_t,rechrom_builtin_add_usize_isize_i16,rechrom_builtin_sub_usize_isize_i16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,int16_t,rechrom_builtin_mul_usize_isize_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,uint16_t,rechrom_builtin_add_usize_isize_u16,rechrom_builtin_sub_usize_isize_u16)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,uint16_t,rechrom_builtin_mul_usize_isize_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,int32_t,rechrom_builtin_add_usize_isize_i32,rechrom_builtin_sub_usize_isize_i32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,int32_t,rechrom_builtin_mul_usize_isize_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,uint32_t,rechrom_builtin_add_usize_isize_u32,rechrom_builtin_sub_usize_isize_u32)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,uint32_t,rechrom_builtin_mul_usize_isize_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,int64_t,rechrom_builtin_add_usize_isize_i64,rechrom_builtin_sub_usize_isize_i64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,int64_t,rechrom_builtin_mul_usize_isize_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,uint64_t,rechrom_builtin_add_usize_isize_u64,rechrom_builtin_sub_usize_isize_u64)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,uint64_t,rechrom_builtin_mul_usize_isize_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,intptr_t,rechrom_builtin_add_usize_isize_isize,rechrom_builtin_sub_usize_isize_isize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,intptr_t,rechrom_builtin_mul_usize_isize_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,intptr_t,uintptr_t,rechrom_builtin_add_usize_isize_usize,rechrom_builtin_sub_usize_isize_usize)
#if !defined(__clang__)
RECHROM_MUL(uintptr_t,intptr_t,uintptr_t,rechrom_builtin_mul_usize_isize_usize)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,int8_t,rechrom_builtin_add_usize_usize_i8,rechrom_builtin_sub_usize_usize_i8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,int8_t,rechrom_builtin_mul_usize_usize_i8)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,uint8_t,rechrom_builtin_add_usize_usize_u8,rechrom_builtin_sub_usize_usize_u8)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,uint8_t,rechrom_builtin_mul_usize_usize_u8)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,int16_t,rechrom_builtin_add_usize_usize_i16,rechrom_builtin_sub_usize_usize_i16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,int16_t,rechrom_builtin_mul_usize_usize_i16)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,uint16_t,rechrom_builtin_add_usize_usize_u16,rechrom_builtin_sub_usize_usize_u16)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,uint16_t,rechrom_builtin_mul_usize_usize_u16)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,int32_t,rechrom_builtin_add_usize_usize_i32,rechrom_builtin_sub_usize_usize_i32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,int32_t,rechrom_builtin_mul_usize_usize_i32)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,uint32_t,rechrom_builtin_add_usize_usize_u32,rechrom_builtin_sub_usize_usize_u32)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,uint32_t,rechrom_builtin_mul_usize_usize_u32)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,int64_t,rechrom_builtin_add_usize_usize_i64,rechrom_builtin_sub_usize_usize_i64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,int64_t,rechrom_builtin_mul_usize_usize_i64)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,uint64_t,rechrom_builtin_add_usize_usize_u64,rechrom_builtin_sub_usize_usize_u64)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,uint64_t,rechrom_builtin_mul_usize_usize_u64)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,intptr_t,rechrom_builtin_add_usize_usize_isize,rechrom_builtin_sub_usize_usize_isize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,intptr_t,rechrom_builtin_mul_usize_usize_isize)
#endif
RECHROM_ADD_SUB(uintptr_t,uintptr_t,uintptr_t,rechrom_builtin_add_usize_usize_usize,rechrom_builtin_sub_usize_usize_usize)
#if !defined(__clang__) || INTPTR_MAX == INT32_MAX || INTPTR_MAX == INT64_MAX
RECHROM_MUL(uintptr_t,uintptr_t,uintptr_t,rechrom_builtin_mul_usize_usize_usize)
#endif
#undef RECHROM_ADD_SUB
#undef RECHROM_MUL
