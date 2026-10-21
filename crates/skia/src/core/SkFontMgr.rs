use super::{
    SkData::SkData,
    SkFontDescriptor::SkFontDescriptor,
    SkScalar::SkScalarRoundToInt,
    SkStream::SkStreamAsset,
    SkString::SkString,
    SkTypeface::{SkTypeface, SkTypefaceRef},
};
use crate::include::core::{
    SkFontArguments::{Coordinate, SkFontArguments},
    SkFontStyle::{SkFontStyle, Slant},
    SkRefCnt::{sk_sp, SkRefCnt, SkRefCntBase, SkRefCntObject},
};
use core::{ffi::c_char, ptr::NonNull};
use std::sync::{Arc, OnceLock};

// Existing canonical Typeface and Data owners are retained, without a proxy or
// secondary counter. New native manager/style-set owners use actual SkRefCnt.
pub type TypefaceOwner = Option<SkTypefaceRef>;

// Source: include/core/SkFontMgr.h:24-29
// Nonconst native style-set virtuals require the caller's exclusive mutation
// lease. An intrusive handle alone never manufactures a mutable object borrow.
pub trait SkFontStyleSet: SkRefCntObject {
    fn count(&mut self) -> i32;
    fn getStyle(
        &mut self,
        index: i32,
        fontStyle: Option<&mut SkFontStyle>,
        style: Option<&mut SkString>,
    );
    fn createTypeface(&mut self, index: i32) -> TypefaceOwner;
    fn matchStyle(&mut self, pattern: &SkFontStyle) -> TypefaceOwner;
}

// Source: include/core/SkFontMgr.h:89-94
#[derive(Clone, Copy)]
pub struct CMapEntry {
    pub character: i32,
    pub variation: i32,
}
// Source: include/core/SkFontMgr.h:89-110
// SkSpan becomes a borrowed slice, retaining its backing rather than copying.
// Nested value types are hoisted to module scope. No Request default is invented.
pub struct Request<'a> {
    pub cmapEntries: &'a [CMapEntry],
    pub bcp47: &'a [*const c_char],
    pub familyName: *const c_char,
    pub model: &'a [Coordinate],
    pub syntheticBold: Option<bool>,
    pub syntheticOblique: Option<bool>,
}

// Source: include/core/SkFontMgr.h:37-41
// Required hooks below are the actual abstract platform contract, not an
// optional provider. The native source only defines defaults for onMatch/Fallback.
pub trait SkFontMgr: SkRefCntObject {
    // Source: include/core/SkFontMgr.h:152-154
    fn onCountFamilies(&self) -> i32;
    fn onGetFamilyName(&self, index: i32, familyName: Option<&mut SkString>);
    fn onCreateStyleSet(&self, index: i32) -> sk_sp<dyn SkFontStyleSet>;
    // Source: include/core/SkFontMgr.h:157-157
    unsafe fn onMatchFamily(&self, familyName: *const c_char) -> sk_sp<dyn SkFontStyleSet>;
    // Source: include/core/SkFontMgr.h:159-164
    unsafe fn onMatchFamilyStyle(
        &self,
        familyName: *const c_char,
        style: &SkFontStyle,
    ) -> TypefaceOwner;
    unsafe fn onMatchFamilyStyleCharacter(
        &self,
        familyName: *const c_char,
        style: &SkFontStyle,
        bcp47: *const *const c_char,
        bcp47Count: i32,
        character: i32,
    ) -> TypefaceOwner;
    // Source: include/core/SkFontMgr.h:165-165
    unsafe fn onMatch(&self, request: &Request<'_>) -> TypefaceOwner {
        on_match(self, request)
    }
    // Source: include/core/SkFontMgr.h:166-166
    unsafe fn onFallback(&self, request: &Request<'_>) -> TypefaceOwner {
        on_fallback(self, request)
    }
    // Source: include/core/SkFontMgr.h:168-175
    fn onMakeFromData(&self, data: Arc<SkData>, ttcIndex: i32) -> TypefaceOwner;
    fn onMakeFromStreamIndex(
        &self,
        stream: core::pin::Pin<Box<dyn SkStreamAsset>>,
        ttcIndex: i32,
    ) -> TypefaceOwner;
    unsafe fn onMakeFromStreamArgs(
        &self,
        stream: core::pin::Pin<Box<dyn SkStreamAsset>>,
        args: &SkFontArguments,
    ) -> TypefaceOwner;
    unsafe fn onMakeFromFile(&self, path: *const c_char, ttcIndex: i32) -> TypefaceOwner;
    unsafe fn onLegacyMakeTypeface(
        &self,
        familyName: *const c_char,
        style: SkFontStyle,
    ) -> TypefaceOwner;
}

// Native nonvirtual wrappers remain inherent methods on the real dynamic
// manager object; implementations cannot override them as new trait slots.
impl dyn SkFontMgr {
    // Source: include/core/SkFontMgr.h:39-39
    pub fn countFamilies(&self) -> i32 {
        count_families(self)
    }
    // Source: include/core/SkFontMgr.h:40-40
    pub fn getFamilyName(&self, index: i32, familyName: Option<&mut SkString>) {
        get_family_name(self, index, familyName)
    }
    // Source: include/core/SkFontMgr.h:41-41
    pub fn createStyleSet(&self, index: i32) -> sk_sp<dyn SkFontStyleSet> {
        create_style_set(self, index)
    }
    // Source: include/core/SkFontMgr.h:54-54
    /// Safety: nonnull familyName addresses a live NUL-terminated string.
    pub unsafe fn matchFamily(&self, familyName: *const c_char) -> sk_sp<dyn SkFontStyleSet> {
        match_family(self, familyName)
    }
    // Source: include/core/SkFontMgr.h:68-68
    pub unsafe fn matchFamilyStyle(
        &self,
        familyName: *const c_char,
        style: &SkFontStyle,
    ) -> TypefaceOwner {
        match_family_style(self, familyName, style)
    }
    // Source: include/core/SkFontMgr.h:85-87
    /// Safety: familyName and each bcp47 element are live nullable/native strings;
    /// the bcp47 array covers bcp47Count and remains borrowed through dispatch.
    pub unsafe fn matchFamilyStyleCharacter(
        &self,
        familyName: *const c_char,
        style: &SkFontStyle,
        bcp47: *const *const c_char,
        bcp47Count: i32,
        character: i32,
    ) -> TypefaceOwner {
        match_family_style_character(self, familyName, style, bcp47, bcp47Count, character)
    }
    // Source: include/core/SkFontMgr.h:113-113
    pub unsafe fn r#match(&self, request: &Request<'_>) -> TypefaceOwner {
        match_request(self, request)
    }
    // Source: include/core/SkFontMgr.h:119-119
    pub unsafe fn fallback(&self, request: &Request<'_>) -> TypefaceOwner {
        fallback(self, request)
    }
    // Source: include/core/SkFontMgr.h:126-126
    pub fn makeFromData(&self, data: Option<Arc<SkData>>, ttcIndex: i32) -> TypefaceOwner {
        make_from_data(self, data, ttcIndex)
    }
    // Source: include/core/SkFontMgr.h:133-133
    pub fn makeFromStream_index(
        &self,
        stream: Option<core::pin::Pin<Box<dyn SkStreamAsset>>>,
        ttcIndex: i32,
    ) -> TypefaceOwner {
        make_from_stream_index(self, stream, ttcIndex)
    }
    // Source: include/core/SkFontMgr.h:136-136
    /// Safety: font arguments' raw coordinate/palette arrays retain their native
    /// readable lifetimes through the actual backend's synchronous decode.
    pub unsafe fn makeFromStream(
        &self,
        stream: Option<core::pin::Pin<Box<dyn SkStreamAsset>>>,
        args: &SkFontArguments,
    ) -> TypefaceOwner {
        make_from_stream_args(self, stream, args)
    }
    // Source: include/core/SkFontMgr.h:144-144
    pub unsafe fn makeFromFile(&self, path: *const c_char, ttcIndex: i32) -> TypefaceOwner {
        make_from_file(self, path, ttcIndex)
    }
    // Source: include/core/SkFontMgr.h:146-146
    pub unsafe fn legacyMakeTypeface(
        &self,
        familyName: *const c_char,
        style: SkFontStyle,
    ) -> TypefaceOwner {
        legacy_make_typeface(self, familyName, style)
    }
}
// Source: include/core/SkFontMgr.h:34-34
impl dyn SkFontStyleSet {
    pub fn matchStyleCSS3(&mut self, pattern: &SkFontStyle) -> TypefaceOwner {
        matchStyleCSS3(self, pattern)
    }
}

// Source: include/core/SkFontMgr.h:126-126
// Source: include/core/SkFontMgr.h:133-133
// Source: include/core/SkFontMgr.h:144-144
// Explicit default-argument spellings replace C++ call-site overload insertion.
impl dyn SkFontMgr {
    pub fn makeFromData_default(&self, data: Option<Arc<SkData>>) -> TypefaceOwner {
        self.makeFromData(data, 0)
    }
    pub fn makeFromStream_default(
        &self,
        stream: Option<core::pin::Pin<Box<dyn SkStreamAsset>>>,
    ) -> TypefaceOwner {
        self.makeFromStream_index(stream, 0)
    }
    pub unsafe fn makeFromFile_default(&self, path: *const c_char) -> TypefaceOwner {
        self.makeFromFile(path, 0)
    }
}

// Source: src/core/SkFontMgr.cpp:21-34
struct SkEmptyFontStyleSet {
    base: SkRefCnt,
}
unsafe impl SkRefCntObject for SkEmptyFontStyleSet {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }

    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.base.ref_cnt_base()
    }
    fn ref_(&self) {
        self.base.ref_();
    }
}
impl SkFontStyleSet for SkEmptyFontStyleSet {
    fn count(&mut self) -> i32 {
        0
    }
    fn getStyle(
        &mut self,
        _index: i32,
        _fontStyle: Option<&mut SkFontStyle>,
        _style: Option<&mut SkString>,
    ) {
        crate::SkDEBUGFAIL!(@source b"src/core/SkFontMgr.cpp\0".as_ptr(),25; b"SkFontStyleSet::getStyle called on empty set\0".as_ptr());
    }
    fn createTypeface(&mut self, _index: i32) -> TypefaceOwner {
        crate::SkDEBUGFAIL!(@source b"src/core/SkFontMgr.cpp\0".as_ptr(),28; b"SkFontStyleSet::createTypeface called on empty set\0".as_ptr());
        None
    }
    fn matchStyle(&mut self, _pattern: &SkFontStyle) -> TypefaceOwner {
        None
    }
}
// Source: include/core/SkFontMgr.h:31-31
// Source: src/core/SkFontMgr.cpp:36-38
impl dyn SkFontStyleSet {
    pub fn CreateEmpty() -> sk_sp<dyn SkFontStyleSet> {
        let object: *mut dyn SkFontStyleSet = Box::into_raw(Box::new(SkEmptyFontStyleSet {
            base: SkRefCnt::new(),
        }));
        unsafe { sk_sp::from_raw(object) }
    }
}

// Source: src/core/SkFontMgr.cpp:42-85
// This is the source's real concrete empty manager, not a platform substitute.
struct SkEmptyFontMgr {
    base: SkRefCnt,
}
unsafe impl SkRefCntObject for SkEmptyFontMgr {
    // Native complete-object virtual disposal dispatch; this actual concrete
    // Sized type pairs the original raw allocation with its typed adapter.
    fn internal_dispose_raw_proc(&self) -> crate::include::core::SkRefCnt::SkRawDisposeProc {
        crate::include::core::SkRefCnt::SkRefCntDispose::<Self>
    }

    fn ref_cnt_base(&self) -> &SkRefCntBase {
        self.base.ref_cnt_base()
    }
    fn ref_(&self) {
        self.base.ref_();
    }
}
impl SkFontMgr for SkEmptyFontMgr {
    fn onCountFamilies(&self) -> i32 {
        0
    }
    fn onGetFamilyName(&self, _index: i32, _familyName: Option<&mut SkString>) {
        crate::SkDEBUGFAIL!(@source b"src/core/SkFontMgr.cpp\0".as_ptr(),48; b"onGetFamilyName called with bad index\0".as_ptr());
    }
    fn onCreateStyleSet(&self, _index: i32) -> sk_sp<dyn SkFontStyleSet> {
        crate::SkDEBUGFAIL!(@source b"src/core/SkFontMgr.cpp\0".as_ptr(),51; b"onCreateStyleSet called with bad index\0".as_ptr());
        sk_sp::default()
    }
    unsafe fn onMatchFamily(&self, _familyName: *const c_char) -> sk_sp<dyn SkFontStyleSet> {
        <dyn SkFontStyleSet>::CreateEmpty()
    }
    unsafe fn onMatchFamilyStyle(
        &self,
        _familyName: *const c_char,
        _style: &SkFontStyle,
    ) -> TypefaceOwner {
        None
    }
    unsafe fn onMatchFamilyStyleCharacter(
        &self,
        _familyName: *const c_char,
        _style: &SkFontStyle,
        _bcp47: *const *const c_char,
        _bcp47Count: i32,
        _character: i32,
    ) -> TypefaceOwner {
        None
    }
    fn onMakeFromData(&self, _data: Arc<SkData>, _ttcIndex: i32) -> TypefaceOwner {
        None
    }
    fn onMakeFromStreamIndex(
        &self,
        _stream: core::pin::Pin<Box<dyn SkStreamAsset>>,
        _ttcIndex: i32,
    ) -> TypefaceOwner {
        None
    }
    unsafe fn onMakeFromStreamArgs(
        &self,
        _stream: core::pin::Pin<Box<dyn SkStreamAsset>>,
        _args: &SkFontArguments,
    ) -> TypefaceOwner {
        None
    }
    unsafe fn onMakeFromFile(&self, _path: *const c_char, _ttcIndex: i32) -> TypefaceOwner {
        None
    }
    unsafe fn onLegacyMakeTypeface(
        &self,
        _familyName: *const c_char,
        _style: SkFontStyle,
    ) -> TypefaceOwner {
        None
    }
}
// Source: src/core/SkFontMgr.cpp:87-92
fn emptyOnNull(mut fsset: sk_sp<dyn SkFontStyleSet>) -> sk_sp<dyn SkFontStyleSet> {
    if fsset.is_none() {
        fsset = <dyn SkFontStyleSet>::CreateEmpty();
    }
    fsset
}
// Source: src/core/SkFontMgr.cpp:94-96
fn count_families<T: SkFontMgr + ?Sized>(this: &T) -> i32 {
    this.onCountFamilies()
}
// Source: src/core/SkFontMgr.cpp:98-100
fn get_family_name<T: SkFontMgr + ?Sized>(this: &T, index: i32, familyName: Option<&mut SkString>) {
    this.onGetFamilyName(index, familyName)
}
// Source: src/core/SkFontMgr.cpp:102-104
fn create_style_set<T: SkFontMgr + ?Sized>(this: &T, index: i32) -> sk_sp<dyn SkFontStyleSet> {
    emptyOnNull(this.onCreateStyleSet(index))
}
// Source: src/core/SkFontMgr.cpp:106-108
unsafe fn match_family<T: SkFontMgr + ?Sized>(
    this: &T,
    familyName: *const c_char,
) -> sk_sp<dyn SkFontStyleSet> {
    emptyOnNull(this.onMatchFamily(familyName))
}
// Source: src/core/SkFontMgr.cpp:110-113
unsafe fn match_family_style<T: SkFontMgr + ?Sized>(
    this: &T,
    familyName: *const c_char,
    style: &SkFontStyle,
) -> TypefaceOwner {
    this.onMatchFamilyStyle(familyName, style)
}
// Source: src/core/SkFontMgr.cpp:115-119
unsafe fn match_family_style_character<T: SkFontMgr + ?Sized>(
    this: &T,
    familyName: *const c_char,
    style: &SkFontStyle,
    bcp47: *const *const c_char,
    bcp47Count: i32,
    character: i32,
) -> TypefaceOwner {
    this.onMatchFamilyStyleCharacter(familyName, style, bcp47, bcp47Count, character)
}
// Source: src/core/SkFontMgr.cpp:121-123
unsafe fn match_request<T: SkFontMgr + ?Sized>(this: &T, request: &Request<'_>) -> TypefaceOwner {
    this.onMatch(request)
}
// Source: src/core/SkFontMgr.cpp:124-126
unsafe fn fallback<T: SkFontMgr + ?Sized>(this: &T, request: &Request<'_>) -> TypefaceOwner {
    this.onFallback(request)
}
impl Request<'_> {
    // Source: include/core/SkFontMgr.h:105-105
    // Source: src/core/SkFontMgr.cpp:128-146
    pub fn fontStyleFromModel(&self) -> SkFontStyle {
        let mut weight = 400.0;
        let mut width = 100.0;
        let mut slant = 0.0;
        let mut italic = 0.0;
        for coord in self.model {
            match coord.axis {
                Coordinate::wght => weight = coord.value,
                Coordinate::wdth => width = coord.value,
                Coordinate::slnt => slant = coord.value,
                Coordinate::ital => italic = coord.value,
                _ => {}
            }
        }
        let mut slantEnum = Slant::kUpright_Slant;
        if slant != 0.0 {
            slantEnum = Slant::kOblique_Slant;
        }
        if 0.0 < italic {
            slantEnum = Slant::kItalic_Slant;
        }
        let widthEnum = SkFontDescriptor::SkFontStyleWidthForWidthAxisValue(width);
        SkFontStyle::new(SkScalarRoundToInt(weight), widthEnum, slantEnum)
    }
    // Source: include/core/SkFontMgr.h:106-106
    // Source: src/core/SkFontMgr.cpp:148-159
    pub fn SetModel(fontStyle: SkFontStyle, model: &mut [Coordinate; 4]) {
        model[0] = Coordinate {
            axis: Coordinate::wght,
            value: fontStyle.weight() as f32,
        };
        model[1] = Coordinate {
            axis: Coordinate::wdth,
            value: SkFontDescriptor::SkFontWidthAxisValueForStyleWidth(fontStyle.width()),
        };
        model[2] = Coordinate {
            axis: Coordinate::slnt,
            value: if fontStyle.slant() == Slant::kOblique_Slant {
                -20.0
            } else {
                0.0
            },
        };
        model[3] = Coordinate {
            axis: Coordinate::ital,
            value: if fontStyle.slant() == Slant::kItalic_Slant {
                1.0
            } else {
                0.0
            },
        };
    }
}
// Source: src/core/SkFontMgr.cpp:161-164
unsafe fn on_match<T: SkFontMgr + ?Sized>(this: &T, request: &Request<'_>) -> TypefaceOwner {
    let style = request.fontStyleFromModel();
    match_family_style(this, request.familyName, &style)
}
// Source: src/core/SkFontMgr.cpp:166-175
unsafe fn on_fallback<T: SkFontMgr + ?Sized>(this: &T, request: &Request<'_>) -> TypefaceOwner {
    let style = request.fontStyleFromModel();
    let mut character = 0x20;
    if !request.cmapEntries.is_empty() {
        character = request.cmapEntries[0].character;
    }
    match_family_style_character(
        this,
        request.familyName,
        &style,
        request.bcp47.as_ptr(),
        request.bcp47.len() as i32,
        character,
    )
}
// Source: src/core/SkFontMgr.cpp:177-182
fn make_from_data<T: SkFontMgr + ?Sized>(
    this: &T,
    data: Option<Arc<SkData>>,
    ttcIndex: i32,
) -> TypefaceOwner {
    let Some(data) = data else {
        return None;
    };
    this.onMakeFromData(data, ttcIndex)
}
// Source: src/core/SkFontMgr.cpp:184-190
fn make_from_stream_index<T: SkFontMgr + ?Sized>(
    this: &T,
    stream: Option<core::pin::Pin<Box<dyn SkStreamAsset>>>,
    ttcIndex: i32,
) -> TypefaceOwner {
    let Some(stream) = stream else {
        return None;
    };
    this.onMakeFromStreamIndex(stream, ttcIndex)
}
// Source: src/core/SkFontMgr.cpp:192-198
unsafe fn make_from_stream_args<T: SkFontMgr + ?Sized>(
    this: &T,
    stream: Option<core::pin::Pin<Box<dyn SkStreamAsset>>>,
    args: &SkFontArguments,
) -> TypefaceOwner {
    let Some(stream) = stream else {
        return None;
    };
    this.onMakeFromStreamArgs(stream, args)
}
// Source: src/core/SkFontMgr.cpp:200-205
unsafe fn make_from_file<T: SkFontMgr + ?Sized>(
    this: &T,
    path: *const c_char,
    ttcIndex: i32,
) -> TypefaceOwner {
    if path.is_null() {
        return None;
    }
    this.onMakeFromFile(path, ttcIndex)
}
// Source: src/core/SkFontMgr.cpp:207-209
unsafe fn legacy_make_typeface<T: SkFontMgr + ?Sized>(
    this: &T,
    familyName: *const c_char,
    style: SkFontStyle,
) -> TypefaceOwner {
    this.onLegacyMakeTypeface(familyName, style)
}
// Source: include/core/SkFontMgr.h:149-149
// Source: src/core/SkFontMgr.cpp:211-214
impl dyn SkFontMgr {
    pub fn RefEmpty() -> sk_sp<dyn SkFontMgr> {
        // The permanent native initial reference stays retained in this actual
        // concrete singleton. No platform abstraction is forced to be Send/Sync.
        static SINGLETON: OnceLock<sk_sp<SkEmptyFontMgr>> = OnceLock::new();
        let singleton = SINGLETON.get_or_init(|| {
            sk_sp::new(SkEmptyFontMgr {
                base: SkRefCnt::new(),
            })
        });
        unsafe {
            sk_sp::clone_from_cast(singleton, |pointer| {
                NonNull::new_unchecked(pointer.as_ptr() as *mut dyn SkFontMgr)
            })
        }
    }
}
// Source: src/core/SkFontMgr.cpp:241-334
pub fn matchStyleCSS3<T: SkFontStyleSet + ?Sized>(
    this: &mut T,
    pattern: &SkFontStyle,
) -> TypefaceOwner {
    let count = this.count();
    if count == 0 {
        return None;
    }
    struct Score {
        score: i32,
        index: i32,
    }
    impl core::ops::AddAssign<i32> for Score {
        fn add_assign(&mut self, rhs: i32) {
            self.score += rhs;
        }
    }
    impl core::ops::ShlAssign<i32> for Score {
        fn shl_assign(&mut self, rhs: i32) {
            self.score <<= rhs;
        }
    }
    impl Score {
        fn less(&self, that: &Self) -> bool {
            self.score < that.score
        }
    }
    let mut maxScore = Score { score: 0, index: 0 };
    for i in 0..count {
        let mut current = SkFontStyle::default();
        this.getStyle(i, Some(&mut current), None);
        let mut currentScore = Score { score: 0, index: i };
        if pattern.width() <= SkFontStyle::kNormal_Width {
            if current.width() <= pattern.width() {
                currentScore += 10 - pattern.width() + current.width();
            } else {
                currentScore += 10 - current.width();
            }
        } else {
            if current.width() > pattern.width() {
                currentScore += 10 + pattern.width() - current.width();
            } else {
                currentScore += current.width();
            }
        }
        currentScore <<= 8;
        const _: () = assert!(
            Slant::kUpright_Slant.0 as i32 == 0
                && Slant::kItalic_Slant.0 as i32 == 1
                && Slant::kOblique_Slant.0 as i32 == 2
        );
        debug_assert!(pattern.slant().0 as i32 <= 2 && current.slant().0 as i32 <= 2);
        const score: [[i32; 3]; 3] = [[3, 1, 2], [1, 3, 2], [1, 2, 3]];
        currentScore += score[pattern.slant().0 as usize][current.slant().0 as usize];
        currentScore <<= 8;
        if pattern.weight() == current.weight() {
            currentScore += 1000;
        } else if pattern.weight() < 400 {
            if current.weight() <= pattern.weight() {
                currentScore += 1000 - pattern.weight() + current.weight();
            } else {
                currentScore += 1000 - current.weight();
            }
        } else if pattern.weight() <= 500 {
            if current.weight() >= pattern.weight() && current.weight() <= 500 {
                currentScore += 1000 + pattern.weight() - current.weight();
            } else if current.weight() <= pattern.weight() {
                currentScore += 500 + current.weight();
            } else {
                currentScore += 1000 - current.weight();
            }
        } else if pattern.weight() > 500 {
            if current.weight() > pattern.weight() {
                currentScore += 1000 + pattern.weight() - current.weight();
            } else {
                currentScore += current.weight();
            }
        }
        if maxScore.less(&currentScore) {
            maxScore = currentScore;
        }
    }
    this.createTypeface(maxScore.index)
}
