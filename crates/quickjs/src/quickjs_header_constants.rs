// Numeric constants from official quickjs.h; MIT, see ../LICENSE.
// Public composite/default macros from quickjs.h. Keep tool consumers on the
// public interface rather than duplicating engine-private literals.
pub const JS_DEFAULT_STACK_SIZE: usize = 1024 * 1024;
pub const JS_PROP_C_W_E: i32 = JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE | JS_PROP_ENUMERABLE;
// c: quickjs.h:77
pub const JS_TAG_FIRST: i32 = -9;
// c: quickjs.h:78
pub const JS_TAG_BIG_INT: i32 = -9;
// c: quickjs.h:79
pub const JS_TAG_SYMBOL: i32 = -8;
// c: quickjs.h:80
pub const JS_TAG_STRING: i32 = -7;
// c: quickjs.h:81
pub const JS_TAG_STRING_ROPE: i32 = -6;
// c: quickjs.h:82
pub const JS_TAG_MODULE: i32 = -3;
// c: quickjs.h:83
pub const JS_TAG_FUNCTION_BYTECODE: i32 = -2;
// c: quickjs.h:84
pub const JS_TAG_OBJECT: i32 = -1;
// c: quickjs.h:85
pub const JS_TAG_INT: i32 = 0;
// c: quickjs.h:87
pub const JS_TAG_BOOL: i32 = 1;
// c: quickjs.h:88
pub const JS_TAG_NULL: i32 = 2;
// c: quickjs.h:89
pub const JS_TAG_UNDEFINED: i32 = 3;
// c: quickjs.h:90
pub const JS_TAG_UNINITIALIZED: i32 = 4;
// c: quickjs.h:91
pub const JS_TAG_CATCH_OFFSET: i32 = 5;
// c: quickjs.h:92
pub const JS_TAG_EXCEPTION: i32 = 6;
// c: quickjs.h:93
pub const JS_TAG_SHORT_BIG_INT: i32 = 7;
// c: quickjs.h:94
pub const JS_TAG_FLOAT64: i32 = 8;
// c: quickjs.h:298
#[allow(unused_parens)]
pub const JS_PROP_CONFIGURABLE: i32 = (1 << 0);
// c: quickjs.h:299
#[allow(unused_parens)]
pub const JS_PROP_WRITABLE: i32 = (1 << 1);
// c: quickjs.h:300
#[allow(unused_parens)]
pub const JS_PROP_ENUMERABLE: i32 = (1 << 2);
// c: quickjs.h:302
#[allow(unused_parens)]
pub const JS_PROP_LENGTH: i32 = (1 << 3);
// c: quickjs.h:303
#[allow(unused_parens)]
pub const JS_PROP_TMASK: i32 = (3 << 4);
// c: quickjs.h:304
#[allow(unused_parens)]
pub const JS_PROP_NORMAL: i32 = (0 << 4);
// c: quickjs.h:305
#[allow(unused_parens)]
pub const JS_PROP_GETSET: i32 = (1 << 4);
// c: quickjs.h:306
#[allow(unused_parens)]
pub const JS_PROP_VARREF: i32 = (2 << 4);
// c: quickjs.h:307
#[allow(unused_parens)]
pub const JS_PROP_AUTOINIT: i32 = (3 << 4);
// c: quickjs.h:310
pub const JS_PROP_HAS_SHIFT: i32 = 8;
// c: quickjs.h:311
#[allow(unused_parens)]
pub const JS_PROP_HAS_CONFIGURABLE: i32 = (1 << 8);
// c: quickjs.h:312
#[allow(unused_parens)]
pub const JS_PROP_HAS_WRITABLE: i32 = (1 << 9);
// c: quickjs.h:313
#[allow(unused_parens)]
pub const JS_PROP_HAS_ENUMERABLE: i32 = (1 << 10);
// c: quickjs.h:314
#[allow(unused_parens)]
pub const JS_PROP_HAS_GET: i32 = (1 << 11);
// c: quickjs.h:315
#[allow(unused_parens)]
pub const JS_PROP_HAS_SET: i32 = (1 << 12);
// c: quickjs.h:316
#[allow(unused_parens)]
pub const JS_PROP_HAS_VALUE: i32 = (1 << 13);
// c: quickjs.h:320
#[allow(unused_parens)]
pub const JS_PROP_THROW: i32 = (1 << 14);
// c: quickjs.h:323
#[allow(unused_parens)]
pub const JS_PROP_THROW_STRICT: i32 = (1 << 15);
// c: quickjs.h:325
#[allow(unused_parens)]
pub const JS_PROP_NO_EXOTIC: i32 = (1 << 16);
// c: quickjs.h:332
#[allow(unused_parens)]
pub const JS_EVAL_TYPE_GLOBAL: i32 = (0 << 0);
// c: quickjs.h:333
#[allow(unused_parens)]
pub const JS_EVAL_TYPE_MODULE: i32 = (1 << 0);
// c: quickjs.h:334
#[allow(unused_parens)]
pub const JS_EVAL_TYPE_DIRECT: i32 = (2 << 0);
// c: quickjs.h:335
#[allow(unused_parens)]
pub const JS_EVAL_TYPE_INDIRECT: i32 = (3 << 0);
// c: quickjs.h:336
#[allow(unused_parens)]
pub const JS_EVAL_TYPE_MASK: i32 = (3 << 0);
// c: quickjs.h:338
#[allow(unused_parens)]
pub const JS_EVAL_FLAG_STRICT: i32 = (1 << 3);
// c: quickjs.h:342
#[allow(unused_parens)]
pub const JS_EVAL_FLAG_COMPILE_ONLY: i32 = (1 << 5);
// c: quickjs.h:344
#[allow(unused_parens)]
pub const JS_EVAL_FLAG_BACKTRACE_BARRIER: i32 = (1 << 6);
// c: quickjs.h:347
#[allow(unused_parens)]
pub const JS_EVAL_FLAG_ASYNC: i32 = (1 << 7);
// c: quickjs.h:451
pub const JS_ATOM_NULL: i32 = 0;
// c: quickjs.h:526
#[allow(unused_parens)]
pub const JS_CALL_FLAG_CONSTRUCTOR: i32 = (1 << 0);
// c: quickjs.h:546
pub const JS_INVALID_CLASS_ID: i32 = 0;
// c: quickjs.h:810
#[allow(unused_parens)]
pub const JS_GPN_STRING_MASK: i32 = (1 << 0);
// c: quickjs.h:811
#[allow(unused_parens)]
pub const JS_GPN_SYMBOL_MASK: i32 = (1 << 1);
// c: quickjs.h:812
#[allow(unused_parens)]
pub const JS_GPN_PRIVATE_MASK: i32 = (1 << 2);
// c: quickjs.h:814
#[allow(unused_parens)]
pub const JS_GPN_ENUM_ONLY: i32 = (1 << 4);
// c: quickjs.h:816
#[allow(unused_parens)]
pub const JS_GPN_SET_ENUM: i32 = (1 << 5);
// c: quickjs.h:864
#[allow(unused_parens)]
pub const JS_PARSE_JSON_EXT: i32 = (1 << 0);
// c: quickjs.h:930
#[allow(unused_parens)]
pub const JS_STRIP_SOURCE: i32 = (1 << 0);
// c: quickjs.h:931
#[allow(unused_parens)]
pub const JS_STRIP_DEBUG: i32 = (1 << 1);
// c: quickjs.h:980
#[allow(unused_parens)]
pub const JS_WRITE_OBJ_BYTECODE: i32 = (1 << 0);
// c: quickjs.h:981
#[allow(unused_parens)]
pub const JS_WRITE_OBJ_BSWAP: i32 = (1 << 1);
// c: quickjs.h:982
#[allow(unused_parens)]
pub const JS_WRITE_OBJ_SAB: i32 = (1 << 2);
// c: quickjs.h:983
#[allow(unused_parens)]
pub const JS_WRITE_OBJ_REFERENCE: i32 = (1 << 3);
// c: quickjs.h:991
#[allow(unused_parens)]
pub const JS_READ_OBJ_BYTECODE: i32 = (1 << 0);
// c: quickjs.h:992
#[allow(unused_parens)]
pub const JS_READ_OBJ_ROM_DATA: i32 = (1 << 1);
// c: quickjs.h:993
#[allow(unused_parens)]
pub const JS_READ_OBJ_SAB: i32 = (1 << 2);
// c: quickjs.h:994
#[allow(unused_parens)]
pub const JS_READ_OBJ_REFERENCE: i32 = (1 << 3);
// c: quickjs.h:1099
pub const JS_DEF_CFUNC: i32 = 0;
// c: quickjs.h:1100
pub const JS_DEF_CGETSET: i32 = 1;
// c: quickjs.h:1101
pub const JS_DEF_CGETSET_MAGIC: i32 = 2;
// c: quickjs.h:1102
pub const JS_DEF_PROP_STRING: i32 = 3;
// c: quickjs.h:1103
pub const JS_DEF_PROP_INT32: i32 = 4;
// c: quickjs.h:1104
pub const JS_DEF_PROP_INT64: i32 = 5;
// c: quickjs.h:1105
pub const JS_DEF_PROP_DOUBLE: i32 = 6;
// c: quickjs.h:1106
pub const JS_DEF_PROP_UNDEFINED: i32 = 7;
// c: quickjs.h:1107
pub const JS_DEF_OBJECT: i32 = 8;
// c: quickjs.h:1108
pub const JS_DEF_ALIAS: i32 = 9;
// c: quickjs.h:1109
pub const JS_DEF_PROP_ATOM: i32 = 10;
// c: quickjs.h:1110
pub const JS_DEF_PROP_BOOL: i32 = 11;
