// Copyright 2006 The Android Open Source Project. BSD-style license.
use crate::include::private::SkFixed::*;
use crate::src::core::SkFDot6::*;
use crate::src::core::SkPoint::SkPoint;
// Source: src/core/SkAnalyticEdge.h:20-53
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Type {
    kLine,
    kQuad,
    kCubic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum Winding {
    kCW = 1,
    kCCW = -1,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkAnalyticEdge {
    pub fNext: *mut Self,
    pub fPrev: *mut Self,
    pub fX: i32,
    pub fDX: i32,
    pub fUpperX: i32,
    pub fY: i32,
    pub fUpperY: i32,
    pub fLowerY: i32,
    pub fDY: i32,
    pub fEdgeType: Type,
    pub fCurveCount: i8,
    pub fCurveShift: u8,
    pub fWinding: Winding,
}
// Source: src/core/SkAnalyticEdge.h:89-97
#[cfg(sk_debug)]
impl SkAnalyticEdge {
    pub fn dump(&self) {
        unsafe extern "C" {
            fn snprintf(
                dst: *mut core::ffi::c_char,
                n: usize,
                fmt: *const core::ffi::c_char,
                ...
            ) -> core::ffi::c_int;
        }
        let mut bytes = [0u8; 512];
        let n = unsafe {
            snprintf(
                bytes.as_mut_ptr().cast(),
                bytes.len(),
                b"edge: upperY:%d lowerY:%d y:%g x:%g dx:%g w:%d\n\0"
                    .as_ptr()
                    .cast(),
                self.fUpperY,
                self.fLowerY,
                (self.fY as f32 / 65536.0) as f64,
                (self.fX as f32 / 65536.0) as f64,
                (self.fDX as f32 / 65536.0) as f64,
                self.fWinding as i8 as i32,
            )
        };
        if n > 0 {
            use std::io::Write;
            let _ = std::io::stderr().write_all(&bytes[..(n as usize).min(bytes.len() - 1)]);
        }
    }
}
impl SkAnalyticEdge {
    pub fn new() -> Self {
        Self {
            fNext: core::ptr::null_mut(),
            fPrev: core::ptr::null_mut(),
            fX: 0,
            fDX: 0,
            fUpperX: 0,
            fY: 0,
            fUpperY: 0,
            fLowerY: 0,
            fDY: 0,
            fEdgeType: Type::kLine,
            fCurveCount: 0,
            fCurveShift: 0,
            fWinding: Winding::kCW,
        }
    }
    // Source: src/core/SkAnalyticEdge.h:54-60
    pub const kDefaultAccuracy: i32 = 2;
    pub fn SnapY(y: i32) -> i32 {
        (y as u32)
            .wrapping_add((SK_Fixed1 >> (Self::kDefaultAccuracy + 1)) as u32)
            .wrapping_shr((16 - Self::kDefaultAccuracy) as u32)
            .wrapping_shl((16 - Self::kDefaultAccuracy) as u32) as i32
    }
    // Source: src/core/SkAnalyticEdge.h:63-74
    pub fn goY(&mut self, y: i32) {
        if y == self.fY.wrapping_add(SK_Fixed1) {
            self.fX = self.fX.wrapping_add(self.fDX);
            self.fY = y;
        } else if y != self.fY {
            self.fX = self
                .fUpperX
                .wrapping_add(SkFixedMul(self.fDX, y.wrapping_sub(self.fUpperY)));
            self.fY = y;
        }
    }
    // Source: src/core/SkAnalyticEdge.h:75-80
    pub fn goYShift(&mut self, y: i32, yShift: i32) {
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 76; @condition b"yShift >= 0 && yShift <= kDefaultAccuracy\0".as_ptr(); yShift >= 0 && yShift <= Self::kDefaultAccuracy);
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 77; @condition b"fDX == 0 || y - fY == SK_Fixed1 >> yShift\0".as_ptr(); self.fDX == 0 || y.wrapping_sub(self.fY) == SK_Fixed1 >> yShift);
        self.fY = y;
        self.fX = self.fX.wrapping_add(self.fDX >> yShift);
    }
}
impl SkAnalyticEdge {
    // Source: src/core/SkAnalyticEdge.h:99-106
    /// Safety: the neighboring intrusive edges must remain live while checking.
    pub unsafe fn validate(&self) {
        #[cfg(sk_debug)]
        {
            crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 100; @condition b"fPrev && fNext\0".as_ptr(); !self.fPrev.is_null() && !self.fNext.is_null());
            crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 101; @condition b"fPrev->fNext == this\0".as_ptr(); core::ptr::eq((*self.fPrev).fNext, self));
            crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 102; @condition b"fNext->fPrev == this\0".as_ptr(); core::ptr::eq((*self.fNext).fPrev, self));
            crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 104; @condition b"fUpperY < fLowerY\0".as_ptr(); self.fUpperY < self.fLowerY);
            crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 105; @condition b"fWinding == Winding::kCW || fWinding == Winding::kCCW\0".as_ptr(); matches!(self.fWinding,Winding::kCW|Winding::kCCW));
        }
    }
}
// Source: src/core/SkAnalyticEdge.cpp:20-132
const kInverseTableSize: i32 = 1024;
fn quick_inverse(x: i32) -> i32 {
    const TABLE: [i32; 1025] = [
        -4096, -4100, -4104, -4108, -4112, -4116, -4120, -4124, -4128, -4132, -4136, -4140, -4144,
        -4148, -4152, -4156, -4161, -4165, -4169, -4173, -4177, -4181, -4185, -4190, -4194, -4198,
        -4202, -4206, -4211, -4215, -4219, -4223, -4228, -4232, -4236, -4240, -4245, -4249, -4253,
        -4258, -4262, -4266, -4271, -4275, -4279, -4284, -4288, -4293, -4297, -4301, -4306, -4310,
        -4315, -4319, -4324, -4328, -4332, -4337, -4341, -4346, -4350, -4355, -4359, -4364, -4369,
        -4373, -4378, -4382, -4387, -4391, -4396, -4401, -4405, -4410, -4415, -4419, -4424, -4429,
        -4433, -4438, -4443, -4447, -4452, -4457, -4462, -4466, -4471, -4476, -4481, -4485, -4490,
        -4495, -4500, -4505, -4510, -4514, -4519, -4524, -4529, -4534, -4539, -4544, -4549, -4554,
        -4559, -4563, -4568, -4573, -4578, -4583, -4588, -4593, -4599, -4604, -4609, -4614, -4619,
        -4624, -4629, -4634, -4639, -4644, -4650, -4655, -4660, -4665, -4670, -4675, -4681, -4686,
        -4691, -4696, -4702, -4707, -4712, -4718, -4723, -4728, -4733, -4739, -4744, -4750, -4755,
        -4760, -4766, -4771, -4777, -4782, -4788, -4793, -4798, -4804, -4809, -4815, -4821, -4826,
        -4832, -4837, -4843, -4848, -4854, -4860, -4865, -4871, -4877, -4882, -4888, -4894, -4899,
        -4905, -4911, -4917, -4922, -4928, -4934, -4940, -4946, -4951, -4957, -4963, -4969, -4975,
        -4981, -4987, -4993, -4999, -5005, -5011, -5017, -5023, -5029, -5035, -5041, -5047, -5053,
        -5059, -5065, -5071, -5077, -5084, -5090, -5096, -5102, -5108, -5115, -5121, -5127, -5133,
        -5140, -5146, -5152, -5159, -5165, -5171, -5178, -5184, -5190, -5197, -5203, -5210, -5216,
        -5223, -5229, -5236, -5242, -5249, -5256, -5262, -5269, -5275, -5282, -5289, -5295, -5302,
        -5309, -5315, -5322, -5329, -5336, -5343, -5349, -5356, -5363, -5370, -5377, -5384, -5391,
        -5398, -5405, -5412, -5418, -5426, -5433, -5440, -5447, -5454, -5461, -5468, -5475, -5482,
        -5489, -5497, -5504, -5511, -5518, -5526, -5533, -5540, -5548, -5555, -5562, -5570, -5577,
        -5584, -5592, -5599, -5607, -5614, -5622, -5629, -5637, -5645, -5652, -5660, -5667, -5675,
        -5683, -5691, -5698, -5706, -5714, -5722, -5729, -5737, -5745, -5753, -5761, -5769, -5777,
        -5785, -5793, -5801, -5809, -5817, -5825, -5833, -5841, -5849, -5857, -5866, -5874, -5882,
        -5890, -5899, -5907, -5915, -5924, -5932, -5940, -5949, -5957, -5966, -5974, -5983, -5991,
        -6000, -6009, -6017, -6026, -6034, -6043, -6052, -6061, -6069, -6078, -6087, -6096, -6105,
        -6114, -6123, -6132, -6141, -6150, -6159, -6168, -6177, -6186, -6195, -6204, -6213, -6223,
        -6232, -6241, -6250, -6260, -6269, -6278, -6288, -6297, -6307, -6316, -6326, -6335, -6345,
        -6355, -6364, -6374, -6384, -6393, -6403, -6413, -6423, -6432, -6442, -6452, -6462, -6472,
        -6482, -6492, -6502, -6512, -6523, -6533, -6543, -6553, -6563, -6574, -6584, -6594, -6605,
        -6615, -6626, -6636, -6647, -6657, -6668, -6678, -6689, -6700, -6710, -6721, -6732, -6743,
        -6754, -6765, -6775, -6786, -6797, -6808, -6820, -6831, -6842, -6853, -6864, -6875, -6887,
        -6898, -6909, -6921, -6932, -6944, -6955, -6967, -6978, -6990, -7002, -7013, -7025, -7037,
        -7049, -7061, -7073, -7084, -7096, -7108, -7121, -7133, -7145, -7157, -7169, -7182, -7194,
        -7206, -7219, -7231, -7244, -7256, -7269, -7281, -7294, -7307, -7319, -7332, -7345, -7358,
        -7371, -7384, -7397, -7410, -7423, -7436, -7449, -7463, -7476, -7489, -7503, -7516, -7530,
        -7543, -7557, -7570, -7584, -7598, -7612, -7626, -7639, -7653, -7667, -7681, -7695, -7710,
        -7724, -7738, -7752, -7767, -7781, -7796, -7810, -7825, -7839, -7854, -7869, -7884, -7898,
        -7913, -7928, -7943, -7958, -7973, -7989, -8004, -8019, -8035, -8050, -8065, -8081, -8097,
        -8112, -8128, -8144, -8160, -8176, -8192, -8208, -8224, -8240, -8256, -8272, -8289, -8305,
        -8322, -8338, -8355, -8371, -8388, -8405, -8422, -8439, -8456, -8473, -8490, -8507, -8525,
        -8542, -8559, -8577, -8594, -8612, -8630, -8648, -8665, -8683, -8701, -8719, -8738, -8756,
        -8774, -8793, -8811, -8830, -8848, -8867, -8886, -8905, -8924, -8943, -8962, -8981, -9000,
        -9020, -9039, -9058, -9078, -9098, -9118, -9137, -9157, -9177, -9198, -9218, -9238, -9258,
        -9279, -9300, -9320, -9341, -9362, -9383, -9404, -9425, -9446, -9467, -9489, -9510, -9532,
        -9554, -9576, -9597, -9619, -9642, -9664, -9686, -9709, -9731, -9754, -9776, -9799, -9822,
        -9845, -9868, -9892, -9915, -9939, -9962, -9986, -10010, -10034, -10058, -10082, -10106,
        -10131, -10155, -10180, -10205, -10230, -10255, -10280, -10305, -10330, -10356, -10381,
        -10407, -10433, -10459, -10485, -10512, -10538, -10564, -10591, -10618, -10645, -10672,
        -10699, -10727, -10754, -10782, -10810, -10837, -10866, -10894, -10922, -10951, -10979,
        -11008, -11037, -11066, -11096, -11125, -11155, -11184, -11214, -11244, -11275, -11305,
        -11335, -11366, -11397, -11428, -11459, -11491, -11522, -11554, -11586, -11618, -11650,
        -11683, -11715, -11748, -11781, -11814, -11848, -11881, -11915, -11949, -11983, -12018,
        -12052, -12087, -12122, -12157, -12192, -12228, -12264, -12300, -12336, -12372, -12409,
        -12446, -12483, -12520, -12557, -12595, -12633, -12671, -12710, -12748, -12787, -12826,
        -12865, -12905, -12945, -12985, -13025, -13066, -13107, -13148, -13189, -13231, -13273,
        -13315, -13357, -13400, -13443, -13486, -13530, -13573, -13617, -13662, -13706, -13751,
        -13797, -13842, -13888, -13934, -13981, -14027, -14074, -14122, -14169, -14217, -14266,
        -14315, -14364, -14413, -14463, -14513, -14563, -14614, -14665, -14716, -14768, -14820,
        -14873, -14926, -14979, -15033, -15087, -15141, -15196, -15252, -15307, -15363, -15420,
        -15477, -15534, -15592, -15650, -15709, -15768, -15827, -15887, -15947, -16008, -16070,
        -16131, -16194, -16256, -16320, -16384, -16448, -16513, -16578, -16644, -16710, -16777,
        -16844, -16912, -16980, -17050, -17119, -17189, -17260, -17331, -17403, -17476, -17549,
        -17623, -17697, -17772, -17848, -17924, -18001, -18078, -18157, -18236, -18315, -18396,
        -18477, -18558, -18641, -18724, -18808, -18893, -18978, -19065, -19152, -19239, -19328,
        -19418, -19508, -19599, -19691, -19784, -19878, -19972, -20068, -20164, -20262, -20360,
        -20460, -20560, -20661, -20763, -20867, -20971, -21076, -21183, -21290, -21399, -21509,
        -21620, -21732, -21845, -21959, -22075, -22192, -22310, -22429, -22550, -22671, -22795,
        -22919, -23045, -23172, -23301, -23431, -23563, -23696, -23831, -23967, -24105, -24244,
        -24385, -24528, -24672, -24818, -24966, -25115, -25266, -25420, -25575, -25731, -25890,
        -26051, -26214, -26379, -26546, -26715, -26886, -27060, -27235, -27413, -27594, -27776,
        -27962, -28149, -28339, -28532, -28728, -28926, -29127, -29330, -29537, -29746, -29959,
        -30174, -30393, -30615, -30840, -31068, -31300, -31536, -31775, -32017, -32263, -32513,
        -32768, -33026, -33288, -33554, -33825, -34100, -34379, -34663, -34952, -35246, -35544,
        -35848, -36157, -36472, -36792, -37117, -37449, -37786, -38130, -38479, -38836, -39199,
        -39568, -39945, -40329, -40721, -41120, -41527, -41943, -42366, -42799, -43240, -43690,
        -44150, -44620, -45100, -45590, -46091, -46603, -47127, -47662, -48210, -48770, -49344,
        -49932, -50533, -51150, -51781, -52428, -53092, -53773, -54471, -55188, -55924, -56679,
        -57456, -58254, -59074, -59918, -60787, -61680, -62601, -63550, -64527, -65536, -66576,
        -67650, -68759, -69905, -71089, -72315, -73584, -74898, -76260, -77672, -79137, -80659,
        -82241, -83886, -85598, -87381, -89240, -91180, -93206, -95325, -97541, -99864, -102300,
        -104857, -107546, -110376, -113359, -116508, -119837, -123361, -127100, -131072, -135300,
        -139810, -144631, -149796, -155344, -161319, -167772, -174762, -182361, -190650, -199728,
        -209715, -220752, -233016, -246723, -262144, -279620, -299593, -322638, -349525, -381300,
        -419430, -466033, -524288, -599186, -699050, -838860, -1048576, -1398101, -2097152,
        -4194304, 0,
    ];
    const LAST: usize = TABLE.len() - 1;
    crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 125; @condition b"SkAbs32(x) <= static_cast<int32_t>(kLastEntry)\0".as_ptr(); x.wrapping_abs() <= LAST as i32);
    if x > 0 {
        -TABLE[LAST - x as usize]
    } else {
        TABLE[(LAST as i32 + x) as usize]
    }
}
// Source: src/core/SkAnalyticEdge.cpp:134-154
fn quick_div(a: i32, b: i32) -> i32 {
    const MIN_BITS: i32 = 3;
    const MAX_BITS: i32 = 31;
    const MAX_ABS_A: i32 = 1 << (MAX_BITS - (22 - MIN_BITS));
    let abs_a = a.wrapping_abs();
    let abs_b = b.wrapping_abs();
    if abs_b >= 1 << MIN_BITS && abs_b < kInverseTableSize && abs_a < MAX_ABS_A {
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 144; @condition b"(int64_t)a * quick_inverse(b) <= SK_MaxS32 && (int64_t)a * quick_inverse(b) >= SK_MinS32\0".as_ptr(); (a as i64) * quick_inverse(b) as i64 <= i32::MAX as i64
                && (a as i64) * quick_inverse(b) as i64 >= i32::MIN as i64);
        let ourAnswer = a.wrapping_mul(quick_inverse(b)) >> 6;
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 147; @condition b"(SkFDot6Div(a,b) == 0 && ourAnswer == 0) || SkFixedDiv(SkAbs32(SkFDot6Div(a,b) - ourAnswer), SkAbs32(SkFDot6Div(a,b))) <= 1 << 10\0".as_ptr(); (SkFDot6Div(a, b) == 0 && ourAnswer == 0)
                || SkFixedDiv(
                    SkFDot6Div(a, b).wrapping_sub(ourAnswer).wrapping_abs(),
                    SkFDot6Div(a, b).wrapping_abs()
                ) <= 1 << 10);
        return ourAnswer;
    }
    SkFDot6Div(a, b)
}
impl SkAnalyticEdge {
    // Source: src/core/SkAnalyticEdge.cpp:156-206
    pub fn setLine<const EVEN: bool>(&mut self, p0: &SkPoint, p1: &SkPoint) -> bool {
        let accuracy = Self::kDefaultAccuracy;
        let quantize = |v: f32| {
            if EVEN {
                SkScalarRoundToFDot6(v, accuracy)
            } else {
                SkScalarToFDot6(v * (1 << accuracy) as f32)
            }
        };
        let mut x0 = SkFDot6ToFixed(quantize(p0.fX)) >> accuracy;
        let mut y0 = Self::SnapY(SkFDot6ToFixed(quantize(p0.fY)) >> accuracy);
        let mut x1 = SkFDot6ToFixed(quantize(p1.fX)) >> accuracy;
        let mut y1 = Self::SnapY(SkFDot6ToFixed(quantize(p1.fY)) >> accuracy);
        let mut winding = Winding::kCW;
        if y0 > y1 {
            core::mem::swap(&mut x0, &mut x1);
            core::mem::swap(&mut y0, &mut y1);
            winding = Winding::kCCW;
        }
        let dy = SkFixedToFDot6(y1.wrapping_sub(y0));
        if dy == 0 {
            return false;
        }
        let dx = SkFixedToFDot6(x1.wrapping_sub(x0));
        let slope = quick_div(dx, dy);
        let absSlope = slope.wrapping_abs();
        self.fX = x0;
        self.fDX = slope;
        self.fUpperX = x0;
        self.fY = y0;
        self.fUpperY = y0;
        self.fLowerY = y1;
        self.fDY = if dx == 0 || slope == 0 {
            i32::MAX
        } else if absSlope < kInverseTableSize {
            quick_inverse(absSlope)
        } else {
            quick_div(dy, dx).wrapping_abs()
        };
        self.fEdgeType = Type::kLine;
        self.fCurveCount = 0;
        self.fWinding = winding;
        self.fCurveShift = 0;
        true
    }
}

// Source: src/core/SkAnalyticEdge.cpp:208-210
fn swap_winding(w: Winding) -> Winding {
    match w {
        Winding::kCW => Winding::kCCW,
        Winding::kCCW => Winding::kCW,
    }
}
impl SkAnalyticEdge {
    // Source: src/core/SkAnalyticEdge.cpp:215-257
    pub fn updateLine(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        mut x1: i32,
        mut y1: i32,
        slope: i32,
    ) -> bool {
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 220; @condition b"fCurveCount != 0\0".as_ptr(); self.fCurveCount != 0);
        if y0 > y1 {
            core::mem::swap(&mut x0, &mut x1);
            core::mem::swap(&mut y0, &mut y1);
            self.fWinding = swap_winding(self.fWinding);
        }
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 231; @condition b"y0 <= y1\0".as_ptr(); y0 <= y1);
        let dx = SkFixedToFDot6(x1.wrapping_sub(x0));
        let dy = SkFixedToFDot6(y1.wrapping_sub(y0));
        if dy == 0 {
            return false;
        }
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 241; @condition b"slope < SK_MaxS32\0".as_ptr(); slope < i32::MAX);
        let absSlope = SkFixedToFDot6(slope).wrapping_abs();
        self.fX = x0;
        self.fDX = slope;
        self.fUpperX = x0;
        self.fY = y0;
        self.fUpperY = y0;
        self.fLowerY = y1;
        self.fDY = if dx == 0 || slope == 0 {
            i32::MAX
        } else if absSlope < kInverseTableSize {
            quick_inverse(absSlope)
        } else {
            quick_div(dy, dx).wrapping_abs()
        };
        true
    }
}

impl SkAnalyticEdge {
    // Source: src/core/SkAnalyticEdge.cpp:259-267
    /// Safety: curve count must match a live repr(C) concrete derived edge
    /// allocation, with this base stored as its first field.
    pub unsafe fn update(&mut self, last_y: i32) -> bool {
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 260; @condition b"last_y >= fLowerY\0".as_ptr(); last_y >= self.fLowerY);
        if self.fCurveCount < 0 {
            (*(self as *mut Self as *mut SkAnalyticCubicEdge)).updateCubic()
        } else if self.fCurveCount > 0 {
            (*(self as *mut Self as *mut SkAnalyticQuadraticEdge)).updateQuadratic()
        } else {
            false
        }
    }
}
// Source: src/core/SkAnalyticEdge.cpp:274-303
const MAX_COEFF_SHIFT: i32 = 6;
fn cheap_distance(mut dx: i32, mut dy: i32) -> i32 {
    dx = dx.wrapping_abs();
    dy = dy.wrapping_abs();
    if dx > dy {
        dx = dx.wrapping_add(dy >> 1);
    } else {
        dx = dy.wrapping_add(dx >> 1);
    }
    dx
}
fn diff_to_shift(dx: i32, dy: i32, shiftAA: i32) -> i32 {
    let mut dist = cheap_distance(dx, dy);
    dist = dist.wrapping_add(1 << (2 + shiftAA)) >> (3 + shiftAA);
    (32 - (dist as u32).leading_zeros() as i32) >> 1
}
// Source: src/core/SkAnalyticEdge.cpp:317-321
fn SkFDot6ToFixedDiv2(value: i32) -> i32 {
    value.wrapping_shl(9)
}
// Source: src/core/SkAnalyticEdge.h:110-130
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkAnalyticQuadraticEdge {
    pub edge: SkAnalyticEdge,
    pub fQx: i32,
    pub fQy: i32,
    pub fQDx: i32,
    pub fQDy: i32,
    pub fQDDx: i32,
    pub fQDDy: i32,
    pub fQLastX: i32,
    pub fQLastY: i32,
    pub fSnappedX: i32,
    pub fSnappedY: i32,
}
impl core::ops::Deref for SkAnalyticQuadraticEdge {
    type Target = SkAnalyticEdge;
    fn deref(&self) -> &SkAnalyticEdge {
        &self.edge
    }
}
impl core::ops::DerefMut for SkAnalyticQuadraticEdge {
    fn deref_mut(&mut self) -> &mut SkAnalyticEdge {
        &mut self.edge
    }
}
impl SkAnalyticQuadraticEdge {
    pub fn new() -> Self {
        Self {
            edge: SkAnalyticEdge::new(),
            fQx: 0,
            fQy: 0,
            fQDx: 0,
            fQDy: 0,
            fQDDx: 0,
            fQDDy: 0,
            fQLastX: 0,
            fQLastY: 0,
            fSnappedX: 0,
            fSnappedY: 0,
        }
    }
    pub fn keepContinuous(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 125; @condition b"SkAbs32(fX - SkFixedMul(fY - fSnappedY, fDX) - fSnappedX) < SK_Fixed1\0".as_ptr(); self.fX
                .wrapping_sub(SkFixedMul(self.fY.wrapping_sub(self.fSnappedY), self.fDX))
                .wrapping_sub(self.fSnappedX)
                .wrapping_abs()
                < SK_Fixed1);
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 126; @condition b"SkAbs32(fY - fSnappedY) < SK_Fixed1\0".as_ptr(); self.fY.wrapping_sub(self.fSnappedY).wrapping_abs() < SK_Fixed1);
        self.fSnappedX = self.fX;
        self.fSnappedY = self.fY;
    }
    // Source: src/core/SkAnalyticEdge.cpp:323-423
    pub fn setQuadraticWithoutUpdate<const EVEN: bool>(
        &mut self,
        pts: &[SkPoint; 3],
        mut shift: i32,
    ) -> bool {
        let q = |v: f32| {
            if EVEN {
                SkScalarRoundToFDot6(v, shift)
            } else {
                (v * (1u32 << (shift + 6)) as f32) as i32
            }
        };
        let mut x0 = q(pts[0].fX);
        let mut y0 = q(pts[0].fY);
        let x1 = q(pts[1].fX);
        let y1 = q(pts[1].fY);
        let mut x2 = q(pts[2].fX);
        let mut y2 = q(pts[2].fY);
        let mut winding = Winding::kCW;
        if y0 > y2 {
            core::mem::swap(&mut x0, &mut x2);
            core::mem::swap(&mut y0, &mut y2);
            winding = Winding::kCCW;
        }
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 353; @condition b"y0 <= y1 && y1 <= y2\0".as_ptr(); y0 <= y1 && y1 <= y2);
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y2);
        if top == bot {
            return false;
        }
        let dx = x1.wrapping_shl(1).wrapping_sub(x0).wrapping_sub(x2) >> 2;
        let dy = y1.wrapping_shl(1).wrapping_sub(y0).wrapping_sub(y2) >> 2;
        shift = diff_to_shift(dx, dy, shift);
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 371; @condition b"shift >= 0\0".as_ptr(); shift >= 0);
        if shift == 0 {
            shift = 1;
        } else if shift > MAX_COEFF_SHIFT {
            shift = MAX_COEFF_SHIFT;
        }
        self.fWinding = winding;
        self.fEdgeType = Type::kQuad;
        self.fCurveCount = (1 << shift) as i8;
        self.fCurveShift = (shift - 1) as u8;
        let mut a = SkFDot6ToFixedDiv2(x0.wrapping_sub(x1).wrapping_sub(x1).wrapping_add(x2));
        let mut b = SkFDot6ToFixed(x1.wrapping_sub(x0));
        self.fQx = SkFDot6ToFixed(x0);
        self.fQDx = b.wrapping_add(a >> shift);
        self.fQDDx = a >> (shift - 1);
        a = SkFDot6ToFixedDiv2(y0.wrapping_sub(y1).wrapping_sub(y1).wrapping_add(y2));
        b = SkFDot6ToFixed(y1.wrapping_sub(y0));
        self.fQy = SkFDot6ToFixed(y0);
        self.fQDy = b.wrapping_add(a >> shift);
        self.fQDDy = a >> (shift - 1);
        self.fQLastX = SkFDot6ToFixed(x2);
        self.fQLastY = SkFDot6ToFixed(y2);
        true
    }
    // Source: src/core/SkAnalyticEdge.cpp:425-446
    pub fn setQuadratic<const EVEN: bool>(&mut self, pts: &[SkPoint; 3]) -> bool {
        if !self.setQuadraticWithoutUpdate::<EVEN>(pts, SkAnalyticEdge::kDefaultAccuracy) {
            return false;
        }
        self.fQx >>= 2;
        self.fQy >>= 2;
        self.fQDx >>= 2;
        self.fQDy >>= 2;
        self.fQDDx >>= 2;
        self.fQDDy >>= 2;
        self.fQLastX >>= 2;
        self.fQLastY >>= 2;
        self.fQy = SkAnalyticEdge::SnapY(self.fQy);
        self.fQLastY = SkAnalyticEdge::SnapY(self.fQLastY);
        self.fEdgeType = Type::kQuad;
        self.fSnappedX = self.fQx;
        self.fSnappedY = self.fQy;
        self.updateQuadratic()
    }
    // Source: src/core/SkAnalyticEdge.cpp:448-510
    pub fn updateQuadratic(&mut self) -> bool {
        let mut success = false;
        let mut count = self.fCurveCount as i32;
        let mut oldx = self.fQx;
        let mut oldy = self.fQy;
        let mut dx = self.fQDx;
        let mut dy = self.fQDy;
        let shift = self.fCurveShift;
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 458; @condition b"count > 0\0".as_ptr(); count > 0);
        let (mut newx, mut newy, mut newSnappedX, mut newSnappedY);
        loop {
            let slope;
            count -= 1;
            if count > 0 {
                newx = oldx.wrapping_add(dx >> shift);
                newy = oldy.wrapping_add(dy >> shift);
                if (dy >> shift).wrapping_abs() >= SK_Fixed1 * 2
                    && ((dy.wrapping_abs() as i64) << 6) > dx.wrapping_abs() as i64
                {
                    let diffY = SkFixedToFDot6(newy.wrapping_sub(self.fSnappedY));
                    slope = if diffY != 0 {
                        quick_div(SkFixedToFDot6(newx.wrapping_sub(self.fSnappedX)), diffY)
                    } else {
                        i32::MAX
                    };
                    newSnappedY = self.fQLastY.min(SkFixedRoundToFixed(newy));
                    newSnappedX =
                        newx.wrapping_sub(SkFixedMul(slope, newy.wrapping_sub(newSnappedY)));
                } else {
                    newSnappedY = self.fQLastY.min(SkAnalyticEdge::SnapY(newy));
                    newSnappedX = newx;
                    let diffY = SkFixedToFDot6(newSnappedY.wrapping_sub(self.fSnappedY));
                    slope = if diffY != 0 {
                        quick_div(SkFixedToFDot6(newx.wrapping_sub(self.fSnappedX)), diffY)
                    } else {
                        i32::MAX
                    };
                }
                dx = dx.wrapping_add(self.fQDDx);
                dy = dy.wrapping_add(self.fQDDy);
            } else {
                newx = self.fQLastX;
                newy = self.fQLastY;
                newSnappedY = newy;
                newSnappedX = newx;
                let diffY = SkFixedToFDot6(newy.wrapping_sub(self.fSnappedY));
                slope = if diffY != 0 {
                    quick_div(SkFixedToFDot6(newx.wrapping_sub(self.fSnappedX)), diffY)
                } else {
                    i32::MAX
                };
            }
            if slope < i32::MAX {
                success = self.edge.updateLine(
                    self.fSnappedX,
                    self.fSnappedY,
                    newSnappedX,
                    newSnappedY,
                    slope,
                );
            }
            oldx = newx;
            oldy = newy;
            if !(count > 0 && !success) {
                break;
            }
        }
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 500; @condition b"newSnappedY <= fQLastY\0".as_ptr(); newSnappedY <= self.fQLastY);
        self.fQx = newx;
        self.fQy = newy;
        self.fQDx = dx;
        self.fQDy = dy;
        self.fSnappedX = newSnappedX;
        self.fSnappedY = newSnappedY;
        self.fCurveCount = count as i8;
        success
    }
}
// Source: src/core/SkAnalyticEdge.h:132-151
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SkAnalyticCubicEdge {
    pub edge: SkAnalyticEdge,
    pub fCx: i32,
    pub fCy: i32,
    pub fCDx: i32,
    pub fCDy: i32,
    pub fCDDx: i32,
    pub fCDDy: i32,
    pub fCDDDx: i32,
    pub fCDDDy: i32,
    pub fCLastX: i32,
    pub fCLastY: i32,
    pub fSnappedY: i32,
    pub fToFixedShift: u8,
}
impl core::ops::Deref for SkAnalyticCubicEdge {
    type Target = SkAnalyticEdge;
    fn deref(&self) -> &SkAnalyticEdge {
        &self.edge
    }
}
impl core::ops::DerefMut for SkAnalyticCubicEdge {
    fn deref_mut(&mut self) -> &mut SkAnalyticEdge {
        &mut self.edge
    }
}
impl SkAnalyticCubicEdge {
    pub fn new() -> Self {
        Self {
            edge: SkAnalyticEdge::new(),
            fCx: 0,
            fCy: 0,
            fCDx: 0,
            fCDy: 0,
            fCDDx: 0,
            fCDDy: 0,
            fCDDDx: 0,
            fCDDDy: 0,
            fCLastX: 0,
            fCLastY: 0,
            fSnappedY: 0,
            fToFixedShift: 0,
        }
    }
    pub fn keepContinuous(&mut self) {
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.h\0".as_ptr(), 147; @condition b"SkAbs32(fX - SkFixedMul(fDX, fY - SnapY(fCy)) - fCx) < SK_Fixed1\0".as_ptr(); self.fX
                .wrapping_sub(SkFixedMul(
                    self.fDX,
                    self.fY.wrapping_sub(SkAnalyticEdge::SnapY(self.fCy))
                ))
                .wrapping_sub(self.fCx)
                .wrapping_abs()
                < SK_Fixed1);
        self.fCx = self.fX;
        self.fSnappedY = self.fY;
    }
    // Source: src/core/SkAnalyticEdge.cpp:512-534
    pub fn setCubic<const EVEN: bool>(&mut self, pts: &[SkPoint; 4]) -> bool {
        if !self.setCubicWithoutUpdate::<EVEN>(pts, SkAnalyticEdge::kDefaultAccuracy) {
            return false;
        }
        self.fCx >>= 2;
        self.fCy >>= 2;
        self.fCDx >>= 2;
        self.fCDy >>= 2;
        self.fCDDx >>= 2;
        self.fCDDy >>= 2;
        self.fCDDDx >>= 2;
        self.fCDDDy >>= 2;
        self.fCLastX >>= 2;
        self.fCLastY >>= 2;
        self.fCy = SkAnalyticEdge::SnapY(self.fCy);
        self.fSnappedY = self.fCy;
        self.fCLastY = SkAnalyticEdge::SnapY(self.fCLastY);
        self.fEdgeType = Type::kCubic;
        self.updateCubic()
    }
}
// Source: src/core/SkAnalyticEdge.cpp:536-539
fn SkFDot6UpShift(x: i32, upShift: i32) -> i32 {
    crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 537; @condition b"(SkLeftShift(x, upShift) >> upShift) == x\0".as_ptr(); x.wrapping_shl(upShift as u32) >> upShift == x);
    x.wrapping_shl(upShift as u32)
}
// Source: src/core/SkAnalyticEdge.cpp:549-556
fn cubic_delta_from_line(a: i32, b: i32, c: i32, d: i32) -> i32 {
    let oneThird = a
        .wrapping_mul(8)
        .wrapping_sub(b.wrapping_mul(15))
        .wrapping_add(c.wrapping_mul(6))
        .wrapping_add(d)
        .wrapping_mul(19)
        >> 9;
    let twoThird = a
        .wrapping_add(b.wrapping_mul(6))
        .wrapping_sub(c.wrapping_mul(15))
        .wrapping_add(d.wrapping_mul(8))
        .wrapping_mul(19)
        >> 9;
    oneThird.wrapping_abs().max(twoThird.wrapping_abs())
}
impl SkAnalyticCubicEdge {
    // Source: src/core/SkAnalyticEdge.cpp:558-658
    pub fn setCubicWithoutUpdate<const EVEN: bool>(
        &mut self,
        pts: &[SkPoint; 4],
        mut shift: i32,
    ) -> bool {
        let q = |v: f32| {
            if EVEN {
                SkScalarRoundToFDot6(v, shift)
            } else {
                (v * (1u32 << (shift + 6)) as f32) as i32
            }
        };
        let mut x0 = q(pts[0].fX);
        let mut y0 = q(pts[0].fY);
        let mut x1 = q(pts[1].fX);
        let mut y1 = q(pts[1].fY);
        let mut x2 = q(pts[2].fX);
        let mut y2 = q(pts[2].fY);
        let mut x3 = q(pts[3].fX);
        let mut y3 = q(pts[3].fY);
        let mut winding = Winding::kCW;
        if y0 > y3 {
            core::mem::swap(&mut x0, &mut x3);
            core::mem::swap(&mut x1, &mut x2);
            core::mem::swap(&mut y0, &mut y3);
            core::mem::swap(&mut y1, &mut y2);
            winding = Winding::kCCW;
        }
        let top = SkFDot6Round(y0);
        let bot = SkFDot6Round(y3);
        if top == bot {
            return false;
        }
        let dx = cubic_delta_from_line(x0, x1, x2, x3);
        let dy = cubic_delta_from_line(y0, y1, y2, y3);
        shift = diff_to_shift(dx, dy, 2) + 1;
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 613; @condition b"shift > 0\0".as_ptr(); shift > 0);
        if shift > MAX_COEFF_SHIFT {
            shift = MAX_COEFF_SHIFT;
        }
        let mut upShift = 6;
        let mut downShift = shift + upShift - 10;
        if downShift < 0 {
            downShift = 0;
            upShift = 10 - shift;
        }
        self.fWinding = winding;
        self.fEdgeType = Type::kCubic;
        self.fCurveCount = (-1i32).wrapping_shl(shift as u32) as i8;
        self.fCurveShift = shift as u8;
        self.fToFixedShift = downShift as u8;
        let coeff = |p0: i32, p1: i32, p2: i32, p3: i32| {
            let b = SkFDot6UpShift(p1.wrapping_sub(p0).wrapping_mul(3), upShift);
            let c = SkFDot6UpShift(
                p0.wrapping_sub(p1)
                    .wrapping_sub(p1)
                    .wrapping_add(p2)
                    .wrapping_mul(3),
                upShift,
            );
            let d = SkFDot6UpShift(
                p3.wrapping_add(p1.wrapping_sub(p2).wrapping_mul(3))
                    .wrapping_sub(p0),
                upShift,
            );
            (
                b.wrapping_add(c >> shift).wrapping_add(d >> (2 * shift)),
                (2 * c as i64 + ((3 * d as i64) >> (shift - 1))) as i32,
                ((3 * d as i64) >> (shift - 1)) as i32,
            )
        };
        self.fCx = SkFDot6ToFixed(x0);
        (self.fCDx, self.fCDDx, self.fCDDDx) = coeff(x0, x1, x2, x3);
        self.fCy = SkFDot6ToFixed(y0);
        (self.fCDy, self.fCDDy, self.fCDDDy) = coeff(y0, y1, y2, y3);
        self.fCLastX = SkFDot6ToFixed(x3);
        self.fCLastY = SkFDot6ToFixed(y3);
        true
    }
    // Source: src/core/SkAnalyticEdge.cpp:660-716
    pub fn updateCubic(&mut self) -> bool {
        let mut count = self.fCurveCount as i32;
        let mut oldx = self.fCx;
        let mut oldy = self.fCy;
        let (mut newx, mut newy);
        let ddshift = self.fCurveShift;
        let dshift = self.fToFixedShift;
        crate::SkASSERT!(@source b"src/core/SkAnalyticEdge.cpp\0".as_ptr(), 669; @condition b"count < 0\0".as_ptr(); count < 0);
        let mut success;
        loop {
            count += 1;
            if count < 0 {
                newx = oldx.wrapping_add(self.fCDx >> dshift);
                self.fCDx = self.fCDx.wrapping_add(self.fCDDx >> ddshift);
                self.fCDDx = self.fCDDx.wrapping_add(self.fCDDDx);
                newy = oldy.wrapping_add(self.fCDy >> dshift);
                self.fCDy = self.fCDy.wrapping_add(self.fCDDy >> ddshift);
                self.fCDDy = self.fCDDy.wrapping_add(self.fCDDDy);
            } else {
                newx = self.fCLastX;
                newy = self.fCLastY;
            }
            if newy < oldy {
                newy = oldy;
            }
            let mut newSnappedY = SkAnalyticEdge::SnapY(newy);
            if self.fCLastY < newSnappedY {
                newSnappedY = self.fCLastY;
                count = 0;
            }
            let diffY = SkFixedToFDot6(newSnappedY.wrapping_sub(self.fSnappedY));
            let slope = if diffY == 0 {
                i32::MAX
            } else {
                SkFDot6Div(SkFixedToFDot6(newx.wrapping_sub(oldx)), diffY)
            };
            success = self
                .edge
                .updateLine(oldx, self.fSnappedY, newx, newSnappedY, slope);
            oldx = newx;
            oldy = newy;
            self.fSnappedY = newSnappedY;
            if !(count < 0 && !success) {
                break;
            }
        }
        self.fCx = newx;
        self.fCy = newy;
        self.fCurveCount = count as i8;
        success
    }
}
