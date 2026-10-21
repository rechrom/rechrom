// ARM64 intrinsic helpers remain interleaved with portable numerical tails.
// The canonical Vec owner below has native dependent byte alignment and owned
// recursive halves; no Clang-vector register ABI claim is made.
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
// Copyright 2019 Google LLC. BSD-style license.
// Numerical algorithms below retain array lane views. The canonical Vec owner
// supplies native recursive alignment for arbitrary valid N/element T;
// its borrowed array projection feeds those same existing numerical algorithms.
// Source: src/core/SkVx.h:85-88
// Dependent alignment is selected by the actual byte extent, not by a finite
// lane-type list. Every legal Rust repr(align) power of two has a zero-size
// witness. Its [Marker; 0] member contributes alignment and no occupied bytes.
// Requires the crate's generic_const_exprs language feature. The bounds below
// are the compile-time native N/power-of-two and element-layout requirements.
pub struct NativeAlignment<const BYTES: usize>;
pub trait AlignmentWitness {
    type Marker: Copy;
}
macro_rules! alignment_witness {
    ($name:ident,$bytes:literal) => {
        #[repr(align($bytes))]
        #[derive(Clone, Copy)]
        pub struct $name;
        impl AlignmentWitness for NativeAlignment<$bytes> {
            type Marker = $name;
        }
    };
}
alignment_witness!(Alignment0, 1);
alignment_witness!(Alignment1, 2);
alignment_witness!(Alignment2, 4);
alignment_witness!(Alignment3, 8);
alignment_witness!(Alignment4, 16);
alignment_witness!(Alignment5, 32);
alignment_witness!(Alignment6, 64);
alignment_witness!(Alignment7, 128);
alignment_witness!(Alignment8, 256);
alignment_witness!(Alignment9, 512);
alignment_witness!(Alignment10, 1024);
alignment_witness!(Alignment11, 2048);
alignment_witness!(Alignment12, 4096);
alignment_witness!(Alignment13, 8192);
alignment_witness!(Alignment14, 16384);
alignment_witness!(Alignment15, 32768);
alignment_witness!(Alignment16, 65536);
alignment_witness!(Alignment17, 131072);
alignment_witness!(Alignment18, 262144);
alignment_witness!(Alignment19, 524288);
alignment_witness!(Alignment20, 1048576);
alignment_witness!(Alignment21, 2097152);
alignment_witness!(Alignment22, 4194304);
alignment_witness!(Alignment23, 8388608);
alignment_witness!(Alignment24, 16777216);
alignment_witness!(Alignment25, 33554432);
alignment_witness!(Alignment26, 67108864);
alignment_witness!(Alignment27, 134217728);
alignment_witness!(Alignment28, 268435456);
alignment_witness!(Alignment29, 536870912);
#[doc(hidden)]
pub const fn scalar_requirement(n: usize, size: usize) -> bool {
    n == 1 && size > 0
}
#[doc(hidden)]
pub const fn pair_requirement(n: usize, size: usize, align: usize) -> bool {
    n > 1 && (n & (n - 1)) == 0 && size >= align
}
#[doc(hidden)]
pub const fn half_requirement(n: usize) -> bool {
    n > 1 && (n & (n - 1)) == 0
}
pub struct NativeRequirement<const VALID: bool>;
pub trait NativeRequirementMet {}
impl NativeRequirementMet for NativeRequirement<true> {}
// Rust stores the same contiguous N lanes in one aligned object. This is
// layout-equivalent to the native recursive lo/hi representation; half views
// below project the two exact native subobject ranges without recursively
// asking the trait solver to normalize Vec<N/2, T>.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeFlatStorage<T, const N: usize, A> {
    lanes: [T; N],
    alignment: [A; 0],
}
pub trait NativeStorageAccess<T, const N: usize> {
    fn from_array(values: [T; N]) -> Self;
    fn lanes(&self) -> &[T; N];
    fn lanes_mut(&mut self) -> &mut [T; N];
}
impl<T, const N: usize, A> NativeStorageAccess<T, N> for NativeFlatStorage<T, N, A> {
    fn from_array(values: [T; N]) -> Self {
        Self {
            lanes: values,
            alignment: [],
        }
    }
    fn lanes(&self) -> &[T; N] {
        &self.lanes
    }
    fn lanes_mut(&mut self) -> &mut [T; N] {
        &mut self.lanes
    }
}

pub trait NativeLayout<const N: usize>: Sized + Copy {
    type Storage;
    fn into_storage(values: [Self; N]) -> Self::Storage;
    fn lanes(storage: &Self::Storage) -> &[Self; N];
    fn lanes_mut(storage: &mut Self::Storage) -> &mut [Self; N];
}
impl<T: Copy, const N: usize> NativeLayout<N> for T
where
    [(); N * core::mem::size_of::<T>()]:,
    NativeRequirement<{ N > 0 }>: NativeRequirementMet,
    NativeRequirement<{ (N & (N - 1)) == 0 }>: NativeRequirementMet,
    NativeRequirement<{ core::mem::size_of::<T>() >= core::mem::align_of::<T>() }>:
        NativeRequirementMet,
    NativeAlignment<{ N * core::mem::size_of::<T>() }>: AlignmentWitness,
{
    type Storage = NativeFlatStorage<
        T,
        N,
        <NativeAlignment<{ N * core::mem::size_of::<T>() }> as AlignmentWitness>::Marker,
    >;
    fn into_storage(values: [Self; N]) -> Self::Storage {
        NativeStorageAccess::from_array(values)
    }
    fn lanes(s: &Self::Storage) -> &[Self; N] {
        NativeStorageAccess::lanes(s)
    }
    fn lanes_mut(s: &mut Self::Storage) -> &mut [Self; N] {
        NativeStorageAccess::lanes_mut(s)
    }
}
mod layout_sealed {
    pub trait Supported<const N: usize> {}
}
impl<T: Copy, const N: usize> layout_sealed::Supported<N> for T where T: NativeLayout<N> {}
// Source: src/core/SkVx.h:85-88
// Source: src/core/SkVx.h:125-127
// Source: src/core/SkVx.h:176-178
#[repr(transparent)]
pub struct Vec<const N: usize, T: NativeLayout<N>> {
    storage: T::Storage,
}
// Copy is a property of the actual owned storage, never an unconditional
// promise for generic T. Clone invokes each lane's genuine Clone operation.
impl<const N: usize, T: NativeLayout<N> + Copy> Copy for Vec<N, T> where T::Storage: Copy {}
impl<const N: usize, T: NativeLayout<N> + Clone> Clone for Vec<N, T> {
    fn clone(&self) -> Self {
        Self::from_array(core::array::from_fn(|i| self.as_array()[i].clone()))
    }
}
// Source: src/core/SkVx.h:95-95
// Source: src/core/SkVx.h:129-129
// Source: src/core/SkVx.h:179-179
// Source: src/core/SkVx.h:220-220
// T's real value-initialization/default operation is required independently of
// the layout. No all-zero bit pattern or invented Clone is imposed on owners.
const impl<const N: usize, T: NativeLayout<N> + Default> Default for Vec<N, T>
where
    T::Storage: [const] Default,
{
    fn default() -> Self {
        Self {
            storage: <T::Storage as Default>::default(),
        }
    }
}
impl<const N: usize, T: NativeLayout<N>> Vec<N, T> {
    // Moving the initialized array transfers each original T exactly once into
    // its actual recursive owned fields; ManuallyDrop prevents a second drop.
    pub fn from_array(values: [T; N]) -> Self {
        Self {
            storage: T::into_storage(values),
        }
    }
    pub fn into_array(self) -> [T; N] {
        let owner = core::mem::ManuallyDrop::new(self);
        unsafe { core::ptr::read(core::ptr::from_ref(&owner.storage).cast::<[T; N]>()) }
    }
    pub fn as_array(&self) -> &[T; N] {
        T::lanes(&self.storage)
    }
    pub fn as_array_mut(&mut self) -> &mut [T; N] {
        T::lanes_mut(&mut self.storage)
    }
    pub fn storage(&self) -> &T::Storage {
        &self.storage
    }
    pub fn storage_mut(&mut self) -> &mut T::Storage {
        &mut self.storage
    }
    // Source: src/core/SkVx.h:112-118
    /// Safety: ptr is readable for N valid T lanes, possibly unaligned. Native
    /// memcpy is only valid for bitwise-copy-safe lane values; for an owning T,
    /// this raw operation must transfer initialization rather than duplicate an
    /// independently droppable owner. The caller must uphold that obligation.
    pub unsafe fn load(src: *const u8) -> Self {
        Self::from_array(unsafe { core::ptr::read_unaligned(src.cast::<[T; N]>()) })
    }
    /// Safety: dst covers the full byte extent and does not overlap this owner.
    /// The bytes do not create a second safe owning T allocation. Any later
    /// interpretation must satisfy the native bitwise-copy/ownership contract.
    pub unsafe fn store(&self, dst: *mut u8) {
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.as_array().as_ptr().cast::<u8>(),
                dst,
                core::mem::size_of::<[T; N]>(),
            )
        }
    }
}
impl<const N: usize, T: NativeLayout<N> + Clone> Vec<N, T> {
    // Source: src/core/SkVx.h:96-96
    pub fn splat(value: T) -> Self {
        Self::from_array(core::array::from_fn(|_| value.clone()))
    }
}
impl<const N: usize, T: NativeLayout<N>> core::ops::Deref for Vec<N, T> {
    type Target = [T; N];
    fn deref(&self) -> &Self::Target {
        self.as_array()
    }
}
impl<const N: usize, T: NativeLayout<N>> core::ops::DerefMut for Vec<N, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_array_mut()
    }
}
// Source: src/core/SkVx.h:120-120
// Source: src/core/SkVx.h:173-173
// Source: src/core/SkVx.h:213-213
pub trait NativeHalf<const N: usize>: NativeLayout<N> {
    type Half;
    fn low(storage: &Self::Storage) -> &Self::Half;
    fn high(storage: &Self::Storage) -> &Self::Half;
    fn low_mut(storage: &mut Self::Storage) -> &mut Self::Half;
    fn high_mut(storage: &mut Self::Storage) -> &mut Self::Half;
}
// Source: src/core/SkVx.h:120-120
// Source: src/core/SkVx.h:173-173
// Source: src/core/SkVx.h:213-213
impl<T, const N: usize> NativeHalf<N> for T
where
    NativeRequirement<{ half_requirement(N) }>: NativeRequirementMet,
    T: NativeLayout<N> + NativeLayout<{ N / 2 }>,
{
    type Half = Vec<{ N / 2 }, T>;
    fn low(s: &<Self as NativeLayout<N>>::Storage) -> &Self::Half {
        unsafe { &*<T as NativeLayout<N>>::lanes(s).as_ptr().cast() }
    }
    fn high(s: &<Self as NativeLayout<N>>::Storage) -> &Self::Half {
        unsafe { &*<T as NativeLayout<N>>::lanes(s).as_ptr().add(N / 2).cast() }
    }
    fn low_mut(s: &mut <Self as NativeLayout<N>>::Storage) -> &mut Self::Half {
        unsafe { &mut *<T as NativeLayout<N>>::lanes_mut(s).as_mut_ptr().cast() }
    }
    fn high_mut(s: &mut <Self as NativeLayout<N>>::Storage) -> &mut Self::Half {
        unsafe {
            &mut *<T as NativeLayout<N>>::lanes_mut(s)
                .as_mut_ptr()
                .add(N / 2)
                .cast()
        }
    }
}
impl<const N: usize, T: NativeHalf<N>> Vec<N, T> {
    pub fn lo(&self) -> &T::Half {
        T::low(&self.storage)
    }
    pub fn hi(&self) -> &T::Half {
        T::high(&self.storage)
    }
    pub fn lo_mut(&mut self) -> &mut T::Half {
        T::low_mut(&mut self.storage)
    }
    pub fn hi_mut(&mut self) -> &mut T::Half {
        T::high_mut(&mut self.storage)
    }
}
impl<const N: usize, T: NativeLayout<N> + ZeroLane> Vec<N, T> {
    // Source: src/core/SkVx.h:100-107
    // Source: src/core/SkVx.h:136-143
    // Source: src/core/SkVx.h:183-190
    // Source: src/core/SkVx.h:223-225
    // This is the native memcpy/zero initializer-list method, distinct from the
    // class's ability to own non-Copy lanes. Only real ZeroLane values use it.
    pub fn from_list(xs: &[T]) -> Self {
        debug_assert!(xs.len() <= N);
        let mut vals = [T::ZERO; N];
        let count = xs.len().min(N);
        vals[..count].copy_from_slice(&xs[..count]);
        Self::from_array(vals)
    }
}
impl<const N: usize, T: NativeLayout<N>> Vec<N, T> {
    // Source: src/core/SkVx.h:112-114
    // Source: src/core/SkVx.h:148-150
    // Source: src/core/SkVx.h:195-197
    // Source: src/core/SkVx.h:230-232
    /// Safety: same real bitwise-copy/ownership boundary as load().
    pub unsafe fn Load(ptr: *const core::ffi::c_void) -> Self {
        unsafe { Self::load(ptr.cast()) }
    }
    // Source: src/core/SkVx.h:110-110
    pub fn lane_mut(&mut self, i: i32) -> &mut T {
        &mut self.as_array_mut()[i as usize]
    }
}
impl<const N: usize, T: NativeLayout<N> + Clone> Vec<N, T> {
    // Source: src/core/SkVx.h:109-109
    pub fn lane(&self, i: i32) -> T {
        self.as_array()[i as usize].clone()
    }
}
impl<T: NativeLayout<1>> Vec<1, T> {
    // Source: src/core/SkVx.h:218-218
    pub fn from_value(value: T) -> Self {
        Self::from_array([value])
    }
    pub fn val_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[0]
    }
}
impl<T: NativeLayout<1> + Clone> Vec<1, T> {
    pub fn val(&self) -> T {
        self.as_array()[0].clone()
    }
}
impl<T: NativeLayout<2>> Vec<2, T> {
    // Source: src/core/SkVx.h:181-181
    pub fn from_xy(x: T, y: T) -> Self {
        Self::from_array([x, y])
    }
    // Source: src/core/SkVx.h:202-203
    pub fn x_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[0]
    }
    pub fn y_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[1]
    }
}
impl<T: NativeLayout<2> + Clone> Vec<2, T> {
    // Source: src/core/SkVx.h:205-206
    pub fn x(&self) -> T {
        self.as_array()[0].clone()
    }
    pub fn y(&self) -> T {
        self.as_array()[1].clone()
    }
    // Source: src/core/SkVx.h:209-209
    pub fn yx(&self) -> Self {
        Self::from_array([self.y(), self.x()])
    }
}
impl<T: NativeLayout<2> + NativeLayout<4> + Clone> Vec<2, T> {
    // Source: src/core/SkVx.h:210-210
    pub fn xyxy(&self) -> Vec<4, T> {
        Vec::<4, T>::from_array([self.x(), self.y(), self.x(), self.y()])
    }
}
impl<T: NativeLayout<4>> Vec<4, T> {
    // Source: src/core/SkVx.h:131-131
    pub fn from_xyzw(x: T, y: T, z: T, w: T) -> Self {
        Self::from_array([x, y, z, w])
    }
    // Source: src/core/SkVx.h:157-160
    pub fn x_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[0]
    }
    pub fn y_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[1]
    }
    pub fn z_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[2]
    }
    pub fn w_mut(&mut self) -> &mut T {
        &mut self.as_array_mut()[3]
    }
}
impl<T: NativeLayout<4> + Clone> Vec<4, T> {
    // Source: src/core/SkVx.h:164-167
    pub fn x(&self) -> T {
        self.as_array()[0].clone()
    }
    pub fn y(&self) -> T {
        self.as_array()[1].clone()
    }
    pub fn z(&self) -> T {
        self.as_array()[2].clone()
    }
    pub fn w(&self) -> T {
        self.as_array()[3].clone()
    }
    // Source: src/core/SkVx.h:170-171
    pub fn yxwz(&self) -> Self {
        Self::from_array([self.y(), self.x(), self.w(), self.z()])
    }
    pub fn zwxy(&self) -> Self {
        Self::from_array([self.z(), self.w(), self.x(), self.y()])
    }
}
impl<T: NativeHalf<4, Half = Vec<2, T>> + NativeLayout<2>> Vec<4, T> {
    // Source: src/core/SkVx.h:155-156
    pub fn xy_mut(&mut self) -> &mut Vec<2, T> {
        self.lo_mut()
    }
    pub fn zw_mut(&mut self) -> &mut Vec<2, T> {
        self.hi_mut()
    }
}
impl<T: NativeHalf<4, Half = Vec<2, T>> + NativeLayout<2> + Clone> Vec<4, T> {
    // Source: src/core/SkVx.h:132-134
    pub fn from_xy_z_w(xy: Vec<2, T>, z: T, w: T) -> Self {
        Self::from_array([xy.x(), xy.y(), z, w])
    }
    pub fn from_x_y_zw(x: T, y: T, zw: Vec<2, T>) -> Self {
        Self::from_array([x, y, zw.x(), zw.y()])
    }
    pub fn from_xy_zw(xy: Vec<2, T>, zw: Vec<2, T>) -> Self {
        Self::from_array([xy.x(), xy.y(), zw.x(), zw.y()])
    }
    // Source: src/core/SkVx.h:162-163
    pub fn xy(&self) -> Vec<2, T> {
        self.lo().clone()
    }
    pub fn zw(&self) -> Vec<2, T> {
        self.hi().clone()
    }
}
// Native VecSize=16/sizeof(T) for current portable CPU memset. This sealed
// selected-instantiation interface keeps real typed consumers generic without
// claiming representation support for an arbitrary Copy aggregate.
pub trait WideLane: Copy + layout_sealed::Supported<1> {
    type Wide: Copy;
    const WIDTH: usize;
    fn splat_wide(value: Self) -> Self::Wide;
    /// Safety: dst writable for WIDTH lanes; does not alias value.
    unsafe fn store_wide(value: &Self::Wide, dst: *mut Self);
}
macro_rules! wide_lane {
    ($t:ty, $n:literal) => {
        impl WideLane for $t {
            type Wide = Vec<$n, Self>;
            const WIDTH: usize = $n;
            fn splat_wide(value: Self) -> Self::Wide {
                Vec::splat(value)
            }
            unsafe fn store_wide(value: &Self::Wide, dst: *mut Self) {
                value.store(dst.cast());
            }
        }
    };
}
wide_lane!(u8, 16);
wide_lane!(u16, 8);
wide_lane!(u32, 4);
wide_lane!(u64, 2);
wide_lane!(super::SkColor::SkColor4f, 1);
// Source: src/core/SkVx.h:95-95
pub fn zero_large<const N: usize, T: ZeroLane>() -> [T; N] {
    assert!(N >= 8 && N.is_power_of_two());
    [T::ZERO; N]
}
// Source: src/core/SkVx.h:96-96
pub fn splat<const N: usize, T: Copy>(value: T) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    [value; N]
}
// Source: src/core/SkVx.h:100-103
// Source: src/core/SkVx.h:105-107
pub fn from_list_large<const N: usize, T: ZeroLane>(xs: &[T]) -> [T; N] {
    assert!(N >= 8 && N.is_power_of_two());
    let mut vals = [T::ZERO; N];
    debug_assert!(xs.len() <= N);
    let count = xs.len().min(N);
    vals[..count].copy_from_slice(&xs[..count]);
    // Native lo/hi unaligned loads become two copies into the same contiguous
    // lane representation; SIMD alignment of the C++ aggregate is not claimed.
    let mut out = [T::ZERO; N];
    let half = N / 2;
    out[..half].copy_from_slice(&vals[..half]);
    out[half..].copy_from_slice(&vals[half..]);
    out
}
// Source: src/core/SkVx.h:109-109
pub fn lane_large<const N: usize, T: Copy>(v: &[T; N], i: i32) -> T {
    assert!(N >= 8 && N.is_power_of_two());
    fn at<T: Copy>(v: &[T], i: i32) -> T {
        if v.len() == 1 {
            debug_assert_eq!(i, 0);
            return v[0];
        }
        let half = v.len() / 2;
        if i < half as i32 {
            at(&v[..half], i)
        } else {
            at(&v[half..], i - half as i32)
        }
    }
    at(v, i)
}
// Source: src/core/SkVx.h:110-110
pub fn lane_large_mut<const N: usize, T>(v: &mut [T; N], i: i32) -> &mut T {
    assert!(N >= 8 && N.is_power_of_two());
    fn at<T>(v: &mut [T], i: i32) -> &mut T {
        if v.len() == 1 {
            debug_assert_eq!(i, 0);
            return &mut v[0];
        }
        let half = v.len() / 2;
        if i < half as i32 {
            at(&mut v[..half], i)
        } else {
            at(&mut v[half..], i - half as i32)
        }
    }
    at(v, i)
}
/// The source must contain N initialized valid T values; unaligned addresses are permitted.
// Source: src/core/SkVx.h:112-114
pub unsafe fn load<const N: usize, T: Copy>(src: *const u8) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    std::ptr::read_unaligned(src.cast::<[T; N]>())
}
/// The destination must have room for N lanes and must not overlap the source array.
/// Like native memcpy, this accepts an unaligned destination.
// Source: src/core/SkVx.h:115-115
// Source: src/core/SkVx.h:117-118
pub unsafe fn store<const N: usize, T: Copy>(value: &[T; N], dst: *mut u8) {
    assert!(N > 0 && N.is_power_of_two());
    std::ptr::copy_nonoverlapping(
        value.as_ptr().cast::<u8>(),
        dst,
        std::mem::size_of::<[T; N]>(),
    );
}
// Source: src/core/SkVx.h:129-129
pub fn zero4<T: ZeroLane>() -> [T; 4] {
    [T::ZERO; 4]
}
// Source: src/core/SkVx.h:130-130
pub fn splat4<T: Copy>(s: T) -> [T; 4] {
    [s; 4]
}
// Source: src/core/SkVx.h:131-131
pub fn from_xyzw<T: Copy>(x: T, y: T, z: T, w: T) -> [T; 4] {
    [x, y, z, w]
}
// Source: src/core/SkVx.h:132-132
pub fn from_xy_z_w<T: Copy>(xy: &[T; 2], z: T, w: T) -> [T; 4] {
    [xy[0], xy[1], z, w]
}
// Source: src/core/SkVx.h:133-133
pub fn from_x_y_zw<T: Copy>(x: T, y: T, zw: &[T; 2]) -> [T; 4] {
    [x, y, zw[0], zw[1]]
}
// Source: src/core/SkVx.h:134-134
pub fn from_xy_zw<T: Copy>(xy: &[T; 2], zw: &[T; 2]) -> [T; 4] {
    join(xy, zw)
}
// Source: src/core/SkVx.h:136-139
// Source: src/core/SkVx.h:141-143
pub fn from_list4<T: ZeroLane>(xs: &[T]) -> [T; 4] {
    let mut vals = [T::ZERO; 4];
    debug_assert!(xs.len() <= 4);
    let count = xs.len().min(4);
    vals[..count].copy_from_slice(&xs[..count]);
    let lo = unsafe { load::<2, T>(vals.as_ptr().cast()) };
    let hi = unsafe { load::<2, T>(vals.as_ptr().add(2).cast()) };
    join(&lo, &hi)
}
// Source: src/core/SkVx.h:145-145
pub fn lane4<T: Copy>(v: &[T; 4], i: i32) -> T {
    if i < 2 {
        lane2(&[v[0], v[1]], i)
    } else {
        lane2(&[v[2], v[3]], i - 2)
    }
}
// Source: src/core/SkVx.h:146-146
pub fn lane4_mut<T>(v: &mut [T; 4], i: i32) -> &mut T {
    if i < 2 {
        lane2_mut(xy4_mut(v), i)
    } else {
        lane2_mut(zw4_mut(v), i - 2)
    }
}
// Source: src/core/SkVx.h:148-150
/// Safety: src holds four initialized valid numeric lanes, with arbitrary byte alignment.
pub unsafe fn load4<T: Copy>(src: *const u8) -> [T; 4] {
    std::ptr::read_unaligned(src.cast())
}
// Source: src/core/SkVx.h:151-153
/// Safety: dst is writable for four lanes and does not overlap v; unaligned is allowed.
pub unsafe fn store4<T: Copy>(v: &[T; 4], dst: *mut u8) {
    std::ptr::copy_nonoverlapping(v.as_ptr().cast::<u8>(), dst, std::mem::size_of::<[T; 4]>());
}
// Source: src/core/SkVx.h:155-155
pub fn xy4_mut<T>(v: &mut [T; 4]) -> &mut [T; 2] {
    (&mut v[..2]).try_into().unwrap()
}
// Source: src/core/SkVx.h:156-156
pub fn zw4_mut<T>(v: &mut [T; 4]) -> &mut [T; 2] {
    (&mut v[2..]).try_into().unwrap()
}
// Source: src/core/SkVx.h:157-157
pub fn x4_mut<T>(v: &mut [T; 4]) -> &mut T {
    &mut v[0]
}
// Source: src/core/SkVx.h:158-158
pub fn y4_mut<T>(v: &mut [T; 4]) -> &mut T {
    &mut v[1]
}
// Source: src/core/SkVx.h:159-159
pub fn z4_mut<T>(v: &mut [T; 4]) -> &mut T {
    &mut v[2]
}
// Source: src/core/SkVx.h:160-160
pub fn w4_mut<T>(v: &mut [T; 4]) -> &mut T {
    &mut v[3]
}
// Source: src/core/SkVx.h:162-162
pub fn xy4<T: Copy>(v: &[T; 4]) -> [T; 2] {
    [v[0], v[1]]
}
// Source: src/core/SkVx.h:163-163
pub fn zw4<T: Copy>(v: &[T; 4]) -> [T; 2] {
    [v[2], v[3]]
}
// Source: src/core/SkVx.h:164-164
pub fn x4<T: Copy>(v: &[T; 4]) -> T {
    v[0]
}
// Source: src/core/SkVx.h:165-165
pub fn y4<T: Copy>(v: &[T; 4]) -> T {
    v[1]
}
// Source: src/core/SkVx.h:166-166
pub fn z4<T: Copy>(v: &[T; 4]) -> T {
    v[2]
}
// Source: src/core/SkVx.h:167-167
pub fn w4<T: Copy>(v: &[T; 4]) -> T {
    v[3]
}
// Source: src/core/SkVx.h:170-170
pub fn yxwz<T: Copy>(x: &[T; 4]) -> [T; 4] {
    shuffle(*x, [1, 0, 3, 2])
}
// Source: src/core/SkVx.h:171-171
pub fn zwxy<T: Copy>(x: &[T; 4]) -> [T; 4] {
    shuffle(*x, [2, 3, 0, 1])
}
// Source: src/core/SkVx.h:180-180
pub fn zero2<T: ZeroLane>() -> [T; 2] {
    [T::ZERO; 2]
}
// Source: src/core/SkVx.h:181-181
pub fn splat2<T: Copy>(s: T) -> [T; 2] {
    [s; 2]
}
// Source: src/core/SkVx.h:182-182
pub fn from_xy<T: Copy>(x: T, y: T) -> [T; 2] {
    [x, y]
}
// Source: src/core/SkVx.h:184-187
// Source: src/core/SkVx.h:189-191
pub fn from_list2<T: ZeroLane>(xs: &[T]) -> [T; 2] {
    let mut vals = [T::ZERO; 2];
    debug_assert!(xs.len() <= 2);
    let count = xs.len().min(2);
    vals[..count].copy_from_slice(&xs[..count]);
    let lo = unsafe { load::<1, T>(vals.as_ptr().cast()) };
    let hi = unsafe { load::<1, T>(vals.as_ptr().add(1).cast()) };
    join(&lo, &hi)
}
// Source: src/core/SkVx.h:193-193
pub fn lane2<T: Copy>(v: &[T; 2], i: i32) -> T {
    if i < 1 {
        lane1(&[v[0]], i)
    } else {
        lane1(&[v[1]], i - 1)
    }
}
// Source: src/core/SkVx.h:194-194
pub fn lane2_mut<T>(v: &mut [T; 2], i: i32) -> &mut T {
    if i < 1 {
        debug_assert_eq!(i, 0);
        &mut v[0]
    } else {
        debug_assert_eq!(i - 1, 0);
        &mut v[1]
    }
}
// Source: src/core/SkVx.h:196-198
/// Safety: src holds two initialized valid numeric lanes, with arbitrary byte alignment.
pub unsafe fn load2<T: Copy>(src: *const u8) -> [T; 2] {
    std::ptr::read_unaligned(src.cast())
}
// Source: src/core/SkVx.h:199-201
/// Safety: dst is writable for two lanes and does not overlap v; unaligned is allowed.
pub unsafe fn store2<T: Copy>(v: &[T; 2], dst: *mut u8) {
    std::ptr::copy_nonoverlapping(v.as_ptr().cast::<u8>(), dst, std::mem::size_of::<[T; 2]>());
}
// Source: src/core/SkVx.h:203-203
pub fn x2_mut<T>(v: &mut [T; 2]) -> &mut T {
    &mut v[0]
}
// Source: src/core/SkVx.h:204-204
pub fn y2_mut<T>(v: &mut [T; 2]) -> &mut T {
    &mut v[1]
}
// Source: src/core/SkVx.h:206-206
pub fn x2<T: Copy>(v: &[T; 2]) -> T {
    v[0]
}
// Source: src/core/SkVx.h:207-207
pub fn y2<T: Copy>(v: &[T; 2]) -> T {
    v[1]
}
// Source: src/core/SkVx.h:210-210
pub fn yx<T: Copy>(x: &[T; 2]) -> [T; 2] {
    shuffle(*x, [1, 0])
}
// Source: src/core/SkVx.h:211-211
pub fn xyxy<T: Copy>(x: &[T; 2]) -> [T; 4] {
    join(x, x)
}
// Source: src/core/SkVx.h:220-220
pub fn zero1<T: ZeroLane>() -> [T; 1] {
    [T::ZERO]
}
// Source: src/core/SkVx.h:221-221
pub fn splat1<T: Copy>(s: T) -> [T; 1] {
    [s]
}
// Source: src/core/SkVx.h:223-225
pub fn from_list1<T: ZeroLane>(xs: &[T]) -> [T; 1] {
    let val = xs.first().copied().unwrap_or(T::ZERO);
    debug_assert!(xs.len() <= 1);
    [val]
}
// Source: src/core/SkVx.h:227-227
pub fn lane1<T: Copy>(v: &[T; 1], i: i32) -> T {
    debug_assert_eq!(i, 0);
    v[0]
}
// Source: src/core/SkVx.h:228-228
pub fn lane1_mut<T>(v: &mut [T; 1], i: i32) -> &mut T {
    debug_assert_eq!(i, 0);
    &mut v[0]
}
// Source: src/core/SkVx.h:230-232
/// Safety: src holds one initialized valid numeric lane, with arbitrary byte alignment.
pub unsafe fn load1<T: Copy>(src: *const u8) -> [T; 1] {
    std::ptr::read_unaligned(src.cast())
}
// Source: src/core/SkVx.h:233-235
/// Safety: dst is writable for one lane and does not overlap v; unaligned is allowed.
pub unsafe fn store1<T: Copy>(v: &[T; 1], dst: *mut u8) {
    std::ptr::copy_nonoverlapping(v.as_ptr().cast::<u8>(), dst, std::mem::size_of::<[T; 1]>());
}
// Uncredited Rust backend-selection protocol. Only concrete primitive impls
// construct unsafe intrinsic bridges; external safe trait impls default to the
// initialized portable algorithm. skvx_disable_simd maps native SKVX_DISABLE_SIMD.
macro_rules! primitive_select_backend {
    ($t:ty,$m:ty) => {
        fn select_backend(cond: &[$m], t: &[$t], e: &[$t], out: &mut [$t]) -> bool {
            #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
            if t.len() > 1 && std::mem::size_of_val(t) == 16 {
                let selected = unsafe {
                    if_then_else_neon128::<{ 16 / std::mem::size_of::<$t>() }, $t>(
                        cond.try_into().unwrap(),
                        t.try_into().unwrap(),
                        e.try_into().unwrap(),
                    )
                };
                out.copy_from_slice(&selected);
                return true;
            }
            false
        }
    };
}
// Native M<T> and sk_bit_cast mapping for supported primitive numeric lanes.
// Source: src/core/SkVx.h:239-242
pub trait BitLane: Copy {
    type Mask: Copy
        + std::ops::BitAnd<Output = Self::Mask>
        + std::ops::BitOr<Output = Self::Mask>
        + std::ops::Not<Output = Self::Mask>;
    fn to_mask(self) -> Self::Mask;
    fn from_mask(bits: Self::Mask) -> Self;
    fn select_backend(_cond: &[Self::Mask], _t: &[Self], _e: &[Self], _out: &mut [Self]) -> bool {
        false
    }
}
macro_rules! integer_bits {($($t:ty=>$m:ty),*)=>{$(impl BitLane for $t {type Mask=$m;fn to_mask(self)->$m{self as $m}fn from_mask(bits:$m)->Self{bits as $t}primitive_select_backend!($t,$m);})*};}
integer_bits!(u8=>u8,i8=>i8,u16=>u16,i16=>i16,u32=>u32,i32=>i32,u64=>u64,i64=>i64,usize=>usize,isize=>isize);
impl BitLane for f32 {
    type Mask = i32;
    fn to_mask(self) -> i32 {
        self.to_bits() as i32
    }
    fn from_mask(bits: i32) -> Self {
        Self::from_bits(bits as u32)
    }
    primitive_select_backend!(f32, i32);
}
impl BitLane for f64 {
    type Mask = i64;
    fn to_mask(self) -> i64 {
        self.to_bits() as i64
    }
    fn from_mask(bits: i64) -> Self {
        Self::from_bits(bits as u64)
    }
    primitive_select_backend!(f64, i64);
}
// Rust stable const generics pass the destination width explicitly; it must equal 2*N.
// Source: src/core/SkVx.h:245-250
pub fn join<const N: usize, const O: usize, T: Copy>(lo: &[T; N], hi: &[T; N]) -> [T; O] {
    assert!(N > 0 && N.is_power_of_two() && O == 2 * N);
    let mut v = std::mem::MaybeUninit::<[T; O]>::uninit();
    unsafe {
        let dst = v.as_mut_ptr().cast::<T>();
        std::ptr::copy_nonoverlapping(lo.as_ptr(), dst, N);
        std::ptr::copy_nonoverlapping(hi.as_ptr(), dst.add(N), N);
        v.assume_init()
    }
}
// Native narrow integer division promotes both operands to int32 before narrowing.
// In particular, i8/i16 MIN/-1 is a defined promoted operation on this target.
pub trait DivLane: Copy {
    fn divide(self, rhs: Self) -> Self;
}
macro_rules! narrow_div {($($t:ty),*)=>{$(impl DivLane for $t {fn divide(self,rhs:Self)->Self{((self as i32)/(rhs as i32)) as $t}})*};}
narrow_div!(u8, i8, u16, i16);
macro_rules! wide_div {($($t:ty),*)=>{$(impl DivLane for $t {fn divide(self,rhs:Self)->Self{self/rhs}})*};}
wide_div!(u32, i32, u64, i64, usize, isize, f32, f64);
// Numeric primitive substitutions for native unary expressions; signed negation
// parity covers inputs where the native expression is defined.
pub trait NegLane: Copy {
    fn negate(self) -> Self;
}
macro_rules! neg_integer {($($t:ty),*)=>{$(impl NegLane for $t {fn negate(self)->Self{self.wrapping_neg()}})*};}
neg_integer!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize);
impl NegLane for f32 {
    fn negate(self) -> Self {
        -self
    }
}
impl NegLane for f64 {
    fn negate(self) -> Self {
        -self
    }
}
pub trait LogicalLane: Copy {
    fn logical_not(self) -> Self;
}
macro_rules! logical_numeric {($($t:ty),*)=>{$(impl LogicalLane for $t {fn logical_not(self)->Self{if self==0 as $t {1 as $t} else {0 as $t}}})*};}
logical_numeric!(u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, f32, f64);
impl LogicalLane for bool {
    fn logical_not(self) -> Self {
        !self
    }
}
// C++ integral promotion is significant for 8/16-bit shifts. Small lanes shift
// as int32 before narrowing; signed right shift follows the selected target's arithmetic shift.
// Shift counts must lie in the native promoted type's valid range.
pub trait ShiftLane: Copy {
    fn shift_left(self, k: i32) -> Self;
    fn shift_right(self, k: i32) -> Self;
}
macro_rules! shift_lane {
    ($t:ty,$w:ty) => {
        impl ShiftLane for $t {
            fn shift_left(self, k: i32) -> Self {
                assert!(k >= 0 && k < <$w>::BITS as i32);
                (self as $w).wrapping_shl(k as u32) as $t
            }
            fn shift_right(self, k: i32) -> Self {
                assert!(k >= 0 && k < <$w>::BITS as i32);
                ((self as $w) >> k) as $t
            }
        }
    };
}
shift_lane!(u8, i32);
shift_lane!(i8, i32);
shift_lane!(u16, i32);
shift_lane!(i16, i32);
shift_lane!(u32, u32);
shift_lane!(i32, i32);
shift_lane!(u64, u64);
shift_lane!(i64, i64);
shift_lane!(usize, usize);
shift_lane!(isize, isize);
// Source: src/core/SkVx.h:338-338
fn add_lane<T: LaneArithmetic>(x: T, y: T) -> T {
    x.lane_add(y)
}
// Source: src/core/SkVx.h:339-339
fn sub_lane<T: LaneArithmetic>(x: T, y: T) -> T {
    x.lane_sub(y)
}
// Source: src/core/SkVx.h:340-340
fn mul_lane<T: LaneArithmetic>(x: T, y: T) -> T {
    x.lane_mul(y)
}
// Source: src/core/SkVx.h:341-341
fn div_lane<T: DivLane>(x: T, y: T) -> T {
    x.divide(y)
}
// Source: src/core/SkVx.h:343-343
fn bit_xor_lane<T: Copy + std::ops::BitXor<Output = T>>(x: T, y: T) -> T {
    x ^ y
}
// Source: src/core/SkVx.h:344-344
fn bit_and_lane<T: Copy + std::ops::BitAnd<Output = T>>(x: T, y: T) -> T {
    x & y
}
// Source: src/core/SkVx.h:345-345
fn bit_or_lane<T: Copy + std::ops::BitOr<Output = T>>(x: T, y: T) -> T {
    x | y
}
// Source: src/core/SkVx.h:347-347
fn logical_not_lane<T: LogicalLane>(x: T) -> T {
    x.logical_not()
}
// Source: src/core/SkVx.h:348-348
fn neg_lane<T: NegLane>(x: T) -> T {
    x.negate()
}
// Source: src/core/SkVx.h:349-349
fn bit_not_lane<T: BitLane + std::ops::Not<Output = T>>(x: T) -> T {
    !x
}
// Source: src/core/SkVx.h:351-351
fn shl_lane<T: ShiftLane>(x: T, k: i32) -> T {
    x.shift_left(k)
}
// Source: src/core/SkVx.h:352-352
fn shr_lane<T: ShiftLane>(x: T, k: i32) -> T {
    x.shift_right(k)
}
// Source: src/core/SkVx.h:354-356
fn cmp_eq_lane<T: BitLane + ZeroLane + PartialOrd>(x: T, y: T) -> T::Mask {
    if x == y {
        !T::ZERO.to_mask()
    } else {
        T::ZERO.to_mask()
    }
}
// Source: src/core/SkVx.h:357-359
fn cmp_ne_lane<T: BitLane + ZeroLane + PartialOrd>(x: T, y: T) -> T::Mask {
    if x != y {
        !T::ZERO.to_mask()
    } else {
        T::ZERO.to_mask()
    }
}
// Source: src/core/SkVx.h:360-362
fn cmp_le_lane<T: BitLane + ZeroLane + PartialOrd>(x: T, y: T) -> T::Mask {
    if x <= y {
        !T::ZERO.to_mask()
    } else {
        T::ZERO.to_mask()
    }
}
// Source: src/core/SkVx.h:363-365
fn cmp_ge_lane<T: BitLane + ZeroLane + PartialOrd>(x: T, y: T) -> T::Mask {
    if x >= y {
        !T::ZERO.to_mask()
    } else {
        T::ZERO.to_mask()
    }
}
// Source: src/core/SkVx.h:366-368
fn cmp_lt_lane<T: BitLane + ZeroLane + PartialOrd>(x: T, y: T) -> T::Mask {
    if x < y {
        !T::ZERO.to_mask()
    } else {
        T::ZERO.to_mask()
    }
}
// Source: src/core/SkVx.h:369-371
fn cmp_gt_lane<T: BitLane + ZeroLane + PartialOrd>(x: T, y: T) -> T::Mask {
    if x > y {
        !T::ZERO.to_mask()
    } else {
        T::ZERO.to_mask()
    }
}
// Source: src/core/SkVx.h:374-376
pub fn add<const N: usize, T: LaneArithmetic>(x: &[T; N], y: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: LaneArithmetic>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = add_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:377-379
pub fn sub<const N: usize, T: LaneArithmetic>(x: &[T; N], y: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: LaneArithmetic>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = sub_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:380-382
pub fn mul<const N: usize, T: LaneArithmetic>(x: &[T; N], y: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: LaneArithmetic>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = mul_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:383-385
pub fn div<const N: usize, T: DivLane>(x: &[T; N], y: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: DivLane>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = div_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:387-389
pub fn bit_xor<const N: usize, T: Copy + std::ops::BitXor<Output = T>>(
    x: &[T; N],
    y: &[T; N],
) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: Copy + std::ops::BitXor<Output = T>>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = bit_xor_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:390-392
pub fn bit_and<const N: usize, T: Copy + std::ops::BitAnd<Output = T>>(
    x: &[T; N],
    y: &[T; N],
) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: Copy + std::ops::BitAnd<Output = T>>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = bit_and_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:393-395
pub fn bit_or<const N: usize, T: Copy + std::ops::BitOr<Output = T>>(
    x: &[T; N],
    y: &[T; N],
) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: Copy + std::ops::BitOr<Output = T>>(x: &[T], y: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = bit_or_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:397-397
pub fn logical_not<const N: usize, T: LogicalLane>(x: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: LogicalLane>(x: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = logical_not_lane(x[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], lo);
            recurse(&x[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, &mut out);
    out
}
// Source: src/core/SkVx.h:398-398
pub fn neg<const N: usize, T: NegLane>(x: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: NegLane>(x: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = neg_lane(x[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], lo);
            recurse(&x[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, &mut out);
    out
}
// Source: src/core/SkVx.h:399-399
pub fn bit_not<const N: usize, T: BitLane + std::ops::Not<Output = T>>(x: &[T; N]) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + std::ops::Not<Output = T>>(x: &[T], out: &mut [T]) {
        if x.len() == 1 {
            out[0] = bit_not_lane(x[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], lo);
            recurse(&x[mid..], hi);
        }
    }
    let mut out = *x;
    recurse(x, &mut out);
    out
}
// Source: src/core/SkVx.h:401-401
pub fn shl<const N: usize, T: ShiftLane>(x: &[T; N], k: i32) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: ShiftLane>(x: &[T], k: i32, out: &mut [T]) {
        if x.len() == 1 {
            out[0] = shl_lane(x[0], k);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], k, lo);
            recurse(&x[mid..], k, hi);
        }
    }
    let mut out = *x;
    recurse(x, k, &mut out);
    out
}
// Source: src/core/SkVx.h:402-402
pub fn shr<const N: usize, T: ShiftLane>(x: &[T; N], k: i32) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: ShiftLane>(x: &[T], k: i32, out: &mut [T]) {
        if x.len() == 1 {
            out[0] = shr_lane(x[0], k);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], k, lo);
            recurse(&x[mid..], k, hi);
        }
    }
    let mut out = *x;
    recurse(x, k, &mut out);
    out
}
// Source: src/core/SkVx.h:404-406
pub fn cmp_eq<const N: usize, T: BitLane + ZeroLane + PartialOrd>(
    x: &[T; N],
    y: &[T; N],
) -> [T::Mask; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + ZeroLane + PartialOrd>(x: &[T], y: &[T], out: &mut [T::Mask]) {
        if x.len() == 1 {
            out[0] = cmp_eq_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = [T::ZERO.to_mask(); N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:407-409
pub fn cmp_ne<const N: usize, T: BitLane + ZeroLane + PartialOrd>(
    x: &[T; N],
    y: &[T; N],
) -> [T::Mask; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + ZeroLane + PartialOrd>(x: &[T], y: &[T], out: &mut [T::Mask]) {
        if x.len() == 1 {
            out[0] = cmp_ne_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = [T::ZERO.to_mask(); N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:410-412
pub fn cmp_le<const N: usize, T: BitLane + ZeroLane + PartialOrd>(
    x: &[T; N],
    y: &[T; N],
) -> [T::Mask; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + ZeroLane + PartialOrd>(x: &[T], y: &[T], out: &mut [T::Mask]) {
        if x.len() == 1 {
            out[0] = cmp_le_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = [T::ZERO.to_mask(); N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:413-415
pub fn cmp_ge<const N: usize, T: BitLane + ZeroLane + PartialOrd>(
    x: &[T; N],
    y: &[T; N],
) -> [T::Mask; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + ZeroLane + PartialOrd>(x: &[T], y: &[T], out: &mut [T::Mask]) {
        if x.len() == 1 {
            out[0] = cmp_ge_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = [T::ZERO.to_mask(); N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:416-418
pub fn cmp_lt<const N: usize, T: BitLane + ZeroLane + PartialOrd>(
    x: &[T; N],
    y: &[T; N],
) -> [T::Mask; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + ZeroLane + PartialOrd>(x: &[T], y: &[T], out: &mut [T::Mask]) {
        if x.len() == 1 {
            out[0] = cmp_lt_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = [T::ZERO.to_mask(); N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:419-421
pub fn cmp_gt<const N: usize, T: BitLane + ZeroLane + PartialOrd>(
    x: &[T; N],
    y: &[T; N],
) -> [T::Mask; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<T: BitLane + ZeroLane + PartialOrd>(x: &[T], y: &[T], out: &mut [T::Mask]) {
        if x.len() == 1 {
            out[0] = cmp_gt_lane(x[0], y[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], &y[..mid], lo);
            recurse(&x[mid..], &y[mid..], hi);
        }
    }
    let mut out = [T::ZERO.to_mask(); N];
    recurse(x, y, &mut out);
    out
}
// Named functions map native array operators, which Rust cannot implement for foreign array types.
// Scalar arguments undergo the native lane conversion before splatting.
// Source: src/core/SkVx.h:425-425
pub fn add_scalar_left<const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T; N] {
    add(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:426-426
pub fn sub_scalar_left<const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T; N] {
    sub(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:427-427
pub fn mul_scalar_left<const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T; N] {
    mul(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:428-428
pub fn div_scalar_left<const N: usize, T: DivLane, U: CastLane<T>>(x: U, y: &[T; N]) -> [T; N] {
    div(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:429-429
pub fn bit_xor_scalar_left<
    const N: usize,
    T: Copy + std::ops::BitXor<Output = T>,
    U: CastLane<T>,
>(
    x: U,
    y: &[T; N],
) -> [T; N] {
    bit_xor(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:430-430
pub fn bit_and_scalar_left<
    const N: usize,
    T: Copy + std::ops::BitAnd<Output = T>,
    U: CastLane<T>,
>(
    x: U,
    y: &[T; N],
) -> [T; N] {
    bit_and(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:431-431
pub fn bit_or_scalar_left<const N: usize, T: Copy + std::ops::BitOr<Output = T>, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T; N] {
    bit_or(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:432-432
pub fn cmp_eq_scalar_left<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T::Mask; N] {
    cmp_eq(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:433-433
pub fn cmp_ne_scalar_left<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T::Mask; N] {
    cmp_ne(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:434-434
pub fn cmp_le_scalar_left<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T::Mask; N] {
    cmp_le(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:435-435
pub fn cmp_ge_scalar_left<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T::Mask; N] {
    cmp_ge(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:436-436
pub fn cmp_lt_scalar_left<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T::Mask; N] {
    cmp_lt(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:437-437
pub fn cmp_gt_scalar_left<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: U,
    y: &[T; N],
) -> [T::Mask; N] {
    cmp_gt(&splat::<N, T>(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:439-439
pub fn add_scalar_right<const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T; N] {
    add(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:440-440
pub fn sub_scalar_right<const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T; N] {
    sub(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:441-441
pub fn mul_scalar_right<const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T; N] {
    mul(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:442-442
pub fn div_scalar_right<const N: usize, T: DivLane, U: CastLane<T>>(x: &[T; N], y: U) -> [T; N] {
    div(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:443-443
pub fn bit_xor_scalar_right<
    const N: usize,
    T: Copy + std::ops::BitXor<Output = T>,
    U: CastLane<T>,
>(
    x: &[T; N],
    y: U,
) -> [T; N] {
    bit_xor(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:444-444
pub fn bit_and_scalar_right<
    const N: usize,
    T: Copy + std::ops::BitAnd<Output = T>,
    U: CastLane<T>,
>(
    x: &[T; N],
    y: U,
) -> [T; N] {
    bit_and(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:445-445
pub fn bit_or_scalar_right<
    const N: usize,
    T: Copy + std::ops::BitOr<Output = T>,
    U: CastLane<T>,
>(
    x: &[T; N],
    y: U,
) -> [T; N] {
    bit_or(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:446-446
pub fn cmp_eq_scalar_right<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T::Mask; N] {
    cmp_eq(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:447-447
pub fn cmp_ne_scalar_right<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T::Mask; N] {
    cmp_ne(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:448-448
pub fn cmp_le_scalar_right<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T::Mask; N] {
    cmp_le(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:449-449
pub fn cmp_ge_scalar_right<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T::Mask; N] {
    cmp_ge(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:450-450
pub fn cmp_lt_scalar_right<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T::Mask; N] {
    cmp_lt(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:451-451
pub fn cmp_gt_scalar_right<const N: usize, T: BitLane + ZeroLane + PartialOrd, U: CastLane<T>>(
    x: &[T; N],
    y: U,
) -> [T::Mask; N] {
    cmp_gt(x, &splat::<N, T>(y.cast_lane()))
}
// Source: src/core/SkVx.h:453-453
pub fn add_assign<'a, const N: usize, T: LaneArithmetic>(
    x: &'a mut [T; N],
    y: &[T; N],
) -> &'a mut [T; N] {
    *x = add(x, y);
    x
}
// Source: src/core/SkVx.h:454-454
pub fn sub_assign<'a, const N: usize, T: LaneArithmetic>(
    x: &'a mut [T; N],
    y: &[T; N],
) -> &'a mut [T; N] {
    *x = sub(x, y);
    x
}
// Source: src/core/SkVx.h:455-455
pub fn mul_assign<'a, const N: usize, T: LaneArithmetic>(
    x: &'a mut [T; N],
    y: &[T; N],
) -> &'a mut [T; N] {
    *x = mul(x, y);
    x
}
// Source: src/core/SkVx.h:456-456
pub fn div_assign<'a, const N: usize, T: DivLane>(x: &'a mut [T; N], y: &[T; N]) -> &'a mut [T; N] {
    *x = div(x, y);
    x
}
// Source: src/core/SkVx.h:457-457
pub fn bit_xor_assign<'a, const N: usize, T: Copy + std::ops::BitXor<Output = T>>(
    x: &'a mut [T; N],
    y: &[T; N],
) -> &'a mut [T; N] {
    *x = bit_xor(x, y);
    x
}
// Source: src/core/SkVx.h:458-458
pub fn bit_and_assign<'a, const N: usize, T: Copy + std::ops::BitAnd<Output = T>>(
    x: &'a mut [T; N],
    y: &[T; N],
) -> &'a mut [T; N] {
    *x = bit_and(x, y);
    x
}
// Source: src/core/SkVx.h:459-459
pub fn bit_or_assign<'a, const N: usize, T: Copy + std::ops::BitOr<Output = T>>(
    x: &'a mut [T; N],
    y: &[T; N],
) -> &'a mut [T; N] {
    *x = bit_or(x, y);
    x
}
// Source: src/core/SkVx.h:461-461
pub fn add_assign_scalar<'a, const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = add_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:462-462
pub fn sub_assign_scalar<'a, const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = sub_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:463-463
pub fn mul_assign_scalar<'a, const N: usize, T: LaneArithmetic, U: CastLane<T>>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = mul_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:464-464
pub fn div_assign_scalar<'a, const N: usize, T: DivLane, U: CastLane<T>>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = div_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:465-465
pub fn bit_xor_assign_scalar<
    'a,
    const N: usize,
    T: Copy + std::ops::BitXor<Output = T>,
    U: CastLane<T>,
>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = bit_xor_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:466-466
pub fn bit_and_assign_scalar<
    'a,
    const N: usize,
    T: Copy + std::ops::BitAnd<Output = T>,
    U: CastLane<T>,
>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = bit_and_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:467-467
pub fn bit_or_assign_scalar<
    'a,
    const N: usize,
    T: Copy + std::ops::BitOr<Output = T>,
    U: CastLane<T>,
>(
    x: &'a mut [T; N],
    y: U,
) -> &'a mut [T; N] {
    *x = bit_or_scalar_right(x, y);
    x
}
// Source: src/core/SkVx.h:469-469
pub fn shl_assign<const N: usize, T: ShiftLane>(x: &mut [T; N], bits: i32) -> &mut [T; N] {
    *x = shl(x, bits);
    x
}
// Source: src/core/SkVx.h:470-470
pub fn shr_assign<const N: usize, T: ShiftLane>(x: &mut [T; N], bits: i32) -> &mut [T; N] {
    *x = shr(x, bits);
    x
}
// Source: src/core/SkVx.h:477-480
pub fn naive_if_then_else<const N: usize, T: BitLane>(
    cond: [T::Mask; N],
    t: [T; N],
    e: [T; N],
) -> [T; N] {
    std::array::from_fn(|i| T::from_mask((cond[i] & t[i].to_mask()) | (!cond[i] & e[i].to_mask())))
}
// Source: src/core/SkVx.h:482-482
// Source: src/core/SkVx.h:484-487
pub fn if_then_else<const N: usize, T: BitLane>(
    cond: [T::Mask; N],
    t: [T; N],
    e: [T; N],
) -> [T; N] {
    assert!(N > 0 && N.is_power_of_two());
    if N == 1 {
        return std::array::from_fn(|i| {
            T::from_mask((cond[i] & t[i].to_mask()) | (!cond[i] & e[i].to_mask()))
        });
    }
    // Rust forwarding glue keeps this existing API on its portable branch.
    let mut out = e;
    if_then_else_portable_tail(&cond, &t, &e, &mut out);
    out
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:504-508
/// Safety: ARM64 NEON is enabled; T and its Mask are primitive numeric lanes
/// with fully initialized object bytes and exactly 16 bytes per input array.
pub unsafe fn if_then_else_neon128<const N: usize, T: BitLane>(
    cond: &[T::Mask; N],
    t: &[T; N],
    e: &[T; N],
) -> [T; N] {
    assert_eq!(std::mem::size_of_val(cond), 16);
    assert_eq!(std::mem::size_of_val(t), 16);
    let c = vld1q_u8(cond.as_ptr().cast());
    let tv = vld1q_u8(t.as_ptr().cast());
    let ev = vld1q_u8(e.as_ptr().cast());
    let selected = vbslq_u8(c, tv, ev);
    let mut out = std::mem::MaybeUninit::<[T; N]>::uninit();
    vst1q_u8(out.as_mut_ptr().cast(), selected);
    out.assume_init()
}
// Source: src/core/SkVx.h:525-528
// Source: src/core/SkVx.h:530-531
fn if_then_else_portable_tail<T: BitLane>(cond: &[T::Mask], t: &[T], e: &[T], out: &mut [T]) {
    if T::select_backend(cond, t, e, out) {
        return;
    }
    if t.len() * std::mem::size_of::<T>() > 16 {
        let mid = t.len() / 2;
        let (lo, hi) = out.split_at_mut(mid);
        if_then_else_portable_tail(&cond[..mid], &t[..mid], &e[..mid], lo);
        if_then_else_portable_tail(&cond[mid..], &t[mid..], &e[mid..], hi);
        return;
    }
    for i in 0..t.len() {
        out[i] = T::from_mask((cond[i] & t[i].to_mask()) | (!cond[i] & e[i].to_mask()));
    }
}
macro_rules! primitive_zero_backends {
    ($t:ty) => {
        fn any_backend(x: &[Self]) -> Option<bool> {
            #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
            if x.len() > 1 {
                if std::mem::size_of_val(x) == 8 {
                    return Some(unsafe {
                        any_neon64::<{ 8 / std::mem::size_of::<$t>() }, $t>(x.try_into().unwrap())
                    });
                }
                if std::mem::size_of_val(x) == 16 {
                    return Some(unsafe {
                        any_neon128::<{ 16 / std::mem::size_of::<$t>() }, $t>(x.try_into().unwrap())
                    });
                }
            }
            None
        }
        fn all_backend(x: &[Self]) -> Option<bool> {
            #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
            if x.len() > 1 && std::mem::size_of::<$t>() <= 4 {
                if std::mem::size_of_val(x) == 8 {
                    return Some(unsafe {
                        all_neon::<{ 8 / std::mem::size_of::<$t>() }, $t>(x.try_into().unwrap())
                    });
                }
                if std::mem::size_of_val(x) == 16 {
                    return Some(unsafe {
                        all_neon::<{ 16 / std::mem::size_of::<$t>() }, $t>(x.try_into().unwrap())
                    });
                }
            }
            None
        }
    };
}
pub trait ZeroLane: Copy + PartialEq {
    const ZERO: Self;
    fn any_backend(_x: &[Self]) -> Option<bool> {
        None
    }
    fn all_backend(_x: &[Self]) -> Option<bool> {
        None
    }
}
macro_rules! zero_lane {($($t:ty),*)=>{$(impl ZeroLane for $t{const ZERO:Self=0 as $t;primitive_zero_backends!($t);})*};}
zero_lane!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
impl ZeroLane for bool {
    const ZERO: Self = false;
}
// Source: src/core/SkVx.h:533-534
pub fn any<const N: usize, T: ZeroLane>(x: &[T; N]) -> bool {
    assert!(N > 0 && N.is_power_of_two());
    if N == 1 {
        x[0] != T::ZERO
    } else {
        any_portable_tail(x)
    }
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:559-559
/// Safety: ARM64 NEON is enabled and the primitive numeric input has eight
/// fully initialized object bytes, with N > 1 (native scalar overload excluded).
/// This tests bits, including float -0 bits.
pub unsafe fn any_neon64<const N: usize, T: BitLane>(x: &[T; N]) -> bool {
    assert!(N > 1);
    assert_eq!(std::mem::size_of_val(x), 8);
    vmaxv_u8(vld1_u8(x.as_ptr().cast())) > 0
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:560-560
/// Safety: same as any_neon64, with sixteen initialized input bytes.
pub unsafe fn any_neon128<const N: usize, T: BitLane>(x: &[T; N]) -> bool {
    assert!(N > 1);
    assert_eq!(std::mem::size_of_val(x), 16);
    vmaxvq_u8(vld1q_u8(x.as_ptr().cast())) > 0
}
// Source: src/core/SkVx.h:581-583
fn any_portable_tail<T: ZeroLane>(x: &[T]) -> bool {
    if x.len() == 1 {
        return x[0] != T::ZERO;
    }
    if let Some(result) = T::any_backend(x) {
        return result;
    }
    let mid = x.len() / 2;
    any_portable_tail(&x[..mid]) || any_portable_tail(&x[mid..])
}
// Source: src/core/SkVx.h:585-586
pub fn all<const N: usize, T: ZeroLane>(x: &[T; N]) -> bool {
    assert!(N > 0 && N.is_power_of_two());
    if N == 1 {
        x[0] != T::ZERO
    } else {
        all_portable_tail(x)
    }
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:598-603
/// Safety: ARM64 NEON is enabled; T is a primitive numeric lane of size
/// 1, 2, or 4, in exactly one of the six source-specialized array shapes.
pub unsafe fn all_neon<const N: usize, T: BitLane>(x: &[T; N]) -> bool {
    let p = x.as_ptr();
    let size = std::mem::size_of::<T>();
    if size == 1 && N == 8 {
        return vminv_u8(vld1_u8(p.cast())) > 0;
    }
    if size == 1 && N == 16 {
        return vminvq_u8(vld1q_u8(p.cast())) > 0;
    }
    if size == 2 && N == 4 {
        return vminv_u16(vld1_u16(p.cast())) > 0;
    }
    if size == 2 && N == 8 {
        return vminvq_u16(vld1q_u16(p.cast())) > 0;
    }
    if size == 4 && N == 2 {
        return vminv_u32(vld1_u32(p.cast())) > 0;
    }
    if size == 4 && N == 4 {
        return vminvq_u32(vld1q_u32(p.cast())) > 0;
    }
    panic!("input is outside the native ARM64 all specialization shapes")
}
// Source: src/core/SkVx.h:624-626
fn all_portable_tail<T: ZeroLane>(x: &[T]) -> bool {
    if x.len() == 1 {
        return x[0] != T::ZERO;
    }
    if let Some(result) = T::all_backend(x) {
        return result;
    }
    let mid = x.len() / 2;
    all_portable_tail(&x[..mid]) && all_portable_tail(&x[mid..])
}
// Primitive C-cast substitution. Float-to-integer inputs must be finite and within
// the native destination conversion range; behavior outside native defined inputs is not claimed.
pub trait CastLane<D>: Copy {
    fn cast_lane(self) -> D;
}
macro_rules! cast_lane_from {($s:ty;$($d:ty),*)=>{$(impl CastLane<$d> for $s {fn cast_lane(self)->$d{self as $d}})*};}
cast_lane_from!(u8;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(i8;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(u16;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(i16;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(u32;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(i32;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(u64;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(i64;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(usize;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(isize;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(f32;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
cast_lane_from!(f64;u8,i8,u16,i16,u32,i32,u64,i64,usize,isize,f32,f64);
// Source: src/core/SkVx.h:630-631
fn cast_lane<D, S: CastLane<D>>(src: S) -> D {
    src.cast_lane()
}
// Source: src/core/SkVx.h:633-634
// Source: src/core/SkVx.h:638-638
// Source: src/core/SkVx.h:640-640
pub fn cast<const N: usize, D: Copy, S: CastLane<D>>(src: &[S; N]) -> [D; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse<D: Copy, S: CastLane<D>>(src: &[S], out: &mut [std::mem::MaybeUninit<D>]) {
        if src.len() == 1 {
            out[0].write(cast_lane(src[0]));
        } else {
            let mid = src.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&src[..mid], lo);
            recurse(&src[mid..], hi);
        }
    }
    let mut out = [std::mem::MaybeUninit::<D>::uninit(); N];
    recurse(src, &mut out);
    // Every lane is initialized by the scalar base case, with no reference into the temporary.
    out.map(|lane| unsafe { lane.assume_init() })
}
// Source: src/core/SkVx.h:643-646
pub fn reduce_min<const N: usize, T: Copy + PartialOrd>(x: &[T; N]) -> T {
    assert!(N > 0 && N.is_power_of_two());
    fn reduce<T: Copy + PartialOrd>(x: &[T]) -> T {
        if x.len() == 1 {
            x[0]
        } else {
            let mid = x.len() / 2;
            let lo = reduce(&x[..mid]);
            let hi = reduce(&x[mid..]);
            if hi < lo {
                hi
            } else {
                lo
            }
        }
    }
    reduce(x)
}
pub fn reduce_max<const N: usize, T: Copy + PartialOrd>(x: &[T; N]) -> T {
    assert!(N > 0 && N.is_power_of_two());
    fn reduce<T: Copy + PartialOrd>(x: &[T]) -> T {
        if x.len() == 1 {
            x[0]
        } else {
            let mid = x.len() / 2;
            let lo = reduce(&x[..mid]);
            let hi = reduce(&x[mid..]);
            if lo < hi {
                hi
            } else {
                lo
            }
        }
    }
    reduce(x)
}
// Source: src/core/SkVx.h:648-649
pub fn min<const N: usize, T: Copy + PartialOrd>(x: [T; N], y: [T; N]) -> [T; N] {
    std::array::from_fn(|i| if y[i] < x[i] { y[i] } else { x[i] })
}
pub fn max<const N: usize, T: Copy + PartialOrd>(x: [T; N], y: [T; N]) -> [T; N] {
    std::array::from_fn(|i| if x[i] < y[i] { y[i] } else { x[i] })
}
// Source: src/core/SkVx.h:651-651
pub fn min_scalar_right<const N: usize, T: Copy + PartialOrd, U: CastLane<T>>(
    x: [T; N],
    y: U,
) -> [T; N] {
    min(x, splat(y.cast_lane()))
}
// Source: src/core/SkVx.h:652-652
pub fn max_scalar_right<const N: usize, T: Copy + PartialOrd, U: CastLane<T>>(
    x: [T; N],
    y: U,
) -> [T; N] {
    max(x, splat(y.cast_lane()))
}
// Source: src/core/SkVx.h:653-653
pub fn min_scalar_left<const N: usize, T: Copy + PartialOrd, U: CastLane<T>>(
    x: U,
    y: [T; N],
) -> [T; N] {
    min(splat(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:654-654
pub fn max_scalar_left<const N: usize, T: Copy + PartialOrd, U: CastLane<T>>(
    x: U,
    y: [T; N],
) -> [T; N] {
    max(splat(x.cast_lane()), y)
}
// Source: src/core/SkVx.h:658-660
pub fn pin<const N: usize, T: Copy + PartialOrd>(x: [T; N], lo: [T; N], hi: [T; N]) -> [T; N] {
    max(lo, min(x, hi))
}
// Source: src/core/SkVx.h:669-670

// Source: src/core/SkVx.h:675-675
// Source: src/core/SkVx.h:677-677
// The index array substitutes C++'s integer template parameter pack; no SIMD shuffle substitution.
pub fn shuffle<const N: usize, const M: usize, T: Copy>(x: [T; N], indices: [usize; M]) -> [T; M] {
    std::array::from_fn(|i| x[indices[i]])
}
// Rust has no variadic type parameters. A recursive tuple carries the native
// argument pack without an arity limit. Native const operator[] returns T
// by value, so callbacks receive copied lane values rather than source references.
pub trait MapArgs<const N: usize> {
    type Lanes;
    fn lanes_at(&self, i: usize) -> Self::Lanes;
}
impl<const N: usize> MapArgs<N> for () {
    type Lanes = ();
    fn lanes_at(&self, _i: usize) {}
}
impl<'a, const N: usize, T: Copy, R: MapArgs<N>> MapArgs<N> for (&'a [T; N], R) {
    type Lanes = (T, R::Lanes);
    fn lanes_at(&self, i: usize) -> Self::Lanes {
        (self.0[i], self.1.lanes_at(i))
    }
}
// Source: src/core/SkVx.h:682-685
// Source: src/core/SkVx.h:691-692
// Source: src/core/SkVx.h:694-695
// Index-array parameter substitutes the native compile-time index sequence.
// A mutable borrow of fn preserves its identity and ordered side effects.
pub fn map_indices<const N: usize, const M: usize, A: MapArgs<N>, F, O>(
    indices: [usize; M],
    fn_: &mut F,
    args: &A,
) -> [O; M]
where
    F: FnMut(A::Lanes) -> O,
{
    assert!(M > 0 && M.is_power_of_two());
    let mut lane = |i| fn_(args.lanes_at(i));
    std::array::from_fn(|i| lane(indices[i]))
}
// Source: src/core/SkVx.h:697-698
// Source: src/core/SkVx.h:700-701
pub fn map<const N: usize, T: Copy, R: MapArgs<N>, F, O>(
    fn_: &mut F,
    first: &[T; N],
    rest: R,
) -> [O; N]
where
    F: FnMut((T, R::Lanes)) -> O,
{
    let indices = std::array::from_fn(|i| i);
    map_indices(indices, fn_, &(first, rest))
}
// Source: src/core/SkVx.h:703-703
pub fn ceil<const N: usize>(x: [f32; N]) -> [f32; N] {
    map(&mut |(v, ()): (f32, ())| v.ceil(), &x, ())
}
// Source: src/core/SkVx.h:704-704
pub fn floor<const N: usize>(x: [f32; N]) -> [f32; N] {
    map(&mut |(v, ()): (f32, ())| v.floor(), &x, ())
}
// Source: src/core/SkVx.h:705-705
pub fn trunc<const N: usize>(x: [f32; N]) -> [f32; N] {
    map(&mut |(v, ()): (f32, ())| v.trunc(), &x, ())
}
// Source: src/core/SkVx.h:706-706
pub fn round<const N: usize>(x: [f32; N]) -> [f32; N] {
    map(&mut |(v, ()): (f32, ())| v.round(), &x, ())
}
// Source: src/core/SkVx.h:707-707
pub fn sqrt<const N: usize>(x: [f32; N]) -> [f32; N] {
    map(&mut |(v, ()): (f32, ())| v.sqrt(), &x, ())
}
// Source: src/core/SkVx.h:708-708
pub fn abs<const N: usize>(x: [f32; N]) -> [f32; N] {
    map(&mut |(v, ()): (f32, ())| v.abs(), &x, ())
}
// Source: src/core/SkVx.h:709-711
// Source: src/core/SkVx.h:713-715
pub fn fma<const N: usize>(x: [f32; N], y: [f32; N], z: [f32; N]) -> [f32; N] {
    let fn_ = |x: f32, y: f32, z: f32| x.mul_add(y, z);
    map(
        &mut |(a, (b, (c, ()))): (f32, (f32, (f32, ())))| fn_(a, b, c),
        &x,
        (&y, (&z, ())),
    )
}
// The native scalar branch calls libc lrintf, including its active rounding-mode
// and floating-point exception semantics. Keep that call rather than replacing it with round().
unsafe extern "C" {
    fn lrintf(value: f32) -> std::ffi::c_long;
}
// Source: src/core/SkVx.h:717-719
fn lrint_lane(x: f32) -> i32 {
    unsafe { lrintf(x) as i32 }
}
// Source: src/core/SkVx.h:720-720
// Source: src/core/SkVx.h:741-743
pub fn lrint<const N: usize>(x: &[f32; N]) -> [i32; N] {
    assert!(N > 0 && N.is_power_of_two());
    fn recurse(x: &[f32], out: &mut [i32]) {
        if x.len() == 1 {
            out[0] = lrint_lane(x[0]);
        } else {
            let mid = x.len() / 2;
            let (lo, hi) = out.split_at_mut(mid);
            recurse(&x[..mid], lo);
            recurse(&x[mid..], hi);
        }
    }
    let mut out = [0; N];
    recurse(x, &mut out);
    out
}
// Source: src/core/SkVx.h:745-745
pub fn fract<const N: usize>(x: [f32; N]) -> [f32; N] {
    let f = floor(x);
    std::array::from_fn(|i| x[i] - f[i])
}
// Source: src/core/SkVx.h:751-752
// Source: src/core/SkVx.h:757-760
pub fn to_half<const N: usize>(x: [f32; N]) -> [u16; N] {
    assert!(N > 0 && N.is_power_of_two());
    let mut out = [0; N];
    to_half_slice(&x, &mut out);
    out
}
fn to_half_slice(x: &[f32], out: &mut [u16]) {
    debug_assert!(x.iter().all(|&v| v == v));
    if x.len() > 4 {
        let mid = x.len() / 2;
        let (lo, hi) = out.split_at_mut(mid);
        to_half_slice(&x[..mid], lo);
        to_half_slice(&x[mid..], hi);
        return;
    }
    #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
    if x.len() == 4 {
        out.copy_from_slice(&unsafe { to_half4_neon(x.try_into().unwrap()) });
        return;
    }
    to_half_portable_slice(x, out);
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:763-764
// Source: src/core/SkVx.h:766-766
/// Safety: ARM64 NEON half conversion is supported; input has no NaNs,
/// matching the enclosing native to_half precondition.
pub unsafe fn to_half4_neon(x: &[f32; 4]) -> [u16; 4] {
    debug_assert!(x.iter().all(|x| !x.is_nan()));
    let converted = vreinterpret_u16_f16(vcvt_f16_f32(vld1q_f32(x.as_ptr())));
    let mut out = [0u16; 4];
    vst1_u16(out.as_mut_ptr(), converted);
    out
}
fn to_half_portable_slice(x: &[f32], out: &mut [u16]) {
    // Source: src/core/SkVx.h:769-773
    // Source: src/core/SkVx.h:777-778
    // Source: src/core/SkVx.h:781-786
    // Each vector operation is evaluated in source order, using the same signed integer bit patterns.
    let mut sem = [0i32; 4];
    let mut sign = [0i32; 4];
    let mut em = [0i32; 4];
    let mut magic = [0i32; 4];
    let mut rounded = [0i32; 4];
    let mut exp = [0i32; 4];
    for i in 0..x.len() {
        sem[i] = x[i].to_bits() as i32;
    }
    for i in 0..x.len() {
        sign[i] = sem[i] & i32::MIN;
    }
    for i in 0..x.len() {
        em[i] = (sem[i] ^ sign[i]).min(0x47800000);
    }
    for i in 0..x.len() {
        let scaled = f32::from_bits(em[i] as u32) * 8192.0;
        let bounded = if scaled < 0.5 { 0.5 } else { scaled };
        magic[i] = bounded.to_bits() as i32 & (255 << 23);
    }
    for i in 0..x.len() {
        rounded[i] =
            (f32::from_bits(em[i] as u32) + f32::from_bits(magic[i] as u32)).to_bits() as i32;
    }
    for i in 0..x.len() {
        exp[i] = (magic[i] >> 13) - ((127 - 15 + 13 + 1) << 10);
    }
    for i in 0..x.len() {
        let f16 = rounded[i].wrapping_add(exp[i]);
        out[i] = ((sign[i] >> 16) | f16) as u16;
    }
}
// Source: src/core/SkVx.h:791-795
pub fn from_half<const N: usize>(x: [u16; N]) -> [f32; N] {
    assert!(N > 0 && N.is_power_of_two());
    let mut out = [0.0; N];
    from_half_slice(&x, &mut out);
    out
}
fn from_half_slice(x: &[u16], out: &mut [f32]) {
    if x.len() > 4 {
        let mid = x.len() / 2;
        let (lo, hi) = out.split_at_mut(mid);
        from_half_slice(&x[..mid], lo);
        from_half_slice(&x[mid..], hi);
        return;
    }
    #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
    if x.len() == 4 {
        out.copy_from_slice(&unsafe { from_half4_neon(x.try_into().unwrap()) });
        return;
    }
    from_half_portable_slice(x, out);
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:798-800
/// Safety: ARM64 NEON half conversion is supported. Native hardware NaN
/// expansion/quieting is retained; this is not the portable half algorithm.
pub unsafe fn from_half4_neon(x: &[u16; 4]) -> [f32; 4] {
    let converted = vcvt_f32_f16(vreinterpret_f16_u16(vld1_u16(x.as_ptr())));
    let mut out = [0f32; 4];
    vst1q_f32(out.as_mut_ptr(), converted);
    out
}
fn from_half_portable_slice(x: &[u16], out: &mut [f32]) {
    // Source: src/core/SkVx.h:803-807
    // Source: src/core/SkVx.h:809-811
    // Source: src/core/SkVx.h:815-816
    let mut wide = [0i32; 4];
    let mut sign = [0i32; 4];
    let mut em = [0i32; 4];
    let mut inf_or_nan = [0i32; 4];
    let mut is_norm = [0i32; 4];
    let mut sub = [0i32; 4];
    let mut norm = [0i32; 4];
    let mut finite = [0i32; 4];
    for i in 0..x.len() {
        wide[i] = x[i] as i32;
    }
    for i in 0..x.len() {
        sign[i] = wide[i] & 0x8000;
    }
    for i in 0..x.len() {
        em[i] = wide[i] ^ sign[i];
    }
    for i in 0..x.len() {
        inf_or_nan[i] = (if em[i] >= (31 << 10) { -1 } else { 0 }) & (255 << 23);
    }
    for i in 0..x.len() {
        is_norm[i] = if em[i] > 0x3ff { -1 } else { 0 };
    }
    for i in 0..x.len() {
        sub[i] = (em[i] as f32 * (1.0 / (1 << 24) as f32)).to_bits() as i32;
    }
    for i in 0..x.len() {
        norm[i] = (em[i] << 13).wrapping_add((127 - 15) << 23);
    }
    for i in 0..x.len() {
        finite[i] = (is_norm[i] & norm[i]) | (!is_norm[i] & sub[i]);
    }
    for i in 0..x.len() {
        out[i] = f32::from_bits(((sign[i] << 16) | finite[i] | inf_or_nan[i]) as u32);
    }
}
// Source: src/core/SkVx.h:819-821
pub fn div255<const N: usize>(x: &[u16; N]) -> [u8; N] {
    cast(&div(&add(x, &splat::<N, u16>(127)), &splat::<N, u16>(255)))
}
// Source: src/core/SkVx.h:825-825
// Source: src/core/SkVx.h:828-831
pub fn approx_scale<const N: usize>(x: [u8; N], y: [u8; N]) -> [u8; N] {
    let X: [u16; N] = std::array::from_fn(|i| x[i] as u16);
    let Y: [u16; N] = std::array::from_fn(|i| y[i] as u16);
    std::array::from_fn(|i| ((X[i] * Y[i] + X[i]) / 256) as u8)
}
pub trait UnsignedLane: LaneArithmetic + PartialOrd {
    const MAX: Self;
}
macro_rules! unsigned_lane {($($t:ty),*)=>{$(impl UnsignedLane for $t{const MAX:Self=<$t>::MAX;})*};}
unsigned_lane!(u8, u16, u32, u64, usize);
// Source: src/core/SkVx.h:834-835
pub fn saturated_add<const N: usize, T: UnsignedLane>(x: [T; N], y: [T; N]) -> [T; N] {
    saturated_add_portable_tail(x, y)
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:840-840
// Source: src/core/SkVx.h:848-849
// Source: src/core/SkVx.h:851-855
/// Safety: ARM64 NEON is enabled. Stable Rust slices carry the native
/// recursive low/high views; small vectors are joined with themselves.
pub unsafe fn saturated_add_u8_neon<const N: usize>(x: &[u8; N], y: &[u8; N]) -> [u8; N] {
    assert!(N > 0 && N.is_power_of_two());
    unsafe fn recurse(x: &[u8], y: &[u8], out: &mut [u8]) {
        if x.len() == 16 {
            vst1q_u8(
                out.as_mut_ptr(),
                vqaddq_u8(vld1q_u8(x.as_ptr()), vld1q_u8(y.as_ptr())),
            );
        } else if x.len() < 16 {
            let xx = std::array::from_fn::<_, 16, _>(|i| x[i % x.len()]);
            let yy = std::array::from_fn::<_, 16, _>(|i| y[i % y.len()]);
            let n = 2 * x.len();
            let mut doubled = [0 as u8; 16];
            recurse(&xx[..n], &yy[..n], &mut doubled[..n]);
            out.copy_from_slice(&doubled[..x.len()]);
        } else {
            let half = x.len() / 2;
            let (lo, hi) = out.split_at_mut(half);
            recurse(&x[..half], &y[..half], lo);
            recurse(&x[half..], &y[half..], hi);
        }
    }
    let mut out = [0 as u8; N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:858-860
fn saturated_add_portable_tail<const N: usize, T: UnsignedLane>(x: [T; N], y: [T; N]) -> [T; N] {
    let sum: [T; N] = std::array::from_fn(|i| x[i].lane_add(y[i]));
    std::array::from_fn(|i| if sum[i] < x[i] { T::MAX } else { sum[i] })
}
// Source: src/core/SkVx.h:880-886
impl ScaledDividerU32 {
    pub fn new(divisor: u32) -> Self {
        let fDivisorFactor = ((1.0 / divisor as f64) * (1u64 << 32) as f64).round() as u32;
        let fHalf = divisor.wrapping_add(1) >> 1;
        debug_assert!(divisor > 1);
        Self {
            fDivisorFactor,
            fHalf,
        }
    }
}
// Source: src/core/SkVx.h:888-888
impl ScaledDividerU32 {
    pub fn divide4(&self, numerator: [u32; 4]) -> [u32; 4] {
        #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
        {
            return unsafe { scaled_divide4_neon(self, &numerator) };
        }
        #[cfg(not(all(target_arch = "aarch64", not(skvx_disable_simd))))]
        scaled_divide4_portable(self, numerator)
    }
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:890-891
// Source: src/core/SkVx.h:893-893
/// Safety: ARM64 NEON is enabled.
pub unsafe fn scaled_divide4_neon(d: &ScaledDividerU32, numerator: &[u32; 4]) -> [u32; 4] {
    let n = vld1q_u32(numerator.as_ptr());
    let hi = vmull_n_u32(vget_high_u32(n), d.fDivisorFactor);
    let lo = vmull_n_u32(vget_low_u32(n), d.fDivisorFactor);
    let result = vcombine_u32(vshrn_n_u64::<32>(lo), vshrn_n_u64::<32>(hi));
    let mut out = [0u32; 4];
    vst1q_u32(out.as_mut_ptr(), result);
    out
}
// Source: src/core/SkVx.h:895-895
// Source: src/core/SkVx.h:897-897
fn scaled_divide4_portable(d: &ScaledDividerU32, numerator: [u32; 4]) -> [u32; 4] {
    numerator.map(|v| ((v as u64 * d.fDivisorFactor as u64) >> 32) as u32)
}
// Source: src/core/SkVx.h:899-900
impl ScaledDividerU32 {
    pub fn half(&self) -> u32 {
        self.fHalf
    }
    pub fn divisorFactor(&self) -> u32 {
        self.fDivisorFactor
    }
}
// Source: src/core/SkVx.h:903-905
pub struct ScaledDividerU32 {
    fDivisorFactor: u32,
    fHalf: u32,
}
pub trait WidenLane: Copy {
    type Wide: Copy + std::ops::Mul<Output = Self::Wide>;
    fn widen(self) -> Self::Wide;
}
impl WidenLane for u8 {
    type Wide = u16;
    fn widen(self) -> u16 {
        self as u16
    }
}
impl WidenLane for u16 {
    type Wide = u32;
    fn widen(self) -> u32 {
        self as u32
    }
}
// Source: src/core/SkVx.h:908-909
pub fn mull<const N: usize, T: WidenLane>(x: [T; N], y: [T; N]) -> [T::Wide; N] {
    mull_portable_tail(x, y)
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:912-918
/// Safety: ARM64 NEON is enabled. Stable Rust slices carry the native
/// recursive low/high views; small vectors are joined with themselves.
pub unsafe fn mull_u8_neon<const N: usize>(x: &[u8; N], y: &[u8; N]) -> [u16; N] {
    assert!(N > 0 && N.is_power_of_two());
    unsafe fn recurse(x: &[u8], y: &[u8], out: &mut [u16]) {
        if x.len() == 8 {
            vst1q_u16(
                out.as_mut_ptr(),
                vmull_u8(vld1_u8(x.as_ptr()), vld1_u8(y.as_ptr())),
            );
        } else if x.len() < 8 {
            let xx = std::array::from_fn::<_, 8, _>(|i| x[i % x.len()]);
            let yy = std::array::from_fn::<_, 8, _>(|i| y[i % y.len()]);
            let n = 2 * x.len();
            let mut doubled = [0 as u16; 8];
            recurse(&xx[..n], &yy[..n], &mut doubled[..n]);
            out.copy_from_slice(&doubled[..x.len()]);
        } else {
            let half = x.len() / 2;
            let (lo, hi) = out.split_at_mut(half);
            recurse(&x[..half], &y[..half], lo);
            recurse(&x[half..], &y[half..], hi);
        }
    }
    let mut out = [0 as u16; N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:920-920
// Source: src/core/SkVx.h:922-922
fn mull_portable_tail<const N: usize, T: WidenLane>(x: [T; N], y: [T; N]) -> [T::Wide; N] {
    let wx: [T::Wide; N] = std::array::from_fn(|i| x[i].widen());
    let wy: [T::Wide; N] = std::array::from_fn(|i| y[i].widen());
    std::array::from_fn(|i| wx[i] * wy[i])
}
// Source: src/core/SkVx.h:924-925
pub fn mull_u16<const N: usize>(x: [u16; N], y: [u16; N]) -> [u32; N] {
    mull_u16_portable_tail(x, y)
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:928-934
/// Safety: ARM64 NEON is enabled. Stable Rust slices carry the native
/// recursive low/high views; small vectors are joined with themselves.
pub unsafe fn mull_u16_neon<const N: usize>(x: &[u16; N], y: &[u16; N]) -> [u32; N] {
    assert!(N > 0 && N.is_power_of_two());
    unsafe fn recurse(x: &[u16], y: &[u16], out: &mut [u32]) {
        if x.len() == 4 {
            vst1q_u32(
                out.as_mut_ptr(),
                vmull_u16(vld1_u16(x.as_ptr()), vld1_u16(y.as_ptr())),
            );
        } else if x.len() < 4 {
            let xx = std::array::from_fn::<_, 4, _>(|i| x[i % x.len()]);
            let yy = std::array::from_fn::<_, 4, _>(|i| y[i % y.len()]);
            let n = 2 * x.len();
            let mut doubled = [0 as u32; 4];
            recurse(&xx[..n], &yy[..n], &mut doubled[..n]);
            out.copy_from_slice(&doubled[..x.len()]);
        } else {
            let half = x.len() / 2;
            let (lo, hi) = out.split_at_mut(half);
            recurse(&x[..half], &y[..half], lo);
            recurse(&x[half..], &y[half..], hi);
        }
    }
    let mut out = [0 as u32; N];
    recurse(x, y, &mut out);
    out
}
// Source: src/core/SkVx.h:936-936
// Source: src/core/SkVx.h:938-938
fn mull_u16_portable_tail<const N: usize>(x: [u16; N], y: [u16; N]) -> [u32; N] {
    mull_portable_tail(x, y)
}
// Source: src/core/SkVx.h:940-941
// Source: src/core/SkVx.h:962-962
// Source: src/core/SkVx.h:964-964
pub fn mulhi<const N: usize>(x: [u16; N], y: [u16; N]) -> [u16; N] {
    let product = mull(x, y);
    std::array::from_fn(|i| (product[i] >> 16) as u16)
}
// Primitive operator mapping for native numeric Vec lanes. Integer operators wrap at
// their native width (including unsigned overflow); signed overflow has no native defined result.
// Native u16 multiplication promotes to int32: parity excludes products above i32::MAX,
// where the C++ expression has signed overflow. Wide signed arithmetic likewise requires
// the native defined-input domain; unsigned wide operations preserve modular arithmetic.
pub trait LaneArithmetic: Copy {
    fn lane_add(self, rhs: Self) -> Self;
    fn lane_mul(self, rhs: Self) -> Self;
    fn lane_sub(self, rhs: Self) -> Self;
    fn reduce_add_backend(_x: &[Self]) -> Option<Self> {
        None
    }
}
macro_rules! integer_lane {($($t:ty),*)=>{$(impl LaneArithmetic for $t {
    fn lane_add(self,rhs:Self)->Self{self.wrapping_add(rhs)}
    fn lane_mul(self,rhs:Self)->Self{self.wrapping_mul(rhs)}
    fn lane_sub(self,rhs:Self)->Self{self.wrapping_sub(rhs)}
})*};}
integer_lane!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
macro_rules! float_lane {($($t:ty),*)=>{$(impl LaneArithmetic for $t {
    fn lane_add(self,rhs:Self)->Self{self+rhs}
    fn lane_mul(self,rhs:Self)->Self{self*rhs}
    fn lane_sub(self,rhs:Self)->Self{self-rhs}
})*};}
impl LaneArithmetic for f32 {
    fn lane_add(self, rhs: Self) -> Self {
        self + rhs
    }
    fn lane_mul(self, rhs: Self) -> Self {
        self * rhs
    }
    fn lane_sub(self, rhs: Self) -> Self {
        self - rhs
    }
    fn reduce_add_backend(x: &[Self]) -> Option<Self> {
        #[cfg(all(target_arch = "aarch64", not(skvx_disable_simd)))]
        if x.len() == 4 {
            return Some(unsafe { reduce_add4_neon(x.try_into().unwrap()) });
        }
        None
    }
}
float_lane!(f64);
// Source: src/core/SkVx.h:966-966
// Source: src/core/SkVx.h:970-982
pub fn dot<const N: usize, T: LaneArithmetic>(a: &[T; N], b: &[T; N]) -> T {
    assert!(N > 0 && N.is_power_of_two());
    let ab: [T; N] = std::array::from_fn(|i| a[i].lane_mul(b[i]));
    if N == 2 {
        ab[0].lane_add(ab[1])
    } else if N == 4 {
        ab[0].lane_add(ab[1]).lane_add(ab[2]).lane_add(ab[3])
    } else {
        let mut sum = ab[0];
        for i in 1..N {
            sum = sum.lane_add(ab[i]);
        }
        sum
    }
}
// Source: src/core/SkVx.h:1008-1009
pub fn reduce_add<const N: usize, T: LaneArithmetic>(x: &[T; N]) -> T {
    assert!(N > 0 && N.is_power_of_two());
    if N == 1 {
        x[0]
    } else {
        let mut scratch = *x;
        reduce_add_portable_tail(&mut scratch)
    }
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1011-1013
/// Safety: ARM64 NEON is enabled. vaddvq's native floating reduction order
/// is preserved, including cancellation and signed-zero behavior.
pub unsafe fn reduce_add4_neon(x: &[f32; 4]) -> f32 {
    vaddvq_f32(vld1q_f32(x.as_ptr()))
}
// Source: src/core/SkVx.h:1045-1046
// Fixed stack lane slices substitute stable Rust's unavailable N/2 array type.
fn reduce_add_portable_tail<T: LaneArithmetic>(x: &mut [T]) -> T {
    if x.len() == 1 {
        return x[0];
    }
    if let Some(result) = T::reduce_add_backend(x) {
        return result;
    }
    let half = x.len() / 2;
    for i in 0..half {
        x[i] = x[i].lane_add(x[half + i]);
    }
    reduce_add_portable_tail(&mut x[..half])
}
// Source: src/core/SkVx.h:1048-1051
pub fn cross<T: LaneArithmetic>(a: &[T; 2], b: &[T; 2]) -> T {
    let bs = shuffle(*b, [1, 0]);
    let x = [a[0].lane_mul(bs[0]), a[1].lane_mul(bs[1])];
    x[0].lane_sub(x[1])
}
// The two native floating overloads share this primitive sqrt/division mapping.
pub trait FloatLane: LaneArithmetic {
    fn lane_sqrt(self) -> Self;
    fn lane_div(self, rhs: Self) -> Self;
}
impl FloatLane for f32 {
    fn lane_sqrt(self) -> Self {
        self.sqrt()
    }
    fn lane_div(self, rhs: Self) -> Self {
        self / rhs
    }
}
impl FloatLane for f64 {
    fn lane_sqrt(self) -> Self {
        self.sqrt()
    }
    fn lane_div(self, rhs: Self) -> Self {
        self / rhs
    }
}
// Source: src/core/SkVx.h:1053-1055
// Source: src/core/SkVx.h:1057-1059
pub fn length<const N: usize, T: FloatLane>(v: &[T; N]) -> T {
    dot(v, v).lane_sqrt()
}
// Source: src/core/SkVx.h:1061-1063
// Source: src/core/SkVx.h:1065-1067
pub fn normalize<const N: usize, T: FloatLane>(v: &[T; N]) -> [T; N] {
    let len = length(v);
    std::array::from_fn(|i| v[i].lane_div(len))
}
// Native type-directed exponent-mask mapping; integer lanes have no nonfinite encoding.
pub trait FiniteLane: Copy {
    const IS_FLOATING_POINT: bool = false;
    fn exponent_is_finite(self) -> bool;
}
impl FiniteLane for f32 {
    const IS_FLOATING_POINT: bool = true;
    fn exponent_is_finite(self) -> bool {
        (self.to_bits() & 0x7f800000) != 0x7f800000
    }
}
impl FiniteLane for f64 {
    const IS_FLOATING_POINT: bool = true;
    fn exponent_is_finite(self) -> bool {
        (self.to_bits() & 0x7ff0000000000000) != 0x7ff0000000000000
    }
}
macro_rules! finite_integer {($($t:ty),*)=>{$(impl FiniteLane for $t {fn exponent_is_finite(self)->bool{true}})*};}
finite_integer!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
// Source: src/core/SkVx.h:1069-1077
pub fn isfinite<const N: usize, T: FiniteLane>(v: &[T; N]) -> bool {
    assert!(N > 0 && N.is_power_of_two());
    // Each lane evaluates the exact exponent-mask comparison before the native all reduction.
    if T::IS_FLOATING_POINT {
        let mask: [bool; N] = std::array::from_fn(|i| v[i].exponent_is_finite());
        return all(&mask);
    }
    true
}
// Source: src/core/SkVx.h:1083-1100
// Array halves substitute Vec.lo/hi without changing the recursive load sequence.
// SKVX_DISABLE_SIMD selects these portable overloads, rather than architecture transposes.
/// `v` must name 4*N readable aligned elements and remain valid through the loads.
pub unsafe fn strided_load4<const N: usize, T: Copy>(
    v: *const T,
    a: &mut [T; N],
    b: &mut [T; N],
    c: &mut [T; N],
    d: &mut [T; N],
) {
    assert!(N > 0 && N.is_power_of_two());
    unsafe fn load<T: Copy>(v: *const T, a: &mut [T], b: &mut [T], c: &mut [T], d: &mut [T]) {
        if a.len() == 1 {
            a[0] = v.read();
            b[0] = v.add(1).read();
            c[0] = v.add(2).read();
            d[0] = v.add(3).read();
        } else {
            let mid = a.len() / 2;
            let (al, ah) = a.split_at_mut(mid);
            let (bl, bh) = b.split_at_mut(mid);
            let (cl, ch) = c.split_at_mut(mid);
            let (dl, dh) = d.split_at_mut(mid);
            load(v, al, bl, cl, dl);
            load(v.add(4 * mid), ah, bh, ch, dh);
        }
    }
    load(v, a, b, c, d);
}
// Rust trait specialization represents the source's concrete overload set.
// Actual native vld4/store bodies live in these implementations, not a caller backend.
#[cfg(target_arch = "aarch64")]
pub trait NeonLoad4<const N: usize>: Copy {
    /// Safety: NEON is enabled; v is readable for 4*N initialized primitive lanes.
    unsafe fn neon_load4(v: *const Self) -> ([Self; N], [Self; N], [Self; N], [Self; N]);
}
#[cfg(target_arch = "aarch64")]
/// Safety: NEON is enabled; v is readable for 4*N initialized T lanes.
/// Supported (T,N) are exactly the native ARM64 overloads below.
pub unsafe fn strided_load4_neon<const N: usize, T: NeonLoad4<N>>(
    v: *const T,
) -> ([T; N], [T; N], [T; N], [T; N]) {
    T::neon_load4(v)
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1102-1113
macro_rules! impl_neon_load4 {
    ($n:literal,$t:ty,$load:ident,$store:ident) => {
        impl NeonLoad4<$n> for $t {
            unsafe fn neon_load4(
                v: *const Self,
            ) -> ([Self; $n], [Self; $n], [Self; $n], [Self; $n]) {
                let mat = $load(v);
                let mut a = [0 as $t; $n];
                let mut b = a;
                let mut c = a;
                let mut d = a;
                $store(a.as_mut_ptr(), mat.0);
                $store(b.as_mut_ptr(), mat.1);
                $store(c.as_mut_ptr(), mat.2);
                $store(d.as_mut_ptr(), mat.3);
                (a, b, c, d)
            }
        }
    };
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1114-1114
impl_neon_load4!(2, u32, vld4_u32, vst1_u32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1115-1115
impl_neon_load4!(4, u16, vld4_u16, vst1_u16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1116-1116
impl_neon_load4!(8, u8, vld4_u8, vst1_u8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1117-1117
impl_neon_load4!(2, i32, vld4_s32, vst1_s32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1118-1118
impl_neon_load4!(4, i16, vld4_s16, vst1_s16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1119-1119
impl_neon_load4!(8, i8, vld4_s8, vst1_s8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1120-1120
impl_neon_load4!(2, f32, vld4_f32, vst1_f32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1121-1121
impl_neon_load4!(4, u32, vld4q_u32, vst1q_u32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1122-1122
impl_neon_load4!(8, u16, vld4q_u16, vst1q_u16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1123-1123
impl_neon_load4!(16, u8, vld4q_u8, vst1q_u8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1124-1124
impl_neon_load4!(4, i32, vld4q_s32, vst1q_s32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1125-1125
impl_neon_load4!(8, i16, vld4q_s16, vst1q_s16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1126-1126
impl_neon_load4!(16, i8, vld4q_s8, vst1q_s8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1127-1127
impl_neon_load4!(4, f32, vld4q_f32, vst1q_f32);
// Source: src/core/SkVx.h:1182-1189
/// `v` must name 2*N readable aligned elements and remain valid through the loads.
pub unsafe fn strided_load2<const N: usize, T: Copy>(v: *const T, a: &mut [T; N], b: &mut [T; N]) {
    assert!(N > 0 && N.is_power_of_two());
    unsafe fn load<T: Copy>(v: *const T, a: &mut [T], b: &mut [T]) {
        if a.len() == 1 {
            a[0] = v.read();
            b[0] = v.add(1).read();
        } else {
            let mid = a.len() / 2;
            let (al, ah) = a.split_at_mut(mid);
            let (bl, bh) = b.split_at_mut(mid);
            load(v, al, bl);
            load(v.add(2 * mid), ah, bh);
        }
    }
    load(v, a, b);
}

#[cfg(target_arch = "aarch64")]
pub trait NeonLoad2<const N: usize>: Copy {
    /// Safety: NEON is enabled; v is readable for 2*N initialized primitive lanes.
    unsafe fn neon_load2(v: *const Self) -> ([Self; N], [Self; N]);
}
#[cfg(target_arch = "aarch64")]
/// Safety: NEON is enabled; v is readable for 2*N initialized T lanes.
/// Supported (T,N) are exactly the native ARM64 overloads below.
pub unsafe fn strided_load2_neon<const N: usize, T: NeonLoad2<N>>(v: *const T) -> ([T; N], [T; N]) {
    T::neon_load2(v)
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1191-1196
macro_rules! impl_neon_load2 {
    ($n:literal,$t:ty,$load:ident,$store:ident) => {
        impl NeonLoad2<$n> for $t {
            unsafe fn neon_load2(v: *const Self) -> ([Self; $n], [Self; $n]) {
                let mat = $load(v);
                let mut a = [0 as $t; $n];
                let mut b = a;
                $store(a.as_mut_ptr(), mat.0);
                $store(b.as_mut_ptr(), mat.1);
                (a, b)
            }
        }
    };
}
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1197-1197
impl_neon_load2!(2, u32, vld2_u32, vst1_u32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1198-1198
impl_neon_load2!(4, u16, vld2_u16, vst1_u16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1199-1199
impl_neon_load2!(8, u8, vld2_u8, vst1_u8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1200-1200
impl_neon_load2!(2, i32, vld2_s32, vst1_s32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1201-1201
impl_neon_load2!(4, i16, vld2_s16, vst1_s16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1202-1202
impl_neon_load2!(8, i8, vld2_s8, vst1_s8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1203-1203
impl_neon_load2!(2, f32, vld2_f32, vst1_f32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1204-1204
impl_neon_load2!(4, u32, vld2q_u32, vst1q_u32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1205-1205
impl_neon_load2!(8, u16, vld2q_u16, vst1q_u16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1206-1206
impl_neon_load2!(16, u8, vld2q_u8, vst1q_u8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1207-1207
impl_neon_load2!(4, i32, vld2q_s32, vst1q_s32);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1208-1208
impl_neon_load2!(8, i16, vld2q_s16, vst1q_s16);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1209-1209
impl_neon_load2!(16, i8, vld2q_s8, vst1q_s8);
#[cfg(target_arch = "aarch64")]
// Source: src/core/SkVx.h:1210-1210
impl_neon_load2!(4, f32, vld2q_f32, vst1q_f32);
// Source: src/core/SkVx.h:1215-1215
#[allow(non_camel_case_types)]
pub type float2 = Vec<2, f32>;
// Source: src/core/SkVx.h:1216-1216
#[allow(non_camel_case_types)]
pub type float4 = Vec<4, f32>;
// Source: src/core/SkVx.h:1217-1217
#[allow(non_camel_case_types)]
pub type float8 = Vec<8, f32>;
// Source: src/core/SkVx.h:1219-1219
#[allow(non_camel_case_types)]
pub type double2 = Vec<2, f64>;
// Source: src/core/SkVx.h:1220-1220
#[allow(non_camel_case_types)]
pub type double4 = Vec<4, f64>;
// Source: src/core/SkVx.h:1221-1221
#[allow(non_camel_case_types)]
pub type double8 = Vec<8, f64>;
// Source: src/core/SkVx.h:1223-1223
#[allow(non_camel_case_types)]
pub type byte2 = Vec<2, u8>;
// Source: src/core/SkVx.h:1224-1224
#[allow(non_camel_case_types)]
pub type byte4 = Vec<4, u8>;
// Source: src/core/SkVx.h:1225-1225
#[allow(non_camel_case_types)]
pub type byte8 = Vec<8, u8>;
// Source: src/core/SkVx.h:1226-1226
#[allow(non_camel_case_types)]
pub type byte16 = Vec<16, u8>;
// Source: src/core/SkVx.h:1228-1228
#[allow(non_camel_case_types)]
pub type int2 = Vec<2, i32>;
// Source: src/core/SkVx.h:1229-1229
#[allow(non_camel_case_types)]
pub type int4 = Vec<4, i32>;
// Source: src/core/SkVx.h:1230-1230
#[allow(non_camel_case_types)]
pub type int8 = Vec<8, i32>;
// Source: src/core/SkVx.h:1232-1232
#[allow(non_camel_case_types)]
pub type ushort2 = Vec<2, u16>;
// Source: src/core/SkVx.h:1233-1233
#[allow(non_camel_case_types)]
pub type ushort4 = Vec<4, u16>;
// Source: src/core/SkVx.h:1234-1234
#[allow(non_camel_case_types)]
pub type ushort8 = Vec<8, u16>;
// Source: src/core/SkVx.h:1236-1236
#[allow(non_camel_case_types)]
pub type uint2 = Vec<2, u32>;
// Source: src/core/SkVx.h:1237-1237
#[allow(non_camel_case_types)]
pub type uint4 = Vec<4, u32>;
// Source: src/core/SkVx.h:1238-1238
#[allow(non_camel_case_types)]
pub type uint8 = Vec<8, u32>;
// Source: src/core/SkVx.h:1240-1240
#[allow(non_camel_case_types)]
pub type long2 = Vec<2, i64>;
// Source: src/core/SkVx.h:1241-1241
#[allow(non_camel_case_types)]
pub type long4 = Vec<4, i64>;
// Source: src/core/SkVx.h:1242-1242
#[allow(non_camel_case_types)]
pub type long8 = Vec<8, i64>;
// Source: src/core/SkVx.h:1245-1245
#[allow(non_camel_case_types)]
pub type half2 = Vec<2, u16>;
// Source: src/core/SkVx.h:1246-1246
#[allow(non_camel_case_types)]
pub type half4 = Vec<4, u16>;
// Source: src/core/SkVx.h:1247-1247
#[allow(non_camel_case_types)]
pub type half8 = Vec<8, u16>;

// Native language boundary (src/core/SkVx.h:69-76,95-235,286-1250):
// SKVX_ALWAYS_INLINE and SI/SIT/SIN/SINT/SINTU do not declare constexpr
// functions. In particular the explicit scalar/list constructors, arithmetic,
// casts, memcpy loads/stores and ISA intrinsics remain runtime operations.
// Native `if constexpr` selects a template branch; it does not make the
// enclosing operation a constexpr function. Existing const-generic shape and
// cfg dispatch preserve that distinction without a scalar CTFE replacement.
