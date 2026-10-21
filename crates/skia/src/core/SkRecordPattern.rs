use crate::src::core::{
    SkPaint::SkPaint,
    SkRecord::{RecordMutator, SkRecord},
    SkRecords::{self, RecordCommand, Type},
};
use core::{marker::PhantomData, ptr};
// Native template overload/SFINAE protocol. Each concrete command supplies its own paint member projection.
pub unsafe trait MatchCommand: RecordCommand {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint;
}
unsafe impl MatchCommand for SkRecords::NoOp {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::Restore {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::Save {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::SaveLayer {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}
unsafe impl MatchCommand for SkRecords::SaveBehind {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::SetMatrix {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::SetM44 {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::Concat {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::Concat44 {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::Translate {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::Scale {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::ClipPath {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::ClipRRect {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::ClipRect {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::ClipRegion {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::ClipShader {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::ResetClip {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::DrawArc {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawDRRect {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawDrawable {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::DrawImage {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}
unsafe impl MatchCommand for SkRecords::DrawImageLattice {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}
unsafe impl MatchCommand for SkRecords::DrawImageRect {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}
unsafe impl MatchCommand for SkRecords::DrawOval {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawPaint {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawBehind {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawPath {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawPicture {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}
unsafe impl MatchCommand for SkRecords::DrawPoints {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawRRect {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawRect {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawRegion {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawTextBlob {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawSlug {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawPatch {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawAtlas {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}
unsafe impl MatchCommand for SkRecords::DrawVertices {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawMesh {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { &mut *(*ptr).paint }
    }
}
unsafe impl MatchCommand for SkRecords::DrawShadowRec {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::DrawAnnotation {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::DrawEdgeAAQuad {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        ptr::null_mut()
    }
}
unsafe impl MatchCommand for SkRecords::DrawEdgeAAImageSet {
    unsafe fn paint(ptr: *mut Self) -> *mut SkPaint {
        unsafe { (*ptr).paint.as_ptr() }
    }
}

// Source: src/core/SkRecordPattern.h:19-41
pub struct Is<T> {
    fPtr: *mut T,
}
impl<T> Default for Is<T> {
    fn default() -> Self {
        Self {
            fPtr: ptr::null_mut(),
        }
    }
}
impl<T> Is<T> {
    pub fn get(&self) -> *mut T {
        self.fPtr
    }
}
pub trait Capture {
    type Value;
    fn get(&self) -> *mut Self::Value;
}
impl<T> Capture for Is<T> {
    type Value = T;
    fn get(&self) -> *mut T {
        self.fPtr
    }
}
pub trait Matcher: Default {
    unsafe fn matches<T: MatchCommand>(&mut self, ptr: *mut T) -> bool;
    unsafe fn match_first(&mut self, record: &mut SkRecord, i: i32) -> i32
    where
        Self: Sized,
    {
        if i < record.count() && unsafe { record.mutate(i, self) } {
            i + 1
        } else {
            0
        }
    }
}
impl<T: RecordCommand> Matcher for Is<T> {
    unsafe fn matches<U: MatchCommand>(&mut self, ptr: *mut U) -> bool {
        if T::TYPE == U::TYPE {
            self.fPtr = ptr.cast();
            true
        } else {
            self.fPtr = ptr::null_mut();
            false
        }
    }
}
// Source: src/core/SkRecordPattern.h:44-76
pub struct IsDraw {
    fPaint: *mut SkPaint,
}
impl Default for IsDraw {
    fn default() -> Self {
        Self {
            fPaint: ptr::null_mut(),
        }
    }
}
impl IsDraw {
    pub fn get(&self) -> *mut SkPaint {
        self.fPaint
    }
}
impl Capture for IsDraw {
    type Value = SkPaint;
    fn get(&self) -> *mut SkPaint {
        self.fPaint
    }
}
impl Matcher for IsDraw {
    unsafe fn matches<T: MatchCommand>(&mut self, ptr: *mut T) -> bool {
        if T::TAGS & SkRecords::kDrawWithPaint_Tag == SkRecords::kDrawWithPaint_Tag {
            self.fPaint = unsafe { T::paint(ptr) };
            true
        } else {
            self.fPaint = ptr::null_mut();
            T::TAGS & SkRecords::kDraw_Tag != 0
        }
    }
}
// Source: src/core/SkRecordPattern.h:79-119
pub struct IsSingleDraw {
    fPaint: *mut SkPaint,
}
impl Default for IsSingleDraw {
    fn default() -> Self {
        Self {
            fPaint: ptr::null_mut(),
        }
    }
}
impl IsSingleDraw {
    pub fn get(&self) -> *mut SkPaint {
        self.fPaint
    }
}
impl Capture for IsSingleDraw {
    type Value = SkPaint;
    fn get(&self) -> *mut SkPaint {
        self.fPaint
    }
}
impl Matcher for IsSingleDraw {
    unsafe fn matches<T: MatchCommand>(&mut self, ptr: *mut T) -> bool {
        if T::TAGS & SkRecords::kDraw_Tag == 0 || T::TAGS & SkRecords::kMultiDraw_Tag != 0 {
            self.fPaint = ptr::null_mut();
            false
        } else {
            self.fPaint = if T::TAGS & SkRecords::kHasPaint_Tag != 0 {
                unsafe { T::paint(ptr) }
            } else {
                ptr::null_mut()
            };
            true
        }
    }
}
// Source: src/core/SkRecordPattern.h:122-127
pub struct Not<M>(PhantomData<M>);
impl<M> Default for Not<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<M: Matcher> Matcher for Not<M> {
    unsafe fn matches<T: MatchCommand>(&mut self, ptr: *mut T) -> bool {
        !unsafe { M::default().matches(ptr) }
    }
}
// Source: src/core/SkRecordPattern.h:130-141
pub struct Or<F, R = OrEnd>(PhantomData<(F, R)>);
pub struct OrEnd;
impl Default for OrEnd {
    fn default() -> Self {
        Self
    }
}
impl Matcher for OrEnd {
    unsafe fn matches<T: MatchCommand>(&mut self, _: *mut T) -> bool {
        false
    }
}
impl<F, R> Default for Or<F, R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<F: Matcher, R: Matcher> Matcher for Or<F, R> {
    unsafe fn matches<T: MatchCommand>(&mut self, ptr: *mut T) -> bool {
        unsafe { F::default().matches(ptr) || R::default().matches(ptr) }
    }
}
// Source: src/core/SkRecordPattern.h:145-151
pub struct Greedy<M>(PhantomData<M>);
impl<M> Default for Greedy<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
impl<M: Matcher> Matcher for Greedy<M> {
    unsafe fn matches<T: MatchCommand>(&mut self, ptr: *mut T) -> bool {
        unsafe { M::default().matches(ptr) }
    }
    unsafe fn match_first(&mut self, record: &mut SkRecord, mut i: i32) -> i32 {
        while i < record.count() {
            if !unsafe { record.mutate(i, self) } {
                return i;
            }
            i += 1;
        }
        0
    }
}
// Source: src/core/SkRecordPattern.h:161-168
#[derive(Default)]
pub struct PatternEnd;
pub trait PatternMatch: Default {
    unsafe fn match_at(&mut self, record: &mut SkRecord, i: i32) -> i32;
}
impl PatternMatch for PatternEnd {
    unsafe fn match_at(&mut self, _: &mut SkRecord, i: i32) -> i32 {
        i
    }
}
// Source: src/core/SkRecordPattern.h:170-213
pub struct Pattern<F, R = PatternEnd> {
    pub fFirst: F,
    pub fRest: R,
}
impl<F: Default, R: Default> Default for Pattern<F, R> {
    fn default() -> Self {
        Self {
            fFirst: F::default(),
            fRest: R::default(),
        }
    }
}
impl<F: Matcher, R: PatternMatch> PatternMatch for Pattern<F, R> {
    unsafe fn match_at(&mut self, record: &mut SkRecord, i: i32) -> i32 {
        let i = unsafe { self.fFirst.match_first(record, i) };
        if i > 0 {
            unsafe { self.fRest.match_at(record, i) }
        } else {
            0
        }
    }
}
impl<F: Matcher, R: PatternMatch> Pattern<F, R> {
    pub unsafe fn search(&mut self, record: &mut SkRecord, begin: &mut i32, end: &mut i32) -> bool {
        *begin = *end;
        while *begin < record.count() {
            *end = unsafe { self.match_at(record, *begin) };
            if *end != 0 {
                return true;
            }
            *begin += 1;
        }
        false
    }
}
impl<F: Capture, R> Pattern<F, R> {
    pub fn first(&self) -> *mut F::Value {
        self.fFirst.get()
    }
}
impl<F, S: Capture, R> Pattern<F, Pattern<S, R>> {
    pub fn second(&self) -> *mut S::Value {
        self.fRest.first()
    }
}
impl<F, S, T: Capture, R> Pattern<F, Pattern<S, Pattern<T, R>>> {
    pub fn third(&self) -> *mut T::Value {
        self.fRest.fRest.first()
    }
}
impl<F, S, T, U: Capture, R> Pattern<F, Pattern<S, Pattern<T, Pattern<U, R>>>> {
    pub fn fourth(&self) -> *mut U::Value {
        self.fRest.fRest.fRest.first()
    }
}
macro_rules! matcher_mutator{($($name:ident:$variant:ident),*)=>{impl<M:Matcher> RecordMutator for M{type Output=bool;$(unsafe fn $name(&mut self,ptr:*mut SkRecords::$name)->bool{unsafe{self.matches(ptr)}})*}};}
SkRecords::SK_RECORD_TYPES!(matcher_mutator);
