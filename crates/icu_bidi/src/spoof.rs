//! ICU primitives for Chromium url_formatter/spoof_checks. Policy, script
//! allowlists, skeleton rules and top-domain decisions stay in that owner.
use std::ptr::NonNull;
pub enum USet {}
pub enum USpoofChecker {}
pub enum URegularExpression {}
macro_rules! ffi {($($name:ident($($arg:ident:$ty:ty),*)$(->$ret:ty)?;)+)=>{unsafe extern "C"{$(#[link_name=concat!(stringify!($name),"_",env!("ICU_MAJOR"))]fn $name($($arg:$ty),*)$(->$ret)?;)+}};}
ffi! {
    uset_openEmpty()->*mut USet;
    uset_openPattern(pattern:*const u16,length:i32,status:*mut i32)->*mut USet;
    uset_clone(set:*const USet)->*mut USet;
    uset_close(set:*mut USet);
    uset_freeze(set:*mut USet);
    uset_add(set:*mut USet,ch:i32);
    uset_addRange(set:*mut USet,start:i32,end:i32);
    uset_addAll(set:*mut USet,other:*const USet);
    uset_remove(set:*mut USet,ch:i32);
    uset_removeRange(set:*mut USet,start:i32,end:i32);
    uset_contains(set:*const USet,ch:i32)->i8;
    uset_containsAll(set:*const USet,other:*const USet)->i8;
    uset_isEmpty(set:*const USet)->i8;
    uset_span(set:*const USet,text:*const u16,length:i32,condition:i32)->i32;
    uspoof_open(status:*mut i32)->*mut USpoofChecker;
    uspoof_close(checker:*mut USpoofChecker);
    uspoof_setRestrictionLevel(checker:*mut USpoofChecker,level:i32);
    uspoof_getChecks(checker:*const USpoofChecker,status:*mut i32)->i32;
    uspoof_setChecks(checker:*mut USpoofChecker,checks:i32,status:*mut i32);
    uspoof_getRecommendedSet(status:*mut i32)->*const USet;
    uspoof_getInclusionSet(status:*mut i32)->*const USet;
    uspoof_setAllowedChars(checker:*mut USpoofChecker,set:*const USet,status:*mut i32);
    uspoof_check(checker:*const USpoofChecker,text:*const u16,length:i32,position:*mut i32,status:*mut i32)->i32;
    uspoof_getSkeleton(checker:*const USpoofChecker,kind:u32,text:*const u16,length:i32,output:*mut u16,capacity:i32,status:*mut i32)->i32;
    uregex_open(pattern:*const u16,length:i32,flags:u32,parse_error:*mut std::ffi::c_void,status:*mut i32)->*mut URegularExpression;
    uregex_close(regex:*mut URegularExpression);
    uregex_setText(regex:*mut URegularExpression,text:*const u16,length:i32,status:*mut i32);
    uregex_find(regex:*mut URegularExpression,start:i32,status:*mut i32)->i8;
}
fn status(value: i32) -> Result<(), i32> {
    if value > 0 {
        Err(value)
    } else {
        Ok(())
    }
}
fn length(value: usize) -> i32 {
    i32::try_from(value).expect("ICU UTF16 length")
}
pub struct UnicodeSet(NonNull<USet>);
impl UnicodeSet {
    pub fn Empty() -> Self {
        Self(NonNull::new(unsafe { uset_openEmpty() }).expect("ICU UnicodeSet allocation"))
    }
    pub fn Pattern(pattern: &[u16]) -> Result<Self, i32> {
        let mut error = 0;
        let set = NonNull::new(unsafe {
            uset_openPattern(pattern.as_ptr(), length(pattern.len()), &mut error)
        })
        .map(Self);
        status(error)?;
        set.ok_or(7)
    }
    pub fn Freeze(&mut self) {
        unsafe { uset_freeze(self.0.as_ptr()) }
    }
    pub fn Add(&mut self, ch: i32) {
        unsafe { uset_add(self.0.as_ptr(), ch) }
    }
    pub fn AddRange(&mut self, start: i32, end: i32) {
        unsafe { uset_addRange(self.0.as_ptr(), start, end) }
    }
    pub fn AddAll(&mut self, other: &Self) {
        unsafe { uset_addAll(self.0.as_ptr(), other.0.as_ptr()) }
    }
    pub fn Remove(&mut self, ch: i32) {
        unsafe { uset_remove(self.0.as_ptr(), ch) }
    }
    pub fn RemoveRange(&mut self, start: i32, end: i32) {
        unsafe { uset_removeRange(self.0.as_ptr(), start, end) }
    }
    pub fn Contains(&self, ch: i32) -> bool {
        unsafe { uset_contains(self.0.as_ptr(), ch) != 0 }
    }
    pub fn ContainsAll(&self, other: &Self) -> bool {
        unsafe { uset_containsAll(self.0.as_ptr(), other.0.as_ptr()) != 0 }
    }
    pub fn IsEmpty(&self) -> bool {
        unsafe { uset_isEmpty(self.0.as_ptr()) != 0 }
    }
    pub fn ContainsSomeText(&self, text: &[u16]) -> bool {
        unsafe {
            uset_span(self.0.as_ptr(), text.as_ptr(), length(text.len()), 0) < length(text.len())
        }
    }
    pub fn ContainsAllText(&self, text: &[u16]) -> bool {
        unsafe {
            uset_span(self.0.as_ptr(), text.as_ptr(), length(text.len()), 1) == length(text.len())
        }
    }
    pub fn RecommendedAndInclusion() -> Result<Self, i32> {
        let mut error = 0;
        let recommended = unsafe { uspoof_getRecommendedSet(&mut error) };
        status(error)?;
        let inclusion = unsafe { uspoof_getInclusionSet(&mut error) };
        status(error)?;
        let result = Self::Empty();
        unsafe {
            uset_addAll(result.0.as_ptr(), recommended);
            uset_addAll(result.0.as_ptr(), inclusion)
        };
        Ok(result)
    }
}
impl Clone for UnicodeSet {
    fn clone(&self) -> Self {
        Self(NonNull::new(unsafe { uset_clone(self.0.as_ptr()) }).expect("ICU UnicodeSet clone"))
    }
}
impl Drop for UnicodeSet {
    fn drop(&mut self) {
        unsafe { uset_close(self.0.as_ptr()) }
    }
}
// Mutable builders can move to another thread, but are never shared.
unsafe impl Send for UnicodeSet {}
#[derive(Clone)]
pub struct FrozenUnicodeSet(UnicodeSet);
impl UnicodeSet {
    pub fn IntoFrozen(mut self) -> FrozenUnicodeSet {
        self.Freeze();
        FrozenUnicodeSet(self)
    }
}
impl std::ops::Deref for FrozenUnicodeSet {
    type Target = UnicodeSet;
    fn deref(&self) -> &UnicodeSet {
        &self.0
    }
}
// uniset.h:583-584: frozen sets are immutable and explicitly thread-safe.
unsafe impl Send for FrozenUnicodeSet {}
unsafe impl Sync for FrozenUnicodeSet {}
pub struct SpoofChecker(NonNull<USpoofChecker>);
impl SpoofChecker {
    pub fn Open() -> Result<Self, i32> {
        let mut error = 0;
        let checker = NonNull::new(unsafe { uspoof_open(&mut error) }).map(Self);
        status(error)?;
        checker.ok_or(7)
    }
    pub fn SetRestrictionLevel(&mut self, level: i32) {
        unsafe { uspoof_setRestrictionLevel(self.0.as_ptr(), level) }
    }
    pub fn Checks(&self) -> Result<i32, i32> {
        let mut error = 0;
        let result = unsafe { uspoof_getChecks(self.0.as_ptr(), &mut error) };
        status(error)?;
        Ok(result)
    }
    pub fn SetChecks(&mut self, checks: i32) -> Result<(), i32> {
        let mut error = 0;
        unsafe { uspoof_setChecks(self.0.as_ptr(), checks, &mut error) };
        status(error)
    }
    pub fn SetAllowedUnicodeSet(&mut self, set: &UnicodeSet) -> Result<(), i32> {
        let mut error = 0;
        unsafe { uspoof_setAllowedChars(self.0.as_ptr(), set.0.as_ptr(), &mut error) };
        status(error)
    }
    pub fn Check(&self, text: &[u16]) -> Result<i32, i32> {
        let mut error = 0;
        let result = unsafe {
            uspoof_check(
                self.0.as_ptr(),
                text.as_ptr(),
                length(text.len()),
                std::ptr::null_mut(),
                &mut error,
            )
        };
        status(error)?;
        Ok(result)
    }
    pub fn Skeleton(&self, text: &[u16]) -> Result<Vec<u16>, i32> {
        let mut output = vec![0; text.len().max(1)];
        loop {
            let mut error = 0;
            let written = unsafe {
                uspoof_getSkeleton(
                    self.0.as_ptr(),
                    0,
                    text.as_ptr(),
                    length(text.len()),
                    output.as_mut_ptr(),
                    length(output.len()),
                    &mut error,
                )
            };
            if error == 15 && written >= 0 {
                output.resize(written as usize, 0);
                continue;
            }
            status(error)?;
            if written < 0 {
                return Err(1);
            }
            output.truncate(written as usize);
            return Ok(output);
        }
    }
}
impl Drop for SpoofChecker {
    fn drop(&mut self) {
        unsafe { uspoof_close(self.0.as_ptr()) }
    }
}
// uspoof.h: immutable checking/skeleton operations are thread-safe; setters
// require &mut self and finish before the original source owner is published.
unsafe impl Send for SpoofChecker {}
unsafe impl Sync for SpoofChecker {}
/// A mutable matcher stays on its original thread. Its text storage outlives
/// every native find operation and is replaced before the next setText call.
pub struct Regex {
    raw: NonNull<URegularExpression>,
    text: Vec<u16>,
    retired: Vec<Vec<u16>>,
}
impl Regex {
    pub fn Open(pattern: &[u16]) -> Result<Self, i32> {
        let mut error = 0;
        let raw = NonNull::new(unsafe {
            uregex_open(
                pattern.as_ptr(),
                length(pattern.len()),
                0,
                std::ptr::null_mut(),
                &mut error,
            )
        });
        let owner = raw.map(|raw| Self {
            raw,
            text: vec![],
            retired: vec![],
        });
        status(error)?;
        owner.ok_or(7)
    }
    pub fn Find(&mut self, text: &[u16]) -> Result<bool, i32> {
        let previous = std::mem::replace(&mut self.text, text.to_vec());
        let mut error = 0;
        unsafe {
            uregex_setText(
                self.raw.as_ptr(),
                self.text.as_ptr(),
                length(self.text.len()),
                &mut error,
            )
        };
        if let Err(error) = status(error) {
            self.retired.push(previous);
            return Err(error);
        }
        drop(previous);
        self.retired.clear();
        let found = unsafe { uregex_find(self.raw.as_ptr(), 0, &mut error) };
        status(error)?;
        Ok(found != 0)
    }
}
impl Drop for Regex {
    fn drop(&mut self) {
        unsafe { uregex_close(self.raw.as_ptr()) }
    }
}

pub enum UTransliterator {}
ffi! {
    utrans_openU(id:*const u16,id_length:i32,direction:i32,rules:*const u16,rules_length:i32,parse_error:*mut std::ffi::c_void,status:*mut i32)->*mut UTransliterator;
    utrans_close(trans:*mut UTransliterator);
    utrans_transUChars(trans:*const UTransliterator,text:*mut u16,text_length:*mut i32,capacity:i32,start:i32,limit:*mut i32,status:*mut i32);
}
/// ICU mutable transliteration object. The url_formatter owner serializes its
/// use, matching base::i18n::Transliterator's owning lock.
pub struct Transliterator(NonNull<UTransliterator>);
unsafe impl Send for Transliterator {}
impl Transliterator {
    pub fn FromRules(id: &str, rules: &str) -> Result<Self, i32> {
        let id: Vec<_> = id.encode_utf16().collect();
        let rules: Vec<_> = rules.encode_utf16().collect();
        let mut error = 0;
        let result = NonNull::new(unsafe {
            utrans_openU(
                id.as_ptr(),
                length(id.len()),
                0,
                rules.as_ptr(),
                length(rules.len()),
                std::ptr::null_mut(),
                &mut error,
            )
        })
        .map(Self);
        status(error)?;
        result.ok_or(7)
    }
    pub fn Transliterate(&mut self, input: &[u16]) -> Result<Vec<u16>, i32> {
        let mut capacity = input.len().saturating_mul(2).max(1);
        loop {
            let mut text = vec![0; capacity];
            text[..input.len()].copy_from_slice(input);
            let mut text_length = length(input.len());
            let mut limit = text_length;
            let mut error = 0;
            unsafe {
                utrans_transUChars(
                    self.0.as_ptr(),
                    text.as_mut_ptr(),
                    &mut text_length,
                    length(capacity),
                    0,
                    &mut limit,
                    &mut error,
                )
            };
            if error == 15 {
                capacity = usize::try_from(text_length).map_err(|_| 1)?;
                continue;
            }
            status(error)?;
            text.truncate(usize::try_from(text_length).map_err(|_| 1)?);
            return Ok(text);
        }
    }
}
impl Drop for Transliterator {
    fn drop(&mut self) {
        unsafe { utrans_close(self.0.as_ptr()) }
    }
}
