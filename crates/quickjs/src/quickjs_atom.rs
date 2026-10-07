/*
 * QuickJS atom definitions
 *
 * Copyright (c) 2017-2018 Fabrice Bellard
 * Copyright (c) 2017-2018 Charlie Gordon
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 * THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 */
// Direct table translation of quickjs-atom.h.
pub const JS_ATOM_NULL: u32 = 0;
// c: quickjs-atom.h:29
pub const JS_ATOM_null: u32 = 1;
// c: quickjs-atom.h:30
pub const JS_ATOM_false: u32 = 2;
// c: quickjs-atom.h:31
pub const JS_ATOM_true: u32 = 3;
// c: quickjs-atom.h:32
pub const JS_ATOM_if: u32 = 4;
// c: quickjs-atom.h:33
pub const JS_ATOM_else: u32 = 5;
// c: quickjs-atom.h:34
pub const JS_ATOM_return: u32 = 6;
// c: quickjs-atom.h:35
pub const JS_ATOM_var: u32 = 7;
// c: quickjs-atom.h:36
pub const JS_ATOM_this: u32 = 8;
// c: quickjs-atom.h:37
pub const JS_ATOM_delete: u32 = 9;
// c: quickjs-atom.h:38
pub const JS_ATOM_void: u32 = 10;
// c: quickjs-atom.h:39
pub const JS_ATOM_typeof: u32 = 11;
// c: quickjs-atom.h:40
pub const JS_ATOM_new: u32 = 12;
// c: quickjs-atom.h:41
pub const JS_ATOM_in: u32 = 13;
// c: quickjs-atom.h:42
pub const JS_ATOM_instanceof: u32 = 14;
// c: quickjs-atom.h:43
pub const JS_ATOM_do: u32 = 15;
// c: quickjs-atom.h:44
pub const JS_ATOM_while: u32 = 16;
// c: quickjs-atom.h:45
pub const JS_ATOM_for: u32 = 17;
// c: quickjs-atom.h:46
pub const JS_ATOM_break: u32 = 18;
// c: quickjs-atom.h:47
pub const JS_ATOM_continue: u32 = 19;
// c: quickjs-atom.h:48
pub const JS_ATOM_switch: u32 = 20;
// c: quickjs-atom.h:49
pub const JS_ATOM_case: u32 = 21;
// c: quickjs-atom.h:50
pub const JS_ATOM_default: u32 = 22;
// c: quickjs-atom.h:51
pub const JS_ATOM_throw: u32 = 23;
// c: quickjs-atom.h:52
pub const JS_ATOM_try: u32 = 24;
// c: quickjs-atom.h:53
pub const JS_ATOM_catch: u32 = 25;
// c: quickjs-atom.h:54
pub const JS_ATOM_finally: u32 = 26;
// c: quickjs-atom.h:55
pub const JS_ATOM_function: u32 = 27;
// c: quickjs-atom.h:56
pub const JS_ATOM_debugger: u32 = 28;
// c: quickjs-atom.h:57
pub const JS_ATOM_with: u32 = 29;
// c: quickjs-atom.h:59
pub const JS_ATOM_class: u32 = 30;
// c: quickjs-atom.h:60
pub const JS_ATOM_const: u32 = 31;
// c: quickjs-atom.h:61
pub const JS_ATOM_enum: u32 = 32;
// c: quickjs-atom.h:62
pub const JS_ATOM_export: u32 = 33;
// c: quickjs-atom.h:63
pub const JS_ATOM_extends: u32 = 34;
// c: quickjs-atom.h:64
pub const JS_ATOM_import: u32 = 35;
// c: quickjs-atom.h:65
pub const JS_ATOM_super: u32 = 36;
// c: quickjs-atom.h:67
pub const JS_ATOM_implements: u32 = 37;
// c: quickjs-atom.h:68
pub const JS_ATOM_interface: u32 = 38;
// c: quickjs-atom.h:69
pub const JS_ATOM_let: u32 = 39;
// c: quickjs-atom.h:70
pub const JS_ATOM_package: u32 = 40;
// c: quickjs-atom.h:71
pub const JS_ATOM_private: u32 = 41;
// c: quickjs-atom.h:72
pub const JS_ATOM_protected: u32 = 42;
// c: quickjs-atom.h:73
pub const JS_ATOM_public: u32 = 43;
// c: quickjs-atom.h:74
pub const JS_ATOM_static: u32 = 44;
// c: quickjs-atom.h:75
pub const JS_ATOM_yield: u32 = 45;
// c: quickjs-atom.h:76
pub const JS_ATOM_await: u32 = 46;
// c: quickjs-atom.h:79
pub const JS_ATOM_empty_string: u32 = 47;
// c: quickjs-atom.h:81
pub const JS_ATOM_keys: u32 = 48;
// c: quickjs-atom.h:82
pub const JS_ATOM_size: u32 = 49;
// c: quickjs-atom.h:83
pub const JS_ATOM_length: u32 = 50;
// c: quickjs-atom.h:84
pub const JS_ATOM_fileName: u32 = 51;
// c: quickjs-atom.h:85
pub const JS_ATOM_lineNumber: u32 = 52;
// c: quickjs-atom.h:86
pub const JS_ATOM_columnNumber: u32 = 53;
// c: quickjs-atom.h:87
pub const JS_ATOM_message: u32 = 54;
// c: quickjs-atom.h:88
pub const JS_ATOM_cause: u32 = 55;
// c: quickjs-atom.h:89
pub const JS_ATOM_errors: u32 = 56;
// c: quickjs-atom.h:90
pub const JS_ATOM_stack: u32 = 57;
// c: quickjs-atom.h:91
pub const JS_ATOM_name: u32 = 58;
// c: quickjs-atom.h:92
pub const JS_ATOM_toString: u32 = 59;
// c: quickjs-atom.h:93
pub const JS_ATOM_toLocaleString: u32 = 60;
// c: quickjs-atom.h:94
pub const JS_ATOM_valueOf: u32 = 61;
// c: quickjs-atom.h:95
pub const JS_ATOM_eval: u32 = 62;
// c: quickjs-atom.h:96
pub const JS_ATOM_prototype: u32 = 63;
// c: quickjs-atom.h:97
pub const JS_ATOM_constructor: u32 = 64;
// c: quickjs-atom.h:98
pub const JS_ATOM_configurable: u32 = 65;
// c: quickjs-atom.h:99
pub const JS_ATOM_writable: u32 = 66;
// c: quickjs-atom.h:100
pub const JS_ATOM_enumerable: u32 = 67;
// c: quickjs-atom.h:101
pub const JS_ATOM_value: u32 = 68;
// c: quickjs-atom.h:102
pub const JS_ATOM_get: u32 = 69;
// c: quickjs-atom.h:103
pub const JS_ATOM_set: u32 = 70;
// c: quickjs-atom.h:104
pub const JS_ATOM_of: u32 = 71;
// c: quickjs-atom.h:105
pub const JS_ATOM___proto__: u32 = 72;
// c: quickjs-atom.h:106
pub const JS_ATOM_undefined: u32 = 73;
// c: quickjs-atom.h:107
pub const JS_ATOM_number: u32 = 74;
// c: quickjs-atom.h:108
pub const JS_ATOM_boolean: u32 = 75;
// c: quickjs-atom.h:109
pub const JS_ATOM_string: u32 = 76;
// c: quickjs-atom.h:110
pub const JS_ATOM_object: u32 = 77;
// c: quickjs-atom.h:111
pub const JS_ATOM_symbol: u32 = 78;
// c: quickjs-atom.h:112
pub const JS_ATOM_integer: u32 = 79;
// c: quickjs-atom.h:113
pub const JS_ATOM_unknown: u32 = 80;
// c: quickjs-atom.h:114
pub const JS_ATOM_arguments: u32 = 81;
// c: quickjs-atom.h:115
pub const JS_ATOM_callee: u32 = 82;
// c: quickjs-atom.h:116
pub const JS_ATOM_caller: u32 = 83;
// c: quickjs-atom.h:117
pub const JS_ATOM__eval_: u32 = 84;
// c: quickjs-atom.h:118
pub const JS_ATOM__ret_: u32 = 85;
// c: quickjs-atom.h:119
pub const JS_ATOM__var_: u32 = 86;
// c: quickjs-atom.h:120
pub const JS_ATOM__arg_var_: u32 = 87;
// c: quickjs-atom.h:121
pub const JS_ATOM__with_: u32 = 88;
// c: quickjs-atom.h:122
pub const JS_ATOM_lastIndex: u32 = 89;
// c: quickjs-atom.h:123
pub const JS_ATOM_target: u32 = 90;
// c: quickjs-atom.h:124
pub const JS_ATOM_index: u32 = 91;
// c: quickjs-atom.h:125
pub const JS_ATOM_input: u32 = 92;
// c: quickjs-atom.h:126
pub const JS_ATOM_defineProperties: u32 = 93;
// c: quickjs-atom.h:127
pub const JS_ATOM_apply: u32 = 94;
// c: quickjs-atom.h:128
pub const JS_ATOM_join: u32 = 95;
// c: quickjs-atom.h:129
pub const JS_ATOM_concat: u32 = 96;
// c: quickjs-atom.h:130
pub const JS_ATOM_split: u32 = 97;
// c: quickjs-atom.h:131
pub const JS_ATOM_construct: u32 = 98;
// c: quickjs-atom.h:132
pub const JS_ATOM_getPrototypeOf: u32 = 99;
// c: quickjs-atom.h:133
pub const JS_ATOM_setPrototypeOf: u32 = 100;
// c: quickjs-atom.h:134
pub const JS_ATOM_isExtensible: u32 = 101;
// c: quickjs-atom.h:135
pub const JS_ATOM_preventExtensions: u32 = 102;
// c: quickjs-atom.h:136
pub const JS_ATOM_has: u32 = 103;
// c: quickjs-atom.h:137
pub const JS_ATOM_deleteProperty: u32 = 104;
// c: quickjs-atom.h:138
pub const JS_ATOM_defineProperty: u32 = 105;
// c: quickjs-atom.h:139
pub const JS_ATOM_getOwnPropertyDescriptor: u32 = 106;
// c: quickjs-atom.h:140
pub const JS_ATOM_ownKeys: u32 = 107;
// c: quickjs-atom.h:141
pub const JS_ATOM_add: u32 = 108;
// c: quickjs-atom.h:142
pub const JS_ATOM_done: u32 = 109;
// c: quickjs-atom.h:143
pub const JS_ATOM_next: u32 = 110;
// c: quickjs-atom.h:144
pub const JS_ATOM_values: u32 = 111;
// c: quickjs-atom.h:145
pub const JS_ATOM_source: u32 = 112;
// c: quickjs-atom.h:146
pub const JS_ATOM_flags: u32 = 113;
// c: quickjs-atom.h:147
pub const JS_ATOM_global: u32 = 114;
// c: quickjs-atom.h:148
pub const JS_ATOM_unicode: u32 = 115;
// c: quickjs-atom.h:149
pub const JS_ATOM_raw: u32 = 116;
// c: quickjs-atom.h:150
pub const JS_ATOM_rawJSON: u32 = 117;
// c: quickjs-atom.h:151
pub const JS_ATOM_new_target: u32 = 118;
// c: quickjs-atom.h:152
pub const JS_ATOM_this_active_func: u32 = 119;
// c: quickjs-atom.h:153
pub const JS_ATOM_home_object: u32 = 120;
// c: quickjs-atom.h:154
pub const JS_ATOM_computed_field: u32 = 121;
// c: quickjs-atom.h:155
pub const JS_ATOM_static_computed_field: u32 = 122;
// c: quickjs-atom.h:156
pub const JS_ATOM_class_fields_init: u32 = 123;
// c: quickjs-atom.h:157
pub const JS_ATOM_brand: u32 = 124;
// c: quickjs-atom.h:158
pub const JS_ATOM_hash_constructor: u32 = 125;
// c: quickjs-atom.h:159
pub const JS_ATOM_as: u32 = 126;
// c: quickjs-atom.h:160
pub const JS_ATOM_from: u32 = 127;
// c: quickjs-atom.h:161
pub const JS_ATOM_meta: u32 = 128;
// c: quickjs-atom.h:162
pub const JS_ATOM__default_: u32 = 129;
// c: quickjs-atom.h:163
pub const JS_ATOM__star_: u32 = 130;
// c: quickjs-atom.h:164
pub const JS_ATOM_Module: u32 = 131;
// c: quickjs-atom.h:165
pub const JS_ATOM_then: u32 = 132;
// c: quickjs-atom.h:166
pub const JS_ATOM_resolve: u32 = 133;
// c: quickjs-atom.h:167
pub const JS_ATOM_reject: u32 = 134;
// c: quickjs-atom.h:168
pub const JS_ATOM_promise: u32 = 135;
// c: quickjs-atom.h:169
pub const JS_ATOM_proxy: u32 = 136;
// c: quickjs-atom.h:170
pub const JS_ATOM_revoke: u32 = 137;
// c: quickjs-atom.h:171
pub const JS_ATOM_async: u32 = 138;
// c: quickjs-atom.h:172
pub const JS_ATOM_exec: u32 = 139;
// c: quickjs-atom.h:173
pub const JS_ATOM_groups: u32 = 140;
// c: quickjs-atom.h:174
pub const JS_ATOM_indices: u32 = 141;
// c: quickjs-atom.h:175
pub const JS_ATOM_status: u32 = 142;
// c: quickjs-atom.h:176
pub const JS_ATOM_reason: u32 = 143;
// c: quickjs-atom.h:177
pub const JS_ATOM_globalThis: u32 = 144;
// c: quickjs-atom.h:178
pub const JS_ATOM_bigint: u32 = 145;
// c: quickjs-atom.h:179
pub const JS_ATOM_minus_zero: u32 = 146;
// c: quickjs-atom.h:180
pub const JS_ATOM_Infinity: u32 = 147;
// c: quickjs-atom.h:181
pub const JS_ATOM_minus_Infinity: u32 = 148;
// c: quickjs-atom.h:182
pub const JS_ATOM_NaN: u32 = 149;
// c: quickjs-atom.h:183
pub const JS_ATOM_hasIndices: u32 = 150;
// c: quickjs-atom.h:184
pub const JS_ATOM_ignoreCase: u32 = 151;
// c: quickjs-atom.h:185
pub const JS_ATOM_multiline: u32 = 152;
// c: quickjs-atom.h:186
pub const JS_ATOM_dotAll: u32 = 153;
// c: quickjs-atom.h:187
pub const JS_ATOM_sticky: u32 = 154;
// c: quickjs-atom.h:188
pub const JS_ATOM_unicodeSets: u32 = 155;
// c: quickjs-atom.h:190
pub const JS_ATOM_not_equal: u32 = 156;
// c: quickjs-atom.h:191
pub const JS_ATOM_timed_out: u32 = 157;
// c: quickjs-atom.h:192
pub const JS_ATOM_ok: u32 = 158;
// c: quickjs-atom.h:193
pub const JS_ATOM_toISOString: u32 = 159;
// c: quickjs-atom.h:194
pub const JS_ATOM_alphabet: u32 = 160;
// c: quickjs-atom.h:195
pub const JS_ATOM_lastChunkHandling: u32 = 161;
// c: quickjs-atom.h:196
pub const JS_ATOM_omitPadding: u32 = 162;
// c: quickjs-atom.h:198
pub const JS_ATOM_toJSON: u32 = 163;
// c: quickjs-atom.h:199
pub const JS_ATOM_maxByteLength: u32 = 164;
// c: quickjs-atom.h:201
pub const JS_ATOM_Object: u32 = 165;
// c: quickjs-atom.h:202
pub const JS_ATOM_Array: u32 = 166;
// c: quickjs-atom.h:203
pub const JS_ATOM_Error: u32 = 167;
// c: quickjs-atom.h:204
pub const JS_ATOM_Number: u32 = 168;
// c: quickjs-atom.h:205
pub const JS_ATOM_String: u32 = 169;
// c: quickjs-atom.h:206
pub const JS_ATOM_Boolean: u32 = 170;
// c: quickjs-atom.h:207
pub const JS_ATOM_Symbol: u32 = 171;
// c: quickjs-atom.h:208
pub const JS_ATOM_Arguments: u32 = 172;
// c: quickjs-atom.h:209
pub const JS_ATOM_Math: u32 = 173;
// c: quickjs-atom.h:210
pub const JS_ATOM_JSON: u32 = 174;
// c: quickjs-atom.h:211
pub const JS_ATOM_Date: u32 = 175;
// c: quickjs-atom.h:212
pub const JS_ATOM_Function: u32 = 176;
// c: quickjs-atom.h:213
pub const JS_ATOM_GeneratorFunction: u32 = 177;
// c: quickjs-atom.h:214
pub const JS_ATOM_ForInIterator: u32 = 178;
// c: quickjs-atom.h:215
pub const JS_ATOM_RegExp: u32 = 179;
// c: quickjs-atom.h:216
pub const JS_ATOM_ArrayBuffer: u32 = 180;
// c: quickjs-atom.h:217
pub const JS_ATOM_SharedArrayBuffer: u32 = 181;
// c: quickjs-atom.h:219
pub const JS_ATOM_Uint8ClampedArray: u32 = 182;
// c: quickjs-atom.h:220
pub const JS_ATOM_Int8Array: u32 = 183;
// c: quickjs-atom.h:221
pub const JS_ATOM_Uint8Array: u32 = 184;
// c: quickjs-atom.h:222
pub const JS_ATOM_Int16Array: u32 = 185;
// c: quickjs-atom.h:223
pub const JS_ATOM_Uint16Array: u32 = 186;
// c: quickjs-atom.h:224
pub const JS_ATOM_Int32Array: u32 = 187;
// c: quickjs-atom.h:225
pub const JS_ATOM_Uint32Array: u32 = 188;
// c: quickjs-atom.h:226
pub const JS_ATOM_BigInt64Array: u32 = 189;
// c: quickjs-atom.h:227
pub const JS_ATOM_BigUint64Array: u32 = 190;
// c: quickjs-atom.h:228
pub const JS_ATOM_Float16Array: u32 = 191;
// c: quickjs-atom.h:229
pub const JS_ATOM_Float32Array: u32 = 192;
// c: quickjs-atom.h:230
pub const JS_ATOM_Float64Array: u32 = 193;
// c: quickjs-atom.h:231
pub const JS_ATOM_DataView: u32 = 194;
// c: quickjs-atom.h:232
pub const JS_ATOM_BigInt: u32 = 195;
// c: quickjs-atom.h:233
pub const JS_ATOM_WeakRef: u32 = 196;
// c: quickjs-atom.h:234
pub const JS_ATOM_FinalizationRegistry: u32 = 197;
// c: quickjs-atom.h:235
pub const JS_ATOM_Map: u32 = 198;
// c: quickjs-atom.h:236
pub const JS_ATOM_Set: u32 = 199;
// c: quickjs-atom.h:237
pub const JS_ATOM_WeakMap: u32 = 200;
// c: quickjs-atom.h:238
pub const JS_ATOM_WeakSet: u32 = 201;
// c: quickjs-atom.h:239
pub const JS_ATOM_Iterator: u32 = 202;
// c: quickjs-atom.h:240
pub const JS_ATOM_IteratorHelper: u32 = 203;
// c: quickjs-atom.h:241
pub const JS_ATOM_IteratorConcat: u32 = 204;
// c: quickjs-atom.h:242
pub const JS_ATOM_IteratorWrap: u32 = 205;
// c: quickjs-atom.h:243
pub const JS_ATOM_Map_Iterator: u32 = 206;
// c: quickjs-atom.h:244
pub const JS_ATOM_Set_Iterator: u32 = 207;
// c: quickjs-atom.h:245
pub const JS_ATOM_Array_Iterator: u32 = 208;
// c: quickjs-atom.h:246
pub const JS_ATOM_String_Iterator: u32 = 209;
// c: quickjs-atom.h:247
pub const JS_ATOM_RegExp_String_Iterator: u32 = 210;
// c: quickjs-atom.h:248
pub const JS_ATOM_Generator: u32 = 211;
// c: quickjs-atom.h:249
pub const JS_ATOM_Proxy: u32 = 212;
// c: quickjs-atom.h:250
pub const JS_ATOM_Promise: u32 = 213;
// c: quickjs-atom.h:251
pub const JS_ATOM_PromiseResolveFunction: u32 = 214;
// c: quickjs-atom.h:252
pub const JS_ATOM_PromiseRejectFunction: u32 = 215;
// c: quickjs-atom.h:253
pub const JS_ATOM_AsyncFunction: u32 = 216;
// c: quickjs-atom.h:254
pub const JS_ATOM_AsyncFunctionResolve: u32 = 217;
// c: quickjs-atom.h:255
pub const JS_ATOM_AsyncFunctionReject: u32 = 218;
// c: quickjs-atom.h:256
pub const JS_ATOM_AsyncGeneratorFunction: u32 = 219;
// c: quickjs-atom.h:257
pub const JS_ATOM_AsyncGenerator: u32 = 220;
// c: quickjs-atom.h:258
pub const JS_ATOM_EvalError: u32 = 221;
// c: quickjs-atom.h:259
pub const JS_ATOM_RangeError: u32 = 222;
// c: quickjs-atom.h:260
pub const JS_ATOM_ReferenceError: u32 = 223;
// c: quickjs-atom.h:261
pub const JS_ATOM_SyntaxError: u32 = 224;
// c: quickjs-atom.h:262
pub const JS_ATOM_TypeError: u32 = 225;
// c: quickjs-atom.h:263
pub const JS_ATOM_URIError: u32 = 226;
// c: quickjs-atom.h:264
pub const JS_ATOM_InternalError: u32 = 227;
// c: quickjs-atom.h:265
pub const JS_ATOM_AggregateError: u32 = 228;
// c: quickjs-atom.h:267
pub const JS_ATOM_Private_brand: u32 = 229;
// c: quickjs-atom.h:269
pub const JS_ATOM_Symbol_toPrimitive: u32 = 230;
// c: quickjs-atom.h:270
pub const JS_ATOM_Symbol_iterator: u32 = 231;
// c: quickjs-atom.h:271
pub const JS_ATOM_Symbol_match: u32 = 232;
// c: quickjs-atom.h:272
pub const JS_ATOM_Symbol_matchAll: u32 = 233;
// c: quickjs-atom.h:273
pub const JS_ATOM_Symbol_replace: u32 = 234;
// c: quickjs-atom.h:274
pub const JS_ATOM_Symbol_search: u32 = 235;
// c: quickjs-atom.h:275
pub const JS_ATOM_Symbol_split: u32 = 236;
// c: quickjs-atom.h:276
pub const JS_ATOM_Symbol_toStringTag: u32 = 237;
// c: quickjs-atom.h:277
pub const JS_ATOM_Symbol_isConcatSpreadable: u32 = 238;
// c: quickjs-atom.h:278
pub const JS_ATOM_Symbol_hasInstance: u32 = 239;
// c: quickjs-atom.h:279
pub const JS_ATOM_Symbol_species: u32 = 240;
// c: quickjs-atom.h:280
pub const JS_ATOM_Symbol_unscopables: u32 = 241;
// c: quickjs-atom.h:281
pub const JS_ATOM_Symbol_asyncIterator: u32 = 242;
pub const JS_ATOM_END: u32 = 243;
pub const JS_ATOM_LAST_KEYWORD: u32 = JS_ATOM_super;
pub const JS_ATOM_LAST_STRICT_KEYWORD: u32 = JS_ATOM_yield;
pub const ATOM_NAMES: [&str; 242] = [
    "null",
    "false",
    "true",
    "if",
    "else",
    "return",
    "var",
    "this",
    "delete",
    "void",
    "typeof",
    "new",
    "in",
    "instanceof",
    "do",
    "while",
    "for",
    "break",
    "continue",
    "switch",
    "case",
    "default",
    "throw",
    "try",
    "catch",
    "finally",
    "function",
    "debugger",
    "with",
    "class",
    "const",
    "enum",
    "export",
    "extends",
    "import",
    "super",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
    "await",
    "",
    "keys",
    "size",
    "length",
    "fileName",
    "lineNumber",
    "columnNumber",
    "message",
    "cause",
    "errors",
    "stack",
    "name",
    "toString",
    "toLocaleString",
    "valueOf",
    "eval",
    "prototype",
    "constructor",
    "configurable",
    "writable",
    "enumerable",
    "value",
    "get",
    "set",
    "of",
    "__proto__",
    "undefined",
    "number",
    "boolean",
    "string",
    "object",
    "symbol",
    "integer",
    "unknown",
    "arguments",
    "callee",
    "caller",
    "<eval>",
    "<ret>",
    "<var>",
    "<arg_var>",
    "<with>",
    "lastIndex",
    "target",
    "index",
    "input",
    "defineProperties",
    "apply",
    "join",
    "concat",
    "split",
    "construct",
    "getPrototypeOf",
    "setPrototypeOf",
    "isExtensible",
    "preventExtensions",
    "has",
    "deleteProperty",
    "defineProperty",
    "getOwnPropertyDescriptor",
    "ownKeys",
    "add",
    "done",
    "next",
    "values",
    "source",
    "flags",
    "global",
    "unicode",
    "raw",
    "rawJSON",
    "new.target",
    "this.active_func",
    "<home_object>",
    "<computed_field>",
    "<static_computed_field>",
    "<class_fields_init>",
    "<brand>",
    "#constructor",
    "as",
    "from",
    "meta",
    "*default*",
    "*",
    "Module",
    "then",
    "resolve",
    "reject",
    "promise",
    "proxy",
    "revoke",
    "async",
    "exec",
    "groups",
    "indices",
    "status",
    "reason",
    "globalThis",
    "bigint",
    "-0",
    "Infinity",
    "-Infinity",
    "NaN",
    "hasIndices",
    "ignoreCase",
    "multiline",
    "dotAll",
    "sticky",
    "unicodeSets",
    "not-equal",
    "timed-out",
    "ok",
    "toISOString",
    "alphabet",
    "lastChunkHandling",
    "omitPadding",
    "toJSON",
    "maxByteLength",
    "Object",
    "Array",
    "Error",
    "Number",
    "String",
    "Boolean",
    "Symbol",
    "Arguments",
    "Math",
    "JSON",
    "Date",
    "Function",
    "GeneratorFunction",
    "ForInIterator",
    "RegExp",
    "ArrayBuffer",
    "SharedArrayBuffer",
    "Uint8ClampedArray",
    "Int8Array",
    "Uint8Array",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "BigInt64Array",
    "BigUint64Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "DataView",
    "BigInt",
    "WeakRef",
    "FinalizationRegistry",
    "Map",
    "Set",
    "WeakMap",
    "WeakSet",
    "Iterator",
    "Iterator Helper",
    "Iterator Concat",
    "Iterator Wrap",
    "Map Iterator",
    "Set Iterator",
    "Array Iterator",
    "String Iterator",
    "RegExp String Iterator",
    "Generator",
    "Proxy",
    "Promise",
    "PromiseResolveFunction",
    "PromiseRejectFunction",
    "AsyncFunction",
    "AsyncFunctionResolve",
    "AsyncFunctionReject",
    "AsyncGeneratorFunction",
    "AsyncGenerator",
    "EvalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError",
    "InternalError",
    "AggregateError",
    "<brand>",
    "Symbol.toPrimitive",
    "Symbol.iterator",
    "Symbol.match",
    "Symbol.matchAll",
    "Symbol.replace",
    "Symbol.search",
    "Symbol.split",
    "Symbol.toStringTag",
    "Symbol.isConcatSpreadable",
    "Symbol.hasInstance",
    "Symbol.species",
    "Symbol.unscopables",
    "Symbol.asyncIterator",
];
pub static js_atom_init: [u8; 2321] = [
    110, 117, 108, 108, 0, 102, 97, 108, 115, 101, 0, 116, 114, 117, 101, 0, 105, 102, 0, 101, 108,
    115, 101, 0, 114, 101, 116, 117, 114, 110, 0, 118, 97, 114, 0, 116, 104, 105, 115, 0, 100, 101,
    108, 101, 116, 101, 0, 118, 111, 105, 100, 0, 116, 121, 112, 101, 111, 102, 0, 110, 101, 119,
    0, 105, 110, 0, 105, 110, 115, 116, 97, 110, 99, 101, 111, 102, 0, 100, 111, 0, 119, 104, 105,
    108, 101, 0, 102, 111, 114, 0, 98, 114, 101, 97, 107, 0, 99, 111, 110, 116, 105, 110, 117, 101,
    0, 115, 119, 105, 116, 99, 104, 0, 99, 97, 115, 101, 0, 100, 101, 102, 97, 117, 108, 116, 0,
    116, 104, 114, 111, 119, 0, 116, 114, 121, 0, 99, 97, 116, 99, 104, 0, 102, 105, 110, 97, 108,
    108, 121, 0, 102, 117, 110, 99, 116, 105, 111, 110, 0, 100, 101, 98, 117, 103, 103, 101, 114,
    0, 119, 105, 116, 104, 0, 99, 108, 97, 115, 115, 0, 99, 111, 110, 115, 116, 0, 101, 110, 117,
    109, 0, 101, 120, 112, 111, 114, 116, 0, 101, 120, 116, 101, 110, 100, 115, 0, 105, 109, 112,
    111, 114, 116, 0, 115, 117, 112, 101, 114, 0, 105, 109, 112, 108, 101, 109, 101, 110, 116, 115,
    0, 105, 110, 116, 101, 114, 102, 97, 99, 101, 0, 108, 101, 116, 0, 112, 97, 99, 107, 97, 103,
    101, 0, 112, 114, 105, 118, 97, 116, 101, 0, 112, 114, 111, 116, 101, 99, 116, 101, 100, 0,
    112, 117, 98, 108, 105, 99, 0, 115, 116, 97, 116, 105, 99, 0, 121, 105, 101, 108, 100, 0, 97,
    119, 97, 105, 116, 0, 0, 107, 101, 121, 115, 0, 115, 105, 122, 101, 0, 108, 101, 110, 103, 116,
    104, 0, 102, 105, 108, 101, 78, 97, 109, 101, 0, 108, 105, 110, 101, 78, 117, 109, 98, 101,
    114, 0, 99, 111, 108, 117, 109, 110, 78, 117, 109, 98, 101, 114, 0, 109, 101, 115, 115, 97,
    103, 101, 0, 99, 97, 117, 115, 101, 0, 101, 114, 114, 111, 114, 115, 0, 115, 116, 97, 99, 107,
    0, 110, 97, 109, 101, 0, 116, 111, 83, 116, 114, 105, 110, 103, 0, 116, 111, 76, 111, 99, 97,
    108, 101, 83, 116, 114, 105, 110, 103, 0, 118, 97, 108, 117, 101, 79, 102, 0, 101, 118, 97,
    108, 0, 112, 114, 111, 116, 111, 116, 121, 112, 101, 0, 99, 111, 110, 115, 116, 114, 117, 99,
    116, 111, 114, 0, 99, 111, 110, 102, 105, 103, 117, 114, 97, 98, 108, 101, 0, 119, 114, 105,
    116, 97, 98, 108, 101, 0, 101, 110, 117, 109, 101, 114, 97, 98, 108, 101, 0, 118, 97, 108, 117,
    101, 0, 103, 101, 116, 0, 115, 101, 116, 0, 111, 102, 0, 95, 95, 112, 114, 111, 116, 111, 95,
    95, 0, 117, 110, 100, 101, 102, 105, 110, 101, 100, 0, 110, 117, 109, 98, 101, 114, 0, 98, 111,
    111, 108, 101, 97, 110, 0, 115, 116, 114, 105, 110, 103, 0, 111, 98, 106, 101, 99, 116, 0, 115,
    121, 109, 98, 111, 108, 0, 105, 110, 116, 101, 103, 101, 114, 0, 117, 110, 107, 110, 111, 119,
    110, 0, 97, 114, 103, 117, 109, 101, 110, 116, 115, 0, 99, 97, 108, 108, 101, 101, 0, 99, 97,
    108, 108, 101, 114, 0, 60, 101, 118, 97, 108, 62, 0, 60, 114, 101, 116, 62, 0, 60, 118, 97,
    114, 62, 0, 60, 97, 114, 103, 95, 118, 97, 114, 62, 0, 60, 119, 105, 116, 104, 62, 0, 108, 97,
    115, 116, 73, 110, 100, 101, 120, 0, 116, 97, 114, 103, 101, 116, 0, 105, 110, 100, 101, 120,
    0, 105, 110, 112, 117, 116, 0, 100, 101, 102, 105, 110, 101, 80, 114, 111, 112, 101, 114, 116,
    105, 101, 115, 0, 97, 112, 112, 108, 121, 0, 106, 111, 105, 110, 0, 99, 111, 110, 99, 97, 116,
    0, 115, 112, 108, 105, 116, 0, 99, 111, 110, 115, 116, 114, 117, 99, 116, 0, 103, 101, 116, 80,
    114, 111, 116, 111, 116, 121, 112, 101, 79, 102, 0, 115, 101, 116, 80, 114, 111, 116, 111, 116,
    121, 112, 101, 79, 102, 0, 105, 115, 69, 120, 116, 101, 110, 115, 105, 98, 108, 101, 0, 112,
    114, 101, 118, 101, 110, 116, 69, 120, 116, 101, 110, 115, 105, 111, 110, 115, 0, 104, 97, 115,
    0, 100, 101, 108, 101, 116, 101, 80, 114, 111, 112, 101, 114, 116, 121, 0, 100, 101, 102, 105,
    110, 101, 80, 114, 111, 112, 101, 114, 116, 121, 0, 103, 101, 116, 79, 119, 110, 80, 114, 111,
    112, 101, 114, 116, 121, 68, 101, 115, 99, 114, 105, 112, 116, 111, 114, 0, 111, 119, 110, 75,
    101, 121, 115, 0, 97, 100, 100, 0, 100, 111, 110, 101, 0, 110, 101, 120, 116, 0, 118, 97, 108,
    117, 101, 115, 0, 115, 111, 117, 114, 99, 101, 0, 102, 108, 97, 103, 115, 0, 103, 108, 111, 98,
    97, 108, 0, 117, 110, 105, 99, 111, 100, 101, 0, 114, 97, 119, 0, 114, 97, 119, 74, 83, 79, 78,
    0, 110, 101, 119, 46, 116, 97, 114, 103, 101, 116, 0, 116, 104, 105, 115, 46, 97, 99, 116, 105,
    118, 101, 95, 102, 117, 110, 99, 0, 60, 104, 111, 109, 101, 95, 111, 98, 106, 101, 99, 116, 62,
    0, 60, 99, 111, 109, 112, 117, 116, 101, 100, 95, 102, 105, 101, 108, 100, 62, 0, 60, 115, 116,
    97, 116, 105, 99, 95, 99, 111, 109, 112, 117, 116, 101, 100, 95, 102, 105, 101, 108, 100, 62,
    0, 60, 99, 108, 97, 115, 115, 95, 102, 105, 101, 108, 100, 115, 95, 105, 110, 105, 116, 62, 0,
    60, 98, 114, 97, 110, 100, 62, 0, 35, 99, 111, 110, 115, 116, 114, 117, 99, 116, 111, 114, 0,
    97, 115, 0, 102, 114, 111, 109, 0, 109, 101, 116, 97, 0, 42, 100, 101, 102, 97, 117, 108, 116,
    42, 0, 42, 0, 77, 111, 100, 117, 108, 101, 0, 116, 104, 101, 110, 0, 114, 101, 115, 111, 108,
    118, 101, 0, 114, 101, 106, 101, 99, 116, 0, 112, 114, 111, 109, 105, 115, 101, 0, 112, 114,
    111, 120, 121, 0, 114, 101, 118, 111, 107, 101, 0, 97, 115, 121, 110, 99, 0, 101, 120, 101, 99,
    0, 103, 114, 111, 117, 112, 115, 0, 105, 110, 100, 105, 99, 101, 115, 0, 115, 116, 97, 116,
    117, 115, 0, 114, 101, 97, 115, 111, 110, 0, 103, 108, 111, 98, 97, 108, 84, 104, 105, 115, 0,
    98, 105, 103, 105, 110, 116, 0, 45, 48, 0, 73, 110, 102, 105, 110, 105, 116, 121, 0, 45, 73,
    110, 102, 105, 110, 105, 116, 121, 0, 78, 97, 78, 0, 104, 97, 115, 73, 110, 100, 105, 99, 101,
    115, 0, 105, 103, 110, 111, 114, 101, 67, 97, 115, 101, 0, 109, 117, 108, 116, 105, 108, 105,
    110, 101, 0, 100, 111, 116, 65, 108, 108, 0, 115, 116, 105, 99, 107, 121, 0, 117, 110, 105, 99,
    111, 100, 101, 83, 101, 116, 115, 0, 110, 111, 116, 45, 101, 113, 117, 97, 108, 0, 116, 105,
    109, 101, 100, 45, 111, 117, 116, 0, 111, 107, 0, 116, 111, 73, 83, 79, 83, 116, 114, 105, 110,
    103, 0, 97, 108, 112, 104, 97, 98, 101, 116, 0, 108, 97, 115, 116, 67, 104, 117, 110, 107, 72,
    97, 110, 100, 108, 105, 110, 103, 0, 111, 109, 105, 116, 80, 97, 100, 100, 105, 110, 103, 0,
    116, 111, 74, 83, 79, 78, 0, 109, 97, 120, 66, 121, 116, 101, 76, 101, 110, 103, 116, 104, 0,
    79, 98, 106, 101, 99, 116, 0, 65, 114, 114, 97, 121, 0, 69, 114, 114, 111, 114, 0, 78, 117,
    109, 98, 101, 114, 0, 83, 116, 114, 105, 110, 103, 0, 66, 111, 111, 108, 101, 97, 110, 0, 83,
    121, 109, 98, 111, 108, 0, 65, 114, 103, 117, 109, 101, 110, 116, 115, 0, 77, 97, 116, 104, 0,
    74, 83, 79, 78, 0, 68, 97, 116, 101, 0, 70, 117, 110, 99, 116, 105, 111, 110, 0, 71, 101, 110,
    101, 114, 97, 116, 111, 114, 70, 117, 110, 99, 116, 105, 111, 110, 0, 70, 111, 114, 73, 110,
    73, 116, 101, 114, 97, 116, 111, 114, 0, 82, 101, 103, 69, 120, 112, 0, 65, 114, 114, 97, 121,
    66, 117, 102, 102, 101, 114, 0, 83, 104, 97, 114, 101, 100, 65, 114, 114, 97, 121, 66, 117,
    102, 102, 101, 114, 0, 85, 105, 110, 116, 56, 67, 108, 97, 109, 112, 101, 100, 65, 114, 114,
    97, 121, 0, 73, 110, 116, 56, 65, 114, 114, 97, 121, 0, 85, 105, 110, 116, 56, 65, 114, 114,
    97, 121, 0, 73, 110, 116, 49, 54, 65, 114, 114, 97, 121, 0, 85, 105, 110, 116, 49, 54, 65, 114,
    114, 97, 121, 0, 73, 110, 116, 51, 50, 65, 114, 114, 97, 121, 0, 85, 105, 110, 116, 51, 50, 65,
    114, 114, 97, 121, 0, 66, 105, 103, 73, 110, 116, 54, 52, 65, 114, 114, 97, 121, 0, 66, 105,
    103, 85, 105, 110, 116, 54, 52, 65, 114, 114, 97, 121, 0, 70, 108, 111, 97, 116, 49, 54, 65,
    114, 114, 97, 121, 0, 70, 108, 111, 97, 116, 51, 50, 65, 114, 114, 97, 121, 0, 70, 108, 111,
    97, 116, 54, 52, 65, 114, 114, 97, 121, 0, 68, 97, 116, 97, 86, 105, 101, 119, 0, 66, 105, 103,
    73, 110, 116, 0, 87, 101, 97, 107, 82, 101, 102, 0, 70, 105, 110, 97, 108, 105, 122, 97, 116,
    105, 111, 110, 82, 101, 103, 105, 115, 116, 114, 121, 0, 77, 97, 112, 0, 83, 101, 116, 0, 87,
    101, 97, 107, 77, 97, 112, 0, 87, 101, 97, 107, 83, 101, 116, 0, 73, 116, 101, 114, 97, 116,
    111, 114, 0, 73, 116, 101, 114, 97, 116, 111, 114, 32, 72, 101, 108, 112, 101, 114, 0, 73, 116,
    101, 114, 97, 116, 111, 114, 32, 67, 111, 110, 99, 97, 116, 0, 73, 116, 101, 114, 97, 116, 111,
    114, 32, 87, 114, 97, 112, 0, 77, 97, 112, 32, 73, 116, 101, 114, 97, 116, 111, 114, 0, 83,
    101, 116, 32, 73, 116, 101, 114, 97, 116, 111, 114, 0, 65, 114, 114, 97, 121, 32, 73, 116, 101,
    114, 97, 116, 111, 114, 0, 83, 116, 114, 105, 110, 103, 32, 73, 116, 101, 114, 97, 116, 111,
    114, 0, 82, 101, 103, 69, 120, 112, 32, 83, 116, 114, 105, 110, 103, 32, 73, 116, 101, 114, 97,
    116, 111, 114, 0, 71, 101, 110, 101, 114, 97, 116, 111, 114, 0, 80, 114, 111, 120, 121, 0, 80,
    114, 111, 109, 105, 115, 101, 0, 80, 114, 111, 109, 105, 115, 101, 82, 101, 115, 111, 108, 118,
    101, 70, 117, 110, 99, 116, 105, 111, 110, 0, 80, 114, 111, 109, 105, 115, 101, 82, 101, 106,
    101, 99, 116, 70, 117, 110, 99, 116, 105, 111, 110, 0, 65, 115, 121, 110, 99, 70, 117, 110, 99,
    116, 105, 111, 110, 0, 65, 115, 121, 110, 99, 70, 117, 110, 99, 116, 105, 111, 110, 82, 101,
    115, 111, 108, 118, 101, 0, 65, 115, 121, 110, 99, 70, 117, 110, 99, 116, 105, 111, 110, 82,
    101, 106, 101, 99, 116, 0, 65, 115, 121, 110, 99, 71, 101, 110, 101, 114, 97, 116, 111, 114,
    70, 117, 110, 99, 116, 105, 111, 110, 0, 65, 115, 121, 110, 99, 71, 101, 110, 101, 114, 97,
    116, 111, 114, 0, 69, 118, 97, 108, 69, 114, 114, 111, 114, 0, 82, 97, 110, 103, 101, 69, 114,
    114, 111, 114, 0, 82, 101, 102, 101, 114, 101, 110, 99, 101, 69, 114, 114, 111, 114, 0, 83,
    121, 110, 116, 97, 120, 69, 114, 114, 111, 114, 0, 84, 121, 112, 101, 69, 114, 114, 111, 114,
    0, 85, 82, 73, 69, 114, 114, 111, 114, 0, 73, 110, 116, 101, 114, 110, 97, 108, 69, 114, 114,
    111, 114, 0, 65, 103, 103, 114, 101, 103, 97, 116, 101, 69, 114, 114, 111, 114, 0, 60, 98, 114,
    97, 110, 100, 62, 0, 83, 121, 109, 98, 111, 108, 46, 116, 111, 80, 114, 105, 109, 105, 116,
    105, 118, 101, 0, 83, 121, 109, 98, 111, 108, 46, 105, 116, 101, 114, 97, 116, 111, 114, 0, 83,
    121, 109, 98, 111, 108, 46, 109, 97, 116, 99, 104, 0, 83, 121, 109, 98, 111, 108, 46, 109, 97,
    116, 99, 104, 65, 108, 108, 0, 83, 121, 109, 98, 111, 108, 46, 114, 101, 112, 108, 97, 99, 101,
    0, 83, 121, 109, 98, 111, 108, 46, 115, 101, 97, 114, 99, 104, 0, 83, 121, 109, 98, 111, 108,
    46, 115, 112, 108, 105, 116, 0, 83, 121, 109, 98, 111, 108, 46, 116, 111, 83, 116, 114, 105,
    110, 103, 84, 97, 103, 0, 83, 121, 109, 98, 111, 108, 46, 105, 115, 67, 111, 110, 99, 97, 116,
    83, 112, 114, 101, 97, 100, 97, 98, 108, 101, 0, 83, 121, 109, 98, 111, 108, 46, 104, 97, 115,
    73, 110, 115, 116, 97, 110, 99, 101, 0, 83, 121, 109, 98, 111, 108, 46, 115, 112, 101, 99, 105,
    101, 115, 0, 83, 121, 109, 98, 111, 108, 46, 117, 110, 115, 99, 111, 112, 97, 98, 108, 101,
    115, 0, 83, 121, 109, 98, 111, 108, 46, 97, 115, 121, 110, 99, 73, 116, 101, 114, 97, 116, 111,
    114, 0, 0,
];
