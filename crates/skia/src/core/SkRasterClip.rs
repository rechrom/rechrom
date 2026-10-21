use crate::include::core::SkClipOp::SkClipOp;
use crate::src::core::SkAAClip::SkAAClip;
use crate::src::core::SkRect::SkIRect;
use crate::src::core::SkRegion::{Op, SkRegion};
use crate::src::shaders::SkShader::SkShader;
use std::sync::Arc;

// Geometric path entries use the actual Scan conversion. The native shader
// lifetime is retained through empty/rect transitions; current Arc is adapted.
// Source: src/core/SkRasterClip.h:98-105
pub struct SkRasterClip {
    // Rust declarations drop in source order; preserve native reverse member
    // destruction: Shader, AAClip, Region. Layout is not claimed ABI identical.
    shader: Option<Arc<dyn SkShader>>,
    aa: SkAAClip,
    bw: SkRegion,
    is_bw: bool,
    is_empty: bool,
    is_rect: bool,
}
// Source: src/core/SkRasterClip.h:42-62
impl SkRasterClip {
    pub fn is_bw(&self) -> bool {
        self.is_bw
    }
    pub fn is_aa(&self) -> bool {
        !self.is_bw
    }
    pub fn bw_rgn(&self) -> &SkRegion {
        crate::SkASSERT!(@source b"src/core/SkRasterClip.h\0".as_ptr(), 44; @condition b"fIsBW\0".as_ptr(); self.is_bw);
        &self.bw
    }
    pub fn aa_rgn(&self) -> &SkAAClip {
        crate::SkASSERT!(@source b"src/core/SkRasterClip.h\0".as_ptr(), 45; @condition b"!fIsBW\0".as_ptr(); !self.is_bw);
        &self.aa
    }
    pub fn is_empty(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkRasterClip.h\0".as_ptr(), 48; @condition b"this->computeIsEmpty() == fIsEmpty\0".as_ptr(); (self.compute_is_empty()) == (self.is_empty));
        self.is_empty
    }
    pub fn is_rect(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkRasterClip.h\0".as_ptr(), 53; @condition b"this->computeIsRect() == fIsRect\0".as_ptr(); (self.compute_is_rect()) == (self.is_rect));
        self.is_rect
    }
    pub fn is_complex(&self) -> bool {
        if self.is_bw {
            self.bw.is_complex()
        } else {
            !self.aa.is_empty()
        }
    }
    pub fn get_bounds(&self) -> &SkIRect {
        if self.is_bw {
            self.bw.get_bounds()
        } else {
            self.aa.get_bounds()
        }
    }
}
// Source: src/core/SkRasterClip.h:76-78
impl SkRasterClip {
    pub fn quick_contains(&self, rect: &SkIRect) -> bool {
        if self.is_bw {
            self.bw.quick_contains(rect)
        } else {
            self.aa.quick_contains_rect(rect)
        }
    }
    // Source: src/core/SkRasterClip.h:85-87
    pub fn quick_reject(&self, rect: &SkIRect) -> bool {
        !SkIRect::intersects(self.get_bounds(), rect)
    }
    // Source: src/core/SkRasterClip.h:95-95
    pub fn clip_shader(&self) -> Option<Arc<dyn SkShader>> {
        self.shader.clone()
    }
}
// Source: src/core/SkRasterClip.h:107-127
impl SkRasterClip {
    fn compute_is_empty(&self) -> bool {
        if self.is_bw {
            self.bw.is_empty()
        } else {
            self.aa.is_empty()
        }
    }
    fn compute_is_rect(&self) -> bool {
        if self.is_bw {
            self.bw.is_rect()
        } else {
            self.aa.is_rect()
        }
    }
    fn update_cache(&mut self, detect_aa_rect: bool) -> bool {
        self.is_empty = self.compute_is_empty();
        if detect_aa_rect && !self.is_empty && !self.is_bw && self.aa.is_rect() {
            self.bw.set_rect(*self.aa.get_bounds());
            self.aa.set_empty();
            self.is_bw = true;
        }
        self.is_rect = self.compute_is_rect();
        !self.is_empty
    }
}
// Source: src/core/SkRasterClip.h:134-144
// The original guard is noncopyable and retains the original clip identity.
// Raw construction permits the native lexical validation scope around mutation
// without holding a Rust shared projection across the mutable operation.
pub struct SkAutoRasterClipValidate<'clip> {
    fRC: core::ptr::NonNull<SkRasterClip>,
    lease: core::marker::PhantomData<&'clip SkRasterClip>,
}
impl<'clip> SkAutoRasterClipValidate<'clip> {
    // Source: src/core/SkRasterClip.h:136-138
    pub fn new(rc: &'clip SkRasterClip) -> Self {
        // This retained immutable borrow is sufficient for readonly scopes.
        unsafe { Self::from_raw(core::ptr::NonNull::from(rc)) }
    }
    /// The exact clip allocation remains live and unmoved until this guard
    /// drops. Mutation is allowed only between validation calls; no reference
    /// created by this guard survives either call, including unwind cleanup.
    pub unsafe fn from_raw(rc: core::ptr::NonNull<SkRasterClip>) -> Self {
        unsafe {
            rc.as_ref().validate();
        }
        Self {
            fRC: rc,
            lease: core::marker::PhantomData,
        }
    }
}
impl Drop for SkAutoRasterClipValidate<'_> {
    // Source: src/core/SkRasterClip.h:139-141
    fn drop(&mut self) {
        unsafe {
            self.fRC.as_ref().validate();
        }
    }
}

// Source: src/core/SkRasterClip.cpp:20-35
impl Clone for SkRasterClip {
    fn clone(&self) -> Self {
        let mut result = Self {
            bw: SkRegion::new(),
            aa: SkAAClip::new(),
            is_bw: self.is_bw,
            is_empty: self.is_empty,
            is_rect: self.is_rect,
            shader: self.shader.clone(),
        };
        let _source_validate = SkAutoRasterClipValidate::new(self);
        if result.is_bw {
            result.bw.set_region(&self.bw);
        } else {
            result.aa.assign(&self.aa);
        }
        result.validate();
        result
    }
    fn clone_from(&mut self, source: &Self) {
        self.assign(source);
    }
}
// Source: src/core/SkRasterClip.cpp:37-52
impl SkRasterClip {
    pub fn assign(&mut self, other: &Self) {
        let _source_validate = SkAutoRasterClipValidate::new(other);
        self.is_bw = other.is_bw;
        if self.is_bw {
            self.bw.set_region(&other.bw);
        } else {
            self.aa.assign(&other.aa);
        }
        self.is_empty = other.is_empty();
        self.is_rect = other.is_rect();
        let old_shader = core::mem::replace(&mut self.shader, other.shader.clone());
        drop(old_shader);
        self.validate();
    }
}
// Source: src/core/SkRasterClip.cpp:54-59
impl SkRasterClip {
    pub fn from_region(region: &SkRegion) -> Self {
        let bw = region.clone();
        let is_empty = bw.is_empty();
        let result = Self {
            bw,
            aa: SkAAClip::new(),
            is_bw: true,
            is_empty,
            is_rect: !is_empty,
            shader: None,
        };
        result.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:61-66
impl SkRasterClip {
    pub fn from_rect(bounds: SkIRect) -> Self {
        let bw = SkRegion::from_rect(bounds);
        let is_empty = bw.is_empty();
        let result = Self {
            bw,
            aa: SkAAClip::new(),
            is_bw: true,
            is_empty,
            is_rect: !is_empty,
            shader: None,
        };
        result.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:68-73
impl SkRasterClip {
    pub fn new() -> Self {
        let result = Self {
            bw: SkRegion::new(),
            aa: SkAAClip::new(),
            is_bw: true,
            is_empty: true,
            is_rect: false,
            shader: None,
        };
        result.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:88-90
impl Drop for SkRasterClip {
    fn drop(&mut self) {
        #[cfg(sk_debug)]
        self.validate();
    }
}
// Source: src/core/SkRasterClip.cpp:92-101
impl SkRasterClip {
    pub fn set_empty(&mut self) -> bool {
        self.validate();
        self.is_bw = true;
        self.bw.set_empty();
        self.aa.set_empty();
        self.is_empty = true;
        self.is_rect = false;
        self.validate();
        false
    }
}
// Source: src/core/SkRasterClip.cpp:103-111
impl SkRasterClip {
    pub fn set_rect(&mut self, rect: SkIRect) -> bool {
        self.validate();
        self.is_bw = true;
        self.aa.set_empty();
        self.is_rect = self.bw.set_rect(rect);
        self.is_empty = !self.is_rect;
        self.validate();
        self.is_rect
    }
}
// Source: src/core/SkRasterClip.cpp:115-124
impl SkRasterClip {
    pub fn op_irect(&mut self, rect: SkIRect, op: SkClipOp) -> bool {
        self.validate();
        if self.is_bw {
            self.bw.op_rect(rect, region_op(op));
        } else {
            self.aa.op_rect(rect, op);
        }
        let result = self.update_cache(true);
        self.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:126-137
impl SkRasterClip {
    pub fn op_region(&mut self, region: &SkRegion, op: SkClipOp) -> bool {
        self.validate();
        if self.is_bw {
            self.bw.op_region(region, region_op(op));
        } else {
            let mut tmp = SkAAClip::new();
            tmp.set_region(region);
            self.aa.op(&tmp, op);
        }
        let result = self.update_cache(true);
        self.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:144-150
fn nearly_integral(mut x: f32) -> bool {
    let domain = 1.0 / 4.0;
    let half_domain = domain / 2.0;
    x += half_domain;
    x - crate::include::core::SkScalar::SkScalarFloorToScalar(x) < domain
}
// Source: src/core/SkRasterClip.cpp:221-243
impl SkRasterClip {
    fn op_clip(&mut self, clip: &Self, op: SkClipOp) -> bool {
        self.validate();
        clip.validate();
        if self.is_bw && clip.is_bw {
            self.bw.op_region(&clip.bw, region_op(op));
        } else {
            let mut tmp = SkAAClip::new();
            if self.is_bw {
                self.convert_to_aa();
            }
            let other = if clip.is_bw {
                tmp.set_region(clip.bw_rgn());
                &tmp
            } else {
                clip.aa_rgn()
            };
            self.aa.op(other, op);
        }
        let result = self.update_cache(true);
        self.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:245-270
impl SkRasterClip {
    pub fn translate_to(&self, dx: i32, dy: i32, dst: Option<&mut Self>) {
        let Some(dst) = dst else { return };
        self.validate();
        if self.is_empty() {
            dst.set_empty();
            return;
        }
        if dx | dy == 0 {
            dst.assign(self);
            return;
        }
        dst.is_bw = self.is_bw;
        if self.is_bw {
            self.bw.translate_to(dx, dy, Some(&mut dst.bw));
            dst.aa.set_empty();
        } else {
            self.aa.translate_to(dx, dy, Some(&mut dst.aa));
            dst.bw.set_empty();
        }
        dst.update_cache(true);
        self.validate();
    }
}
// Source: src/core/SkRasterClip.cpp:272-282
impl SkRasterClip {
    fn convert_to_aa(&mut self) {
        self.validate();
        crate::SkASSERT!(@source b"src/core/SkRasterClip.cpp\0".as_ptr(), 275; @condition b"fIsBW\0".as_ptr(); self.is_bw);
        self.aa.set_region(&self.bw);
        self.is_bw = false;
        self.update_cache(false);
        self.validate();
    }
}
// Source: src/core/SkRasterClip.cpp:284-297
impl SkRasterClip {
    pub fn validate(&self) {
        #[cfg(sk_debug)]
        {
            if self.is_bw {
                debug_assert!(self.aa.is_empty());
            }
            crate::src::core::SkRegionPriv::SkRegionPriv::Validate(&self.bw);
            self.aa.validate();
            crate::SkASSERT!(@source b"src/core/SkRasterClip.cpp\0".as_ptr(), 294; @condition b"this->computeIsEmpty() == fIsEmpty\0".as_ptr(); (self.compute_is_empty()) == (self.is_empty));
            crate::SkASSERT!(@source b"src/core/SkRasterClip.cpp\0".as_ptr(), 295; @condition b"this->computeIsRect() == fIsRect\0".as_ptr(); (self.compute_is_rect()) == (self.is_rect));
        }
    }
}
// Enum cast in native SkRegion shares the first two SkClipOp discriminants.
fn region_op(op: SkClipOp) -> Op {
    match op {
        SkClipOp::kDifference => Op::Difference,
        SkClipOp::kIntersect => Op::Intersect,
    }
}

// Source: src/core/SkRasterClip.cpp:210-219
impl SkRasterClip {
    pub fn op_shader(&mut self, shader: Option<Arc<dyn SkShader>>) -> bool {
        self.validate();
        if self.shader.is_none() {
            self.shader = shader;
        } else {
            let replacement = crate::src::shaders::SkBlendShader::Blend(
                crate::src::core::SkBlendMode::SkBlendMode::kSrcIn,
                shader,
                self.shader.clone(),
            );
            let old_shader = core::mem::replace(&mut self.shader, replacement);
            drop(old_shader);
        }
        let result = !self.is_empty();
        self.validate();
        result
    }
}

// Source: src/core/SkRasterClip.h:185-189
// Rust selectors replace the two self-referential result pointers. The owned
// region and AA blitter remain present in both modes, preserving scratch reuse.
pub struct SkAAClipBlitterWrapper<'a> {
    bw_region: SkRegion,
    aa_blitter: core::mem::ManuallyDrop<crate::src::core::SkAAClip::SkAAClipBlitter<'a>>,
    borrowed_region: Option<&'a SkRegion>,
    borrowed_blitter: Option<core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>>,
    use_aa: bool,
}
// Source: src/core/SkRasterClip.cpp:301-305
impl<'a> SkAAClipBlitterWrapper<'a> {
    pub fn new() -> Self {
        Self {
            bw_region: SkRegion::new(),
            aa_blitter: core::mem::ManuallyDrop::new(
                crate::src::core::SkAAClip::SkAAClipBlitter::new(),
            ),
            borrowed_region: None,
            borrowed_blitter: None,
            use_aa: false,
        }
    }
}
// Source: src/core/SkRasterClip.cpp:306-309
impl<'a> SkAAClipBlitterWrapper<'a> {
    pub fn from_clip(
        clip: &'a SkRasterClip,
        blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
    ) -> Self {
        let mut wrapper = Self::new();
        wrapper.init_fields(clip, blitter);
        wrapper
    }
}
// Source: src/core/SkRasterClip.cpp:311-320
impl<'a> SkAAClipBlitterWrapper<'a> {
    pub fn from_aa_clip(
        clip: &'a SkAAClip,
        blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
    ) -> Self {
        let mut wrapper = Self::new();
        wrapper.bw_region.set_rect(*clip.get_bounds());
        wrapper.aa_blitter.init_fields(blitter, clip);
        wrapper.use_aa = true;
        wrapper
    }
}
// Source: src/core/SkRasterClip.cpp:322-335
impl<'a> SkAAClipBlitterWrapper<'a> {
    pub fn init(
        self: core::pin::Pin<&mut Self>,
        clip: &'a SkRasterClip,
        blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
    ) {
        unsafe { self.get_unchecked_mut() }.init_fields(clip, blitter);
    }
    // Constructor-only configuration writes no self-address or arena allocation.
    fn init_fields(
        &mut self,
        clip: &'a SkRasterClip,
        blitter: core::pin::Pin<&'a mut dyn crate::src::core::SkBlitter::SkBlitter>,
    ) {
        if clip.is_bw() {
            self.borrowed_region = Some(clip.bw_rgn());
            self.borrowed_blitter = Some(blitter);
            self.use_aa = false;
        } else {
            let aa = clip.aa_rgn();
            self.bw_region.set_rect(*aa.get_bounds());
            self.aa_blitter.init_fields(blitter, aa);
            self.use_aa = true;
        }
    }
}
// Source: src/core/SkRasterClip.h:171-182
impl<'a> SkAAClipBlitterWrapper<'a> {
    pub fn get_bounds(&self) -> &SkIRect {
        self.get_rgn().get_bounds()
    }
    pub fn get_rgn(&self) -> &SkRegion {
        if self.use_aa {
            &self.bw_region
        } else {
            self.borrowed_region.expect("initialized clip region")
        }
    }
    pub fn get_blitter(
        self: core::pin::Pin<&mut Self>,
    ) -> core::pin::Pin<&mut (dyn crate::src::core::SkBlitter::SkBlitter + 'a)> {
        let this = unsafe { self.get_unchecked_mut() };
        if this.use_aa {
            unsafe { core::pin::Pin::new_unchecked(&mut *this.aa_blitter) }
        } else {
            this.borrowed_blitter
                .as_mut()
                .expect("initialized blitter")
                .as_mut()
        }
    }
    // Disjoint actual region and pinned blitter projections share the wrapper lease.
    pub fn parts(
        self: core::pin::Pin<&mut Self>,
    ) -> (
        &SkRegion,
        core::pin::Pin<&mut (dyn crate::src::core::SkBlitter::SkBlitter + 'a)>,
    ) {
        let this = unsafe { self.get_unchecked_mut() };
        if this.use_aa {
            (&this.bw_region, unsafe {
                core::pin::Pin::new_unchecked(&mut *this.aa_blitter)
            })
        } else {
            (
                this.borrowed_region.expect("initialized clip region"),
                this.borrowed_blitter
                    .as_mut()
                    .expect("initialized blitter")
                    .as_mut(),
            )
        }
    }
}

// C++ destroys the AA blitter member before its owned fallback BW region.
impl Drop for SkAAClipBlitterWrapper<'_> {
    fn drop(&mut self) {
        // Same native reverse-member order, in place even after pin publication.
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.aa_blitter);
        }
    }
}

// Source: src/core/SkRasterClip.cpp:75-86
impl SkRasterClip {
    pub fn from_path(
        path: &crate::src::core::SkPath::SkPath,
        bounds: SkIRect,
        do_aa: bool,
    ) -> Self {
        let mut result = Self {
            bw: SkRegion::new(),
            aa: SkAAClip::new(),
            is_bw: true,
            is_empty: true,
            is_rect: false,
            shader: None,
        };
        if do_aa {
            result.is_bw = false;
            result.aa.set_path(path, bounds, true);
        } else {
            result.is_bw = true;
            result.bw.set_path(path, &SkRegion::from_rect(bounds));
        }
        result.is_empty = result.compute_is_empty();
        result.is_rect = result.compute_is_rect();
        result.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:152-179
impl SkRasterClip {
    pub fn op_rect(
        &mut self,
        rect: &crate::src::core::SkRect::SkRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        mut do_aa: bool,
    ) -> bool {
        use crate::include::core::SkPathTypes::{SkPathDirection, SkPathFillType};
        use crate::src::core::SkPath::SkPath;
        use crate::src::core::SkRect::SkRect;
        self.validate();
        let is_scale_translate = matrix.isScaleTranslate();
        if !is_scale_translate {
            return self.op_path(
                &SkPath::Rect(rect, SkPathFillType::kDefault, SkPathDirection::kCW, 0),
                matrix,
                op,
                do_aa,
            );
        }
        let mut device_rect = SkRect::MakeEmpty();
        matrix.mapRect(&mut device_rect, rect);
        if self.is_bw && do_aa {
            if nearly_integral(device_rect.fLeft)
                && nearly_integral(device_rect.fTop)
                && nearly_integral(device_rect.fRight)
                && nearly_integral(device_rect.fBottom)
            {
                do_aa = false;
            }
        }
        if self.is_bw && !do_aa {
            self.bw.op_rect(device_rect.round(), region_op(op));
        } else {
            if self.is_bw {
                self.convert_to_aa();
            }
            self.aa.op_float_rect(&device_rect, op, do_aa);
        }
        let result = self.update_cache(true);
        self.validate();
        result
    }
}
// Source: src/core/SkRasterClip.cpp:181-183
impl SkRasterClip {
    pub fn op_rrect(
        &mut self,
        rrect: &crate::src::core::SkRRect::SkRRect,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) -> bool {
        self.op_path(
            &crate::src::core::SkPath::SkPath::RRect(
                rrect,
                crate::include::core::SkPathTypes::SkPathDirection::kCW,
                6,
            ),
            matrix,
            op,
            do_aa,
        )
    }
}
// Source: src/core/SkRasterClip.cpp:185-208
impl SkRasterClip {
    pub fn op_path(
        &mut self,
        path: &crate::src::core::SkPath::SkPath,
        matrix: &crate::src::core::SkMatrix::SkMatrix,
        op: SkClipOp,
        do_aa: bool,
    ) -> bool {
        self.validate();
        let device_path = path.makeTransform(matrix);
        if self.is_rect() && op == SkClipOp::kIntersect {
            if do_aa && self.is_bw {
                self.convert_to_aa();
            }
            let bounds = *self.get_bounds();
            if self.is_bw {
                self.bw.set_path(&device_path, &SkRegion::from_rect(bounds));
            } else {
                self.aa.set_path(&device_path, bounds, do_aa);
            }
            let result = self.update_cache(true);
            self.validate();
            result
        } else {
            let other = Self::from_path(&device_path, *self.get_bounds(), do_aa);
            self.op_clip(&other, op)
        }
    }
}
