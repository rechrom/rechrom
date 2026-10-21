//! Native abstract device clipping state. Concrete drawing subclasses compose this base.
use crate::include::core::{SkClipOp::SkClipOp, SkSurfaceProps::SkSurfaceProps};
use crate::src::core::{
    SkClipStack::{B2TIter, BoundsType, Element, SkClipStack},
    SkDevice::SkDevice,
    SkImageInfo::SkImageInfo,
    SkMatrix::SkMatrix,
    SkPath::SkPath,
    SkPathBuilder::SkPathBuilder,
    SkRRect::SkRRect,
    SkRect::{SkIRect, SkRect},
    SkRegion::{Op, SkRegion},
};
use crate::src::shaders::SkShader::SkShader;
use core::{
    marker::PhantomPinned,
    mem::{self, MaybeUninit},
    pin::Pin,
    ptr,
};
use std::sync::Arc;

// Source: src/core/SkClipStackDevice.h:57-57
const PREALLOC_COUNT: usize = 16;
// Source: src/core/SkClipStackDevice.h:61-62
const STORAGE_WORDS: usize = PREALLOC_COUNT * mem::size_of::<Element>() / mem::size_of::<isize>();
// The native inheritance/inline-storage lifetime is composed rather than erased.
// Drop the complete stack before its base and backing storage; initialize only after Pin.
pub struct SkClipStackDevice {
    clip_stack: Option<SkClipStack<'static>>,
    storage: [MaybeUninit<isize>; STORAGE_WORDS],
    _pin: PhantomPinned,
    // Source: src/core/SkClipStackDevice.h:26-26
    // The same embedded parent is destroyed after native derived members.
    base: SkDevice,
}
impl SkClipStackDevice {
    // Source: src/core/SkClipStackDevice.h:28-31
    // Rust stages the self-reference until its embedding allocation is pinned.
    pub fn new(info: &SkImageInfo, props: &SkSurfaceProps) -> Pin<Box<Self>> {
        let mut device = Box::pin(unsafe { Self::unbound(info, props) });
        unsafe {
            device.as_mut().initialize();
        }
        device
    }
    /// # Safety
    /// Before clipping operations, pin the complete embedding device at its final
    /// address and call initialize. Never move it afterwards.
    pub unsafe fn unbound(info: &SkImageInfo, props: &SkSurfaceProps) -> Self {
        const {
            assert!(mem::align_of::<Element>() <= mem::align_of::<isize>());
        }
        Self {
            clip_stack: None,
            base: SkDevice::new(info, props),
            storage: [MaybeUninit::uninit(); STORAGE_WORDS],
            _pin: PhantomPinned,
        }
    }
    /// # Safety
    /// Initialize exactly once, at the complete object's permanent pinned address.
    pub unsafe fn initialize(self: Pin<&mut Self>) {
        let owner = unsafe { self.get_unchecked_mut() as *mut Self };
        assert!(unsafe { (*owner).clip_stack.is_none() });
        let storage = unsafe { ptr::addr_of_mut!((*owner).storage).cast::<u8>() };
        let stack = unsafe {
            SkClipStack::with_storage(
                storage,
                mem::size_of::<[MaybeUninit<isize>; STORAGE_WORDS]>(),
            )
        };
        unsafe {
            ptr::addr_of_mut!((*owner).clip_stack).write(Some(stack));
        }
    }
    pub fn base(&self) -> &SkDevice {
        &self.base
    }
    /// # Safety
    /// Hold the exclusive complete-device lease; never move/replace the base/counter.
    pub unsafe fn base_mut(self: Pin<&mut Self>) -> &mut SkDevice {
        let owner = unsafe { self.get_unchecked_mut() as *mut Self };
        if unsafe { (*owner).clip_stack.is_some() } {
            unsafe {
                Self::stack_mut(owner);
            }
        }
        unsafe { &mut (*owner).base }
    }
    // Source: src/core/SkClipStackDevice.h:33-34
    pub fn cs(&self) -> &SkClipStack<'_> {
        self.clip_stack
            .as_ref()
            .expect("pinned clip device initialized")
    }
    /// # Safety
    /// Do not move, replace or leak the self-referencing stack. Its inline backing
    /// storage is borrowed from this pinned device for every operation.
    pub unsafe fn cs_mut(self: Pin<&mut Self>) -> &mut SkClipStack<'static> {
        let owner = unsafe { self.get_unchecked_mut() as *mut Self };
        unsafe { Self::stack_mut(owner) }
    }
    unsafe fn stack_mut<'a>(owner: *mut Self) -> &'a mut SkClipStack<'static> {
        let storage = unsafe { ptr::addr_of_mut!((*owner).storage).cast::<u8>() };
        let stack = unsafe {
            (*owner)
                .clip_stack
                .as_mut()
                .expect("pinned clip device initialized")
        };
        // Every exclusive parent borrow rederives the inline allocation provenance.
        unsafe {
            stack.rebind_initial_storage(
                storage,
                mem::size_of::<[MaybeUninit<isize>; STORAGE_WORDS]>(),
            );
        }
        stack
    }
    // Source: src/core/SkClipStackDevice.h:53-53
    // Source: src/core/SkClipStackDevice.cpp:25-31
    pub fn devClipBounds(&self) -> SkIRect {
        let r = self
            .cs()
            .bounds(&self.base.image_info().bounds())
            .roundOut();
        if !r.is_empty() {
            crate::SkASSERT!(@source b"src/core/SkClipStackDevice.cpp\0".as_ptr(), 28; @condition b"this->imageInfo().bounds().contains(r)\0".as_ptr(); self.base.image_info().bounds().contains_rect(&r));
        }
        r
    }
    // Source: src/core/SkClipStackDevice.h:36-36
    // Source: src/core/SkClipStackDevice.cpp:35-37
    pub fn pushClipStack(self: Pin<&mut Self>) {
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).save();
        }
    }
    // Source: src/core/SkClipStackDevice.h:37-37
    // Source: src/core/SkClipStackDevice.cpp:39-41
    pub fn popClipStack(self: Pin<&mut Self>) {
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).restore();
        }
    }
    // Source: src/core/SkClipStackDevice.h:39-39
    // Source: src/core/SkClipStackDevice.cpp:43-45
    pub fn clipRect(self: Pin<&mut Self>, rect: &SkRect, op: SkClipOp, aa: bool) {
        let matrix = self.base.local_to_device().clone();
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).clip_rect(rect, &matrix, op, aa);
        }
    }
    // Source: src/core/SkClipStackDevice.h:40-40
    // Source: src/core/SkClipStackDevice.cpp:47-49
    pub fn clipRRect(self: Pin<&mut Self>, rrect: &SkRRect, op: SkClipOp, aa: bool) {
        let matrix = self.base.local_to_device().clone();
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).clip_rrect(rrect, &matrix, op, aa);
        }
    }
    // Source: src/core/SkClipStackDevice.h:41-41
    // Source: src/core/SkClipStackDevice.cpp:51-53
    pub fn clipPath(self: Pin<&mut Self>, path: &SkPath, op: SkClipOp, aa: bool) {
        let matrix = self.base.local_to_device().clone();
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).clip_path(path, &matrix, op, aa);
        }
    }
    // Source: src/core/SkClipStackDevice.h:59-59
    // Source: src/core/SkClipStackDevice.cpp:55-57
    // Current canonical shader ownership is Arc; the actual retained object is forwarded.
    pub fn onClipShader(self: Pin<&mut Self>, shader: Arc<dyn SkShader>) {
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).clip_shader(shader);
        }
    }
    // Source: src/core/SkClipStackDevice.h:42-42
    // Source: src/core/SkClipStackDevice.cpp:59-66
    pub fn clipRegion(self: Pin<&mut Self>, region: &SkRegion, op: SkClipOp) {
        let origin = self.base.get_origin();
        let _tmp = SkRegion::new();
        let mut builder = SkPathBuilder::new();
        region.add_boundary_path(&mut builder);
        builder.transform(&SkMatrix::Translate(
            -(origin.fX as f32),
            -(origin.fY as f32),
        ));
        let path = builder.detach(None);
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).clip_path(&path, SkMatrix::I(), op, false);
        }
    }
    // Source: src/core/SkClipStackDevice.h:44-44
    // Source: src/core/SkClipStackDevice.cpp:68-71
    pub fn replaceClip(self: Pin<&mut Self>, rect: &SkIRect) {
        let device_rect = crate::src::core::SkMatrixPriv::SkMatrixPriv::MapRect(
            self.base.global_to_device(),
            &SkRect::Make(rect),
        );
        unsafe {
            Self::stack_mut(self.get_unchecked_mut()).replace_clip(&device_rect, false);
        }
    }
    // Source: src/core/SkClipStackDevice.h:46-46
    // Source: src/core/SkClipStackDevice.cpp:73-83
    pub fn isClipAntiAliased(&self) -> bool {
        let mut iter = B2TIter::new(self.cs());
        while let Some(element) = iter.next() {
            if element.is_aa() {
                return true;
            }
        }
        false
    }
    // Source: src/core/SkClipStackDevice.h:47-47
    // Source: src/core/SkClipStackDevice.cpp:85-87
    pub fn isClipWideOpen(&self) -> bool {
        self.cs()
            .quick_contains_rect(&SkRect::MakeIWH(self.base.width(), self.base.height()))
    }
    // Source: src/core/SkClipStackDevice.h:48-48
    // Source: src/core/SkClipStackDevice.cpp:89-91
    pub fn isClipEmpty(&self) -> bool {
        self.cs()
            .is_empty(&SkIRect::make_wh(self.base.width(), self.base.height()))
    }
    // Source: src/core/SkClipStackDevice.h:49-49
    // Source: src/core/SkClipStackDevice.cpp:93-105
    pub fn isClipRect(&self) -> bool {
        if self.isClipWideOpen() {
            return true;
        } else if self.isClipEmpty() {
            return false;
        }
        let mut bounds = SkRect::MakeEmpty();
        let mut bound_type = BoundsType::kNormal_BoundsType;
        let mut intersection_of_rects = false;
        self.cs().get_bounds(
            &mut bounds,
            &mut bound_type,
            Some(&mut intersection_of_rects),
        );
        intersection_of_rects && bound_type == BoundsType::kNormal_BoundsType
    }
    // Source: src/core/SkClipStackDevice.h:51-51
    // Source: src/core/SkClipStackDevice.cpp:107-134
    pub fn android_utils_clipAsRgn(&self, region: &mut SkRegion) {
        let mut bounds = SkRect::MakeEmpty();
        let mut bound_type = BoundsType::kNormal_BoundsType;
        let mut intersection_of_rects = false;
        self.cs().get_bounds(
            &mut bounds,
            &mut bound_type,
            Some(&mut intersection_of_rects),
        );
        if intersection_of_rects && bound_type == BoundsType::kNormal_BoundsType {
            region.set_rect(bounds.round());
        } else {
            let bounds_region =
                SkRegion::from_rect(SkIRect::make_wh(self.base.width(), self.base.height()));
            region.assign(&bounds_region);
            let mut iter = B2TIter::new(self.cs());
            while let Some(element) = iter.next() {
                let path = element.as_device_space_path();
                let mut tmp_region = SkRegion::new();
                tmp_region.set_path(&path, &bounds_region);
                if element.is_replace_op() {
                    region.set_rect(element.get_device_space_rect().round());
                } else {
                    region.op_region(
                        &tmp_region,
                        match element.get_op() {
                            SkClipOp::kDifference => Op::Difference,
                            SkClipOp::kIntersect => Op::Intersect,
                        },
                    );
                }
            }
        }
    }
}
impl Drop for SkClipStackDevice {
    fn drop(&mut self) {
        let owner = self as *mut Self;
        if self.clip_stack.is_some() {
            // Full parent provenance is rebound before native element destruction.
            unsafe {
                Self::stack_mut(owner);
            }
        }
    }
}
