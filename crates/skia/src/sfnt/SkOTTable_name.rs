// Fixed upstream cacf77bdba7ba7df8ea7236d7e14b08c658ff368.
// Unverified translation. Packed fields retain raw big-endian storage.
// Source: src/sfnt/SkOTTable_name.h:17-34
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkOTTableName {
    pub format: u16,
    pub count: u16,
    pub stringOffset: u16,
}
impl SkOTTableName {
    pub const TAG0: i8 = b'n' as i8;
    pub const TAG1: i8 = b'a' as i8;
    pub const TAG2: i8 = b'm' as i8;
    pub const TAG3: i8 = b'e' as i8;
    pub const TAG: u32 = u32::from_be_bytes(*b"name").to_be();
    pub const format_0: u16 = 0u16.to_be();
    pub const format_1: u16 = 1u16.to_be();
}
// Source: src/sfnt/SkOTTable_name.h:38-44
pub mod PlatformID {
    // Source: src/sfnt/SkOTTable_name.h:39-39
    pub const Unicode: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:40-40
    pub const Macintosh: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:41-41
    pub const ISO: u16 = (2u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:42-42
    pub const Windows: u16 = (3u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:43-43
    pub const Custom: u16 = (4u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:52-60
pub mod Unicode {
    // Source: src/sfnt/SkOTTable_name.h:53-53
    pub const Unicode10: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:54-54
    pub const Unicode11: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:55-55
    pub const ISO10646: u16 = (2u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:56-56
    pub const Unicode20BMP: u16 = (3u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:57-57
    pub const Unicode20: u16 = (4u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:58-58
    pub const UnicodeVariationSequences: u16 = (5u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:59-59
    pub const UnicodeFull: u16 = (6u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:68-102
pub mod MacintoshEncoding {
    // Source: src/sfnt/SkOTTable_name.h:69-69
    pub const Roman: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:70-70
    pub const Japanese: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:71-71
    pub const ChineseTraditional: u16 = (2u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:72-72
    pub const Korean: u16 = (3u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:73-73
    pub const Arabic: u16 = (4u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:74-74
    pub const Hebrew: u16 = (5u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:75-75
    pub const Greek: u16 = (6u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:76-76
    pub const Russian: u16 = (7u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:77-77
    pub const RSymbol: u16 = (8u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:78-78
    pub const Devanagari: u16 = (9u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:79-79
    pub const Gurmukhi: u16 = (10u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:80-80
    pub const Gujarati: u16 = (11u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:81-81
    pub const Oriya: u16 = (12u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:82-82
    pub const Bengali: u16 = (13u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:83-83
    pub const Tamil: u16 = (14u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:84-84
    pub const Telugu: u16 = (15u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:85-85
    pub const Kannada: u16 = (16u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:86-86
    pub const Malayalam: u16 = (17u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:87-87
    pub const Sinhalese: u16 = (18u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:88-88
    pub const Burmese: u16 = (19u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:89-89
    pub const Khmer: u16 = (20u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:90-90
    pub const Thai: u16 = (21u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:91-91
    pub const Laotian: u16 = (22u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:92-92
    pub const Georgian: u16 = (23u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:93-93
    pub const Armenian: u16 = (24u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:94-94
    pub const ChineseSimplified: u16 = (25u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:95-95
    pub const Tibetan: u16 = (26u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:96-96
    pub const Mongolian: u16 = (27u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:97-97
    pub const Geez: u16 = (28u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:98-98
    pub const Slavic: u16 = (29u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:99-99
    pub const Vietnamese: u16 = (30u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:100-100
    pub const Sindhi: u16 = (31u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:101-101
    pub const Uninterpreted: u16 = (32u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:107-111
pub mod ISO {
    // Source: src/sfnt/SkOTTable_name.h:108-108
    pub const ASCII7: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:109-109
    pub const ISO10646: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:110-110
    pub const ISO88591: u16 = (2u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:119-128
pub mod WindowsEncoding {
    // Source: src/sfnt/SkOTTable_name.h:120-120
    pub const Symbol: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:121-121
    pub const UnicodeBMPUCS2: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:122-122
    pub const ShiftJIS: u16 = (2u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:123-123
    pub const PRC: u16 = (3u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:124-124
    pub const Big5: u16 = (4u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:125-125
    pub const Wansung: u16 = (5u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:126-126
    pub const Johab: u16 = (6u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:127-127
    pub const UnicodeUCS4: u16 = (10u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:146-265
pub mod MacintoshLanguage {
    // Source: src/sfnt/SkOTTable_name.h:147-147
    pub const English: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:148-148
    pub const French: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:149-149
    pub const German: u16 = (2u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:150-150
    pub const Italian: u16 = (3u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:151-151
    pub const Dutch: u16 = (4u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:152-152
    pub const Swedish: u16 = (5u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:153-153
    pub const Spanish: u16 = (6u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:154-154
    pub const Danish: u16 = (7u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:155-155
    pub const Portuguese: u16 = (8u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:156-156
    pub const Norwegian: u16 = (9u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:157-157
    pub const Hebrew: u16 = (10u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:158-158
    pub const Japanese: u16 = (11u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:159-159
    pub const Arabic: u16 = (12u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:160-160
    pub const Finnish: u16 = (13u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:161-161
    pub const Greek: u16 = (14u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:162-162
    pub const Icelandic: u16 = (15u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:163-163
    pub const Maltese: u16 = (16u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:164-164
    pub const Turkish: u16 = (17u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:165-165
    pub const Croatian: u16 = (18u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:166-166
    pub const ChineseTraditional: u16 = (19u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:167-167
    pub const Urdu: u16 = (20u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:168-168
    pub const Hindi: u16 = (21u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:169-169
    pub const Thai: u16 = (22u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:170-170
    pub const Korean: u16 = (23u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:171-171
    pub const Lithuanian: u16 = (24u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:172-172
    pub const Polish: u16 = (25u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:173-173
    pub const Hungarian: u16 = (26u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:174-174
    pub const Estonian: u16 = (27u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:175-175
    pub const Latvian: u16 = (28u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:176-176
    pub const Sami: u16 = (29u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:177-177
    pub const Faroese: u16 = (30u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:178-178
    pub const Farsi_Persian: u16 = (31u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:179-179
    pub const Russian: u16 = (32u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:180-180
    pub const ChineseSimplified: u16 = (33u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:181-181
    pub const Flemish: u16 = (34u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:182-182
    pub const IrishGaelic: u16 = (35u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:183-183
    pub const Albanian: u16 = (36u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:184-184
    pub const Romanian: u16 = (37u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:185-185
    pub const Czech: u16 = (38u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:186-186
    pub const Slovak: u16 = (39u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:187-187
    pub const Slovenian: u16 = (40u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:188-188
    pub const Yiddish: u16 = (41u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:189-189
    pub const Serbian: u16 = (42u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:190-190
    pub const Macedonian: u16 = (43u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:191-191
    pub const Bulgarian: u16 = (44u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:192-192
    pub const Ukrainian: u16 = (45u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:193-193
    pub const Byelorussian: u16 = (46u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:194-194
    pub const Uzbek: u16 = (47u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:195-195
    pub const Kazakh: u16 = (48u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:196-196
    pub const AzerbaijaniCyrillic: u16 = (49u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:197-197
    pub const AzerbaijaniArabic: u16 = (50u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:198-198
    pub const Armenian: u16 = (51u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:199-199
    pub const Georgian: u16 = (52u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:200-200
    pub const Moldavian: u16 = (53u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:201-201
    pub const Kirghiz: u16 = (54u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:202-202
    pub const Tajiki: u16 = (55u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:203-203
    pub const Turkmen: u16 = (56u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:204-204
    pub const MongolianTraditional: u16 = (57u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:205-205
    pub const MongolianCyrillic: u16 = (58u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:206-206
    pub const Pashto: u16 = (59u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:207-207
    pub const Kurdish: u16 = (60u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:208-208
    pub const Kashmiri: u16 = (61u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:209-209
    pub const Sindhi: u16 = (62u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:210-210
    pub const Tibetan: u16 = (63u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:211-211
    pub const Nepali: u16 = (64u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:212-212
    pub const Sanskrit: u16 = (65u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:213-213
    pub const Marathi: u16 = (66u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:214-214
    pub const Bengali: u16 = (67u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:215-215
    pub const Assamese: u16 = (68u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:216-216
    pub const Gujarati: u16 = (69u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:217-217
    pub const Punjabi: u16 = (70u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:218-218
    pub const Oriya: u16 = (71u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:219-219
    pub const Malayalam: u16 = (72u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:220-220
    pub const Kannada: u16 = (73u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:221-221
    pub const Tamil: u16 = (74u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:222-222
    pub const Telugu: u16 = (75u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:223-223
    pub const Sinhalese: u16 = (76u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:224-224
    pub const Burmese: u16 = (77u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:225-225
    pub const Khmer: u16 = (78u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:226-226
    pub const Lao: u16 = (79u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:227-227
    pub const Vietnamese: u16 = (80u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:228-228
    pub const Indonesian: u16 = (81u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:229-229
    pub const Tagalong: u16 = (82u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:230-230
    pub const MalayRoman: u16 = (83u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:231-231
    pub const MalayArabic: u16 = (84u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:232-232
    pub const Amharic: u16 = (85u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:233-233
    pub const Tigrinya: u16 = (86u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:234-234
    pub const Galla: u16 = (87u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:235-235
    pub const Somali: u16 = (88u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:236-236
    pub const Swahili: u16 = (89u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:237-237
    pub const Kinyarwanda_Ruanda: u16 = (90u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:238-238
    pub const Rundi: u16 = (91u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:239-239
    pub const Nyanja_Chewa: u16 = (92u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:240-240
    pub const Malagasy: u16 = (93u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:241-241
    pub const Esperanto: u16 = (94u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:242-242
    pub const Welsh: u16 = (128u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:243-243
    pub const Basque: u16 = (129u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:244-244
    pub const Catalan: u16 = (130u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:245-245
    pub const Latin: u16 = (131u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:246-246
    pub const Quenchua: u16 = (132u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:247-247
    pub const Guarani: u16 = (133u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:248-248
    pub const Aymara: u16 = (134u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:249-249
    pub const Tatar: u16 = (135u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:250-250
    pub const Uighur: u16 = (136u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:251-251
    pub const Dzongkha: u16 = (137u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:252-252
    pub const JavaneseRoman: u16 = (138u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:253-253
    pub const SundaneseRoman: u16 = (139u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:254-254
    pub const Galician: u16 = (140u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:255-255
    pub const Afrikaans: u16 = (141u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:256-256
    pub const Breton: u16 = (142u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:257-257
    pub const Inuktitut: u16 = (143u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:258-258
    pub const ScottishGaelic: u16 = (144u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:259-259
    pub const ManxGaelic: u16 = (145u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:260-260
    pub const IrishGaelicWithLenition: u16 = (146u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:261-261
    pub const Tongan: u16 = (147u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:262-262
    pub const GreekPolytonic: u16 = (148u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:263-263
    pub const Greenlandic: u16 = (149u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:264-264
    pub const AzerbaijaniRoman: u16 = (150u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:272-478
pub mod WindowsLanguage {
    // Source: src/sfnt/SkOTTable_name.h:273-273
    pub const Afrikaans_SouthAfrica: u16 = (0x0436u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:274-274
    pub const Albanian_Albania: u16 = (0x041Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:275-275
    pub const Alsatian_France: u16 = (0x0484u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:276-276
    pub const Amharic_Ethiopia: u16 = (0x045Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:277-277
    pub const Arabic_Algeria: u16 = (0x1401u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:278-278
    pub const Arabic_Bahrain: u16 = (0x3C01u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:279-279
    pub const Arabic_Egypt: u16 = (0x0C01u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:280-280
    pub const Arabic_Iraq: u16 = (0x0801u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:281-281
    pub const Arabic_Jordan: u16 = (0x2C01u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:282-282
    pub const Arabic_Kuwait: u16 = (0x3401u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:283-283
    pub const Arabic_Lebanon: u16 = (0x3001u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:284-284
    pub const Arabic_Libya: u16 = (0x1001u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:285-285
    pub const Arabic_Morocco: u16 = (0x1801u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:286-286
    pub const Arabic_Oman: u16 = (0x2001u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:287-287
    pub const Arabic_Qatar: u16 = (0x4001u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:288-288
    pub const Arabic_SaudiArabia: u16 = (0x0401u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:289-289
    pub const Arabic_Syria: u16 = (0x2801u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:290-290
    pub const Arabic_Tunisia: u16 = (0x1C01u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:291-291
    pub const Arabic_UAE: u16 = (0x3801u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:292-292
    pub const Arabic_Yemen: u16 = (0x2401u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:293-293
    pub const Armenian_Armenia: u16 = (0x042Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:294-294
    pub const Assamese_India: u16 = (0x044Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:295-295
    pub const AzeriCyrillic_Azerbaijan: u16 = (0x082Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:296-296
    pub const AzeriLatin_Azerbaijan: u16 = (0x042Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:297-297
    pub const Bashkir_Russia: u16 = (0x046Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:298-298
    pub const Basque_Basque: u16 = (0x042Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:299-299
    pub const Belarusian_Belarus: u16 = (0x0423u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:300-300
    pub const Bengali_Bangladesh: u16 = (0x0845u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:301-301
    pub const Bengali_India: u16 = (0x0445u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:302-302
    pub const BosnianCyrillic_BosniaAndHerzegovina: u16 = (0x201Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:303-303
    pub const BosnianLatin_BosniaAndHerzegovina: u16 = (0x141Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:304-304
    pub const Breton_France: u16 = (0x047Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:305-305
    pub const Bulgarian_Bulgaria: u16 = (0x0402u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:306-306
    pub const Catalan_Catalan: u16 = (0x0403u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:307-307
    pub const Chinese_HongKongSAR: u16 = (0x0C04u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:308-308
    pub const Chinese_MacaoSAR: u16 = (0x1404u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:309-309
    pub const Chinese_PeoplesRepublicOfChina: u16 = (0x0804u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:310-310
    pub const Chinese_Singapore: u16 = (0x1004u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:311-311
    pub const Chinese_Taiwan: u16 = (0x0404u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:312-312
    pub const Corsican_France: u16 = (0x0483u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:313-313
    pub const Croatian_Croatia: u16 = (0x041Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:314-314
    pub const CroatianLatin_BosniaAndHerzegovina: u16 = (0x101Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:315-315
    pub const Czech_CzechRepublic: u16 = (0x0405u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:316-316
    pub const Danish_Denmark: u16 = (0x0406u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:317-317
    pub const Dari_Afghanistan: u16 = (0x048Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:318-318
    pub const Divehi_Maldives: u16 = (0x0465u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:319-319
    pub const Dutch_Belgium: u16 = (0x0813u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:320-320
    pub const Dutch_Netherlands: u16 = (0x0413u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:321-321
    pub const English_Australia: u16 = (0x0C09u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:322-322
    pub const English_Belize: u16 = (0x2809u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:323-323
    pub const English_Canada: u16 = (0x1009u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:324-324
    pub const English_Caribbean: u16 = (0x2409u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:325-325
    pub const English_India: u16 = (0x4009u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:326-326
    pub const English_Ireland: u16 = (0x1809u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:327-327
    pub const English_Jamaica: u16 = (0x2009u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:328-328
    pub const English_Malaysia: u16 = (0x4409u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:329-329
    pub const English_NewZealand: u16 = (0x1409u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:330-330
    pub const English_RepublicOfThePhilippines: u16 = (0x3409u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:331-331
    pub const English_Singapore: u16 = (0x4809u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:332-332
    pub const English_SouthAfrica: u16 = (0x1C09u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:333-333
    pub const English_TrinidadAndTobago: u16 = (0x2C09u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:334-334
    pub const English_UnitedKingdom: u16 = (0x0809u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:335-335
    pub const English_UnitedStates: u16 = (0x0409u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:336-336
    pub const English_Zimbabwe: u16 = (0x3009u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:337-337
    pub const Estonian_Estonia: u16 = (0x0425u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:338-338
    pub const Faroese_FaroeIslands: u16 = (0x0438u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:339-339
    pub const Filipino_Philippines: u16 = (0x0464u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:340-340
    pub const Finnish_Finland: u16 = (0x040Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:341-341
    pub const French_Belgium: u16 = (0x080Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:342-342
    pub const French_Canada: u16 = (0x0C0Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:343-343
    pub const French_France: u16 = (0x040Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:344-344
    pub const French_Luxembourg: u16 = (0x140cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:345-345
    pub const French_PrincipalityOfMonoco: u16 = (0x180Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:346-346
    pub const French_Switzerland: u16 = (0x100Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:347-347
    pub const Frisian_Netherlands: u16 = (0x0462u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:348-348
    pub const Galician_Galician: u16 = (0x0456u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:349-349
    pub const Georgian_Georgia: u16 = (0x0437u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:350-350
    pub const German_Austria: u16 = (0x0C07u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:351-351
    pub const German_Germany: u16 = (0x0407u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:352-352
    pub const German_Liechtenstein: u16 = (0x1407u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:353-353
    pub const German_Luxembourg: u16 = (0x1007u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:354-354
    pub const German_Switzerland: u16 = (0x0807u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:355-355
    pub const Greek_Greece: u16 = (0x0408u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:356-356
    pub const Greenlandic_Greenland: u16 = (0x046Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:357-357
    pub const Gujarati_India: u16 = (0x0447u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:358-358
    pub const HausaLatin_Nigeria: u16 = (0x0468u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:359-359
    pub const Hebrew_Israel: u16 = (0x040Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:360-360
    pub const Hindi_India: u16 = (0x0439u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:361-361
    pub const Hungarian_Hungary: u16 = (0x040Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:362-362
    pub const Icelandic_Iceland: u16 = (0x040Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:363-363
    pub const Igbo_Nigeria: u16 = (0x0470u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:364-364
    pub const Indonesian_Indonesia: u16 = (0x0421u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:365-365
    pub const Inuktitut_Canada: u16 = (0x045Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:366-366
    pub const InuktitutLatin_Canada: u16 = (0x085Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:367-367
    pub const Irish_Ireland: u16 = (0x083Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:368-368
    pub const isiXhosa_SouthAfrica: u16 = (0x0434u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:369-369
    pub const isiZulu_SouthAfrica: u16 = (0x0435u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:370-370
    pub const Italian_Italy: u16 = (0x0410u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:371-371
    pub const Italian_Switzerland: u16 = (0x0810u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:372-372
    pub const Japanese_Japan: u16 = (0x0411u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:373-373
    pub const Kannada_India: u16 = (0x044Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:374-374
    pub const Kazakh_Kazakhstan: u16 = (0x043Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:375-375
    pub const Khmer_Cambodia: u16 = (0x0453u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:376-376
    pub const Kiche_Guatemala: u16 = (0x0486u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:377-377
    pub const Kinyarwanda_Rwanda: u16 = (0x0487u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:378-378
    pub const Kiswahili_Kenya: u16 = (0x0441u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:379-379
    pub const Konkani_India: u16 = (0x0457u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:380-380
    pub const Korean_Korea: u16 = (0x0412u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:381-381
    pub const Kyrgyz_Kyrgyzstan: u16 = (0x0440u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:382-382
    pub const Lao_LaoPDR: u16 = (0x0454u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:383-383
    pub const Latvian_Latvia: u16 = (0x0426u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:384-384
    pub const Lithuanian_Lithuania: u16 = (0x0427u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:385-385
    pub const LowerSorbian_Germany: u16 = (0x082Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:386-386
    pub const Luxembourgish_Luxembourg: u16 = (0x046Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:387-387
    pub const MacedonianFYROM_FormerYugoslavRepublicOfMacedonia: u16 = (0x042Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:388-388
    pub const Malay_BruneiDarussalam: u16 = (0x083Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:389-389
    pub const Malay_Malaysia: u16 = (0x043Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:390-390
    pub const Malayalam_India: u16 = (0x044Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:391-391
    pub const Maltese_Malta: u16 = (0x043Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:392-392
    pub const Maori_NewZealand: u16 = (0x0481u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:393-393
    pub const Mapudungun_Chile: u16 = (0x047Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:394-394
    pub const Marathi_India: u16 = (0x044Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:395-395
    pub const Mohawk_Mohawk: u16 = (0x047Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:396-396
    pub const MongolianCyrillic_Mongolia: u16 = (0x0450u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:397-397
    pub const MongolianTraditional_PeoplesRepublicOfChina: u16 = (0x0850u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:398-398
    pub const Nepali_Nepal: u16 = (0x0461u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:399-399
    pub const NorwegianBokmal_Norway: u16 = (0x0414u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:400-400
    pub const NorwegianNynorsk_Norway: u16 = (0x0814u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:401-401
    pub const Occitan_France: u16 = (0x0482u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:402-402
    pub const Odia_India: u16 = (0x0448u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:403-403
    pub const Pashto_Afghanistan: u16 = (0x0463u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:404-404
    pub const Polish_Poland: u16 = (0x0415u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:405-405
    pub const Portuguese_Brazil: u16 = (0x0416u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:406-406
    pub const Portuguese_Portugal: u16 = (0x0816u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:407-407
    pub const Punjabi_India: u16 = (0x0446u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:408-408
    pub const Quechua_Bolivia: u16 = (0x046Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:409-409
    pub const Quechua_Ecuador: u16 = (0x086Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:410-410
    pub const Quechua_Peru: u16 = (0x0C6Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:411-411
    pub const Romanian_Romania: u16 = (0x0418u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:412-412
    pub const Romansh_Switzerland: u16 = (0x0417u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:413-413
    pub const Russian_Russia: u16 = (0x0419u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:414-414
    pub const SamiInari_Finland: u16 = (0x243Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:415-415
    pub const SamiLule_Norway: u16 = (0x103Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:416-416
    pub const SamiLule_Sweden: u16 = (0x143Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:417-417
    pub const SamiNorthern_Finland: u16 = (0x0C3Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:418-418
    pub const SamiNorthern_Norway: u16 = (0x043Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:419-419
    pub const SamiNorthern_Sweden: u16 = (0x083Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:420-420
    pub const SamiSkolt_Finland: u16 = (0x203Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:421-421
    pub const SamiSouthern_Norway: u16 = (0x183Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:422-422
    pub const SamiSouthern_Sweden: u16 = (0x1C3Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:423-423
    pub const Sanskrit_India: u16 = (0x044Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:424-424
    pub const SerbianCyrillic_BosniaAndHerzegovina: u16 = (0x1C1Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:425-425
    pub const SerbianCyrillic_Serbia: u16 = (0x0C1Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:426-426
    pub const SerbianLatin_BosniaAndHerzegovina: u16 = (0x181Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:427-427
    pub const SerbianLatin_Serbia: u16 = (0x081Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:428-428
    pub const SesothoSaLeboa_SouthAfrica: u16 = (0x046Cu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:429-429
    pub const Setswana_SouthAfrica: u16 = (0x0432u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:430-430
    pub const Sinhala_SriLanka: u16 = (0x045Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:431-431
    pub const Slovak_Slovakia: u16 = (0x041Bu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:432-432
    pub const Slovenian_Slovenia: u16 = (0x0424u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:433-433
    pub const Spanish_Argentina: u16 = (0x2C0Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:434-434
    pub const Spanish_Bolivia: u16 = (0x400Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:435-435
    pub const Spanish_Chile: u16 = (0x340Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:436-436
    pub const Spanish_Colombia: u16 = (0x240Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:437-437
    pub const Spanish_CostaRica: u16 = (0x140Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:438-438
    pub const Spanish_DominicanRepublic: u16 = (0x1C0Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:439-439
    pub const Spanish_Ecuador: u16 = (0x300Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:440-440
    pub const Spanish_ElSalvador: u16 = (0x440Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:441-441
    pub const Spanish_Guatemala: u16 = (0x100Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:442-442
    pub const Spanish_Honduras: u16 = (0x480Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:443-443
    pub const Spanish_Mexico: u16 = (0x080Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:444-444
    pub const Spanish_Nicaragua: u16 = (0x4C0Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:445-445
    pub const Spanish_Panama: u16 = (0x180Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:446-446
    pub const Spanish_Paraguay: u16 = (0x3C0Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:447-447
    pub const Spanish_Peru: u16 = (0x280Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:448-448
    pub const Spanish_PuertoRico: u16 = (0x500Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:449-449
    pub const SpanishModernSort_Spain: u16 = (0x0C0Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:450-450
    pub const SpanishTraditionalSort_Spain: u16 = (0x040Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:451-451
    pub const Spanish_UnitedStates: u16 = (0x540Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:452-452
    pub const Spanish_Uruguay: u16 = (0x380Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:453-453
    pub const Spanish_Venezuela: u16 = (0x200Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:454-454
    pub const Sweden_Finland: u16 = (0x081Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:455-455
    pub const Swedish_Sweden: u16 = (0x041Du16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:456-456
    pub const Syriac_Syria: u16 = (0x045Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:457-457
    pub const TajikCyrillic_Tajikistan: u16 = (0x0428u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:458-458
    pub const TamazightLatin_Algeria: u16 = (0x085Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:459-459
    pub const Tamil_India: u16 = (0x0449u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:460-460
    pub const Tatar_Russia: u16 = (0x0444u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:461-461
    pub const Telugu_India: u16 = (0x044Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:462-462
    pub const Thai_Thailand: u16 = (0x041Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:463-463
    pub const Tibetan_PRC: u16 = (0x0451u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:464-464
    pub const Turkish_Turkey: u16 = (0x041Fu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:465-465
    pub const Turkmen_Turkmenistan: u16 = (0x0442u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:466-466
    pub const Uighur_PRC: u16 = (0x0480u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:467-467
    pub const Ukrainian_Ukraine: u16 = (0x0422u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:468-468
    pub const UpperSorbian_Germany: u16 = (0x042Eu16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:469-469
    pub const Urdu_IslamicRepublicOfPakistan: u16 = (0x0420u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:470-470
    pub const UzbekCyrillic_Uzbekistan: u16 = (0x0843u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:471-471
    pub const UzbekLatin_Uzbekistan: u16 = (0x0443u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:472-472
    pub const Vietnamese_Vietnam: u16 = (0x042Au16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:473-473
    pub const Welsh_UnitedKingdom: u16 = (0x0452u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:474-474
    pub const Wolof_Senegal: u16 = (0x0488u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:475-475
    pub const Yakut_Russia: u16 = (0x0485u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:476-476
    pub const Yi_PRC: u16 = (0x0478u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:477-477
    pub const Yoruba_Nigeria: u16 = (0x046Au16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:487-510
pub mod Predefined {
    // Source: src/sfnt/SkOTTable_name.h:488-488
    pub const CopyrightNotice: u16 = (0u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:489-489
    pub const FontFamilyName: u16 = (1u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:490-490
    pub const FontSubfamilyName: u16 = (2u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:491-491
    pub const UniqueFontIdentifier: u16 = (3u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:492-492
    pub const FullFontName: u16 = (4u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:493-493
    pub const VersionString: u16 = (5u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:494-494
    pub const PostscriptName: u16 = (6u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:495-495
    pub const Trademark: u16 = (7u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:496-496
    pub const ManufacturerName: u16 = (8u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:497-497
    pub const Designer: u16 = (9u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:498-498
    pub const Description: u16 = (10u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:499-499
    pub const URLVendor: u16 = (11u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:500-500
    pub const URLDesigner: u16 = (12u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:501-501
    pub const LicenseDescription: u16 = (13u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:502-502
    pub const LicenseInfoURL: u16 = (14u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:503-503
    pub const PreferredFamily: u16 = (16u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:504-504
    pub const PreferredSubfamily: u16 = (17u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:505-505
    pub const CompatibleFullName: u16 = (18u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:506-506
    pub const SampleText: u16 = (19u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:507-507
    pub const PostscriptCIDFindfontName: u16 = (20u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:508-508
    pub const WWSFamilyName: u16 = (21u16).to_be();
    // Source: src/sfnt/SkOTTable_name.h:509-509
    pub const WWSSubfamilyName: u16 = (22u16).to_be();
}
// Source: src/sfnt/SkOTTable_name.h:36-515
// Native nested wrappers/unions share the same u16 storage. Flattened Rust
// field projection is a language adaptation, not an alternate table format.
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Record {
    pub platformID: u16,
    pub encodingID: u16,
    pub languageID: u16,
    pub nameID: u16,
    pub length: u16,
    pub offset: u16,
}
// Source: src/sfnt/SkOTTable_name.h:517-539
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Format1Ext {
    pub langTagCount: u16,
}
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LangTagRecord {
    pub length: u16,
    pub offset: u16,
}
// Source: src/sfnt/SkOTTable_name.h:544-569
pub struct Iterator {
    fNameTable: *const u8,
    fNameTableSize: usize,
    fIndex: usize,
    fType: i32,
}
#[derive(Default)]
pub struct IteratorRecord {
    pub name: String,
    pub language: String,
    pub r#type: u16,
}
impl Iterator {
    /// The backing table must remain readable and immovable for this iterator's lifetime.
    pub unsafe fn new(nameTable: *const u8, size: usize) -> Self {
        Self {
            fNameTable: nameTable,
            fNameTableSize: size,
            fIndex: 0,
            fType: -1,
        }
    }
    pub unsafe fn with_type(nameTable: *const u8, size: usize, r#type: u16) -> Self {
        Self {
            fNameTable: nameTable,
            fNameTableSize: size,
            fIndex: 0,
            fType: r#type as i32,
        }
    }
    pub fn reset(&mut self, r#type: u16) {
        self.fIndex = 0;
        self.fType = r#type as i32;
    }
}
// Source: src/sfnt/SkOTTable_name.h:573-576
const _: () = assert!(core::mem::size_of::<SkOTTableName>() == 6);
const _: () = assert!(core::mem::size_of::<Format1Ext>() == 2);
const _: () = assert!(core::mem::size_of::<LangTagRecord>() == 4);
const _: () = assert!(core::mem::size_of::<Record>() == 12);
// Source: src/sfnt/SkOTTable_name.cpp:15-49
fn next_unichar_UTF16BE(src: &mut &[u8]) -> u32 {
    debug_assert!(!src.is_empty());
    if src.len() < 2 {
        *src = &[];
        return 0xfffd;
    }
    let mut c = u16::from_be_bytes([src[0], src[1]]) as u32;
    *src = &src[2..];
    if (0xdc00..=0xdfff).contains(&c) {
        return 0xfffd;
    }
    if (0xd800..=0xdbff).contains(&c) {
        if src.len() < 2 {
            *src = &[];
            return 0xfffd;
        }
        let c2 = u16::from_be_bytes([src[0], src[1]]) as u32;
        if !(0xdc00..=0xdfff).contains(&c2) {
            return 0xfffd;
        }
        *src = &src[2..];
        c = (c << 10)
            .wrapping_add(c2)
            .wrapping_add(0x10000u32.wrapping_sub(0xd800 << 10).wrapping_sub(0xdc00));
    }
    c
}
// Source: src/sfnt/SkOTTable_name.cpp:51-59
fn SkString_from_UTF16BE(mut bytes: &[u8], out: &mut String) {
    out.clear();
    while !bytes.is_empty() {
        out.push(char::from_u32(next_unichar_UTF16BE(&mut bytes)).unwrap());
    }
}
// Source: src/sfnt/SkOTTable_name.cpp:66-83
const UnicodeFromMacRoman: [u16; 128] = [
    0x00C4, 0x00C5, 0x00C7, 0x00C9, 0x00D1, 0x00D6, 0x00DC, 0x00E1, 0x00E0, 0x00E2, 0x00E4, 0x00E3,
    0x00E5, 0x00E7, 0x00E9, 0x00E8, 0x00EA, 0x00EB, 0x00ED, 0x00EC, 0x00EE, 0x00EF, 0x00F1, 0x00F3,
    0x00F2, 0x00F4, 0x00F6, 0x00F5, 0x00FA, 0x00F9, 0x00FB, 0x00FC, 0x2020, 0x00B0, 0x00A2, 0x00A3,
    0x00A7, 0x2022, 0x00B6, 0x00DF, 0x00AE, 0x00A9, 0x2122, 0x00B4, 0x00A8, 0x2260, 0x00C6, 0x00D8,
    0x221E, 0x00B1, 0x2264, 0x2265, 0x00A5, 0x00B5, 0x2202, 0x2211, 0x220F, 0x03C0, 0x222B, 0x00AA,
    0x00BA, 0x03A9, 0x00E6, 0x00F8, 0x00BF, 0x00A1, 0x00AC, 0x221A, 0x0192, 0x2248, 0x2206, 0x00AB,
    0x00BB, 0x2026, 0x00A0, 0x00C0, 0x00C3, 0x00D5, 0x0152, 0x0153, 0x2013, 0x2014, 0x201C, 0x201D,
    0x2018, 0x2019, 0x00F7, 0x25CA, 0x00FF, 0x0178, 0x2044, 0x20AC, 0x2039, 0x203A, 0xFB01, 0xFB02,
    0x2021, 0x00B7, 0x201A, 0x201E, 0x2030, 0x00C2, 0x00CA, 0x00C1, 0x00CB, 0x00C8, 0x00CD, 0x00CE,
    0x00CF, 0x00CC, 0x00D3, 0x00D4, 0xF8FF, 0x00D2, 0x00DA, 0x00DB, 0x00D9, 0x0131, 0x02C6, 0x02DC,
    0x00AF, 0x02D8, 0x02D9, 0x02DA, 0x00B8, 0x02DD, 0x02DB, 0x02C7,
];
// Source: src/sfnt/SkOTTable_name.cpp:88-95
fn SkStringFromMacRoman(bytes: &[u8], out: &mut String) {
    out.clear();
    for &b in bytes {
        out.push(
            char::from_u32(if b < 0x80 {
                b as u32
            } else {
                UnicodeFromMacRoman[(b - 0x80) as usize] as u32
            })
            .unwrap(),
        );
    }
}
// Source: src/sfnt/SkOTTable_name.cpp:97-449
const BCP47FromLanguageID: &[(u16, &str)] = &[
    // Source: src/sfnt/SkOTTable_name.cpp:103-103
    (0, "en"),
    // Source: src/sfnt/SkOTTable_name.cpp:104-104
    (1, "fr"),
    // Source: src/sfnt/SkOTTable_name.cpp:105-105
    (2, "de"),
    // Source: src/sfnt/SkOTTable_name.cpp:106-106
    (3, "it"),
    // Source: src/sfnt/SkOTTable_name.cpp:107-107
    (4, "nl"),
    // Source: src/sfnt/SkOTTable_name.cpp:108-108
    (5, "sv"),
    // Source: src/sfnt/SkOTTable_name.cpp:109-109
    (6, "es"),
    // Source: src/sfnt/SkOTTable_name.cpp:110-110
    (7, "da"),
    // Source: src/sfnt/SkOTTable_name.cpp:111-111
    (8, "pt"),
    // Source: src/sfnt/SkOTTable_name.cpp:112-112
    (9, "nb"),
    // Source: src/sfnt/SkOTTable_name.cpp:113-113
    (10, "he"),
    // Source: src/sfnt/SkOTTable_name.cpp:114-114
    (11, "ja"),
    // Source: src/sfnt/SkOTTable_name.cpp:115-115
    (12, "ar"),
    // Source: src/sfnt/SkOTTable_name.cpp:116-116
    (13, "fi"),
    // Source: src/sfnt/SkOTTable_name.cpp:117-117
    (14, "el"),
    // Source: src/sfnt/SkOTTable_name.cpp:118-118
    (15, "is"),
    // Source: src/sfnt/SkOTTable_name.cpp:119-119
    (16, "mt"),
    // Source: src/sfnt/SkOTTable_name.cpp:120-120
    (17, "tr"),
    // Source: src/sfnt/SkOTTable_name.cpp:121-121
    (18, "hr"),
    // Source: src/sfnt/SkOTTable_name.cpp:122-122
    (19, "zh-Hant"),
    // Source: src/sfnt/SkOTTable_name.cpp:123-123
    (20, "ur"),
    // Source: src/sfnt/SkOTTable_name.cpp:124-124
    (21, "hi"),
    // Source: src/sfnt/SkOTTable_name.cpp:125-125
    (22, "th"),
    // Source: src/sfnt/SkOTTable_name.cpp:126-126
    (23, "ko"),
    // Source: src/sfnt/SkOTTable_name.cpp:127-127
    (24, "lt"),
    // Source: src/sfnt/SkOTTable_name.cpp:128-128
    (25, "pl"),
    // Source: src/sfnt/SkOTTable_name.cpp:129-129
    (26, "hu"),
    // Source: src/sfnt/SkOTTable_name.cpp:130-130
    (27, "et"),
    // Source: src/sfnt/SkOTTable_name.cpp:131-131
    (28, "lv"),
    // Source: src/sfnt/SkOTTable_name.cpp:132-132
    (29, "se"),
    // Source: src/sfnt/SkOTTable_name.cpp:133-133
    (30, "fo"),
    // Source: src/sfnt/SkOTTable_name.cpp:134-134
    (31, "fa"),
    // Source: src/sfnt/SkOTTable_name.cpp:135-135
    (32, "ru"),
    // Source: src/sfnt/SkOTTable_name.cpp:136-136
    (33, "zh-Hans"),
    // Source: src/sfnt/SkOTTable_name.cpp:137-137
    (34, "nl"),
    // Source: src/sfnt/SkOTTable_name.cpp:138-138
    (35, "ga"),
    // Source: src/sfnt/SkOTTable_name.cpp:139-139
    (36, "sq"),
    // Source: src/sfnt/SkOTTable_name.cpp:140-140
    (37, "ro"),
    // Source: src/sfnt/SkOTTable_name.cpp:141-141
    (38, "cs"),
    // Source: src/sfnt/SkOTTable_name.cpp:142-142
    (39, "sk"),
    // Source: src/sfnt/SkOTTable_name.cpp:143-143
    (40, "sl"),
    // Source: src/sfnt/SkOTTable_name.cpp:144-144
    (41, "yi"),
    // Source: src/sfnt/SkOTTable_name.cpp:145-145
    (42, "sr"),
    // Source: src/sfnt/SkOTTable_name.cpp:146-146
    (43, "mk"),
    // Source: src/sfnt/SkOTTable_name.cpp:147-147
    (44, "bg"),
    // Source: src/sfnt/SkOTTable_name.cpp:148-148
    (45, "uk"),
    // Source: src/sfnt/SkOTTable_name.cpp:149-149
    (46, "be"),
    // Source: src/sfnt/SkOTTable_name.cpp:150-150
    (47, "uz"),
    // Source: src/sfnt/SkOTTable_name.cpp:151-151
    (48, "kk"),
    // Source: src/sfnt/SkOTTable_name.cpp:152-152
    (49, "az-Cyrl"),
    // Source: src/sfnt/SkOTTable_name.cpp:153-153
    (50, "az-Arab"),
    // Source: src/sfnt/SkOTTable_name.cpp:154-154
    (51, "hy"),
    // Source: src/sfnt/SkOTTable_name.cpp:155-155
    (52, "ka"),
    // Source: src/sfnt/SkOTTable_name.cpp:156-156
    (53, "mo"),
    // Source: src/sfnt/SkOTTable_name.cpp:157-157
    (54, "ky"),
    // Source: src/sfnt/SkOTTable_name.cpp:158-158
    (55, "tg"),
    // Source: src/sfnt/SkOTTable_name.cpp:159-159
    (56, "tk"),
    // Source: src/sfnt/SkOTTable_name.cpp:160-160
    (57, "mn-Mong"),
    // Source: src/sfnt/SkOTTable_name.cpp:161-161
    (58, "mn-Cyrl"),
    // Source: src/sfnt/SkOTTable_name.cpp:162-162
    (59, "ps"),
    // Source: src/sfnt/SkOTTable_name.cpp:163-163
    (60, "ku"),
    // Source: src/sfnt/SkOTTable_name.cpp:164-164
    (61, "ks"),
    // Source: src/sfnt/SkOTTable_name.cpp:165-165
    (62, "sd"),
    // Source: src/sfnt/SkOTTable_name.cpp:166-166
    (63, "bo"),
    // Source: src/sfnt/SkOTTable_name.cpp:167-167
    (64, "ne"),
    // Source: src/sfnt/SkOTTable_name.cpp:168-168
    (65, "sa"),
    // Source: src/sfnt/SkOTTable_name.cpp:169-169
    (66, "mr"),
    // Source: src/sfnt/SkOTTable_name.cpp:170-170
    (67, "bn"),
    // Source: src/sfnt/SkOTTable_name.cpp:171-171
    (68, "as"),
    // Source: src/sfnt/SkOTTable_name.cpp:172-172
    (69, "gu"),
    // Source: src/sfnt/SkOTTable_name.cpp:173-173
    (70, "pa"),
    // Source: src/sfnt/SkOTTable_name.cpp:174-174
    (71, "or"),
    // Source: src/sfnt/SkOTTable_name.cpp:175-175
    (72, "ml"),
    // Source: src/sfnt/SkOTTable_name.cpp:176-176
    (73, "kn"),
    // Source: src/sfnt/SkOTTable_name.cpp:177-177
    (74, "ta"),
    // Source: src/sfnt/SkOTTable_name.cpp:178-178
    (75, "te"),
    // Source: src/sfnt/SkOTTable_name.cpp:179-179
    (76, "si"),
    // Source: src/sfnt/SkOTTable_name.cpp:180-180
    (77, "my"),
    // Source: src/sfnt/SkOTTable_name.cpp:181-181
    (78, "km"),
    // Source: src/sfnt/SkOTTable_name.cpp:182-182
    (79, "lo"),
    // Source: src/sfnt/SkOTTable_name.cpp:183-183
    (80, "vi"),
    // Source: src/sfnt/SkOTTable_name.cpp:184-184
    (81, "id"),
    // Source: src/sfnt/SkOTTable_name.cpp:185-185
    (82, "tl"),
    // Source: src/sfnt/SkOTTable_name.cpp:186-186
    (83, "ms-Latn"),
    // Source: src/sfnt/SkOTTable_name.cpp:187-187
    (84, "ms-Arab"),
    // Source: src/sfnt/SkOTTable_name.cpp:188-188
    (85, "am"),
    // Source: src/sfnt/SkOTTable_name.cpp:189-189
    (86, "ti"),
    // Source: src/sfnt/SkOTTable_name.cpp:190-190
    (87, "om"),
    // Source: src/sfnt/SkOTTable_name.cpp:191-191
    (88, "so"),
    // Source: src/sfnt/SkOTTable_name.cpp:192-192
    (89, "sw"),
    // Source: src/sfnt/SkOTTable_name.cpp:193-193
    (90, "rw"),
    // Source: src/sfnt/SkOTTable_name.cpp:194-194
    (91, "rn"),
    // Source: src/sfnt/SkOTTable_name.cpp:195-195
    (92, "ny"),
    // Source: src/sfnt/SkOTTable_name.cpp:196-196
    (93, "mg"),
    // Source: src/sfnt/SkOTTable_name.cpp:197-197
    (94, "eo"),
    // Source: src/sfnt/SkOTTable_name.cpp:198-198
    (128, "cy"),
    // Source: src/sfnt/SkOTTable_name.cpp:199-199
    (129, "eu"),
    // Source: src/sfnt/SkOTTable_name.cpp:200-200
    (130, "ca"),
    // Source: src/sfnt/SkOTTable_name.cpp:201-201
    (131, "la"),
    // Source: src/sfnt/SkOTTable_name.cpp:202-202
    (132, "qu"),
    // Source: src/sfnt/SkOTTable_name.cpp:203-203
    (133, "gn"),
    // Source: src/sfnt/SkOTTable_name.cpp:204-204
    (134, "ay"),
    // Source: src/sfnt/SkOTTable_name.cpp:205-205
    (135, "tt"),
    // Source: src/sfnt/SkOTTable_name.cpp:206-206
    (136, "ug"),
    // Source: src/sfnt/SkOTTable_name.cpp:207-207
    (137, "dz"),
    // Source: src/sfnt/SkOTTable_name.cpp:208-208
    (138, "jv-Latn"),
    // Source: src/sfnt/SkOTTable_name.cpp:209-209
    (139, "su-Latn"),
    // Source: src/sfnt/SkOTTable_name.cpp:210-210
    (140, "gl"),
    // Source: src/sfnt/SkOTTable_name.cpp:211-211
    (141, "af"),
    // Source: src/sfnt/SkOTTable_name.cpp:212-212
    (142, "br"),
    // Source: src/sfnt/SkOTTable_name.cpp:213-213
    (143, "iu"),
    // Source: src/sfnt/SkOTTable_name.cpp:214-214
    (144, "gd"),
    // Source: src/sfnt/SkOTTable_name.cpp:215-215
    (145, "gv"),
    // Source: src/sfnt/SkOTTable_name.cpp:216-216
    (146, "ga"),
    // Source: src/sfnt/SkOTTable_name.cpp:217-217
    (147, "to"),
    // Source: src/sfnt/SkOTTable_name.cpp:218-218
    (148, "el"),
    // Source: src/sfnt/SkOTTable_name.cpp:219-219
    (149, "kl"),
    // Source: src/sfnt/SkOTTable_name.cpp:220-220
    (150, "az-Latn"),
    // Source: src/sfnt/SkOTTable_name.cpp:221-221
    (151, "nn"),
    // Source: src/sfnt/SkOTTable_name.cpp:229-229
    (0x0401, "ar-SA"),
    // Source: src/sfnt/SkOTTable_name.cpp:230-230
    (0x0402, "bg-BG"),
    // Source: src/sfnt/SkOTTable_name.cpp:231-231
    (0x0403, "ca-ES"),
    // Source: src/sfnt/SkOTTable_name.cpp:232-232
    (0x0404, "zh-TW"),
    // Source: src/sfnt/SkOTTable_name.cpp:233-233
    (0x0405, "cs-CZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:234-234
    (0x0406, "da-DK"),
    // Source: src/sfnt/SkOTTable_name.cpp:235-235
    (0x0407, "de-DE"),
    // Source: src/sfnt/SkOTTable_name.cpp:236-236
    (0x0408, "el-GR"),
    // Source: src/sfnt/SkOTTable_name.cpp:237-237
    (0x0409, "en-US"),
    // Source: src/sfnt/SkOTTable_name.cpp:238-238
    (0x040a, "es-ES_tradnl"),
    // Source: src/sfnt/SkOTTable_name.cpp:239-239
    (0x040b, "fi-FI"),
    // Source: src/sfnt/SkOTTable_name.cpp:240-240
    (0x040c, "fr-FR"),
    // Source: src/sfnt/SkOTTable_name.cpp:241-241
    (0x040d, "he-IL"),
    // Source: src/sfnt/SkOTTable_name.cpp:242-242
    (0x040d, "he"),
    // Source: src/sfnt/SkOTTable_name.cpp:243-243
    (0x040e, "hu-HU"),
    // Source: src/sfnt/SkOTTable_name.cpp:244-244
    (0x040e, "hu"),
    // Source: src/sfnt/SkOTTable_name.cpp:245-245
    (0x040f, "is-IS"),
    // Source: src/sfnt/SkOTTable_name.cpp:246-246
    (0x0410, "it-IT"),
    // Source: src/sfnt/SkOTTable_name.cpp:247-247
    (0x0411, "ja-JP"),
    // Source: src/sfnt/SkOTTable_name.cpp:248-248
    (0x0412, "ko-KR"),
    // Source: src/sfnt/SkOTTable_name.cpp:249-249
    (0x0413, "nl-NL"),
    // Source: src/sfnt/SkOTTable_name.cpp:250-250
    (0x0414, "nb-NO"),
    // Source: src/sfnt/SkOTTable_name.cpp:251-251
    (0x0415, "pl-PL"),
    // Source: src/sfnt/SkOTTable_name.cpp:252-252
    (0x0416, "pt-BR"),
    // Source: src/sfnt/SkOTTable_name.cpp:253-253
    (0x0417, "rm-CH"),
    // Source: src/sfnt/SkOTTable_name.cpp:254-254
    (0x0418, "ro-RO"),
    // Source: src/sfnt/SkOTTable_name.cpp:255-255
    (0x0419, "ru-RU"),
    // Source: src/sfnt/SkOTTable_name.cpp:256-256
    (0x041a, "hr-HR"),
    // Source: src/sfnt/SkOTTable_name.cpp:257-257
    (0x041b, "sk-SK"),
    // Source: src/sfnt/SkOTTable_name.cpp:258-258
    (0x041c, "sq-AL"),
    // Source: src/sfnt/SkOTTable_name.cpp:259-259
    (0x041d, "sv-SE"),
    // Source: src/sfnt/SkOTTable_name.cpp:260-260
    (0x041e, "th-TH"),
    // Source: src/sfnt/SkOTTable_name.cpp:261-261
    (0x041f, "tr-TR"),
    // Source: src/sfnt/SkOTTable_name.cpp:262-262
    (0x0420, "ur-PK"),
    // Source: src/sfnt/SkOTTable_name.cpp:263-263
    (0x0421, "id-ID"),
    // Source: src/sfnt/SkOTTable_name.cpp:264-264
    (0x0422, "uk-UA"),
    // Source: src/sfnt/SkOTTable_name.cpp:265-265
    (0x0423, "be-BY"),
    // Source: src/sfnt/SkOTTable_name.cpp:266-266
    (0x0424, "sl-SI"),
    // Source: src/sfnt/SkOTTable_name.cpp:267-267
    (0x0425, "et-EE"),
    // Source: src/sfnt/SkOTTable_name.cpp:268-268
    (0x0426, "lv-LV"),
    // Source: src/sfnt/SkOTTable_name.cpp:269-269
    (0x0427, "lt-LT"),
    // Source: src/sfnt/SkOTTable_name.cpp:270-270
    (0x0428, "tg-Cyrl-TJ"),
    // Source: src/sfnt/SkOTTable_name.cpp:271-271
    (0x0429, "fa-IR"),
    // Source: src/sfnt/SkOTTable_name.cpp:272-272
    (0x042a, "vi-VN"),
    // Source: src/sfnt/SkOTTable_name.cpp:273-273
    (0x042b, "hy-AM"),
    // Source: src/sfnt/SkOTTable_name.cpp:274-274
    (0x042c, "az-Latn-AZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:275-275
    (0x042d, "eu-ES"),
    // Source: src/sfnt/SkOTTable_name.cpp:276-276
    (0x042e, "hsb-DE"),
    // Source: src/sfnt/SkOTTable_name.cpp:277-277
    (0x042f, "mk-MK"),
    // Source: src/sfnt/SkOTTable_name.cpp:278-278
    (0x0432, "tn-ZA"),
    // Source: src/sfnt/SkOTTable_name.cpp:279-279
    (0x0434, "xh-ZA"),
    // Source: src/sfnt/SkOTTable_name.cpp:280-280
    (0x0435, "zu-ZA"),
    // Source: src/sfnt/SkOTTable_name.cpp:281-281
    (0x0436, "af-ZA"),
    // Source: src/sfnt/SkOTTable_name.cpp:282-282
    (0x0437, "ka-GE"),
    // Source: src/sfnt/SkOTTable_name.cpp:283-283
    (0x0438, "fo-FO"),
    // Source: src/sfnt/SkOTTable_name.cpp:284-284
    (0x0439, "hi-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:285-285
    (0x043a, "mt-MT"),
    // Source: src/sfnt/SkOTTable_name.cpp:286-286
    (0x043b, "se-NO"),
    // Source: src/sfnt/SkOTTable_name.cpp:287-287
    (0x043e, "ms-MY"),
    // Source: src/sfnt/SkOTTable_name.cpp:288-288
    (0x043f, "kk-KZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:289-289
    (0x0440, "ky-KG"),
    // Source: src/sfnt/SkOTTable_name.cpp:290-290
    (0x0441, "sw-KE"),
    // Source: src/sfnt/SkOTTable_name.cpp:291-291
    (0x0442, "tk-TM"),
    // Source: src/sfnt/SkOTTable_name.cpp:292-292
    (0x0443, "uz-Latn-UZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:293-293
    (0x0443, "uz"),
    // Source: src/sfnt/SkOTTable_name.cpp:294-294
    (0x0444, "tt-RU"),
    // Source: src/sfnt/SkOTTable_name.cpp:295-295
    (0x0445, "bn-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:296-296
    (0x0446, "pa-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:297-297
    (0x0447, "gu-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:298-298
    (0x0448, "or-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:299-299
    (0x0449, "ta-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:300-300
    (0x044a, "te-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:301-301
    (0x044b, "kn-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:302-302
    (0x044c, "ml-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:303-303
    (0x044d, "as-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:304-304
    (0x044e, "mr-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:305-305
    (0x044f, "sa-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:306-306
    (0x0450, "mn-Cyrl"),
    // Source: src/sfnt/SkOTTable_name.cpp:307-307
    (0x0451, "bo-CN"),
    // Source: src/sfnt/SkOTTable_name.cpp:308-308
    (0x0452, "cy-GB"),
    // Source: src/sfnt/SkOTTable_name.cpp:309-309
    (0x0453, "km-KH"),
    // Source: src/sfnt/SkOTTable_name.cpp:310-310
    (0x0454, "lo-LA"),
    // Source: src/sfnt/SkOTTable_name.cpp:311-311
    (0x0456, "gl-ES"),
    // Source: src/sfnt/SkOTTable_name.cpp:312-312
    (0x0457, "kok-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:313-313
    (0x045a, "syr-SY"),
    // Source: src/sfnt/SkOTTable_name.cpp:314-314
    (0x045b, "si-LK"),
    // Source: src/sfnt/SkOTTable_name.cpp:315-315
    (0x045d, "iu-Cans-CA"),
    // Source: src/sfnt/SkOTTable_name.cpp:316-316
    (0x045e, "am-ET"),
    // Source: src/sfnt/SkOTTable_name.cpp:317-317
    (0x0461, "ne-NP"),
    // Source: src/sfnt/SkOTTable_name.cpp:318-318
    (0x0462, "fy-NL"),
    // Source: src/sfnt/SkOTTable_name.cpp:319-319
    (0x0463, "ps-AF"),
    // Source: src/sfnt/SkOTTable_name.cpp:320-320
    (0x0464, "fil-PH"),
    // Source: src/sfnt/SkOTTable_name.cpp:321-321
    (0x0465, "dv-MV"),
    // Source: src/sfnt/SkOTTable_name.cpp:322-322
    (0x0468, "ha-Latn-NG"),
    // Source: src/sfnt/SkOTTable_name.cpp:323-323
    (0x046a, "yo-NG"),
    // Source: src/sfnt/SkOTTable_name.cpp:324-324
    (0x046b, "quz-BO"),
    // Source: src/sfnt/SkOTTable_name.cpp:325-325
    (0x046c, "nso-ZA"),
    // Source: src/sfnt/SkOTTable_name.cpp:326-326
    (0x046d, "ba-RU"),
    // Source: src/sfnt/SkOTTable_name.cpp:327-327
    (0x046e, "lb-LU"),
    // Source: src/sfnt/SkOTTable_name.cpp:328-328
    (0x046f, "kl-GL"),
    // Source: src/sfnt/SkOTTable_name.cpp:329-329
    (0x0470, "ig-NG"),
    // Source: src/sfnt/SkOTTable_name.cpp:330-330
    (0x0478, "ii-CN"),
    // Source: src/sfnt/SkOTTable_name.cpp:331-331
    (0x047a, "arn-CL"),
    // Source: src/sfnt/SkOTTable_name.cpp:332-332
    (0x047c, "moh-CA"),
    // Source: src/sfnt/SkOTTable_name.cpp:333-333
    (0x047e, "br-FR"),
    // Source: src/sfnt/SkOTTable_name.cpp:334-334
    (0x0480, "ug-CN"),
    // Source: src/sfnt/SkOTTable_name.cpp:335-335
    (0x0481, "mi-NZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:336-336
    (0x0482, "oc-FR"),
    // Source: src/sfnt/SkOTTable_name.cpp:337-337
    (0x0483, "co-FR"),
    // Source: src/sfnt/SkOTTable_name.cpp:338-338
    (0x0484, "gsw-FR"),
    // Source: src/sfnt/SkOTTable_name.cpp:339-339
    (0x0485, "sah-RU"),
    // Source: src/sfnt/SkOTTable_name.cpp:340-340
    (0x0486, "qut-GT"),
    // Source: src/sfnt/SkOTTable_name.cpp:341-341
    (0x0487, "rw-RW"),
    // Source: src/sfnt/SkOTTable_name.cpp:342-342
    (0x0488, "wo-SN"),
    // Source: src/sfnt/SkOTTable_name.cpp:343-343
    (0x048c, "prs-AF"),
    // Source: src/sfnt/SkOTTable_name.cpp:344-344
    (0x0491, "gd-GB"),
    // Source: src/sfnt/SkOTTable_name.cpp:345-345
    (0x0801, "ar-IQ"),
    // Source: src/sfnt/SkOTTable_name.cpp:346-346
    (0x0804, "zh-Hans"),
    // Source: src/sfnt/SkOTTable_name.cpp:347-347
    (0x0807, "de-CH"),
    // Source: src/sfnt/SkOTTable_name.cpp:348-348
    (0x0809, "en-GB"),
    // Source: src/sfnt/SkOTTable_name.cpp:349-349
    (0x080a, "es-MX"),
    // Source: src/sfnt/SkOTTable_name.cpp:350-350
    (0x080c, "fr-BE"),
    // Source: src/sfnt/SkOTTable_name.cpp:351-351
    (0x0810, "it-CH"),
    // Source: src/sfnt/SkOTTable_name.cpp:352-352
    (0x0813, "nl-BE"),
    // Source: src/sfnt/SkOTTable_name.cpp:353-353
    (0x0814, "nn-NO"),
    // Source: src/sfnt/SkOTTable_name.cpp:354-354
    (0x0816, "pt-PT"),
    // Source: src/sfnt/SkOTTable_name.cpp:355-355
    (0x081a, "sr-Latn-CS"),
    // Source: src/sfnt/SkOTTable_name.cpp:356-356
    (0x081d, "sv-FI"),
    // Source: src/sfnt/SkOTTable_name.cpp:357-357
    (0x082c, "az-Cyrl-AZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:358-358
    (0x082e, "dsb-DE"),
    // Source: src/sfnt/SkOTTable_name.cpp:359-359
    (0x082e, "dsb"),
    // Source: src/sfnt/SkOTTable_name.cpp:360-360
    (0x083b, "se-SE"),
    // Source: src/sfnt/SkOTTable_name.cpp:361-361
    (0x083c, "ga-IE"),
    // Source: src/sfnt/SkOTTable_name.cpp:362-362
    (0x083e, "ms-BN"),
    // Source: src/sfnt/SkOTTable_name.cpp:363-363
    (0x0843, "uz-Cyrl-UZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:364-364
    (0x0845, "bn-BD"),
    // Source: src/sfnt/SkOTTable_name.cpp:365-365
    (0x0850, "mn-Mong-CN"),
    // Source: src/sfnt/SkOTTable_name.cpp:366-366
    (0x085d, "iu-Latn-CA"),
    // Source: src/sfnt/SkOTTable_name.cpp:367-367
    (0x085f, "tzm-Latn-DZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:368-368
    (0x086b, "quz-EC"),
    // Source: src/sfnt/SkOTTable_name.cpp:369-369
    (0x0c01, "ar-EG"),
    // Source: src/sfnt/SkOTTable_name.cpp:370-370
    (0x0c04, "zh-Hant"),
    // Source: src/sfnt/SkOTTable_name.cpp:371-371
    (0x0c07, "de-AT"),
    // Source: src/sfnt/SkOTTable_name.cpp:372-372
    (0x0c09, "en-AU"),
    // Source: src/sfnt/SkOTTable_name.cpp:373-373
    (0x0c0a, "es-ES"),
    // Source: src/sfnt/SkOTTable_name.cpp:374-374
    (0x0c0c, "fr-CA"),
    // Source: src/sfnt/SkOTTable_name.cpp:375-375
    (0x0c1a, "sr-Cyrl-CS"),
    // Source: src/sfnt/SkOTTable_name.cpp:376-376
    (0x0c3b, "se-FI"),
    // Source: src/sfnt/SkOTTable_name.cpp:377-377
    (0x0c6b, "quz-PE"),
    // Source: src/sfnt/SkOTTable_name.cpp:378-378
    (0x1001, "ar-LY"),
    // Source: src/sfnt/SkOTTable_name.cpp:379-379
    (0x1004, "zh-SG"),
    // Source: src/sfnt/SkOTTable_name.cpp:380-380
    (0x1007, "de-LU"),
    // Source: src/sfnt/SkOTTable_name.cpp:381-381
    (0x1009, "en-CA"),
    // Source: src/sfnt/SkOTTable_name.cpp:382-382
    (0x100a, "es-GT"),
    // Source: src/sfnt/SkOTTable_name.cpp:383-383
    (0x100c, "fr-CH"),
    // Source: src/sfnt/SkOTTable_name.cpp:384-384
    (0x101a, "hr-BA"),
    // Source: src/sfnt/SkOTTable_name.cpp:385-385
    (0x103b, "smj-NO"),
    // Source: src/sfnt/SkOTTable_name.cpp:386-386
    (0x1401, "ar-DZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:387-387
    (0x1404, "zh-MO"),
    // Source: src/sfnt/SkOTTable_name.cpp:388-388
    (0x1407, "de-LI"),
    // Source: src/sfnt/SkOTTable_name.cpp:389-389
    (0x1409, "en-NZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:390-390
    (0x140a, "es-CR"),
    // Source: src/sfnt/SkOTTable_name.cpp:391-391
    (0x140c, "fr-LU"),
    // Source: src/sfnt/SkOTTable_name.cpp:392-392
    (0x141a, "bs-Latn-BA"),
    // Source: src/sfnt/SkOTTable_name.cpp:393-393
    (0x141a, "bs"),
    // Source: src/sfnt/SkOTTable_name.cpp:394-394
    (0x143b, "smj-SE"),
    // Source: src/sfnt/SkOTTable_name.cpp:395-395
    (0x143b, "smj"),
    // Source: src/sfnt/SkOTTable_name.cpp:396-396
    (0x1801, "ar-MA"),
    // Source: src/sfnt/SkOTTable_name.cpp:397-397
    (0x1809, "en-IE"),
    // Source: src/sfnt/SkOTTable_name.cpp:398-398
    (0x180a, "es-PA"),
    // Source: src/sfnt/SkOTTable_name.cpp:399-399
    (0x180c, "fr-MC"),
    // Source: src/sfnt/SkOTTable_name.cpp:400-400
    (0x181a, "sr-Latn-BA"),
    // Source: src/sfnt/SkOTTable_name.cpp:401-401
    (0x183b, "sma-NO"),
    // Source: src/sfnt/SkOTTable_name.cpp:402-402
    (0x1c01, "ar-TN"),
    // Source: src/sfnt/SkOTTable_name.cpp:403-403
    (0x1c09, "en-ZA"),
    // Source: src/sfnt/SkOTTable_name.cpp:404-404
    (0x1c0a, "es-DO"),
    // Source: src/sfnt/SkOTTable_name.cpp:405-405
    (0x1c1a, "sr-Cyrl-BA"),
    // Source: src/sfnt/SkOTTable_name.cpp:406-406
    (0x1c3b, "sma-SE"),
    // Source: src/sfnt/SkOTTable_name.cpp:407-407
    (0x1c3b, "sma"),
    // Source: src/sfnt/SkOTTable_name.cpp:408-408
    (0x2001, "ar-OM"),
    // Source: src/sfnt/SkOTTable_name.cpp:409-409
    (0x2009, "en-JM"),
    // Source: src/sfnt/SkOTTable_name.cpp:410-410
    (0x200a, "es-VE"),
    // Source: src/sfnt/SkOTTable_name.cpp:411-411
    (0x201a, "bs-Cyrl-BA"),
    // Source: src/sfnt/SkOTTable_name.cpp:412-412
    (0x201a, "bs-Cyrl"),
    // Source: src/sfnt/SkOTTable_name.cpp:413-413
    (0x203b, "sms-FI"),
    // Source: src/sfnt/SkOTTable_name.cpp:414-414
    (0x203b, "sms"),
    // Source: src/sfnt/SkOTTable_name.cpp:415-415
    (0x2401, "ar-YE"),
    // Source: src/sfnt/SkOTTable_name.cpp:416-416
    (0x2409, "en-029"),
    // Source: src/sfnt/SkOTTable_name.cpp:417-417
    (0x240a, "es-CO"),
    // Source: src/sfnt/SkOTTable_name.cpp:418-418
    (0x241a, "sr-Latn-RS"),
    // Source: src/sfnt/SkOTTable_name.cpp:419-419
    (0x243b, "smn-FI"),
    // Source: src/sfnt/SkOTTable_name.cpp:420-420
    (0x2801, "ar-SY"),
    // Source: src/sfnt/SkOTTable_name.cpp:421-421
    (0x2809, "en-BZ"),
    // Source: src/sfnt/SkOTTable_name.cpp:422-422
    (0x280a, "es-PE"),
    // Source: src/sfnt/SkOTTable_name.cpp:423-423
    (0x281a, "sr-Cyrl-RS"),
    // Source: src/sfnt/SkOTTable_name.cpp:424-424
    (0x2c01, "ar-JO"),
    // Source: src/sfnt/SkOTTable_name.cpp:425-425
    (0x2c09, "en-TT"),
    // Source: src/sfnt/SkOTTable_name.cpp:426-426
    (0x2c0a, "es-AR"),
    // Source: src/sfnt/SkOTTable_name.cpp:427-427
    (0x2c1a, "sr-Latn-ME"),
    // Source: src/sfnt/SkOTTable_name.cpp:428-428
    (0x3001, "ar-LB"),
    // Source: src/sfnt/SkOTTable_name.cpp:429-429
    (0x3009, "en-ZW"),
    // Source: src/sfnt/SkOTTable_name.cpp:430-430
    (0x300a, "es-EC"),
    // Source: src/sfnt/SkOTTable_name.cpp:431-431
    (0x301a, "sr-Cyrl-ME"),
    // Source: src/sfnt/SkOTTable_name.cpp:432-432
    (0x3401, "ar-KW"),
    // Source: src/sfnt/SkOTTable_name.cpp:433-433
    (0x3409, "en-PH"),
    // Source: src/sfnt/SkOTTable_name.cpp:434-434
    (0x340a, "es-CL"),
    // Source: src/sfnt/SkOTTable_name.cpp:435-435
    (0x3801, "ar-AE"),
    // Source: src/sfnt/SkOTTable_name.cpp:436-436
    (0x380a, "es-UY"),
    // Source: src/sfnt/SkOTTable_name.cpp:437-437
    (0x3c01, "ar-BH"),
    // Source: src/sfnt/SkOTTable_name.cpp:438-438
    (0x3c0a, "es-PY"),
    // Source: src/sfnt/SkOTTable_name.cpp:439-439
    (0x4001, "ar-QA"),
    // Source: src/sfnt/SkOTTable_name.cpp:440-440
    (0x4009, "en-IN"),
    // Source: src/sfnt/SkOTTable_name.cpp:441-441
    (0x400a, "es-BO"),
    // Source: src/sfnt/SkOTTable_name.cpp:442-442
    (0x4409, "en-MY"),
    // Source: src/sfnt/SkOTTable_name.cpp:443-443
    (0x440a, "es-SV"),
    // Source: src/sfnt/SkOTTable_name.cpp:444-444
    (0x4809, "en-SG"),
    // Source: src/sfnt/SkOTTable_name.cpp:445-445
    (0x480a, "es-HN"),
    // Source: src/sfnt/SkOTTable_name.cpp:446-446
    (0x4c0a, "es-NI"),
    // Source: src/sfnt/SkOTTable_name.cpp:447-447
    (0x500a, "es-PR"),
    // Source: src/sfnt/SkOTTable_name.cpp:448-448
    (0x540a, "es-US"),
];
// Source: src/sfnt/SkOTTable_name.cpp:451-455
fn BCP47FromLanguageIdLess(a: &(u16, &str), b: &(u16, &str)) -> bool {
    a.0 < b.0
}
impl Iterator {
    // Source: src/sfnt/SkOTTable_name.cpp:457-586
    pub fn next(&mut self, record: &mut IteratorRecord) -> bool {
        unsafe {
            use core::mem::size_of;
            if self.fNameTableSize < size_of::<SkOTTableName>() {
                return false;
            }
            let nameTable = self.fNameTable.cast::<SkOTTableName>().read_unaligned();
            let nameRecords = self.fNameTable.add(size_of::<SkOTTableName>());
            let nameRecordsSize = self.fNameTableSize - size_of::<SkOTTableName>();
            let stringTableOffset = u16::from_be(nameTable.stringOffset) as usize;
            if self.fNameTableSize < stringTableOffset {
                return false;
            }
            let stringTable = self.fNameTable.add(stringTableOffset);
            let stringTableSize = self.fNameTableSize - stringTableOffset;
            let nameRecordsCount = u16::from_be(nameTable.count) as usize;
            let nameRecordsMax = nameRecordsCount.min(nameRecordsSize / size_of::<Record>());
            let nameRecord = loop {
                if self.fIndex >= nameRecordsMax {
                    return false;
                }
                let r = nameRecords
                    .add(size_of::<Record>() * self.fIndex)
                    .cast::<Record>()
                    .read_unaligned();
                self.fIndex += 1;
                if self.fType == -1 || r.nameID as i32 == self.fType {
                    break r;
                }
            };
            record.r#type = nameRecord.nameID;
            let nameOffset = u16::from_be(nameRecord.offset) as usize;
            let nameLength = u16::from_be(nameRecord.length) as usize;
            if stringTableSize < nameOffset + nameLength {
                return false;
            }
            let nameString = core::slice::from_raw_parts(stringTable.add(nameOffset), nameLength);
            match nameRecord.platformID {
                PlatformID::Windows => {
                    if nameRecord.encodingID != WindowsEncoding::UnicodeBMPUCS2
                        && nameRecord.encodingID != WindowsEncoding::UnicodeUCS4
                        && nameRecord.encodingID != WindowsEncoding::Symbol
                    {
                        record.name.clear();
                    } else {
                        SkString_from_UTF16BE(nameString, &mut record.name);
                    }
                }
                PlatformID::Unicode | PlatformID::ISO => {
                    SkString_from_UTF16BE(nameString, &mut record.name)
                }
                PlatformID::Macintosh => {
                    if nameRecord.encodingID != MacintoshEncoding::Roman {
                        record.name.clear();
                    } else {
                        SkStringFromMacRoman(nameString, &mut record.name);
                    }
                }
                _ => {
                    crate::SkASSERT!(@source b"src/sfnt/SkOTTable_name.cpp\0".as_ptr(),528; @condition b"false\0".as_ptr(); false);
                    record.name.clear();
                }
            }
            let languageID = u16::from_be(nameRecord.languageID);
            if SkOTTableName::format_1 == nameTable.format && languageID >= 0x8000 {
                let languageTagRecordIndex = (languageID - 0x8000) as usize;
                if nameRecordsSize < size_of::<Record>() * nameRecordsCount {
                    return false;
                }
                let format1extData = nameRecords.add(size_of::<Record>() * nameRecordsCount);
                let format1extSize = nameRecordsSize - size_of::<Record>() * nameRecordsCount;
                if format1extSize < size_of::<Format1Ext>() {
                    return false;
                }
                let format1ext = format1extData.cast::<Format1Ext>().read_unaligned();
                let languageTagRecords = format1extData.add(size_of::<Format1Ext>());
                let languageTagRecordsSize = format1extSize - size_of::<Format1Ext>();
                if languageTagRecordIndex < (u16::from_be(format1ext.langTagCount) as usize) {
                    if languageTagRecordsSize
                        < size_of::<LangTagRecord>() * (languageTagRecordIndex + 1)
                    {
                        return false;
                    }
                    let languageTagRecord = languageTagRecords
                        .add(size_of::<LangTagRecord>() * languageTagRecordIndex)
                        .cast::<LangTagRecord>()
                        .read_unaligned();
                    let languageOffset = u16::from_be(languageTagRecord.offset) as usize;
                    let languageLength = u16::from_be(languageTagRecord.length) as usize;
                    if self.fNameTableSize < stringTableOffset + languageOffset + languageLength {
                        return false;
                    }
                    let languageString = core::slice::from_raw_parts(
                        stringTable.add(languageOffset),
                        languageLength,
                    );
                    SkString_from_UTF16BE(languageString, &mut record.language);
                    return true;
                }
            }
            let target = (languageID, "");
            let languageIndex = crate::src::core::SkTSearch::SkTSearchWith(
                BCP47FromLanguageID.as_ptr(),
                BCP47FromLanguageID.len() as i32,
                &target,
                size_of::<(u16, &str)>(),
                BCP47FromLanguageIdLess,
            );
            if languageIndex >= 0 {
                record.language = BCP47FromLanguageID[languageIndex as usize].1.into();
                return true;
            }
            record.language = "und".into();
            true
        }
    }
}
