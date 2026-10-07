#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

// cpp: font_engine/text/native/grapheme_break_property_data.h:15-31
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphemeProperty {
    kOther,
    kCr,
    kLf,
    kControl,
    kExtend,
    kZwj,
    kRegionalIndicator,
    kPrepend,
    kSpacingMark,
    kL,
    kV,
    kT,
    kLv,
    kLvt,
}

// cpp: font_engine/text/native/grapheme_break_property_data.h:32-37
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndicConjunctBreak {
    kNone,
    kConsonant,
    kExtend,
    kLinker,
}

// cpp: font_engine/text/native/grapheme_break_property_data.h:39-40
type CodePointRange = (u32, u32);
type PropertyRange = (u32, u32, GraphemeProperty);

// cpp: font_engine/text/native/grapheme_break_property_data.h:41
const kGraphemePropertyRanges: [(u32, u32, GraphemeProperty); 1419] = [
    // cpp: font_engine/text/native/grapheme_break_property_data.h:42
    (0x0, 0x9, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:43
    (0xA, 0xA, GraphemeProperty::kLf),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:44
    (0xB, 0xC, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:45
    (0xD, 0xD, GraphemeProperty::kCr),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:46
    (0xE, 0x1F, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:47
    (0x7F, 0x9F, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:48
    (0xAD, 0xAD, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:49
    (0x300, 0x36F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:50
    (0x483, 0x487, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:51
    (0x488, 0x489, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:52
    (0x591, 0x5BD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:53
    (0x5BF, 0x5BF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:54
    (0x5C1, 0x5C2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:55
    (0x5C4, 0x5C5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:56
    (0x5C7, 0x5C7, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:57
    (0x600, 0x605, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:58
    (0x610, 0x61A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:59
    (0x61C, 0x61C, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:60
    (0x64B, 0x65F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:61
    (0x670, 0x670, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:62
    (0x6D6, 0x6DC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:63
    (0x6DD, 0x6DD, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:64
    (0x6DF, 0x6E4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:65
    (0x6E7, 0x6E8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:66
    (0x6EA, 0x6ED, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:67
    (0x70F, 0x70F, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:68
    (0x711, 0x711, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:69
    (0x730, 0x74A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:70
    (0x7A6, 0x7B0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:71
    (0x7EB, 0x7F3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:72
    (0x7FD, 0x7FD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:73
    (0x816, 0x819, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:74
    (0x81B, 0x823, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:75
    (0x825, 0x827, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:76
    (0x829, 0x82D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:77
    (0x859, 0x85B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:78
    (0x890, 0x891, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:79
    (0x897, 0x89F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:80
    (0x8CA, 0x8E1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:81
    (0x8E2, 0x8E2, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:82
    (0x8E3, 0x902, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:83
    (0x903, 0x903, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:84
    (0x93A, 0x93A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:85
    (0x93B, 0x93B, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:86
    (0x93C, 0x93C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:87
    (0x93E, 0x940, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:88
    (0x941, 0x948, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:89
    (0x949, 0x94C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:90
    (0x94D, 0x94D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:91
    (0x94E, 0x94F, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:92
    (0x951, 0x957, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:93
    (0x962, 0x963, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:94
    (0x981, 0x981, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:95
    (0x982, 0x983, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:96
    (0x9BC, 0x9BC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:97
    (0x9BE, 0x9BE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:98
    (0x9BF, 0x9C0, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:99
    (0x9C1, 0x9C4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:100
    (0x9C7, 0x9C8, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:101
    (0x9CB, 0x9CC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:102
    (0x9CD, 0x9CD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:103
    (0x9D7, 0x9D7, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:104
    (0x9E2, 0x9E3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:105
    (0x9FE, 0x9FE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:106
    (0xA01, 0xA02, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:107
    (0xA03, 0xA03, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:108
    (0xA3C, 0xA3C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:109
    (0xA3E, 0xA40, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:110
    (0xA41, 0xA42, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:111
    (0xA47, 0xA48, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:112
    (0xA4B, 0xA4D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:113
    (0xA51, 0xA51, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:114
    (0xA70, 0xA71, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:115
    (0xA75, 0xA75, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:116
    (0xA81, 0xA82, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:117
    (0xA83, 0xA83, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:118
    (0xABC, 0xABC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:119
    (0xABE, 0xAC0, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:120
    (0xAC1, 0xAC5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:121
    (0xAC7, 0xAC8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:122
    (0xAC9, 0xAC9, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:123
    (0xACB, 0xACC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:124
    (0xACD, 0xACD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:125
    (0xAE2, 0xAE3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:126
    (0xAFA, 0xAFF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:127
    (0xB01, 0xB01, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:128
    (0xB02, 0xB03, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:129
    (0xB3C, 0xB3C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:130
    (0xB3E, 0xB3E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:131
    (0xB3F, 0xB3F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:132
    (0xB40, 0xB40, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:133
    (0xB41, 0xB44, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:134
    (0xB47, 0xB48, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:135
    (0xB4B, 0xB4C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:136
    (0xB4D, 0xB4D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:137
    (0xB55, 0xB56, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:138
    (0xB57, 0xB57, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:139
    (0xB62, 0xB63, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:140
    (0xB82, 0xB82, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:141
    (0xBBE, 0xBBE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:142
    (0xBBF, 0xBBF, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:143
    (0xBC0, 0xBC0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:144
    (0xBC1, 0xBC2, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:145
    (0xBC6, 0xBC8, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:146
    (0xBCA, 0xBCC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:147
    (0xBCD, 0xBCD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:148
    (0xBD7, 0xBD7, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:149
    (0xC00, 0xC00, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:150
    (0xC01, 0xC03, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:151
    (0xC04, 0xC04, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:152
    (0xC3C, 0xC3C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:153
    (0xC3E, 0xC40, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:154
    (0xC41, 0xC44, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:155
    (0xC46, 0xC48, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:156
    (0xC4A, 0xC4D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:157
    (0xC55, 0xC56, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:158
    (0xC62, 0xC63, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:159
    (0xC81, 0xC81, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:160
    (0xC82, 0xC83, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:161
    (0xCBC, 0xCBC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:162
    (0xCBE, 0xCBE, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:163
    (0xCBF, 0xCBF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:164
    (0xCC0, 0xCC0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:165
    (0xCC1, 0xCC1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:166
    (0xCC2, 0xCC2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:167
    (0xCC3, 0xCC4, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:168
    (0xCC6, 0xCC6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:169
    (0xCC7, 0xCC8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:170
    (0xCCA, 0xCCB, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:171
    (0xCCC, 0xCCD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:172
    (0xCD5, 0xCD6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:173
    (0xCE2, 0xCE3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:174
    (0xCF3, 0xCF3, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:175
    (0xD00, 0xD01, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:176
    (0xD02, 0xD03, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:177
    (0xD3B, 0xD3C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:178
    (0xD3E, 0xD3E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:179
    (0xD3F, 0xD40, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:180
    (0xD41, 0xD44, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:181
    (0xD46, 0xD48, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:182
    (0xD4A, 0xD4C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:183
    (0xD4D, 0xD4D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:184
    (0xD4E, 0xD4E, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:185
    (0xD57, 0xD57, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:186
    (0xD62, 0xD63, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:187
    (0xD81, 0xD81, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:188
    (0xD82, 0xD83, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:189
    (0xDCA, 0xDCA, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:190
    (0xDCF, 0xDCF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:191
    (0xDD0, 0xDD1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:192
    (0xDD2, 0xDD4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:193
    (0xDD6, 0xDD6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:194
    (0xDD8, 0xDDE, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:195
    (0xDDF, 0xDDF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:196
    (0xDF2, 0xDF3, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:197
    (0xE31, 0xE31, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:198
    (0xE33, 0xE33, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:199
    (0xE34, 0xE3A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:200
    (0xE47, 0xE4E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:201
    (0xEB1, 0xEB1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:202
    (0xEB3, 0xEB3, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:203
    (0xEB4, 0xEBC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:204
    (0xEC8, 0xECE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:205
    (0xF18, 0xF19, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:206
    (0xF35, 0xF35, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:207
    (0xF37, 0xF37, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:208
    (0xF39, 0xF39, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:209
    (0xF3E, 0xF3F, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:210
    (0xF71, 0xF7E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:211
    (0xF7F, 0xF7F, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:212
    (0xF80, 0xF84, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:213
    (0xF86, 0xF87, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:214
    (0xF8D, 0xF97, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:215
    (0xF99, 0xFBC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:216
    (0xFC6, 0xFC6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:217
    (0x102D, 0x1030, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:218
    (0x1031, 0x1031, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:219
    (0x1032, 0x1037, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:220
    (0x1039, 0x103A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:221
    (0x103B, 0x103C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:222
    (0x103D, 0x103E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:223
    (0x1056, 0x1057, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:224
    (0x1058, 0x1059, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:225
    (0x105E, 0x1060, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:226
    (0x1071, 0x1074, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:227
    (0x1082, 0x1082, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:228
    (0x1084, 0x1084, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:229
    (0x1085, 0x1086, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:230
    (0x108D, 0x108D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:231
    (0x109D, 0x109D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:232
    (0x1100, 0x115F, GraphemeProperty::kL),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:233
    (0x1160, 0x11A7, GraphemeProperty::kV),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:234
    (0x11A8, 0x11FF, GraphemeProperty::kT),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:235
    (0x135D, 0x135F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:236
    (0x1712, 0x1714, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:237
    (0x1715, 0x1715, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:238
    (0x1732, 0x1733, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:239
    (0x1734, 0x1734, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:240
    (0x1752, 0x1753, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:241
    (0x1772, 0x1773, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:242
    (0x17B4, 0x17B5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:243
    (0x17B6, 0x17B6, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:244
    (0x17B7, 0x17BD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:245
    (0x17BE, 0x17C5, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:246
    (0x17C6, 0x17C6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:247
    (0x17C7, 0x17C8, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:248
    (0x17C9, 0x17D3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:249
    (0x17DD, 0x17DD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:250
    (0x180B, 0x180D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:251
    (0x180E, 0x180E, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:252
    (0x180F, 0x180F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:253
    (0x1885, 0x1886, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:254
    (0x18A9, 0x18A9, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:255
    (0x1920, 0x1922, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:256
    (0x1923, 0x1926, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:257
    (0x1927, 0x1928, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:258
    (0x1929, 0x192B, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:259
    (0x1930, 0x1931, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:260
    (0x1932, 0x1932, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:261
    (0x1933, 0x1938, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:262
    (0x1939, 0x193B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:263
    (0x1A17, 0x1A18, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:264
    (0x1A19, 0x1A1A, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:265
    (0x1A1B, 0x1A1B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:266
    (0x1A55, 0x1A55, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:267
    (0x1A56, 0x1A56, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:268
    (0x1A57, 0x1A57, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:269
    (0x1A58, 0x1A5E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:270
    (0x1A60, 0x1A60, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:271
    (0x1A62, 0x1A62, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:272
    (0x1A65, 0x1A6C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:273
    (0x1A6D, 0x1A72, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:274
    (0x1A73, 0x1A7C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:275
    (0x1A7F, 0x1A7F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:276
    (0x1AB0, 0x1ABD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:277
    (0x1ABE, 0x1ABE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:278
    (0x1ABF, 0x1ACE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:279
    (0x1B00, 0x1B03, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:280
    (0x1B04, 0x1B04, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:281
    (0x1B34, 0x1B34, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:282
    (0x1B35, 0x1B35, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:283
    (0x1B36, 0x1B3A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:284
    (0x1B3B, 0x1B3B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:285
    (0x1B3C, 0x1B3C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:286
    (0x1B3D, 0x1B3D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:287
    (0x1B3E, 0x1B41, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:288
    (0x1B42, 0x1B42, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:289
    (0x1B43, 0x1B44, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:290
    (0x1B6B, 0x1B73, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:291
    (0x1B80, 0x1B81, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:292
    (0x1B82, 0x1B82, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:293
    (0x1BA1, 0x1BA1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:294
    (0x1BA2, 0x1BA5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:295
    (0x1BA6, 0x1BA7, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:296
    (0x1BA8, 0x1BA9, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:297
    (0x1BAA, 0x1BAA, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:298
    (0x1BAB, 0x1BAD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:299
    (0x1BE6, 0x1BE6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:300
    (0x1BE7, 0x1BE7, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:301
    (0x1BE8, 0x1BE9, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:302
    (0x1BEA, 0x1BEC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:303
    (0x1BED, 0x1BED, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:304
    (0x1BEE, 0x1BEE, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:305
    (0x1BEF, 0x1BF1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:306
    (0x1BF2, 0x1BF3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:307
    (0x1C24, 0x1C2B, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:308
    (0x1C2C, 0x1C33, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:309
    (0x1C34, 0x1C35, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:310
    (0x1C36, 0x1C37, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:311
    (0x1CD0, 0x1CD2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:312
    (0x1CD4, 0x1CE0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:313
    (0x1CE1, 0x1CE1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:314
    (0x1CE2, 0x1CE8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:315
    (0x1CED, 0x1CED, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:316
    (0x1CF4, 0x1CF4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:317
    (0x1CF7, 0x1CF7, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:318
    (0x1CF8, 0x1CF9, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:319
    (0x1DC0, 0x1DFF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:320
    (0x200B, 0x200B, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:321
    (0x200C, 0x200C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:322
    (0x200D, 0x200D, GraphemeProperty::kZwj),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:323
    (0x200E, 0x200F, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:324
    (0x2028, 0x2028, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:325
    (0x2029, 0x2029, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:326
    (0x202A, 0x202E, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:327
    (0x2060, 0x2064, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:328
    (0x2065, 0x2065, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:329
    (0x2066, 0x206F, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:330
    (0x20D0, 0x20DC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:331
    (0x20DD, 0x20E0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:332
    (0x20E1, 0x20E1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:333
    (0x20E2, 0x20E4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:334
    (0x20E5, 0x20F0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:335
    (0x2CEF, 0x2CF1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:336
    (0x2D7F, 0x2D7F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:337
    (0x2DE0, 0x2DFF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:338
    (0x302A, 0x302D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:339
    (0x302E, 0x302F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:340
    (0x3099, 0x309A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:341
    (0xA66F, 0xA66F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:342
    (0xA670, 0xA672, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:343
    (0xA674, 0xA67D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:344
    (0xA69E, 0xA69F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:345
    (0xA6F0, 0xA6F1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:346
    (0xA802, 0xA802, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:347
    (0xA806, 0xA806, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:348
    (0xA80B, 0xA80B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:349
    (0xA823, 0xA824, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:350
    (0xA825, 0xA826, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:351
    (0xA827, 0xA827, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:352
    (0xA82C, 0xA82C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:353
    (0xA880, 0xA881, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:354
    (0xA8B4, 0xA8C3, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:355
    (0xA8C4, 0xA8C5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:356
    (0xA8E0, 0xA8F1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:357
    (0xA8FF, 0xA8FF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:358
    (0xA926, 0xA92D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:359
    (0xA947, 0xA951, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:360
    (0xA952, 0xA952, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:361
    (0xA953, 0xA953, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:362
    (0xA960, 0xA97C, GraphemeProperty::kL),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:363
    (0xA980, 0xA982, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:364
    (0xA983, 0xA983, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:365
    (0xA9B3, 0xA9B3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:366
    (0xA9B4, 0xA9B5, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:367
    (0xA9B6, 0xA9B9, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:368
    (0xA9BA, 0xA9BB, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:369
    (0xA9BC, 0xA9BD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:370
    (0xA9BE, 0xA9BF, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:371
    (0xA9C0, 0xA9C0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:372
    (0xA9E5, 0xA9E5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:373
    (0xAA29, 0xAA2E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:374
    (0xAA2F, 0xAA30, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:375
    (0xAA31, 0xAA32, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:376
    (0xAA33, 0xAA34, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:377
    (0xAA35, 0xAA36, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:378
    (0xAA43, 0xAA43, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:379
    (0xAA4C, 0xAA4C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:380
    (0xAA4D, 0xAA4D, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:381
    (0xAA7C, 0xAA7C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:382
    (0xAAB0, 0xAAB0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:383
    (0xAAB2, 0xAAB4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:384
    (0xAAB7, 0xAAB8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:385
    (0xAABE, 0xAABF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:386
    (0xAAC1, 0xAAC1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:387
    (0xAAEB, 0xAAEB, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:388
    (0xAAEC, 0xAAED, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:389
    (0xAAEE, 0xAAEF, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:390
    (0xAAF5, 0xAAF5, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:391
    (0xAAF6, 0xAAF6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:392
    (0xABE3, 0xABE4, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:393
    (0xABE5, 0xABE5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:394
    (0xABE6, 0xABE7, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:395
    (0xABE8, 0xABE8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:396
    (0xABE9, 0xABEA, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:397
    (0xABEC, 0xABEC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:398
    (0xABED, 0xABED, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:399
    (0xAC00, 0xAC00, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:400
    (0xAC01, 0xAC1B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:401
    (0xAC1C, 0xAC1C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:402
    (0xAC1D, 0xAC37, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:403
    (0xAC38, 0xAC38, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:404
    (0xAC39, 0xAC53, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:405
    (0xAC54, 0xAC54, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:406
    (0xAC55, 0xAC6F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:407
    (0xAC70, 0xAC70, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:408
    (0xAC71, 0xAC8B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:409
    (0xAC8C, 0xAC8C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:410
    (0xAC8D, 0xACA7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:411
    (0xACA8, 0xACA8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:412
    (0xACA9, 0xACC3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:413
    (0xACC4, 0xACC4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:414
    (0xACC5, 0xACDF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:415
    (0xACE0, 0xACE0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:416
    (0xACE1, 0xACFB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:417
    (0xACFC, 0xACFC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:418
    (0xACFD, 0xAD17, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:419
    (0xAD18, 0xAD18, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:420
    (0xAD19, 0xAD33, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:421
    (0xAD34, 0xAD34, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:422
    (0xAD35, 0xAD4F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:423
    (0xAD50, 0xAD50, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:424
    (0xAD51, 0xAD6B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:425
    (0xAD6C, 0xAD6C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:426
    (0xAD6D, 0xAD87, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:427
    (0xAD88, 0xAD88, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:428
    (0xAD89, 0xADA3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:429
    (0xADA4, 0xADA4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:430
    (0xADA5, 0xADBF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:431
    (0xADC0, 0xADC0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:432
    (0xADC1, 0xADDB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:433
    (0xADDC, 0xADDC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:434
    (0xADDD, 0xADF7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:435
    (0xADF8, 0xADF8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:436
    (0xADF9, 0xAE13, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:437
    (0xAE14, 0xAE14, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:438
    (0xAE15, 0xAE2F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:439
    (0xAE30, 0xAE30, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:440
    (0xAE31, 0xAE4B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:441
    (0xAE4C, 0xAE4C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:442
    (0xAE4D, 0xAE67, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:443
    (0xAE68, 0xAE68, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:444
    (0xAE69, 0xAE83, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:445
    (0xAE84, 0xAE84, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:446
    (0xAE85, 0xAE9F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:447
    (0xAEA0, 0xAEA0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:448
    (0xAEA1, 0xAEBB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:449
    (0xAEBC, 0xAEBC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:450
    (0xAEBD, 0xAED7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:451
    (0xAED8, 0xAED8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:452
    (0xAED9, 0xAEF3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:453
    (0xAEF4, 0xAEF4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:454
    (0xAEF5, 0xAF0F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:455
    (0xAF10, 0xAF10, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:456
    (0xAF11, 0xAF2B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:457
    (0xAF2C, 0xAF2C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:458
    (0xAF2D, 0xAF47, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:459
    (0xAF48, 0xAF48, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:460
    (0xAF49, 0xAF63, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:461
    (0xAF64, 0xAF64, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:462
    (0xAF65, 0xAF7F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:463
    (0xAF80, 0xAF80, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:464
    (0xAF81, 0xAF9B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:465
    (0xAF9C, 0xAF9C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:466
    (0xAF9D, 0xAFB7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:467
    (0xAFB8, 0xAFB8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:468
    (0xAFB9, 0xAFD3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:469
    (0xAFD4, 0xAFD4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:470
    (0xAFD5, 0xAFEF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:471
    (0xAFF0, 0xAFF0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:472
    (0xAFF1, 0xB00B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:473
    (0xB00C, 0xB00C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:474
    (0xB00D, 0xB027, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:475
    (0xB028, 0xB028, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:476
    (0xB029, 0xB043, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:477
    (0xB044, 0xB044, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:478
    (0xB045, 0xB05F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:479
    (0xB060, 0xB060, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:480
    (0xB061, 0xB07B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:481
    (0xB07C, 0xB07C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:482
    (0xB07D, 0xB097, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:483
    (0xB098, 0xB098, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:484
    (0xB099, 0xB0B3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:485
    (0xB0B4, 0xB0B4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:486
    (0xB0B5, 0xB0CF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:487
    (0xB0D0, 0xB0D0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:488
    (0xB0D1, 0xB0EB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:489
    (0xB0EC, 0xB0EC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:490
    (0xB0ED, 0xB107, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:491
    (0xB108, 0xB108, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:492
    (0xB109, 0xB123, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:493
    (0xB124, 0xB124, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:494
    (0xB125, 0xB13F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:495
    (0xB140, 0xB140, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:496
    (0xB141, 0xB15B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:497
    (0xB15C, 0xB15C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:498
    (0xB15D, 0xB177, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:499
    (0xB178, 0xB178, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:500
    (0xB179, 0xB193, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:501
    (0xB194, 0xB194, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:502
    (0xB195, 0xB1AF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:503
    (0xB1B0, 0xB1B0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:504
    (0xB1B1, 0xB1CB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:505
    (0xB1CC, 0xB1CC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:506
    (0xB1CD, 0xB1E7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:507
    (0xB1E8, 0xB1E8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:508
    (0xB1E9, 0xB203, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:509
    (0xB204, 0xB204, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:510
    (0xB205, 0xB21F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:511
    (0xB220, 0xB220, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:512
    (0xB221, 0xB23B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:513
    (0xB23C, 0xB23C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:514
    (0xB23D, 0xB257, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:515
    (0xB258, 0xB258, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:516
    (0xB259, 0xB273, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:517
    (0xB274, 0xB274, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:518
    (0xB275, 0xB28F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:519
    (0xB290, 0xB290, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:520
    (0xB291, 0xB2AB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:521
    (0xB2AC, 0xB2AC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:522
    (0xB2AD, 0xB2C7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:523
    (0xB2C8, 0xB2C8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:524
    (0xB2C9, 0xB2E3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:525
    (0xB2E4, 0xB2E4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:526
    (0xB2E5, 0xB2FF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:527
    (0xB300, 0xB300, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:528
    (0xB301, 0xB31B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:529
    (0xB31C, 0xB31C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:530
    (0xB31D, 0xB337, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:531
    (0xB338, 0xB338, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:532
    (0xB339, 0xB353, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:533
    (0xB354, 0xB354, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:534
    (0xB355, 0xB36F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:535
    (0xB370, 0xB370, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:536
    (0xB371, 0xB38B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:537
    (0xB38C, 0xB38C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:538
    (0xB38D, 0xB3A7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:539
    (0xB3A8, 0xB3A8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:540
    (0xB3A9, 0xB3C3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:541
    (0xB3C4, 0xB3C4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:542
    (0xB3C5, 0xB3DF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:543
    (0xB3E0, 0xB3E0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:544
    (0xB3E1, 0xB3FB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:545
    (0xB3FC, 0xB3FC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:546
    (0xB3FD, 0xB417, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:547
    (0xB418, 0xB418, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:548
    (0xB419, 0xB433, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:549
    (0xB434, 0xB434, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:550
    (0xB435, 0xB44F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:551
    (0xB450, 0xB450, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:552
    (0xB451, 0xB46B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:553
    (0xB46C, 0xB46C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:554
    (0xB46D, 0xB487, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:555
    (0xB488, 0xB488, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:556
    (0xB489, 0xB4A3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:557
    (0xB4A4, 0xB4A4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:558
    (0xB4A5, 0xB4BF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:559
    (0xB4C0, 0xB4C0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:560
    (0xB4C1, 0xB4DB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:561
    (0xB4DC, 0xB4DC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:562
    (0xB4DD, 0xB4F7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:563
    (0xB4F8, 0xB4F8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:564
    (0xB4F9, 0xB513, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:565
    (0xB514, 0xB514, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:566
    (0xB515, 0xB52F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:567
    (0xB530, 0xB530, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:568
    (0xB531, 0xB54B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:569
    (0xB54C, 0xB54C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:570
    (0xB54D, 0xB567, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:571
    (0xB568, 0xB568, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:572
    (0xB569, 0xB583, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:573
    (0xB584, 0xB584, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:574
    (0xB585, 0xB59F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:575
    (0xB5A0, 0xB5A0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:576
    (0xB5A1, 0xB5BB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:577
    (0xB5BC, 0xB5BC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:578
    (0xB5BD, 0xB5D7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:579
    (0xB5D8, 0xB5D8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:580
    (0xB5D9, 0xB5F3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:581
    (0xB5F4, 0xB5F4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:582
    (0xB5F5, 0xB60F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:583
    (0xB610, 0xB610, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:584
    (0xB611, 0xB62B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:585
    (0xB62C, 0xB62C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:586
    (0xB62D, 0xB647, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:587
    (0xB648, 0xB648, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:588
    (0xB649, 0xB663, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:589
    (0xB664, 0xB664, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:590
    (0xB665, 0xB67F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:591
    (0xB680, 0xB680, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:592
    (0xB681, 0xB69B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:593
    (0xB69C, 0xB69C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:594
    (0xB69D, 0xB6B7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:595
    (0xB6B8, 0xB6B8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:596
    (0xB6B9, 0xB6D3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:597
    (0xB6D4, 0xB6D4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:598
    (0xB6D5, 0xB6EF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:599
    (0xB6F0, 0xB6F0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:600
    (0xB6F1, 0xB70B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:601
    (0xB70C, 0xB70C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:602
    (0xB70D, 0xB727, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:603
    (0xB728, 0xB728, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:604
    (0xB729, 0xB743, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:605
    (0xB744, 0xB744, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:606
    (0xB745, 0xB75F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:607
    (0xB760, 0xB760, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:608
    (0xB761, 0xB77B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:609
    (0xB77C, 0xB77C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:610
    (0xB77D, 0xB797, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:611
    (0xB798, 0xB798, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:612
    (0xB799, 0xB7B3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:613
    (0xB7B4, 0xB7B4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:614
    (0xB7B5, 0xB7CF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:615
    (0xB7D0, 0xB7D0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:616
    (0xB7D1, 0xB7EB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:617
    (0xB7EC, 0xB7EC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:618
    (0xB7ED, 0xB807, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:619
    (0xB808, 0xB808, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:620
    (0xB809, 0xB823, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:621
    (0xB824, 0xB824, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:622
    (0xB825, 0xB83F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:623
    (0xB840, 0xB840, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:624
    (0xB841, 0xB85B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:625
    (0xB85C, 0xB85C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:626
    (0xB85D, 0xB877, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:627
    (0xB878, 0xB878, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:628
    (0xB879, 0xB893, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:629
    (0xB894, 0xB894, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:630
    (0xB895, 0xB8AF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:631
    (0xB8B0, 0xB8B0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:632
    (0xB8B1, 0xB8CB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:633
    (0xB8CC, 0xB8CC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:634
    (0xB8CD, 0xB8E7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:635
    (0xB8E8, 0xB8E8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:636
    (0xB8E9, 0xB903, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:637
    (0xB904, 0xB904, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:638
    (0xB905, 0xB91F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:639
    (0xB920, 0xB920, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:640
    (0xB921, 0xB93B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:641
    (0xB93C, 0xB93C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:642
    (0xB93D, 0xB957, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:643
    (0xB958, 0xB958, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:644
    (0xB959, 0xB973, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:645
    (0xB974, 0xB974, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:646
    (0xB975, 0xB98F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:647
    (0xB990, 0xB990, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:648
    (0xB991, 0xB9AB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:649
    (0xB9AC, 0xB9AC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:650
    (0xB9AD, 0xB9C7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:651
    (0xB9C8, 0xB9C8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:652
    (0xB9C9, 0xB9E3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:653
    (0xB9E4, 0xB9E4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:654
    (0xB9E5, 0xB9FF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:655
    (0xBA00, 0xBA00, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:656
    (0xBA01, 0xBA1B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:657
    (0xBA1C, 0xBA1C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:658
    (0xBA1D, 0xBA37, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:659
    (0xBA38, 0xBA38, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:660
    (0xBA39, 0xBA53, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:661
    (0xBA54, 0xBA54, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:662
    (0xBA55, 0xBA6F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:663
    (0xBA70, 0xBA70, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:664
    (0xBA71, 0xBA8B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:665
    (0xBA8C, 0xBA8C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:666
    (0xBA8D, 0xBAA7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:667
    (0xBAA8, 0xBAA8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:668
    (0xBAA9, 0xBAC3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:669
    (0xBAC4, 0xBAC4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:670
    (0xBAC5, 0xBADF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:671
    (0xBAE0, 0xBAE0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:672
    (0xBAE1, 0xBAFB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:673
    (0xBAFC, 0xBAFC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:674
    (0xBAFD, 0xBB17, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:675
    (0xBB18, 0xBB18, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:676
    (0xBB19, 0xBB33, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:677
    (0xBB34, 0xBB34, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:678
    (0xBB35, 0xBB4F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:679
    (0xBB50, 0xBB50, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:680
    (0xBB51, 0xBB6B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:681
    (0xBB6C, 0xBB6C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:682
    (0xBB6D, 0xBB87, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:683
    (0xBB88, 0xBB88, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:684
    (0xBB89, 0xBBA3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:685
    (0xBBA4, 0xBBA4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:686
    (0xBBA5, 0xBBBF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:687
    (0xBBC0, 0xBBC0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:688
    (0xBBC1, 0xBBDB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:689
    (0xBBDC, 0xBBDC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:690
    (0xBBDD, 0xBBF7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:691
    (0xBBF8, 0xBBF8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:692
    (0xBBF9, 0xBC13, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:693
    (0xBC14, 0xBC14, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:694
    (0xBC15, 0xBC2F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:695
    (0xBC30, 0xBC30, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:696
    (0xBC31, 0xBC4B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:697
    (0xBC4C, 0xBC4C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:698
    (0xBC4D, 0xBC67, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:699
    (0xBC68, 0xBC68, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:700
    (0xBC69, 0xBC83, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:701
    (0xBC84, 0xBC84, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:702
    (0xBC85, 0xBC9F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:703
    (0xBCA0, 0xBCA0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:704
    (0xBCA1, 0xBCBB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:705
    (0xBCBC, 0xBCBC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:706
    (0xBCBD, 0xBCD7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:707
    (0xBCD8, 0xBCD8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:708
    (0xBCD9, 0xBCF3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:709
    (0xBCF4, 0xBCF4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:710
    (0xBCF5, 0xBD0F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:711
    (0xBD10, 0xBD10, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:712
    (0xBD11, 0xBD2B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:713
    (0xBD2C, 0xBD2C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:714
    (0xBD2D, 0xBD47, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:715
    (0xBD48, 0xBD48, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:716
    (0xBD49, 0xBD63, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:717
    (0xBD64, 0xBD64, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:718
    (0xBD65, 0xBD7F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:719
    (0xBD80, 0xBD80, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:720
    (0xBD81, 0xBD9B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:721
    (0xBD9C, 0xBD9C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:722
    (0xBD9D, 0xBDB7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:723
    (0xBDB8, 0xBDB8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:724
    (0xBDB9, 0xBDD3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:725
    (0xBDD4, 0xBDD4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:726
    (0xBDD5, 0xBDEF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:727
    (0xBDF0, 0xBDF0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:728
    (0xBDF1, 0xBE0B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:729
    (0xBE0C, 0xBE0C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:730
    (0xBE0D, 0xBE27, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:731
    (0xBE28, 0xBE28, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:732
    (0xBE29, 0xBE43, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:733
    (0xBE44, 0xBE44, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:734
    (0xBE45, 0xBE5F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:735
    (0xBE60, 0xBE60, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:736
    (0xBE61, 0xBE7B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:737
    (0xBE7C, 0xBE7C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:738
    (0xBE7D, 0xBE97, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:739
    (0xBE98, 0xBE98, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:740
    (0xBE99, 0xBEB3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:741
    (0xBEB4, 0xBEB4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:742
    (0xBEB5, 0xBECF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:743
    (0xBED0, 0xBED0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:744
    (0xBED1, 0xBEEB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:745
    (0xBEEC, 0xBEEC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:746
    (0xBEED, 0xBF07, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:747
    (0xBF08, 0xBF08, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:748
    (0xBF09, 0xBF23, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:749
    (0xBF24, 0xBF24, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:750
    (0xBF25, 0xBF3F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:751
    (0xBF40, 0xBF40, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:752
    (0xBF41, 0xBF5B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:753
    (0xBF5C, 0xBF5C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:754
    (0xBF5D, 0xBF77, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:755
    (0xBF78, 0xBF78, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:756
    (0xBF79, 0xBF93, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:757
    (0xBF94, 0xBF94, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:758
    (0xBF95, 0xBFAF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:759
    (0xBFB0, 0xBFB0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:760
    (0xBFB1, 0xBFCB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:761
    (0xBFCC, 0xBFCC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:762
    (0xBFCD, 0xBFE7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:763
    (0xBFE8, 0xBFE8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:764
    (0xBFE9, 0xC003, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:765
    (0xC004, 0xC004, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:766
    (0xC005, 0xC01F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:767
    (0xC020, 0xC020, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:768
    (0xC021, 0xC03B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:769
    (0xC03C, 0xC03C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:770
    (0xC03D, 0xC057, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:771
    (0xC058, 0xC058, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:772
    (0xC059, 0xC073, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:773
    (0xC074, 0xC074, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:774
    (0xC075, 0xC08F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:775
    (0xC090, 0xC090, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:776
    (0xC091, 0xC0AB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:777
    (0xC0AC, 0xC0AC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:778
    (0xC0AD, 0xC0C7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:779
    (0xC0C8, 0xC0C8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:780
    (0xC0C9, 0xC0E3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:781
    (0xC0E4, 0xC0E4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:782
    (0xC0E5, 0xC0FF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:783
    (0xC100, 0xC100, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:784
    (0xC101, 0xC11B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:785
    (0xC11C, 0xC11C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:786
    (0xC11D, 0xC137, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:787
    (0xC138, 0xC138, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:788
    (0xC139, 0xC153, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:789
    (0xC154, 0xC154, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:790
    (0xC155, 0xC16F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:791
    (0xC170, 0xC170, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:792
    (0xC171, 0xC18B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:793
    (0xC18C, 0xC18C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:794
    (0xC18D, 0xC1A7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:795
    (0xC1A8, 0xC1A8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:796
    (0xC1A9, 0xC1C3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:797
    (0xC1C4, 0xC1C4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:798
    (0xC1C5, 0xC1DF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:799
    (0xC1E0, 0xC1E0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:800
    (0xC1E1, 0xC1FB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:801
    (0xC1FC, 0xC1FC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:802
    (0xC1FD, 0xC217, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:803
    (0xC218, 0xC218, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:804
    (0xC219, 0xC233, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:805
    (0xC234, 0xC234, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:806
    (0xC235, 0xC24F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:807
    (0xC250, 0xC250, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:808
    (0xC251, 0xC26B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:809
    (0xC26C, 0xC26C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:810
    (0xC26D, 0xC287, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:811
    (0xC288, 0xC288, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:812
    (0xC289, 0xC2A3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:813
    (0xC2A4, 0xC2A4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:814
    (0xC2A5, 0xC2BF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:815
    (0xC2C0, 0xC2C0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:816
    (0xC2C1, 0xC2DB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:817
    (0xC2DC, 0xC2DC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:818
    (0xC2DD, 0xC2F7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:819
    (0xC2F8, 0xC2F8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:820
    (0xC2F9, 0xC313, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:821
    (0xC314, 0xC314, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:822
    (0xC315, 0xC32F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:823
    (0xC330, 0xC330, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:824
    (0xC331, 0xC34B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:825
    (0xC34C, 0xC34C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:826
    (0xC34D, 0xC367, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:827
    (0xC368, 0xC368, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:828
    (0xC369, 0xC383, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:829
    (0xC384, 0xC384, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:830
    (0xC385, 0xC39F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:831
    (0xC3A0, 0xC3A0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:832
    (0xC3A1, 0xC3BB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:833
    (0xC3BC, 0xC3BC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:834
    (0xC3BD, 0xC3D7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:835
    (0xC3D8, 0xC3D8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:836
    (0xC3D9, 0xC3F3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:837
    (0xC3F4, 0xC3F4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:838
    (0xC3F5, 0xC40F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:839
    (0xC410, 0xC410, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:840
    (0xC411, 0xC42B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:841
    (0xC42C, 0xC42C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:842
    (0xC42D, 0xC447, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:843
    (0xC448, 0xC448, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:844
    (0xC449, 0xC463, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:845
    (0xC464, 0xC464, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:846
    (0xC465, 0xC47F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:847
    (0xC480, 0xC480, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:848
    (0xC481, 0xC49B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:849
    (0xC49C, 0xC49C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:850
    (0xC49D, 0xC4B7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:851
    (0xC4B8, 0xC4B8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:852
    (0xC4B9, 0xC4D3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:853
    (0xC4D4, 0xC4D4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:854
    (0xC4D5, 0xC4EF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:855
    (0xC4F0, 0xC4F0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:856
    (0xC4F1, 0xC50B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:857
    (0xC50C, 0xC50C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:858
    (0xC50D, 0xC527, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:859
    (0xC528, 0xC528, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:860
    (0xC529, 0xC543, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:861
    (0xC544, 0xC544, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:862
    (0xC545, 0xC55F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:863
    (0xC560, 0xC560, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:864
    (0xC561, 0xC57B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:865
    (0xC57C, 0xC57C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:866
    (0xC57D, 0xC597, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:867
    (0xC598, 0xC598, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:868
    (0xC599, 0xC5B3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:869
    (0xC5B4, 0xC5B4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:870
    (0xC5B5, 0xC5CF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:871
    (0xC5D0, 0xC5D0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:872
    (0xC5D1, 0xC5EB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:873
    (0xC5EC, 0xC5EC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:874
    (0xC5ED, 0xC607, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:875
    (0xC608, 0xC608, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:876
    (0xC609, 0xC623, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:877
    (0xC624, 0xC624, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:878
    (0xC625, 0xC63F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:879
    (0xC640, 0xC640, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:880
    (0xC641, 0xC65B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:881
    (0xC65C, 0xC65C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:882
    (0xC65D, 0xC677, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:883
    (0xC678, 0xC678, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:884
    (0xC679, 0xC693, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:885
    (0xC694, 0xC694, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:886
    (0xC695, 0xC6AF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:887
    (0xC6B0, 0xC6B0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:888
    (0xC6B1, 0xC6CB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:889
    (0xC6CC, 0xC6CC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:890
    (0xC6CD, 0xC6E7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:891
    (0xC6E8, 0xC6E8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:892
    (0xC6E9, 0xC703, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:893
    (0xC704, 0xC704, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:894
    (0xC705, 0xC71F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:895
    (0xC720, 0xC720, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:896
    (0xC721, 0xC73B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:897
    (0xC73C, 0xC73C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:898
    (0xC73D, 0xC757, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:899
    (0xC758, 0xC758, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:900
    (0xC759, 0xC773, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:901
    (0xC774, 0xC774, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:902
    (0xC775, 0xC78F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:903
    (0xC790, 0xC790, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:904
    (0xC791, 0xC7AB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:905
    (0xC7AC, 0xC7AC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:906
    (0xC7AD, 0xC7C7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:907
    (0xC7C8, 0xC7C8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:908
    (0xC7C9, 0xC7E3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:909
    (0xC7E4, 0xC7E4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:910
    (0xC7E5, 0xC7FF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:911
    (0xC800, 0xC800, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:912
    (0xC801, 0xC81B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:913
    (0xC81C, 0xC81C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:914
    (0xC81D, 0xC837, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:915
    (0xC838, 0xC838, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:916
    (0xC839, 0xC853, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:917
    (0xC854, 0xC854, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:918
    (0xC855, 0xC86F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:919
    (0xC870, 0xC870, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:920
    (0xC871, 0xC88B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:921
    (0xC88C, 0xC88C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:922
    (0xC88D, 0xC8A7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:923
    (0xC8A8, 0xC8A8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:924
    (0xC8A9, 0xC8C3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:925
    (0xC8C4, 0xC8C4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:926
    (0xC8C5, 0xC8DF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:927
    (0xC8E0, 0xC8E0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:928
    (0xC8E1, 0xC8FB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:929
    (0xC8FC, 0xC8FC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:930
    (0xC8FD, 0xC917, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:931
    (0xC918, 0xC918, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:932
    (0xC919, 0xC933, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:933
    (0xC934, 0xC934, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:934
    (0xC935, 0xC94F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:935
    (0xC950, 0xC950, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:936
    (0xC951, 0xC96B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:937
    (0xC96C, 0xC96C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:938
    (0xC96D, 0xC987, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:939
    (0xC988, 0xC988, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:940
    (0xC989, 0xC9A3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:941
    (0xC9A4, 0xC9A4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:942
    (0xC9A5, 0xC9BF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:943
    (0xC9C0, 0xC9C0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:944
    (0xC9C1, 0xC9DB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:945
    (0xC9DC, 0xC9DC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:946
    (0xC9DD, 0xC9F7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:947
    (0xC9F8, 0xC9F8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:948
    (0xC9F9, 0xCA13, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:949
    (0xCA14, 0xCA14, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:950
    (0xCA15, 0xCA2F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:951
    (0xCA30, 0xCA30, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:952
    (0xCA31, 0xCA4B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:953
    (0xCA4C, 0xCA4C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:954
    (0xCA4D, 0xCA67, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:955
    (0xCA68, 0xCA68, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:956
    (0xCA69, 0xCA83, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:957
    (0xCA84, 0xCA84, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:958
    (0xCA85, 0xCA9F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:959
    (0xCAA0, 0xCAA0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:960
    (0xCAA1, 0xCABB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:961
    (0xCABC, 0xCABC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:962
    (0xCABD, 0xCAD7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:963
    (0xCAD8, 0xCAD8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:964
    (0xCAD9, 0xCAF3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:965
    (0xCAF4, 0xCAF4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:966
    (0xCAF5, 0xCB0F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:967
    (0xCB10, 0xCB10, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:968
    (0xCB11, 0xCB2B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:969
    (0xCB2C, 0xCB2C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:970
    (0xCB2D, 0xCB47, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:971
    (0xCB48, 0xCB48, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:972
    (0xCB49, 0xCB63, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:973
    (0xCB64, 0xCB64, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:974
    (0xCB65, 0xCB7F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:975
    (0xCB80, 0xCB80, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:976
    (0xCB81, 0xCB9B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:977
    (0xCB9C, 0xCB9C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:978
    (0xCB9D, 0xCBB7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:979
    (0xCBB8, 0xCBB8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:980
    (0xCBB9, 0xCBD3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:981
    (0xCBD4, 0xCBD4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:982
    (0xCBD5, 0xCBEF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:983
    (0xCBF0, 0xCBF0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:984
    (0xCBF1, 0xCC0B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:985
    (0xCC0C, 0xCC0C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:986
    (0xCC0D, 0xCC27, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:987
    (0xCC28, 0xCC28, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:988
    (0xCC29, 0xCC43, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:989
    (0xCC44, 0xCC44, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:990
    (0xCC45, 0xCC5F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:991
    (0xCC60, 0xCC60, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:992
    (0xCC61, 0xCC7B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:993
    (0xCC7C, 0xCC7C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:994
    (0xCC7D, 0xCC97, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:995
    (0xCC98, 0xCC98, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:996
    (0xCC99, 0xCCB3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:997
    (0xCCB4, 0xCCB4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:998
    (0xCCB5, 0xCCCF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:999
    (0xCCD0, 0xCCD0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1000
    (0xCCD1, 0xCCEB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1001
    (0xCCEC, 0xCCEC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1002
    (0xCCED, 0xCD07, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1003
    (0xCD08, 0xCD08, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1004
    (0xCD09, 0xCD23, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1005
    (0xCD24, 0xCD24, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1006
    (0xCD25, 0xCD3F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1007
    (0xCD40, 0xCD40, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1008
    (0xCD41, 0xCD5B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1009
    (0xCD5C, 0xCD5C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1010
    (0xCD5D, 0xCD77, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1011
    (0xCD78, 0xCD78, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1012
    (0xCD79, 0xCD93, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1013
    (0xCD94, 0xCD94, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1014
    (0xCD95, 0xCDAF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1015
    (0xCDB0, 0xCDB0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1016
    (0xCDB1, 0xCDCB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1017
    (0xCDCC, 0xCDCC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1018
    (0xCDCD, 0xCDE7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1019
    (0xCDE8, 0xCDE8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1020
    (0xCDE9, 0xCE03, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1021
    (0xCE04, 0xCE04, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1022
    (0xCE05, 0xCE1F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1023
    (0xCE20, 0xCE20, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1024
    (0xCE21, 0xCE3B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1025
    (0xCE3C, 0xCE3C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1026
    (0xCE3D, 0xCE57, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1027
    (0xCE58, 0xCE58, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1028
    (0xCE59, 0xCE73, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1029
    (0xCE74, 0xCE74, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1030
    (0xCE75, 0xCE8F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1031
    (0xCE90, 0xCE90, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1032
    (0xCE91, 0xCEAB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1033
    (0xCEAC, 0xCEAC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1034
    (0xCEAD, 0xCEC7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1035
    (0xCEC8, 0xCEC8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1036
    (0xCEC9, 0xCEE3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1037
    (0xCEE4, 0xCEE4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1038
    (0xCEE5, 0xCEFF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1039
    (0xCF00, 0xCF00, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1040
    (0xCF01, 0xCF1B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1041
    (0xCF1C, 0xCF1C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1042
    (0xCF1D, 0xCF37, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1043
    (0xCF38, 0xCF38, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1044
    (0xCF39, 0xCF53, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1045
    (0xCF54, 0xCF54, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1046
    (0xCF55, 0xCF6F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1047
    (0xCF70, 0xCF70, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1048
    (0xCF71, 0xCF8B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1049
    (0xCF8C, 0xCF8C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1050
    (0xCF8D, 0xCFA7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1051
    (0xCFA8, 0xCFA8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1052
    (0xCFA9, 0xCFC3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1053
    (0xCFC4, 0xCFC4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1054
    (0xCFC5, 0xCFDF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1055
    (0xCFE0, 0xCFE0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1056
    (0xCFE1, 0xCFFB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1057
    (0xCFFC, 0xCFFC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1058
    (0xCFFD, 0xD017, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1059
    (0xD018, 0xD018, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1060
    (0xD019, 0xD033, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1061
    (0xD034, 0xD034, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1062
    (0xD035, 0xD04F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1063
    (0xD050, 0xD050, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1064
    (0xD051, 0xD06B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1065
    (0xD06C, 0xD06C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1066
    (0xD06D, 0xD087, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1067
    (0xD088, 0xD088, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1068
    (0xD089, 0xD0A3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1069
    (0xD0A4, 0xD0A4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1070
    (0xD0A5, 0xD0BF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1071
    (0xD0C0, 0xD0C0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1072
    (0xD0C1, 0xD0DB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1073
    (0xD0DC, 0xD0DC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1074
    (0xD0DD, 0xD0F7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1075
    (0xD0F8, 0xD0F8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1076
    (0xD0F9, 0xD113, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1077
    (0xD114, 0xD114, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1078
    (0xD115, 0xD12F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1079
    (0xD130, 0xD130, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1080
    (0xD131, 0xD14B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1081
    (0xD14C, 0xD14C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1082
    (0xD14D, 0xD167, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1083
    (0xD168, 0xD168, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1084
    (0xD169, 0xD183, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1085
    (0xD184, 0xD184, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1086
    (0xD185, 0xD19F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1087
    (0xD1A0, 0xD1A0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1088
    (0xD1A1, 0xD1BB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1089
    (0xD1BC, 0xD1BC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1090
    (0xD1BD, 0xD1D7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1091
    (0xD1D8, 0xD1D8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1092
    (0xD1D9, 0xD1F3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1093
    (0xD1F4, 0xD1F4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1094
    (0xD1F5, 0xD20F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1095
    (0xD210, 0xD210, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1096
    (0xD211, 0xD22B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1097
    (0xD22C, 0xD22C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1098
    (0xD22D, 0xD247, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1099
    (0xD248, 0xD248, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1100
    (0xD249, 0xD263, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1101
    (0xD264, 0xD264, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1102
    (0xD265, 0xD27F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1103
    (0xD280, 0xD280, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1104
    (0xD281, 0xD29B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1105
    (0xD29C, 0xD29C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1106
    (0xD29D, 0xD2B7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1107
    (0xD2B8, 0xD2B8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1108
    (0xD2B9, 0xD2D3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1109
    (0xD2D4, 0xD2D4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1110
    (0xD2D5, 0xD2EF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1111
    (0xD2F0, 0xD2F0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1112
    (0xD2F1, 0xD30B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1113
    (0xD30C, 0xD30C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1114
    (0xD30D, 0xD327, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1115
    (0xD328, 0xD328, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1116
    (0xD329, 0xD343, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1117
    (0xD344, 0xD344, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1118
    (0xD345, 0xD35F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1119
    (0xD360, 0xD360, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1120
    (0xD361, 0xD37B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1121
    (0xD37C, 0xD37C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1122
    (0xD37D, 0xD397, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1123
    (0xD398, 0xD398, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1124
    (0xD399, 0xD3B3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1125
    (0xD3B4, 0xD3B4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1126
    (0xD3B5, 0xD3CF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1127
    (0xD3D0, 0xD3D0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1128
    (0xD3D1, 0xD3EB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1129
    (0xD3EC, 0xD3EC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1130
    (0xD3ED, 0xD407, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1131
    (0xD408, 0xD408, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1132
    (0xD409, 0xD423, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1133
    (0xD424, 0xD424, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1134
    (0xD425, 0xD43F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1135
    (0xD440, 0xD440, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1136
    (0xD441, 0xD45B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1137
    (0xD45C, 0xD45C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1138
    (0xD45D, 0xD477, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1139
    (0xD478, 0xD478, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1140
    (0xD479, 0xD493, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1141
    (0xD494, 0xD494, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1142
    (0xD495, 0xD4AF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1143
    (0xD4B0, 0xD4B0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1144
    (0xD4B1, 0xD4CB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1145
    (0xD4CC, 0xD4CC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1146
    (0xD4CD, 0xD4E7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1147
    (0xD4E8, 0xD4E8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1148
    (0xD4E9, 0xD503, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1149
    (0xD504, 0xD504, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1150
    (0xD505, 0xD51F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1151
    (0xD520, 0xD520, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1152
    (0xD521, 0xD53B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1153
    (0xD53C, 0xD53C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1154
    (0xD53D, 0xD557, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1155
    (0xD558, 0xD558, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1156
    (0xD559, 0xD573, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1157
    (0xD574, 0xD574, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1158
    (0xD575, 0xD58F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1159
    (0xD590, 0xD590, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1160
    (0xD591, 0xD5AB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1161
    (0xD5AC, 0xD5AC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1162
    (0xD5AD, 0xD5C7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1163
    (0xD5C8, 0xD5C8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1164
    (0xD5C9, 0xD5E3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1165
    (0xD5E4, 0xD5E4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1166
    (0xD5E5, 0xD5FF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1167
    (0xD600, 0xD600, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1168
    (0xD601, 0xD61B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1169
    (0xD61C, 0xD61C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1170
    (0xD61D, 0xD637, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1171
    (0xD638, 0xD638, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1172
    (0xD639, 0xD653, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1173
    (0xD654, 0xD654, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1174
    (0xD655, 0xD66F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1175
    (0xD670, 0xD670, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1176
    (0xD671, 0xD68B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1177
    (0xD68C, 0xD68C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1178
    (0xD68D, 0xD6A7, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1179
    (0xD6A8, 0xD6A8, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1180
    (0xD6A9, 0xD6C3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1181
    (0xD6C4, 0xD6C4, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1182
    (0xD6C5, 0xD6DF, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1183
    (0xD6E0, 0xD6E0, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1184
    (0xD6E1, 0xD6FB, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1185
    (0xD6FC, 0xD6FC, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1186
    (0xD6FD, 0xD717, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1187
    (0xD718, 0xD718, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1188
    (0xD719, 0xD733, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1189
    (0xD734, 0xD734, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1190
    (0xD735, 0xD74F, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1191
    (0xD750, 0xD750, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1192
    (0xD751, 0xD76B, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1193
    (0xD76C, 0xD76C, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1194
    (0xD76D, 0xD787, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1195
    (0xD788, 0xD788, GraphemeProperty::kLv),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1196
    (0xD789, 0xD7A3, GraphemeProperty::kLvt),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1197
    (0xD7B0, 0xD7C6, GraphemeProperty::kV),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1198
    (0xD7CB, 0xD7FB, GraphemeProperty::kT),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1199
    (0xFB1E, 0xFB1E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1200
    (0xFE00, 0xFE0F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1201
    (0xFE20, 0xFE2F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1202
    (0xFEFF, 0xFEFF, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1203
    (0xFF9E, 0xFF9F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1204
    (0xFFF0, 0xFFF8, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1205
    (0xFFF9, 0xFFFB, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1206
    (0x101FD, 0x101FD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1207
    (0x102E0, 0x102E0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1208
    (0x10376, 0x1037A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1209
    (0x10A01, 0x10A03, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1210
    (0x10A05, 0x10A06, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1211
    (0x10A0C, 0x10A0F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1212
    (0x10A38, 0x10A3A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1213
    (0x10A3F, 0x10A3F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1214
    (0x10AE5, 0x10AE6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1215
    (0x10D24, 0x10D27, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1216
    (0x10D69, 0x10D6D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1217
    (0x10EAB, 0x10EAC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1218
    (0x10EFC, 0x10EFF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1219
    (0x10F46, 0x10F50, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1220
    (0x10F82, 0x10F85, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1221
    (0x11000, 0x11000, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1222
    (0x11001, 0x11001, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1223
    (0x11002, 0x11002, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1224
    (0x11038, 0x11046, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1225
    (0x11070, 0x11070, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1226
    (0x11073, 0x11074, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1227
    (0x1107F, 0x11081, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1228
    (0x11082, 0x11082, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1229
    (0x110B0, 0x110B2, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1230
    (0x110B3, 0x110B6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1231
    (0x110B7, 0x110B8, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1232
    (0x110B9, 0x110BA, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1233
    (0x110BD, 0x110BD, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1234
    (0x110C2, 0x110C2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1235
    (0x110CD, 0x110CD, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1236
    (0x11100, 0x11102, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1237
    (0x11127, 0x1112B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1238
    (0x1112C, 0x1112C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1239
    (0x1112D, 0x11134, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1240
    (0x11145, 0x11146, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1241
    (0x11173, 0x11173, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1242
    (0x11180, 0x11181, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1243
    (0x11182, 0x11182, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1244
    (0x111B3, 0x111B5, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1245
    (0x111B6, 0x111BE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1246
    (0x111BF, 0x111BF, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1247
    (0x111C0, 0x111C0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1248
    (0x111C2, 0x111C3, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1249
    (0x111C9, 0x111CC, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1250
    (0x111CE, 0x111CE, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1251
    (0x111CF, 0x111CF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1252
    (0x1122C, 0x1122E, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1253
    (0x1122F, 0x11231, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1254
    (0x11232, 0x11233, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1255
    (0x11234, 0x11234, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1256
    (0x11235, 0x11235, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1257
    (0x11236, 0x11237, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1258
    (0x1123E, 0x1123E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1259
    (0x11241, 0x11241, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1260
    (0x112DF, 0x112DF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1261
    (0x112E0, 0x112E2, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1262
    (0x112E3, 0x112EA, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1263
    (0x11300, 0x11301, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1264
    (0x11302, 0x11303, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1265
    (0x1133B, 0x1133C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1266
    (0x1133E, 0x1133E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1267
    (0x1133F, 0x1133F, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1268
    (0x11340, 0x11340, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1269
    (0x11341, 0x11344, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1270
    (0x11347, 0x11348, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1271
    (0x1134B, 0x1134C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1272
    (0x1134D, 0x1134D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1273
    (0x11357, 0x11357, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1274
    (0x11362, 0x11363, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1275
    (0x11366, 0x1136C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1276
    (0x11370, 0x11374, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1277
    (0x113B8, 0x113B8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1278
    (0x113B9, 0x113BA, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1279
    (0x113BB, 0x113C0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1280
    (0x113C2, 0x113C2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1281
    (0x113C5, 0x113C5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1282
    (0x113C7, 0x113C9, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1283
    (0x113CA, 0x113CA, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1284
    (0x113CC, 0x113CD, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1285
    (0x113CE, 0x113CE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1286
    (0x113CF, 0x113CF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1287
    (0x113D0, 0x113D0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1288
    (0x113D1, 0x113D1, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1289
    (0x113D2, 0x113D2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1290
    (0x113E1, 0x113E2, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1291
    (0x11435, 0x11437, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1292
    (0x11438, 0x1143F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1293
    (0x11440, 0x11441, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1294
    (0x11442, 0x11444, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1295
    (0x11445, 0x11445, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1296
    (0x11446, 0x11446, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1297
    (0x1145E, 0x1145E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1298
    (0x114B0, 0x114B0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1299
    (0x114B1, 0x114B2, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1300
    (0x114B3, 0x114B8, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1301
    (0x114B9, 0x114B9, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1302
    (0x114BA, 0x114BA, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1303
    (0x114BB, 0x114BC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1304
    (0x114BD, 0x114BD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1305
    (0x114BE, 0x114BE, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1306
    (0x114BF, 0x114C0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1307
    (0x114C1, 0x114C1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1308
    (0x114C2, 0x114C3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1309
    (0x115AF, 0x115AF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1310
    (0x115B0, 0x115B1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1311
    (0x115B2, 0x115B5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1312
    (0x115B8, 0x115BB, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1313
    (0x115BC, 0x115BD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1314
    (0x115BE, 0x115BE, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1315
    (0x115BF, 0x115C0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1316
    (0x115DC, 0x115DD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1317
    (0x11630, 0x11632, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1318
    (0x11633, 0x1163A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1319
    (0x1163B, 0x1163C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1320
    (0x1163D, 0x1163D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1321
    (0x1163E, 0x1163E, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1322
    (0x1163F, 0x11640, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1323
    (0x116AB, 0x116AB, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1324
    (0x116AC, 0x116AC, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1325
    (0x116AD, 0x116AD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1326
    (0x116AE, 0x116AF, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1327
    (0x116B0, 0x116B5, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1328
    (0x116B6, 0x116B6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1329
    (0x116B7, 0x116B7, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1330
    (0x1171D, 0x1171D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1331
    (0x1171E, 0x1171E, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1332
    (0x1171F, 0x1171F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1333
    (0x11722, 0x11725, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1334
    (0x11726, 0x11726, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1335
    (0x11727, 0x1172B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1336
    (0x1182C, 0x1182E, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1337
    (0x1182F, 0x11837, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1338
    (0x11838, 0x11838, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1339
    (0x11839, 0x1183A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1340
    (0x11930, 0x11930, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1341
    (0x11931, 0x11935, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1342
    (0x11937, 0x11938, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1343
    (0x1193B, 0x1193C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1344
    (0x1193D, 0x1193D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1345
    (0x1193E, 0x1193E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1346
    (0x1193F, 0x1193F, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1347
    (0x11940, 0x11940, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1348
    (0x11941, 0x11941, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1349
    (0x11942, 0x11942, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1350
    (0x11943, 0x11943, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1351
    (0x119D1, 0x119D3, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1352
    (0x119D4, 0x119D7, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1353
    (0x119DA, 0x119DB, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1354
    (0x119DC, 0x119DF, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1355
    (0x119E0, 0x119E0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1356
    (0x119E4, 0x119E4, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1357
    (0x11A01, 0x11A0A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1358
    (0x11A33, 0x11A38, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1359
    (0x11A39, 0x11A39, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1360
    (0x11A3A, 0x11A3A, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1361
    (0x11A3B, 0x11A3E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1362
    (0x11A47, 0x11A47, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1363
    (0x11A51, 0x11A56, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1364
    (0x11A57, 0x11A58, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1365
    (0x11A59, 0x11A5B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1366
    (0x11A84, 0x11A89, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1367
    (0x11A8A, 0x11A96, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1368
    (0x11A97, 0x11A97, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1369
    (0x11A98, 0x11A99, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1370
    (0x11C2F, 0x11C2F, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1371
    (0x11C30, 0x11C36, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1372
    (0x11C38, 0x11C3D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1373
    (0x11C3E, 0x11C3E, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1374
    (0x11C3F, 0x11C3F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1375
    (0x11C92, 0x11CA7, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1376
    (0x11CA9, 0x11CA9, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1377
    (0x11CAA, 0x11CB0, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1378
    (0x11CB1, 0x11CB1, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1379
    (0x11CB2, 0x11CB3, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1380
    (0x11CB4, 0x11CB4, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1381
    (0x11CB5, 0x11CB6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1382
    (0x11D31, 0x11D36, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1383
    (0x11D3A, 0x11D3A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1384
    (0x11D3C, 0x11D3D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1385
    (0x11D3F, 0x11D45, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1386
    (0x11D46, 0x11D46, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1387
    (0x11D47, 0x11D47, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1388
    (0x11D8A, 0x11D8E, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1389
    (0x11D90, 0x11D91, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1390
    (0x11D93, 0x11D94, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1391
    (0x11D95, 0x11D95, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1392
    (0x11D96, 0x11D96, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1393
    (0x11D97, 0x11D97, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1394
    (0x11EF3, 0x11EF4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1395
    (0x11EF5, 0x11EF6, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1396
    (0x11F00, 0x11F01, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1397
    (0x11F02, 0x11F02, GraphemeProperty::kPrepend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1398
    (0x11F03, 0x11F03, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1399
    (0x11F34, 0x11F35, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1400
    (0x11F36, 0x11F3A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1401
    (0x11F3E, 0x11F3F, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1402
    (0x11F40, 0x11F40, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1403
    (0x11F41, 0x11F41, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1404
    (0x11F42, 0x11F42, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1405
    (0x11F5A, 0x11F5A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1406
    (0x13430, 0x1343F, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1407
    (0x13440, 0x13440, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1408
    (0x13447, 0x13455, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1409
    (0x1611E, 0x16129, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1410
    (0x1612A, 0x1612C, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1411
    (0x1612D, 0x1612F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1412
    (0x16AF0, 0x16AF4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1413
    (0x16B30, 0x16B36, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1414
    (0x16D63, 0x16D63, GraphemeProperty::kV),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1415
    (0x16D67, 0x16D6A, GraphemeProperty::kV),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1416
    (0x16F4F, 0x16F4F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1417
    (0x16F51, 0x16F87, GraphemeProperty::kSpacingMark),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1418
    (0x16F8F, 0x16F92, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1419
    (0x16FE4, 0x16FE4, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1420
    (0x16FF0, 0x16FF1, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1421
    (0x1BC9D, 0x1BC9E, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1422
    (0x1BCA0, 0x1BCA3, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1423
    (0x1CF00, 0x1CF2D, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1424
    (0x1CF30, 0x1CF46, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1425
    (0x1D165, 0x1D166, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1426
    (0x1D167, 0x1D169, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1427
    (0x1D16D, 0x1D172, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1428
    (0x1D173, 0x1D17A, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1429
    (0x1D17B, 0x1D182, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1430
    (0x1D185, 0x1D18B, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1431
    (0x1D1AA, 0x1D1AD, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1432
    (0x1D242, 0x1D244, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1433
    (0x1DA00, 0x1DA36, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1434
    (0x1DA3B, 0x1DA6C, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1435
    (0x1DA75, 0x1DA75, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1436
    (0x1DA84, 0x1DA84, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1437
    (0x1DA9B, 0x1DA9F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1438
    (0x1DAA1, 0x1DAAF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1439
    (0x1E000, 0x1E006, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1440
    (0x1E008, 0x1E018, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1441
    (0x1E01B, 0x1E021, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1442
    (0x1E023, 0x1E024, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1443
    (0x1E026, 0x1E02A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1444
    (0x1E08F, 0x1E08F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1445
    (0x1E130, 0x1E136, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1446
    (0x1E2AE, 0x1E2AE, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1447
    (0x1E2EC, 0x1E2EF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1448
    (0x1E4EC, 0x1E4EF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1449
    (0x1E5EE, 0x1E5EF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1450
    (0x1E8D0, 0x1E8D6, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1451
    (0x1E944, 0x1E94A, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1452
    (0x1F1E6, 0x1F1FF, GraphemeProperty::kRegionalIndicator),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1453
    (0x1F3FB, 0x1F3FF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1454
    (0xE0000, 0xE0000, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1455
    (0xE0001, 0xE0001, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1456
    (0xE0002, 0xE001F, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1457
    (0xE0020, 0xE007F, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1458
    (0xE0080, 0xE00FF, GraphemeProperty::kControl),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1459
    (0xE0100, 0xE01EF, GraphemeProperty::kExtend),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1460
    (0xE01F0, 0xE0FFF, GraphemeProperty::kControl),
];

// cpp: font_engine/text/native/grapheme_break_property_data.h:1462
const kExtendedPictographicRanges: [(u32, u32); 517] = [
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1463
    (0xA9, 0xA9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1464
    (0xAE, 0xAE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1465
    (0x203C, 0x203C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1466
    (0x2049, 0x2049),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1467
    (0x2122, 0x2122),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1468
    (0x2139, 0x2139),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1469
    (0x2194, 0x2199),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1470
    (0x21A9, 0x21AA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1471
    (0x231A, 0x231B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1472
    (0x2328, 0x2328),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1473
    (0x2388, 0x2388),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1474
    (0x23CF, 0x23CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1475
    (0x23E9, 0x23EC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1476
    (0x23ED, 0x23EE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1477
    (0x23EF, 0x23EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1478
    (0x23F0, 0x23F0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1479
    (0x23F1, 0x23F2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1480
    (0x23F3, 0x23F3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1481
    (0x23F8, 0x23FA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1482
    (0x24C2, 0x24C2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1483
    (0x25AA, 0x25AB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1484
    (0x25B6, 0x25B6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1485
    (0x25C0, 0x25C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1486
    (0x25FB, 0x25FE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1487
    (0x2600, 0x2601),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1488
    (0x2602, 0x2603),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1489
    (0x2604, 0x2604),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1490
    (0x2605, 0x2605),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1491
    (0x2607, 0x260D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1492
    (0x260E, 0x260E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1493
    (0x260F, 0x2610),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1494
    (0x2611, 0x2611),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1495
    (0x2612, 0x2612),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1496
    (0x2614, 0x2615),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1497
    (0x2616, 0x2617),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1498
    (0x2618, 0x2618),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1499
    (0x2619, 0x261C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1500
    (0x261D, 0x261D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1501
    (0x261E, 0x261F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1502
    (0x2620, 0x2620),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1503
    (0x2621, 0x2621),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1504
    (0x2622, 0x2623),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1505
    (0x2624, 0x2625),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1506
    (0x2626, 0x2626),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1507
    (0x2627, 0x2629),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1508
    (0x262A, 0x262A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1509
    (0x262B, 0x262D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1510
    (0x262E, 0x262E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1511
    (0x262F, 0x262F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1512
    (0x2630, 0x2637),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1513
    (0x2638, 0x2639),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1514
    (0x263A, 0x263A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1515
    (0x263B, 0x263F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1516
    (0x2640, 0x2640),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1517
    (0x2641, 0x2641),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1518
    (0x2642, 0x2642),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1519
    (0x2643, 0x2647),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1520
    (0x2648, 0x2653),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1521
    (0x2654, 0x265E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1522
    (0x265F, 0x265F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1523
    (0x2660, 0x2660),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1524
    (0x2661, 0x2662),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1525
    (0x2663, 0x2663),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1526
    (0x2664, 0x2664),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1527
    (0x2665, 0x2666),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1528
    (0x2667, 0x2667),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1529
    (0x2668, 0x2668),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1530
    (0x2669, 0x267A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1531
    (0x267B, 0x267B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1532
    (0x267C, 0x267D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1533
    (0x267E, 0x267E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1534
    (0x267F, 0x267F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1535
    (0x2680, 0x2685),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1536
    (0x2690, 0x2691),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1537
    (0x2692, 0x2692),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1538
    (0x2693, 0x2693),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1539
    (0x2694, 0x2694),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1540
    (0x2695, 0x2695),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1541
    (0x2696, 0x2697),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1542
    (0x2698, 0x2698),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1543
    (0x2699, 0x2699),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1544
    (0x269A, 0x269A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1545
    (0x269B, 0x269C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1546
    (0x269D, 0x269F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1547
    (0x26A0, 0x26A1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1548
    (0x26A2, 0x26A6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1549
    (0x26A7, 0x26A7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1550
    (0x26A8, 0x26A9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1551
    (0x26AA, 0x26AB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1552
    (0x26AC, 0x26AF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1553
    (0x26B0, 0x26B1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1554
    (0x26B2, 0x26BC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1555
    (0x26BD, 0x26BE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1556
    (0x26BF, 0x26C3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1557
    (0x26C4, 0x26C5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1558
    (0x26C6, 0x26C7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1559
    (0x26C8, 0x26C8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1560
    (0x26C9, 0x26CD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1561
    (0x26CE, 0x26CE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1562
    (0x26CF, 0x26CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1563
    (0x26D0, 0x26D0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1564
    (0x26D1, 0x26D1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1565
    (0x26D2, 0x26D2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1566
    (0x26D3, 0x26D3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1567
    (0x26D4, 0x26D4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1568
    (0x26D5, 0x26E8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1569
    (0x26E9, 0x26E9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1570
    (0x26EA, 0x26EA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1571
    (0x26EB, 0x26EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1572
    (0x26F0, 0x26F1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1573
    (0x26F2, 0x26F3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1574
    (0x26F4, 0x26F4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1575
    (0x26F5, 0x26F5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1576
    (0x26F6, 0x26F6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1577
    (0x26F7, 0x26F9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1578
    (0x26FA, 0x26FA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1579
    (0x26FB, 0x26FC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1580
    (0x26FD, 0x26FD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1581
    (0x26FE, 0x2701),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1582
    (0x2702, 0x2702),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1583
    (0x2703, 0x2704),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1584
    (0x2705, 0x2705),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1585
    (0x2708, 0x270C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1586
    (0x270D, 0x270D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1587
    (0x270E, 0x270E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1588
    (0x270F, 0x270F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1589
    (0x2710, 0x2711),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1590
    (0x2712, 0x2712),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1591
    (0x2714, 0x2714),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1592
    (0x2716, 0x2716),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1593
    (0x271D, 0x271D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1594
    (0x2721, 0x2721),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1595
    (0x2728, 0x2728),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1596
    (0x2733, 0x2734),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1597
    (0x2744, 0x2744),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1598
    (0x2747, 0x2747),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1599
    (0x274C, 0x274C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1600
    (0x274E, 0x274E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1601
    (0x2753, 0x2755),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1602
    (0x2757, 0x2757),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1603
    (0x2763, 0x2763),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1604
    (0x2764, 0x2764),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1605
    (0x2765, 0x2767),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1606
    (0x2795, 0x2797),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1607
    (0x27A1, 0x27A1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1608
    (0x27B0, 0x27B0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1609
    (0x27BF, 0x27BF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1610
    (0x2934, 0x2935),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1611
    (0x2B05, 0x2B07),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1612
    (0x2B1B, 0x2B1C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1613
    (0x2B50, 0x2B50),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1614
    (0x2B55, 0x2B55),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1615
    (0x3030, 0x3030),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1616
    (0x303D, 0x303D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1617
    (0x3297, 0x3297),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1618
    (0x3299, 0x3299),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1619
    (0x1F000, 0x1F003),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1620
    (0x1F004, 0x1F004),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1621
    (0x1F005, 0x1F0CE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1622
    (0x1F0CF, 0x1F0CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1623
    (0x1F0D0, 0x1F0FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1624
    (0x1F10D, 0x1F10F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1625
    (0x1F12F, 0x1F12F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1626
    (0x1F16C, 0x1F16F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1627
    (0x1F170, 0x1F171),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1628
    (0x1F17E, 0x1F17F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1629
    (0x1F18E, 0x1F18E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1630
    (0x1F191, 0x1F19A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1631
    (0x1F1AD, 0x1F1E5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1632
    (0x1F201, 0x1F202),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1633
    (0x1F203, 0x1F20F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1634
    (0x1F21A, 0x1F21A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1635
    (0x1F22F, 0x1F22F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1636
    (0x1F232, 0x1F23A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1637
    (0x1F23C, 0x1F23F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1638
    (0x1F249, 0x1F24F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1639
    (0x1F250, 0x1F251),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1640
    (0x1F252, 0x1F2FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1641
    (0x1F300, 0x1F30C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1642
    (0x1F30D, 0x1F30E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1643
    (0x1F30F, 0x1F30F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1644
    (0x1F310, 0x1F310),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1645
    (0x1F311, 0x1F311),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1646
    (0x1F312, 0x1F312),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1647
    (0x1F313, 0x1F315),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1648
    (0x1F316, 0x1F318),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1649
    (0x1F319, 0x1F319),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1650
    (0x1F31A, 0x1F31A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1651
    (0x1F31B, 0x1F31B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1652
    (0x1F31C, 0x1F31C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1653
    (0x1F31D, 0x1F31E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1654
    (0x1F31F, 0x1F320),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1655
    (0x1F321, 0x1F321),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1656
    (0x1F322, 0x1F323),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1657
    (0x1F324, 0x1F32C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1658
    (0x1F32D, 0x1F32F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1659
    (0x1F330, 0x1F331),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1660
    (0x1F332, 0x1F333),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1661
    (0x1F334, 0x1F335),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1662
    (0x1F336, 0x1F336),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1663
    (0x1F337, 0x1F34A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1664
    (0x1F34B, 0x1F34B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1665
    (0x1F34C, 0x1F34F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1666
    (0x1F350, 0x1F350),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1667
    (0x1F351, 0x1F37B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1668
    (0x1F37C, 0x1F37C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1669
    (0x1F37D, 0x1F37D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1670
    (0x1F37E, 0x1F37F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1671
    (0x1F380, 0x1F393),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1672
    (0x1F394, 0x1F395),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1673
    (0x1F396, 0x1F397),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1674
    (0x1F398, 0x1F398),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1675
    (0x1F399, 0x1F39B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1676
    (0x1F39C, 0x1F39D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1677
    (0x1F39E, 0x1F39F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1678
    (0x1F3A0, 0x1F3C4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1679
    (0x1F3C5, 0x1F3C5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1680
    (0x1F3C6, 0x1F3C6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1681
    (0x1F3C7, 0x1F3C7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1682
    (0x1F3C8, 0x1F3C8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1683
    (0x1F3C9, 0x1F3C9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1684
    (0x1F3CA, 0x1F3CA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1685
    (0x1F3CB, 0x1F3CE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1686
    (0x1F3CF, 0x1F3D3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1687
    (0x1F3D4, 0x1F3DF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1688
    (0x1F3E0, 0x1F3E3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1689
    (0x1F3E4, 0x1F3E4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1690
    (0x1F3E5, 0x1F3F0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1691
    (0x1F3F1, 0x1F3F2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1692
    (0x1F3F3, 0x1F3F3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1693
    (0x1F3F4, 0x1F3F4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1694
    (0x1F3F5, 0x1F3F5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1695
    (0x1F3F6, 0x1F3F6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1696
    (0x1F3F7, 0x1F3F7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1697
    (0x1F3F8, 0x1F3FA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1698
    (0x1F400, 0x1F407),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1699
    (0x1F408, 0x1F408),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1700
    (0x1F409, 0x1F40B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1701
    (0x1F40C, 0x1F40E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1702
    (0x1F40F, 0x1F410),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1703
    (0x1F411, 0x1F412),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1704
    (0x1F413, 0x1F413),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1705
    (0x1F414, 0x1F414),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1706
    (0x1F415, 0x1F415),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1707
    (0x1F416, 0x1F416),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1708
    (0x1F417, 0x1F429),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1709
    (0x1F42A, 0x1F42A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1710
    (0x1F42B, 0x1F43E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1711
    (0x1F43F, 0x1F43F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1712
    (0x1F440, 0x1F440),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1713
    (0x1F441, 0x1F441),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1714
    (0x1F442, 0x1F464),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1715
    (0x1F465, 0x1F465),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1716
    (0x1F466, 0x1F46B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1717
    (0x1F46C, 0x1F46D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1718
    (0x1F46E, 0x1F4AC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1719
    (0x1F4AD, 0x1F4AD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1720
    (0x1F4AE, 0x1F4B5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1721
    (0x1F4B6, 0x1F4B7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1722
    (0x1F4B8, 0x1F4EB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1723
    (0x1F4EC, 0x1F4ED),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1724
    (0x1F4EE, 0x1F4EE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1725
    (0x1F4EF, 0x1F4EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1726
    (0x1F4F0, 0x1F4F4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1727
    (0x1F4F5, 0x1F4F5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1728
    (0x1F4F6, 0x1F4F7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1729
    (0x1F4F8, 0x1F4F8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1730
    (0x1F4F9, 0x1F4FC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1731
    (0x1F4FD, 0x1F4FD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1732
    (0x1F4FE, 0x1F4FE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1733
    (0x1F4FF, 0x1F502),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1734
    (0x1F503, 0x1F503),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1735
    (0x1F504, 0x1F507),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1736
    (0x1F508, 0x1F508),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1737
    (0x1F509, 0x1F509),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1738
    (0x1F50A, 0x1F514),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1739
    (0x1F515, 0x1F515),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1740
    (0x1F516, 0x1F52B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1741
    (0x1F52C, 0x1F52D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1742
    (0x1F52E, 0x1F53D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1743
    (0x1F546, 0x1F548),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1744
    (0x1F549, 0x1F54A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1745
    (0x1F54B, 0x1F54E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1746
    (0x1F54F, 0x1F54F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1747
    (0x1F550, 0x1F55B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1748
    (0x1F55C, 0x1F567),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1749
    (0x1F568, 0x1F56E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1750
    (0x1F56F, 0x1F570),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1751
    (0x1F571, 0x1F572),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1752
    (0x1F573, 0x1F579),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1753
    (0x1F57A, 0x1F57A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1754
    (0x1F57B, 0x1F586),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1755
    (0x1F587, 0x1F587),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1756
    (0x1F588, 0x1F589),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1757
    (0x1F58A, 0x1F58D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1758
    (0x1F58E, 0x1F58F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1759
    (0x1F590, 0x1F590),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1760
    (0x1F591, 0x1F594),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1761
    (0x1F595, 0x1F596),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1762
    (0x1F597, 0x1F5A3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1763
    (0x1F5A4, 0x1F5A4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1764
    (0x1F5A5, 0x1F5A5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1765
    (0x1F5A6, 0x1F5A7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1766
    (0x1F5A8, 0x1F5A8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1767
    (0x1F5A9, 0x1F5B0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1768
    (0x1F5B1, 0x1F5B2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1769
    (0x1F5B3, 0x1F5BB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1770
    (0x1F5BC, 0x1F5BC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1771
    (0x1F5BD, 0x1F5C1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1772
    (0x1F5C2, 0x1F5C4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1773
    (0x1F5C5, 0x1F5D0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1774
    (0x1F5D1, 0x1F5D3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1775
    (0x1F5D4, 0x1F5DB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1776
    (0x1F5DC, 0x1F5DE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1777
    (0x1F5DF, 0x1F5E0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1778
    (0x1F5E1, 0x1F5E1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1779
    (0x1F5E2, 0x1F5E2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1780
    (0x1F5E3, 0x1F5E3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1781
    (0x1F5E4, 0x1F5E7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1782
    (0x1F5E8, 0x1F5E8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1783
    (0x1F5E9, 0x1F5EE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1784
    (0x1F5EF, 0x1F5EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1785
    (0x1F5F0, 0x1F5F2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1786
    (0x1F5F3, 0x1F5F3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1787
    (0x1F5F4, 0x1F5F9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1788
    (0x1F5FA, 0x1F5FA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1789
    (0x1F5FB, 0x1F5FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1790
    (0x1F600, 0x1F600),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1791
    (0x1F601, 0x1F606),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1792
    (0x1F607, 0x1F608),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1793
    (0x1F609, 0x1F60D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1794
    (0x1F60E, 0x1F60E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1795
    (0x1F60F, 0x1F60F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1796
    (0x1F610, 0x1F610),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1797
    (0x1F611, 0x1F611),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1798
    (0x1F612, 0x1F614),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1799
    (0x1F615, 0x1F615),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1800
    (0x1F616, 0x1F616),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1801
    (0x1F617, 0x1F617),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1802
    (0x1F618, 0x1F618),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1803
    (0x1F619, 0x1F619),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1804
    (0x1F61A, 0x1F61A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1805
    (0x1F61B, 0x1F61B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1806
    (0x1F61C, 0x1F61E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1807
    (0x1F61F, 0x1F61F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1808
    (0x1F620, 0x1F625),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1809
    (0x1F626, 0x1F627),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1810
    (0x1F628, 0x1F62B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1811
    (0x1F62C, 0x1F62C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1812
    (0x1F62D, 0x1F62D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1813
    (0x1F62E, 0x1F62F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1814
    (0x1F630, 0x1F633),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1815
    (0x1F634, 0x1F634),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1816
    (0x1F635, 0x1F635),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1817
    (0x1F636, 0x1F636),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1818
    (0x1F637, 0x1F640),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1819
    (0x1F641, 0x1F644),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1820
    (0x1F645, 0x1F64F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1821
    (0x1F680, 0x1F680),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1822
    (0x1F681, 0x1F682),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1823
    (0x1F683, 0x1F685),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1824
    (0x1F686, 0x1F686),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1825
    (0x1F687, 0x1F687),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1826
    (0x1F688, 0x1F688),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1827
    (0x1F689, 0x1F689),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1828
    (0x1F68A, 0x1F68B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1829
    (0x1F68C, 0x1F68C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1830
    (0x1F68D, 0x1F68D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1831
    (0x1F68E, 0x1F68E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1832
    (0x1F68F, 0x1F68F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1833
    (0x1F690, 0x1F690),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1834
    (0x1F691, 0x1F693),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1835
    (0x1F694, 0x1F694),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1836
    (0x1F695, 0x1F695),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1837
    (0x1F696, 0x1F696),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1838
    (0x1F697, 0x1F697),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1839
    (0x1F698, 0x1F698),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1840
    (0x1F699, 0x1F69A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1841
    (0x1F69B, 0x1F6A1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1842
    (0x1F6A2, 0x1F6A2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1843
    (0x1F6A3, 0x1F6A3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1844
    (0x1F6A4, 0x1F6A5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1845
    (0x1F6A6, 0x1F6A6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1846
    (0x1F6A7, 0x1F6AD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1847
    (0x1F6AE, 0x1F6B1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1848
    (0x1F6B2, 0x1F6B2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1849
    (0x1F6B3, 0x1F6B5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1850
    (0x1F6B6, 0x1F6B6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1851
    (0x1F6B7, 0x1F6B8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1852
    (0x1F6B9, 0x1F6BE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1853
    (0x1F6BF, 0x1F6BF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1854
    (0x1F6C0, 0x1F6C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1855
    (0x1F6C1, 0x1F6C5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1856
    (0x1F6C6, 0x1F6CA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1857
    (0x1F6CB, 0x1F6CB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1858
    (0x1F6CC, 0x1F6CC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1859
    (0x1F6CD, 0x1F6CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1860
    (0x1F6D0, 0x1F6D0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1861
    (0x1F6D1, 0x1F6D2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1862
    (0x1F6D3, 0x1F6D4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1863
    (0x1F6D5, 0x1F6D5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1864
    (0x1F6D6, 0x1F6D7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1865
    (0x1F6D8, 0x1F6DB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1866
    (0x1F6DC, 0x1F6DC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1867
    (0x1F6DD, 0x1F6DF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1868
    (0x1F6E0, 0x1F6E5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1869
    (0x1F6E6, 0x1F6E8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1870
    (0x1F6E9, 0x1F6E9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1871
    (0x1F6EA, 0x1F6EA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1872
    (0x1F6EB, 0x1F6EC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1873
    (0x1F6ED, 0x1F6EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1874
    (0x1F6F0, 0x1F6F0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1875
    (0x1F6F1, 0x1F6F2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1876
    (0x1F6F3, 0x1F6F3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1877
    (0x1F6F4, 0x1F6F6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1878
    (0x1F6F7, 0x1F6F8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1879
    (0x1F6F9, 0x1F6F9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1880
    (0x1F6FA, 0x1F6FA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1881
    (0x1F6FB, 0x1F6FC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1882
    (0x1F6FD, 0x1F6FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1883
    (0x1F774, 0x1F77F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1884
    (0x1F7D5, 0x1F7DF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1885
    (0x1F7E0, 0x1F7EB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1886
    (0x1F7EC, 0x1F7EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1887
    (0x1F7F0, 0x1F7F0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1888
    (0x1F7F1, 0x1F7FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1889
    (0x1F80C, 0x1F80F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1890
    (0x1F848, 0x1F84F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1891
    (0x1F85A, 0x1F85F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1892
    (0x1F888, 0x1F88F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1893
    (0x1F8AE, 0x1F8FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1894
    (0x1F90C, 0x1F90C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1895
    (0x1F90D, 0x1F90F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1896
    (0x1F910, 0x1F918),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1897
    (0x1F919, 0x1F91E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1898
    (0x1F91F, 0x1F91F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1899
    (0x1F920, 0x1F927),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1900
    (0x1F928, 0x1F92F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1901
    (0x1F930, 0x1F930),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1902
    (0x1F931, 0x1F932),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1903
    (0x1F933, 0x1F93A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1904
    (0x1F93C, 0x1F93E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1905
    (0x1F93F, 0x1F93F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1906
    (0x1F940, 0x1F945),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1907
    (0x1F947, 0x1F94B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1908
    (0x1F94C, 0x1F94C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1909
    (0x1F94D, 0x1F94F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1910
    (0x1F950, 0x1F95E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1911
    (0x1F95F, 0x1F96B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1912
    (0x1F96C, 0x1F970),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1913
    (0x1F971, 0x1F971),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1914
    (0x1F972, 0x1F972),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1915
    (0x1F973, 0x1F976),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1916
    (0x1F977, 0x1F978),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1917
    (0x1F979, 0x1F979),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1918
    (0x1F97A, 0x1F97A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1919
    (0x1F97B, 0x1F97B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1920
    (0x1F97C, 0x1F97F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1921
    (0x1F980, 0x1F984),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1922
    (0x1F985, 0x1F991),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1923
    (0x1F992, 0x1F997),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1924
    (0x1F998, 0x1F9A2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1925
    (0x1F9A3, 0x1F9A4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1926
    (0x1F9A5, 0x1F9AA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1927
    (0x1F9AB, 0x1F9AD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1928
    (0x1F9AE, 0x1F9AF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1929
    (0x1F9B0, 0x1F9B9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1930
    (0x1F9BA, 0x1F9BF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1931
    (0x1F9C0, 0x1F9C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1932
    (0x1F9C1, 0x1F9C2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1933
    (0x1F9C3, 0x1F9CA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1934
    (0x1F9CB, 0x1F9CB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1935
    (0x1F9CC, 0x1F9CC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1936
    (0x1F9CD, 0x1F9CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1937
    (0x1F9D0, 0x1F9E6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1938
    (0x1F9E7, 0x1F9FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1939
    (0x1FA00, 0x1FA6F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1940
    (0x1FA70, 0x1FA73),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1941
    (0x1FA74, 0x1FA74),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1942
    (0x1FA75, 0x1FA77),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1943
    (0x1FA78, 0x1FA7A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1944
    (0x1FA7B, 0x1FA7C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1945
    (0x1FA7D, 0x1FA7F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1946
    (0x1FA80, 0x1FA82),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1947
    (0x1FA83, 0x1FA86),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1948
    (0x1FA87, 0x1FA88),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1949
    (0x1FA89, 0x1FA89),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1950
    (0x1FA8A, 0x1FA8E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1951
    (0x1FA8F, 0x1FA8F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1952
    (0x1FA90, 0x1FA95),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1953
    (0x1FA96, 0x1FAA8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1954
    (0x1FAA9, 0x1FAAC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1955
    (0x1FAAD, 0x1FAAF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1956
    (0x1FAB0, 0x1FAB6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1957
    (0x1FAB7, 0x1FABA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1958
    (0x1FABB, 0x1FABD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1959
    (0x1FABE, 0x1FABE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1960
    (0x1FABF, 0x1FABF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1961
    (0x1FAC0, 0x1FAC2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1962
    (0x1FAC3, 0x1FAC5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1963
    (0x1FAC6, 0x1FAC6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1964
    (0x1FAC7, 0x1FACD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1965
    (0x1FACE, 0x1FACF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1966
    (0x1FAD0, 0x1FAD6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1967
    (0x1FAD7, 0x1FAD9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1968
    (0x1FADA, 0x1FADB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1969
    (0x1FADC, 0x1FADC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1970
    (0x1FADD, 0x1FADE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1971
    (0x1FADF, 0x1FADF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1972
    (0x1FAE0, 0x1FAE7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1973
    (0x1FAE8, 0x1FAE8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1974
    (0x1FAE9, 0x1FAE9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1975
    (0x1FAEA, 0x1FAEF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1976
    (0x1FAF0, 0x1FAF6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1977
    (0x1FAF7, 0x1FAF8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1978
    (0x1FAF9, 0x1FAFF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1979
    (0x1FC00, 0x1FFFD),
];

// cpp: font_engine/text/native/grapheme_break_property_data.h:1981
const kIndicConsonantRanges: [(u32, u32); 76] = [
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1982
    (0x915, 0x939),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1983
    (0x958, 0x95F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1984
    (0x978, 0x97F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1985
    (0x995, 0x9A8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1986
    (0x9AA, 0x9B0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1987
    (0x9B2, 0x9B2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1988
    (0x9B6, 0x9B9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1989
    (0x9DC, 0x9DD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1990
    (0x9DF, 0x9DF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1991
    (0x9F0, 0x9F1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1992
    (0xA95, 0xAA8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1993
    (0xAAA, 0xAB0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1994
    (0xAB2, 0xAB3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1995
    (0xAB5, 0xAB9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1996
    (0xAF9, 0xAF9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1997
    (0xB15, 0xB28),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1998
    (0xB2A, 0xB30),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:1999
    (0xB32, 0xB33),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2000
    (0xB35, 0xB39),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2001
    (0xB5C, 0xB5D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2002
    (0xB5F, 0xB5F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2003
    (0xB71, 0xB71),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2004
    (0xC15, 0xC28),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2005
    (0xC2A, 0xC39),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2006
    (0xC58, 0xC5A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2007
    (0xD15, 0xD3A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2008
    (0x1000, 0x102A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2009
    (0x103F, 0x103F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2010
    (0x1050, 0x1055),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2011
    (0x105A, 0x105D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2012
    (0x1061, 0x1061),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2013
    (0x1065, 0x1066),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2014
    (0x106E, 0x1070),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2015
    (0x1075, 0x1081),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2016
    (0x108E, 0x108E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2017
    (0x1780, 0x17B3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2018
    (0x1A20, 0x1A54),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2019
    (0x1B0B, 0x1B0C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2020
    (0x1B13, 0x1B33),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2021
    (0x1B45, 0x1B4C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2022
    (0x1B83, 0x1BA0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2023
    (0x1BAE, 0x1BAF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2024
    (0x1BBB, 0x1BBD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2025
    (0xA989, 0xA98B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2026
    (0xA98F, 0xA9B2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2027
    (0xA9E0, 0xA9E4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2028
    (0xA9E7, 0xA9EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2029
    (0xA9FA, 0xA9FE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2030
    (0xAA60, 0xAA6F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2031
    (0xAA71, 0xAA73),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2032
    (0xAA7A, 0xAA7A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2033
    (0xAA7E, 0xAA7F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2034
    (0xAAE0, 0xAAEA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2035
    (0xABC0, 0xABDA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2036
    (0x10A00, 0x10A00),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2037
    (0x10A10, 0x10A13),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2038
    (0x10A15, 0x10A17),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2039
    (0x10A19, 0x10A35),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2040
    (0x11103, 0x11126),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2041
    (0x11144, 0x11144),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2042
    (0x11147, 0x11147),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2043
    (0x11380, 0x11389),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2044
    (0x1138B, 0x1138B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2045
    (0x1138E, 0x1138E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2046
    (0x11390, 0x113B5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2047
    (0x11900, 0x11906),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2048
    (0x11909, 0x11909),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2049
    (0x1190C, 0x11913),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2050
    (0x11915, 0x11916),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2051
    (0x11918, 0x1192F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2052
    (0x11A00, 0x11A00),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2053
    (0x11A0B, 0x11A32),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2054
    (0x11A50, 0x11A50),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2055
    (0x11A5C, 0x11A83),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2056
    (0x11F04, 0x11F10),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2057
    (0x11F12, 0x11F33),
];

// cpp: font_engine/text/native/grapheme_break_property_data.h:2059
const kIndicExtendRanges: [(u32, u32); 409] = [
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2060
    (0x300, 0x36F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2061
    (0x483, 0x487),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2062
    (0x488, 0x489),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2063
    (0x591, 0x5BD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2064
    (0x5BF, 0x5BF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2065
    (0x5C1, 0x5C2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2066
    (0x5C4, 0x5C5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2067
    (0x5C7, 0x5C7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2068
    (0x610, 0x61A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2069
    (0x64B, 0x65F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2070
    (0x670, 0x670),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2071
    (0x6D6, 0x6DC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2072
    (0x6DF, 0x6E4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2073
    (0x6E7, 0x6E8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2074
    (0x6EA, 0x6ED),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2075
    (0x711, 0x711),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2076
    (0x730, 0x74A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2077
    (0x7A6, 0x7B0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2078
    (0x7EB, 0x7F3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2079
    (0x7FD, 0x7FD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2080
    (0x816, 0x819),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2081
    (0x81B, 0x823),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2082
    (0x825, 0x827),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2083
    (0x829, 0x82D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2084
    (0x859, 0x85B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2085
    (0x897, 0x89F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2086
    (0x8CA, 0x8E1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2087
    (0x8E3, 0x902),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2088
    (0x93A, 0x93A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2089
    (0x93C, 0x93C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2090
    (0x941, 0x948),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2091
    (0x951, 0x957),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2092
    (0x962, 0x963),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2093
    (0x981, 0x981),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2094
    (0x9BC, 0x9BC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2095
    (0x9BE, 0x9BE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2096
    (0x9C1, 0x9C4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2097
    (0x9D7, 0x9D7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2098
    (0x9E2, 0x9E3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2099
    (0x9FE, 0x9FE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2100
    (0xA01, 0xA02),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2101
    (0xA3C, 0xA3C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2102
    (0xA41, 0xA42),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2103
    (0xA47, 0xA48),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2104
    (0xA4B, 0xA4D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2105
    (0xA51, 0xA51),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2106
    (0xA70, 0xA71),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2107
    (0xA75, 0xA75),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2108
    (0xA81, 0xA82),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2109
    (0xABC, 0xABC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2110
    (0xAC1, 0xAC5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2111
    (0xAC7, 0xAC8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2112
    (0xAE2, 0xAE3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2113
    (0xAFA, 0xAFF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2114
    (0xB01, 0xB01),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2115
    (0xB3C, 0xB3C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2116
    (0xB3E, 0xB3E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2117
    (0xB3F, 0xB3F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2118
    (0xB41, 0xB44),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2119
    (0xB55, 0xB56),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2120
    (0xB57, 0xB57),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2121
    (0xB62, 0xB63),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2122
    (0xB82, 0xB82),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2123
    (0xBBE, 0xBBE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2124
    (0xBC0, 0xBC0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2125
    (0xBCD, 0xBCD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2126
    (0xBD7, 0xBD7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2127
    (0xC00, 0xC00),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2128
    (0xC04, 0xC04),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2129
    (0xC3C, 0xC3C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2130
    (0xC3E, 0xC40),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2131
    (0xC46, 0xC48),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2132
    (0xC4A, 0xC4C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2133
    (0xC55, 0xC56),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2134
    (0xC62, 0xC63),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2135
    (0xC81, 0xC81),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2136
    (0xCBC, 0xCBC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2137
    (0xCBF, 0xCBF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2138
    (0xCC0, 0xCC0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2139
    (0xCC2, 0xCC2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2140
    (0xCC6, 0xCC6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2141
    (0xCC7, 0xCC8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2142
    (0xCCA, 0xCCB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2143
    (0xCCC, 0xCCD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2144
    (0xCD5, 0xCD6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2145
    (0xCE2, 0xCE3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2146
    (0xD00, 0xD01),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2147
    (0xD3B, 0xD3C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2148
    (0xD3E, 0xD3E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2149
    (0xD41, 0xD44),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2150
    (0xD57, 0xD57),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2151
    (0xD62, 0xD63),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2152
    (0xD81, 0xD81),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2153
    (0xDCA, 0xDCA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2154
    (0xDCF, 0xDCF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2155
    (0xDD2, 0xDD4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2156
    (0xDD6, 0xDD6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2157
    (0xDDF, 0xDDF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2158
    (0xE31, 0xE31),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2159
    (0xE34, 0xE3A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2160
    (0xE47, 0xE4E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2161
    (0xEB1, 0xEB1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2162
    (0xEB4, 0xEBC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2163
    (0xEC8, 0xECE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2164
    (0xF18, 0xF19),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2165
    (0xF35, 0xF35),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2166
    (0xF37, 0xF37),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2167
    (0xF39, 0xF39),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2168
    (0xF71, 0xF7E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2169
    (0xF80, 0xF84),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2170
    (0xF86, 0xF87),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2171
    (0xF8D, 0xF97),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2172
    (0xF99, 0xFBC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2173
    (0xFC6, 0xFC6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2174
    (0x102D, 0x1030),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2175
    (0x1032, 0x1037),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2176
    (0x103A, 0x103A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2177
    (0x103D, 0x103E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2178
    (0x1058, 0x1059),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2179
    (0x105E, 0x1060),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2180
    (0x1071, 0x1074),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2181
    (0x1082, 0x1082),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2182
    (0x1085, 0x1086),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2183
    (0x108D, 0x108D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2184
    (0x109D, 0x109D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2185
    (0x135D, 0x135F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2186
    (0x1712, 0x1714),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2187
    (0x1715, 0x1715),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2188
    (0x1732, 0x1733),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2189
    (0x1734, 0x1734),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2190
    (0x1752, 0x1753),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2191
    (0x1772, 0x1773),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2192
    (0x17B4, 0x17B5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2193
    (0x17B7, 0x17BD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2194
    (0x17C6, 0x17C6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2195
    (0x17C9, 0x17D1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2196
    (0x17D3, 0x17D3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2197
    (0x17DD, 0x17DD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2198
    (0x180B, 0x180D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2199
    (0x180F, 0x180F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2200
    (0x1885, 0x1886),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2201
    (0x18A9, 0x18A9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2202
    (0x1920, 0x1922),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2203
    (0x1927, 0x1928),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2204
    (0x1932, 0x1932),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2205
    (0x1939, 0x193B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2206
    (0x1A17, 0x1A18),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2207
    (0x1A1B, 0x1A1B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2208
    (0x1A56, 0x1A56),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2209
    (0x1A58, 0x1A5E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2210
    (0x1A62, 0x1A62),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2211
    (0x1A65, 0x1A6C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2212
    (0x1A73, 0x1A7C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2213
    (0x1A7F, 0x1A7F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2214
    (0x1AB0, 0x1ABD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2215
    (0x1ABE, 0x1ABE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2216
    (0x1ABF, 0x1ADD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2217
    (0x1AE0, 0x1AEB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2218
    (0x1B00, 0x1B03),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2219
    (0x1B34, 0x1B34),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2220
    (0x1B35, 0x1B35),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2221
    (0x1B36, 0x1B3A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2222
    (0x1B3B, 0x1B3B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2223
    (0x1B3C, 0x1B3C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2224
    (0x1B3D, 0x1B3D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2225
    (0x1B42, 0x1B42),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2226
    (0x1B43, 0x1B43),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2227
    (0x1B6B, 0x1B73),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2228
    (0x1B80, 0x1B81),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2229
    (0x1BA2, 0x1BA5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2230
    (0x1BA8, 0x1BA9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2231
    (0x1BAA, 0x1BAA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2232
    (0x1BAC, 0x1BAD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2233
    (0x1BE6, 0x1BE6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2234
    (0x1BE8, 0x1BE9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2235
    (0x1BED, 0x1BED),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2236
    (0x1BEF, 0x1BF1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2237
    (0x1BF2, 0x1BF3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2238
    (0x1C2C, 0x1C33),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2239
    (0x1C36, 0x1C37),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2240
    (0x1CD0, 0x1CD2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2241
    (0x1CD4, 0x1CE0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2242
    (0x1CE2, 0x1CE8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2243
    (0x1CED, 0x1CED),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2244
    (0x1CF4, 0x1CF4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2245
    (0x1CF8, 0x1CF9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2246
    (0x1DC0, 0x1DFF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2247
    (0x200D, 0x200D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2248
    (0x20D0, 0x20DC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2249
    (0x20DD, 0x20E0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2250
    (0x20E1, 0x20E1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2251
    (0x20E2, 0x20E4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2252
    (0x20E5, 0x20F0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2253
    (0x2CEF, 0x2CF1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2254
    (0x2D7F, 0x2D7F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2255
    (0x2DE0, 0x2DFF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2256
    (0x302A, 0x302D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2257
    (0x302E, 0x302F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2258
    (0x3099, 0x309A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2259
    (0xA66F, 0xA66F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2260
    (0xA670, 0xA672),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2261
    (0xA674, 0xA67D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2262
    (0xA69E, 0xA69F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2263
    (0xA6F0, 0xA6F1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2264
    (0xA802, 0xA802),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2265
    (0xA806, 0xA806),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2266
    (0xA80B, 0xA80B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2267
    (0xA825, 0xA826),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2268
    (0xA82C, 0xA82C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2269
    (0xA8C4, 0xA8C5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2270
    (0xA8E0, 0xA8F1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2271
    (0xA8FF, 0xA8FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2272
    (0xA926, 0xA92D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2273
    (0xA947, 0xA951),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2274
    (0xA953, 0xA953),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2275
    (0xA980, 0xA982),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2276
    (0xA9B3, 0xA9B3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2277
    (0xA9B6, 0xA9B9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2278
    (0xA9BC, 0xA9BD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2279
    (0xA9E5, 0xA9E5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2280
    (0xAA29, 0xAA2E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2281
    (0xAA31, 0xAA32),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2282
    (0xAA35, 0xAA36),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2283
    (0xAA43, 0xAA43),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2284
    (0xAA4C, 0xAA4C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2285
    (0xAA7C, 0xAA7C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2286
    (0xAAB0, 0xAAB0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2287
    (0xAAB2, 0xAAB4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2288
    (0xAAB7, 0xAAB8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2289
    (0xAABE, 0xAABF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2290
    (0xAAC1, 0xAAC1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2291
    (0xAAEC, 0xAAED),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2292
    (0xABE5, 0xABE5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2293
    (0xABE8, 0xABE8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2294
    (0xABED, 0xABED),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2295
    (0xFB1E, 0xFB1E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2296
    (0xFE00, 0xFE0F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2297
    (0xFE20, 0xFE2F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2298
    (0xFF9E, 0xFF9F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2299
    (0x101FD, 0x101FD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2300
    (0x102E0, 0x102E0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2301
    (0x10376, 0x1037A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2302
    (0x10A01, 0x10A03),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2303
    (0x10A05, 0x10A06),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2304
    (0x10A0C, 0x10A0F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2305
    (0x10A38, 0x10A3A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2306
    (0x10AE5, 0x10AE6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2307
    (0x10D24, 0x10D27),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2308
    (0x10D69, 0x10D6D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2309
    (0x10EAB, 0x10EAC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2310
    (0x10EFA, 0x10EFF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2311
    (0x10F46, 0x10F50),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2312
    (0x10F82, 0x10F85),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2313
    (0x11001, 0x11001),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2314
    (0x11038, 0x11046),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2315
    (0x11070, 0x11070),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2316
    (0x11073, 0x11074),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2317
    (0x1107F, 0x11081),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2318
    (0x110B3, 0x110B6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2319
    (0x110B9, 0x110BA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2320
    (0x110C2, 0x110C2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2321
    (0x11100, 0x11102),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2322
    (0x11127, 0x1112B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2323
    (0x1112D, 0x11132),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2324
    (0x11134, 0x11134),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2325
    (0x11173, 0x11173),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2326
    (0x11180, 0x11181),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2327
    (0x111B6, 0x111BE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2328
    (0x111C0, 0x111C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2329
    (0x111C9, 0x111CC),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2330
    (0x111CF, 0x111CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2331
    (0x1122F, 0x11231),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2332
    (0x11234, 0x11234),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2333
    (0x11235, 0x11235),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2334
    (0x11236, 0x11237),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2335
    (0x1123E, 0x1123E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2336
    (0x11241, 0x11241),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2337
    (0x112DF, 0x112DF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2338
    (0x112E3, 0x112EA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2339
    (0x11300, 0x11301),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2340
    (0x1133B, 0x1133C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2341
    (0x1133E, 0x1133E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2342
    (0x11340, 0x11340),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2343
    (0x1134D, 0x1134D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2344
    (0x11357, 0x11357),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2345
    (0x11366, 0x1136C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2346
    (0x11370, 0x11374),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2347
    (0x113B8, 0x113B8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2348
    (0x113BB, 0x113C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2349
    (0x113C2, 0x113C2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2350
    (0x113C5, 0x113C5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2351
    (0x113C7, 0x113C9),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2352
    (0x113CE, 0x113CE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2353
    (0x113CF, 0x113CF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2354
    (0x113D2, 0x113D2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2355
    (0x113E1, 0x113E2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2356
    (0x11438, 0x1143F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2357
    (0x11442, 0x11444),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2358
    (0x11446, 0x11446),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2359
    (0x1145E, 0x1145E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2360
    (0x114B0, 0x114B0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2361
    (0x114B3, 0x114B8),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2362
    (0x114BA, 0x114BA),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2363
    (0x114BD, 0x114BD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2364
    (0x114BF, 0x114C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2365
    (0x114C2, 0x114C3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2366
    (0x115AF, 0x115AF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2367
    (0x115B2, 0x115B5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2368
    (0x115BC, 0x115BD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2369
    (0x115BF, 0x115C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2370
    (0x115DC, 0x115DD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2371
    (0x11633, 0x1163A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2372
    (0x1163D, 0x1163D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2373
    (0x1163F, 0x11640),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2374
    (0x116AB, 0x116AB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2375
    (0x116AD, 0x116AD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2376
    (0x116B0, 0x116B5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2377
    (0x116B6, 0x116B6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2378
    (0x116B7, 0x116B7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2379
    (0x1171D, 0x1171D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2380
    (0x1171F, 0x1171F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2381
    (0x11722, 0x11725),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2382
    (0x11727, 0x1172B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2383
    (0x1182F, 0x11837),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2384
    (0x11839, 0x1183A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2385
    (0x11930, 0x11930),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2386
    (0x1193B, 0x1193C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2387
    (0x1193D, 0x1193D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2388
    (0x11943, 0x11943),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2389
    (0x119D4, 0x119D7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2390
    (0x119DA, 0x119DB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2391
    (0x119E0, 0x119E0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2392
    (0x11A01, 0x11A0A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2393
    (0x11A33, 0x11A38),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2394
    (0x11A3B, 0x11A3E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2395
    (0x11A51, 0x11A56),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2396
    (0x11A59, 0x11A5B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2397
    (0x11A8A, 0x11A96),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2398
    (0x11A98, 0x11A98),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2399
    (0x11B60, 0x11B60),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2400
    (0x11B62, 0x11B64),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2401
    (0x11B66, 0x11B66),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2402
    (0x11C30, 0x11C36),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2403
    (0x11C38, 0x11C3D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2404
    (0x11C3F, 0x11C3F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2405
    (0x11C92, 0x11CA7),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2406
    (0x11CAA, 0x11CB0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2407
    (0x11CB2, 0x11CB3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2408
    (0x11CB5, 0x11CB6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2409
    (0x11D31, 0x11D36),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2410
    (0x11D3A, 0x11D3A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2411
    (0x11D3C, 0x11D3D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2412
    (0x11D3F, 0x11D45),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2413
    (0x11D47, 0x11D47),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2414
    (0x11D90, 0x11D91),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2415
    (0x11D95, 0x11D95),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2416
    (0x11D97, 0x11D97),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2417
    (0x11EF3, 0x11EF4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2418
    (0x11F00, 0x11F01),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2419
    (0x11F36, 0x11F3A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2420
    (0x11F40, 0x11F40),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2421
    (0x11F41, 0x11F41),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2422
    (0x11F5A, 0x11F5A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2423
    (0x13440, 0x13440),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2424
    (0x13447, 0x13455),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2425
    (0x1611E, 0x16129),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2426
    (0x1612D, 0x1612F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2427
    (0x16AF0, 0x16AF4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2428
    (0x16B30, 0x16B36),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2429
    (0x16F4F, 0x16F4F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2430
    (0x16F8F, 0x16F92),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2431
    (0x16FE4, 0x16FE4),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2432
    (0x16FF0, 0x16FF1),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2433
    (0x1BC9D, 0x1BC9E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2434
    (0x1CF00, 0x1CF2D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2435
    (0x1CF30, 0x1CF46),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2436
    (0x1D165, 0x1D166),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2437
    (0x1D167, 0x1D169),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2438
    (0x1D16D, 0x1D172),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2439
    (0x1D17B, 0x1D182),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2440
    (0x1D185, 0x1D18B),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2441
    (0x1D1AA, 0x1D1AD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2442
    (0x1D242, 0x1D244),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2443
    (0x1DA00, 0x1DA36),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2444
    (0x1DA3B, 0x1DA6C),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2445
    (0x1DA75, 0x1DA75),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2446
    (0x1DA84, 0x1DA84),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2447
    (0x1DA9B, 0x1DA9F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2448
    (0x1DAA1, 0x1DAAF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2449
    (0x1E000, 0x1E006),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2450
    (0x1E008, 0x1E018),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2451
    (0x1E01B, 0x1E021),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2452
    (0x1E023, 0x1E024),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2453
    (0x1E026, 0x1E02A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2454
    (0x1E08F, 0x1E08F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2455
    (0x1E130, 0x1E136),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2456
    (0x1E2AE, 0x1E2AE),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2457
    (0x1E2EC, 0x1E2EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2458
    (0x1E4EC, 0x1E4EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2459
    (0x1E5EE, 0x1E5EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2460
    (0x1E6E3, 0x1E6E3),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2461
    (0x1E6E6, 0x1E6E6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2462
    (0x1E6EE, 0x1E6EF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2463
    (0x1E6F5, 0x1E6F5),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2464
    (0x1E8D0, 0x1E8D6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2465
    (0x1E944, 0x1E94A),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2466
    (0x1F3FB, 0x1F3FF),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2467
    (0xE0020, 0xE007F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2468
    (0xE0100, 0xE01EF),
];

// cpp: font_engine/text/native/grapheme_break_property_data.h:2470
const kIndicLinkerRanges: [(u32, u32); 20] = [
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2471
    (0x94D, 0x94D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2472
    (0x9CD, 0x9CD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2473
    (0xACD, 0xACD),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2474
    (0xB4D, 0xB4D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2475
    (0xC4D, 0xC4D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2476
    (0xD4D, 0xD4D),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2477
    (0x1039, 0x1039),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2478
    (0x17D2, 0x17D2),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2479
    (0x1A60, 0x1A60),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2480
    (0x1B44, 0x1B44),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2481
    (0x1BAB, 0x1BAB),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2482
    (0xA9C0, 0xA9C0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2483
    (0xAAF6, 0xAAF6),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2484
    (0x10A3F, 0x10A3F),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2485
    (0x11133, 0x11133),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2486
    (0x113D0, 0x113D0),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2487
    (0x1193E, 0x1193E),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2488
    (0x11A47, 0x11A47),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2489
    (0x11A99, 0x11A99),
    // cpp: font_engine/text/native/grapheme_break_property_data.h:2490
    (0x11F42, 0x11F42),
];

// cpp: font_engine/text/native/grapheme_break_property_data.h:2493-2505
trait RangeBounds {
    fn first(&self) -> u32;
    fn last(&self) -> u32;
}
impl RangeBounds for CodePointRange {
    fn first(&self) -> u32 {
        self.0
    }
    fn last(&self) -> u32 {
        self.1
    }
}
impl RangeBounds for PropertyRange {
    fn first(&self) -> u32 {
        self.0
    }
    fn last(&self) -> u32 {
        self.1
    }
}
fn FindRange<T: RangeBounds>(code_point: u32, ranges: &[T]) -> Option<&T> {
    let upper = ranges.partition_point(|range| range.first() <= code_point);
    if upper == 0 {
        return None;
    }
    let candidate = &ranges[upper - 1];
    (code_point <= candidate.last()).then_some(candidate)
}

// cpp: font_engine/text/native/grapheme_break_property_data.h:2507-2510
fn IsInRanges(code_point: u32, ranges: &[CodePointRange]) -> bool {
    FindRange(code_point, ranges).is_some()
}

// cpp: font_engine/text/native/grapheme_break_property_data.h:2512-2517
pub fn GraphemeBreakProperty(code_point: u32) -> GraphemeProperty {
    FindRange(code_point, &kGraphemePropertyRanges)
        .map_or(GraphemeProperty::kOther, |range| range.2)
}

// cpp: font_engine/text/native/grapheme_break_property_data.h:2519-2521
pub fn IsExtendedPictographic(code_point: u32) -> bool {
    IsInRanges(code_point, &kExtendedPictographicRanges)
}

// cpp: font_engine/text/native/grapheme_break_property_data.h:2523-2532
pub fn IndicConjunctProperty(code_point: u32) -> IndicConjunctBreak {
    if IsInRanges(code_point, &kIndicConsonantRanges) {
        return IndicConjunctBreak::kConsonant;
    }
    if IsInRanges(code_point, &kIndicLinkerRanges) {
        return IndicConjunctBreak::kLinker;
    }
    if IsInRanges(code_point, &kIndicExtendRanges) {
        return IndicConjunctBreak::kExtend;
    }
    IndicConjunctBreak::kNone
}
