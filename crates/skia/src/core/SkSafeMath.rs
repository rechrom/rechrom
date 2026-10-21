use crate::include::private::SkTFitsIn::{sk_strip_enum, SkCastFrom, SkTFitsIn};
// Adapted source: src/core/SkSafeMath.h:23-24: flat Rust storage declaration separated from native class inline scope
// Adapted source: src/core/SkSafeMath.h:159-160: Rust field type and Default initializer are separate items; native in-class field initializer cannot bind to the storage item alone
pub struct SkSafeMath {
    fOK: bool,
}
// Source: src/core/SkSafeMath.h:25-25
impl Default for SkSafeMath {
    fn default() -> Self {
        Self { fOK: true }
    }
}
impl SkSafeMath {
    pub const fn new() -> Self {
        Self { fOK: true }
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:27-28
    pub fn ok(&self) -> bool {
        self.fOK
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:30-32
    pub fn mul(&mut self, x: usize, y: usize) -> usize {
        if std::mem::size_of::<usize>() == 8 {
            self.mul64(x as u64, y as u64) as usize
        } else {
            self.mul32(x as u32, y as u32) as usize
        }
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:34-38
    pub fn add(&mut self, x: usize, y: usize) -> usize {
        let result = x.wrapping_add(y);
        self.fOK &= result >= x;
        result
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:40-43
    pub fn sub(&mut self, x: usize, y: usize) -> usize {
        self.fOK &= x >= y;
        x.wrapping_sub(y)
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:45-51
    pub fn addInt(&mut self, a: i32, b: i32) -> i32 {
        let result = a as i64 + b as i64;
        if !SkTFitsIn::<i32, i64>(result) {
            self.fOK = false
        }
        result as i32
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:53-59
    pub fn subInt(&mut self, a: i32, b: i32) -> i32 {
        let result = a as i64 - b as i64;
        if !SkTFitsIn::<i32, i64>(result) {
            self.fOK = false
        }
        result as i32
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:61-67
    pub fn mulInt(&mut self, x: i32, y: i32) -> i32 {
        let result = x as i64 * y as i64;
        if !SkTFitsIn::<i32, i64>(result) {
            self.fOK = false
        }
        result as i32
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:69-75
    pub fn divInt(&mut self, a: i32, b: i32) -> i32 {
        if b == 0 || (a == i32::MIN && b == -1) {
            self.fOK = false;
            return a;
        }
        a / b
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:77-83
    pub fn modInt(&mut self, a: i32, b: i32) -> i32 {
        if b == 0 || (a == i32::MIN && b == -1) {
            self.fOK = false;
            return a;
        }
        a % b
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:85-88
    pub fn alignUp(&mut self, x: usize, alignment: usize) -> usize {
        crate::SkASSERT!(@source b"src/core/SkSafeMath.h\0".as_ptr(),86; @condition b"alignment && !(alignment & (alignment - 1))\0".as_ptr();alignment!=0 && alignment & alignment.wrapping_sub(1)==0);
        self.add(x, alignment.wrapping_sub(1)) & !alignment.wrapping_sub(1)
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:90-97
    pub fn alignUpNonPow2(&mut self, x: usize, alignment: usize) -> usize {
        if alignment == 0 {
            self.fOK = false;
            return 0;
        }
        let aligned = self.add(x, alignment - 1);
        (aligned / alignment) * alignment
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:99-106
    pub fn lcm(&mut self, a: usize, b: usize) -> usize {
        if a == 0 || b == 0 {
            self.fOK = false;
            return 0;
        }
        let (mut x, mut y) = (a, b);
        while y != 0 {
            let r = x % y;
            x = y;
            y = r
        }
        let gcd = x;
        self.mul(a / gcd, b)
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:108-113
    pub fn castTo<D, S>(&mut self, value: S) -> D
    where
        D: sk_strip_enum + SkCastFrom<S>,
        S: sk_strip_enum
            + PartialOrd
            + SkCastFrom<D>
            + SkCastFrom<i32>
            + SkCastFrom<<D as sk_strip_enum>::Type>,
    {
        if !SkTFitsIn::<D, S>(value) {
            self.fOK = false;
        }
        D::sk_cast_from(value)
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:118-121
    pub fn Align4(x: usize) -> usize {
        let mut safe = Self::new();
        safe.alignUp(x, 4)
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:124-128
    // Source: src/core/SkSafeMath.h:130-131
    fn mul32(&mut self, x: u32, y: u32) -> u32 {
        let bx = x as u64;
        let by = y as u64;
        let result = bx * by;
        self.fOK &= result >> 32 == 0;
        result as u32
    }
}
impl SkSafeMath {
    // Source: src/core/SkSafeMath.h:133-139
    // Source: src/core/SkSafeMath.h:141-148
    // Source: src/core/SkSafeMath.h:156-158
    fn mul64(&mut self, x: u64, y: u64) -> u64 {
        if x <= u64::MAX >> 32 && y <= u64::MAX >> 32 {
            x * y
        } else {
            let hi = |x: u64| x >> 32;
            let lo = |x: u64| x & 0xFFFFFFFF;
            let lx_ly = lo(x) * lo(y);
            let hx_ly = hi(x) * lo(y);
            let lx_hy = lo(x) * hi(y);
            let hx_hy = hi(x) * hi(y);
            let mut result = 0u64;
            result = self.add(lx_ly as usize, (hx_ly << 32) as usize) as u64;
            result = self.add(result as usize, (lx_hy << 32) as usize) as u64;
            self.fOK &= hx_hy.wrapping_add(hx_ly >> 32).wrapping_add(lx_hy >> 32) == 0;
            // Source: src/core/SkSafeMath.h:151-153
            // Clang's x86-64 debug-only unsigned __int128 arithmetic is a Rust
            // u128 language substitution, retaining the native diagnostic branch.
            #[cfg(all(sk_debug, sk_native_clang, target_arch = "x86_64"))]
            {
                let double_check = x as u128 * y as u128;
                crate::SkASSERT!(@source b"src/core/SkSafeMath.h\0".as_ptr(),152; @condition b"result == (double_check & 0xFFFFFFFFFFFFFFFF)\0".as_ptr();result as u128==double_check&0xFFFFFFFFFFFFFFFF);
                crate::SkASSERT!(@source b"src/core/SkSafeMath.h\0".as_ptr(),153; @condition b"!fOK || (double_check >> 64 == 0)\0".as_ptr();!self.fOK||(double_check>>64)==0);
            }
            result
        }
    }
}
// Source: src/core/SkSafeMath.h:116-116
// Source: src/core/SkSafeMath.cpp:10-14
impl SkSafeMath {
    pub fn Add(x: usize, y: usize) -> usize {
        let mut tmp = Self::new();
        let sum = tmp.add(x, y);
        if tmp.ok() {
            sum
        } else {
            usize::MAX
        }
    }
}
// Source: src/core/SkSafeMath.h:117-117
// Source: src/core/SkSafeMath.cpp:16-20
impl SkSafeMath {
    pub fn Mul(x: usize, y: usize) -> usize {
        let mut tmp = Self::new();
        let prod = tmp.mul(x, y);
        if tmp.ok() {
            prod
        } else {
            usize::MAX
        }
    }
}
