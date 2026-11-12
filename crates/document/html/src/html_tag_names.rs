#![allow(non_snake_case, non_camel_case_types)]

// cpp: html/html_tag_names.h:12-13
pub mod html_names {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum HTMLTag {
        // cpp: html/html_tag_names.h:16
        kUnknown = 0,
        // cpp: html/html_tag_names.h:17
        kA,
        // cpp: html/html_tag_names.h:18
        kAbbr,
        // cpp: html/html_tag_names.h:19
        kAcronym,
        // cpp: html/html_tag_names.h:20
        kAddress,
        // cpp: html/html_tag_names.h:21
        kApplet,
        // cpp: html/html_tag_names.h:22
        kArea,
        // cpp: html/html_tag_names.h:23
        kArticle,
        // cpp: html/html_tag_names.h:24
        kAside,
        // cpp: html/html_tag_names.h:25
        kAudio,
        // cpp: html/html_tag_names.h:26
        kB,
        // cpp: html/html_tag_names.h:27
        kBase,
        // cpp: html/html_tag_names.h:28
        kBasefont,
        // cpp: html/html_tag_names.h:29
        kBdi,
        // cpp: html/html_tag_names.h:30
        kBdo,
        // cpp: html/html_tag_names.h:31
        kBgsound,
        // cpp: html/html_tag_names.h:32
        kBig,
        // cpp: html/html_tag_names.h:33
        kBlockquote,
        // cpp: html/html_tag_names.h:34
        kBody,
        // cpp: html/html_tag_names.h:35
        kBr,
        // cpp: html/html_tag_names.h:36
        kButton,
        // cpp: html/html_tag_names.h:37
        kCamera,
        // cpp: html/html_tag_names.h:38
        kCanvas,
        // cpp: html/html_tag_names.h:39
        kCaption,
        // cpp: html/html_tag_names.h:40
        kCenter,
        // cpp: html/html_tag_names.h:41
        kCite,
        // cpp: html/html_tag_names.h:42
        kCode,
        // cpp: html/html_tag_names.h:43
        kCol,
        // cpp: html/html_tag_names.h:44
        kColgroup,
        // cpp: html/html_tag_names.h:45
        kCommand,
        // cpp: html/html_tag_names.h:46
        kCredential,
        // cpp: html/html_tag_names.h:47
        kData,
        // cpp: html/html_tag_names.h:48
        kDatalist,
        // cpp: html/html_tag_names.h:49
        kDd,
        // cpp: html/html_tag_names.h:50
        kDel,
        // cpp: html/html_tag_names.h:51
        kDetails,
        // cpp: html/html_tag_names.h:52
        kDfn,
        // cpp: html/html_tag_names.h:53
        kDialog,
        // cpp: html/html_tag_names.h:54
        kDir,
        // cpp: html/html_tag_names.h:55
        kDiv,
        // cpp: html/html_tag_names.h:56
        kDl,
        // cpp: html/html_tag_names.h:57
        kDt,
        // cpp: html/html_tag_names.h:58
        kEm,
        // cpp: html/html_tag_names.h:59
        kEmbed,
        // cpp: html/html_tag_names.h:60
        kFencedframeOrUnknown,
        // cpp: html/html_tag_names.h:61
        kFieldset,
        // cpp: html/html_tag_names.h:62
        kFigcaption,
        // cpp: html/html_tag_names.h:63
        kFigure,
        // cpp: html/html_tag_names.h:64
        kFont,
        // cpp: html/html_tag_names.h:65
        kFooter,
        // cpp: html/html_tag_names.h:66
        kForm,
        // cpp: html/html_tag_names.h:67
        kFrame,
        // cpp: html/html_tag_names.h:68
        kFrameset,
        // cpp: html/html_tag_names.h:69
        kGeolocation,
        // cpp: html/html_tag_names.h:70
        kH1,
        // cpp: html/html_tag_names.h:71
        kH2,
        // cpp: html/html_tag_names.h:72
        kH3,
        // cpp: html/html_tag_names.h:73
        kH4,
        // cpp: html/html_tag_names.h:74
        kH5,
        // cpp: html/html_tag_names.h:75
        kH6,
        // cpp: html/html_tag_names.h:76
        kHead,
        // cpp: html/html_tag_names.h:77
        kHeader,
        // cpp: html/html_tag_names.h:78
        kHgroup,
        // cpp: html/html_tag_names.h:79
        kHr,
        // cpp: html/html_tag_names.h:80
        kHTML,
        // cpp: html/html_tag_names.h:81
        kI,
        // cpp: html/html_tag_names.h:82
        kIFrame,
        // cpp: html/html_tag_names.h:83
        kImage,
        // cpp: html/html_tag_names.h:84
        kImg,
        // cpp: html/html_tag_names.h:85
        kInput,
        // cpp: html/html_tag_names.h:86
        kIns,
        // cpp: html/html_tag_names.h:87
        kInstallOrUnknown,
        // cpp: html/html_tag_names.h:88
        kKbd,
        // cpp: html/html_tag_names.h:89
        kKeygen,
        // cpp: html/html_tag_names.h:90
        kLabel,
        // cpp: html/html_tag_names.h:91
        kLayer,
        // cpp: html/html_tag_names.h:92
        kLegend,
        // cpp: html/html_tag_names.h:93
        kLi,
        // cpp: html/html_tag_names.h:94
        kLink,
        // cpp: html/html_tag_names.h:95
        kListing,
        // cpp: html/html_tag_names.h:96
        kLogin,
        // cpp: html/html_tag_names.h:97
        kMain,
        // cpp: html/html_tag_names.h:98
        kMap,
        // cpp: html/html_tag_names.h:99
        kMark,
        // cpp: html/html_tag_names.h:100
        kMarquee,
        // cpp: html/html_tag_names.h:101
        kMenu,
        // cpp: html/html_tag_names.h:102
        kMenubar,
        // cpp: html/html_tag_names.h:103
        kMenuitem,
        // cpp: html/html_tag_names.h:104
        kMenulist,
        // cpp: html/html_tag_names.h:105
        kMeta,
        // cpp: html/html_tag_names.h:106
        kMeter,
        // cpp: html/html_tag_names.h:107
        kMicrophone,
        // cpp: html/html_tag_names.h:108
        kNav,
        // cpp: html/html_tag_names.h:109
        kNobr,
        // cpp: html/html_tag_names.h:110
        kNoembed,
        // cpp: html/html_tag_names.h:111
        kNoframes,
        // cpp: html/html_tag_names.h:112
        kNolayer,
        // cpp: html/html_tag_names.h:113
        kNoscript,
        // cpp: html/html_tag_names.h:114
        kObject,
        // cpp: html/html_tag_names.h:115
        kOl,
        // cpp: html/html_tag_names.h:116
        kOptgroup,
        // cpp: html/html_tag_names.h:117
        kOption,
        // cpp: html/html_tag_names.h:118
        kOutput,
        // cpp: html/html_tag_names.h:119
        kP,
        // cpp: html/html_tag_names.h:120
        kParam,
        // cpp: html/html_tag_names.h:121
        kPicture,
        // cpp: html/html_tag_names.h:122
        kPlaintext,
        // cpp: html/html_tag_names.h:123
        kPre,
        // cpp: html/html_tag_names.h:124
        kProgress,
        // cpp: html/html_tag_names.h:125
        kQ,
        // cpp: html/html_tag_names.h:126
        kRb,
        // cpp: html/html_tag_names.h:127
        kRp,
        // cpp: html/html_tag_names.h:128
        kRt,
        // cpp: html/html_tag_names.h:129
        kRTC,
        // cpp: html/html_tag_names.h:130
        kRuby,
        // cpp: html/html_tag_names.h:131
        kS,
        // cpp: html/html_tag_names.h:132
        kSamp,
        // cpp: html/html_tag_names.h:133
        kScript,
        // cpp: html/html_tag_names.h:134
        kSearch,
        // cpp: html/html_tag_names.h:135
        kSection,
        // cpp: html/html_tag_names.h:136
        kSelect,
        // cpp: html/html_tag_names.h:137
        kSelectedcontent,
        // cpp: html/html_tag_names.h:138
        kSlot,
        // cpp: html/html_tag_names.h:139
        kSmall,
        // cpp: html/html_tag_names.h:140
        kSource,
        // cpp: html/html_tag_names.h:141
        kSpan,
        // cpp: html/html_tag_names.h:142
        kStrike,
        // cpp: html/html_tag_names.h:143
        kStrong,
        // cpp: html/html_tag_names.h:144
        kStyle,
        // cpp: html/html_tag_names.h:145
        kSub,
        // cpp: html/html_tag_names.h:146
        kSubmenu,
        // cpp: html/html_tag_names.h:147
        kSummary,
        // cpp: html/html_tag_names.h:148
        kSup,
        // cpp: html/html_tag_names.h:149
        kTable,
        // cpp: html/html_tag_names.h:150
        kTbody,
        // cpp: html/html_tag_names.h:151
        kTd,
        // cpp: html/html_tag_names.h:152
        kTemplate,
        // cpp: html/html_tag_names.h:153
        kTextarea,
        // cpp: html/html_tag_names.h:154
        kTfoot,
        // cpp: html/html_tag_names.h:155
        kTh,
        // cpp: html/html_tag_names.h:156
        kThead,
        // cpp: html/html_tag_names.h:157
        kTime,
        // cpp: html/html_tag_names.h:158
        kTitle,
        // cpp: html/html_tag_names.h:159
        kTr,
        // cpp: html/html_tag_names.h:160
        kTrack,
        // cpp: html/html_tag_names.h:161
        kTt,
        // cpp: html/html_tag_names.h:162
        kU,
        // cpp: html/html_tag_names.h:163
        kUl,
        // cpp: html/html_tag_names.h:164
        kUsermediaOrUnknown,
        // cpp: html/html_tag_names.h:165
        kVar,
        // cpp: html/html_tag_names.h:166
        kVideo,
        // cpp: html/html_tag_names.h:167
        kWbr,
        // cpp: html/html_tag_names.h:168
        kXmp,
    }
}

use self::html_names::HTMLTag;

pub trait HtmlTagCodeUnit: Sized {
    fn lookup(span: &[Self]) -> HTMLTag;
}

// cpp: html/html_tag_names.h:172-173
pub fn LookupHtmlTag<T: HtmlTagCodeUnit>(span: &[T]) -> HTMLTag {
    T::lookup(span)
}

// cpp: html/html_tag_names.cc:18-19
impl HtmlTagCodeUnit for u16 {
    fn lookup(span: &[Self]) -> HTMLTag {
        // cpp: html/html_tag_names.cc:20-21
        let data = span;
        let length = span.len();
        // cpp: html/html_tag_names.cc:22
        debug_assert!(!data.as_ptr().is_null());
        // cpp: html/html_tag_names.cc:23
        debug_assert_ne!(length, 0);
        // cpp: html/html_tag_names.cc:24
        match length {
            // cpp: html/html_tag_names.cc:25
            1 => {
                // cpp: html/html_tag_names.cc:26
                match data[0] {
                    // cpp: html/html_tag_names.cc:27
                    97 => {
                        // cpp: html/html_tag_names.cc:28
                        return HTMLTag::kA;
                    }
                    // cpp: html/html_tag_names.cc:29
                    98 => {
                        // cpp: html/html_tag_names.cc:30
                        return HTMLTag::kB;
                    }
                    // cpp: html/html_tag_names.cc:31
                    105 => {
                        // cpp: html/html_tag_names.cc:32
                        return HTMLTag::kI;
                    }
                    // cpp: html/html_tag_names.cc:33
                    112 => {
                        // cpp: html/html_tag_names.cc:34
                        return HTMLTag::kP;
                    }
                    // cpp: html/html_tag_names.cc:35
                    113 => {
                        // cpp: html/html_tag_names.cc:36
                        return HTMLTag::kQ;
                    }
                    // cpp: html/html_tag_names.cc:37
                    115 => {
                        // cpp: html/html_tag_names.cc:38
                        return HTMLTag::kS;
                    }
                    // cpp: html/html_tag_names.cc:39
                    117 => {
                        // cpp: html/html_tag_names.cc:40
                        return HTMLTag::kU;
                    }
                    // cpp: html/html_tag_names.cc:41
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:42
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:43
            2 => {
                // cpp: html/html_tag_names.cc:44
                match data[0] {
                    // cpp: html/html_tag_names.cc:45
                    98 => {
                        // cpp: html/html_tag_names.cc:46
                        if data[1] == 114 {
                            // cpp: html/html_tag_names.cc:47
                            return HTMLTag::kBr;
                            // cpp: html/html_tag_names.cc:48
                        }
                        // cpp: html/html_tag_names.cc:49
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:50
                    100 => {
                        // cpp: html/html_tag_names.cc:51
                        match data[1] {
                            // cpp: html/html_tag_names.cc:52
                            100 => {
                                // cpp: html/html_tag_names.cc:53
                                return HTMLTag::kDd;
                            }
                            // cpp: html/html_tag_names.cc:54
                            108 => {
                                // cpp: html/html_tag_names.cc:55
                                return HTMLTag::kDl;
                            }
                            // cpp: html/html_tag_names.cc:56
                            116 => {
                                // cpp: html/html_tag_names.cc:57
                                return HTMLTag::kDt;
                            }
                            // cpp: html/html_tag_names.cc:58
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:59
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:60
                    101 => {
                        // cpp: html/html_tag_names.cc:61
                        if data[1] == 109 {
                            // cpp: html/html_tag_names.cc:62
                            return HTMLTag::kEm;
                            // cpp: html/html_tag_names.cc:63
                        }
                        // cpp: html/html_tag_names.cc:64
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:65
                    104 => {
                        // cpp: html/html_tag_names.cc:66
                        match data[1] {
                            // cpp: html/html_tag_names.cc:67
                            49 => {
                                // cpp: html/html_tag_names.cc:68
                                return HTMLTag::kH1;
                            }
                            // cpp: html/html_tag_names.cc:69
                            50 => {
                                // cpp: html/html_tag_names.cc:70
                                return HTMLTag::kH2;
                            }
                            // cpp: html/html_tag_names.cc:71
                            51 => {
                                // cpp: html/html_tag_names.cc:72
                                return HTMLTag::kH3;
                            }
                            // cpp: html/html_tag_names.cc:73
                            52 => {
                                // cpp: html/html_tag_names.cc:74
                                return HTMLTag::kH4;
                            }
                            // cpp: html/html_tag_names.cc:75
                            53 => {
                                // cpp: html/html_tag_names.cc:76
                                return HTMLTag::kH5;
                            }
                            // cpp: html/html_tag_names.cc:77
                            54 => {
                                // cpp: html/html_tag_names.cc:78
                                return HTMLTag::kH6;
                            }
                            // cpp: html/html_tag_names.cc:79
                            114 => {
                                // cpp: html/html_tag_names.cc:80
                                return HTMLTag::kHr;
                            }
                            // cpp: html/html_tag_names.cc:81
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:82
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:83
                    108 => {
                        // cpp: html/html_tag_names.cc:84
                        if data[1] == 105 {
                            // cpp: html/html_tag_names.cc:85
                            return HTMLTag::kLi;
                            // cpp: html/html_tag_names.cc:86
                        }
                        // cpp: html/html_tag_names.cc:87
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:88
                    111 => {
                        // cpp: html/html_tag_names.cc:89
                        if data[1] == 108 {
                            // cpp: html/html_tag_names.cc:90
                            return HTMLTag::kOl;
                            // cpp: html/html_tag_names.cc:91
                        }
                        // cpp: html/html_tag_names.cc:92
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:93
                    114 => {
                        // cpp: html/html_tag_names.cc:94
                        match data[1] {
                            // cpp: html/html_tag_names.cc:95
                            98 => {
                                // cpp: html/html_tag_names.cc:96
                                return HTMLTag::kRb;
                            }
                            // cpp: html/html_tag_names.cc:97
                            112 => {
                                // cpp: html/html_tag_names.cc:98
                                return HTMLTag::kRp;
                            }
                            // cpp: html/html_tag_names.cc:99
                            116 => {
                                // cpp: html/html_tag_names.cc:100
                                return HTMLTag::kRt;
                            }
                            // cpp: html/html_tag_names.cc:101
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:102
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:103
                    116 => {
                        // cpp: html/html_tag_names.cc:104
                        match data[1] {
                            // cpp: html/html_tag_names.cc:105
                            100 => {
                                // cpp: html/html_tag_names.cc:106
                                return HTMLTag::kTd;
                            }
                            // cpp: html/html_tag_names.cc:107
                            104 => {
                                // cpp: html/html_tag_names.cc:108
                                return HTMLTag::kTh;
                            }
                            // cpp: html/html_tag_names.cc:109
                            114 => {
                                // cpp: html/html_tag_names.cc:110
                                return HTMLTag::kTr;
                            }
                            // cpp: html/html_tag_names.cc:111
                            116 => {
                                // cpp: html/html_tag_names.cc:112
                                return HTMLTag::kTt;
                            }
                            // cpp: html/html_tag_names.cc:113
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:114
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:115
                    117 => {
                        // cpp: html/html_tag_names.cc:116
                        if data[1] == 108 {
                            // cpp: html/html_tag_names.cc:117
                            return HTMLTag::kUl;
                            // cpp: html/html_tag_names.cc:118
                        }
                        // cpp: html/html_tag_names.cc:119
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:120
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:121
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:122
            3 => {
                // cpp: html/html_tag_names.cc:123
                match data[0] {
                    // cpp: html/html_tag_names.cc:124
                    98 => {
                        // cpp: html/html_tag_names.cc:125
                        match data[1] {
                            // cpp: html/html_tag_names.cc:126
                            100 => {
                                // cpp: html/html_tag_names.cc:127
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:128
                                    105 => {
                                        // cpp: html/html_tag_names.cc:129
                                        return HTMLTag::kBdi;
                                    }
                                    // cpp: html/html_tag_names.cc:130
                                    111 => {
                                        // cpp: html/html_tag_names.cc:131
                                        return HTMLTag::kBdo;
                                    }
                                    // cpp: html/html_tag_names.cc:132
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:133
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:134
                            105 => {
                                // cpp: html/html_tag_names.cc:135
                                if data[2] == 103 {
                                    // cpp: html/html_tag_names.cc:136
                                    return HTMLTag::kBig;
                                    // cpp: html/html_tag_names.cc:137
                                }
                                // cpp: html/html_tag_names.cc:138
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:139
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:140
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:141
                    99 => {
                        // cpp: html/html_tag_names.cc:142
                        if &data[1..3] == &[111, 108][..] {
                            // cpp: html/html_tag_names.cc:143
                            return HTMLTag::kCol;
                            // cpp: html/html_tag_names.cc:144
                        }
                        // cpp: html/html_tag_names.cc:145
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:146
                    100 => {
                        // cpp: html/html_tag_names.cc:147
                        match data[1] {
                            // cpp: html/html_tag_names.cc:148
                            101 => {
                                // cpp: html/html_tag_names.cc:149
                                if data[2] == 108 {
                                    // cpp: html/html_tag_names.cc:150
                                    return HTMLTag::kDel;
                                    // cpp: html/html_tag_names.cc:151
                                }
                                // cpp: html/html_tag_names.cc:152
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:153
                            102 => {
                                // cpp: html/html_tag_names.cc:154
                                if data[2] == 110 {
                                    // cpp: html/html_tag_names.cc:155
                                    return HTMLTag::kDfn;
                                    // cpp: html/html_tag_names.cc:156
                                }
                                // cpp: html/html_tag_names.cc:157
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:158
                            105 => {
                                // cpp: html/html_tag_names.cc:159
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:160
                                    114 => {
                                        // cpp: html/html_tag_names.cc:161
                                        return HTMLTag::kDir;
                                    }
                                    // cpp: html/html_tag_names.cc:162
                                    118 => {
                                        // cpp: html/html_tag_names.cc:163
                                        return HTMLTag::kDiv;
                                    }
                                    // cpp: html/html_tag_names.cc:164
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:165
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:166
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:167
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:168
                    105 => {
                        // cpp: html/html_tag_names.cc:169
                        match data[1] {
                            // cpp: html/html_tag_names.cc:170
                            109 => {
                                // cpp: html/html_tag_names.cc:171
                                if data[2] == 103 {
                                    // cpp: html/html_tag_names.cc:172
                                    return HTMLTag::kImg;
                                    // cpp: html/html_tag_names.cc:173
                                }
                                // cpp: html/html_tag_names.cc:174
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:175
                            110 => {
                                // cpp: html/html_tag_names.cc:176
                                if data[2] == 115 {
                                    // cpp: html/html_tag_names.cc:177
                                    return HTMLTag::kIns;
                                    // cpp: html/html_tag_names.cc:178
                                }
                                // cpp: html/html_tag_names.cc:179
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:180
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:181
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:182
                    107 => {
                        // cpp: html/html_tag_names.cc:183
                        if &data[1..3] == &[98, 100][..] {
                            // cpp: html/html_tag_names.cc:184
                            return HTMLTag::kKbd;
                            // cpp: html/html_tag_names.cc:185
                        }
                        // cpp: html/html_tag_names.cc:186
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:187
                    109 => {
                        // cpp: html/html_tag_names.cc:188
                        if &data[1..3] == &[97, 112][..] {
                            // cpp: html/html_tag_names.cc:189
                            return HTMLTag::kMap;
                            // cpp: html/html_tag_names.cc:190
                        }
                        // cpp: html/html_tag_names.cc:191
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:192
                    110 => {
                        // cpp: html/html_tag_names.cc:193
                        if &data[1..3] == &[97, 118][..] {
                            // cpp: html/html_tag_names.cc:194
                            return HTMLTag::kNav;
                            // cpp: html/html_tag_names.cc:195
                        }
                        // cpp: html/html_tag_names.cc:196
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:197
                    112 => {
                        // cpp: html/html_tag_names.cc:198
                        if &data[1..3] == &[114, 101][..] {
                            // cpp: html/html_tag_names.cc:199
                            return HTMLTag::kPre;
                            // cpp: html/html_tag_names.cc:200
                        }
                        // cpp: html/html_tag_names.cc:201
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:202
                    114 => {
                        // cpp: html/html_tag_names.cc:203
                        if &data[1..3] == &[116, 99][..] {
                            // cpp: html/html_tag_names.cc:204
                            return HTMLTag::kRTC;
                            // cpp: html/html_tag_names.cc:205
                        }
                        // cpp: html/html_tag_names.cc:206
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:207
                    115 => {
                        // cpp: html/html_tag_names.cc:208
                        match data[1] {
                            // cpp: html/html_tag_names.cc:209
                            117 => {
                                // cpp: html/html_tag_names.cc:210
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:211
                                    98 => {
                                        // cpp: html/html_tag_names.cc:212
                                        return HTMLTag::kSub;
                                    }
                                    // cpp: html/html_tag_names.cc:213
                                    112 => {
                                        // cpp: html/html_tag_names.cc:214
                                        return HTMLTag::kSup;
                                    }
                                    // cpp: html/html_tag_names.cc:215
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:216
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:217
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:218
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:219
                    118 => {
                        // cpp: html/html_tag_names.cc:220
                        if &data[1..3] == &[97, 114][..] {
                            // cpp: html/html_tag_names.cc:221
                            return HTMLTag::kVar;
                            // cpp: html/html_tag_names.cc:222
                        }
                        // cpp: html/html_tag_names.cc:223
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:224
                    119 => {
                        // cpp: html/html_tag_names.cc:225
                        if &data[1..3] == &[98, 114][..] {
                            // cpp: html/html_tag_names.cc:226
                            return HTMLTag::kWbr;
                            // cpp: html/html_tag_names.cc:227
                        }
                        // cpp: html/html_tag_names.cc:228
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:229
                    120 => {
                        // cpp: html/html_tag_names.cc:230
                        if &data[1..3] == &[109, 112][..] {
                            // cpp: html/html_tag_names.cc:231
                            return HTMLTag::kXmp;
                            // cpp: html/html_tag_names.cc:232
                        }
                        // cpp: html/html_tag_names.cc:233
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:234
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:235
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:236
            4 => {
                // cpp: html/html_tag_names.cc:237
                match data[0] {
                    // cpp: html/html_tag_names.cc:238
                    97 => {
                        // cpp: html/html_tag_names.cc:239
                        match data[1] {
                            // cpp: html/html_tag_names.cc:240
                            98 => {
                                // cpp: html/html_tag_names.cc:241
                                if &data[2..4] == &[98, 114][..] {
                                    // cpp: html/html_tag_names.cc:242
                                    return HTMLTag::kAbbr;
                                    // cpp: html/html_tag_names.cc:243
                                }
                                // cpp: html/html_tag_names.cc:244
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:245
                            114 => {
                                // cpp: html/html_tag_names.cc:246
                                if &data[2..4] == &[101, 97][..] {
                                    // cpp: html/html_tag_names.cc:247
                                    return HTMLTag::kArea;
                                    // cpp: html/html_tag_names.cc:248
                                }
                                // cpp: html/html_tag_names.cc:249
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:250
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:251
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:252
                    98 => {
                        // cpp: html/html_tag_names.cc:253
                        match data[1] {
                            // cpp: html/html_tag_names.cc:254
                            97 => {
                                // cpp: html/html_tag_names.cc:255
                                if &data[2..4] == &[115, 101][..] {
                                    // cpp: html/html_tag_names.cc:256
                                    return HTMLTag::kBase;
                                    // cpp: html/html_tag_names.cc:257
                                }
                                // cpp: html/html_tag_names.cc:258
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:259
                            111 => {
                                // cpp: html/html_tag_names.cc:260
                                if &data[2..4] == &[100, 121][..] {
                                    // cpp: html/html_tag_names.cc:261
                                    return HTMLTag::kBody;
                                    // cpp: html/html_tag_names.cc:262
                                }
                                // cpp: html/html_tag_names.cc:263
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:264
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:265
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:266
                    99 => {
                        // cpp: html/html_tag_names.cc:267
                        match data[1] {
                            // cpp: html/html_tag_names.cc:268
                            105 => {
                                // cpp: html/html_tag_names.cc:269
                                if &data[2..4] == &[116, 101][..] {
                                    // cpp: html/html_tag_names.cc:270
                                    return HTMLTag::kCite;
                                    // cpp: html/html_tag_names.cc:271
                                }
                                // cpp: html/html_tag_names.cc:272
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:273
                            111 => {
                                // cpp: html/html_tag_names.cc:274
                                if &data[2..4] == &[100, 101][..] {
                                    // cpp: html/html_tag_names.cc:275
                                    return HTMLTag::kCode;
                                    // cpp: html/html_tag_names.cc:276
                                }
                                // cpp: html/html_tag_names.cc:277
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:278
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:279
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:280
                    100 => {
                        // cpp: html/html_tag_names.cc:281
                        if &data[1..4] == &[97, 116, 97][..] {
                            // cpp: html/html_tag_names.cc:282
                            return HTMLTag::kData;
                            // cpp: html/html_tag_names.cc:283
                        }
                        // cpp: html/html_tag_names.cc:284
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:285
                    102 => {
                        // cpp: html/html_tag_names.cc:286
                        match data[1] {
                            // cpp: html/html_tag_names.cc:287
                            111 => {
                                // cpp: html/html_tag_names.cc:288
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:289
                                    110 => {
                                        // cpp: html/html_tag_names.cc:290
                                        if data[3] == 116 {
                                            // cpp: html/html_tag_names.cc:291
                                            return HTMLTag::kFont;
                                            // cpp: html/html_tag_names.cc:292
                                        }
                                        // cpp: html/html_tag_names.cc:293
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:294
                                    114 => {
                                        // cpp: html/html_tag_names.cc:295
                                        if data[3] == 109 {
                                            // cpp: html/html_tag_names.cc:296
                                            return HTMLTag::kForm;
                                            // cpp: html/html_tag_names.cc:297
                                        }
                                        // cpp: html/html_tag_names.cc:298
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:299
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:300
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:301
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:302
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:303
                    104 => {
                        // cpp: html/html_tag_names.cc:304
                        match data[1] {
                            // cpp: html/html_tag_names.cc:305
                            101 => {
                                // cpp: html/html_tag_names.cc:306
                                if &data[2..4] == &[97, 100][..] {
                                    // cpp: html/html_tag_names.cc:307
                                    return HTMLTag::kHead;
                                    // cpp: html/html_tag_names.cc:308
                                }
                                // cpp: html/html_tag_names.cc:309
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:310
                            116 => {
                                // cpp: html/html_tag_names.cc:311
                                if &data[2..4] == &[109, 108][..] {
                                    // cpp: html/html_tag_names.cc:312
                                    return HTMLTag::kHTML;
                                    // cpp: html/html_tag_names.cc:313
                                }
                                // cpp: html/html_tag_names.cc:314
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:315
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:316
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:317
                    108 => {
                        // cpp: html/html_tag_names.cc:318
                        if &data[1..4] == &[105, 110, 107][..] {
                            // cpp: html/html_tag_names.cc:319
                            return HTMLTag::kLink;
                            // cpp: html/html_tag_names.cc:320
                        }
                        // cpp: html/html_tag_names.cc:321
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:322
                    109 => {
                        // cpp: html/html_tag_names.cc:323
                        match data[1] {
                            // cpp: html/html_tag_names.cc:324
                            97 => {
                                // cpp: html/html_tag_names.cc:325
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:326
                                    105 => {
                                        // cpp: html/html_tag_names.cc:327
                                        if data[3] == 110 {
                                            // cpp: html/html_tag_names.cc:328
                                            return HTMLTag::kMain;
                                            // cpp: html/html_tag_names.cc:329
                                        }
                                        // cpp: html/html_tag_names.cc:330
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:331
                                    114 => {
                                        // cpp: html/html_tag_names.cc:332
                                        if data[3] == 107 {
                                            // cpp: html/html_tag_names.cc:333
                                            return HTMLTag::kMark;
                                            // cpp: html/html_tag_names.cc:334
                                        }
                                        // cpp: html/html_tag_names.cc:335
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:336
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:337
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:338
                            101 => {
                                // cpp: html/html_tag_names.cc:339
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:340
                                    110 => {
                                        // cpp: html/html_tag_names.cc:341
                                        if data[3] == 117 {
                                            // cpp: html/html_tag_names.cc:342
                                            return HTMLTag::kMenu;
                                            // cpp: html/html_tag_names.cc:343
                                        }
                                        // cpp: html/html_tag_names.cc:344
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:345
                                    116 => {
                                        // cpp: html/html_tag_names.cc:346
                                        if data[3] == 97 {
                                            // cpp: html/html_tag_names.cc:347
                                            return HTMLTag::kMeta;
                                            // cpp: html/html_tag_names.cc:348
                                        }
                                        // cpp: html/html_tag_names.cc:349
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:350
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:351
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:352
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:353
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:354
                    110 => {
                        // cpp: html/html_tag_names.cc:355
                        if &data[1..4] == &[111, 98, 114][..] {
                            // cpp: html/html_tag_names.cc:356
                            return HTMLTag::kNobr;
                            // cpp: html/html_tag_names.cc:357
                        }
                        // cpp: html/html_tag_names.cc:358
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:359
                    114 => {
                        // cpp: html/html_tag_names.cc:360
                        if &data[1..4] == &[117, 98, 121][..] {
                            // cpp: html/html_tag_names.cc:361
                            return HTMLTag::kRuby;
                            // cpp: html/html_tag_names.cc:362
                        }
                        // cpp: html/html_tag_names.cc:363
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:364
                    115 => {
                        // cpp: html/html_tag_names.cc:365
                        match data[1] {
                            // cpp: html/html_tag_names.cc:366
                            97 => {
                                // cpp: html/html_tag_names.cc:367
                                if &data[2..4] == &[109, 112][..] {
                                    // cpp: html/html_tag_names.cc:368
                                    return HTMLTag::kSamp;
                                    // cpp: html/html_tag_names.cc:369
                                }
                                // cpp: html/html_tag_names.cc:370
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:371
                            108 => {
                                // cpp: html/html_tag_names.cc:372
                                if &data[2..4] == &[111, 116][..] {
                                    // cpp: html/html_tag_names.cc:373
                                    return HTMLTag::kSlot;
                                    // cpp: html/html_tag_names.cc:374
                                }
                                // cpp: html/html_tag_names.cc:375
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:376
                            112 => {
                                // cpp: html/html_tag_names.cc:377
                                if &data[2..4] == &[97, 110][..] {
                                    // cpp: html/html_tag_names.cc:378
                                    return HTMLTag::kSpan;
                                    // cpp: html/html_tag_names.cc:379
                                }
                                // cpp: html/html_tag_names.cc:380
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:381
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:382
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:383
                    116 => {
                        // cpp: html/html_tag_names.cc:384
                        if &data[1..4] == &[105, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:385
                            return HTMLTag::kTime;
                            // cpp: html/html_tag_names.cc:386
                        }
                        // cpp: html/html_tag_names.cc:387
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:388
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:389
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:390
            5 => {
                // cpp: html/html_tag_names.cc:391
                match data[0] {
                    // cpp: html/html_tag_names.cc:392
                    97 => {
                        // cpp: html/html_tag_names.cc:393
                        match data[1] {
                            // cpp: html/html_tag_names.cc:394
                            115 => {
                                // cpp: html/html_tag_names.cc:395
                                if &data[2..5] == &[105, 100, 101][..] {
                                    // cpp: html/html_tag_names.cc:396
                                    return HTMLTag::kAside;
                                    // cpp: html/html_tag_names.cc:397
                                }
                                // cpp: html/html_tag_names.cc:398
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:399
                            117 => {
                                // cpp: html/html_tag_names.cc:400
                                if &data[2..5] == &[100, 105, 111][..] {
                                    // cpp: html/html_tag_names.cc:401
                                    return HTMLTag::kAudio;
                                    // cpp: html/html_tag_names.cc:402
                                }
                                // cpp: html/html_tag_names.cc:403
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:404
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:405
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:406
                    101 => {
                        // cpp: html/html_tag_names.cc:407
                        if &data[1..5] == &[109, 98, 101, 100][..] {
                            // cpp: html/html_tag_names.cc:408
                            return HTMLTag::kEmbed;
                            // cpp: html/html_tag_names.cc:409
                        }
                        // cpp: html/html_tag_names.cc:410
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:411
                    102 => {
                        // cpp: html/html_tag_names.cc:412
                        if &data[1..5] == &[114, 97, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:413
                            return HTMLTag::kFrame;
                            // cpp: html/html_tag_names.cc:414
                        }
                        // cpp: html/html_tag_names.cc:415
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:416
                    105 => {
                        // cpp: html/html_tag_names.cc:417
                        match data[1] {
                            // cpp: html/html_tag_names.cc:418
                            109 => {
                                // cpp: html/html_tag_names.cc:419
                                if &data[2..5] == &[97, 103, 101][..] {
                                    // cpp: html/html_tag_names.cc:420
                                    return HTMLTag::kImage;
                                    // cpp: html/html_tag_names.cc:421
                                }
                                // cpp: html/html_tag_names.cc:422
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:423
                            110 => {
                                // cpp: html/html_tag_names.cc:424
                                if &data[2..5] == &[112, 117, 116][..] {
                                    // cpp: html/html_tag_names.cc:425
                                    return HTMLTag::kInput;
                                    // cpp: html/html_tag_names.cc:426
                                }
                                // cpp: html/html_tag_names.cc:427
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:428
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:429
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:430
                    108 => {
                        // cpp: html/html_tag_names.cc:431
                        match data[1] {
                            // cpp: html/html_tag_names.cc:432
                            97 => {
                                // cpp: html/html_tag_names.cc:433
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:434
                                    98 => {
                                        // cpp: html/html_tag_names.cc:435
                                        if &data[3..5] == &[101, 108][..] {
                                            // cpp: html/html_tag_names.cc:436
                                            return HTMLTag::kLabel;
                                            // cpp: html/html_tag_names.cc:437
                                        }
                                        // cpp: html/html_tag_names.cc:438
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:439
                                    121 => {
                                        // cpp: html/html_tag_names.cc:440
                                        if &data[3..5] == &[101, 114][..] {
                                            // cpp: html/html_tag_names.cc:441
                                            return HTMLTag::kLayer;
                                            // cpp: html/html_tag_names.cc:442
                                        }
                                        // cpp: html/html_tag_names.cc:443
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:444
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:445
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:446
                            111 => {
                                // cpp: html/html_tag_names.cc:447
                                if &data[2..5] == &[103, 105, 110][..] {
                                    // cpp: html/html_tag_names.cc:448-457
                                    return if false {
                                        HTMLTag::kLogin
                                    } else {
                                        HTMLTag::kUnknown
                                    };
                                    // cpp: html/html_tag_names.cc:458
                                }
                                // cpp: html/html_tag_names.cc:459
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:460
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:461
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:462
                    109 => {
                        // cpp: html/html_tag_names.cc:463
                        if &data[1..5] == &[101, 116, 101, 114][..] {
                            // cpp: html/html_tag_names.cc:464
                            return HTMLTag::kMeter;
                            // cpp: html/html_tag_names.cc:465
                        }
                        // cpp: html/html_tag_names.cc:466
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:467
                    112 => {
                        // cpp: html/html_tag_names.cc:468
                        if &data[1..5] == &[97, 114, 97, 109][..] {
                            // cpp: html/html_tag_names.cc:469
                            return HTMLTag::kParam;
                            // cpp: html/html_tag_names.cc:470
                        }
                        // cpp: html/html_tag_names.cc:471
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:472
                    115 => {
                        // cpp: html/html_tag_names.cc:473
                        match data[1] {
                            // cpp: html/html_tag_names.cc:474
                            109 => {
                                // cpp: html/html_tag_names.cc:475
                                if &data[2..5] == &[97, 108, 108][..] {
                                    // cpp: html/html_tag_names.cc:476
                                    return HTMLTag::kSmall;
                                    // cpp: html/html_tag_names.cc:477
                                }
                                // cpp: html/html_tag_names.cc:478
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:479
                            116 => {
                                // cpp: html/html_tag_names.cc:480
                                if &data[2..5] == &[121, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:481
                                    return HTMLTag::kStyle;
                                    // cpp: html/html_tag_names.cc:482
                                }
                                // cpp: html/html_tag_names.cc:483
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:484
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:485
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:486
                    116 => {
                        // cpp: html/html_tag_names.cc:487
                        match data[1] {
                            // cpp: html/html_tag_names.cc:488
                            97 => {
                                // cpp: html/html_tag_names.cc:489
                                if &data[2..5] == &[98, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:490
                                    return HTMLTag::kTable;
                                    // cpp: html/html_tag_names.cc:491
                                }
                                // cpp: html/html_tag_names.cc:492
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:493
                            98 => {
                                // cpp: html/html_tag_names.cc:494
                                if &data[2..5] == &[111, 100, 121][..] {
                                    // cpp: html/html_tag_names.cc:495
                                    return HTMLTag::kTbody;
                                    // cpp: html/html_tag_names.cc:496
                                }
                                // cpp: html/html_tag_names.cc:497
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:498
                            102 => {
                                // cpp: html/html_tag_names.cc:499
                                if &data[2..5] == &[111, 111, 116][..] {
                                    // cpp: html/html_tag_names.cc:500
                                    return HTMLTag::kTfoot;
                                    // cpp: html/html_tag_names.cc:501
                                }
                                // cpp: html/html_tag_names.cc:502
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:503
                            104 => {
                                // cpp: html/html_tag_names.cc:504
                                if &data[2..5] == &[101, 97, 100][..] {
                                    // cpp: html/html_tag_names.cc:505
                                    return HTMLTag::kThead;
                                    // cpp: html/html_tag_names.cc:506
                                }
                                // cpp: html/html_tag_names.cc:507
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:508
                            105 => {
                                // cpp: html/html_tag_names.cc:509
                                if &data[2..5] == &[116, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:510
                                    return HTMLTag::kTitle;
                                    // cpp: html/html_tag_names.cc:511
                                }
                                // cpp: html/html_tag_names.cc:512
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:513
                            114 => {
                                // cpp: html/html_tag_names.cc:514
                                if &data[2..5] == &[97, 99, 107][..] {
                                    // cpp: html/html_tag_names.cc:515
                                    return HTMLTag::kTrack;
                                    // cpp: html/html_tag_names.cc:516
                                }
                                // cpp: html/html_tag_names.cc:517
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:518
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:519
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:520
                    118 => {
                        // cpp: html/html_tag_names.cc:521
                        if &data[1..5] == &[105, 100, 101, 111][..] {
                            // cpp: html/html_tag_names.cc:522
                            return HTMLTag::kVideo;
                            // cpp: html/html_tag_names.cc:523
                        }
                        // cpp: html/html_tag_names.cc:524
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:525
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:526
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:527
            6 => {
                // cpp: html/html_tag_names.cc:528
                match data[0] {
                    // cpp: html/html_tag_names.cc:529
                    97 => {
                        // cpp: html/html_tag_names.cc:530
                        if &data[1..6] == &[112, 112, 108, 101, 116][..] {
                            // cpp: html/html_tag_names.cc:531
                            return HTMLTag::kApplet;
                            // cpp: html/html_tag_names.cc:532
                        }
                        // cpp: html/html_tag_names.cc:533
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:534
                    98 => {
                        // cpp: html/html_tag_names.cc:535
                        if &data[1..6] == &[117, 116, 116, 111, 110][..] {
                            // cpp: html/html_tag_names.cc:536
                            return HTMLTag::kButton;
                            // cpp: html/html_tag_names.cc:537
                        }
                        // cpp: html/html_tag_names.cc:538
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:539
                    99 => {
                        // cpp: html/html_tag_names.cc:540
                        match data[1] {
                            // cpp: html/html_tag_names.cc:541
                            97 => {
                                // cpp: html/html_tag_names.cc:542
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:543
                                    109 => {
                                        // cpp: html/html_tag_names.cc:544
                                        if &data[3..6] == &[101, 114, 97][..] {
                                            // cpp: html/html_tag_names.cc:545-554
                                            return if false {
                                                HTMLTag::kCamera
                                            } else {
                                                HTMLTag::kUnknown
                                            };
                                            // cpp: html/html_tag_names.cc:555
                                        }
                                        // cpp: html/html_tag_names.cc:556
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:557
                                    110 => {
                                        // cpp: html/html_tag_names.cc:558
                                        if &data[3..6] == &[118, 97, 115][..] {
                                            // cpp: html/html_tag_names.cc:559
                                            return HTMLTag::kCanvas;
                                            // cpp: html/html_tag_names.cc:560
                                        }
                                        // cpp: html/html_tag_names.cc:561
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:562
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:563
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:564
                            101 => {
                                // cpp: html/html_tag_names.cc:565
                                if &data[2..6] == &[110, 116, 101, 114][..] {
                                    // cpp: html/html_tag_names.cc:566
                                    return HTMLTag::kCenter;
                                    // cpp: html/html_tag_names.cc:567
                                }
                                // cpp: html/html_tag_names.cc:568
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:569
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:570
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:571
                    100 => {
                        // cpp: html/html_tag_names.cc:572
                        if &data[1..6] == &[105, 97, 108, 111, 103][..] {
                            // cpp: html/html_tag_names.cc:573
                            return HTMLTag::kDialog;
                            // cpp: html/html_tag_names.cc:574
                        }
                        // cpp: html/html_tag_names.cc:575
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:576
                    102 => {
                        // cpp: html/html_tag_names.cc:577
                        match data[1] {
                            // cpp: html/html_tag_names.cc:578
                            105 => {
                                // cpp: html/html_tag_names.cc:579
                                if &data[2..6] == &[103, 117, 114, 101][..] {
                                    // cpp: html/html_tag_names.cc:580
                                    return HTMLTag::kFigure;
                                    // cpp: html/html_tag_names.cc:581
                                }
                                // cpp: html/html_tag_names.cc:582
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:583
                            111 => {
                                // cpp: html/html_tag_names.cc:584
                                if &data[2..6] == &[111, 116, 101, 114][..] {
                                    // cpp: html/html_tag_names.cc:585
                                    return HTMLTag::kFooter;
                                    // cpp: html/html_tag_names.cc:586
                                }
                                // cpp: html/html_tag_names.cc:587
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:588
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:589
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:590
                    104 => {
                        // cpp: html/html_tag_names.cc:591
                        match data[1] {
                            // cpp: html/html_tag_names.cc:592
                            101 => {
                                // cpp: html/html_tag_names.cc:593
                                if &data[2..6] == &[97, 100, 101, 114][..] {
                                    // cpp: html/html_tag_names.cc:594
                                    return HTMLTag::kHeader;
                                    // cpp: html/html_tag_names.cc:595
                                }
                                // cpp: html/html_tag_names.cc:596
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:597
                            103 => {
                                // cpp: html/html_tag_names.cc:598
                                if &data[2..6] == &[114, 111, 117, 112][..] {
                                    // cpp: html/html_tag_names.cc:599
                                    return HTMLTag::kHgroup;
                                    // cpp: html/html_tag_names.cc:600
                                }
                                // cpp: html/html_tag_names.cc:601
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:602
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:603
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:604
                    105 => {
                        // cpp: html/html_tag_names.cc:605
                        if &data[1..6] == &[102, 114, 97, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:606
                            return HTMLTag::kIFrame;
                            // cpp: html/html_tag_names.cc:607
                        }
                        // cpp: html/html_tag_names.cc:608
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:609
                    107 => {
                        // cpp: html/html_tag_names.cc:610
                        if &data[1..6] == &[101, 121, 103, 101, 110][..] {
                            // cpp: html/html_tag_names.cc:611
                            return HTMLTag::kKeygen;
                            // cpp: html/html_tag_names.cc:612
                        }
                        // cpp: html/html_tag_names.cc:613
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:614
                    108 => {
                        // cpp: html/html_tag_names.cc:615
                        if &data[1..6] == &[101, 103, 101, 110, 100][..] {
                            // cpp: html/html_tag_names.cc:616
                            return HTMLTag::kLegend;
                            // cpp: html/html_tag_names.cc:617
                        }
                        // cpp: html/html_tag_names.cc:618
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:619
                    111 => {
                        // cpp: html/html_tag_names.cc:620
                        match data[1] {
                            // cpp: html/html_tag_names.cc:621
                            98 => {
                                // cpp: html/html_tag_names.cc:622
                                if &data[2..6] == &[106, 101, 99, 116][..] {
                                    // cpp: html/html_tag_names.cc:623
                                    return HTMLTag::kObject;
                                    // cpp: html/html_tag_names.cc:624
                                }
                                // cpp: html/html_tag_names.cc:625
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:626
                            112 => {
                                // cpp: html/html_tag_names.cc:627
                                if &data[2..6] == &[116, 105, 111, 110][..] {
                                    // cpp: html/html_tag_names.cc:628
                                    return HTMLTag::kOption;
                                    // cpp: html/html_tag_names.cc:629
                                }
                                // cpp: html/html_tag_names.cc:630
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:631
                            117 => {
                                // cpp: html/html_tag_names.cc:632
                                if &data[2..6] == &[116, 112, 117, 116][..] {
                                    // cpp: html/html_tag_names.cc:633
                                    return HTMLTag::kOutput;
                                    // cpp: html/html_tag_names.cc:634
                                }
                                // cpp: html/html_tag_names.cc:635
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:636
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:637
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:638
                    115 => {
                        // cpp: html/html_tag_names.cc:639
                        match data[1] {
                            // cpp: html/html_tag_names.cc:640
                            99 => {
                                // cpp: html/html_tag_names.cc:641
                                if &data[2..6] == &[114, 105, 112, 116][..] {
                                    // cpp: html/html_tag_names.cc:642
                                    return HTMLTag::kScript;
                                    // cpp: html/html_tag_names.cc:643
                                }
                                // cpp: html/html_tag_names.cc:644
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:645
                            101 => {
                                // cpp: html/html_tag_names.cc:646
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:647
                                    97 => {
                                        // cpp: html/html_tag_names.cc:648
                                        if &data[3..6] == &[114, 99, 104][..] {
                                            // cpp: html/html_tag_names.cc:649
                                            return HTMLTag::kSearch;
                                            // cpp: html/html_tag_names.cc:650
                                        }
                                        // cpp: html/html_tag_names.cc:651
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:652
                                    108 => {
                                        // cpp: html/html_tag_names.cc:653
                                        if &data[3..6] == &[101, 99, 116][..] {
                                            // cpp: html/html_tag_names.cc:654
                                            return HTMLTag::kSelect;
                                            // cpp: html/html_tag_names.cc:655
                                        }
                                        // cpp: html/html_tag_names.cc:656
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:657
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:658
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:659
                            111 => {
                                // cpp: html/html_tag_names.cc:660
                                if &data[2..6] == &[117, 114, 99, 101][..] {
                                    // cpp: html/html_tag_names.cc:661
                                    return HTMLTag::kSource;
                                    // cpp: html/html_tag_names.cc:662
                                }
                                // cpp: html/html_tag_names.cc:663
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:664
                            116 => {
                                // cpp: html/html_tag_names.cc:665
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:666
                                    114 => {
                                        // cpp: html/html_tag_names.cc:667
                                        match data[3] {
                                            // cpp: html/html_tag_names.cc:668
                                            105 => {
                                                // cpp: html/html_tag_names.cc:669
                                                if &data[4..6] == &[107, 101][..] {
                                                    // cpp: html/html_tag_names.cc:670
                                                    return HTMLTag::kStrike;
                                                    // cpp: html/html_tag_names.cc:671
                                                }
                                                // cpp: html/html_tag_names.cc:672
                                                // C++ break ends this match arm.
                                            }
                                            // cpp: html/html_tag_names.cc:673
                                            111 => {
                                                // cpp: html/html_tag_names.cc:674
                                                if &data[4..6] == &[110, 103][..] {
                                                    // cpp: html/html_tag_names.cc:675
                                                    return HTMLTag::kStrong;
                                                    // cpp: html/html_tag_names.cc:676
                                                }
                                                // cpp: html/html_tag_names.cc:677
                                                // C++ break ends this match arm.
                                            }
                                            // cpp: html/html_tag_names.cc:678
                                            _ => {}
                                        }
                                        // cpp: html/html_tag_names.cc:679
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:680
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:681
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:682
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:683
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:684
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:685
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:686
            7 => {
                // cpp: html/html_tag_names.cc:687
                match data[0] {
                    // cpp: html/html_tag_names.cc:688
                    97 => {
                        // cpp: html/html_tag_names.cc:689
                        match data[1] {
                            // cpp: html/html_tag_names.cc:690
                            99 => {
                                // cpp: html/html_tag_names.cc:691
                                if &data[2..7] == &[114, 111, 110, 121, 109][..] {
                                    // cpp: html/html_tag_names.cc:692
                                    return HTMLTag::kAcronym;
                                    // cpp: html/html_tag_names.cc:693
                                }
                                // cpp: html/html_tag_names.cc:694
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:695
                            100 => {
                                // cpp: html/html_tag_names.cc:696
                                if &data[2..7] == &[100, 114, 101, 115, 115][..] {
                                    // cpp: html/html_tag_names.cc:697
                                    return HTMLTag::kAddress;
                                    // cpp: html/html_tag_names.cc:698
                                }
                                // cpp: html/html_tag_names.cc:699
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:700
                            114 => {
                                // cpp: html/html_tag_names.cc:701
                                if &data[2..7] == &[116, 105, 99, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:702
                                    return HTMLTag::kArticle;
                                    // cpp: html/html_tag_names.cc:703
                                }
                                // cpp: html/html_tag_names.cc:704
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:705
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:706
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:707
                    98 => {
                        // cpp: html/html_tag_names.cc:708
                        if &data[1..7] == &[103, 115, 111, 117, 110, 100][..] {
                            // cpp: html/html_tag_names.cc:709
                            return HTMLTag::kBgsound;
                            // cpp: html/html_tag_names.cc:710
                        }
                        // cpp: html/html_tag_names.cc:711
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:712
                    99 => {
                        // cpp: html/html_tag_names.cc:713
                        match data[1] {
                            // cpp: html/html_tag_names.cc:714
                            97 => {
                                // cpp: html/html_tag_names.cc:715
                                if &data[2..7] == &[112, 116, 105, 111, 110][..] {
                                    // cpp: html/html_tag_names.cc:716
                                    return HTMLTag::kCaption;
                                    // cpp: html/html_tag_names.cc:717
                                }
                                // cpp: html/html_tag_names.cc:718
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:719
                            111 => {
                                // cpp: html/html_tag_names.cc:720
                                if &data[2..7] == &[109, 109, 97, 110, 100][..] {
                                    // cpp: html/html_tag_names.cc:721
                                    return HTMLTag::kCommand;
                                    // cpp: html/html_tag_names.cc:722
                                }
                                // cpp: html/html_tag_names.cc:723
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:724
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:725
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:726
                    100 => {
                        // cpp: html/html_tag_names.cc:727
                        if &data[1..7] == &[101, 116, 97, 105, 108, 115][..] {
                            // cpp: html/html_tag_names.cc:728
                            return HTMLTag::kDetails;
                            // cpp: html/html_tag_names.cc:729
                        }
                        // cpp: html/html_tag_names.cc:730
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:731
                    105 => {
                        // cpp: html/html_tag_names.cc:732
                        if &data[1..7] == &[110, 115, 116, 97, 108, 108][..] {
                            // cpp: html/html_tag_names.cc:733
                            return HTMLTag::kInstallOrUnknown;
                            // cpp: html/html_tag_names.cc:734
                        }
                        // cpp: html/html_tag_names.cc:735
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:736
                    108 => {
                        // cpp: html/html_tag_names.cc:737
                        if &data[1..7] == &[105, 115, 116, 105, 110, 103][..] {
                            // cpp: html/html_tag_names.cc:738
                            return HTMLTag::kListing;
                            // cpp: html/html_tag_names.cc:739
                        }
                        // cpp: html/html_tag_names.cc:740
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:741
                    109 => {
                        // cpp: html/html_tag_names.cc:742
                        match data[1] {
                            // cpp: html/html_tag_names.cc:743
                            97 => {
                                // cpp: html/html_tag_names.cc:744
                                if &data[2..7] == &[114, 113, 117, 101, 101][..] {
                                    // cpp: html/html_tag_names.cc:745
                                    return HTMLTag::kMarquee;
                                    // cpp: html/html_tag_names.cc:746
                                }
                                // cpp: html/html_tag_names.cc:747
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:748
                            101 => {
                                // cpp: html/html_tag_names.cc:749
                                if &data[2..7] == &[110, 117, 98, 97, 114][..] {
                                    // cpp: html/html_tag_names.cc:750-759
                                    return if false {
                                        HTMLTag::kMenubar
                                    } else {
                                        HTMLTag::kUnknown
                                    };
                                    // cpp: html/html_tag_names.cc:760
                                }
                                // cpp: html/html_tag_names.cc:761
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:762
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:763
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:764
                    110 => {
                        // cpp: html/html_tag_names.cc:765
                        match data[1] {
                            // cpp: html/html_tag_names.cc:766
                            111 => {
                                // cpp: html/html_tag_names.cc:767
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:768
                                    101 => {
                                        // cpp: html/html_tag_names.cc:769
                                        if &data[3..7] == &[109, 98, 101, 100][..] {
                                            // cpp: html/html_tag_names.cc:770
                                            return HTMLTag::kNoembed;
                                            // cpp: html/html_tag_names.cc:771
                                        }
                                        // cpp: html/html_tag_names.cc:772
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:773
                                    108 => {
                                        // cpp: html/html_tag_names.cc:774
                                        if &data[3..7] == &[97, 121, 101, 114][..] {
                                            // cpp: html/html_tag_names.cc:775
                                            return HTMLTag::kNolayer;
                                            // cpp: html/html_tag_names.cc:776
                                        }
                                        // cpp: html/html_tag_names.cc:777
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:778
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:779
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:780
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:781
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:782
                    112 => {
                        // cpp: html/html_tag_names.cc:783
                        if &data[1..7] == &[105, 99, 116, 117, 114, 101][..] {
                            // cpp: html/html_tag_names.cc:784
                            return HTMLTag::kPicture;
                            // cpp: html/html_tag_names.cc:785
                        }
                        // cpp: html/html_tag_names.cc:786
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:787
                    115 => {
                        // cpp: html/html_tag_names.cc:788
                        match data[1] {
                            // cpp: html/html_tag_names.cc:789
                            101 => {
                                // cpp: html/html_tag_names.cc:790
                                if &data[2..7] == &[99, 116, 105, 111, 110][..] {
                                    // cpp: html/html_tag_names.cc:791
                                    return HTMLTag::kSection;
                                    // cpp: html/html_tag_names.cc:792
                                }
                                // cpp: html/html_tag_names.cc:793
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:794
                            117 => {
                                // cpp: html/html_tag_names.cc:795
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:796
                                    98 => {
                                        // cpp: html/html_tag_names.cc:797
                                        if &data[3..7] == &[109, 101, 110, 117][..] {
                                            // cpp: html/html_tag_names.cc:798-807
                                            return if false {
                                                HTMLTag::kSubmenu
                                            } else {
                                                HTMLTag::kUnknown
                                            };
                                            // cpp: html/html_tag_names.cc:808
                                        }
                                        // cpp: html/html_tag_names.cc:809
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:810
                                    109 => {
                                        // cpp: html/html_tag_names.cc:811
                                        if &data[3..7] == &[109, 97, 114, 121][..] {
                                            // cpp: html/html_tag_names.cc:812
                                            return HTMLTag::kSummary;
                                            // cpp: html/html_tag_names.cc:813
                                        }
                                        // cpp: html/html_tag_names.cc:814
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:815
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:816
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:817
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:818
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:819
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:820
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:821
            8 => {
                // cpp: html/html_tag_names.cc:822
                match data[0] {
                    // cpp: html/html_tag_names.cc:823
                    98 => {
                        // cpp: html/html_tag_names.cc:824
                        if &data[1..8] == &[97, 115, 101, 102, 111, 110, 116][..] {
                            // cpp: html/html_tag_names.cc:825
                            return HTMLTag::kBasefont;
                            // cpp: html/html_tag_names.cc:826
                        }
                        // cpp: html/html_tag_names.cc:827
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:828
                    99 => {
                        // cpp: html/html_tag_names.cc:829
                        if &data[1..8] == &[111, 108, 103, 114, 111, 117, 112][..] {
                            // cpp: html/html_tag_names.cc:830
                            return HTMLTag::kColgroup;
                            // cpp: html/html_tag_names.cc:831
                        }
                        // cpp: html/html_tag_names.cc:832
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:833
                    100 => {
                        // cpp: html/html_tag_names.cc:834
                        if &data[1..8] == &[97, 116, 97, 108, 105, 115, 116][..] {
                            // cpp: html/html_tag_names.cc:835
                            return HTMLTag::kDatalist;
                            // cpp: html/html_tag_names.cc:836
                        }
                        // cpp: html/html_tag_names.cc:837
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:838
                    102 => {
                        // cpp: html/html_tag_names.cc:839
                        match data[1] {
                            // cpp: html/html_tag_names.cc:840
                            105 => {
                                // cpp: html/html_tag_names.cc:841
                                if &data[2..8] == &[101, 108, 100, 115, 101, 116][..] {
                                    // cpp: html/html_tag_names.cc:842
                                    return HTMLTag::kFieldset;
                                    // cpp: html/html_tag_names.cc:843
                                }
                                // cpp: html/html_tag_names.cc:844
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:845
                            114 => {
                                // cpp: html/html_tag_names.cc:846
                                if &data[2..8] == &[97, 109, 101, 115, 101, 116][..] {
                                    // cpp: html/html_tag_names.cc:847
                                    return HTMLTag::kFrameset;
                                    // cpp: html/html_tag_names.cc:848
                                }
                                // cpp: html/html_tag_names.cc:849
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:850
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:851
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:852
                    109 => {
                        // cpp: html/html_tag_names.cc:853
                        match data[1] {
                            // cpp: html/html_tag_names.cc:854
                            101 => {
                                // cpp: html/html_tag_names.cc:855
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:856
                                    110 => {
                                        // cpp: html/html_tag_names.cc:857
                                        match data[3] {
                                            // cpp: html/html_tag_names.cc:858
                                            117 => {
                                                // cpp: html/html_tag_names.cc:859
                                                match data[4] {
                                                    // cpp: html/html_tag_names.cc:860
                                                    105 => {
                                                        // cpp: html/html_tag_names.cc:861
                                                        if &data[5..8] == &[116, 101, 109][..] {
                                                            // cpp: html/html_tag_names.cc:862-871
                                                            return if false {
                                                                HTMLTag::kMenuitem
                                                            } else {
                                                                HTMLTag::kUnknown
                                                            };
                                                            // cpp: html/html_tag_names.cc:872
                                                        }
                                                        // cpp: html/html_tag_names.cc:873
                                                        // C++ break ends this match arm.
                                                    }
                                                    // cpp: html/html_tag_names.cc:874
                                                    108 => {
                                                        // cpp: html/html_tag_names.cc:875
                                                        if &data[5..8] == &[105, 115, 116][..] {
                                                            // cpp: html/html_tag_names.cc:876-885
                                                            return if false {
                                                                HTMLTag::kMenulist
                                                            } else {
                                                                HTMLTag::kUnknown
                                                            };
                                                            // cpp: html/html_tag_names.cc:886
                                                        }
                                                        // cpp: html/html_tag_names.cc:887
                                                        // C++ break ends this match arm.
                                                    }
                                                    // cpp: html/html_tag_names.cc:888
                                                    _ => {}
                                                }
                                                // cpp: html/html_tag_names.cc:889
                                                // C++ break ends this match arm.
                                            }
                                            // cpp: html/html_tag_names.cc:890
                                            _ => {}
                                        }
                                        // cpp: html/html_tag_names.cc:891
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:892
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:893
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:894
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:895
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:896
                    110 => {
                        // cpp: html/html_tag_names.cc:897
                        match data[1] {
                            // cpp: html/html_tag_names.cc:898
                            111 => {
                                // cpp: html/html_tag_names.cc:899
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:900
                                    102 => {
                                        // cpp: html/html_tag_names.cc:901
                                        if &data[3..8] == &[114, 97, 109, 101, 115][..] {
                                            // cpp: html/html_tag_names.cc:902
                                            return HTMLTag::kNoframes;
                                            // cpp: html/html_tag_names.cc:903
                                        }
                                        // cpp: html/html_tag_names.cc:904
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:905
                                    115 => {
                                        // cpp: html/html_tag_names.cc:906
                                        if &data[3..8] == &[99, 114, 105, 112, 116][..] {
                                            // cpp: html/html_tag_names.cc:907
                                            return HTMLTag::kNoscript;
                                            // cpp: html/html_tag_names.cc:908
                                        }
                                        // cpp: html/html_tag_names.cc:909
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:910
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:911
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:912
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:913
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:914
                    111 => {
                        // cpp: html/html_tag_names.cc:915
                        if &data[1..8] == &[112, 116, 103, 114, 111, 117, 112][..] {
                            // cpp: html/html_tag_names.cc:916
                            return HTMLTag::kOptgroup;
                            // cpp: html/html_tag_names.cc:917
                        }
                        // cpp: html/html_tag_names.cc:918
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:919
                    112 => {
                        // cpp: html/html_tag_names.cc:920
                        if &data[1..8] == &[114, 111, 103, 114, 101, 115, 115][..] {
                            // cpp: html/html_tag_names.cc:921
                            return HTMLTag::kProgress;
                            // cpp: html/html_tag_names.cc:922
                        }
                        // cpp: html/html_tag_names.cc:923
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:924
                    116 => {
                        // cpp: html/html_tag_names.cc:925
                        match data[1] {
                            // cpp: html/html_tag_names.cc:926
                            101 => {
                                // cpp: html/html_tag_names.cc:927
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:928
                                    109 => {
                                        // cpp: html/html_tag_names.cc:929
                                        if &data[3..8] == &[112, 108, 97, 116, 101][..] {
                                            // cpp: html/html_tag_names.cc:930
                                            return HTMLTag::kTemplate;
                                            // cpp: html/html_tag_names.cc:931
                                        }
                                        // cpp: html/html_tag_names.cc:932
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:933
                                    120 => {
                                        // cpp: html/html_tag_names.cc:934
                                        if &data[3..8] == &[116, 97, 114, 101, 97][..] {
                                            // cpp: html/html_tag_names.cc:935
                                            return HTMLTag::kTextarea;
                                            // cpp: html/html_tag_names.cc:936
                                        }
                                        // cpp: html/html_tag_names.cc:937
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:938
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:939
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:940
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:941
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:942
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:943
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:944
            9 => {
                // cpp: html/html_tag_names.cc:945
                match data[0] {
                    // cpp: html/html_tag_names.cc:946
                    112 => {
                        // cpp: html/html_tag_names.cc:947
                        if &data[1..9] == &[108, 97, 105, 110, 116, 101, 120, 116][..] {
                            // cpp: html/html_tag_names.cc:948
                            return HTMLTag::kPlaintext;
                            // cpp: html/html_tag_names.cc:949
                        }
                        // cpp: html/html_tag_names.cc:950
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:951
                    117 => {
                        // cpp: html/html_tag_names.cc:952
                        if &data[1..9] == &[115, 101, 114, 109, 101, 100, 105, 97][..] {
                            // cpp: html/html_tag_names.cc:953
                            return HTMLTag::kUsermediaOrUnknown;
                            // cpp: html/html_tag_names.cc:954
                        }
                        // cpp: html/html_tag_names.cc:955
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:956
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:957
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:958
            10 => {
                // cpp: html/html_tag_names.cc:959
                match data[0] {
                    // cpp: html/html_tag_names.cc:960
                    98 => {
                        // cpp: html/html_tag_names.cc:961
                        if &data[1..10] == &[108, 111, 99, 107, 113, 117, 111, 116, 101][..] {
                            // cpp: html/html_tag_names.cc:962
                            return HTMLTag::kBlockquote;
                            // cpp: html/html_tag_names.cc:963
                        }
                        // cpp: html/html_tag_names.cc:964
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:965
                    99 => {
                        // cpp: html/html_tag_names.cc:966
                        if &data[1..10] == &[114, 101, 100, 101, 110, 116, 105, 97, 108][..] {
                            // cpp: html/html_tag_names.cc:967-976
                            return if false {
                                HTMLTag::kCredential
                            } else {
                                HTMLTag::kUnknown
                            };
                            // cpp: html/html_tag_names.cc:977
                        }
                        // cpp: html/html_tag_names.cc:978
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:979
                    102 => {
                        // cpp: html/html_tag_names.cc:980
                        if &data[1..10] == &[105, 103, 99, 97, 112, 116, 105, 111, 110][..] {
                            // cpp: html/html_tag_names.cc:981
                            return HTMLTag::kFigcaption;
                            // cpp: html/html_tag_names.cc:982
                        }
                        // cpp: html/html_tag_names.cc:983
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:984
                    109 => {
                        // cpp: html/html_tag_names.cc:985
                        if &data[1..10] == &[105, 99, 114, 111, 112, 104, 111, 110, 101][..] {
                            // cpp: html/html_tag_names.cc:986-995
                            return if false {
                                HTMLTag::kMicrophone
                            } else {
                                HTMLTag::kUnknown
                            };
                            // cpp: html/html_tag_names.cc:996
                        }
                        // cpp: html/html_tag_names.cc:997
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:998
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:999
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1000
            11 => {
                // cpp: html/html_tag_names.cc:1001
                match data[0] {
                    // cpp: html/html_tag_names.cc:1002
                    102 => {
                        // cpp: html/html_tag_names.cc:1003
                        if &data[1..11] == &[101, 110, 99, 101, 100, 102, 114, 97, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:1004
                            return HTMLTag::kFencedframeOrUnknown;
                            // cpp: html/html_tag_names.cc:1005
                        }
                        // cpp: html/html_tag_names.cc:1006
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1007
                    103 => {
                        // cpp: html/html_tag_names.cc:1008
                        if &data[1..11] == &[101, 111, 108, 111, 99, 97, 116, 105, 111, 110][..] {
                            // cpp: html/html_tag_names.cc:1009-1018
                            return if false {
                                HTMLTag::kGeolocation
                            } else {
                                HTMLTag::kUnknown
                            };
                            // cpp: html/html_tag_names.cc:1019
                        }
                        // cpp: html/html_tag_names.cc:1020
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1021
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1022
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1023
            15 => {
                // cpp: html/html_tag_names.cc:1024
                match data[0] {
                    // cpp: html/html_tag_names.cc:1025
                    115 => {
                        // cpp: html/html_tag_names.cc:1026
                        if &data[1..15]
                            == &[
                                101, 108, 101, 99, 116, 101, 100, 99, 111, 110, 116, 101, 110, 116,
                            ][..]
                        {
                            // cpp: html/html_tag_names.cc:1027
                            return HTMLTag::kSelectedcontent;
                            // cpp: html/html_tag_names.cc:1028
                        }
                        // cpp: html/html_tag_names.cc:1029
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1030
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1031
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1032
            _ => {}
        }
        // cpp: html/html_tag_names.cc:1034
        HTMLTag::kUnknown
        // cpp: html/html_tag_names.cc:1035
    }
}

// cpp: html/html_tag_names.cc:1037-1038
impl HtmlTagCodeUnit for u8 {
    fn lookup(span: &[Self]) -> HTMLTag {
        // cpp: html/html_tag_names.cc:1039-1040
        let data = span;
        let length = span.len();
        // cpp: html/html_tag_names.cc:1041
        debug_assert!(!data.as_ptr().is_null());
        // cpp: html/html_tag_names.cc:1042
        debug_assert_ne!(length, 0);
        // cpp: html/html_tag_names.cc:1043
        match length {
            // cpp: html/html_tag_names.cc:1044
            1 => {
                // cpp: html/html_tag_names.cc:1045
                match data[0] {
                    // cpp: html/html_tag_names.cc:1046
                    97 => {
                        // cpp: html/html_tag_names.cc:1047
                        return HTMLTag::kA;
                    }
                    // cpp: html/html_tag_names.cc:1048
                    98 => {
                        // cpp: html/html_tag_names.cc:1049
                        return HTMLTag::kB;
                    }
                    // cpp: html/html_tag_names.cc:1050
                    105 => {
                        // cpp: html/html_tag_names.cc:1051
                        return HTMLTag::kI;
                    }
                    // cpp: html/html_tag_names.cc:1052
                    112 => {
                        // cpp: html/html_tag_names.cc:1053
                        return HTMLTag::kP;
                    }
                    // cpp: html/html_tag_names.cc:1054
                    113 => {
                        // cpp: html/html_tag_names.cc:1055
                        return HTMLTag::kQ;
                    }
                    // cpp: html/html_tag_names.cc:1056
                    115 => {
                        // cpp: html/html_tag_names.cc:1057
                        return HTMLTag::kS;
                    }
                    // cpp: html/html_tag_names.cc:1058
                    117 => {
                        // cpp: html/html_tag_names.cc:1059
                        return HTMLTag::kU;
                    }
                    // cpp: html/html_tag_names.cc:1060
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1061
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1062
            2 => {
                // cpp: html/html_tag_names.cc:1063
                match data[0] {
                    // cpp: html/html_tag_names.cc:1064
                    98 => {
                        // cpp: html/html_tag_names.cc:1065
                        if data[1] == 114 {
                            // cpp: html/html_tag_names.cc:1066
                            return HTMLTag::kBr;
                            // cpp: html/html_tag_names.cc:1067
                        }
                        // cpp: html/html_tag_names.cc:1068
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1069
                    100 => {
                        // cpp: html/html_tag_names.cc:1070
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1071
                            100 => {
                                // cpp: html/html_tag_names.cc:1072
                                return HTMLTag::kDd;
                            }
                            // cpp: html/html_tag_names.cc:1073
                            108 => {
                                // cpp: html/html_tag_names.cc:1074
                                return HTMLTag::kDl;
                            }
                            // cpp: html/html_tag_names.cc:1075
                            116 => {
                                // cpp: html/html_tag_names.cc:1076
                                return HTMLTag::kDt;
                            }
                            // cpp: html/html_tag_names.cc:1077
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1078
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1079
                    101 => {
                        // cpp: html/html_tag_names.cc:1080
                        if data[1] == 109 {
                            // cpp: html/html_tag_names.cc:1081
                            return HTMLTag::kEm;
                            // cpp: html/html_tag_names.cc:1082
                        }
                        // cpp: html/html_tag_names.cc:1083
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1084
                    104 => {
                        // cpp: html/html_tag_names.cc:1085
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1086
                            49 => {
                                // cpp: html/html_tag_names.cc:1087
                                return HTMLTag::kH1;
                            }
                            // cpp: html/html_tag_names.cc:1088
                            50 => {
                                // cpp: html/html_tag_names.cc:1089
                                return HTMLTag::kH2;
                            }
                            // cpp: html/html_tag_names.cc:1090
                            51 => {
                                // cpp: html/html_tag_names.cc:1091
                                return HTMLTag::kH3;
                            }
                            // cpp: html/html_tag_names.cc:1092
                            52 => {
                                // cpp: html/html_tag_names.cc:1093
                                return HTMLTag::kH4;
                            }
                            // cpp: html/html_tag_names.cc:1094
                            53 => {
                                // cpp: html/html_tag_names.cc:1095
                                return HTMLTag::kH5;
                            }
                            // cpp: html/html_tag_names.cc:1096
                            54 => {
                                // cpp: html/html_tag_names.cc:1097
                                return HTMLTag::kH6;
                            }
                            // cpp: html/html_tag_names.cc:1098
                            114 => {
                                // cpp: html/html_tag_names.cc:1099
                                return HTMLTag::kHr;
                            }
                            // cpp: html/html_tag_names.cc:1100
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1101
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1102
                    108 => {
                        // cpp: html/html_tag_names.cc:1103
                        if data[1] == 105 {
                            // cpp: html/html_tag_names.cc:1104
                            return HTMLTag::kLi;
                            // cpp: html/html_tag_names.cc:1105
                        }
                        // cpp: html/html_tag_names.cc:1106
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1107
                    111 => {
                        // cpp: html/html_tag_names.cc:1108
                        if data[1] == 108 {
                            // cpp: html/html_tag_names.cc:1109
                            return HTMLTag::kOl;
                            // cpp: html/html_tag_names.cc:1110
                        }
                        // cpp: html/html_tag_names.cc:1111
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1112
                    114 => {
                        // cpp: html/html_tag_names.cc:1113
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1114
                            98 => {
                                // cpp: html/html_tag_names.cc:1115
                                return HTMLTag::kRb;
                            }
                            // cpp: html/html_tag_names.cc:1116
                            112 => {
                                // cpp: html/html_tag_names.cc:1117
                                return HTMLTag::kRp;
                            }
                            // cpp: html/html_tag_names.cc:1118
                            116 => {
                                // cpp: html/html_tag_names.cc:1119
                                return HTMLTag::kRt;
                            }
                            // cpp: html/html_tag_names.cc:1120
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1121
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1122
                    116 => {
                        // cpp: html/html_tag_names.cc:1123
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1124
                            100 => {
                                // cpp: html/html_tag_names.cc:1125
                                return HTMLTag::kTd;
                            }
                            // cpp: html/html_tag_names.cc:1126
                            104 => {
                                // cpp: html/html_tag_names.cc:1127
                                return HTMLTag::kTh;
                            }
                            // cpp: html/html_tag_names.cc:1128
                            114 => {
                                // cpp: html/html_tag_names.cc:1129
                                return HTMLTag::kTr;
                            }
                            // cpp: html/html_tag_names.cc:1130
                            116 => {
                                // cpp: html/html_tag_names.cc:1131
                                return HTMLTag::kTt;
                            }
                            // cpp: html/html_tag_names.cc:1132
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1133
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1134
                    117 => {
                        // cpp: html/html_tag_names.cc:1135
                        if data[1] == 108 {
                            // cpp: html/html_tag_names.cc:1136
                            return HTMLTag::kUl;
                            // cpp: html/html_tag_names.cc:1137
                        }
                        // cpp: html/html_tag_names.cc:1138
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1139
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1140
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1141
            3 => {
                // cpp: html/html_tag_names.cc:1142
                match data[0] {
                    // cpp: html/html_tag_names.cc:1143
                    98 => {
                        // cpp: html/html_tag_names.cc:1144
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1145
                            100 => {
                                // cpp: html/html_tag_names.cc:1146
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1147
                                    105 => {
                                        // cpp: html/html_tag_names.cc:1148
                                        return HTMLTag::kBdi;
                                    }
                                    // cpp: html/html_tag_names.cc:1149
                                    111 => {
                                        // cpp: html/html_tag_names.cc:1150
                                        return HTMLTag::kBdo;
                                    }
                                    // cpp: html/html_tag_names.cc:1151
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1152
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1153
                            105 => {
                                // cpp: html/html_tag_names.cc:1154
                                if data[2] == 103 {
                                    // cpp: html/html_tag_names.cc:1155
                                    return HTMLTag::kBig;
                                    // cpp: html/html_tag_names.cc:1156
                                }
                                // cpp: html/html_tag_names.cc:1157
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1158
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1159
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1160
                    99 => {
                        // cpp: html/html_tag_names.cc:1161
                        if &data[1..3] == &[111, 108][..] {
                            // cpp: html/html_tag_names.cc:1162
                            return HTMLTag::kCol;
                            // cpp: html/html_tag_names.cc:1163
                        }
                        // cpp: html/html_tag_names.cc:1164
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1165
                    100 => {
                        // cpp: html/html_tag_names.cc:1166
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1167
                            101 => {
                                // cpp: html/html_tag_names.cc:1168
                                if data[2] == 108 {
                                    // cpp: html/html_tag_names.cc:1169
                                    return HTMLTag::kDel;
                                    // cpp: html/html_tag_names.cc:1170
                                }
                                // cpp: html/html_tag_names.cc:1171
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1172
                            102 => {
                                // cpp: html/html_tag_names.cc:1173
                                if data[2] == 110 {
                                    // cpp: html/html_tag_names.cc:1174
                                    return HTMLTag::kDfn;
                                    // cpp: html/html_tag_names.cc:1175
                                }
                                // cpp: html/html_tag_names.cc:1176
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1177
                            105 => {
                                // cpp: html/html_tag_names.cc:1178
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1179
                                    114 => {
                                        // cpp: html/html_tag_names.cc:1180
                                        return HTMLTag::kDir;
                                    }
                                    // cpp: html/html_tag_names.cc:1181
                                    118 => {
                                        // cpp: html/html_tag_names.cc:1182
                                        return HTMLTag::kDiv;
                                    }
                                    // cpp: html/html_tag_names.cc:1183
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1184
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1185
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1186
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1187
                    105 => {
                        // cpp: html/html_tag_names.cc:1188
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1189
                            109 => {
                                // cpp: html/html_tag_names.cc:1190
                                if data[2] == 103 {
                                    // cpp: html/html_tag_names.cc:1191
                                    return HTMLTag::kImg;
                                    // cpp: html/html_tag_names.cc:1192
                                }
                                // cpp: html/html_tag_names.cc:1193
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1194
                            110 => {
                                // cpp: html/html_tag_names.cc:1195
                                if data[2] == 115 {
                                    // cpp: html/html_tag_names.cc:1196
                                    return HTMLTag::kIns;
                                    // cpp: html/html_tag_names.cc:1197
                                }
                                // cpp: html/html_tag_names.cc:1198
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1199
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1200
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1201
                    107 => {
                        // cpp: html/html_tag_names.cc:1202
                        if &data[1..3] == &[98, 100][..] {
                            // cpp: html/html_tag_names.cc:1203
                            return HTMLTag::kKbd;
                            // cpp: html/html_tag_names.cc:1204
                        }
                        // cpp: html/html_tag_names.cc:1205
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1206
                    109 => {
                        // cpp: html/html_tag_names.cc:1207
                        if &data[1..3] == &[97, 112][..] {
                            // cpp: html/html_tag_names.cc:1208
                            return HTMLTag::kMap;
                            // cpp: html/html_tag_names.cc:1209
                        }
                        // cpp: html/html_tag_names.cc:1210
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1211
                    110 => {
                        // cpp: html/html_tag_names.cc:1212
                        if &data[1..3] == &[97, 118][..] {
                            // cpp: html/html_tag_names.cc:1213
                            return HTMLTag::kNav;
                            // cpp: html/html_tag_names.cc:1214
                        }
                        // cpp: html/html_tag_names.cc:1215
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1216
                    112 => {
                        // cpp: html/html_tag_names.cc:1217
                        if &data[1..3] == &[114, 101][..] {
                            // cpp: html/html_tag_names.cc:1218
                            return HTMLTag::kPre;
                            // cpp: html/html_tag_names.cc:1219
                        }
                        // cpp: html/html_tag_names.cc:1220
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1221
                    114 => {
                        // cpp: html/html_tag_names.cc:1222
                        if &data[1..3] == &[116, 99][..] {
                            // cpp: html/html_tag_names.cc:1223
                            return HTMLTag::kRTC;
                            // cpp: html/html_tag_names.cc:1224
                        }
                        // cpp: html/html_tag_names.cc:1225
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1226
                    115 => {
                        // cpp: html/html_tag_names.cc:1227
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1228
                            117 => {
                                // cpp: html/html_tag_names.cc:1229
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1230
                                    98 => {
                                        // cpp: html/html_tag_names.cc:1231
                                        return HTMLTag::kSub;
                                    }
                                    // cpp: html/html_tag_names.cc:1232
                                    112 => {
                                        // cpp: html/html_tag_names.cc:1233
                                        return HTMLTag::kSup;
                                    }
                                    // cpp: html/html_tag_names.cc:1234
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1235
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1236
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1237
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1238
                    118 => {
                        // cpp: html/html_tag_names.cc:1239
                        if &data[1..3] == &[97, 114][..] {
                            // cpp: html/html_tag_names.cc:1240
                            return HTMLTag::kVar;
                            // cpp: html/html_tag_names.cc:1241
                        }
                        // cpp: html/html_tag_names.cc:1242
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1243
                    119 => {
                        // cpp: html/html_tag_names.cc:1244
                        if &data[1..3] == &[98, 114][..] {
                            // cpp: html/html_tag_names.cc:1245
                            return HTMLTag::kWbr;
                            // cpp: html/html_tag_names.cc:1246
                        }
                        // cpp: html/html_tag_names.cc:1247
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1248
                    120 => {
                        // cpp: html/html_tag_names.cc:1249
                        if &data[1..3] == &[109, 112][..] {
                            // cpp: html/html_tag_names.cc:1250
                            return HTMLTag::kXmp;
                            // cpp: html/html_tag_names.cc:1251
                        }
                        // cpp: html/html_tag_names.cc:1252
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1253
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1254
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1255
            4 => {
                // cpp: html/html_tag_names.cc:1256
                match data[0] {
                    // cpp: html/html_tag_names.cc:1257
                    97 => {
                        // cpp: html/html_tag_names.cc:1258
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1259
                            98 => {
                                // cpp: html/html_tag_names.cc:1260
                                if &data[2..4] == &[98, 114][..] {
                                    // cpp: html/html_tag_names.cc:1261
                                    return HTMLTag::kAbbr;
                                    // cpp: html/html_tag_names.cc:1262
                                }
                                // cpp: html/html_tag_names.cc:1263
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1264
                            114 => {
                                // cpp: html/html_tag_names.cc:1265
                                if &data[2..4] == &[101, 97][..] {
                                    // cpp: html/html_tag_names.cc:1266
                                    return HTMLTag::kArea;
                                    // cpp: html/html_tag_names.cc:1267
                                }
                                // cpp: html/html_tag_names.cc:1268
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1269
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1270
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1271
                    98 => {
                        // cpp: html/html_tag_names.cc:1272
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1273
                            97 => {
                                // cpp: html/html_tag_names.cc:1274
                                if &data[2..4] == &[115, 101][..] {
                                    // cpp: html/html_tag_names.cc:1275
                                    return HTMLTag::kBase;
                                    // cpp: html/html_tag_names.cc:1276
                                }
                                // cpp: html/html_tag_names.cc:1277
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1278
                            111 => {
                                // cpp: html/html_tag_names.cc:1279
                                if &data[2..4] == &[100, 121][..] {
                                    // cpp: html/html_tag_names.cc:1280
                                    return HTMLTag::kBody;
                                    // cpp: html/html_tag_names.cc:1281
                                }
                                // cpp: html/html_tag_names.cc:1282
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1283
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1284
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1285
                    99 => {
                        // cpp: html/html_tag_names.cc:1286
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1287
                            105 => {
                                // cpp: html/html_tag_names.cc:1288
                                if &data[2..4] == &[116, 101][..] {
                                    // cpp: html/html_tag_names.cc:1289
                                    return HTMLTag::kCite;
                                    // cpp: html/html_tag_names.cc:1290
                                }
                                // cpp: html/html_tag_names.cc:1291
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1292
                            111 => {
                                // cpp: html/html_tag_names.cc:1293
                                if &data[2..4] == &[100, 101][..] {
                                    // cpp: html/html_tag_names.cc:1294
                                    return HTMLTag::kCode;
                                    // cpp: html/html_tag_names.cc:1295
                                }
                                // cpp: html/html_tag_names.cc:1296
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1297
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1298
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1299
                    100 => {
                        // cpp: html/html_tag_names.cc:1300
                        if &data[1..4] == &[97, 116, 97][..] {
                            // cpp: html/html_tag_names.cc:1301
                            return HTMLTag::kData;
                            // cpp: html/html_tag_names.cc:1302
                        }
                        // cpp: html/html_tag_names.cc:1303
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1304
                    102 => {
                        // cpp: html/html_tag_names.cc:1305
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1306
                            111 => {
                                // cpp: html/html_tag_names.cc:1307
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1308
                                    110 => {
                                        // cpp: html/html_tag_names.cc:1309
                                        if data[3] == 116 {
                                            // cpp: html/html_tag_names.cc:1310
                                            return HTMLTag::kFont;
                                            // cpp: html/html_tag_names.cc:1311
                                        }
                                        // cpp: html/html_tag_names.cc:1312
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1313
                                    114 => {
                                        // cpp: html/html_tag_names.cc:1314
                                        if data[3] == 109 {
                                            // cpp: html/html_tag_names.cc:1315
                                            return HTMLTag::kForm;
                                            // cpp: html/html_tag_names.cc:1316
                                        }
                                        // cpp: html/html_tag_names.cc:1317
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1318
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1319
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1320
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1321
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1322
                    104 => {
                        // cpp: html/html_tag_names.cc:1323
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1324
                            101 => {
                                // cpp: html/html_tag_names.cc:1325
                                if &data[2..4] == &[97, 100][..] {
                                    // cpp: html/html_tag_names.cc:1326
                                    return HTMLTag::kHead;
                                    // cpp: html/html_tag_names.cc:1327
                                }
                                // cpp: html/html_tag_names.cc:1328
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1329
                            116 => {
                                // cpp: html/html_tag_names.cc:1330
                                if &data[2..4] == &[109, 108][..] {
                                    // cpp: html/html_tag_names.cc:1331
                                    return HTMLTag::kHTML;
                                    // cpp: html/html_tag_names.cc:1332
                                }
                                // cpp: html/html_tag_names.cc:1333
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1334
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1335
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1336
                    108 => {
                        // cpp: html/html_tag_names.cc:1337
                        if &data[1..4] == &[105, 110, 107][..] {
                            // cpp: html/html_tag_names.cc:1338
                            return HTMLTag::kLink;
                            // cpp: html/html_tag_names.cc:1339
                        }
                        // cpp: html/html_tag_names.cc:1340
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1341
                    109 => {
                        // cpp: html/html_tag_names.cc:1342
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1343
                            97 => {
                                // cpp: html/html_tag_names.cc:1344
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1345
                                    105 => {
                                        // cpp: html/html_tag_names.cc:1346
                                        if data[3] == 110 {
                                            // cpp: html/html_tag_names.cc:1347
                                            return HTMLTag::kMain;
                                            // cpp: html/html_tag_names.cc:1348
                                        }
                                        // cpp: html/html_tag_names.cc:1349
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1350
                                    114 => {
                                        // cpp: html/html_tag_names.cc:1351
                                        if data[3] == 107 {
                                            // cpp: html/html_tag_names.cc:1352
                                            return HTMLTag::kMark;
                                            // cpp: html/html_tag_names.cc:1353
                                        }
                                        // cpp: html/html_tag_names.cc:1354
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1355
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1356
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1357
                            101 => {
                                // cpp: html/html_tag_names.cc:1358
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1359
                                    110 => {
                                        // cpp: html/html_tag_names.cc:1360
                                        if data[3] == 117 {
                                            // cpp: html/html_tag_names.cc:1361
                                            return HTMLTag::kMenu;
                                            // cpp: html/html_tag_names.cc:1362
                                        }
                                        // cpp: html/html_tag_names.cc:1363
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1364
                                    116 => {
                                        // cpp: html/html_tag_names.cc:1365
                                        if data[3] == 97 {
                                            // cpp: html/html_tag_names.cc:1366
                                            return HTMLTag::kMeta;
                                            // cpp: html/html_tag_names.cc:1367
                                        }
                                        // cpp: html/html_tag_names.cc:1368
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1369
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1370
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1371
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1372
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1373
                    110 => {
                        // cpp: html/html_tag_names.cc:1374
                        if &data[1..4] == &[111, 98, 114][..] {
                            // cpp: html/html_tag_names.cc:1375
                            return HTMLTag::kNobr;
                            // cpp: html/html_tag_names.cc:1376
                        }
                        // cpp: html/html_tag_names.cc:1377
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1378
                    114 => {
                        // cpp: html/html_tag_names.cc:1379
                        if &data[1..4] == &[117, 98, 121][..] {
                            // cpp: html/html_tag_names.cc:1380
                            return HTMLTag::kRuby;
                            // cpp: html/html_tag_names.cc:1381
                        }
                        // cpp: html/html_tag_names.cc:1382
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1383
                    115 => {
                        // cpp: html/html_tag_names.cc:1384
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1385
                            97 => {
                                // cpp: html/html_tag_names.cc:1386
                                if &data[2..4] == &[109, 112][..] {
                                    // cpp: html/html_tag_names.cc:1387
                                    return HTMLTag::kSamp;
                                    // cpp: html/html_tag_names.cc:1388
                                }
                                // cpp: html/html_tag_names.cc:1389
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1390
                            108 => {
                                // cpp: html/html_tag_names.cc:1391
                                if &data[2..4] == &[111, 116][..] {
                                    // cpp: html/html_tag_names.cc:1392
                                    return HTMLTag::kSlot;
                                    // cpp: html/html_tag_names.cc:1393
                                }
                                // cpp: html/html_tag_names.cc:1394
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1395
                            112 => {
                                // cpp: html/html_tag_names.cc:1396
                                if &data[2..4] == &[97, 110][..] {
                                    // cpp: html/html_tag_names.cc:1397
                                    return HTMLTag::kSpan;
                                    // cpp: html/html_tag_names.cc:1398
                                }
                                // cpp: html/html_tag_names.cc:1399
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1400
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1401
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1402
                    116 => {
                        // cpp: html/html_tag_names.cc:1403
                        if &data[1..4] == &[105, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:1404
                            return HTMLTag::kTime;
                            // cpp: html/html_tag_names.cc:1405
                        }
                        // cpp: html/html_tag_names.cc:1406
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1407
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1408
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1409
            5 => {
                // cpp: html/html_tag_names.cc:1410
                match data[0] {
                    // cpp: html/html_tag_names.cc:1411
                    97 => {
                        // cpp: html/html_tag_names.cc:1412
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1413
                            115 => {
                                // cpp: html/html_tag_names.cc:1414
                                if &data[2..5] == &[105, 100, 101][..] {
                                    // cpp: html/html_tag_names.cc:1415
                                    return HTMLTag::kAside;
                                    // cpp: html/html_tag_names.cc:1416
                                }
                                // cpp: html/html_tag_names.cc:1417
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1418
                            117 => {
                                // cpp: html/html_tag_names.cc:1419
                                if &data[2..5] == &[100, 105, 111][..] {
                                    // cpp: html/html_tag_names.cc:1420
                                    return HTMLTag::kAudio;
                                    // cpp: html/html_tag_names.cc:1421
                                }
                                // cpp: html/html_tag_names.cc:1422
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1423
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1424
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1425
                    101 => {
                        // cpp: html/html_tag_names.cc:1426
                        if &data[1..5] == &[109, 98, 101, 100][..] {
                            // cpp: html/html_tag_names.cc:1427
                            return HTMLTag::kEmbed;
                            // cpp: html/html_tag_names.cc:1428
                        }
                        // cpp: html/html_tag_names.cc:1429
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1430
                    102 => {
                        // cpp: html/html_tag_names.cc:1431
                        if &data[1..5] == &[114, 97, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:1432
                            return HTMLTag::kFrame;
                            // cpp: html/html_tag_names.cc:1433
                        }
                        // cpp: html/html_tag_names.cc:1434
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1435
                    105 => {
                        // cpp: html/html_tag_names.cc:1436
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1437
                            109 => {
                                // cpp: html/html_tag_names.cc:1438
                                if &data[2..5] == &[97, 103, 101][..] {
                                    // cpp: html/html_tag_names.cc:1439
                                    return HTMLTag::kImage;
                                    // cpp: html/html_tag_names.cc:1440
                                }
                                // cpp: html/html_tag_names.cc:1441
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1442
                            110 => {
                                // cpp: html/html_tag_names.cc:1443
                                if &data[2..5] == &[112, 117, 116][..] {
                                    // cpp: html/html_tag_names.cc:1444
                                    return HTMLTag::kInput;
                                    // cpp: html/html_tag_names.cc:1445
                                }
                                // cpp: html/html_tag_names.cc:1446
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1447
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1448
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1449
                    108 => {
                        // cpp: html/html_tag_names.cc:1450
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1451
                            97 => {
                                // cpp: html/html_tag_names.cc:1452
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1453
                                    98 => {
                                        // cpp: html/html_tag_names.cc:1454
                                        if &data[3..5] == &[101, 108][..] {
                                            // cpp: html/html_tag_names.cc:1455
                                            return HTMLTag::kLabel;
                                            // cpp: html/html_tag_names.cc:1456
                                        }
                                        // cpp: html/html_tag_names.cc:1457
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1458
                                    121 => {
                                        // cpp: html/html_tag_names.cc:1459
                                        if &data[3..5] == &[101, 114][..] {
                                            // cpp: html/html_tag_names.cc:1460
                                            return HTMLTag::kLayer;
                                            // cpp: html/html_tag_names.cc:1461
                                        }
                                        // cpp: html/html_tag_names.cc:1462
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1463
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1464
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1465
                            111 => {
                                // cpp: html/html_tag_names.cc:1466
                                if &data[2..5] == &[103, 105, 110][..] {
                                    // cpp: html/html_tag_names.cc:1467-1476
                                    return if false {
                                        HTMLTag::kLogin
                                    } else {
                                        HTMLTag::kUnknown
                                    };
                                    // cpp: html/html_tag_names.cc:1477
                                }
                                // cpp: html/html_tag_names.cc:1478
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1479
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1480
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1481
                    109 => {
                        // cpp: html/html_tag_names.cc:1482
                        if &data[1..5] == &[101, 116, 101, 114][..] {
                            // cpp: html/html_tag_names.cc:1483
                            return HTMLTag::kMeter;
                            // cpp: html/html_tag_names.cc:1484
                        }
                        // cpp: html/html_tag_names.cc:1485
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1486
                    112 => {
                        // cpp: html/html_tag_names.cc:1487
                        if &data[1..5] == &[97, 114, 97, 109][..] {
                            // cpp: html/html_tag_names.cc:1488
                            return HTMLTag::kParam;
                            // cpp: html/html_tag_names.cc:1489
                        }
                        // cpp: html/html_tag_names.cc:1490
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1491
                    115 => {
                        // cpp: html/html_tag_names.cc:1492
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1493
                            109 => {
                                // cpp: html/html_tag_names.cc:1494
                                if &data[2..5] == &[97, 108, 108][..] {
                                    // cpp: html/html_tag_names.cc:1495
                                    return HTMLTag::kSmall;
                                    // cpp: html/html_tag_names.cc:1496
                                }
                                // cpp: html/html_tag_names.cc:1497
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1498
                            116 => {
                                // cpp: html/html_tag_names.cc:1499
                                if &data[2..5] == &[121, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:1500
                                    return HTMLTag::kStyle;
                                    // cpp: html/html_tag_names.cc:1501
                                }
                                // cpp: html/html_tag_names.cc:1502
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1503
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1504
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1505
                    116 => {
                        // cpp: html/html_tag_names.cc:1506
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1507
                            97 => {
                                // cpp: html/html_tag_names.cc:1508
                                if &data[2..5] == &[98, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:1509
                                    return HTMLTag::kTable;
                                    // cpp: html/html_tag_names.cc:1510
                                }
                                // cpp: html/html_tag_names.cc:1511
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1512
                            98 => {
                                // cpp: html/html_tag_names.cc:1513
                                if &data[2..5] == &[111, 100, 121][..] {
                                    // cpp: html/html_tag_names.cc:1514
                                    return HTMLTag::kTbody;
                                    // cpp: html/html_tag_names.cc:1515
                                }
                                // cpp: html/html_tag_names.cc:1516
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1517
                            102 => {
                                // cpp: html/html_tag_names.cc:1518
                                if &data[2..5] == &[111, 111, 116][..] {
                                    // cpp: html/html_tag_names.cc:1519
                                    return HTMLTag::kTfoot;
                                    // cpp: html/html_tag_names.cc:1520
                                }
                                // cpp: html/html_tag_names.cc:1521
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1522
                            104 => {
                                // cpp: html/html_tag_names.cc:1523
                                if &data[2..5] == &[101, 97, 100][..] {
                                    // cpp: html/html_tag_names.cc:1524
                                    return HTMLTag::kThead;
                                    // cpp: html/html_tag_names.cc:1525
                                }
                                // cpp: html/html_tag_names.cc:1526
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1527
                            105 => {
                                // cpp: html/html_tag_names.cc:1528
                                if &data[2..5] == &[116, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:1529
                                    return HTMLTag::kTitle;
                                    // cpp: html/html_tag_names.cc:1530
                                }
                                // cpp: html/html_tag_names.cc:1531
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1532
                            114 => {
                                // cpp: html/html_tag_names.cc:1533
                                if &data[2..5] == &[97, 99, 107][..] {
                                    // cpp: html/html_tag_names.cc:1534
                                    return HTMLTag::kTrack;
                                    // cpp: html/html_tag_names.cc:1535
                                }
                                // cpp: html/html_tag_names.cc:1536
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1537
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1538
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1539
                    118 => {
                        // cpp: html/html_tag_names.cc:1540
                        if &data[1..5] == &[105, 100, 101, 111][..] {
                            // cpp: html/html_tag_names.cc:1541
                            return HTMLTag::kVideo;
                            // cpp: html/html_tag_names.cc:1542
                        }
                        // cpp: html/html_tag_names.cc:1543
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1544
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1545
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1546
            6 => {
                // cpp: html/html_tag_names.cc:1547
                match data[0] {
                    // cpp: html/html_tag_names.cc:1548
                    97 => {
                        // cpp: html/html_tag_names.cc:1549
                        if &data[1..6] == &[112, 112, 108, 101, 116][..] {
                            // cpp: html/html_tag_names.cc:1550
                            return HTMLTag::kApplet;
                            // cpp: html/html_tag_names.cc:1551
                        }
                        // cpp: html/html_tag_names.cc:1552
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1553
                    98 => {
                        // cpp: html/html_tag_names.cc:1554
                        if &data[1..6] == &[117, 116, 116, 111, 110][..] {
                            // cpp: html/html_tag_names.cc:1555
                            return HTMLTag::kButton;
                            // cpp: html/html_tag_names.cc:1556
                        }
                        // cpp: html/html_tag_names.cc:1557
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1558
                    99 => {
                        // cpp: html/html_tag_names.cc:1559
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1560
                            97 => {
                                // cpp: html/html_tag_names.cc:1561
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1562
                                    109 => {
                                        // cpp: html/html_tag_names.cc:1563
                                        if &data[3..6] == &[101, 114, 97][..] {
                                            // cpp: html/html_tag_names.cc:1564-1573
                                            return if false {
                                                HTMLTag::kCamera
                                            } else {
                                                HTMLTag::kUnknown
                                            };
                                            // cpp: html/html_tag_names.cc:1574
                                        }
                                        // cpp: html/html_tag_names.cc:1575
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1576
                                    110 => {
                                        // cpp: html/html_tag_names.cc:1577
                                        if &data[3..6] == &[118, 97, 115][..] {
                                            // cpp: html/html_tag_names.cc:1578
                                            return HTMLTag::kCanvas;
                                            // cpp: html/html_tag_names.cc:1579
                                        }
                                        // cpp: html/html_tag_names.cc:1580
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1581
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1582
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1583
                            101 => {
                                // cpp: html/html_tag_names.cc:1584
                                if &data[2..6] == &[110, 116, 101, 114][..] {
                                    // cpp: html/html_tag_names.cc:1585
                                    return HTMLTag::kCenter;
                                    // cpp: html/html_tag_names.cc:1586
                                }
                                // cpp: html/html_tag_names.cc:1587
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1588
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1589
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1590
                    100 => {
                        // cpp: html/html_tag_names.cc:1591
                        if &data[1..6] == &[105, 97, 108, 111, 103][..] {
                            // cpp: html/html_tag_names.cc:1592
                            return HTMLTag::kDialog;
                            // cpp: html/html_tag_names.cc:1593
                        }
                        // cpp: html/html_tag_names.cc:1594
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1595
                    102 => {
                        // cpp: html/html_tag_names.cc:1596
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1597
                            105 => {
                                // cpp: html/html_tag_names.cc:1598
                                if &data[2..6] == &[103, 117, 114, 101][..] {
                                    // cpp: html/html_tag_names.cc:1599
                                    return HTMLTag::kFigure;
                                    // cpp: html/html_tag_names.cc:1600
                                }
                                // cpp: html/html_tag_names.cc:1601
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1602
                            111 => {
                                // cpp: html/html_tag_names.cc:1603
                                if &data[2..6] == &[111, 116, 101, 114][..] {
                                    // cpp: html/html_tag_names.cc:1604
                                    return HTMLTag::kFooter;
                                    // cpp: html/html_tag_names.cc:1605
                                }
                                // cpp: html/html_tag_names.cc:1606
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1607
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1608
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1609
                    104 => {
                        // cpp: html/html_tag_names.cc:1610
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1611
                            101 => {
                                // cpp: html/html_tag_names.cc:1612
                                if &data[2..6] == &[97, 100, 101, 114][..] {
                                    // cpp: html/html_tag_names.cc:1613
                                    return HTMLTag::kHeader;
                                    // cpp: html/html_tag_names.cc:1614
                                }
                                // cpp: html/html_tag_names.cc:1615
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1616
                            103 => {
                                // cpp: html/html_tag_names.cc:1617
                                if &data[2..6] == &[114, 111, 117, 112][..] {
                                    // cpp: html/html_tag_names.cc:1618
                                    return HTMLTag::kHgroup;
                                    // cpp: html/html_tag_names.cc:1619
                                }
                                // cpp: html/html_tag_names.cc:1620
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1621
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1622
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1623
                    105 => {
                        // cpp: html/html_tag_names.cc:1624
                        if &data[1..6] == &[102, 114, 97, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:1625
                            return HTMLTag::kIFrame;
                            // cpp: html/html_tag_names.cc:1626
                        }
                        // cpp: html/html_tag_names.cc:1627
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1628
                    107 => {
                        // cpp: html/html_tag_names.cc:1629
                        if &data[1..6] == &[101, 121, 103, 101, 110][..] {
                            // cpp: html/html_tag_names.cc:1630
                            return HTMLTag::kKeygen;
                            // cpp: html/html_tag_names.cc:1631
                        }
                        // cpp: html/html_tag_names.cc:1632
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1633
                    108 => {
                        // cpp: html/html_tag_names.cc:1634
                        if &data[1..6] == &[101, 103, 101, 110, 100][..] {
                            // cpp: html/html_tag_names.cc:1635
                            return HTMLTag::kLegend;
                            // cpp: html/html_tag_names.cc:1636
                        }
                        // cpp: html/html_tag_names.cc:1637
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1638
                    111 => {
                        // cpp: html/html_tag_names.cc:1639
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1640
                            98 => {
                                // cpp: html/html_tag_names.cc:1641
                                if &data[2..6] == &[106, 101, 99, 116][..] {
                                    // cpp: html/html_tag_names.cc:1642
                                    return HTMLTag::kObject;
                                    // cpp: html/html_tag_names.cc:1643
                                }
                                // cpp: html/html_tag_names.cc:1644
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1645
                            112 => {
                                // cpp: html/html_tag_names.cc:1646
                                if &data[2..6] == &[116, 105, 111, 110][..] {
                                    // cpp: html/html_tag_names.cc:1647
                                    return HTMLTag::kOption;
                                    // cpp: html/html_tag_names.cc:1648
                                }
                                // cpp: html/html_tag_names.cc:1649
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1650
                            117 => {
                                // cpp: html/html_tag_names.cc:1651
                                if &data[2..6] == &[116, 112, 117, 116][..] {
                                    // cpp: html/html_tag_names.cc:1652
                                    return HTMLTag::kOutput;
                                    // cpp: html/html_tag_names.cc:1653
                                }
                                // cpp: html/html_tag_names.cc:1654
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1655
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1656
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1657
                    115 => {
                        // cpp: html/html_tag_names.cc:1658
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1659
                            99 => {
                                // cpp: html/html_tag_names.cc:1660
                                if &data[2..6] == &[114, 105, 112, 116][..] {
                                    // cpp: html/html_tag_names.cc:1661
                                    return HTMLTag::kScript;
                                    // cpp: html/html_tag_names.cc:1662
                                }
                                // cpp: html/html_tag_names.cc:1663
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1664
                            101 => {
                                // cpp: html/html_tag_names.cc:1665
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1666
                                    97 => {
                                        // cpp: html/html_tag_names.cc:1667
                                        if &data[3..6] == &[114, 99, 104][..] {
                                            // cpp: html/html_tag_names.cc:1668
                                            return HTMLTag::kSearch;
                                            // cpp: html/html_tag_names.cc:1669
                                        }
                                        // cpp: html/html_tag_names.cc:1670
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1671
                                    108 => {
                                        // cpp: html/html_tag_names.cc:1672
                                        if &data[3..6] == &[101, 99, 116][..] {
                                            // cpp: html/html_tag_names.cc:1673
                                            return HTMLTag::kSelect;
                                            // cpp: html/html_tag_names.cc:1674
                                        }
                                        // cpp: html/html_tag_names.cc:1675
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1676
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1677
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1678
                            111 => {
                                // cpp: html/html_tag_names.cc:1679
                                if &data[2..6] == &[117, 114, 99, 101][..] {
                                    // cpp: html/html_tag_names.cc:1680
                                    return HTMLTag::kSource;
                                    // cpp: html/html_tag_names.cc:1681
                                }
                                // cpp: html/html_tag_names.cc:1682
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1683
                            116 => {
                                // cpp: html/html_tag_names.cc:1684
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1685
                                    114 => {
                                        // cpp: html/html_tag_names.cc:1686
                                        match data[3] {
                                            // cpp: html/html_tag_names.cc:1687
                                            105 => {
                                                // cpp: html/html_tag_names.cc:1688
                                                if &data[4..6] == &[107, 101][..] {
                                                    // cpp: html/html_tag_names.cc:1689
                                                    return HTMLTag::kStrike;
                                                    // cpp: html/html_tag_names.cc:1690
                                                }
                                                // cpp: html/html_tag_names.cc:1691
                                                // C++ break ends this match arm.
                                            }
                                            // cpp: html/html_tag_names.cc:1692
                                            111 => {
                                                // cpp: html/html_tag_names.cc:1693
                                                if &data[4..6] == &[110, 103][..] {
                                                    // cpp: html/html_tag_names.cc:1694
                                                    return HTMLTag::kStrong;
                                                    // cpp: html/html_tag_names.cc:1695
                                                }
                                                // cpp: html/html_tag_names.cc:1696
                                                // C++ break ends this match arm.
                                            }
                                            // cpp: html/html_tag_names.cc:1697
                                            _ => {}
                                        }
                                        // cpp: html/html_tag_names.cc:1698
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1699
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1700
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1701
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1702
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1703
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1704
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1705
            7 => {
                // cpp: html/html_tag_names.cc:1706
                match data[0] {
                    // cpp: html/html_tag_names.cc:1707
                    97 => {
                        // cpp: html/html_tag_names.cc:1708
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1709
                            99 => {
                                // cpp: html/html_tag_names.cc:1710
                                if &data[2..7] == &[114, 111, 110, 121, 109][..] {
                                    // cpp: html/html_tag_names.cc:1711
                                    return HTMLTag::kAcronym;
                                    // cpp: html/html_tag_names.cc:1712
                                }
                                // cpp: html/html_tag_names.cc:1713
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1714
                            100 => {
                                // cpp: html/html_tag_names.cc:1715
                                if &data[2..7] == &[100, 114, 101, 115, 115][..] {
                                    // cpp: html/html_tag_names.cc:1716
                                    return HTMLTag::kAddress;
                                    // cpp: html/html_tag_names.cc:1717
                                }
                                // cpp: html/html_tag_names.cc:1718
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1719
                            114 => {
                                // cpp: html/html_tag_names.cc:1720
                                if &data[2..7] == &[116, 105, 99, 108, 101][..] {
                                    // cpp: html/html_tag_names.cc:1721
                                    return HTMLTag::kArticle;
                                    // cpp: html/html_tag_names.cc:1722
                                }
                                // cpp: html/html_tag_names.cc:1723
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1724
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1725
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1726
                    98 => {
                        // cpp: html/html_tag_names.cc:1727
                        if &data[1..7] == &[103, 115, 111, 117, 110, 100][..] {
                            // cpp: html/html_tag_names.cc:1728
                            return HTMLTag::kBgsound;
                            // cpp: html/html_tag_names.cc:1729
                        }
                        // cpp: html/html_tag_names.cc:1730
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1731
                    99 => {
                        // cpp: html/html_tag_names.cc:1732
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1733
                            97 => {
                                // cpp: html/html_tag_names.cc:1734
                                if &data[2..7] == &[112, 116, 105, 111, 110][..] {
                                    // cpp: html/html_tag_names.cc:1735
                                    return HTMLTag::kCaption;
                                    // cpp: html/html_tag_names.cc:1736
                                }
                                // cpp: html/html_tag_names.cc:1737
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1738
                            111 => {
                                // cpp: html/html_tag_names.cc:1739
                                if &data[2..7] == &[109, 109, 97, 110, 100][..] {
                                    // cpp: html/html_tag_names.cc:1740
                                    return HTMLTag::kCommand;
                                    // cpp: html/html_tag_names.cc:1741
                                }
                                // cpp: html/html_tag_names.cc:1742
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1743
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1744
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1745
                    100 => {
                        // cpp: html/html_tag_names.cc:1746
                        if &data[1..7] == &[101, 116, 97, 105, 108, 115][..] {
                            // cpp: html/html_tag_names.cc:1747
                            return HTMLTag::kDetails;
                            // cpp: html/html_tag_names.cc:1748
                        }
                        // cpp: html/html_tag_names.cc:1749
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1750
                    105 => {
                        // cpp: html/html_tag_names.cc:1751
                        if &data[1..7] == &[110, 115, 116, 97, 108, 108][..] {
                            // cpp: html/html_tag_names.cc:1752
                            return HTMLTag::kInstallOrUnknown;
                            // cpp: html/html_tag_names.cc:1753
                        }
                        // cpp: html/html_tag_names.cc:1754
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1755
                    108 => {
                        // cpp: html/html_tag_names.cc:1756
                        if &data[1..7] == &[105, 115, 116, 105, 110, 103][..] {
                            // cpp: html/html_tag_names.cc:1757
                            return HTMLTag::kListing;
                            // cpp: html/html_tag_names.cc:1758
                        }
                        // cpp: html/html_tag_names.cc:1759
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1760
                    109 => {
                        // cpp: html/html_tag_names.cc:1761
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1762
                            97 => {
                                // cpp: html/html_tag_names.cc:1763
                                if &data[2..7] == &[114, 113, 117, 101, 101][..] {
                                    // cpp: html/html_tag_names.cc:1764
                                    return HTMLTag::kMarquee;
                                    // cpp: html/html_tag_names.cc:1765
                                }
                                // cpp: html/html_tag_names.cc:1766
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1767
                            101 => {
                                // cpp: html/html_tag_names.cc:1768
                                if &data[2..7] == &[110, 117, 98, 97, 114][..] {
                                    // cpp: html/html_tag_names.cc:1769-1778
                                    return if false {
                                        HTMLTag::kMenubar
                                    } else {
                                        HTMLTag::kUnknown
                                    };
                                    // cpp: html/html_tag_names.cc:1779
                                }
                                // cpp: html/html_tag_names.cc:1780
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1781
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1782
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1783
                    110 => {
                        // cpp: html/html_tag_names.cc:1784
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1785
                            111 => {
                                // cpp: html/html_tag_names.cc:1786
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1787
                                    101 => {
                                        // cpp: html/html_tag_names.cc:1788
                                        if &data[3..7] == &[109, 98, 101, 100][..] {
                                            // cpp: html/html_tag_names.cc:1789
                                            return HTMLTag::kNoembed;
                                            // cpp: html/html_tag_names.cc:1790
                                        }
                                        // cpp: html/html_tag_names.cc:1791
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1792
                                    108 => {
                                        // cpp: html/html_tag_names.cc:1793
                                        if &data[3..7] == &[97, 121, 101, 114][..] {
                                            // cpp: html/html_tag_names.cc:1794
                                            return HTMLTag::kNolayer;
                                            // cpp: html/html_tag_names.cc:1795
                                        }
                                        // cpp: html/html_tag_names.cc:1796
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1797
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1798
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1799
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1800
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1801
                    112 => {
                        // cpp: html/html_tag_names.cc:1802
                        if &data[1..7] == &[105, 99, 116, 117, 114, 101][..] {
                            // cpp: html/html_tag_names.cc:1803
                            return HTMLTag::kPicture;
                            // cpp: html/html_tag_names.cc:1804
                        }
                        // cpp: html/html_tag_names.cc:1805
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1806
                    115 => {
                        // cpp: html/html_tag_names.cc:1807
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1808
                            101 => {
                                // cpp: html/html_tag_names.cc:1809
                                if &data[2..7] == &[99, 116, 105, 111, 110][..] {
                                    // cpp: html/html_tag_names.cc:1810
                                    return HTMLTag::kSection;
                                    // cpp: html/html_tag_names.cc:1811
                                }
                                // cpp: html/html_tag_names.cc:1812
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1813
                            117 => {
                                // cpp: html/html_tag_names.cc:1814
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1815
                                    98 => {
                                        // cpp: html/html_tag_names.cc:1816
                                        if &data[3..7] == &[109, 101, 110, 117][..] {
                                            // cpp: html/html_tag_names.cc:1817-1826
                                            return if false {
                                                HTMLTag::kSubmenu
                                            } else {
                                                HTMLTag::kUnknown
                                            };
                                            // cpp: html/html_tag_names.cc:1827
                                        }
                                        // cpp: html/html_tag_names.cc:1828
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1829
                                    109 => {
                                        // cpp: html/html_tag_names.cc:1830
                                        if &data[3..7] == &[109, 97, 114, 121][..] {
                                            // cpp: html/html_tag_names.cc:1831
                                            return HTMLTag::kSummary;
                                            // cpp: html/html_tag_names.cc:1832
                                        }
                                        // cpp: html/html_tag_names.cc:1833
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1834
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1835
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1836
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1837
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1838
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1839
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1840
            8 => {
                // cpp: html/html_tag_names.cc:1841
                match data[0] {
                    // cpp: html/html_tag_names.cc:1842
                    98 => {
                        // cpp: html/html_tag_names.cc:1843
                        if &data[1..8] == &[97, 115, 101, 102, 111, 110, 116][..] {
                            // cpp: html/html_tag_names.cc:1844
                            return HTMLTag::kBasefont;
                            // cpp: html/html_tag_names.cc:1845
                        }
                        // cpp: html/html_tag_names.cc:1846
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1847
                    99 => {
                        // cpp: html/html_tag_names.cc:1848
                        if &data[1..8] == &[111, 108, 103, 114, 111, 117, 112][..] {
                            // cpp: html/html_tag_names.cc:1849
                            return HTMLTag::kColgroup;
                            // cpp: html/html_tag_names.cc:1850
                        }
                        // cpp: html/html_tag_names.cc:1851
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1852
                    100 => {
                        // cpp: html/html_tag_names.cc:1853
                        if &data[1..8] == &[97, 116, 97, 108, 105, 115, 116][..] {
                            // cpp: html/html_tag_names.cc:1854
                            return HTMLTag::kDatalist;
                            // cpp: html/html_tag_names.cc:1855
                        }
                        // cpp: html/html_tag_names.cc:1856
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1857
                    102 => {
                        // cpp: html/html_tag_names.cc:1858
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1859
                            105 => {
                                // cpp: html/html_tag_names.cc:1860
                                if &data[2..8] == &[101, 108, 100, 115, 101, 116][..] {
                                    // cpp: html/html_tag_names.cc:1861
                                    return HTMLTag::kFieldset;
                                    // cpp: html/html_tag_names.cc:1862
                                }
                                // cpp: html/html_tag_names.cc:1863
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1864
                            114 => {
                                // cpp: html/html_tag_names.cc:1865
                                if &data[2..8] == &[97, 109, 101, 115, 101, 116][..] {
                                    // cpp: html/html_tag_names.cc:1866
                                    return HTMLTag::kFrameset;
                                    // cpp: html/html_tag_names.cc:1867
                                }
                                // cpp: html/html_tag_names.cc:1868
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1869
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1870
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1871
                    109 => {
                        // cpp: html/html_tag_names.cc:1872
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1873
                            101 => {
                                // cpp: html/html_tag_names.cc:1874
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1875
                                    110 => {
                                        // cpp: html/html_tag_names.cc:1876
                                        match data[3] {
                                            // cpp: html/html_tag_names.cc:1877
                                            117 => {
                                                // cpp: html/html_tag_names.cc:1878
                                                match data[4] {
                                                    // cpp: html/html_tag_names.cc:1879
                                                    105 => {
                                                        // cpp: html/html_tag_names.cc:1880
                                                        if &data[5..8] == &[116, 101, 109][..] {
                                                            // cpp: html/html_tag_names.cc:1881-1890
                                                            return if false {
                                                                HTMLTag::kMenuitem
                                                            } else {
                                                                HTMLTag::kUnknown
                                                            };
                                                            // cpp: html/html_tag_names.cc:1891
                                                        }
                                                        // cpp: html/html_tag_names.cc:1892
                                                        // C++ break ends this match arm.
                                                    }
                                                    // cpp: html/html_tag_names.cc:1893
                                                    108 => {
                                                        // cpp: html/html_tag_names.cc:1894
                                                        if &data[5..8] == &[105, 115, 116][..] {
                                                            // cpp: html/html_tag_names.cc:1895-1904
                                                            return if false {
                                                                HTMLTag::kMenulist
                                                            } else {
                                                                HTMLTag::kUnknown
                                                            };
                                                            // cpp: html/html_tag_names.cc:1905
                                                        }
                                                        // cpp: html/html_tag_names.cc:1906
                                                        // C++ break ends this match arm.
                                                    }
                                                    // cpp: html/html_tag_names.cc:1907
                                                    _ => {}
                                                }
                                                // cpp: html/html_tag_names.cc:1908
                                                // C++ break ends this match arm.
                                            }
                                            // cpp: html/html_tag_names.cc:1909
                                            _ => {}
                                        }
                                        // cpp: html/html_tag_names.cc:1910
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1911
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1912
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1913
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1914
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1915
                    110 => {
                        // cpp: html/html_tag_names.cc:1916
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1917
                            111 => {
                                // cpp: html/html_tag_names.cc:1918
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1919
                                    102 => {
                                        // cpp: html/html_tag_names.cc:1920
                                        if &data[3..8] == &[114, 97, 109, 101, 115][..] {
                                            // cpp: html/html_tag_names.cc:1921
                                            return HTMLTag::kNoframes;
                                            // cpp: html/html_tag_names.cc:1922
                                        }
                                        // cpp: html/html_tag_names.cc:1923
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1924
                                    115 => {
                                        // cpp: html/html_tag_names.cc:1925
                                        if &data[3..8] == &[99, 114, 105, 112, 116][..] {
                                            // cpp: html/html_tag_names.cc:1926
                                            return HTMLTag::kNoscript;
                                            // cpp: html/html_tag_names.cc:1927
                                        }
                                        // cpp: html/html_tag_names.cc:1928
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1929
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1930
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1931
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1932
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1933
                    111 => {
                        // cpp: html/html_tag_names.cc:1934
                        if &data[1..8] == &[112, 116, 103, 114, 111, 117, 112][..] {
                            // cpp: html/html_tag_names.cc:1935
                            return HTMLTag::kOptgroup;
                            // cpp: html/html_tag_names.cc:1936
                        }
                        // cpp: html/html_tag_names.cc:1937
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1938
                    112 => {
                        // cpp: html/html_tag_names.cc:1939
                        if &data[1..8] == &[114, 111, 103, 114, 101, 115, 115][..] {
                            // cpp: html/html_tag_names.cc:1940
                            return HTMLTag::kProgress;
                            // cpp: html/html_tag_names.cc:1941
                        }
                        // cpp: html/html_tag_names.cc:1942
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1943
                    116 => {
                        // cpp: html/html_tag_names.cc:1944
                        match data[1] {
                            // cpp: html/html_tag_names.cc:1945
                            101 => {
                                // cpp: html/html_tag_names.cc:1946
                                match data[2] {
                                    // cpp: html/html_tag_names.cc:1947
                                    109 => {
                                        // cpp: html/html_tag_names.cc:1948
                                        if &data[3..8] == &[112, 108, 97, 116, 101][..] {
                                            // cpp: html/html_tag_names.cc:1949
                                            return HTMLTag::kTemplate;
                                            // cpp: html/html_tag_names.cc:1950
                                        }
                                        // cpp: html/html_tag_names.cc:1951
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1952
                                    120 => {
                                        // cpp: html/html_tag_names.cc:1953
                                        if &data[3..8] == &[116, 97, 114, 101, 97][..] {
                                            // cpp: html/html_tag_names.cc:1954
                                            return HTMLTag::kTextarea;
                                            // cpp: html/html_tag_names.cc:1955
                                        }
                                        // cpp: html/html_tag_names.cc:1956
                                        // C++ break ends this match arm.
                                    }
                                    // cpp: html/html_tag_names.cc:1957
                                    _ => {}
                                }
                                // cpp: html/html_tag_names.cc:1958
                                // C++ break ends this match arm.
                            }
                            // cpp: html/html_tag_names.cc:1959
                            _ => {}
                        }
                        // cpp: html/html_tag_names.cc:1960
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1961
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1962
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1963
            9 => {
                // cpp: html/html_tag_names.cc:1964
                match data[0] {
                    // cpp: html/html_tag_names.cc:1965
                    112 => {
                        // cpp: html/html_tag_names.cc:1966
                        if &data[1..9] == &[108, 97, 105, 110, 116, 101, 120, 116][..] {
                            // cpp: html/html_tag_names.cc:1967
                            return HTMLTag::kPlaintext;
                            // cpp: html/html_tag_names.cc:1968
                        }
                        // cpp: html/html_tag_names.cc:1969
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1970
                    117 => {
                        // cpp: html/html_tag_names.cc:1971
                        if &data[1..9] == &[115, 101, 114, 109, 101, 100, 105, 97][..] {
                            // cpp: html/html_tag_names.cc:1972
                            return HTMLTag::kUsermediaOrUnknown;
                            // cpp: html/html_tag_names.cc:1973
                        }
                        // cpp: html/html_tag_names.cc:1974
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1975
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:1976
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:1977
            10 => {
                // cpp: html/html_tag_names.cc:1978
                match data[0] {
                    // cpp: html/html_tag_names.cc:1979
                    98 => {
                        // cpp: html/html_tag_names.cc:1980
                        if &data[1..10] == &[108, 111, 99, 107, 113, 117, 111, 116, 101][..] {
                            // cpp: html/html_tag_names.cc:1981
                            return HTMLTag::kBlockquote;
                            // cpp: html/html_tag_names.cc:1982
                        }
                        // cpp: html/html_tag_names.cc:1983
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1984
                    99 => {
                        // cpp: html/html_tag_names.cc:1985
                        if &data[1..10] == &[114, 101, 100, 101, 110, 116, 105, 97, 108][..] {
                            // cpp: html/html_tag_names.cc:1986-1995
                            return if false {
                                HTMLTag::kCredential
                            } else {
                                HTMLTag::kUnknown
                            };
                            // cpp: html/html_tag_names.cc:1996
                        }
                        // cpp: html/html_tag_names.cc:1997
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:1998
                    102 => {
                        // cpp: html/html_tag_names.cc:1999
                        if &data[1..10] == &[105, 103, 99, 97, 112, 116, 105, 111, 110][..] {
                            // cpp: html/html_tag_names.cc:2000
                            return HTMLTag::kFigcaption;
                            // cpp: html/html_tag_names.cc:2001
                        }
                        // cpp: html/html_tag_names.cc:2002
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:2003
                    109 => {
                        // cpp: html/html_tag_names.cc:2004
                        if &data[1..10] == &[105, 99, 114, 111, 112, 104, 111, 110, 101][..] {
                            // cpp: html/html_tag_names.cc:2005-2014
                            return if false {
                                HTMLTag::kMicrophone
                            } else {
                                HTMLTag::kUnknown
                            };
                            // cpp: html/html_tag_names.cc:2015
                        }
                        // cpp: html/html_tag_names.cc:2016
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:2017
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:2018
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:2019
            11 => {
                // cpp: html/html_tag_names.cc:2020
                match data[0] {
                    // cpp: html/html_tag_names.cc:2021
                    102 => {
                        // cpp: html/html_tag_names.cc:2022
                        if &data[1..11] == &[101, 110, 99, 101, 100, 102, 114, 97, 109, 101][..] {
                            // cpp: html/html_tag_names.cc:2023
                            return HTMLTag::kFencedframeOrUnknown;
                            // cpp: html/html_tag_names.cc:2024
                        }
                        // cpp: html/html_tag_names.cc:2025
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:2026
                    103 => {
                        // cpp: html/html_tag_names.cc:2027
                        if &data[1..11] == &[101, 111, 108, 111, 99, 97, 116, 105, 111, 110][..] {
                            // cpp: html/html_tag_names.cc:2028-2037
                            return if false {
                                HTMLTag::kGeolocation
                            } else {
                                HTMLTag::kUnknown
                            };
                            // cpp: html/html_tag_names.cc:2038
                        }
                        // cpp: html/html_tag_names.cc:2039
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:2040
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:2041
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:2042
            15 => {
                // cpp: html/html_tag_names.cc:2043
                match data[0] {
                    // cpp: html/html_tag_names.cc:2044
                    115 => {
                        // cpp: html/html_tag_names.cc:2045
                        if &data[1..15]
                            == &[
                                101, 108, 101, 99, 116, 101, 100, 99, 111, 110, 116, 101, 110, 116,
                            ][..]
                        {
                            // cpp: html/html_tag_names.cc:2046
                            return HTMLTag::kSelectedcontent;
                            // cpp: html/html_tag_names.cc:2047
                        }
                        // cpp: html/html_tag_names.cc:2048
                        // C++ break ends this match arm.
                    }
                    // cpp: html/html_tag_names.cc:2049
                    _ => {}
                }
                // cpp: html/html_tag_names.cc:2050
                // C++ break ends this match arm.
            }
            // cpp: html/html_tag_names.cc:2051
            _ => {}
        }
        // cpp: html/html_tag_names.cc:2053
        HTMLTag::kUnknown
        // cpp: html/html_tag_names.cc:2054
    }
}
