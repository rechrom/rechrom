// Source: partition_alloc/partition_alloc_forward.h:54-60
pub unsafe trait Offset:Copy{
 unsafe fn advance<T>(self,p:*mut T)->*mut T;
 unsafe fn retreat<T>(self,p:*mut T)->*mut T;
}
macro_rules! signed_offsets{($($t:ty),*)=>{$(unsafe impl Offset for $t{
 unsafe fn advance<T>(self,p:*mut T)->*mut T{const{assert!(core::mem::size_of::<$t>()<=core::mem::size_of::<isize>())};unsafe{p.offset(self as isize)}}
 unsafe fn retreat<T>(self,p:*mut T)->*mut T{const{assert!(core::mem::size_of::<$t>()<=core::mem::size_of::<isize>())};unsafe{p.offset((self as isize).wrapping_neg())}}
})*};}
macro_rules! unsigned_offsets{($($t:ty),*)=>{$(unsafe impl Offset for $t{
 unsafe fn advance<T>(self,p:*mut T)->*mut T{const{assert!(core::mem::size_of::<$t>()<=core::mem::size_of::<isize>())};unsafe{p.add(self as usize)}}
 unsafe fn retreat<T>(self,p:*mut T)->*mut T{const{assert!(core::mem::size_of::<$t>()<=core::mem::size_of::<isize>())};unsafe{p.sub(self as usize)}}
})*};}
signed_offsets!(i8,i16,i32,i64,isize);unsigned_offsets!(u8,u16,u32,u64,usize,bool);

// Remaining unrelated partition_alloc_forward.h declarations are not claimed
// translated by this direct offset-type dependency excerpt.
