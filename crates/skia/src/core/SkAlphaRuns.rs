// Copyright 2006 The Android Open Source Project. BSD-style license.
// Rust adaptation: borrowed slices retain the caller-owned sparse run buffers;
// integer indices replace interior pointers without changing run splitting.
// Source: src/core/SkAlphaRuns.h:24-33
pub struct SkAlphaRuns<'a> {
    pub fRuns: &'a mut [i16],
    pub fAlpha: &'a mut [u8],
    // Source: src/core/SkAlphaRuns.h:197
    #[cfg(sk_debug)]
    fWidth: core::mem::MaybeUninit<i32>,
    // Adapted initialization witness for safe Rust entry points before reset.
    #[cfg(sk_debug)]
    width_initialized: bool,
}
impl<'a> SkAlphaRuns<'a> {
    pub fn new(runs: &'a mut [i16], alpha: &'a mut [u8]) -> Self {
        Self {
            fRuns: runs,
            fAlpha: alpha,
            #[cfg(sk_debug)]
            fWidth: core::mem::MaybeUninit::uninit(),
            #[cfg(sk_debug)]
            width_initialized: false,
        }
    }
    // Rust view of an already initialized native sparse buffer; no reset or
    // storage allocation occurs, matching the original fRuns rebinding.
    pub(crate) fn from_initialized(runs: &'a mut [i16], alpha: &'a mut [u8], width: usize) -> Self {
        assert!(runs.len() > width && alpha.len() > width);
        Self {
            fRuns: runs,
            fAlpha: alpha,
            // Adapted view of the caller's already initialized sparse buffer.
            #[cfg(sk_debug)]
            fWidth: core::mem::MaybeUninit::new(width as i32),
            #[cfg(sk_debug)]
            width_initialized: true,
        }
    }
    #[cfg(sk_debug)]
    fn native_width(&self) -> i32 {
        // Rust safety precondition, not a native release field/default value.
        assert!(
            self.width_initialized,
            "SkAlphaRuns width read before reset"
        );
        unsafe { self.fWidth.assume_init_read() }
    }
    pub fn CatchOverflow(alpha: u32) -> u8 {
        crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 31; @condition b"alpha >= 0 && alpha <= 256\0".as_ptr(); alpha <= 256);
        (alpha - (alpha >> 8)) as u8
    }
    // Source: src/core/SkAlphaRuns.h:37-40
    pub fn empty(&self) -> bool {
        crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 38; @condition b"fRuns[0] > 0\0".as_ptr(); self.fRuns[0] > 0);
        self.fAlpha[0] == 0 && self.fRuns[self.fRuns[0] as usize] == 0
    }
    // Source: src/core/SkAlphaRuns.h:58-113
    pub fn add(
        &mut self,
        mut x: usize,
        start_alpha: u32,
        mut middle_count: usize,
        stop_alpha: u32,
        max_value: u32,
        offset_x: usize,
    ) -> usize {
        crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 61; @condition b"x >= 0 && x + (startAlpha != 0) + middleCount + (stopAlpha != 0) <= fWidth\0".as_ptr(); x + usize::from(start_alpha != 0) + middle_count + usize::from(stop_alpha != 0)
                <= self.native_width() as usize);
        crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 63; @condition b"fRuns[offsetX] >= 0\0".as_ptr(); self.fRuns[offset_x] >= 0);
        let mut base = offset_x;
        let mut last_alpha = base;
        x -= offset_x;
        if start_alpha != 0 {
            Self::Break(&mut self.fRuns[base..], &mut self.fAlpha[base..], x, 1);
            let tmp = self.fAlpha[base + x] as u32 + start_alpha;
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 78; @condition b"tmp <= 256\0".as_ptr(); tmp <= 256);
            self.fAlpha[base + x] = (tmp - (tmp >> 8)) as u8;
            base += x + 1;
            x = 0;
            #[cfg(sk_debug)]
            self.validate();
        }
        if middle_count != 0 {
            Self::Break(
                &mut self.fRuns[base..],
                &mut self.fAlpha[base..],
                x,
                middle_count,
            );
            base += x;
            x = 0;
            loop {
                self.fAlpha[base] = Self::CatchOverflow(self.fAlpha[base] as u32 + max_value);
                let n = self.fRuns[base] as usize;
                crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 95; @condition b"n <= middleCount\0".as_ptr(); n <= middle_count);
                base += n;
                middle_count -= n;
                if middle_count == 0 {
                    break;
                }
            }
            #[cfg(sk_debug)]
            self.validate();
            last_alpha = base;
        }
        if stop_alpha != 0 {
            Self::Break(&mut self.fRuns[base..], &mut self.fAlpha[base..], x, 1);
            base += x;
            self.fAlpha[base] = (self.fAlpha[base] as u32 + stop_alpha) as u8;
            #[cfg(sk_debug)]
            self.validate();
            last_alpha = base;
        }
        last_alpha
    }
    // Source: src/core/SkAlphaRuns.h:126-171
    pub fn Break(runs: &mut [i16], alpha: &mut [u8], mut x: usize, count: usize) {
        crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 127; @condition b"count > 0 && x >= 0\0".as_ptr(); count > 0);
        let next = x;
        let mut base = 0;
        while x > 0 {
            let n = runs[base] as usize;
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 137; @condition b"n > 0\0".as_ptr(); runs[base] > 0);
            if x < n {
                alpha[base + x] = alpha[base];
                runs[base] = x as i16;
                runs[base + x] = (n - x) as i16;
                break;
            }
            base += n;
            x -= n;
        }
        base = next;
        x = count;
        loop {
            let n = runs[base] as usize;
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 156; @condition b"n > 0\0".as_ptr(); runs[base] > 0);
            if x < n {
                alpha[base + x] = alpha[base];
                runs[base] = x as i16;
                runs[base + x] = (n - x) as i16;
                break;
            }
            x -= n;
            if x == 0 {
                break;
            }
            base += n;
        }
    }
    // Source: src/core/SkAlphaRuns.h:179-194
    pub fn BreakAt(runs: &mut [i16], alpha: &mut [u8], mut x: usize) {
        let mut base = 0;
        while x > 0 {
            let n = runs[base] as usize;
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.h\0".as_ptr(), 182; @condition b"n > 0\0".as_ptr(); runs[base] > 0);
            if x < n {
                alpha[base + x] = alpha[base];
                runs[base] = x as i16;
                runs[base + x] = (n - x) as i16;
                break;
            }
            base += n;
            x -= n;
        }
    }
    // Source: src/core/SkAlphaRuns.cpp:11-23
    pub fn reset(&mut self, width: usize) {
        debug_assert!(width > 0 && width <= i16::MAX as usize);
        #[cfg(sk_debug)]
        self.fRuns[..width].fill(-42);
        self.fRuns[0] = width as i16;
        self.fRuns[width] = 0;
        self.fAlpha[0] = 0;
        // Source: src/core/SkAlphaRuns.cpp:21
        #[cfg(sk_debug)]
        {
            self.fWidth.write(width as i32);
            self.width_initialized = true;
        }
        #[cfg(sk_debug)]
        self.validate();
    }
    // Source: src/core/SkAlphaRuns.cpp:26-37
    #[cfg(sk_debug)]
    pub fn assertValid(&self, y: i32, max_step: i32) {
        let max = (y + 1) * max_step - i32::from(y == max_step - 1);
        let mut base = 0;
        while self.fRuns[base] != 0 {
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 33; @condition b"*alpha <= max\0".as_ptr(); self.fAlpha[base] as i32 <= max);
            base += self.fRuns[base] as usize;
        }
    }
    // Source: src/core/SkAlphaRuns.cpp:39-56
    #[cfg(sk_debug)]
    pub fn dump(&self) {
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 43; b"Runs\0".as_ptr());
        }
        let mut base = 0;
        while self.fRuns[base] != 0 {
            let n = self.fRuns[base] as usize;
            unsafe {
                crate::SkDebugf!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 47; b" %02x\0".as_ptr(),self.fAlpha[base] as core::ffi::c_int);
            }
            if n > 1 {
                unsafe {
                    crate::SkDebugf!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 49; b",%d\0".as_ptr(),n as core::ffi::c_int);
                }
            }
            base += n;
        }
        unsafe {
            crate::SkDebugf!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 54; b"\n\0".as_ptr());
        }
    }
    // Source: src/core/SkAlphaRuns.cpp:58-70
    #[cfg(sk_debug)]
    pub fn validate(&self) {
        #[cfg(sk_debug)]
        {
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 58; @condition b"fWidth > 0\0".as_ptr(); self.native_width() > 0);
            let mut count = 0;
            while self.fRuns[count] != 0 {
                crate::SkASSERT!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 64; @condition b"*runs > 0\0".as_ptr(); self.fRuns[count] > 0);
                count += self.fRuns[count] as usize;
                crate::SkASSERT!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 66; @condition b"count <= fWidth\0".as_ptr(); count <= self.native_width() as usize);
            }
            crate::SkASSERT!(@source b"src/core/SkAlphaRuns.cpp\0".as_ptr(), 69; @condition b"count == fWidth\0".as_ptr(); (count) == (self.native_width() as usize));
        }
    }
}
