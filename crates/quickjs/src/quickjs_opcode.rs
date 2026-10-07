/*
 * QuickJS opcode definitions
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
// Direct table translation of quickjs-opcode.h.
pub type OPCodeFormat = i32;
pub const OP_FMT_none: OPCodeFormat = 0;
pub const OP_FMT_none_int: OPCodeFormat = 1;
pub const OP_FMT_none_loc: OPCodeFormat = 2;
pub const OP_FMT_none_arg: OPCodeFormat = 3;
pub const OP_FMT_none_var_ref: OPCodeFormat = 4;
pub const OP_FMT_u8: OPCodeFormat = 5;
pub const OP_FMT_i8: OPCodeFormat = 6;
pub const OP_FMT_loc8: OPCodeFormat = 7;
pub const OP_FMT_const8: OPCodeFormat = 8;
pub const OP_FMT_label8: OPCodeFormat = 9;
pub const OP_FMT_u16: OPCodeFormat = 10;
pub const OP_FMT_i16: OPCodeFormat = 11;
pub const OP_FMT_label16: OPCodeFormat = 12;
pub const OP_FMT_npop: OPCodeFormat = 13;
pub const OP_FMT_npopx: OPCodeFormat = 14;
pub const OP_FMT_npop_u16: OPCodeFormat = 15;
pub const OP_FMT_loc: OPCodeFormat = 16;
pub const OP_FMT_arg: OPCodeFormat = 17;
pub const OP_FMT_var_ref: OPCodeFormat = 18;
pub const OP_FMT_u32: OPCodeFormat = 19;
pub const OP_FMT_i32: OPCodeFormat = 20;
pub const OP_FMT_const: OPCodeFormat = 21;
pub const OP_FMT_label: OPCodeFormat = 22;
pub const OP_FMT_atom: OPCodeFormat = 23;
pub const OP_FMT_atom_u8: OPCodeFormat = 24;
pub const OP_FMT_atom_u16: OPCodeFormat = 25;
pub const OP_FMT_atom_label_u8: OPCodeFormat = 26;
pub const OP_FMT_atom_label_u16: OPCodeFormat = 27;
pub const OP_FMT_label_u16: OPCodeFormat = 28;
// c: quickjs-opcode.h:65
pub const OP_invalid: u16 = 0;
// c: quickjs-opcode.h:68
pub const OP_push_i32: u16 = 1;
// c: quickjs-opcode.h:69
pub const OP_push_const: u16 = 2;
// c: quickjs-opcode.h:70
pub const OP_fclosure: u16 = 3;
// c: quickjs-opcode.h:71
pub const OP_push_atom_value: u16 = 4;
// c: quickjs-opcode.h:72
pub const OP_private_symbol: u16 = 5;
// c: quickjs-opcode.h:73
pub const OP_undefined: u16 = 6;
// c: quickjs-opcode.h:74
pub const OP_null: u16 = 7;
// c: quickjs-opcode.h:75
pub const OP_push_this: u16 = 8;
// c: quickjs-opcode.h:76
pub const OP_push_false: u16 = 9;
// c: quickjs-opcode.h:77
pub const OP_push_true: u16 = 10;
// c: quickjs-opcode.h:78
pub const OP_object: u16 = 11;
// c: quickjs-opcode.h:79
pub const OP_special_object: u16 = 12;
// c: quickjs-opcode.h:80
pub const OP_rest: u16 = 13;
// c: quickjs-opcode.h:82
pub const OP_drop: u16 = 14;
// c: quickjs-opcode.h:83
pub const OP_nip: u16 = 15;
// c: quickjs-opcode.h:84
pub const OP_nip1: u16 = 16;
// c: quickjs-opcode.h:85
pub const OP_dup: u16 = 17;
// c: quickjs-opcode.h:86
pub const OP_dup1: u16 = 18;
// c: quickjs-opcode.h:87
pub const OP_dup2: u16 = 19;
// c: quickjs-opcode.h:88
pub const OP_dup3: u16 = 20;
// c: quickjs-opcode.h:89
pub const OP_insert2: u16 = 21;
// c: quickjs-opcode.h:90
pub const OP_insert3: u16 = 22;
// c: quickjs-opcode.h:91
pub const OP_insert4: u16 = 23;
// c: quickjs-opcode.h:92
pub const OP_perm3: u16 = 24;
// c: quickjs-opcode.h:93
pub const OP_perm4: u16 = 25;
// c: quickjs-opcode.h:94
pub const OP_perm5: u16 = 26;
// c: quickjs-opcode.h:95
pub const OP_swap: u16 = 27;
// c: quickjs-opcode.h:96
pub const OP_swap2: u16 = 28;
// c: quickjs-opcode.h:97
pub const OP_rot3l: u16 = 29;
// c: quickjs-opcode.h:98
pub const OP_rot3r: u16 = 30;
// c: quickjs-opcode.h:99
pub const OP_rot4l: u16 = 31;
// c: quickjs-opcode.h:100
pub const OP_rot5l: u16 = 32;
// c: quickjs-opcode.h:102
pub const OP_call_constructor: u16 = 33;
// c: quickjs-opcode.h:103
pub const OP_call: u16 = 34;
// c: quickjs-opcode.h:104
pub const OP_tail_call: u16 = 35;
// c: quickjs-opcode.h:105
pub const OP_call_method: u16 = 36;
// c: quickjs-opcode.h:106
pub const OP_tail_call_method: u16 = 37;
// c: quickjs-opcode.h:107
pub const OP_array_from: u16 = 38;
// c: quickjs-opcode.h:108
pub const OP_apply: u16 = 39;
// c: quickjs-opcode.h:109
pub const OP_return: u16 = 40;
// c: quickjs-opcode.h:110
pub const OP_return_undef: u16 = 41;
// c: quickjs-opcode.h:111
pub const OP_check_ctor_return: u16 = 42;
// c: quickjs-opcode.h:112
pub const OP_check_ctor: u16 = 43;
// c: quickjs-opcode.h:113
pub const OP_init_ctor: u16 = 44;
// c: quickjs-opcode.h:114
pub const OP_check_brand: u16 = 45;
// c: quickjs-opcode.h:115
pub const OP_add_brand: u16 = 46;
// c: quickjs-opcode.h:116
pub const OP_return_async: u16 = 47;
// c: quickjs-opcode.h:117
pub const OP_throw: u16 = 48;
// c: quickjs-opcode.h:118
pub const OP_throw_error: u16 = 49;
// c: quickjs-opcode.h:119
pub const OP_eval: u16 = 50;
// c: quickjs-opcode.h:120
pub const OP_apply_eval: u16 = 51;
// c: quickjs-opcode.h:121
pub const OP_regexp: u16 = 52;
// c: quickjs-opcode.h:123
pub const OP_get_super: u16 = 53;
// c: quickjs-opcode.h:124
pub const OP_import: u16 = 54;
// c: quickjs-opcode.h:126
pub const OP_get_var_undef: u16 = 55;
// c: quickjs-opcode.h:127
pub const OP_get_var: u16 = 56;
// c: quickjs-opcode.h:128
pub const OP_put_var: u16 = 57;
// c: quickjs-opcode.h:129
pub const OP_put_var_init: u16 = 58;
// c: quickjs-opcode.h:131
pub const OP_get_ref_value: u16 = 59;
// c: quickjs-opcode.h:132
pub const OP_put_ref_value: u16 = 60;
// c: quickjs-opcode.h:134
pub const OP_get_field: u16 = 61;
// c: quickjs-opcode.h:135
pub const OP_get_field2: u16 = 62;
// c: quickjs-opcode.h:136
pub const OP_put_field: u16 = 63;
// c: quickjs-opcode.h:137
pub const OP_get_private_field: u16 = 64;
// c: quickjs-opcode.h:138
pub const OP_put_private_field: u16 = 65;
// c: quickjs-opcode.h:139
pub const OP_define_private_field: u16 = 66;
// c: quickjs-opcode.h:140
pub const OP_get_array_el: u16 = 67;
// c: quickjs-opcode.h:141
pub const OP_get_array_el2: u16 = 68;
// c: quickjs-opcode.h:142
pub const OP_get_array_el3: u16 = 69;
// c: quickjs-opcode.h:143
pub const OP_put_array_el: u16 = 70;
// c: quickjs-opcode.h:144
pub const OP_get_super_value: u16 = 71;
// c: quickjs-opcode.h:145
pub const OP_put_super_value: u16 = 72;
// c: quickjs-opcode.h:146
pub const OP_define_field: u16 = 73;
// c: quickjs-opcode.h:147
pub const OP_set_name: u16 = 74;
// c: quickjs-opcode.h:148
pub const OP_set_name_computed: u16 = 75;
// c: quickjs-opcode.h:149
pub const OP_set_proto: u16 = 76;
// c: quickjs-opcode.h:150
pub const OP_set_home_object: u16 = 77;
// c: quickjs-opcode.h:151
pub const OP_define_array_el: u16 = 78;
// c: quickjs-opcode.h:152
pub const OP_append: u16 = 79;
// c: quickjs-opcode.h:153
pub const OP_copy_data_properties: u16 = 80;
// c: quickjs-opcode.h:154
pub const OP_define_method: u16 = 81;
// c: quickjs-opcode.h:155
pub const OP_define_method_computed: u16 = 82;
// c: quickjs-opcode.h:156
pub const OP_define_class: u16 = 83;
// c: quickjs-opcode.h:157
pub const OP_define_class_computed: u16 = 84;
// c: quickjs-opcode.h:159
pub const OP_get_loc: u16 = 85;
// c: quickjs-opcode.h:160
pub const OP_put_loc: u16 = 86;
// c: quickjs-opcode.h:161
pub const OP_set_loc: u16 = 87;
// c: quickjs-opcode.h:162
pub const OP_get_arg: u16 = 88;
// c: quickjs-opcode.h:163
pub const OP_put_arg: u16 = 89;
// c: quickjs-opcode.h:164
pub const OP_set_arg: u16 = 90;
// c: quickjs-opcode.h:165
pub const OP_get_var_ref: u16 = 91;
// c: quickjs-opcode.h:166
pub const OP_put_var_ref: u16 = 92;
// c: quickjs-opcode.h:167
pub const OP_set_var_ref: u16 = 93;
// c: quickjs-opcode.h:168
pub const OP_set_loc_uninitialized: u16 = 94;
// c: quickjs-opcode.h:169
pub const OP_get_loc_check: u16 = 95;
// c: quickjs-opcode.h:170
pub const OP_put_loc_check: u16 = 96;
// c: quickjs-opcode.h:171
pub const OP_set_loc_check: u16 = 97;
// c: quickjs-opcode.h:172
pub const OP_put_loc_check_init: u16 = 98;
// c: quickjs-opcode.h:173
pub const OP_get_loc_checkthis: u16 = 99;
// c: quickjs-opcode.h:174
pub const OP_get_var_ref_check: u16 = 100;
// c: quickjs-opcode.h:175
pub const OP_put_var_ref_check: u16 = 101;
// c: quickjs-opcode.h:176
pub const OP_put_var_ref_check_init: u16 = 102;
// c: quickjs-opcode.h:177
pub const OP_close_loc: u16 = 103;
// c: quickjs-opcode.h:178
pub const OP_if_false: u16 = 104;
// c: quickjs-opcode.h:179
pub const OP_if_true: u16 = 105;
// c: quickjs-opcode.h:180
pub const OP_goto: u16 = 106;
// c: quickjs-opcode.h:181
pub const OP_catch: u16 = 107;
// c: quickjs-opcode.h:182
pub const OP_gosub: u16 = 108;
// c: quickjs-opcode.h:183
pub const OP_ret: u16 = 109;
// c: quickjs-opcode.h:184
pub const OP_nip_catch: u16 = 110;
// c: quickjs-opcode.h:186
pub const OP_to_object: u16 = 111;
// c: quickjs-opcode.h:188
pub const OP_to_propkey: u16 = 112;
// c: quickjs-opcode.h:190
pub const OP_with_get_var: u16 = 113;
// c: quickjs-opcode.h:191
pub const OP_with_put_var: u16 = 114;
// c: quickjs-opcode.h:192
pub const OP_with_delete_var: u16 = 115;
// c: quickjs-opcode.h:193
pub const OP_with_make_ref: u16 = 116;
// c: quickjs-opcode.h:194
pub const OP_with_get_ref: u16 = 117;
// c: quickjs-opcode.h:196
pub const OP_make_loc_ref: u16 = 118;
// c: quickjs-opcode.h:197
pub const OP_make_arg_ref: u16 = 119;
// c: quickjs-opcode.h:198
pub const OP_make_var_ref_ref: u16 = 120;
// c: quickjs-opcode.h:199
pub const OP_make_var_ref: u16 = 121;
// c: quickjs-opcode.h:201
pub const OP_for_in_start: u16 = 122;
// c: quickjs-opcode.h:202
pub const OP_for_of_start: u16 = 123;
// c: quickjs-opcode.h:203
pub const OP_for_await_of_start: u16 = 124;
// c: quickjs-opcode.h:204
pub const OP_for_in_next: u16 = 125;
// c: quickjs-opcode.h:205
pub const OP_for_of_next: u16 = 126;
// c: quickjs-opcode.h:206
pub const OP_for_await_of_next: u16 = 127;
// c: quickjs-opcode.h:207
pub const OP_iterator_check_object: u16 = 128;
// c: quickjs-opcode.h:208
pub const OP_iterator_get_value_done: u16 = 129;
// c: quickjs-opcode.h:209
pub const OP_iterator_close: u16 = 130;
// c: quickjs-opcode.h:210
pub const OP_iterator_next: u16 = 131;
// c: quickjs-opcode.h:211
pub const OP_iterator_call: u16 = 132;
// c: quickjs-opcode.h:212
pub const OP_initial_yield: u16 = 133;
// c: quickjs-opcode.h:213
pub const OP_yield: u16 = 134;
// c: quickjs-opcode.h:214
pub const OP_yield_star: u16 = 135;
// c: quickjs-opcode.h:215
pub const OP_async_yield_star: u16 = 136;
// c: quickjs-opcode.h:216
pub const OP_await: u16 = 137;
// c: quickjs-opcode.h:219
pub const OP_neg: u16 = 138;
// c: quickjs-opcode.h:220
pub const OP_plus: u16 = 139;
// c: quickjs-opcode.h:221
pub const OP_dec: u16 = 140;
// c: quickjs-opcode.h:222
pub const OP_inc: u16 = 141;
// c: quickjs-opcode.h:223
pub const OP_post_dec: u16 = 142;
// c: quickjs-opcode.h:224
pub const OP_post_inc: u16 = 143;
// c: quickjs-opcode.h:225
pub const OP_dec_loc: u16 = 144;
// c: quickjs-opcode.h:226
pub const OP_inc_loc: u16 = 145;
// c: quickjs-opcode.h:227
pub const OP_add_loc: u16 = 146;
// c: quickjs-opcode.h:228
pub const OP_not: u16 = 147;
// c: quickjs-opcode.h:229
pub const OP_lnot: u16 = 148;
// c: quickjs-opcode.h:230
pub const OP_typeof: u16 = 149;
// c: quickjs-opcode.h:231
pub const OP_delete: u16 = 150;
// c: quickjs-opcode.h:232
pub const OP_delete_var: u16 = 151;
// c: quickjs-opcode.h:234
pub const OP_mul: u16 = 152;
// c: quickjs-opcode.h:235
pub const OP_div: u16 = 153;
// c: quickjs-opcode.h:236
pub const OP_mod: u16 = 154;
// c: quickjs-opcode.h:237
pub const OP_add: u16 = 155;
// c: quickjs-opcode.h:238
pub const OP_sub: u16 = 156;
// c: quickjs-opcode.h:239
pub const OP_pow: u16 = 157;
// c: quickjs-opcode.h:240
pub const OP_shl: u16 = 158;
// c: quickjs-opcode.h:241
pub const OP_sar: u16 = 159;
// c: quickjs-opcode.h:242
pub const OP_shr: u16 = 160;
// c: quickjs-opcode.h:243
pub const OP_lt: u16 = 161;
// c: quickjs-opcode.h:244
pub const OP_lte: u16 = 162;
// c: quickjs-opcode.h:245
pub const OP_gt: u16 = 163;
// c: quickjs-opcode.h:246
pub const OP_gte: u16 = 164;
// c: quickjs-opcode.h:247
pub const OP_instanceof: u16 = 165;
// c: quickjs-opcode.h:248
pub const OP_in: u16 = 166;
// c: quickjs-opcode.h:249
pub const OP_eq: u16 = 167;
// c: quickjs-opcode.h:250
pub const OP_neq: u16 = 168;
// c: quickjs-opcode.h:251
pub const OP_strict_eq: u16 = 169;
// c: quickjs-opcode.h:252
pub const OP_strict_neq: u16 = 170;
// c: quickjs-opcode.h:253
pub const OP_and: u16 = 171;
// c: quickjs-opcode.h:254
pub const OP_xor: u16 = 172;
// c: quickjs-opcode.h:255
pub const OP_or: u16 = 173;
// c: quickjs-opcode.h:256
pub const OP_is_undefined_or_null: u16 = 174;
// c: quickjs-opcode.h:257
pub const OP_private_in: u16 = 175;
// c: quickjs-opcode.h:258
pub const OP_push_bigint_i32: u16 = 176;
// c: quickjs-opcode.h:260
pub const OP_nop: u16 = 177;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:290
pub const OP_push_minus1: u16 = 178;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:291
pub const OP_push_0: u16 = 179;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:292
pub const OP_push_1: u16 = 180;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:293
pub const OP_push_2: u16 = 181;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:294
pub const OP_push_3: u16 = 182;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:295
pub const OP_push_4: u16 = 183;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:296
pub const OP_push_5: u16 = 184;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:297
pub const OP_push_6: u16 = 185;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:298
pub const OP_push_7: u16 = 186;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:299
pub const OP_push_i8: u16 = 187;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:300
pub const OP_push_i16: u16 = 188;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:301
pub const OP_push_const8: u16 = 189;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:302
pub const OP_fclosure8: u16 = 190;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:303
pub const OP_push_empty_string: u16 = 191;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:305
pub const OP_get_loc8: u16 = 192;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:306
pub const OP_put_loc8: u16 = 193;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:307
pub const OP_set_loc8: u16 = 194;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:309
pub const OP_get_loc0: u16 = 195;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:310
pub const OP_get_loc1: u16 = 196;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:311
pub const OP_get_loc2: u16 = 197;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:312
pub const OP_get_loc3: u16 = 198;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:313
pub const OP_put_loc0: u16 = 199;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:314
pub const OP_put_loc1: u16 = 200;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:315
pub const OP_put_loc2: u16 = 201;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:316
pub const OP_put_loc3: u16 = 202;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:317
pub const OP_set_loc0: u16 = 203;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:318
pub const OP_set_loc1: u16 = 204;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:319
pub const OP_set_loc2: u16 = 205;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:320
pub const OP_set_loc3: u16 = 206;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:321
pub const OP_get_arg0: u16 = 207;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:322
pub const OP_get_arg1: u16 = 208;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:323
pub const OP_get_arg2: u16 = 209;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:324
pub const OP_get_arg3: u16 = 210;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:325
pub const OP_put_arg0: u16 = 211;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:326
pub const OP_put_arg1: u16 = 212;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:327
pub const OP_put_arg2: u16 = 213;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:328
pub const OP_put_arg3: u16 = 214;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:329
pub const OP_set_arg0: u16 = 215;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:330
pub const OP_set_arg1: u16 = 216;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:331
pub const OP_set_arg2: u16 = 217;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:332
pub const OP_set_arg3: u16 = 218;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:333
pub const OP_get_var_ref0: u16 = 219;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:334
pub const OP_get_var_ref1: u16 = 220;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:335
pub const OP_get_var_ref2: u16 = 221;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:336
pub const OP_get_var_ref3: u16 = 222;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:337
pub const OP_put_var_ref0: u16 = 223;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:338
pub const OP_put_var_ref1: u16 = 224;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:339
pub const OP_put_var_ref2: u16 = 225;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:340
pub const OP_put_var_ref3: u16 = 226;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:341
pub const OP_set_var_ref0: u16 = 227;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:342
pub const OP_set_var_ref1: u16 = 228;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:343
pub const OP_set_var_ref2: u16 = 229;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:344
pub const OP_set_var_ref3: u16 = 230;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:346
pub const OP_get_length: u16 = 231;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:348
pub const OP_if_false8: u16 = 232;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:349
pub const OP_if_true8: u16 = 233;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:350
pub const OP_goto8: u16 = 234;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:351
pub const OP_goto16: u16 = 235;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:353
pub const OP_call0: u16 = 236;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:354
pub const OP_call1: u16 = 237;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:355
pub const OP_call2: u16 = 238;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:356
pub const OP_call3: u16 = 239;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:358
pub const OP_is_undefined: u16 = 240;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:359
pub const OP_is_null: u16 = 241;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:360
pub const OP_typeof_is_undefined: u16 = 242;
#[cfg(feature = "short-opcodes")]
// c: quickjs-opcode.h:361
pub const OP_typeof_is_function: u16 = 243;
#[cfg(feature = "short-opcodes")]
pub const OP_COUNT: usize = 244;
#[cfg(not(feature = "short-opcodes"))]
pub const OP_COUNT: usize = 178;
pub const OP_TEMP_START: u16 = OP_nop + 1;
pub const OP___dummy: u16 = OP_TEMP_START - 1;
pub const OP_enter_scope: u16 = OP_TEMP_START + 0;
pub const OP_leave_scope: u16 = OP_TEMP_START + 1;
pub const OP_label: u16 = OP_TEMP_START + 2;
pub const OP_scope_get_var_undef: u16 = OP_TEMP_START + 3;
pub const OP_scope_get_var: u16 = OP_TEMP_START + 4;
pub const OP_scope_put_var: u16 = OP_TEMP_START + 5;
pub const OP_scope_delete_var: u16 = OP_TEMP_START + 6;
pub const OP_scope_make_ref: u16 = OP_TEMP_START + 7;
pub const OP_scope_get_ref: u16 = OP_TEMP_START + 8;
pub const OP_scope_put_var_init: u16 = OP_TEMP_START + 9;
pub const OP_scope_get_var_checkthis: u16 = OP_TEMP_START + 10;
pub const OP_scope_get_private_field: u16 = OP_TEMP_START + 11;
pub const OP_scope_get_private_field2: u16 = OP_TEMP_START + 12;
pub const OP_scope_put_private_field: u16 = OP_TEMP_START + 13;
pub const OP_scope_in_private_field: u16 = OP_TEMP_START + 14;
pub const OP_get_field_opt_chain: u16 = OP_TEMP_START + 15;
pub const OP_get_array_el_opt_chain: u16 = OP_TEMP_START + 16;
pub const OP_set_class_name: u16 = OP_TEMP_START + 17;
pub const OP_line_num: u16 = OP_TEMP_START + 18;
pub const OP_TEMP_END: u16 = OP_TEMP_START + 19;
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct JSOpCode {
    pub size: u8,
    pub n_pop: u8,
    pub n_push: u8,
    pub fmt: u8,
}
pub static opcode_info: &[JSOpCode] = &[
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_i32 as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_const as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_const as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_u8 as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_u16 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 6,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 4,
        n_push: 5,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 4,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 5,
        n_push: 5,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 4,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 4,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 5,
        n_push: 5,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_npop as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_npop as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_npop as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_npop as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 2,
        n_push: 0,
        fmt: OP_FMT_npop as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_npop as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 3,
        n_push: 1,
        fmt: OP_FMT_u16 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 6,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_atom_u8 as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_npop_u16 as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_u16 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 2,
        n_push: 0,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 4,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 3,
        n_push: 3,
        fmt: OP_FMT_u8 as u8,
    },
    JSOpCode {
        size: 6,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_atom_u8 as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 3,
        n_push: 1,
        fmt: OP_FMT_u8 as u8,
    },
    JSOpCode {
        size: 6,
        n_pop: 2,
        n_push: 2,
        fmt: OP_FMT_atom_u8 as u8,
    },
    JSOpCode {
        size: 6,
        n_pop: 3,
        n_push: 3,
        fmt: OP_FMT_atom_u8 as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_arg as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_arg as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_arg as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_var_ref as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_loc as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_label as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_label as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_label as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_label as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_label as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 10,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_atom_label_u8 as u8,
    },
    JSOpCode {
        size: 10,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_atom_label_u8 as u8,
    },
    JSOpCode {
        size: 10,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_atom_label_u8 as u8,
    },
    JSOpCode {
        size: 10,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_atom_label_u8 as u8,
    },
    JSOpCode {
        size: 10,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_atom_label_u8 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 3,
        n_push: 5,
        fmt: OP_FMT_u8 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 3,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 3,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 4,
        n_push: 4,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 4,
        n_push: 5,
        fmt: OP_FMT_u8 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_loc8 as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_loc8 as u8,
    },
    JSOpCode {
        size: 2,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_loc8 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_i32 as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_u16 as u8,
    },
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_u16 as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_label as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 11,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom_label_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 2,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 1,
        n_push: 2,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 2,
        n_push: 0,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 7,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_atom_u16 as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_atom as u8,
    },
    JSOpCode {
        size: 1,
        n_pop: 2,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_u32 as u8,
    },
    JSOpCode {
        size: 5,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_u32 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_int as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_i8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_i16 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_const8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_const8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_loc8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_loc8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_loc8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_loc as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_arg as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 0,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none_var_ref as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_label8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 1,
        n_push: 0,
        fmt: OP_FMT_label8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 2,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_label8 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 3,
        n_pop: 0,
        n_push: 0,
        fmt: OP_FMT_label16 as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_npopx as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_npopx as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_npopx as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_npopx as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
    #[cfg(feature = "short-opcodes")]
    JSOpCode {
        size: 1,
        n_pop: 1,
        n_push: 1,
        fmt: OP_FMT_none as u8,
    },
];
pub fn short_opcode_info(op: usize) -> &'static JSOpCode {
    #[cfg(feature = "short-opcodes")]
    let op = if op >= OP_TEMP_START as usize {
        op + (OP_TEMP_END - OP_TEMP_START) as usize
    } else {
        op
    };
    &opcode_info[op]
}
pub static OPCODE_NAMES: &[&str] = &[
    "invalid",
    "push_i32",
    "push_const",
    "fclosure",
    "push_atom_value",
    "private_symbol",
    "undefined",
    "null",
    "push_this",
    "push_false",
    "push_true",
    "object",
    "special_object",
    "rest",
    "drop",
    "nip",
    "nip1",
    "dup",
    "dup1",
    "dup2",
    "dup3",
    "insert2",
    "insert3",
    "insert4",
    "perm3",
    "perm4",
    "perm5",
    "swap",
    "swap2",
    "rot3l",
    "rot3r",
    "rot4l",
    "rot5l",
    "call_constructor",
    "call",
    "tail_call",
    "call_method",
    "tail_call_method",
    "array_from",
    "apply",
    "return",
    "return_undef",
    "check_ctor_return",
    "check_ctor",
    "init_ctor",
    "check_brand",
    "add_brand",
    "return_async",
    "throw",
    "throw_error",
    "eval",
    "apply_eval",
    "regexp",
    "get_super",
    "import",
    "get_var_undef",
    "get_var",
    "put_var",
    "put_var_init",
    "get_ref_value",
    "put_ref_value",
    "get_field",
    "get_field2",
    "put_field",
    "get_private_field",
    "put_private_field",
    "define_private_field",
    "get_array_el",
    "get_array_el2",
    "get_array_el3",
    "put_array_el",
    "get_super_value",
    "put_super_value",
    "define_field",
    "set_name",
    "set_name_computed",
    "set_proto",
    "set_home_object",
    "define_array_el",
    "append",
    "copy_data_properties",
    "define_method",
    "define_method_computed",
    "define_class",
    "define_class_computed",
    "get_loc",
    "put_loc",
    "set_loc",
    "get_arg",
    "put_arg",
    "set_arg",
    "get_var_ref",
    "put_var_ref",
    "set_var_ref",
    "set_loc_uninitialized",
    "get_loc_check",
    "put_loc_check",
    "set_loc_check",
    "put_loc_check_init",
    "get_loc_checkthis",
    "get_var_ref_check",
    "put_var_ref_check",
    "put_var_ref_check_init",
    "close_loc",
    "if_false",
    "if_true",
    "goto",
    "catch",
    "gosub",
    "ret",
    "nip_catch",
    "to_object",
    "to_propkey",
    "with_get_var",
    "with_put_var",
    "with_delete_var",
    "with_make_ref",
    "with_get_ref",
    "make_loc_ref",
    "make_arg_ref",
    "make_var_ref_ref",
    "make_var_ref",
    "for_in_start",
    "for_of_start",
    "for_await_of_start",
    "for_in_next",
    "for_of_next",
    "for_await_of_next",
    "iterator_check_object",
    "iterator_get_value_done",
    "iterator_close",
    "iterator_next",
    "iterator_call",
    "initial_yield",
    "yield",
    "yield_star",
    "async_yield_star",
    "await",
    "neg",
    "plus",
    "dec",
    "inc",
    "post_dec",
    "post_inc",
    "dec_loc",
    "inc_loc",
    "add_loc",
    "not",
    "lnot",
    "typeof",
    "delete",
    "delete_var",
    "mul",
    "div",
    "mod",
    "add",
    "sub",
    "pow",
    "shl",
    "sar",
    "shr",
    "lt",
    "lte",
    "gt",
    "gte",
    "instanceof",
    "in",
    "eq",
    "neq",
    "strict_eq",
    "strict_neq",
    "and",
    "xor",
    "or",
    "is_undefined_or_null",
    "private_in",
    "push_bigint_i32",
    "nop",
    "enter_scope",
    "leave_scope",
    "label",
    "scope_get_var_undef",
    "scope_get_var",
    "scope_put_var",
    "scope_delete_var",
    "scope_make_ref",
    "scope_get_ref",
    "scope_put_var_init",
    "scope_get_var_checkthis",
    "scope_get_private_field",
    "scope_get_private_field2",
    "scope_put_private_field",
    "scope_in_private_field",
    "get_field_opt_chain",
    "get_array_el_opt_chain",
    "set_class_name",
    "line_num",
    #[cfg(feature = "short-opcodes")]
    "push_minus1",
    #[cfg(feature = "short-opcodes")]
    "push_0",
    #[cfg(feature = "short-opcodes")]
    "push_1",
    #[cfg(feature = "short-opcodes")]
    "push_2",
    #[cfg(feature = "short-opcodes")]
    "push_3",
    #[cfg(feature = "short-opcodes")]
    "push_4",
    #[cfg(feature = "short-opcodes")]
    "push_5",
    #[cfg(feature = "short-opcodes")]
    "push_6",
    #[cfg(feature = "short-opcodes")]
    "push_7",
    #[cfg(feature = "short-opcodes")]
    "push_i8",
    #[cfg(feature = "short-opcodes")]
    "push_i16",
    #[cfg(feature = "short-opcodes")]
    "push_const8",
    #[cfg(feature = "short-opcodes")]
    "fclosure8",
    #[cfg(feature = "short-opcodes")]
    "push_empty_string",
    #[cfg(feature = "short-opcodes")]
    "get_loc8",
    #[cfg(feature = "short-opcodes")]
    "put_loc8",
    #[cfg(feature = "short-opcodes")]
    "set_loc8",
    #[cfg(feature = "short-opcodes")]
    "get_loc0",
    #[cfg(feature = "short-opcodes")]
    "get_loc1",
    #[cfg(feature = "short-opcodes")]
    "get_loc2",
    #[cfg(feature = "short-opcodes")]
    "get_loc3",
    #[cfg(feature = "short-opcodes")]
    "put_loc0",
    #[cfg(feature = "short-opcodes")]
    "put_loc1",
    #[cfg(feature = "short-opcodes")]
    "put_loc2",
    #[cfg(feature = "short-opcodes")]
    "put_loc3",
    #[cfg(feature = "short-opcodes")]
    "set_loc0",
    #[cfg(feature = "short-opcodes")]
    "set_loc1",
    #[cfg(feature = "short-opcodes")]
    "set_loc2",
    #[cfg(feature = "short-opcodes")]
    "set_loc3",
    #[cfg(feature = "short-opcodes")]
    "get_arg0",
    #[cfg(feature = "short-opcodes")]
    "get_arg1",
    #[cfg(feature = "short-opcodes")]
    "get_arg2",
    #[cfg(feature = "short-opcodes")]
    "get_arg3",
    #[cfg(feature = "short-opcodes")]
    "put_arg0",
    #[cfg(feature = "short-opcodes")]
    "put_arg1",
    #[cfg(feature = "short-opcodes")]
    "put_arg2",
    #[cfg(feature = "short-opcodes")]
    "put_arg3",
    #[cfg(feature = "short-opcodes")]
    "set_arg0",
    #[cfg(feature = "short-opcodes")]
    "set_arg1",
    #[cfg(feature = "short-opcodes")]
    "set_arg2",
    #[cfg(feature = "short-opcodes")]
    "set_arg3",
    #[cfg(feature = "short-opcodes")]
    "get_var_ref0",
    #[cfg(feature = "short-opcodes")]
    "get_var_ref1",
    #[cfg(feature = "short-opcodes")]
    "get_var_ref2",
    #[cfg(feature = "short-opcodes")]
    "get_var_ref3",
    #[cfg(feature = "short-opcodes")]
    "put_var_ref0",
    #[cfg(feature = "short-opcodes")]
    "put_var_ref1",
    #[cfg(feature = "short-opcodes")]
    "put_var_ref2",
    #[cfg(feature = "short-opcodes")]
    "put_var_ref3",
    #[cfg(feature = "short-opcodes")]
    "set_var_ref0",
    #[cfg(feature = "short-opcodes")]
    "set_var_ref1",
    #[cfg(feature = "short-opcodes")]
    "set_var_ref2",
    #[cfg(feature = "short-opcodes")]
    "set_var_ref3",
    #[cfg(feature = "short-opcodes")]
    "get_length",
    #[cfg(feature = "short-opcodes")]
    "if_false8",
    #[cfg(feature = "short-opcodes")]
    "if_true8",
    #[cfg(feature = "short-opcodes")]
    "goto8",
    #[cfg(feature = "short-opcodes")]
    "goto16",
    #[cfg(feature = "short-opcodes")]
    "call0",
    #[cfg(feature = "short-opcodes")]
    "call1",
    #[cfg(feature = "short-opcodes")]
    "call2",
    #[cfg(feature = "short-opcodes")]
    "call3",
    #[cfg(feature = "short-opcodes")]
    "is_undefined",
    #[cfg(feature = "short-opcodes")]
    "is_null",
    #[cfg(feature = "short-opcodes")]
    "typeof_is_undefined",
    #[cfg(feature = "short-opcodes")]
    "typeof_is_function",
];
