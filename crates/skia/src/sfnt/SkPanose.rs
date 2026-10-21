// Packed native declarations; raw big-endian integer storage is retained.
// Transparent enum values admit every wire byte, including unknown values.
// Bit-field accessors use the source byte order and named low-to-high bit indices.
// No C++ object references are formed into unaligned font storage.
// Source: src/sfnt/SkPanose.h:15-520
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkPanose {
    // Source: src/sfnt/SkPanose.h:24-24
    pub bFamilyType: SkPanose_FamilyType,
    // Source: src/sfnt/SkPanose.h:519-519
    pub data: SkPanose_Data,
}
// Source: src/sfnt/SkPanose.h:17-24
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_FamilyType(pub u8);
impl SkPanose_FamilyType {
    // Source: src/sfnt/SkPanose.h:18-18
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:19-19
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:20-20
    pub const TextAndDisplay: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:21-21
    pub const Script: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:22-22
    pub const Decorative: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:23-23
    pub const Pictoral: Self = Self(5);
}
// Source: src/sfnt/SkPanose.h:26-519
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union SkPanose_Data {
    // Source: src/sfnt/SkPanose.h:181-181
    pub textAndDisplay: SkPanose_Data_TextAndDisplay,
    // Source: src/sfnt/SkPanose.h:299-299
    pub script: SkPanose_Data_Script,
    // Source: src/sfnt/SkPanose.h:433-433
    pub decorative: SkPanose_Data_Decorative,
    // Source: src/sfnt/SkPanose.h:518-518
    pub pictoral: SkPanose_Data_Pictoral,
}
// Source: src/sfnt/SkPanose.h:27-181
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkPanose_Data_TextAndDisplay {
    // Source: src/sfnt/SkPanose.h:45-45
    pub bSerifStyle: SkPanose_Data_TextAndDisplay_SerifStyle,
    // Source: src/sfnt/SkPanose.h:60-60
    pub bWeight: SkPanose_Data_TextAndDisplay_Weight,
    // Source: src/sfnt/SkPanose.h:73-73
    pub bProportion: SkPanose_Data_TextAndDisplay_Proportion,
    // Source: src/sfnt/SkPanose.h:86-86
    pub bContrast: SkPanose_Data_TextAndDisplay_Contrast,
    // Source: src/sfnt/SkPanose.h:101-101
    #[cfg(SK_WIN_PANOSE)]
    pub bStrokeVariation: SkPanose_Data_TextAndDisplay_StrokeVariation,
    // Source: src/sfnt/SkPanose.h:117-117
    #[cfg(not(SK_WIN_PANOSE))]
    pub bStrokeVariation: SkPanose_Data_TextAndDisplay_StrokeVariation,
    // Source: src/sfnt/SkPanose.h:133-133
    pub bArmStyle: SkPanose_Data_TextAndDisplay_ArmStyle,
    // Source: src/sfnt/SkPanose.h:152-152
    pub bLetterform: SkPanose_Data_TextAndDisplay_Letterform,
    // Source: src/sfnt/SkPanose.h:169-169
    pub bMidline: SkPanose_Data_TextAndDisplay_Midline,
    // Source: src/sfnt/SkPanose.h:180-180
    pub bXHeight: SkPanose_Data_TextAndDisplay_XHeight,
}
// Source: src/sfnt/SkPanose.h:28-45
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_SerifStyle(pub u8);
impl SkPanose_Data_TextAndDisplay_SerifStyle {
    // Source: src/sfnt/SkPanose.h:29-29
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:30-30
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:31-31
    pub const Cove: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:32-32
    pub const ObtuseCove: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:33-33
    pub const SquareCove: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:34-34
    pub const ObtuseSquareCove: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:35-35
    pub const Square: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:36-36
    pub const Thin: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:37-37
    pub const Bone: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:38-38
    pub const Exaggerated: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:39-39
    pub const Triangle: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:40-40
    pub const NormalSans: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:41-41
    pub const ObtuseSans: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:42-42
    pub const PerpSans: Self = Self(13);
    // Source: src/sfnt/SkPanose.h:43-43
    pub const Flared: Self = Self(14);
    // Source: src/sfnt/SkPanose.h:44-44
    pub const Rounded: Self = Self(15);
}
// Source: src/sfnt/SkPanose.h:47-60
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_Weight(pub u8);
impl SkPanose_Data_TextAndDisplay_Weight {
    // Source: src/sfnt/SkPanose.h:48-48
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:49-49
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:50-50
    pub const VeryLight: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:51-51
    pub const Light: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:52-52
    pub const Thin: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:53-53
    pub const Book: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:54-54
    pub const Medium: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:55-55
    pub const Demi: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:56-56
    pub const Bold: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:57-57
    pub const Heavy: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:58-58
    pub const Black: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:59-59
    pub const ExtraBlack: Self = Self(11);
}
// Source: src/sfnt/SkPanose.h:62-73
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_Proportion(pub u8);
impl SkPanose_Data_TextAndDisplay_Proportion {
    // Source: src/sfnt/SkPanose.h:63-63
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:64-64
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:65-65
    pub const OldStyle: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:66-66
    pub const Modern: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:67-67
    pub const EvenWidth: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:68-68
    pub const Expanded: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:69-69
    pub const Condensed: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:70-70
    pub const VeryExpanded: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:71-71
    pub const VeryCondensed: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:72-72
    pub const Monospaced: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:75-86
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_Contrast(pub u8);
impl SkPanose_Data_TextAndDisplay_Contrast {
    // Source: src/sfnt/SkPanose.h:76-76
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:77-77
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:78-78
    pub const None: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:79-79
    pub const VeryLow: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:80-80
    pub const Low: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:81-81
    pub const MediumLow: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:82-82
    pub const Medium: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:83-83
    pub const MediumHigh: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:84-84
    pub const High: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:85-85
    pub const VeryHigh: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:91-101
#[cfg(SK_WIN_PANOSE)]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_StrokeVariation(pub u8);
#[cfg(SK_WIN_PANOSE)]
impl SkPanose_Data_TextAndDisplay_StrokeVariation {
    // Source: src/sfnt/SkPanose.h:92-92
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:93-93
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:94-94
    pub const GradualDiagonal: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:95-95
    pub const GradualTransitional: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:96-96
    pub const GradualVertical: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:97-97
    pub const GradualHorizontal: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:98-98
    pub const RapidVertical: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:99-99
    pub const RapidHorizontal: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:100-100
    pub const InstantVertical: Self = Self(8);
}
// Source: src/sfnt/SkPanose.h:105-117
#[cfg(not(SK_WIN_PANOSE))]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_StrokeVariation(pub u8);
#[cfg(not(SK_WIN_PANOSE))]
impl SkPanose_Data_TextAndDisplay_StrokeVariation {
    // Source: src/sfnt/SkPanose.h:106-106
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:107-107
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:108-108
    pub const NoVariation: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:109-109
    pub const Gradual_Diagonal: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:110-110
    pub const Gradual_Transitional: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:111-111
    pub const Gradual_Vertical: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:112-112
    pub const Gradual_Horizontal: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:113-113
    pub const Rapid_Vertical: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:114-114
    pub const Rapid_Horizontal: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:115-115
    pub const Instant_Vertical: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:116-116
    pub const Instant_Horizontal: Self = Self(10);
}
// Source: src/sfnt/SkPanose.h:120-133
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_ArmStyle(pub u8);
impl SkPanose_Data_TextAndDisplay_ArmStyle {
    // Source: src/sfnt/SkPanose.h:121-121
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:122-122
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:123-123
    pub const StraightArms_Horizontal: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:124-124
    pub const StraightArms_Wedge: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:125-125
    pub const StraightArms_Vertical: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:126-126
    pub const StraightArms_SingleSerif: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:127-127
    pub const StraightArms_DoubleSerif: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:128-128
    pub const NonStraightArms_Horizontal: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:129-129
    pub const NonStraightArms_Wedge: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:130-130
    pub const NonStraightArms_Vertical: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:131-131
    pub const NonStraightArms_SingleSerif: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:132-132
    pub const NonStraightArms_DoubleSerif: Self = Self(11);
}
// Source: src/sfnt/SkPanose.h:135-152
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_Letterform(pub u8);
impl SkPanose_Data_TextAndDisplay_Letterform {
    // Source: src/sfnt/SkPanose.h:136-136
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:137-137
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:138-138
    pub const Normal_Contact: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:139-139
    pub const Normal_Weighted: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:140-140
    pub const Normal_Boxed: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:141-141
    pub const Normal_Flattened: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:142-142
    pub const Normal_Rounded: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:143-143
    pub const Normal_OffCenter: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:144-144
    pub const Normal_Square: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:145-145
    pub const Oblique_Contact: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:146-146
    pub const Oblique_Weighted: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:147-147
    pub const Oblique_Boxed: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:148-148
    pub const Oblique_Flattened: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:149-149
    pub const Oblique_Rounded: Self = Self(13);
    // Source: src/sfnt/SkPanose.h:150-150
    pub const Oblique_OffCenter: Self = Self(14);
    // Source: src/sfnt/SkPanose.h:151-151
    pub const Oblique_Square: Self = Self(15);
}
// Source: src/sfnt/SkPanose.h:154-169
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_Midline(pub u8);
impl SkPanose_Data_TextAndDisplay_Midline {
    // Source: src/sfnt/SkPanose.h:155-155
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:156-156
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:157-157
    pub const Standard_Trimmed: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:158-158
    pub const Standard_Pointed: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:159-159
    pub const Standard_Serifed: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:160-160
    pub const High_Trimmed: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:161-161
    pub const High_Pointed: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:162-162
    pub const High_Serifed: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:163-163
    pub const Constant_Trimmed: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:164-164
    pub const Constant_Pointed: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:165-165
    pub const Constant_Serifed: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:166-166
    pub const Low_Trimmed: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:167-167
    pub const Low_Pointed: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:168-168
    pub const Low_Serifed: Self = Self(13);
}
// Source: src/sfnt/SkPanose.h:171-180
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_TextAndDisplay_XHeight(pub u8);
impl SkPanose_Data_TextAndDisplay_XHeight {
    // Source: src/sfnt/SkPanose.h:172-172
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:173-173
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:174-174
    pub const Constant_Small: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:175-175
    pub const Constant_Standard: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:176-176
    pub const Constant_Large: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:177-177
    pub const Ducking_Small: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:178-178
    pub const Ducking_Standard: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:179-179
    pub const Ducking_Large: Self = Self(7);
}
// Source: src/sfnt/SkPanose.h:183-299
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkPanose_Data_Script {
    // Source: src/sfnt/SkPanose.h:195-195
    pub bToolKind: SkPanose_Data_Script_ToolKind,
    // Source: src/sfnt/SkPanose.h:210-210
    pub bWeight: SkPanose_Data_Script_Weight,
    // Source: src/sfnt/SkPanose.h:217-217
    pub bSpacing: SkPanose_Data_Script_Spacing,
    // Source: src/sfnt/SkPanose.h:227-227
    pub bAspectRatio: SkPanose_Data_Script_AspectRatio,
    // Source: src/sfnt/SkPanose.h:240-240
    pub bContrast: SkPanose_Data_Script_Contrast,
    // Source: src/sfnt/SkPanose.h:254-254
    pub bTopology: SkPanose_Data_Script_Topology,
    // Source: src/sfnt/SkPanose.h:271-271
    pub bForm: SkPanose_Data_Script_Form,
    // Source: src/sfnt/SkPanose.h:288-288
    pub bFinials: SkPanose_Data_Script_Finials,
    // Source: src/sfnt/SkPanose.h:298-298
    pub bXAscent: SkPanose_Data_Script_XAscent,
}
// Source: src/sfnt/SkPanose.h:184-195
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_ToolKind(pub u8);
impl SkPanose_Data_Script_ToolKind {
    // Source: src/sfnt/SkPanose.h:185-185
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:186-186
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:187-187
    pub const FlatNib: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:188-188
    pub const PressurePoint: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:189-189
    pub const Engraved: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:190-190
    pub const Ball: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:191-191
    pub const Brush: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:192-192
    pub const Rough: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:193-193
    pub const FeltPen: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:194-194
    pub const WildBrush: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:197-210
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_Weight(pub u8);
impl SkPanose_Data_Script_Weight {
    // Source: src/sfnt/SkPanose.h:198-198
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:199-199
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:200-200
    pub const VeryLight: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:201-201
    pub const Light: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:202-202
    pub const Thin: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:203-203
    pub const Book: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:204-204
    pub const Medium: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:205-205
    pub const Demi: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:206-206
    pub const Bold: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:207-207
    pub const Heavy: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:208-208
    pub const Black: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:209-209
    pub const ExtraBlack: Self = Self(11);
}
// Source: src/sfnt/SkPanose.h:212-217
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_Spacing(pub u8);
impl SkPanose_Data_Script_Spacing {
    // Source: src/sfnt/SkPanose.h:213-213
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:214-214
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:215-215
    pub const ProportionalSpaced: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:216-216
    pub const Monospaced: Self = Self(3);
}
// Source: src/sfnt/SkPanose.h:219-227
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_AspectRatio(pub u8);
impl SkPanose_Data_Script_AspectRatio {
    // Source: src/sfnt/SkPanose.h:220-220
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:221-221
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:222-222
    pub const VeryCondensed: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:223-223
    pub const Condensed: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:224-224
    pub const Normal: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:225-225
    pub const Expanded: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:226-226
    pub const VeryExpanded: Self = Self(6);
}
// Source: src/sfnt/SkPanose.h:229-240
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_Contrast(pub u8);
impl SkPanose_Data_Script_Contrast {
    // Source: src/sfnt/SkPanose.h:230-230
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:231-231
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:232-232
    pub const None: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:233-233
    pub const VeryLow: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:234-234
    pub const Low: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:235-235
    pub const MediumLow: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:236-236
    pub const Medium: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:237-237
    pub const MediumHigh: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:238-238
    pub const High: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:239-239
    pub const VeryHigh: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:242-254
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_Topology(pub u8);
impl SkPanose_Data_Script_Topology {
    // Source: src/sfnt/SkPanose.h:243-243
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:244-244
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:245-245
    pub const Roman_Disconnected: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:246-246
    pub const Roman_Trailing: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:247-247
    pub const Roman_Connected: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:248-248
    pub const Cursive_Disconnected: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:249-249
    pub const Cursive_Trailing: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:250-250
    pub const Cursive_Connected: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:251-251
    pub const Blackletter_Disconnected: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:252-252
    pub const Blackletter_Trailing: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:253-253
    pub const Blackletter_Connected: Self = Self(10);
}
// Source: src/sfnt/SkPanose.h:256-271
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_Form(pub u8);
impl SkPanose_Data_Script_Form {
    // Source: src/sfnt/SkPanose.h:257-257
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:258-258
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:259-259
    pub const Upright_NoWrapping: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:260-260
    pub const Upright_SomeWrapping: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:261-261
    pub const Upright_MoreWrapping: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:262-262
    pub const Upright_ExtremeWrapping: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:263-263
    pub const Oblique_NoWrapping: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:264-264
    pub const Oblique_SomeWrapping: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:265-265
    pub const Oblique_MoreWrapping: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:266-266
    pub const Oblique_ExtremeWrapping: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:267-267
    pub const Exaggerated_NoWrapping: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:268-268
    pub const Exaggerated_SomeWrapping: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:269-269
    pub const Exaggerated_MoreWrapping: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:270-270
    pub const Exaggerated_ExtremeWrapping: Self = Self(13);
}
// Source: src/sfnt/SkPanose.h:273-288
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_Finials(pub u8);
impl SkPanose_Data_Script_Finials {
    // Source: src/sfnt/SkPanose.h:274-274
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:275-275
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:276-276
    pub const None_NoLoops: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:277-277
    pub const None_ClosedLoops: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:278-278
    pub const None_OpenLoops: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:279-279
    pub const Sharp_NoLoops: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:280-280
    pub const Sharp_ClosedLoops: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:281-281
    pub const Sharp_OpenLoops: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:282-282
    pub const Tapered_NoLoops: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:283-283
    pub const Tapered_ClosedLoops: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:284-284
    pub const Tapered_OpenLoops: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:285-285
    pub const Round_NoLoops: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:286-286
    pub const Round_ClosedLoops: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:287-287
    pub const Round_OpenLoops: Self = Self(13);
}
// Source: src/sfnt/SkPanose.h:290-298
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Script_XAscent(pub u8);
impl SkPanose_Data_Script_XAscent {
    // Source: src/sfnt/SkPanose.h:291-291
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:292-292
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:293-293
    pub const VeryLow: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:294-294
    pub const Low: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:295-295
    pub const Medium: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:296-296
    pub const High: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:297-297
    pub const VeryHigh: Self = Self(6);
}
// Source: src/sfnt/SkPanose.h:301-433
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkPanose_Data_Decorative {
    // Source: src/sfnt/SkPanose.h:316-316
    pub bClass: SkPanose_Data_Decorative_Class,
    // Source: src/sfnt/SkPanose.h:331-331
    pub bWeight: SkPanose_Data_Decorative_Weight,
    // Source: src/sfnt/SkPanose.h:344-344
    pub bAspect: SkPanose_Data_Decorative_Aspect,
    // Source: src/sfnt/SkPanose.h:361-361
    pub bContrast: SkPanose_Data_Decorative_Contrast,
    // Source: src/sfnt/SkPanose.h:381-381
    pub bSerifVariant: SkPanose_Data_Decorative_SerifVariant,
    // Source: src/sfnt/SkPanose.h:392-392
    pub bTreatment: SkPanose_Data_Decorative_Treatment,
    // Source: src/sfnt/SkPanose.h:404-404
    pub bLining: SkPanose_Data_Decorative_Lining,
    // Source: src/sfnt/SkPanose.h:423-423
    pub bTopology: SkPanose_Data_Decorative_Topology,
    // Source: src/sfnt/SkPanose.h:432-432
    pub bRangeOfCharacters: SkPanose_Data_Decorative_RangeOfCharacters,
}
// Source: src/sfnt/SkPanose.h:302-316
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Class(pub u8);
impl SkPanose_Data_Decorative_Class {
    // Source: src/sfnt/SkPanose.h:303-303
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:304-304
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:305-305
    pub const Derivative: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:306-306
    pub const NonStandard_Topology: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:307-307
    pub const NonStandard_Elements: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:308-308
    pub const NonStandard_Aspect: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:309-309
    pub const Initials: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:310-310
    pub const Cartoon: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:311-311
    pub const PictureStems: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:312-312
    pub const Ornamented: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:313-313
    pub const TextAndBackground: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:314-314
    pub const Collage: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:315-315
    pub const Montage: Self = Self(12);
}
// Source: src/sfnt/SkPanose.h:318-331
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Weight(pub u8);
impl SkPanose_Data_Decorative_Weight {
    // Source: src/sfnt/SkPanose.h:319-319
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:320-320
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:321-321
    pub const VeryLight: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:322-322
    pub const Light: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:323-323
    pub const Thin: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:324-324
    pub const Book: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:325-325
    pub const Medium: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:326-326
    pub const Demi: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:327-327
    pub const Bold: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:328-328
    pub const Heavy: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:329-329
    pub const Black: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:330-330
    pub const ExtraBlack: Self = Self(11);
}
// Source: src/sfnt/SkPanose.h:333-344
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Aspect(pub u8);
impl SkPanose_Data_Decorative_Aspect {
    // Source: src/sfnt/SkPanose.h:334-334
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:335-335
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:336-336
    pub const SuperCondensed: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:337-337
    pub const VeryCondensed: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:338-338
    pub const Condensed: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:339-339
    pub const Normal: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:340-340
    pub const Extended: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:341-341
    pub const VeryExtended: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:342-342
    pub const SuperExtended: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:343-343
    pub const Monospaced: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:346-361
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Contrast(pub u8);
impl SkPanose_Data_Decorative_Contrast {
    // Source: src/sfnt/SkPanose.h:347-347
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:348-348
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:349-349
    pub const None: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:350-350
    pub const VeryLow: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:351-351
    pub const Low: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:352-352
    pub const MediumLow: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:353-353
    pub const Medium: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:354-354
    pub const MediumHigh: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:355-355
    pub const High: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:356-356
    pub const VeryHigh: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:357-357
    pub const HorizontalLow: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:358-358
    pub const HorizontalMedium: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:359-359
    pub const HorizontalHigh: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:360-360
    pub const Broken: Self = Self(13);
}
// Source: src/sfnt/SkPanose.h:363-381
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_SerifVariant(pub u8);
impl SkPanose_Data_Decorative_SerifVariant {
    // Source: src/sfnt/SkPanose.h:364-364
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:365-365
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:366-366
    pub const Cove: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:367-367
    pub const ObtuseCove: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:368-368
    pub const SquareCove: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:369-369
    pub const ObtuseSquareCove: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:370-370
    pub const Square: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:371-371
    pub const Thin: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:372-372
    pub const Oval: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:373-373
    pub const Exaggerated: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:374-374
    pub const Triangle: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:375-375
    pub const NormalSans: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:376-376
    pub const ObtuseSans: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:377-377
    pub const PerpendicularSans: Self = Self(13);
    // Source: src/sfnt/SkPanose.h:378-378
    pub const Flared: Self = Self(14);
    // Source: src/sfnt/SkPanose.h:379-379
    pub const Rounded: Self = Self(15);
    // Source: src/sfnt/SkPanose.h:380-380
    pub const Script: Self = Self(16);
}
// Source: src/sfnt/SkPanose.h:383-392
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Treatment(pub u8);
impl SkPanose_Data_Decorative_Treatment {
    // Source: src/sfnt/SkPanose.h:384-384
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:385-385
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:386-386
    pub const None_StandardSolidFill: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:387-387
    pub const White_NoFill: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:388-388
    pub const PatternedFill: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:389-389
    pub const ComplexFill: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:390-390
    pub const ShapedFill: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:391-391
    pub const DrawnDistressed: Self = Self(7);
}
// Source: src/sfnt/SkPanose.h:394-404
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Lining(pub u8);
impl SkPanose_Data_Decorative_Lining {
    // Source: src/sfnt/SkPanose.h:395-395
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:396-396
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:397-397
    pub const None: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:398-398
    pub const Inline: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:399-399
    pub const Outline: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:400-400
    pub const Engraved: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:401-401
    pub const Shadow: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:402-402
    pub const Relief: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:403-403
    pub const Backdrop: Self = Self(8);
}
// Source: src/sfnt/SkPanose.h:406-423
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_Topology(pub u8);
impl SkPanose_Data_Decorative_Topology {
    // Source: src/sfnt/SkPanose.h:407-407
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:408-408
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:409-409
    pub const Standard: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:410-410
    pub const Square: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:411-411
    pub const MultipleSegment: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:412-412
    pub const DecoWacoMidlines: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:413-413
    pub const UnevenWeighting: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:414-414
    pub const DiverseArms: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:415-415
    pub const DiverseForms: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:416-416
    pub const LombardicForms: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:417-417
    pub const UpperCaseInLowerCase: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:418-418
    pub const ImpliedTopology: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:419-419
    pub const HorseshoeEandA: Self = Self(12);
    // Source: src/sfnt/SkPanose.h:420-420
    pub const Cursive: Self = Self(13);
    // Source: src/sfnt/SkPanose.h:421-421
    pub const Blackletter: Self = Self(14);
    // Source: src/sfnt/SkPanose.h:422-422
    pub const SwashVariance: Self = Self(15);
}
// Source: src/sfnt/SkPanose.h:425-432
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Decorative_RangeOfCharacters(pub u8);
impl SkPanose_Data_Decorative_RangeOfCharacters {
    // Source: src/sfnt/SkPanose.h:426-426
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:427-427
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:428-428
    pub const ExtendedCollection: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:429-429
    pub const Litterals: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:430-430
    pub const NoLowerCase: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:431-431
    pub const SmallCaps: Self = Self(5);
}
// Source: src/sfnt/SkPanose.h:435-518
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct SkPanose_Data_Pictoral {
    // Source: src/sfnt/SkPanose.h:450-450
    pub bKind: SkPanose_Data_Pictoral_Kind,
    // Source: src/sfnt/SkPanose.h:454-454
    pub bWeight: SkPanose_Data_Pictoral_Weight,
    // Source: src/sfnt/SkPanose.h:461-461
    pub bSpacing: SkPanose_Data_Pictoral_Spacing,
    // Source: src/sfnt/SkPanose.h:465-465
    pub bAspectRatioAndContrast: SkPanose_Data_Pictoral_AspectRatioAndContrast,
    // Source: src/sfnt/SkPanose.h:478-478
    pub bAspectRatio94: SkPanose_Data_Pictoral_AspectRatio94,
    // Source: src/sfnt/SkPanose.h:491-491
    pub bAspectRatio119: SkPanose_Data_Pictoral_AspectRatio119,
    // Source: src/sfnt/SkPanose.h:504-504
    pub bAspectRatio157: SkPanose_Data_Pictoral_AspectRatio157,
    // Source: src/sfnt/SkPanose.h:517-517
    pub bAspectRatio163: SkPanose_Data_Pictoral_AspectRatio163,
}
// Source: src/sfnt/SkPanose.h:436-450
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_Kind(pub u8);
impl SkPanose_Data_Pictoral_Kind {
    // Source: src/sfnt/SkPanose.h:437-437
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:438-438
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:439-439
    pub const Montages: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:440-440
    pub const Pictures: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:441-441
    pub const Shapes: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:442-442
    pub const Scientific: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:443-443
    pub const Music: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:444-444
    pub const Expert: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:445-445
    pub const Patterns: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:446-446
    pub const Boarders: Self = Self(9);
    // Source: src/sfnt/SkPanose.h:447-447
    pub const Icons: Self = Self(10);
    // Source: src/sfnt/SkPanose.h:448-448
    pub const Logos: Self = Self(11);
    // Source: src/sfnt/SkPanose.h:449-449
    pub const IndustrySpecific: Self = Self(12);
}
// Source: src/sfnt/SkPanose.h:452-454
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_Weight(pub u8);
impl SkPanose_Data_Pictoral_Weight {
    // Source: src/sfnt/SkPanose.h:453-453
    pub const NoFit: Self = Self(1);
}
// Source: src/sfnt/SkPanose.h:456-461
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_Spacing(pub u8);
impl SkPanose_Data_Pictoral_Spacing {
    // Source: src/sfnt/SkPanose.h:457-457
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:458-458
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:459-459
    pub const ProportionalSpaced: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:460-460
    pub const Monospaced: Self = Self(3);
}
// Source: src/sfnt/SkPanose.h:463-465
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_AspectRatioAndContrast(pub u8);
impl SkPanose_Data_Pictoral_AspectRatioAndContrast {
    // Source: src/sfnt/SkPanose.h:464-464
    pub const NoFit: Self = Self(1);
}
// Source: src/sfnt/SkPanose.h:467-478
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_AspectRatio94(pub u8);
impl SkPanose_Data_Pictoral_AspectRatio94 {
    // Source: src/sfnt/SkPanose.h:468-468
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:469-469
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:470-470
    pub const NoWidth: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:471-471
    pub const ExceptionallyWide: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:472-472
    pub const SuperWide: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:473-473
    pub const VeryWide: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:474-474
    pub const Wide: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:475-475
    pub const Normal: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:476-476
    pub const Narrow: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:477-477
    pub const VeryNarrow: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:480-491
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_AspectRatio119(pub u8);
impl SkPanose_Data_Pictoral_AspectRatio119 {
    // Source: src/sfnt/SkPanose.h:481-481
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:482-482
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:483-483
    pub const NoWidth: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:484-484
    pub const ExceptionallyWide: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:485-485
    pub const SuperWide: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:486-486
    pub const VeryWide: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:487-487
    pub const Wide: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:488-488
    pub const Normal: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:489-489
    pub const Narrow: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:490-490
    pub const VeryNarrow: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:493-504
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_AspectRatio157(pub u8);
impl SkPanose_Data_Pictoral_AspectRatio157 {
    // Source: src/sfnt/SkPanose.h:494-494
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:495-495
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:496-496
    pub const NoWidth: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:497-497
    pub const ExceptionallyWide: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:498-498
    pub const SuperWide: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:499-499
    pub const VeryWide: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:500-500
    pub const Wide: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:501-501
    pub const Normal: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:502-502
    pub const Narrow: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:503-503
    pub const VeryNarrow: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:506-517
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SkPanose_Data_Pictoral_AspectRatio163(pub u8);
impl SkPanose_Data_Pictoral_AspectRatio163 {
    // Source: src/sfnt/SkPanose.h:507-507
    pub const Any: Self = Self(0);
    // Source: src/sfnt/SkPanose.h:508-508
    pub const NoFit: Self = Self(1);
    // Source: src/sfnt/SkPanose.h:509-509
    pub const NoWidth: Self = Self(2);
    // Source: src/sfnt/SkPanose.h:510-510
    pub const ExceptionallyWide: Self = Self(3);
    // Source: src/sfnt/SkPanose.h:511-511
    pub const SuperWide: Self = Self(4);
    // Source: src/sfnt/SkPanose.h:512-512
    pub const VeryWide: Self = Self(5);
    // Source: src/sfnt/SkPanose.h:513-513
    pub const Wide: Self = Self(6);
    // Source: src/sfnt/SkPanose.h:514-514
    pub const Normal: Self = Self(7);
    // Source: src/sfnt/SkPanose.h:515-515
    pub const Narrow: Self = Self(8);
    // Source: src/sfnt/SkPanose.h:516-516
    pub const VeryNarrow: Self = Self(9);
}
// Source: src/sfnt/SkPanose.h:525-525
const _: () = assert!(core::mem::size_of::<SkPanose>() == 10);
