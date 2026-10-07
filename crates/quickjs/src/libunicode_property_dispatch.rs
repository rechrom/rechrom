// Dispatch from libunicode.c; Copyright 2017-2018 Fabrice Bellard, MIT.
pub unsafe fn unicode_prop(cr:*mut CharRange,prop_name:*const c_char)->i32 {
let prop_idx=unicode_find_name(unicode_prop_name_table.as_ptr(),prop_name);if prop_idx<0 {return -2;}
let prop_idx=prop_idx+UNICODE_PROP_ASCII_Hex_Digit;
match prop_idx {
UNICODE_PROP_ASCII=>{
if cr_add_interval(cr,0x00,0x7f + 1)!=0 {-1} else {0}
},
UNICODE_PROP_Any=>{
if cr_add_interval(cr,0x00000,0x10ffff + 1)!=0 {-1} else {0}
},
UNICODE_PROP_Assigned=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Cn),
                               POP_INVERT,
                               POP_END])
},
UNICODE_PROP_Math=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Sm),
                               POP_PROP, UNICODE_PROP_Other_Math as u32,
                               POP_UNION,
                               POP_END])
},
UNICODE_PROP_Lowercase=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Ll),
                               POP_PROP, UNICODE_PROP_Other_Lowercase as u32,
                               POP_UNION,
                               POP_END])
},
UNICODE_PROP_Uppercase=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Lu),
                               POP_PROP, UNICODE_PROP_Other_Uppercase as u32,
                               POP_UNION,
                               POP_END])
},
UNICODE_PROP_Cased=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll) | M!(UNICODE_GC_Lt),
                               POP_PROP, UNICODE_PROP_Other_Uppercase as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_Other_Lowercase as u32,
                               POP_UNION,
                               POP_END])
},
UNICODE_PROP_Alphabetic=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll) | M!(UNICODE_GC_Lt) | M!(UNICODE_GC_Lm) | M!(UNICODE_GC_Lo) | M!(UNICODE_GC_Nl),
                               POP_PROP, UNICODE_PROP_Other_Uppercase as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_Other_Lowercase as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_Other_Alphabetic as u32,
                               POP_UNION,
                               POP_END])
},
UNICODE_PROP_Grapheme_Base=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Cc) | M!(UNICODE_GC_Cf) | M!(UNICODE_GC_Cs) | M!(UNICODE_GC_Co) | M!(UNICODE_GC_Cn) | M!(UNICODE_GC_Zl) | M!(UNICODE_GC_Zp) | M!(UNICODE_GC_Me) | M!(UNICODE_GC_Mn),
                               POP_PROP, UNICODE_PROP_Other_Grapheme_Extend as u32,
                               POP_UNION,
                               POP_INVERT,
                               POP_END])
},
UNICODE_PROP_Grapheme_Extend=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Me) | M!(UNICODE_GC_Mn),
                               POP_PROP, UNICODE_PROP_Other_Grapheme_Extend as u32,
                               POP_UNION,
                               POP_END])
},
UNICODE_PROP_XID_Start=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll) | M!(UNICODE_GC_Lt) | M!(UNICODE_GC_Lm) | M!(UNICODE_GC_Lo) | M!(UNICODE_GC_Nl),
                               POP_PROP, UNICODE_PROP_Other_ID_Start as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_Pattern_Syntax as u32,
                               POP_PROP, UNICODE_PROP_Pattern_White_Space as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_XID_Start1 as u32,
                               POP_UNION,
                               POP_INVERT,
                               POP_INTER,
                               POP_END])
},
UNICODE_PROP_XID_Continue=>{
unicode_prop_ops(cr,&[
                               POP_GC, M!(UNICODE_GC_Lu) | M!(UNICODE_GC_Ll) | M!(UNICODE_GC_Lt) | M!(UNICODE_GC_Lm) | M!(UNICODE_GC_Lo) | M!(UNICODE_GC_Nl) |
                               M!(UNICODE_GC_Mn) | M!(UNICODE_GC_Mc) | M!(UNICODE_GC_Nd) | M!(UNICODE_GC_Pc),
                               POP_PROP, UNICODE_PROP_Other_ID_Start as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_Other_ID_Continue as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_Pattern_Syntax as u32,
                               POP_PROP, UNICODE_PROP_Pattern_White_Space as u32,
                               POP_UNION,
                               POP_PROP, UNICODE_PROP_XID_Continue1 as u32,
                               POP_UNION,
                               POP_INVERT,
                               POP_INTER,
                               POP_END])
},
UNICODE_PROP_Changes_When_Uppercased=>{
unicode_case1(cr,CASE_U)
},
UNICODE_PROP_Changes_When_Lowercased=>{
unicode_case1(cr,CASE_L)
},
UNICODE_PROP_Changes_When_Casemapped=>{
unicode_case1(cr,CASE_U | CASE_L | CASE_F)
},
UNICODE_PROP_Changes_When_Titlecased=>{
unicode_prop_ops(cr,&[
                               POP_CASE, CASE_U as u32,
                               POP_PROP, UNICODE_PROP_Changes_When_Titlecased1 as u32,
                               POP_XOR,
                               POP_END])
},
UNICODE_PROP_Changes_When_Casefolded=>{
unicode_prop_ops(cr,&[
                               POP_CASE, CASE_F as u32,
                               POP_PROP, UNICODE_PROP_Changes_When_Casefolded1 as u32,
                               POP_XOR,
                               POP_END])
},
UNICODE_PROP_Changes_When_NFKC_Casefolded=>{
unicode_prop_ops(cr,&[
                               POP_CASE, CASE_F as u32,
                               POP_PROP, UNICODE_PROP_Changes_When_NFKC_Casefolded1 as u32,
                               POP_XOR,
                               POP_END])
},
UNICODE_PROP_ID_Continue=>{
unicode_prop_ops(cr,&[
                               POP_PROP, UNICODE_PROP_ID_Start as u32,
                               POP_PROP, UNICODE_PROP_ID_Continue1 as u32,
                               POP_XOR,
                               POP_END])
},
_=>{if prop_idx as usize>=unicode_prop_table.len() {-2} else {unicode_prop1(cr,prop_idx as usize)}},
}
}
