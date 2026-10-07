// unicode_gen_def.h:1..323. Bellard/Gordon MIT; original X-macro order retained.
#![allow(non_upper_case_globals, dead_code)]
use core::ffi::c_char;
pub type UnicodeGCEnum1 = i32;
// unicode_gen_def.h:2
pub const GCAT_Cn: i32 = 0;
// unicode_gen_def.h:3
pub const GCAT_Lu: i32 = 1;
// unicode_gen_def.h:4
pub const GCAT_Ll: i32 = 2;
// unicode_gen_def.h:5
pub const GCAT_Lt: i32 = 3;
// unicode_gen_def.h:6
pub const GCAT_Lm: i32 = 4;
// unicode_gen_def.h:7
pub const GCAT_Lo: i32 = 5;
// unicode_gen_def.h:8
pub const GCAT_Mn: i32 = 6;
// unicode_gen_def.h:9
pub const GCAT_Mc: i32 = 7;
// unicode_gen_def.h:10
pub const GCAT_Me: i32 = 8;
// unicode_gen_def.h:11
pub const GCAT_Nd: i32 = 9;
// unicode_gen_def.h:12
pub const GCAT_Nl: i32 = 10;
// unicode_gen_def.h:13
pub const GCAT_No: i32 = 11;
// unicode_gen_def.h:14
pub const GCAT_Sm: i32 = 12;
// unicode_gen_def.h:15
pub const GCAT_Sc: i32 = 13;
// unicode_gen_def.h:16
pub const GCAT_Sk: i32 = 14;
// unicode_gen_def.h:17
pub const GCAT_So: i32 = 15;
// unicode_gen_def.h:18
pub const GCAT_Pc: i32 = 16;
// unicode_gen_def.h:19
pub const GCAT_Pd: i32 = 17;
// unicode_gen_def.h:20
pub const GCAT_Ps: i32 = 18;
// unicode_gen_def.h:21
pub const GCAT_Pe: i32 = 19;
// unicode_gen_def.h:22
pub const GCAT_Pi: i32 = 20;
// unicode_gen_def.h:23
pub const GCAT_Pf: i32 = 21;
// unicode_gen_def.h:24
pub const GCAT_Po: i32 = 22;
// unicode_gen_def.h:25
pub const GCAT_Zs: i32 = 23;
// unicode_gen_def.h:26
pub const GCAT_Zl: i32 = 24;
// unicode_gen_def.h:27
pub const GCAT_Zp: i32 = 25;
// unicode_gen_def.h:28
pub const GCAT_Cc: i32 = 26;
// unicode_gen_def.h:29
pub const GCAT_Cf: i32 = 27;
// unicode_gen_def.h:30
pub const GCAT_Cs: i32 = 28;
// unicode_gen_def.h:31
pub const GCAT_Co: i32 = 29;
// unicode_gen_def.h:33
pub const GCAT_LC: i32 = 30;
// unicode_gen_def.h:34
pub const GCAT_L: i32 = 31;
// unicode_gen_def.h:35
pub const GCAT_M: i32 = 32;
// unicode_gen_def.h:36
pub const GCAT_N: i32 = 33;
// unicode_gen_def.h:37
pub const GCAT_S: i32 = 34;
// unicode_gen_def.h:38
pub const GCAT_P: i32 = 35;
// unicode_gen_def.h:39
pub const GCAT_Z: i32 = 36;
// unicode_gen_def.h:40
pub const GCAT_C: i32 = 37;
pub const GCAT_COUNT: i32 = 38;
pub static mut unicode_gc_name: [*const c_char; 38] = [
    c"Cn".as_ptr(),
    c"Lu".as_ptr(),
    c"Ll".as_ptr(),
    c"Lt".as_ptr(),
    c"Lm".as_ptr(),
    c"Lo".as_ptr(),
    c"Mn".as_ptr(),
    c"Mc".as_ptr(),
    c"Me".as_ptr(),
    c"Nd".as_ptr(),
    c"Nl".as_ptr(),
    c"No".as_ptr(),
    c"Sm".as_ptr(),
    c"Sc".as_ptr(),
    c"Sk".as_ptr(),
    c"So".as_ptr(),
    c"Pc".as_ptr(),
    c"Pd".as_ptr(),
    c"Ps".as_ptr(),
    c"Pe".as_ptr(),
    c"Pi".as_ptr(),
    c"Pf".as_ptr(),
    c"Po".as_ptr(),
    c"Zs".as_ptr(),
    c"Zl".as_ptr(),
    c"Zp".as_ptr(),
    c"Cc".as_ptr(),
    c"Cf".as_ptr(),
    c"Cs".as_ptr(),
    c"Co".as_ptr(),
    c"LC".as_ptr(),
    c"L".as_ptr(),
    c"M".as_ptr(),
    c"N".as_ptr(),
    c"S".as_ptr(),
    c"P".as_ptr(),
    c"Z".as_ptr(),
    c"C".as_ptr(),
];
pub static mut unicode_gc_short_name: [*const c_char; 38] = [
    c"Unassigned".as_ptr(),
    c"Uppercase_Letter".as_ptr(),
    c"Lowercase_Letter".as_ptr(),
    c"Titlecase_Letter".as_ptr(),
    c"Modifier_Letter".as_ptr(),
    c"Other_Letter".as_ptr(),
    c"Nonspacing_Mark".as_ptr(),
    c"Spacing_Mark".as_ptr(),
    c"Enclosing_Mark".as_ptr(),
    c"Decimal_Number,digit".as_ptr(),
    c"Letter_Number".as_ptr(),
    c"Other_Number".as_ptr(),
    c"Math_Symbol".as_ptr(),
    c"Currency_Symbol".as_ptr(),
    c"Modifier_Symbol".as_ptr(),
    c"Other_Symbol".as_ptr(),
    c"Connector_Punctuation".as_ptr(),
    c"Dash_Punctuation".as_ptr(),
    c"Open_Punctuation".as_ptr(),
    c"Close_Punctuation".as_ptr(),
    c"Initial_Punctuation".as_ptr(),
    c"Final_Punctuation".as_ptr(),
    c"Other_Punctuation".as_ptr(),
    c"Space_Separator".as_ptr(),
    c"Line_Separator".as_ptr(),
    c"Paragraph_Separator".as_ptr(),
    c"Control,cntrl".as_ptr(),
    c"Format".as_ptr(),
    c"Surrogate".as_ptr(),
    c"Private_Use".as_ptr(),
    c"Cased_Letter".as_ptr(),
    c"Letter".as_ptr(),
    c"Mark,Combining_Mark".as_ptr(),
    c"Number".as_ptr(),
    c"Symbol".as_ptr(),
    c"Punctuation,punct".as_ptr(),
    c"Separator".as_ptr(),
    c"Other".as_ptr(),
];
pub type UnicodeScriptEnum1 = i32;
// unicode_gen_def.h:45
pub const SCRIPT_Unknown: i32 = 0;
// unicode_gen_def.h:46
pub const SCRIPT_Adlam: i32 = 1;
// unicode_gen_def.h:47
pub const SCRIPT_Ahom: i32 = 2;
// unicode_gen_def.h:48
pub const SCRIPT_Anatolian_Hieroglyphs: i32 = 3;
// unicode_gen_def.h:49
pub const SCRIPT_Arabic: i32 = 4;
// unicode_gen_def.h:50
pub const SCRIPT_Armenian: i32 = 5;
// unicode_gen_def.h:51
pub const SCRIPT_Avestan: i32 = 6;
// unicode_gen_def.h:52
pub const SCRIPT_Balinese: i32 = 7;
// unicode_gen_def.h:53
pub const SCRIPT_Bamum: i32 = 8;
// unicode_gen_def.h:54
pub const SCRIPT_Bassa_Vah: i32 = 9;
// unicode_gen_def.h:55
pub const SCRIPT_Batak: i32 = 10;
// unicode_gen_def.h:56
pub const SCRIPT_Beria_Erfe: i32 = 11;
// unicode_gen_def.h:57
pub const SCRIPT_Bengali: i32 = 12;
// unicode_gen_def.h:58
pub const SCRIPT_Bhaiksuki: i32 = 13;
// unicode_gen_def.h:59
pub const SCRIPT_Bopomofo: i32 = 14;
// unicode_gen_def.h:60
pub const SCRIPT_Brahmi: i32 = 15;
// unicode_gen_def.h:61
pub const SCRIPT_Braille: i32 = 16;
// unicode_gen_def.h:62
pub const SCRIPT_Buginese: i32 = 17;
// unicode_gen_def.h:63
pub const SCRIPT_Buhid: i32 = 18;
// unicode_gen_def.h:64
pub const SCRIPT_Canadian_Aboriginal: i32 = 19;
// unicode_gen_def.h:65
pub const SCRIPT_Carian: i32 = 20;
// unicode_gen_def.h:66
pub const SCRIPT_Caucasian_Albanian: i32 = 21;
// unicode_gen_def.h:67
pub const SCRIPT_Chakma: i32 = 22;
// unicode_gen_def.h:68
pub const SCRIPT_Cham: i32 = 23;
// unicode_gen_def.h:69
pub const SCRIPT_Cherokee: i32 = 24;
// unicode_gen_def.h:70
pub const SCRIPT_Chorasmian: i32 = 25;
// unicode_gen_def.h:71
pub const SCRIPT_Common: i32 = 26;
// unicode_gen_def.h:72
pub const SCRIPT_Coptic: i32 = 27;
// unicode_gen_def.h:73
pub const SCRIPT_Cuneiform: i32 = 28;
// unicode_gen_def.h:74
pub const SCRIPT_Cypriot: i32 = 29;
// unicode_gen_def.h:75
pub const SCRIPT_Cyrillic: i32 = 30;
// unicode_gen_def.h:76
pub const SCRIPT_Cypro_Minoan: i32 = 31;
// unicode_gen_def.h:77
pub const SCRIPT_Deseret: i32 = 32;
// unicode_gen_def.h:78
pub const SCRIPT_Devanagari: i32 = 33;
// unicode_gen_def.h:79
pub const SCRIPT_Dives_Akuru: i32 = 34;
// unicode_gen_def.h:80
pub const SCRIPT_Dogra: i32 = 35;
// unicode_gen_def.h:81
pub const SCRIPT_Duployan: i32 = 36;
// unicode_gen_def.h:82
pub const SCRIPT_Egyptian_Hieroglyphs: i32 = 37;
// unicode_gen_def.h:83
pub const SCRIPT_Elbasan: i32 = 38;
// unicode_gen_def.h:84
pub const SCRIPT_Elymaic: i32 = 39;
// unicode_gen_def.h:85
pub const SCRIPT_Ethiopic: i32 = 40;
// unicode_gen_def.h:86
pub const SCRIPT_Garay: i32 = 41;
// unicode_gen_def.h:87
pub const SCRIPT_Georgian: i32 = 42;
// unicode_gen_def.h:88
pub const SCRIPT_Glagolitic: i32 = 43;
// unicode_gen_def.h:89
pub const SCRIPT_Gothic: i32 = 44;
// unicode_gen_def.h:90
pub const SCRIPT_Grantha: i32 = 45;
// unicode_gen_def.h:91
pub const SCRIPT_Greek: i32 = 46;
// unicode_gen_def.h:92
pub const SCRIPT_Gujarati: i32 = 47;
// unicode_gen_def.h:93
pub const SCRIPT_Gunjala_Gondi: i32 = 48;
// unicode_gen_def.h:94
pub const SCRIPT_Gurmukhi: i32 = 49;
// unicode_gen_def.h:95
pub const SCRIPT_Gurung_Khema: i32 = 50;
// unicode_gen_def.h:96
pub const SCRIPT_Han: i32 = 51;
// unicode_gen_def.h:97
pub const SCRIPT_Hangul: i32 = 52;
// unicode_gen_def.h:98
pub const SCRIPT_Hanifi_Rohingya: i32 = 53;
// unicode_gen_def.h:99
pub const SCRIPT_Hanunoo: i32 = 54;
// unicode_gen_def.h:100
pub const SCRIPT_Hatran: i32 = 55;
// unicode_gen_def.h:101
pub const SCRIPT_Hebrew: i32 = 56;
// unicode_gen_def.h:102
pub const SCRIPT_Hiragana: i32 = 57;
// unicode_gen_def.h:103
pub const SCRIPT_Imperial_Aramaic: i32 = 58;
// unicode_gen_def.h:104
pub const SCRIPT_Inherited: i32 = 59;
// unicode_gen_def.h:105
pub const SCRIPT_Inscriptional_Pahlavi: i32 = 60;
// unicode_gen_def.h:106
pub const SCRIPT_Inscriptional_Parthian: i32 = 61;
// unicode_gen_def.h:107
pub const SCRIPT_Javanese: i32 = 62;
// unicode_gen_def.h:108
pub const SCRIPT_Kaithi: i32 = 63;
// unicode_gen_def.h:109
pub const SCRIPT_Kannada: i32 = 64;
// unicode_gen_def.h:110
pub const SCRIPT_Katakana: i32 = 65;
// unicode_gen_def.h:111
pub const SCRIPT_Katakana_Or_Hiragana: i32 = 66;
// unicode_gen_def.h:112
pub const SCRIPT_Kawi: i32 = 67;
// unicode_gen_def.h:113
pub const SCRIPT_Kayah_Li: i32 = 68;
// unicode_gen_def.h:114
pub const SCRIPT_Kharoshthi: i32 = 69;
// unicode_gen_def.h:115
pub const SCRIPT_Khmer: i32 = 70;
// unicode_gen_def.h:116
pub const SCRIPT_Khojki: i32 = 71;
// unicode_gen_def.h:117
pub const SCRIPT_Khitan_Small_Script: i32 = 72;
// unicode_gen_def.h:118
pub const SCRIPT_Khudawadi: i32 = 73;
// unicode_gen_def.h:119
pub const SCRIPT_Kirat_Rai: i32 = 74;
// unicode_gen_def.h:120
pub const SCRIPT_Lao: i32 = 75;
// unicode_gen_def.h:121
pub const SCRIPT_Latin: i32 = 76;
// unicode_gen_def.h:122
pub const SCRIPT_Lepcha: i32 = 77;
// unicode_gen_def.h:123
pub const SCRIPT_Limbu: i32 = 78;
// unicode_gen_def.h:124
pub const SCRIPT_Linear_A: i32 = 79;
// unicode_gen_def.h:125
pub const SCRIPT_Linear_B: i32 = 80;
// unicode_gen_def.h:126
pub const SCRIPT_Lisu: i32 = 81;
// unicode_gen_def.h:127
pub const SCRIPT_Lycian: i32 = 82;
// unicode_gen_def.h:128
pub const SCRIPT_Lydian: i32 = 83;
// unicode_gen_def.h:129
pub const SCRIPT_Makasar: i32 = 84;
// unicode_gen_def.h:130
pub const SCRIPT_Mahajani: i32 = 85;
// unicode_gen_def.h:131
pub const SCRIPT_Malayalam: i32 = 86;
// unicode_gen_def.h:132
pub const SCRIPT_Mandaic: i32 = 87;
// unicode_gen_def.h:133
pub const SCRIPT_Manichaean: i32 = 88;
// unicode_gen_def.h:134
pub const SCRIPT_Marchen: i32 = 89;
// unicode_gen_def.h:135
pub const SCRIPT_Masaram_Gondi: i32 = 90;
// unicode_gen_def.h:136
pub const SCRIPT_Medefaidrin: i32 = 91;
// unicode_gen_def.h:137
pub const SCRIPT_Meetei_Mayek: i32 = 92;
// unicode_gen_def.h:138
pub const SCRIPT_Mende_Kikakui: i32 = 93;
// unicode_gen_def.h:139
pub const SCRIPT_Meroitic_Cursive: i32 = 94;
// unicode_gen_def.h:140
pub const SCRIPT_Meroitic_Hieroglyphs: i32 = 95;
// unicode_gen_def.h:141
pub const SCRIPT_Miao: i32 = 96;
// unicode_gen_def.h:142
pub const SCRIPT_Modi: i32 = 97;
// unicode_gen_def.h:143
pub const SCRIPT_Mongolian: i32 = 98;
// unicode_gen_def.h:144
pub const SCRIPT_Mro: i32 = 99;
// unicode_gen_def.h:145
pub const SCRIPT_Multani: i32 = 100;
// unicode_gen_def.h:146
pub const SCRIPT_Myanmar: i32 = 101;
// unicode_gen_def.h:147
pub const SCRIPT_Nabataean: i32 = 102;
// unicode_gen_def.h:148
pub const SCRIPT_Nag_Mundari: i32 = 103;
// unicode_gen_def.h:149
pub const SCRIPT_Nandinagari: i32 = 104;
// unicode_gen_def.h:150
pub const SCRIPT_New_Tai_Lue: i32 = 105;
// unicode_gen_def.h:151
pub const SCRIPT_Newa: i32 = 106;
// unicode_gen_def.h:152
pub const SCRIPT_Nko: i32 = 107;
// unicode_gen_def.h:153
pub const SCRIPT_Nushu: i32 = 108;
// unicode_gen_def.h:154
pub const SCRIPT_Nyiakeng_Puachue_Hmong: i32 = 109;
// unicode_gen_def.h:155
pub const SCRIPT_Ogham: i32 = 110;
// unicode_gen_def.h:156
pub const SCRIPT_Ol_Chiki: i32 = 111;
// unicode_gen_def.h:157
pub const SCRIPT_Ol_Onal: i32 = 112;
// unicode_gen_def.h:158
pub const SCRIPT_Old_Hungarian: i32 = 113;
// unicode_gen_def.h:159
pub const SCRIPT_Old_Italic: i32 = 114;
// unicode_gen_def.h:160
pub const SCRIPT_Old_North_Arabian: i32 = 115;
// unicode_gen_def.h:161
pub const SCRIPT_Old_Permic: i32 = 116;
// unicode_gen_def.h:162
pub const SCRIPT_Old_Persian: i32 = 117;
// unicode_gen_def.h:163
pub const SCRIPT_Old_Sogdian: i32 = 118;
// unicode_gen_def.h:164
pub const SCRIPT_Old_South_Arabian: i32 = 119;
// unicode_gen_def.h:165
pub const SCRIPT_Old_Turkic: i32 = 120;
// unicode_gen_def.h:166
pub const SCRIPT_Old_Uyghur: i32 = 121;
// unicode_gen_def.h:167
pub const SCRIPT_Oriya: i32 = 122;
// unicode_gen_def.h:168
pub const SCRIPT_Osage: i32 = 123;
// unicode_gen_def.h:169
pub const SCRIPT_Osmanya: i32 = 124;
// unicode_gen_def.h:170
pub const SCRIPT_Pahawh_Hmong: i32 = 125;
// unicode_gen_def.h:171
pub const SCRIPT_Palmyrene: i32 = 126;
// unicode_gen_def.h:172
pub const SCRIPT_Pau_Cin_Hau: i32 = 127;
// unicode_gen_def.h:173
pub const SCRIPT_Phags_Pa: i32 = 128;
// unicode_gen_def.h:174
pub const SCRIPT_Phoenician: i32 = 129;
// unicode_gen_def.h:175
pub const SCRIPT_Psalter_Pahlavi: i32 = 130;
// unicode_gen_def.h:176
pub const SCRIPT_Rejang: i32 = 131;
// unicode_gen_def.h:177
pub const SCRIPT_Runic: i32 = 132;
// unicode_gen_def.h:178
pub const SCRIPT_Samaritan: i32 = 133;
// unicode_gen_def.h:179
pub const SCRIPT_Saurashtra: i32 = 134;
// unicode_gen_def.h:180
pub const SCRIPT_Sharada: i32 = 135;
// unicode_gen_def.h:181
pub const SCRIPT_Shavian: i32 = 136;
// unicode_gen_def.h:182
pub const SCRIPT_Siddham: i32 = 137;
// unicode_gen_def.h:183
pub const SCRIPT_Sidetic: i32 = 138;
// unicode_gen_def.h:184
pub const SCRIPT_SignWriting: i32 = 139;
// unicode_gen_def.h:185
pub const SCRIPT_Sinhala: i32 = 140;
// unicode_gen_def.h:186
pub const SCRIPT_Sogdian: i32 = 141;
// unicode_gen_def.h:187
pub const SCRIPT_Sora_Sompeng: i32 = 142;
// unicode_gen_def.h:188
pub const SCRIPT_Soyombo: i32 = 143;
// unicode_gen_def.h:189
pub const SCRIPT_Sundanese: i32 = 144;
// unicode_gen_def.h:190
pub const SCRIPT_Sunuwar: i32 = 145;
// unicode_gen_def.h:191
pub const SCRIPT_Syloti_Nagri: i32 = 146;
// unicode_gen_def.h:192
pub const SCRIPT_Syriac: i32 = 147;
// unicode_gen_def.h:193
pub const SCRIPT_Tagalog: i32 = 148;
// unicode_gen_def.h:194
pub const SCRIPT_Tagbanwa: i32 = 149;
// unicode_gen_def.h:195
pub const SCRIPT_Tai_Le: i32 = 150;
// unicode_gen_def.h:196
pub const SCRIPT_Tai_Tham: i32 = 151;
// unicode_gen_def.h:197
pub const SCRIPT_Tai_Viet: i32 = 152;
// unicode_gen_def.h:198
pub const SCRIPT_Tai_Yo: i32 = 153;
// unicode_gen_def.h:199
pub const SCRIPT_Takri: i32 = 154;
// unicode_gen_def.h:200
pub const SCRIPT_Tamil: i32 = 155;
// unicode_gen_def.h:201
pub const SCRIPT_Tangut: i32 = 156;
// unicode_gen_def.h:202
pub const SCRIPT_Telugu: i32 = 157;
// unicode_gen_def.h:203
pub const SCRIPT_Thaana: i32 = 158;
// unicode_gen_def.h:204
pub const SCRIPT_Thai: i32 = 159;
// unicode_gen_def.h:205
pub const SCRIPT_Tibetan: i32 = 160;
// unicode_gen_def.h:206
pub const SCRIPT_Tifinagh: i32 = 161;
// unicode_gen_def.h:207
pub const SCRIPT_Tirhuta: i32 = 162;
// unicode_gen_def.h:208
pub const SCRIPT_Tangsa: i32 = 163;
// unicode_gen_def.h:209
pub const SCRIPT_Todhri: i32 = 164;
// unicode_gen_def.h:210
pub const SCRIPT_Tolong_Siki: i32 = 165;
// unicode_gen_def.h:211
pub const SCRIPT_Toto: i32 = 166;
// unicode_gen_def.h:212
pub const SCRIPT_Tulu_Tigalari: i32 = 167;
// unicode_gen_def.h:213
pub const SCRIPT_Ugaritic: i32 = 168;
// unicode_gen_def.h:214
pub const SCRIPT_Vai: i32 = 169;
// unicode_gen_def.h:215
pub const SCRIPT_Vithkuqi: i32 = 170;
// unicode_gen_def.h:216
pub const SCRIPT_Wancho: i32 = 171;
// unicode_gen_def.h:217
pub const SCRIPT_Warang_Citi: i32 = 172;
// unicode_gen_def.h:218
pub const SCRIPT_Yezidi: i32 = 173;
// unicode_gen_def.h:219
pub const SCRIPT_Yi: i32 = 174;
// unicode_gen_def.h:220
pub const SCRIPT_Zanabazar_Square: i32 = 175;
pub const SCRIPT_COUNT: i32 = 176;
pub static mut unicode_script_name: [*const c_char; 176] = [
    c"Unknown".as_ptr(),
    c"Adlam".as_ptr(),
    c"Ahom".as_ptr(),
    c"Anatolian_Hieroglyphs".as_ptr(),
    c"Arabic".as_ptr(),
    c"Armenian".as_ptr(),
    c"Avestan".as_ptr(),
    c"Balinese".as_ptr(),
    c"Bamum".as_ptr(),
    c"Bassa_Vah".as_ptr(),
    c"Batak".as_ptr(),
    c"Beria_Erfe".as_ptr(),
    c"Bengali".as_ptr(),
    c"Bhaiksuki".as_ptr(),
    c"Bopomofo".as_ptr(),
    c"Brahmi".as_ptr(),
    c"Braille".as_ptr(),
    c"Buginese".as_ptr(),
    c"Buhid".as_ptr(),
    c"Canadian_Aboriginal".as_ptr(),
    c"Carian".as_ptr(),
    c"Caucasian_Albanian".as_ptr(),
    c"Chakma".as_ptr(),
    c"Cham".as_ptr(),
    c"Cherokee".as_ptr(),
    c"Chorasmian".as_ptr(),
    c"Common".as_ptr(),
    c"Coptic".as_ptr(),
    c"Cuneiform".as_ptr(),
    c"Cypriot".as_ptr(),
    c"Cyrillic".as_ptr(),
    c"Cypro_Minoan".as_ptr(),
    c"Deseret".as_ptr(),
    c"Devanagari".as_ptr(),
    c"Dives_Akuru".as_ptr(),
    c"Dogra".as_ptr(),
    c"Duployan".as_ptr(),
    c"Egyptian_Hieroglyphs".as_ptr(),
    c"Elbasan".as_ptr(),
    c"Elymaic".as_ptr(),
    c"Ethiopic".as_ptr(),
    c"Garay".as_ptr(),
    c"Georgian".as_ptr(),
    c"Glagolitic".as_ptr(),
    c"Gothic".as_ptr(),
    c"Grantha".as_ptr(),
    c"Greek".as_ptr(),
    c"Gujarati".as_ptr(),
    c"Gunjala_Gondi".as_ptr(),
    c"Gurmukhi".as_ptr(),
    c"Gurung_Khema".as_ptr(),
    c"Han".as_ptr(),
    c"Hangul".as_ptr(),
    c"Hanifi_Rohingya".as_ptr(),
    c"Hanunoo".as_ptr(),
    c"Hatran".as_ptr(),
    c"Hebrew".as_ptr(),
    c"Hiragana".as_ptr(),
    c"Imperial_Aramaic".as_ptr(),
    c"Inherited".as_ptr(),
    c"Inscriptional_Pahlavi".as_ptr(),
    c"Inscriptional_Parthian".as_ptr(),
    c"Javanese".as_ptr(),
    c"Kaithi".as_ptr(),
    c"Kannada".as_ptr(),
    c"Katakana".as_ptr(),
    c"Katakana_Or_Hiragana".as_ptr(),
    c"Kawi".as_ptr(),
    c"Kayah_Li".as_ptr(),
    c"Kharoshthi".as_ptr(),
    c"Khmer".as_ptr(),
    c"Khojki".as_ptr(),
    c"Khitan_Small_Script".as_ptr(),
    c"Khudawadi".as_ptr(),
    c"Kirat_Rai".as_ptr(),
    c"Lao".as_ptr(),
    c"Latin".as_ptr(),
    c"Lepcha".as_ptr(),
    c"Limbu".as_ptr(),
    c"Linear_A".as_ptr(),
    c"Linear_B".as_ptr(),
    c"Lisu".as_ptr(),
    c"Lycian".as_ptr(),
    c"Lydian".as_ptr(),
    c"Makasar".as_ptr(),
    c"Mahajani".as_ptr(),
    c"Malayalam".as_ptr(),
    c"Mandaic".as_ptr(),
    c"Manichaean".as_ptr(),
    c"Marchen".as_ptr(),
    c"Masaram_Gondi".as_ptr(),
    c"Medefaidrin".as_ptr(),
    c"Meetei_Mayek".as_ptr(),
    c"Mende_Kikakui".as_ptr(),
    c"Meroitic_Cursive".as_ptr(),
    c"Meroitic_Hieroglyphs".as_ptr(),
    c"Miao".as_ptr(),
    c"Modi".as_ptr(),
    c"Mongolian".as_ptr(),
    c"Mro".as_ptr(),
    c"Multani".as_ptr(),
    c"Myanmar".as_ptr(),
    c"Nabataean".as_ptr(),
    c"Nag_Mundari".as_ptr(),
    c"Nandinagari".as_ptr(),
    c"New_Tai_Lue".as_ptr(),
    c"Newa".as_ptr(),
    c"Nko".as_ptr(),
    c"Nushu".as_ptr(),
    c"Nyiakeng_Puachue_Hmong".as_ptr(),
    c"Ogham".as_ptr(),
    c"Ol_Chiki".as_ptr(),
    c"Ol_Onal".as_ptr(),
    c"Old_Hungarian".as_ptr(),
    c"Old_Italic".as_ptr(),
    c"Old_North_Arabian".as_ptr(),
    c"Old_Permic".as_ptr(),
    c"Old_Persian".as_ptr(),
    c"Old_Sogdian".as_ptr(),
    c"Old_South_Arabian".as_ptr(),
    c"Old_Turkic".as_ptr(),
    c"Old_Uyghur".as_ptr(),
    c"Oriya".as_ptr(),
    c"Osage".as_ptr(),
    c"Osmanya".as_ptr(),
    c"Pahawh_Hmong".as_ptr(),
    c"Palmyrene".as_ptr(),
    c"Pau_Cin_Hau".as_ptr(),
    c"Phags_Pa".as_ptr(),
    c"Phoenician".as_ptr(),
    c"Psalter_Pahlavi".as_ptr(),
    c"Rejang".as_ptr(),
    c"Runic".as_ptr(),
    c"Samaritan".as_ptr(),
    c"Saurashtra".as_ptr(),
    c"Sharada".as_ptr(),
    c"Shavian".as_ptr(),
    c"Siddham".as_ptr(),
    c"Sidetic".as_ptr(),
    c"SignWriting".as_ptr(),
    c"Sinhala".as_ptr(),
    c"Sogdian".as_ptr(),
    c"Sora_Sompeng".as_ptr(),
    c"Soyombo".as_ptr(),
    c"Sundanese".as_ptr(),
    c"Sunuwar".as_ptr(),
    c"Syloti_Nagri".as_ptr(),
    c"Syriac".as_ptr(),
    c"Tagalog".as_ptr(),
    c"Tagbanwa".as_ptr(),
    c"Tai_Le".as_ptr(),
    c"Tai_Tham".as_ptr(),
    c"Tai_Viet".as_ptr(),
    c"Tai_Yo".as_ptr(),
    c"Takri".as_ptr(),
    c"Tamil".as_ptr(),
    c"Tangut".as_ptr(),
    c"Telugu".as_ptr(),
    c"Thaana".as_ptr(),
    c"Thai".as_ptr(),
    c"Tibetan".as_ptr(),
    c"Tifinagh".as_ptr(),
    c"Tirhuta".as_ptr(),
    c"Tangsa".as_ptr(),
    c"Todhri".as_ptr(),
    c"Tolong_Siki".as_ptr(),
    c"Toto".as_ptr(),
    c"Tulu_Tigalari".as_ptr(),
    c"Ugaritic".as_ptr(),
    c"Vai".as_ptr(),
    c"Vithkuqi".as_ptr(),
    c"Wancho".as_ptr(),
    c"Warang_Citi".as_ptr(),
    c"Yezidi".as_ptr(),
    c"Yi".as_ptr(),
    c"Zanabazar_Square".as_ptr(),
];
pub static mut unicode_script_short_name: [*const c_char; 176] = [
    c"Zzzz".as_ptr(),
    c"Adlm".as_ptr(),
    c"Ahom".as_ptr(),
    c"Hluw".as_ptr(),
    c"Arab".as_ptr(),
    c"Armn".as_ptr(),
    c"Avst".as_ptr(),
    c"Bali".as_ptr(),
    c"Bamu".as_ptr(),
    c"Bass".as_ptr(),
    c"Batk".as_ptr(),
    c"Berf".as_ptr(),
    c"Beng".as_ptr(),
    c"Bhks".as_ptr(),
    c"Bopo".as_ptr(),
    c"Brah".as_ptr(),
    c"Brai".as_ptr(),
    c"Bugi".as_ptr(),
    c"Buhd".as_ptr(),
    c"Cans".as_ptr(),
    c"Cari".as_ptr(),
    c"Aghb".as_ptr(),
    c"Cakm".as_ptr(),
    c"Cham".as_ptr(),
    c"Cher".as_ptr(),
    c"Chrs".as_ptr(),
    c"Zyyy".as_ptr(),
    c"Copt,Qaac".as_ptr(),
    c"Xsux".as_ptr(),
    c"Cprt".as_ptr(),
    c"Cyrl".as_ptr(),
    c"Cpmn".as_ptr(),
    c"Dsrt".as_ptr(),
    c"Deva".as_ptr(),
    c"Diak".as_ptr(),
    c"Dogr".as_ptr(),
    c"Dupl".as_ptr(),
    c"Egyp".as_ptr(),
    c"Elba".as_ptr(),
    c"Elym".as_ptr(),
    c"Ethi".as_ptr(),
    c"Gara".as_ptr(),
    c"Geor".as_ptr(),
    c"Glag".as_ptr(),
    c"Goth".as_ptr(),
    c"Gran".as_ptr(),
    c"Grek".as_ptr(),
    c"Gujr".as_ptr(),
    c"Gong".as_ptr(),
    c"Guru".as_ptr(),
    c"Gukh".as_ptr(),
    c"Hani".as_ptr(),
    c"Hang".as_ptr(),
    c"Rohg".as_ptr(),
    c"Hano".as_ptr(),
    c"Hatr".as_ptr(),
    c"Hebr".as_ptr(),
    c"Hira".as_ptr(),
    c"Armi".as_ptr(),
    c"Zinh,Qaai".as_ptr(),
    c"Phli".as_ptr(),
    c"Prti".as_ptr(),
    c"Java".as_ptr(),
    c"Kthi".as_ptr(),
    c"Knda".as_ptr(),
    c"Kana".as_ptr(),
    c"Hrkt".as_ptr(),
    c"Kawi".as_ptr(),
    c"Kali".as_ptr(),
    c"Khar".as_ptr(),
    c"Khmr".as_ptr(),
    c"Khoj".as_ptr(),
    c"Kits".as_ptr(),
    c"Sind".as_ptr(),
    c"Krai".as_ptr(),
    c"Laoo".as_ptr(),
    c"Latn".as_ptr(),
    c"Lepc".as_ptr(),
    c"Limb".as_ptr(),
    c"Lina".as_ptr(),
    c"Linb".as_ptr(),
    c"Lisu".as_ptr(),
    c"Lyci".as_ptr(),
    c"Lydi".as_ptr(),
    c"Maka".as_ptr(),
    c"Mahj".as_ptr(),
    c"Mlym".as_ptr(),
    c"Mand".as_ptr(),
    c"Mani".as_ptr(),
    c"Marc".as_ptr(),
    c"Gonm".as_ptr(),
    c"Medf".as_ptr(),
    c"Mtei".as_ptr(),
    c"Mend".as_ptr(),
    c"Merc".as_ptr(),
    c"Mero".as_ptr(),
    c"Plrd".as_ptr(),
    c"Modi".as_ptr(),
    c"Mong".as_ptr(),
    c"Mroo".as_ptr(),
    c"Mult".as_ptr(),
    c"Mymr".as_ptr(),
    c"Nbat".as_ptr(),
    c"Nagm".as_ptr(),
    c"Nand".as_ptr(),
    c"Talu".as_ptr(),
    c"Newa".as_ptr(),
    c"Nkoo".as_ptr(),
    c"Nshu".as_ptr(),
    c"Hmnp".as_ptr(),
    c"Ogam".as_ptr(),
    c"Olck".as_ptr(),
    c"Onao".as_ptr(),
    c"Hung".as_ptr(),
    c"Ital".as_ptr(),
    c"Narb".as_ptr(),
    c"Perm".as_ptr(),
    c"Xpeo".as_ptr(),
    c"Sogo".as_ptr(),
    c"Sarb".as_ptr(),
    c"Orkh".as_ptr(),
    c"Ougr".as_ptr(),
    c"Orya".as_ptr(),
    c"Osge".as_ptr(),
    c"Osma".as_ptr(),
    c"Hmng".as_ptr(),
    c"Palm".as_ptr(),
    c"Pauc".as_ptr(),
    c"Phag".as_ptr(),
    c"Phnx".as_ptr(),
    c"Phlp".as_ptr(),
    c"Rjng".as_ptr(),
    c"Runr".as_ptr(),
    c"Samr".as_ptr(),
    c"Saur".as_ptr(),
    c"Shrd".as_ptr(),
    c"Shaw".as_ptr(),
    c"Sidd".as_ptr(),
    c"Sidt".as_ptr(),
    c"Sgnw".as_ptr(),
    c"Sinh".as_ptr(),
    c"Sogd".as_ptr(),
    c"Sora".as_ptr(),
    c"Soyo".as_ptr(),
    c"Sund".as_ptr(),
    c"Sunu".as_ptr(),
    c"Sylo".as_ptr(),
    c"Syrc".as_ptr(),
    c"Tglg".as_ptr(),
    c"Tagb".as_ptr(),
    c"Tale".as_ptr(),
    c"Lana".as_ptr(),
    c"Tavt".as_ptr(),
    c"Tayo".as_ptr(),
    c"Takr".as_ptr(),
    c"Taml".as_ptr(),
    c"Tang".as_ptr(),
    c"Telu".as_ptr(),
    c"Thaa".as_ptr(),
    c"Thai".as_ptr(),
    c"Tibt".as_ptr(),
    c"Tfng".as_ptr(),
    c"Tirh".as_ptr(),
    c"Tnsa".as_ptr(),
    c"Todr".as_ptr(),
    c"Tols".as_ptr(),
    c"Toto".as_ptr(),
    c"Tutg".as_ptr(),
    c"Ugar".as_ptr(),
    c"Vaii".as_ptr(),
    c"Vith".as_ptr(),
    c"Wcho".as_ptr(),
    c"Wara".as_ptr(),
    c"Yezi".as_ptr(),
    c"Yiii".as_ptr(),
    c"Zanb".as_ptr(),
];
pub type UnicodePropEnum1 = i32;
// unicode_gen_def.h:225
pub const PROP_Hyphen: i32 = 0;
// unicode_gen_def.h:226
pub const PROP_Other_Math: i32 = 1;
// unicode_gen_def.h:227
pub const PROP_Other_Alphabetic: i32 = 2;
// unicode_gen_def.h:228
pub const PROP_Other_Lowercase: i32 = 3;
// unicode_gen_def.h:229
pub const PROP_Other_Uppercase: i32 = 4;
// unicode_gen_def.h:230
pub const PROP_Other_Grapheme_Extend: i32 = 5;
// unicode_gen_def.h:231
pub const PROP_Other_Default_Ignorable_Code_Point: i32 = 6;
// unicode_gen_def.h:232
pub const PROP_Other_ID_Start: i32 = 7;
// unicode_gen_def.h:233
pub const PROP_Other_ID_Continue: i32 = 8;
// unicode_gen_def.h:234
pub const PROP_Prepended_Concatenation_Mark: i32 = 9;
// unicode_gen_def.h:236
pub const PROP_ID_Continue1: i32 = 10;
// unicode_gen_def.h:237
pub const PROP_XID_Start1: i32 = 11;
// unicode_gen_def.h:238
pub const PROP_XID_Continue1: i32 = 12;
// unicode_gen_def.h:239
pub const PROP_Changes_When_Titlecased1: i32 = 13;
// unicode_gen_def.h:240
pub const PROP_Changes_When_Casefolded1: i32 = 14;
// unicode_gen_def.h:241
pub const PROP_Changes_When_NFKC_Casefolded1: i32 = 15;
// unicode_gen_def.h:242
pub const PROP_Basic_Emoji1: i32 = 16;
// unicode_gen_def.h:243
pub const PROP_Basic_Emoji2: i32 = 17;
// unicode_gen_def.h:244
pub const PROP_RGI_Emoji_Modifier_Sequence: i32 = 18;
// unicode_gen_def.h:245
pub const PROP_RGI_Emoji_Flag_Sequence: i32 = 19;
// unicode_gen_def.h:246
pub const PROP_Emoji_Keycap_Sequence: i32 = 20;
// unicode_gen_def.h:249
pub const PROP_ASCII_Hex_Digit: i32 = 21;
// unicode_gen_def.h:250
pub const PROP_Bidi_Control: i32 = 22;
// unicode_gen_def.h:251
pub const PROP_Dash: i32 = 23;
// unicode_gen_def.h:252
pub const PROP_Deprecated: i32 = 24;
// unicode_gen_def.h:253
pub const PROP_Diacritic: i32 = 25;
// unicode_gen_def.h:254
pub const PROP_Extender: i32 = 26;
// unicode_gen_def.h:255
pub const PROP_Hex_Digit: i32 = 27;
// unicode_gen_def.h:256
pub const PROP_IDS_Unary_Operator: i32 = 28;
// unicode_gen_def.h:257
pub const PROP_IDS_Binary_Operator: i32 = 29;
// unicode_gen_def.h:258
pub const PROP_IDS_Trinary_Operator: i32 = 30;
// unicode_gen_def.h:259
pub const PROP_Ideographic: i32 = 31;
// unicode_gen_def.h:260
pub const PROP_Join_Control: i32 = 32;
// unicode_gen_def.h:261
pub const PROP_Logical_Order_Exception: i32 = 33;
// unicode_gen_def.h:262
pub const PROP_Modifier_Combining_Mark: i32 = 34;
// unicode_gen_def.h:263
pub const PROP_Noncharacter_Code_Point: i32 = 35;
// unicode_gen_def.h:264
pub const PROP_Pattern_Syntax: i32 = 36;
// unicode_gen_def.h:265
pub const PROP_Pattern_White_Space: i32 = 37;
// unicode_gen_def.h:266
pub const PROP_Quotation_Mark: i32 = 38;
// unicode_gen_def.h:267
pub const PROP_Radical: i32 = 39;
// unicode_gen_def.h:268
pub const PROP_Regional_Indicator: i32 = 40;
// unicode_gen_def.h:269
pub const PROP_Sentence_Terminal: i32 = 41;
// unicode_gen_def.h:270
pub const PROP_Soft_Dotted: i32 = 42;
// unicode_gen_def.h:271
pub const PROP_Terminal_Punctuation: i32 = 43;
// unicode_gen_def.h:272
pub const PROP_Unified_Ideograph: i32 = 44;
// unicode_gen_def.h:273
pub const PROP_Variation_Selector: i32 = 45;
// unicode_gen_def.h:274
pub const PROP_White_Space: i32 = 46;
// unicode_gen_def.h:275
pub const PROP_Bidi_Mirrored: i32 = 47;
// unicode_gen_def.h:276
pub const PROP_Emoji: i32 = 48;
// unicode_gen_def.h:277
pub const PROP_Emoji_Component: i32 = 49;
// unicode_gen_def.h:278
pub const PROP_Emoji_Modifier: i32 = 50;
// unicode_gen_def.h:279
pub const PROP_Emoji_Modifier_Base: i32 = 51;
// unicode_gen_def.h:280
pub const PROP_Emoji_Presentation: i32 = 52;
// unicode_gen_def.h:281
pub const PROP_Extended_Pictographic: i32 = 53;
// unicode_gen_def.h:282
pub const PROP_Default_Ignorable_Code_Point: i32 = 54;
// unicode_gen_def.h:283
pub const PROP_ID_Start: i32 = 55;
// unicode_gen_def.h:284
pub const PROP_Case_Ignorable: i32 = 56;
// unicode_gen_def.h:287
pub const PROP_ASCII: i32 = 57;
// unicode_gen_def.h:288
pub const PROP_Alphabetic: i32 = 58;
// unicode_gen_def.h:289
pub const PROP_Any: i32 = 59;
// unicode_gen_def.h:290
pub const PROP_Assigned: i32 = 60;
// unicode_gen_def.h:291
pub const PROP_Cased: i32 = 61;
// unicode_gen_def.h:292
pub const PROP_Changes_When_Casefolded: i32 = 62;
// unicode_gen_def.h:293
pub const PROP_Changes_When_Casemapped: i32 = 63;
// unicode_gen_def.h:294
pub const PROP_Changes_When_Lowercased: i32 = 64;
// unicode_gen_def.h:295
pub const PROP_Changes_When_NFKC_Casefolded: i32 = 65;
// unicode_gen_def.h:296
pub const PROP_Changes_When_Titlecased: i32 = 66;
// unicode_gen_def.h:297
pub const PROP_Changes_When_Uppercased: i32 = 67;
// unicode_gen_def.h:298
pub const PROP_Grapheme_Base: i32 = 68;
// unicode_gen_def.h:299
pub const PROP_Grapheme_Extend: i32 = 69;
// unicode_gen_def.h:300
pub const PROP_ID_Continue: i32 = 70;
// unicode_gen_def.h:301
pub const PROP_ID_Compat_Math_Start: i32 = 71;
// unicode_gen_def.h:302
pub const PROP_ID_Compat_Math_Continue: i32 = 72;
// unicode_gen_def.h:303
pub const PROP_InCB: i32 = 73;
// unicode_gen_def.h:304
pub const PROP_Lowercase: i32 = 74;
// unicode_gen_def.h:305
pub const PROP_Math: i32 = 75;
// unicode_gen_def.h:306
pub const PROP_Uppercase: i32 = 76;
// unicode_gen_def.h:307
pub const PROP_XID_Continue: i32 = 77;
// unicode_gen_def.h:308
pub const PROP_XID_Start: i32 = 78;
// unicode_gen_def.h:311
pub const PROP_Cased1: i32 = 79;
pub const PROP_COUNT: i32 = 80;
pub static mut unicode_prop_name: [*const c_char; 80] = [
    c"Hyphen".as_ptr(),
    c"Other_Math".as_ptr(),
    c"Other_Alphabetic".as_ptr(),
    c"Other_Lowercase".as_ptr(),
    c"Other_Uppercase".as_ptr(),
    c"Other_Grapheme_Extend".as_ptr(),
    c"Other_Default_Ignorable_Code_Point".as_ptr(),
    c"Other_ID_Start".as_ptr(),
    c"Other_ID_Continue".as_ptr(),
    c"Prepended_Concatenation_Mark".as_ptr(),
    c"ID_Continue1".as_ptr(),
    c"XID_Start1".as_ptr(),
    c"XID_Continue1".as_ptr(),
    c"Changes_When_Titlecased1".as_ptr(),
    c"Changes_When_Casefolded1".as_ptr(),
    c"Changes_When_NFKC_Casefolded1".as_ptr(),
    c"Basic_Emoji1".as_ptr(),
    c"Basic_Emoji2".as_ptr(),
    c"RGI_Emoji_Modifier_Sequence".as_ptr(),
    c"RGI_Emoji_Flag_Sequence".as_ptr(),
    c"Emoji_Keycap_Sequence".as_ptr(),
    c"ASCII_Hex_Digit".as_ptr(),
    c"Bidi_Control".as_ptr(),
    c"Dash".as_ptr(),
    c"Deprecated".as_ptr(),
    c"Diacritic".as_ptr(),
    c"Extender".as_ptr(),
    c"Hex_Digit".as_ptr(),
    c"IDS_Unary_Operator".as_ptr(),
    c"IDS_Binary_Operator".as_ptr(),
    c"IDS_Trinary_Operator".as_ptr(),
    c"Ideographic".as_ptr(),
    c"Join_Control".as_ptr(),
    c"Logical_Order_Exception".as_ptr(),
    c"Modifier_Combining_Mark".as_ptr(),
    c"Noncharacter_Code_Point".as_ptr(),
    c"Pattern_Syntax".as_ptr(),
    c"Pattern_White_Space".as_ptr(),
    c"Quotation_Mark".as_ptr(),
    c"Radical".as_ptr(),
    c"Regional_Indicator".as_ptr(),
    c"Sentence_Terminal".as_ptr(),
    c"Soft_Dotted".as_ptr(),
    c"Terminal_Punctuation".as_ptr(),
    c"Unified_Ideograph".as_ptr(),
    c"Variation_Selector".as_ptr(),
    c"White_Space".as_ptr(),
    c"Bidi_Mirrored".as_ptr(),
    c"Emoji".as_ptr(),
    c"Emoji_Component".as_ptr(),
    c"Emoji_Modifier".as_ptr(),
    c"Emoji_Modifier_Base".as_ptr(),
    c"Emoji_Presentation".as_ptr(),
    c"Extended_Pictographic".as_ptr(),
    c"Default_Ignorable_Code_Point".as_ptr(),
    c"ID_Start".as_ptr(),
    c"Case_Ignorable".as_ptr(),
    c"ASCII".as_ptr(),
    c"Alphabetic".as_ptr(),
    c"Any".as_ptr(),
    c"Assigned".as_ptr(),
    c"Cased".as_ptr(),
    c"Changes_When_Casefolded".as_ptr(),
    c"Changes_When_Casemapped".as_ptr(),
    c"Changes_When_Lowercased".as_ptr(),
    c"Changes_When_NFKC_Casefolded".as_ptr(),
    c"Changes_When_Titlecased".as_ptr(),
    c"Changes_When_Uppercased".as_ptr(),
    c"Grapheme_Base".as_ptr(),
    c"Grapheme_Extend".as_ptr(),
    c"ID_Continue".as_ptr(),
    c"ID_Compat_Math_Start".as_ptr(),
    c"ID_Compat_Math_Continue".as_ptr(),
    c"InCB".as_ptr(),
    c"Lowercase".as_ptr(),
    c"Math".as_ptr(),
    c"Uppercase".as_ptr(),
    c"XID_Continue".as_ptr(),
    c"XID_Start".as_ptr(),
    c"Cased1".as_ptr(),
];
pub static mut unicode_prop_short_name: [*const c_char; 80] = [
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"AHex".as_ptr(),
    c"Bidi_C".as_ptr(),
    c"".as_ptr(),
    c"Dep".as_ptr(),
    c"Dia".as_ptr(),
    c"Ext".as_ptr(),
    c"Hex".as_ptr(),
    c"IDSU".as_ptr(),
    c"IDSB".as_ptr(),
    c"IDST".as_ptr(),
    c"Ideo".as_ptr(),
    c"Join_C".as_ptr(),
    c"LOE".as_ptr(),
    c"MCM".as_ptr(),
    c"NChar".as_ptr(),
    c"Pat_Syn".as_ptr(),
    c"Pat_WS".as_ptr(),
    c"QMark".as_ptr(),
    c"".as_ptr(),
    c"RI".as_ptr(),
    c"STerm".as_ptr(),
    c"SD".as_ptr(),
    c"Term".as_ptr(),
    c"UIdeo".as_ptr(),
    c"VS".as_ptr(),
    c"space".as_ptr(),
    c"Bidi_M".as_ptr(),
    c"".as_ptr(),
    c"EComp".as_ptr(),
    c"EMod".as_ptr(),
    c"EBase".as_ptr(),
    c"EPres".as_ptr(),
    c"ExtPict".as_ptr(),
    c"DI".as_ptr(),
    c"IDS".as_ptr(),
    c"CI".as_ptr(),
    c"".as_ptr(),
    c"Alpha".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"CWCF".as_ptr(),
    c"CWCM".as_ptr(),
    c"CWL".as_ptr(),
    c"CWKCF".as_ptr(),
    c"CWT".as_ptr(),
    c"CWU".as_ptr(),
    c"Gr_Base".as_ptr(),
    c"Gr_Ext".as_ptr(),
    c"IDC".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"".as_ptr(),
    c"Lower".as_ptr(),
    c"".as_ptr(),
    c"Upper".as_ptr(),
    c"XIDC".as_ptr(),
    c"XIDS".as_ptr(),
    c"".as_ptr(),
];
pub type UnicodeSequencePropEnum1 = i32;
// unicode_gen_def.h:316
pub const SEQUENCE_PROP_Basic_Emoji: i32 = 0;
// unicode_gen_def.h:317
pub const SEQUENCE_PROP_Emoji_Keycap_Sequence: i32 = 1;
// unicode_gen_def.h:318
pub const SEQUENCE_PROP_RGI_Emoji_Modifier_Sequence: i32 = 2;
// unicode_gen_def.h:319
pub const SEQUENCE_PROP_RGI_Emoji_Flag_Sequence: i32 = 3;
// unicode_gen_def.h:320
pub const SEQUENCE_PROP_RGI_Emoji_Tag_Sequence: i32 = 4;
// unicode_gen_def.h:321
pub const SEQUENCE_PROP_RGI_Emoji_ZWJ_Sequence: i32 = 5;
// unicode_gen_def.h:322
pub const SEQUENCE_PROP_RGI_Emoji: i32 = 6;
pub const SEQUENCE_PROP_COUNT: i32 = 7;
pub static mut unicode_sequence_prop_name: [*const c_char; 7] = [
    c"Basic_Emoji".as_ptr(),
    c"Emoji_Keycap_Sequence".as_ptr(),
    c"RGI_Emoji_Modifier_Sequence".as_ptr(),
    c"RGI_Emoji_Flag_Sequence".as_ptr(),
    c"RGI_Emoji_Tag_Sequence".as_ptr(),
    c"RGI_Emoji_ZWJ_Sequence".as_ptr(),
    c"RGI_Emoji".as_ptr(),
];
