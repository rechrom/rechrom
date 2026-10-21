use super::SkCodec::FrameInfo;
use super::SkEncodedInfo::Alpha;
use crate::include::codec::SkCodecAnimation::{Blend, DisposalMethod};
use crate::src::core::SkRect::SkIRect;
// Source: src/codec/SkFrameHolder.h:25-34
// Source: src/codec/SkFrameHolder.h:153-161
// Actual noncopyable base state. Rust moves implement the explicit native move;
// no Clone or concrete implementation of the pure virtual alpha query is added.
pub struct SkFrame {
    pub(super) fId: i32,
    pub(super) fHasAlpha: bool,
    pub(super) fRequiredFrame: i32,
    pub(super) fRect: SkIRect,
    pub(super) fDisposalMethod: DisposalMethod,
    pub(super) fDuration: i32,
    pub(super) fBlend: Blend,
}
impl SkFrame {
    const kUninitialized: i32 = -2;
    pub fn new(id: i32) -> Self {
        Self {
            fId: id,
            fHasAlpha: false,
            fRequiredFrame: Self::kUninitialized,
            fDisposalMethod: DisposalMethod::kKeep,
            fDuration: 0,
            fBlend: Blend::kSrcOver,
            fRect: SkIRect::make_empty(),
        }
    }
    // Source: src/codec/SkFrameHolder.h:52-52
    pub fn frameId(&self) -> i32 {
        self.fId
    }
    // Source: src/codec/SkFrameHolder.h:69-74
    pub fn hasAlpha(&self) -> bool {
        self.fHasAlpha
    }
    pub fn setHasAlpha(&mut self, alpha: bool) {
        self.fHasAlpha = alpha;
    }
    // Source: src/codec/SkFrameHolder.h:80-80
    pub fn reachedStartOfData(&self) -> bool {
        self.fRequiredFrame != Self::kUninitialized
    }
    // Source: src/codec/SkFrameHolder.h:87-90
    pub fn getRequiredFrame(&self) -> i32 {
        crate::SkASSERT!(@source b"src/codec/SkFrameHolder.h\0".as_ptr(),88; @condition b"this->reachedStartOfData()\0".as_ptr(); self.reachedStartOfData());
        self.fRequiredFrame
    }
    // Source: src/codec/SkFrameHolder.h:95-95
    pub fn setRequiredFrame(&mut self, req: i32) {
        self.fRequiredFrame = req;
    }
    // Source: src/codec/SkFrameHolder.h:100-102
    pub fn setXYWH(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.fRect.set_xywh(x, y, width, height);
    }
    // Source: src/codec/SkFrameHolder.h:107-112
    pub fn frameRect(&self) -> SkIRect {
        self.fRect
    }
    pub fn xOffset(&self) -> i32 {
        self.fRect.fLeft
    }
    pub fn yOffset(&self) -> i32 {
        self.fRect.fTop
    }
    pub fn width(&self) -> i32 {
        self.fRect.width()
    }
    pub fn height(&self) -> i32 {
        self.fRect.height()
    }
    // Source: src/codec/SkFrameHolder.h:114-142
    pub fn getDisposalMethod(&self) -> DisposalMethod {
        self.fDisposalMethod
    }
    pub fn setDisposalMethod(&mut self, method: DisposalMethod) {
        self.fDisposalMethod = method;
    }
    pub fn setDuration(&mut self, duration: i32) {
        self.fDuration = duration;
    }
    pub fn getDuration(&self) -> i32 {
        self.fDuration
    }
    pub fn setBlend(&mut self, blend: Blend) {
        self.fBlend = blend;
    }
    pub fn getBlend(&self) -> Blend {
        self.fBlend
    }
}
// Source: src/codec/SkFrameHolder.h:36-47
// Source: src/codec/SkFrameHolder.h:61-63
// Source: src/codec/SkFrameHolder.h:150-150
// The real subclass supplies onReportedAlpha and owns this one base state.
// Trait-object destruction invokes the actual complete object's Rust destructor.
pub trait SkFrameVirtual {
    fn frame(&self) -> &SkFrame;
    fn frame_mut(&mut self) -> &mut SkFrame;
    fn onReportedAlpha(&self) -> Alpha;
    fn reportedAlpha(&self) -> Alpha {
        self.onReportedAlpha()
    }
    // Source: src/codec/SkFrameHolder.h:52-142
    // Native inherited nonvirtual interface projects the same embedded state.
    fn frameId(&self) -> i32 {
        self.frame().frameId()
    }
    fn hasAlpha(&self) -> bool {
        self.frame().hasAlpha()
    }
    fn setHasAlpha(&mut self, value: bool) {
        self.frame_mut().setHasAlpha(value);
    }
    fn reachedStartOfData(&self) -> bool {
        self.frame().reachedStartOfData()
    }
    fn getRequiredFrame(&self) -> i32 {
        self.frame().getRequiredFrame()
    }
    fn setRequiredFrame(&mut self, value: i32) {
        self.frame_mut().setRequiredFrame(value);
    }
    fn setXYWH(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.frame_mut().setXYWH(x, y, w, h);
    }
    fn frameRect(&self) -> SkIRect {
        self.frame().frameRect()
    }
    fn xOffset(&self) -> i32 {
        self.frame().xOffset()
    }
    fn yOffset(&self) -> i32 {
        self.frame().yOffset()
    }
    fn width(&self) -> i32 {
        self.frame().width()
    }
    fn height(&self) -> i32 {
        self.frame().height()
    }
    fn getDisposalMethod(&self) -> DisposalMethod {
        self.frame().getDisposalMethod()
    }
    fn setDisposalMethod(&mut self, value: DisposalMethod) {
        self.frame_mut().setDisposalMethod(value);
    }
    fn setDuration(&mut self, value: i32) {
        self.frame_mut().setDuration(value);
    }
    fn getDuration(&self) -> i32 {
        self.frame().getDuration()
    }
    fn setBlend(&mut self, value: Blend) {
        self.frame_mut().setBlend(value);
    }
    fn getBlend(&self) -> Blend {
        self.frame().getBlend()
    }
    // Source: src/codec/SkFrameHolder.h:147-147
    fn fillIn(&self, info: &mut FrameInfo, fullyReceived: bool) {
        super::SkCodec::SkFrame_fillIn(self, info, fullyReceived);
    }
}
// Source: src/codec/SkFrameHolder.h:170-173
// Source: src/codec/SkFrameHolder.h:199-200
pub struct SkFrameHolder {
    pub fScreenWidth: i32,
    pub fScreenHeight: i32,
}
impl SkFrameHolder {
    pub const fn new() -> Self {
        Self {
            fScreenWidth: 0,
            fScreenHeight: 0,
        }
    }
    // Source: src/codec/SkFrameHolder.h:181-182
    pub fn screenWidth(&self) -> i32 {
        self.fScreenWidth
    }
    pub fn screenHeight(&self) -> i32 {
        self.fScreenHeight
    }
}
// Source: src/codec/SkFrameHolder.h:175-175
// Source: src/codec/SkFrameHolder.h:189-202
pub trait SkFrameHolderVirtual {
    fn frame_holder(&self) -> &SkFrameHolder;
    fn onGetFrame(&self, i: i32) -> Option<&dyn SkFrameVirtual>;
    // Source: src/codec/SkFrameHolder.h:181-182
    fn screenWidth(&self) -> i32 {
        self.frame_holder().screenWidth()
    }
    fn screenHeight(&self) -> i32 {
        self.frame_holder().screenHeight()
    }
    fn getFrame(&self, i: i32) -> Option<&dyn SkFrameVirtual> {
        self.onGetFrame(i)
    }
    // Source: src/codec/SkFrameHolder.h:189-189
    fn setAlphaAndRequiredFrame(&self, frame: &mut dyn SkFrameVirtual) {
        super::SkCodec::SkFrameHolder_setAlphaAndRequiredFrame(self, frame);
    }
}
