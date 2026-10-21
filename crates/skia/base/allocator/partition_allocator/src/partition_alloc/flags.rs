// Source: partition_alloc/flags.h:47-98
// Native flags macro expanded only for genuine integral-payload enum mappings.
// The all-flags value is the native specialization (RawPtrTraits::kAllMask),
// not an invented maximum discriminant of a Rust closed enum.
macro_rules! PA_DEFINE_OPERATORS_FOR_FLAGS {
 ($ty:ty,$all:expr)=>{
  const impl core::ops::BitAnd for $ty{type Output=Self;fn bitand(self,rhs:Self)->Self{Self(self.0&rhs.0)}}
  const impl core::ops::BitAndAssign for $ty{fn bitand_assign(&mut self,rhs:Self){*self=Self(self.0&rhs.0);}}
  const impl core::ops::BitOr for $ty{type Output=Self;fn bitor(self,rhs:Self)->Self{Self(self.0|rhs.0)}}
  const impl core::ops::BitOrAssign for $ty{fn bitor_assign(&mut self,rhs:Self){*self=Self(self.0|rhs.0);}}
  const impl core::ops::BitXor for $ty{type Output=Self;fn bitxor(self,rhs:Self)->Self{Self(self.0^rhs.0)}}
  const impl core::ops::BitXorAssign for $ty{fn bitxor_assign(&mut self,rhs:Self){*self=Self(self.0^rhs.0);}}
  const impl core::ops::Not for $ty{type Output=Self;fn not(self)->Self{Self($all&!self.0)}}
 };
}
pub(crate) use PA_DEFINE_OPERATORS_FOR_FLAGS;
// Source: partition_alloc/flags.h:19-45
// Generic scoped-enum constraint and specialized all-mask metadata. No Rust
// closed enum invalid values are constructed by integer bitwise operations.
pub const trait ScopedFlags:Copy{const ALL_MASK:Self;fn bits(self)->u32;}
pub const fn AreValidFlags<T:[const] ScopedFlags>(value:T)->bool{value.bits()&!T::ALL_MASK.bits()==0}
pub const fn ContainsFlags<T:[const] ScopedFlags>(superset:T,subset:T)->bool{superset.bits()&subset.bits()==subset.bits()}
// This u32 projection supports the actual RawPtrTraits underlying type. Other
// underlying-width generic kAllFlags<Enum> expressions remain separate pending
// native instantiations; no signedness/promotion equivalence is claimed for them.
