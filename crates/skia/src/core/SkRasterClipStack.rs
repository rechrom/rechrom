use crate::include::core::SkClipOp::SkClipOp;
use crate::src::core::SkMatrix::SkMatrix;
use crate::src::core::SkPath::SkPath;
use crate::src::core::SkRRect::SkRRect;
use crate::src::core::SkRasterClip::SkRasterClip;
use crate::src::core::SkRect::{SkIRect, SkRect};
use crate::src::core::SkRegion::SkRegion;
use crate::src::core::SkScan::SkScan;
use crate::src::core::SkTBlockList::SkTBlockList;
use core::pin::Pin;
// Source: src/core/SkRasterClipStack.h:98-109
struct Rec {
    rc: SkRasterClip,
    deferred_count: i32,
}
impl Rec {
    fn new(rc: &SkRasterClip) -> Self {
        Self {
            rc: rc.clone(),
            deferred_count: 0,
        }
    }
}
pub struct SkRasterClipStack {
    stack: SkTBlockList<Rec, 16>,
    root_bounds: SkIRect,
    disable_aa: bool,
    #[cfg(sk_debug)]
    counter: i32,
}
impl SkRasterClipStack {
    fn stack(self: Pin<&mut Self>) -> Pin<&mut SkTBlockList<Rec, 16>> {
        unsafe { self.map_unchecked_mut(|this| &mut this.stack) }
    }
    /// Safety: pin then call initialize() before any other operation. This entry
    /// lets a larger native device embed the stack without another allocation.
    pub unsafe fn unbound(width: i32, height: i32) -> Self {
        let bounds = SkIRect::make_wh(width, height);
        Self {
            stack: SkTBlockList::new(),
            root_bounds: bounds,
            disable_aa: SkScan::PathRequiresTiling(&bounds),
            #[cfg(sk_debug)]
            counter: 0,
        }
    }
}
// Source: src/core/SkRasterClipStack.h:18-23
impl SkRasterClipStack {
    pub fn new(width: i32, height: i32) -> Pin<Box<Self>> {
        let mut stack = Box::pin(unsafe { Self::unbound(width, height) });
        stack.as_mut().initialize();
        stack
    }
    pub fn initialize(mut self: Pin<&mut Self>) {
        assert!(self.stack.empty());
        let rc = SkRasterClip::from_rect(self.root_bounds);
        self.as_mut().stack().emplace_back(|| Rec::new(&rc));
        drop(rc);
        crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 22; @condition b"fStack.count() == 1\0".as_ptr(); (self.stack.count()) == (1));
    }
}
// Source: src/core/SkRasterClipStack.h:25-32
impl SkRasterClipStack {
    pub fn set_new_size(mut self: Pin<&mut Self>, w: i32, h: i32) {
        unsafe { self.as_mut().get_unchecked_mut() }
            .root_bounds
            .set_xywh(0, 0, w, h);
        let root_bounds = self.root_bounds;
        crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 28; @condition b"fStack.count() == 1\0".as_ptr(); (self.stack.count()) == (1));
        let rec = self.stack().back_mut();
        crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 30; @condition b"rec.fDeferredCount == 0\0".as_ptr(); (rec.deferred_count) == (0));
        rec.rc.set_rect(root_bounds);
    }
}
// Source: src/core/SkRasterClipStack.h:34-40
impl SkRasterClipStack {
    pub fn rc(&self) -> &SkRasterClip {
        &self.stack.back().rc
    }
    pub fn save(mut self: Pin<&mut Self>) {
        #[cfg(sk_debug)]
        {
            unsafe { self.as_mut().get_unchecked_mut() }.counter += 1;
        }
        let rec = self.stack().back_mut();
        crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 38; @condition b"fStack.back().fDeferredCount >= 0\0".as_ptr(); rec.deferred_count >= 0);
        rec.deferred_count = rec.deferred_count.wrapping_add(1);
    }
}
// Source: src/core/SkRasterClipStack.h:42-51
impl SkRasterClipStack {
    pub fn restore(mut self: Pin<&mut Self>) {
        #[cfg(sk_debug)]
        {
            let this = unsafe { self.as_mut().get_unchecked_mut() };
            this.counter -= 1;
            crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 44; @condition b"fCounter >= 0\0".as_ptr(); this.counter >= 0);
        }
        let rec = self.as_mut().stack().back_mut();
        rec.deferred_count = rec.deferred_count.wrapping_sub(1);
        if rec.deferred_count < 0 {
            crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 47; @condition b"fStack.back().fDeferredCount == -1\0".as_ptr(); (rec.deferred_count) == ( -1));
            crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 48; @condition b"fStack.count() > 1\0".as_ptr(); self.stack.count() > 1);
            self.stack().pop_back();
        }
    }
}
// Source: src/core/SkRasterClipStack.h:53-66
impl SkRasterClipStack {
    pub fn clip_rect(
        mut self: Pin<&mut Self>,
        ctm: &SkMatrix,
        rect: &SkRect,
        op: SkClipOp,
        aa: bool,
    ) {
        let aa = self.final_aa(aa);
        self.as_mut().writable_rc().op_rect(rect, ctm, op, aa);
        self.validate();
    }
    pub fn clip_rrect(
        mut self: Pin<&mut Self>,
        ctm: &SkMatrix,
        rrect: &SkRRect,
        op: SkClipOp,
        aa: bool,
    ) {
        let aa = self.final_aa(aa);
        self.as_mut().writable_rc().op_rrect(rrect, ctm, op, aa);
        self.validate();
    }
    pub fn clip_path(
        mut self: Pin<&mut Self>,
        ctm: &SkMatrix,
        path: &SkPath,
        op: SkClipOp,
        aa: bool,
    ) {
        let aa = self.final_aa(aa);
        self.as_mut().writable_rc().op_path(path, ctm, op, aa);
        self.validate();
    }
}
// Source: src/core/SkRasterClipStack.h:68-71
// Current canonical SkShader retains Arc ownership; native intrusive migration pending.
impl SkRasterClipStack {
    pub fn clip_shader(
        mut self: Pin<&mut Self>,
        shader: Option<std::sync::Arc<dyn crate::src::shaders::SkShader::SkShader>>,
    ) {
        self.as_mut().writable_rc().op_shader(shader);
        self.validate();
    }
}
// Source: src/core/SkRasterClipStack.h:73-76
impl SkRasterClipStack {
    pub fn clip_region(mut self: Pin<&mut Self>, region: &SkRegion, op: SkClipOp) {
        self.as_mut().writable_rc().op_region(region, op);
        self.validate();
    }
}
// Source: src/core/SkRasterClipStack.h:78-85
impl SkRasterClipStack {
    pub fn replace_clip(self: Pin<&mut Self>, rect: SkIRect) {
        let mut dev = rect;
        if !dev.intersect(&self.root_bounds) {
            self.writable_rc().set_empty();
        } else {
            self.writable_rc().set_rect(dev);
        }
    }
}
// Source: src/core/SkRasterClipStack.h:87-96
impl SkRasterClipStack {
    pub fn validate(&self) {
        #[cfg(sk_debug)]
        {
            let clip = self.rc();
            if self.root_bounds.is_empty() {
                debug_assert!(clip.is_empty());
            } else if !clip.is_empty() {
                crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 93; @condition b"fRootBounds.contains(clip.getBounds())\0".as_ptr(); self.root_bounds.contains_rect(clip.get_bounds()));
            }
        }
    }
}
// Source: src/core/SkRasterClipStack.h:111-120
impl SkRasterClipStack {
    fn writable_rc(mut self: Pin<&mut Self>) -> &mut SkRasterClip {
        crate::SkASSERT!(@source b"src/core/SkRasterClipStack.h\0".as_ptr(), 112; @condition b"fStack.back().fDeferredCount >= 0\0".as_ptr(); self.stack.back().deferred_count >= 0);
        if self.stack.back().deferred_count > 0 {
            self.as_mut().stack().back_mut().deferred_count -= 1;
            self.as_mut()
                .stack()
                .emplace_from_back(|rec| Rec::new(&rec.rc));
        }
        &mut self.stack().back_mut().rc
    }
    fn final_aa(&self, aa: bool) -> bool {
        aa && !self.disable_aa
    }
}
