//! Native integer/enum fit checks. Traits express only C++ type properties and
//! static casts; they add no runtime state, object, owner or conversion policy.
// Source: include/private/SkTFitsIn.h:18-25
pub trait sk_strip_enum: Copy {
    type Type: SkArithmetic;
}
pub trait SkArithmetic: Copy {
    const SIGNED: bool;
    const SIZE: usize;
    const MAX: Self;
}
/// Rust language projection of the actual C++ static_cast operation.
pub const trait SkCastFrom<S> {
    fn sk_cast_from(source: S) -> Self;
}
macro_rules! arithmetic {
 ($($t:ty=>$signed:expr),* $(,)?)=>{$(
  impl sk_strip_enum for $t {type Type=Self;}
  impl SkArithmetic for $t {const SIGNED:bool=$signed;const SIZE:usize=core::mem::size_of::<Self>();const MAX:Self=<$t>::MAX;}
 )*};
}
arithmetic!(i8=>true,u8=>false,i16=>true,u16=>false,i32=>true,u32=>false,i64=>true,u64=>false,isize=>true,usize=>false);
impl sk_strip_enum for bool {
    type Type = Self;
}
impl SkArithmetic for bool {
    const SIGNED: bool = false;
    const SIZE: usize = core::mem::size_of::<Self>();
    const MAX: Self = true;
}
macro_rules! cast_sources {
 ($dst:ty;$($src:ty),* $(,)?)=>{$(const impl SkCastFrom<$src> for $dst {fn sk_cast_from(source:$src)->Self {source as $dst}})*};
}
macro_rules! cast_destinations {
 ($($dst:ty),* $(,)?)=>{$(cast_sources!($dst;i8,u8,i16,u16,i32,u32,i64,u64,isize,usize,f32,f64);)*};
}
cast_destinations!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
macro_rules! bool_casts {
 ($($t:ty),* $(,)?)=>{$(
  const impl SkCastFrom<bool> for $t {fn sk_cast_from(source:bool)->Self {source as u8 as $t}}
  const impl SkCastFrom<$t> for bool {fn sk_cast_from(source:$t)->Self {source!=0 as $t}}
 )*};
}
bool_casts!(i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64);
const impl SkCastFrom<bool> for bool {
    fn sk_cast_from(source: bool) -> Self {
        source
    }
}

// Source: include/private/SkTFitsIn.h:26-67

/**
 * In C++ an unsigned to signed cast where the source value cannot be represented in the destination
 * type results in an implementation defined destination value. Unlike C, C++ does not allow a trap.
 * This makes "(S)(D)s == s" a possibly useful test. However, there are two cases where this is
 * incorrect:
 *
 * when testing if a value of a smaller signed type can be represented in a larger unsigned type
 * (int8_t)(uint16_t)-1 == -1 => (int8_t)0xFFFF == -1 => [implementation defined] == -1
 *
 * when testing if a value of a larger unsigned type can be represented in a smaller signed type
 * (uint16_t)(int8_t)0xFFFF == 0xFFFF => (uint16_t)-1 == 0xFFFF => 0xFFFF == 0xFFFF => true.
 *
 * Consider the cases:
 *   u = unsigned, less digits
 *   U = unsigned, more digits
 *   s = signed, less digits
 *   S = signed, more digits
 *   v is the value we're considering.
 *
 * u -> U: (u)(U)v == v, trivially true
 * U -> u: (U)(u)v == v, both casts well defined, test works
 * s -> S: (s)(S)v == v, trivially true
 * S -> s: (S)(s)v == v, first cast implementation value, second cast defined, test works
 * s -> U: (s)(U)v == v, *this is bad*, the second cast results in implementation defined value
 * S -> u: (S)(u)v == v, the second cast is required to prevent promotion of rhs to unsigned
 * u -> S: (u)(S)v == v, trivially true
 * U -> s: (U)(s)v == v, *this is bad*,
 *                             first cast results in implementation defined value,
 *                             second cast is defined. However, this creates false positives
 *                             uint16_t x = 0xFFFF
 *                                (uint16_t)(int8_t)x == x
 *                             => (uint16_t)-1        == x
 *                             => 0xFFFF              == x
 *                             => true
 *
 * So for the eight cases three are trivially true, three more are valid casts, and two are special.
 * The two 'full' checks which otherwise require two comparisons are valid cast checks.
 * The two remaining checks s -> U [v >= 0] and U -> s [v <= max(s)] can be done with one op.
 */

// The above native rationale describes the unchanged three-way expression
// below. These are documentation rows, not extra executable helper bodies.
// Source: include/private/SkTFitsIn.h:71-95
// The three native branches, including signed/unsigned cast exceptions, are
// kept directly. Rust integral wrappers bind sk_strip_enum to their actual
// underlying arithmetic type and provide their real cast language operations.
pub const fn SkTFitsIn<D, S>(src: S) -> bool
where
    D: sk_strip_enum + [const] SkCastFrom<S>,
    S: sk_strip_enum
        + [const] PartialOrd
        + [const] SkCastFrom<D>
        + [const] SkCastFrom<i32>
        + [const] SkCastFrom<<D as sk_strip_enum>::Type>,
{
    if <S::Type as SkArithmetic>::SIGNED
        && !<D::Type as SkArithmetic>::SIGNED
        && <S::Type as SkArithmetic>::SIZE <= <D::Type as SkArithmetic>::SIZE
    {
        <S as SkCastFrom<i32>>::sk_cast_from(0) <= src
    } else if <D::Type as SkArithmetic>::SIGNED
        && !<S::Type as SkArithmetic>::SIGNED
        && <D::Type as SkArithmetic>::SIZE <= <S::Type as SkArithmetic>::SIZE
    {
        src <= <S as SkCastFrom<D::Type>>::sk_cast_from(<D::Type as SkArithmetic>::MAX)
    } else {
        <S as SkCastFrom<D>>::sk_cast_from(<D as SkCastFrom<S>>::sk_cast_from(src)) == src
    }
}
