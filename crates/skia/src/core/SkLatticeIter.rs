use crate::src::core::{
    SkCanvas::{Lattice, LatticeRectType},
    SkMatrix::SkMatrix,
    SkRect::{SkIRect, SkRect},
};

// Source: src/core/SkLatticeIter.h:23-24
// Source: src/core/SkLatticeIter.h:66-76
// Adapted: native TArray owners become contiguous Vec owners. RectType storage
// keeps its native u8 discriminant, including open enum values from serialized
// callers, without ever constructing an invalid Rust enum. All arrays are copied
// before the input lattice backing lease ends, exactly as in the native owner.
pub struct SkLatticeIter {
    // Adapted private declaration order preserves native reverse array-owner
    // destruction. Named constructor expressions remain in native source order.
    // Source: src/core/SkLatticeIter.h:66-77
    fNumRectsToDraw: i32,
    fNumRectsInLattice: i32,
    fCurrY: i32,
    fCurrX: i32,
    fColors: Vec<u32>,
    fRectTypes: Vec<u8>,
    fDstY: Vec<f32>,
    fDstX: Vec<f32>,
    fSrcY: Vec<i32>,
    fSrcX: Vec<i32>,
}
impl SkLatticeIter {
    // Source: src/core/SkLatticeIter.h:26-26
    pub unsafe fn Valid(width: i32, height: i32, lattice: &Lattice) -> bool {
        unsafe { Self::valid_lattice(width, height, lattice) }
    }
    // Source: src/core/SkLatticeIter.h:28-28
    pub unsafe fn new(lattice: &Lattice, dst: &SkRect) -> Self {
        unsafe { Self::new_lattice(lattice, dst) }
    }
    // Source: src/core/SkLatticeIter.h:30-30
    pub fn Valid_center(width: i32, height: i32, center: &SkIRect) -> bool {
        Self::valid_center(width, height, center)
    }
    // Source: src/core/SkLatticeIter.h:32-32
    pub fn from_center(width: i32, height: i32, center: &SkIRect, dst: &SkRect) -> Self {
        Self::new_center(width, height, center, dst)
    }
    // Source: src/core/SkLatticeIter.h:39-40
    pub fn next_irect(
        &mut self,
        src: &mut SkIRect,
        dst: &mut SkRect,
        is_fixed: Option<&mut bool>,
        color: Option<&mut u32>,
    ) -> bool {
        self.next_irect_impl(src, dst, is_fixed, color)
    }
    // Source: src/core/SkLatticeIter.h:43-51
    pub fn next(
        &mut self,
        src: &mut SkRect,
        dst: &mut SkRect,
        is_fixed: Option<&mut bool>,
        color: Option<&mut u32>,
    ) -> bool {
        let mut integer_src = SkIRect::make_empty();
        if self.next_irect(&mut integer_src, dst, is_fixed, color) {
            *src = SkRect::Make(&integer_src);
            return true;
        }
        false
    }
    // Source: src/core/SkLatticeIter.h:56-56
    pub fn mapDstScaleTranslate(&mut self, matrix: &SkMatrix) {
        self.map_dst_scale_translate(matrix)
    }
    // Source: src/core/SkLatticeIter.h:61-63
    pub fn numRectsToDraw(&self) -> i32 {
        self.fNumRectsToDraw
    }
}

// Source: src/core/SkLatticeIter.cpp:20-30
unsafe fn valid_divs(divs: *const i32, count: i32, start: i32, end: i32) -> bool {
    let mut previous = start - 1;
    for i in 0..count {
        let value = unsafe { divs.add(i as usize).read() };
        if previous >= value || value > end {
            return false;
        }
        previous = value;
    }
    true
}
impl SkLatticeIter {
    // Source: src/core/SkLatticeIter.cpp:32-50
    /// Safety: bounds is initialized/non-null; each positive div count has that
    /// many readable initialized integers, retained for this synchronous call.
    unsafe fn valid_lattice(width: i32, height: i32, lattice: &Lattice) -> bool {
        let total = SkIRect::make_wh(width, height);
        crate::SkASSERT!(@source b"src/core/SkLatticeIter.cpp\0".as_ptr(), 34; @condition b"lattice.fBounds\0".as_ptr(); !lattice.fBounds.is_null());
        let bounds = unsafe { lattice.fBounds.read() };
        if !total.contains_rect(&bounds) {
            return false;
        }
        let zero_x = lattice.fXCount <= 0
            || (lattice.fXCount == 1 && bounds.fLeft == unsafe { lattice.fXDivs.read() });
        let zero_y = lattice.fYCount <= 0
            || (lattice.fYCount == 1 && bounds.fTop == unsafe { lattice.fYDivs.read() });
        if zero_x && zero_y {
            return false;
        }
        unsafe {
            valid_divs(lattice.fXDivs, lattice.fXCount, bounds.fLeft, bounds.fRight)
                && valid_divs(lattice.fYDivs, lattice.fYCount, bounds.fTop, bounds.fBottom)
        }
    }
}
// Source: src/core/SkLatticeIter.cpp:55-80
unsafe fn count_scalable_pixels(
    divs: *const i32,
    count: i32,
    first: bool,
    start: i32,
    end: i32,
) -> i32 {
    if count == 0 {
        return if first { end - start } else { 0 };
    }
    let (mut i, mut pixels) = if first {
        (1, unsafe { divs.read() } - start)
    } else {
        (0, 0)
    };
    while i < count {
        let left = unsafe { divs.add(i as usize).read() };
        let right = if i + 1 < count {
            unsafe { divs.add((i + 1) as usize).read() }
        } else {
            end
        };
        pixels += right - left;
        i += 2;
    }
    pixels
}
// Source: src/core/SkLatticeIter.cpp:85-118
unsafe fn set_points(
    dst: &mut [f32],
    src: &mut [i32],
    divs: *const i32,
    count: i32,
    fixed: i32,
    scalable: i32,
    start: i32,
    end: i32,
    dst_start: f32,
    dst_end: f32,
    mut is_scalable: bool,
) {
    let length = dst_end - dst_start;
    let scale = if fixed as f32 <= length {
        (length - fixed as f32) / scalable as f32
    } else {
        length / fixed as f32
    };
    src[0] = start;
    dst[0] = dst_start;
    for i in 0..count as usize {
        src[i + 1] = unsafe { divs.add(i).read() };
        let delta = src[i + 1] - src[i];
        let dst_delta = if fixed as f32 <= length {
            if is_scalable {
                scale * delta as f32
            } else {
                delta as f32
            }
        } else {
            if is_scalable {
                0.0
            } else {
                scale * delta as f32
            }
        };
        dst[i + 1] = dst[i] + dst_delta;
        is_scalable = !is_scalable;
    }
    src[count as usize + 1] = end;
    dst[count as usize + 1] = dst_end;
}
impl SkLatticeIter {
    // Source: src/core/SkLatticeIter.cpp:120-213
    /// Safety: native lattice validity/count/array requirements hold. RectTypes
    /// covers the original grid; each fixed-color entry has a readable color.
    /// The constructor copies all required source arrays into this original owner.
    unsafe fn new_lattice(lattice: &Lattice, dst: &SkRect) -> Self {
        let mut x_divs = lattice.fXDivs;
        let original_x = lattice.fXCount;
        let mut y_divs = lattice.fYDivs;
        let original_y = lattice.fYCount;
        crate::SkASSERT!(@source b"src/core/SkLatticeIter.cpp\0".as_ptr(), 125; @condition b"lattice.fBounds\0".as_ptr(); !lattice.fBounds.is_null());
        let src = unsafe { lattice.fBounds.read() };
        let mut x_count = original_x;
        let mut y_count = original_y;
        let x_scalable = x_count > 0 && src.fLeft == unsafe { x_divs.read() };
        if x_scalable {
            x_divs = unsafe { x_divs.add(1) };
            x_count -= 1;
        }
        let y_scalable = y_count > 0 && src.fTop == unsafe { y_divs.read() };
        if y_scalable {
            y_divs = unsafe { y_divs.add(1) };
            y_count -= 1;
        }
        let x_sc =
            unsafe { count_scalable_pixels(x_divs, x_count, x_scalable, src.fLeft, src.fRight) };
        let x_fixed = src.width() - x_sc;
        let y_sc =
            unsafe { count_scalable_pixels(y_divs, y_count, y_scalable, src.fTop, src.fBottom) };
        let y_fixed = src.height() - y_sc;
        // Native reset creates uninitialized slots overwritten by set_points.
        // Initialized Vec scratch makes Rust references valid before those writes.
        let mut source_x = vec![0; (x_count + 2) as usize];
        let mut target_x = vec![0.0; (x_count + 2) as usize];
        unsafe {
            set_points(
                &mut target_x,
                &mut source_x,
                x_divs,
                x_count,
                x_fixed,
                x_sc,
                src.fLeft,
                src.fRight,
                dst.fLeft,
                dst.fRight,
                x_scalable,
            )
        };
        let mut source_y = vec![0; (y_count + 2) as usize];
        let mut target_y = vec![0.0; (y_count + 2) as usize];
        unsafe {
            set_points(
                &mut target_y,
                &mut source_y,
                y_divs,
                y_count,
                y_fixed,
                y_sc,
                src.fTop,
                src.fBottom,
                dst.fTop,
                dst.fBottom,
                y_scalable,
            )
        };
        let total = (x_count + 1) * (y_count + 1);
        let mut result = Self {
            fSrcX: source_x,
            fSrcY: source_y,
            fDstX: target_x,
            fDstY: target_y,
            fRectTypes: Vec::new(),
            fColors: Vec::new(),
            fCurrX: 0,
            fCurrY: 0,
            fNumRectsInLattice: total,
            fNumRectsToDraw: total,
        };
        if !lattice.fRectTypes.is_null() {
            result.fRectTypes.resize(total as usize, 0);
            result.fColors.resize(total as usize, 0);
            let mut flags = lattice.fRectTypes.cast::<u8>();
            let mut colors = lattice.fColors;
            let pad_row = y_count != original_y;
            let pad_column = x_count != original_x;
            if pad_row {
                flags = flags.wrapping_add((original_x + 1) as usize);
                colors = colors.wrapping_add((original_x + 1) as usize);
            }
            let mut i = 0;
            for _y in 0..y_count + 1 {
                for x in 0..original_x + 1 {
                    if x == 0 && pad_column {
                        flags = flags.wrapping_add(1);
                        colors = colors.wrapping_add(1);
                        continue;
                    }
                    let flag = unsafe { flags.read() };
                    result.fRectTypes[i] = flag;
                    result.fColors[i] = if flag == LatticeRectType::kFixedColor as u8 {
                        unsafe { colors.read() }
                    } else {
                        0
                    };
                    flags = flags.wrapping_add(1);
                    colors = colors.wrapping_add(1);
                    i += 1;
                }
            }
            for &flag in &result.fRectTypes {
                if flag == LatticeRectType::kTransparent as u8 {
                    result.fNumRectsToDraw -= 1;
                }
            }
        }
        result
    }
    // Source: src/core/SkLatticeIter.cpp:215-217
    fn valid_center(width: i32, height: i32, center: &SkIRect) -> bool {
        !center.is_empty() && SkIRect::make_wh(width, height).contains_rect(center)
    }
    // Source: src/core/SkLatticeIter.cpp:219-261
    fn new_center(width: i32, height: i32, center: &SkIRect, dst: &SkRect) -> Self {
        crate::SkASSERT!(@source b"src/core/SkLatticeIter.cpp\0".as_ptr(), 220; @condition b"SkIRect::MakeWH(w, h).contains(c)\0".as_ptr(); SkIRect::make_wh(width,height).contains_rect(center));
        // Preserve the native SkIntToScalar→int assignment's rounding, rather
        // than silently replacing it with an exact integer copy.
        let source_x = vec![
            0,
            center.fLeft as f32 as i32,
            center.fRight as f32 as i32,
            width as f32 as i32,
        ];
        let source_y = vec![
            0,
            center.fTop as f32 as i32,
            center.fBottom as f32 as i32,
            height as f32 as i32,
        ];
        let mut target_x = vec![
            dst.fLeft,
            dst.fLeft + center.fLeft as f32,
            dst.fRight - (width - center.fRight) as f32,
            dst.fRight,
        ];
        let mut target_y = vec![
            dst.fTop,
            dst.fTop + center.fTop as f32,
            dst.fBottom - (height - center.fBottom) as f32,
            dst.fBottom,
        ];
        if target_x[1] > target_x[2] {
            target_x[1] = target_x[0]
                + (target_x[3] - target_x[0]) * center.fLeft as f32
                    / (width - center.width()) as f32;
            target_x[2] = target_x[1];
        }
        if target_y[1] > target_y[2] {
            target_y[1] = target_y[0]
                + (target_y[3] - target_y[0]) * center.fTop as f32
                    / (height - center.height()) as f32;
            target_y[2] = target_y[1];
        }
        Self {
            fSrcX: source_x,
            fSrcY: source_y,
            fDstX: target_x,
            fDstY: target_y,
            fRectTypes: Vec::new(),
            fColors: Vec::new(),
            fCurrX: 0,
            fCurrY: 0,
            fNumRectsInLattice: 9,
            fNumRectsToDraw: 9,
        }
    }
    // Source: src/core/SkLatticeIter.cpp:263-292
    fn next_irect_impl(
        &mut self,
        src: &mut SkIRect,
        dst: &mut SkRect,
        is_fixed: Option<&mut bool>,
        color: Option<&mut u32>,
    ) -> bool {
        let current = self.fCurrX + self.fCurrY * (self.fSrcX.len() as i32 - 1);
        if current == self.fNumRectsInLattice {
            return false;
        }
        let x = self.fCurrX as usize;
        let y = self.fCurrY as usize;
        {
            crate::SkASSERT!(@source b"src/core/SkLatticeIter.cpp\0".as_ptr(), 270; @condition b"x >= 0 && x < fSrcX.size() - 1\0".as_ptr(); x<self.fSrcX.len()-1);
            crate::SkASSERT!(@source b"src/core/SkLatticeIter.cpp\0".as_ptr(), 271; @condition b"y >= 0 && y < fSrcY.size() - 1\0".as_ptr(); y<self.fSrcY.len()-1);
        };
        self.fCurrX += 1;
        if self.fSrcX.len() as i32 - 1 == self.fCurrX {
            self.fCurrX = 0;
            self.fCurrY += 1;
        }
        if !self.fRectTypes.is_empty()
            && self.fRectTypes[current as usize] == LatticeRectType::kTransparent as u8
        {
            return self.next_irect(src, dst, is_fixed, color);
        }
        src.set_ltrb(
            self.fSrcX[x],
            self.fSrcY[y],
            self.fSrcX[x + 1],
            self.fSrcY[y + 1],
        );
        dst.setLTRB(
            self.fDstX[x],
            self.fDstY[y],
            self.fDstX[x + 1],
            self.fDstY[y + 1],
        );
        if let (Some(is_fixed), Some(color)) = (is_fixed, color) {
            *is_fixed = !self.fRectTypes.is_empty()
                && self.fRectTypes[current as usize] == LatticeRectType::kFixedColor as u8;
            if *is_fixed {
                *color = self.fColors[current as usize];
            }
        }
        true
    }
    // Source: src/core/SkLatticeIter.cpp:294-307
    fn map_dst_scale_translate(&mut self, matrix: &SkMatrix) {
        crate::SkASSERT!(@source b"src/core/SkLatticeIter.cpp\0".as_ptr(), 295; @condition b"matrix.isScaleTranslate()\0".as_ptr(); matrix.isScaleTranslate());
        let tx = matrix.getTranslateX();
        let sx = matrix.getScaleX();
        for x in &mut self.fDstX {
            *x = *x * sx + tx;
        }
        let ty = matrix.getTranslateY();
        let sy = matrix.getScaleY();
        for y in &mut self.fDstY {
            *y = *y * sy + ty;
        }
    }
}
