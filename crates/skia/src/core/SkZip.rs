// Source: src/core/SkZip.h:29-49
// One iterator implementation serves both the existing pair and arbitrary pointer lists.
pub struct ZipIterator<'z, Z: ZipView> {
    fZip: &'z Z,
    fIndex: usize,
}
pub type Iterator<'z, A, B> = ZipIterator<'z, SkZip<A, B>>;
impl<Z: ZipView> Copy for ZipIterator<'_, Z> {}
impl<Z: ZipView> Clone for ZipIterator<'_, Z> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'z, Z: ZipView> ZipIterator<'z, Z> {
    pub const fn new(zip: &'z Z, index: usize) -> Self {
        Self {
            fZip: zip,
            fIndex: index,
        }
    }
    pub fn increment(&mut self) -> &mut Self {
        self.fIndex = self.fIndex.wrapping_add(1);
        self
    }
    pub fn post_increment(&mut self) -> Self {
        let previous = *self;
        self.increment();
        previous
    }
    /// Safety: original storage remains initialized/aligned/live and shared-readable
    /// for 'a; this raw view never extends any lease or backing ownership.
    pub unsafe fn value<'a>(&self) -> Z::References<'a>
    where
        Z: 'a,
    {
        self.fZip.zip_at(self.fIndex)
    }
    pub fn difference(self, rhs: Self) -> isize {
        self.fIndex.wrapping_sub(rhs.fIndex) as isize
    }
}
impl<Z: ZipView> PartialEq for ZipIterator<'_, Z> {
    fn eq(&self, rhs: &Self) -> bool {
        self.fIndex == rhs.fIndex
    }
}
impl<Z: ZipView> Eq for ZipIterator<'_, Z> {}
impl<'z, Z: ZipView> core::ops::Sub for ZipIterator<'z, Z> {
    type Output = isize;
    fn sub(self, rhs: Self) -> isize {
        self.difference(rhs)
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:55-55
    pub const fn new() -> Self {
        Self {
            pointers: (std::ptr::null(), std::ptr::null()),
            size: 0,
        }
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:57-57
    pub const fn from_raw(size: usize, a: *const A, b: *const B) -> Self {
        Self {
            pointers: (a, b),
            size,
        }
    }
}
// Source: src/core/SkZip.h:58-59
impl<A, B> Copy for SkZip<A, B> {}
impl<A, B> Clone for SkZip<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}
// Source: src/core/SkZip.h:62-64
// Rust const qualification is a transparent element view, not a new allocation.
// Its field is private and it supplies no mutable projection or public constructor.
// All returned backing references still obey SkZip's original unsafe leases.
#[repr(transparent)]
pub struct ConstElement<T>(T);
impl<T> core::ops::Deref for ConstElement<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
// Explicit witness for the native U* -> T* implicit conversion AND equal-size test.
// Safety: convert_pointer implements that real pointer conversion, preserves object
// provenance/alignment, and never removes const qualification. No arbitrary bit cast.
pub unsafe trait CanConvertToConst<T>: Sized {
    const SAME_SIZE: () = assert!(core::mem::size_of::<Self>() == core::mem::size_of::<T>());
    fn convert_pointer(pointer: *const Self) -> *const T;
}
unsafe impl<T> CanConvertToConst<T> for T {
    fn convert_pointer(pointer: *const T) -> *const T {
        pointer
    }
}
unsafe impl<T> CanConvertToConst<ConstElement<T>> for T {
    fn convert_pointer(pointer: *const T) -> *const ConstElement<T> {
        pointer.cast()
    }
}
// Source: src/core/SkZip.h:67-69
impl<A, B> SkZip<A, B> {
    pub fn from_const_convertible<U: CanConvertToConst<A>, V: CanConvertToConst<B>>(
        that: &SkZip<U, V>,
    ) -> Self {
        Self::from_variadic(
            SkZipVariadic::<ZipCons<A, ZipCons<B, ZipNil>>>::from_const_convertible(
                &that.as_variadic(),
            ),
        )
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:71-71
    pub unsafe fn at<'a>(&self, i: usize) -> (&'a A, &'a B) {
        self.index(i)
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:72-72
    pub const fn size(&self) -> usize {
        self.size
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:73-73
    pub const fn empty(&self) -> bool {
        self.size() == 0
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:74-74
    pub unsafe fn front<'a>(&self) -> (&'a A, &'a B) {
        self.index(0)
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:75-75
    pub unsafe fn back<'a>(&self) -> (&'a A, &'a B) {
        self.index(self.size().wrapping_sub(1))
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:76-77
    pub fn begin(&self) -> Iterator<'_, A, B> {
        Iterator::new(self, 0)
    }
    pub fn end(&self) -> Iterator<'_, A, B> {
        Iterator::new(self, self.size())
    }
    /// Safety: all initialized backing elements stay alive and shared-readable
    /// for 'a, with no mutable alias until this iterator and its yielded references expire.
    pub unsafe fn iter<'a>(&'a self) -> BorrowedIter<'a, A, B> {
        BorrowedIter {
            current: self.begin(),
        }
    }
    /// Safety: both columns are initialized, alive, mutually disjoint and
    /// exclusively writable for the chosen reference lifetimes. No copied view may alias them.
    pub unsafe fn at_mut<'a>(&mut self, i: usize) -> (&'a mut A, &'a mut B) {
        let (a, (b, ())) = self.as_variadic().at_mut(i);
        (a, b)
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:78-80
    /// Safety: column0 is exclusively writable, initialized and alive for 'a;
    /// no reference from another zip copy or from the original storage aliases it.
    pub unsafe fn get0_mut<'a>(&mut self) -> &'a mut [A] {
        if self.size == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(self.pointers.0.cast_mut(), self.size) }
        }
    }
    /// Safety: column1 satisfies the same exclusive backing lease as get0_mut.
    pub unsafe fn get1_mut<'a>(&mut self) -> &'a mut [B] {
        if self.size == 0 {
            &mut []
        } else {
            unsafe { core::slice::from_raw_parts_mut(self.pointers.1.cast_mut(), self.size) }
        }
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:78-80
    pub unsafe fn get0<'a>(&self) -> &'a [A] {
        if self.size == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.pointers.0, self.size)
        }
    }
}
impl<A, B> SkZip<A, B> {
    pub unsafe fn get1<'a>(&self) -> &'a [B] {
        if self.size == 0 {
            &[]
        } else {
            std::slice::from_raw_parts(self.pointers.1, self.size)
        }
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:81-81
    pub const fn data(&self) -> (*const A, *const B) {
        self.pointers
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:82-86
    pub fn first(&self, n: usize) -> Self {
        Self::from_variadic(self.as_variadic().first(n))
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:87-91
    pub unsafe fn last(&self, n: usize) -> Self {
        Self::from_variadic(self.as_variadic().last(n))
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:92-97
    pub unsafe fn subspan(&self, offset: usize, count: usize) -> Self {
        Self::from_variadic(self.as_variadic().subspan(offset, count))
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:102-106
    // Source: src/core/SkZip.h:108-111
    unsafe fn index<'a>(&self, i: usize) -> (&'a A, &'a B) {
        let (a, (b, ())) = self.as_variadic().at(i);
        (a, b)
    }
}
impl<A, B> SkZip<A, B> {
    // Source: src/core/SkZip.h:113-117
    // Source: src/core/SkZip.h:119-122
    unsafe fn pointers_at(&self, i: usize) -> (*const A, *const B) {
        let pointers = self.as_variadic().pointers_at(i);
        (pointers.head, pointers.tail.head)
    }
}
// UNVERIFIED frozen-scope arbitrary-arity template adaptation. No validation run.
// Source: src/core/SkZip.h:24-26
// Recursive heterogeneous tuples retain pointer order without an arity cap.
// References are nested (head,tail), the Rust representation of native tuple expansion.
pub struct ZipNil;
pub struct ZipCons<T, Tail> {
    pub head: *const T,
    pub tail: Tail,
}
impl Copy for ZipNil {}
impl Clone for ZipNil {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T, Tail: Copy> Copy for ZipCons<T, Tail> {}
impl<T, Tail: Copy> Clone for ZipCons<T, Tail> {
    fn clone(&self) -> Self {
        *self
    }
}
pub unsafe trait ZipPointers: Copy {
    type References<'a>
    where
        Self: 'a;
    type MutableReferences<'a>
    where
        Self: 'a;
    fn nulls() -> Self;
    unsafe fn references<'a>(&self, index: usize) -> Self::References<'a>
    where
        Self: 'a;
    unsafe fn references_mut<'a>(&self, index: usize) -> Self::MutableReferences<'a>
    where
        Self: 'a;
    unsafe fn offset(&self, index: usize) -> Self;
}
unsafe impl ZipPointers for ZipNil {
    type References<'a>
        = ()
    where
        Self: 'a;
    type MutableReferences<'a>
        = ()
    where
        Self: 'a;
    fn nulls() -> Self {
        Self
    }
    unsafe fn references<'a>(&self, _index: usize)
    where
        Self: 'a,
    {
    }
    unsafe fn references_mut<'a>(&self, _index: usize)
    where
        Self: 'a,
    {
    }
    unsafe fn offset(&self, _index: usize) -> Self {
        Self
    }
}
unsafe impl<T, Tail: ZipPointers> ZipPointers for ZipCons<T, Tail> {
    type References<'a>
        = (&'a T, Tail::References<'a>)
    where
        Self: 'a;
    type MutableReferences<'a>
        = (&'a mut T, Tail::MutableReferences<'a>)
    where
        Self: 'a;
    fn nulls() -> Self {
        Self {
            head: core::ptr::null(),
            tail: Tail::nulls(),
        }
    }
    // Source: src/core/SkZip.h:108-111
    unsafe fn references<'a>(&self, index: usize) -> Self::References<'a>
    where
        Self: 'a,
    {
        (&*self.head.add(index), self.tail.references(index))
    }
    // Exclusive references require every column to be writable and mutually disjoint;
    // const-qualified backing never satisfies this whole-list operation. Select a
    // writable individual column through get<I>() when other columns are const.
    unsafe fn references_mut<'a>(&self, index: usize) -> Self::MutableReferences<'a>
    where
        Self: 'a,
    {
        (
            &mut *self.head.cast_mut().add(index),
            self.tail.references_mut(index),
        )
    }
    // Source: src/core/SkZip.h:119-122
    unsafe fn offset(&self, index: usize) -> Self {
        Self {
            head: self.head.add(index),
            tail: self.tail.offset(index),
        }
    }
}
// Both existing pair and generic view use the very same iterator protocol.
pub unsafe trait ZipView {
    type References<'a>
    where
        Self: 'a;
    unsafe fn zip_at<'a>(&self, index: usize) -> Self::References<'a>
    where
        Self: 'a;
}
// Source: src/core/SkZip.h:62-69
pub unsafe trait ConvertZipPointers<Target: ZipPointers>: ZipPointers {
    fn converted(self) -> Target;
}
unsafe impl ConvertZipPointers<ZipNil> for ZipNil {
    fn converted(self) -> ZipNil {
        self
    }
}
unsafe impl<U, T, Us, Ts> ConvertZipPointers<ZipCons<T, Ts>> for ZipCons<U, Us>
where
    U: CanConvertToConst<T>,
    Us: ConvertZipPointers<Ts>,
    Ts: ZipPointers,
{
    fn converted(self) -> ZipCons<T, Ts> {
        let () = <U as CanConvertToConst<T>>::SAME_SIZE;
        ZipCons {
            head: U::convert_pointer(self.head),
            tail: self.tail.converted(),
        }
    }
}
// Source: src/core/SkZip.h:78-80
pub struct ZipHere;
pub struct ZipNext<I>(core::marker::PhantomData<I>);
pub trait ZipGet<I>: ZipPointers {
    type Element;
    fn pointer(&self) -> *const Self::Element;
}
impl<T, Tail: ZipPointers> ZipGet<ZipHere> for ZipCons<T, Tail> {
    type Element = T;
    fn pointer(&self) -> *const T {
        self.head
    }
}
impl<T, Tail: ZipGet<I>, I> ZipGet<ZipNext<I>> for ZipCons<T, Tail> {
    type Element = Tail::Element;
    fn pointer(&self) -> *const Self::Element {
        self.tail.pointer()
    }
}
// Raw nonowning SkSpan metadata. Access remains unsafe with the original column lease.
pub struct ZipColumn<T> {
    pub data: *const T,
    pub size: usize,
}
// Source: src/core/SkZip.h:124-126
pub struct SkZipVariadic<P: ZipPointers> {
    fPointers: P,
    fSize: usize,
}
impl<P: ZipPointers> Copy for SkZipVariadic<P> {}
impl<P: ZipPointers> Clone for SkZipVariadic<P> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<P: ZipPointers> SkZipVariadic<P> {
    // Source: src/core/SkZip.h:51-57
    pub fn new() -> Self {
        Self {
            fPointers: P::nulls(),
            fSize: 0,
        }
    }
    pub fn from_pointers(size: usize, pointers: P) -> Self {
        Self {
            fPointers: pointers,
            fSize: size,
        }
    }
    // Source: src/core/SkZip.h:67-69
    pub fn from_const_convertible<U: ConvertZipPointers<P>>(that: &SkZipVariadic<U>) -> Self {
        Self::from_pointers(that.size(), that.data().converted())
    }
    // Source: src/core/SkZip.h:71-81
    pub unsafe fn at<'a>(&self, index: usize) -> P::References<'a>
    where
        P: 'a,
    {
        zip_index_bounds(self.fSize, index);
        self.fPointers.references(index)
    }
    /// Safety: all columns are initialized, writable, mutually disjoint, and
    /// exclusively borrowed for 'a; copied views may not alias this access.
    pub unsafe fn at_mut<'a>(&mut self, index: usize) -> P::MutableReferences<'a>
    where
        P: 'a,
    {
        zip_index_bounds(self.fSize, index);
        self.fPointers.references_mut(index)
    }
    pub fn size(&self) -> usize {
        self.fSize
    }
    pub fn empty(&self) -> bool {
        self.size() == 0
    }
    pub unsafe fn front<'a>(&self) -> P::References<'a>
    where
        P: 'a,
    {
        self.at(0)
    }
    pub unsafe fn back<'a>(&self) -> P::References<'a>
    where
        P: 'a,
    {
        self.at(self.size().wrapping_sub(1))
    }
    pub fn begin(&self) -> ZipIterator<'_, Self> {
        ZipIterator::new(self, 0)
    }
    pub fn end(&self) -> ZipIterator<'_, Self> {
        ZipIterator::new(self, self.size())
    }
    pub fn get<I>(&self) -> ZipColumn<P::Element>
    where
        P: ZipGet<I>,
    {
        ZipColumn {
            data: self.fPointers.pointer(),
            size: self.fSize,
        }
    }
    pub fn data(&self) -> P {
        self.fPointers
    }
    // Source: src/core/SkZip.h:82-86
    pub fn first(&self, n: usize) -> Self {
        crate::SkASSERT!(@source b"src/core/SkZip.h\0".as_ptr(),83; @condition b"n <= this->size()\0".as_ptr(); n<=self.size());
        if n == 0 {
            return Self::new();
        }
        Self::from_pointers(n, self.fPointers)
    }
    // Source: src/core/SkZip.h:87-91
    pub unsafe fn last(&self, n: usize) -> Self {
        crate::SkASSERT!(@source b"src/core/SkZip.h\0".as_ptr(),88; @condition b"n <= this->size()\0".as_ptr(); n<=self.size());
        if n == 0 {
            return Self::new();
        }
        Self::from_pointers(n, self.pointers_at(self.fSize - n))
    }
    // Source: src/core/SkZip.h:92-97
    pub unsafe fn subspan(&self, offset: usize, count: usize) -> Self {
        crate::SkASSERT!(@source b"src/core/SkZip.h\0".as_ptr(),93; @condition b"offset < this->size()\0".as_ptr(); offset<self.size());
        crate::SkASSERT!(@source b"src/core/SkZip.h\0".as_ptr(),94; @condition b"count <= this->size() - offset\0".as_ptr(); count<=self.size()-offset);
        if count == 0 {
            return Self::new();
        }
        Self::from_pointers(count, self.pointers_at(offset))
    }
    // Source: src/core/SkZip.h:113-117
    unsafe fn pointers_at(&self, index: usize) -> P {
        zip_index_bounds(self.fSize, index);
        self.fPointers.offset(index)
    }
}
// Source: src/core/SkZip.h:102-106
fn zip_index_bounds(size: usize, index: usize) {
    debug_assert!(size > 0);
    debug_assert!(index < size);
}
unsafe impl<P: ZipPointers> ZipView for SkZipVariadic<P> {
    type References<'a>
        = P::References<'a>
    where
        Self: 'a;
    unsafe fn zip_at<'a>(&self, index: usize) -> Self::References<'a>
    where
        Self: 'a,
    {
        self.at(index)
    }
}
unsafe impl<A, B> ZipView for SkZip<A, B> {
    type References<'a>
        = (&'a A, &'a B)
    where
        Self: 'a;
    unsafe fn zip_at<'a>(&self, index: usize) -> Self::References<'a>
    where
        Self: 'a,
    {
        self.at(index)
    }
}
impl<A, B> SkZip<A, B> {
    fn as_variadic(&self) -> SkZipVariadic<ZipCons<A, ZipCons<B, ZipNil>>> {
        SkZipVariadic::from_pointers(
            self.size,
            ZipCons {
                head: self.pointers.0,
                tail: ZipCons {
                    head: self.pointers.1,
                    tail: ZipNil,
                },
            },
        )
    }
    fn from_variadic(view: SkZipVariadic<ZipCons<A, ZipCons<B, ZipNil>>>) -> Self {
        let p = view.data();
        Self::from_raw(view.size(), p.head, p.tail.head)
    }
}
// Existing two-column API used by CPU GlyphRun; delegates the same arbitrary-arity view algorithms.
// The native view is non-owning. Dereferencing APIs are unsafe so copying a raw view cannot
// create a safe reference after its storage owner or builder lease has expired.
// Adapted source: src/core/SkZip.h:24-26: two-column Rust storage and reference-returning functions split native variadic class/ReturnTuple scope
// Source: src/core/SkZip.h:124-126
pub struct SkZip<A, B> {
    pointers: (*const A, *const B),
    size: usize,
}
// The checked lifetime lease above adapts native begin/end loops to Rust's
// safe Iterator protocol; it does not acquire or extend ownership of the columns.
pub struct BorrowedIter<'a, A, B> {
    current: Iterator<'a, A, B>,
}
impl<'a, A: 'a, B: 'a> core::iter::Iterator for BorrowedIter<'a, A, B> {
    type Item = (&'a A, &'a B);
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.fIndex == self.current.fZip.size() {
            return None;
        }
        let previous = self.current.post_increment();
        Some(unsafe { previous.value() })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.current.fZip.size() - self.current.fIndex;
        (size, Some(size))
    }
}
impl<A, B> core::iter::ExactSizeIterator for BorrowedIter<'_, A, B> {}
impl<A, B> core::iter::FusedIterator for BorrowedIter<'_, A, B> {}

// UNVERIFIED frozen-scope template adaptation; no compilation/tests were run.
// Source: src/core/SkZip.h:129-133
// Rust references already encode lvalue-ness, raw pointer values already decay;
// normalization keeps pointee constness separately from the pointer's own cv.
pub struct KnownZipSize;
pub struct UnknownZipSize;
pub trait ZipInput {
    type Element;
    type SizeKind;
    fn Data(&self) -> *const Self::Element;
    fn Size(&self) -> usize;
}
// Source: src/core/SkZip.h:135-139
impl<T> ZipInput for *const T {
    type Element = ConstElement<T>;
    type SizeKind = UnknownZipSize;
    fn Data(&self) -> *const Self::Element {
        (*self).cast()
    }
    fn Size(&self) -> usize {
        usize::MAX
    }
}
impl<T> ZipInput for *mut T {
    type Element = T;
    type SizeKind = UnknownZipSize;
    fn Data(&self) -> *const T {
        *self
    }
    fn Size(&self) -> usize {
        usize::MAX
    }
}
// Source: src/core/SkZip.h:140-145
// Source: src/core/SkZip.h:152-158
// Only borrowed collections implement ZipInput below. Owning Vec/array/TArray
// rvalues do not, exactly preserving the native rejection of temporary containers.
// Safety: data/size identify a single live contiguous array, retaining its element
// type and full count. Mutable access additionally requires exclusive backing.
pub unsafe trait ZipContiguous {
    type Element;
    fn zip_data(&self) -> *const Self::Element;
    fn zip_size(&self) -> usize;
    fn zip_data_mut(&mut self) -> *mut Self::Element;
}
unsafe impl<T, const N: usize> ZipContiguous for [T; N] {
    type Element = T;
    fn zip_data(&self) -> *const T {
        self.as_ptr()
    }
    fn zip_size(&self) -> usize {
        N
    }
    fn zip_data_mut(&mut self) -> *mut T {
        self.as_mut_ptr()
    }
}
unsafe impl<T> ZipContiguous for [T] {
    type Element = T;
    fn zip_data(&self) -> *const T {
        self.as_ptr()
    }
    fn zip_size(&self) -> usize {
        self.len()
    }
    fn zip_data_mut(&mut self) -> *mut T {
        self.as_mut_ptr()
    }
}
unsafe impl<T> ZipContiguous for Vec<T> {
    type Element = T;
    fn zip_data(&self) -> *const T {
        self.as_ptr()
    }
    fn zip_size(&self) -> usize {
        self.len()
    }
    fn zip_data_mut(&mut self) -> *mut T {
        self.as_mut_ptr()
    }
}
unsafe impl<T, const M: bool> ZipContiguous for crate::include::private::SkTArray::TArray<T, M> {
    type Element = T;
    fn zip_data(&self) -> *const T {
        self.data()
    }
    fn zip_size(&self) -> usize {
        self.size() as usize
    }
    fn zip_data_mut(&mut self) -> *mut T {
        self.data_mut()
    }
}
impl<C: ZipContiguous + ?Sized> ZipInput for &C {
    type Element = ConstElement<C::Element>;
    type SizeKind = KnownZipSize;
    fn Data(&self) -> *const Self::Element {
        (**self).zip_data().cast()
    }
    fn Size(&self) -> usize {
        (**self).zip_size()
    }
}
impl<C: ZipContiguous + ?Sized> ZipInput for &mut C {
    type Element = C::Element;
    type SizeKind = KnownZipSize;
    // Native mutable collection data is a mutable pointer even when its temporary
    // metadata is read through &self. No reference to an element is created here.
    fn Data(&self) -> *const Self::Element {
        (**self).zip_data()
    }
    fn Size(&self) -> usize {
        (**self).zip_size()
    }
}
// Source: src/core/SkZip.h:146-151
// Actual span-view rvalues are allowed; these wrappers retain the caller's lease.
// No backing array is copied or owned. A copied native-style view never prolongs
// the original initialized/shared or exclusive backing-storage access permission.
pub struct ZipSpan<'a, T> {
    data: *const T,
    size: usize,
    lease: core::marker::PhantomData<&'a [T]>,
}
impl<'a, T> ZipSpan<'a, T> {
    pub fn from_slice(slice: &'a [T]) -> Self {
        Self {
            data: slice.as_ptr(),
            size: slice.len(),
            lease: core::marker::PhantomData,
        }
    }
    pub unsafe fn from_raw(data: *const T, size: usize) -> Self {
        Self {
            data,
            size,
            lease: core::marker::PhantomData,
        }
    }
}
impl<T> Copy for ZipSpan<'_, T> {}
impl<T> Clone for ZipSpan<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> ZipInput for ZipSpan<'_, T> {
    type Element = ConstElement<T>;
    type SizeKind = KnownZipSize;
    fn Data(&self) -> *const Self::Element {
        self.data.cast()
    }
    fn Size(&self) -> usize {
        self.size
    }
}
pub struct ZipSpanMut<'a, T> {
    data: *mut T,
    size: usize,
    lease: core::marker::PhantomData<&'a mut [T]>,
}
impl<'a, T> ZipSpanMut<'a, T> {
    pub fn from_slice(slice: &'a mut [T]) -> Self {
        Self {
            data: slice.as_mut_ptr(),
            size: slice.len(),
            lease: core::marker::PhantomData,
        }
    }
    pub unsafe fn from_raw(data: *mut T, size: usize) -> Self {
        Self {
            data,
            size,
            lease: core::marker::PhantomData,
        }
    }
}
impl<T> ZipInput for ZipSpanMut<'_, T> {
    type Element = T;
    type SizeKind = KnownZipSize;
    fn Data(&self) -> *const T {
        self.data
    }
    fn Size(&self) -> usize {
        self.size
    }
}
// Source: src/core/SkZip.h:159-160
// Source: src/core/SkZip.h:162-176
// Arbitrary input-list normalization and first-sized compile-time dispatch follow.
// Source: src/core/SkZip.h:129-176
pub struct ZipArgNil;
pub struct ZipArgCons<H, Tail> {
    pub head: H,
    pub tail: Tail,
}
impl<H, Tail> ZipArgCons<H, Tail> {
    pub fn new(head: H, tail: Tail) -> Self {
        Self { head, tail }
    }
}
pub trait ZipArguments {
    type Pointers: ZipPointers;
    fn pointers(&self) -> Self::Pointers;
    fn visit_sizes(&self, visit: &mut dyn FnMut(usize));
}
impl ZipArguments for ZipArgNil {
    type Pointers = ZipNil;
    fn pointers(&self) -> ZipNil {
        ZipNil
    }
    fn visit_sizes(&self, _visit: &mut dyn FnMut(usize)) {}
}
impl<H: ZipInput, Tail: ZipArguments> ZipArguments for ZipArgCons<H, Tail> {
    type Pointers = ZipCons<H::Element, Tail::Pointers>;
    fn pointers(&self) -> Self::Pointers {
        ZipCons {
            head: self.head.Data(),
            tail: self.tail.pointers(),
        }
    }
    fn visit_sizes(&self, visit: &mut dyn FnMut(usize)) {
        visit(self.head.Size());
        self.tail.visit_sizes(visit);
    }
}
// Source: src/core/SkZip.h:162-176
// No PickOneZipSize for an empty/all-pointer list: native PickOneSize has no such base.
pub trait PickOneZipSize: ZipArguments {
    fn first_size(&self) -> usize;
}
pub trait PickZipHead<H: ZipInput, Tail: ZipArguments> {
    fn first(head: &H, tail: &Tail) -> usize;
}
impl<H: ZipInput, Tail: ZipArguments> PickZipHead<H, Tail> for KnownZipSize {
    fn first(head: &H, _tail: &Tail) -> usize {
        head.Size()
    }
}
impl<H: ZipInput, Tail: PickOneZipSize> PickZipHead<H, Tail> for UnknownZipSize {
    fn first(_head: &H, tail: &Tail) -> usize {
        tail.first_size()
    }
}
impl<H: ZipInput, Tail: ZipArguments> PickOneZipSize for ZipArgCons<H, Tail>
where
    H::SizeKind: PickZipHead<H, Tail>,
{
    fn first_size(&self) -> usize {
        H::SizeKind::first(&self.head, &self.tail)
    }
}
// Source: src/core/SkZip.h:179-199
pub fn SkMakeZipList<Args: PickOneZipSize>(args: Args) -> SkZipVariadic<Args::Pointers> {
    let size = args.first_size();
    #[cfg(sk_debug)]
    {
        let mut minSize = usize::MAX;
        let mut maxSize = 0usize;
        args.visit_sizes(&mut |s| {
            if s != usize::MAX {
                minSize = core::cmp::min(minSize, s);
                maxSize = core::cmp::max(maxSize, s);
            }
        });
        crate::SkASSERT!(@source b"src/core/SkZip.h\0".as_ptr(),196; @condition b"minSize == maxSize\0".as_ptr(); (minSize)==(maxSize));
    }
    SkZipVariadic::from_pointers(size, args.pointers())
}
// Source: src/core/SkZip.h:207-209
pub fn SkMakeZipVariadic<Args: PickOneZipSize>(args: Args) -> SkZipVariadic<Args::Pointers> {
    SkMakeZipList(args)
}
// Source: src/core/SkZip.h:179-199
// Two-column compatibility facade of the shared arbitrary-arity factory; no shortest-zip rule.
// Returned SkZip is nonowning raw metadata. Every dereference still requires the
// original storage/span lifetime and aliasing lease from its unsafe accessor.
pub fn SkMakeZipFrom<L: ZipInput, R: ZipInput>(left: L, right: R) -> SkZip<L::Element, R::Element>
where
    ZipArgCons<L, ZipArgCons<R, ZipArgNil>>: PickOneZipSize,
{
    let args = ZipArgCons::new(left, ZipArgCons::new(right, ZipArgNil));
    let size = args.first_size();
    let left = args.head.Data();
    let right = args.tail.head.Data();
    SkZip::from_raw(size, left, right)
}
// Source: src/core/SkZip.h:204-205
// Source: src/core/SkZip.h:207-209
pub fn SkMakeZipGeneric<L: ZipInput, R: ZipInput>(
    left: L,
    right: R,
) -> SkZip<L::Element, R::Element>
where
    ZipArgCons<L, ZipArgCons<R, ZipArgNil>>: PickOneZipSize,
{
    SkMakeZipFrom(left, right)
}
// Compatibility normalization for the pre-existing raw-metadata pair APIs.
// These keep their original element types and unsafe dereference permissions;
// the newer ZipInput borrow API expresses const qualification explicitly.
struct LegacyZipColumn<T, K> {
    data: *const T,
    size: usize,
    kind: core::marker::PhantomData<K>,
}
impl<T, K> ZipInput for LegacyZipColumn<T, K> {
    type Element = T;
    type SizeKind = K;
    fn Data(&self) -> *const T {
        self.data
    }
    fn Size(&self) -> usize {
        self.size
    }
}
fn legacy_sized<T>(data: *const T, size: usize) -> LegacyZipColumn<T, KnownZipSize> {
    LegacyZipColumn {
        data,
        size,
        kind: core::marker::PhantomData,
    }
}
fn legacy_raw<T>(data: *const T) -> LegacyZipColumn<T, UnknownZipSize> {
    LegacyZipColumn {
        data,
        size: usize::MAX,
        kind: core::marker::PhantomData,
    }
}
// Source: src/core/SkZip.h:136-139
// Source: src/core/SkZip.h:148-151
// Source: src/core/SkZip.h:162-176
// Source: src/core/SkZip.h:179-199
// Rust slices cover native array/SkSpan/contiguous-container deduction. A raw
// pointer has SIZE_MAX unknown length; the first sized column supplies the count.
pub fn SkMakeZipRawLeft<A, B>(a: *const A, b: &[B]) -> SkZip<A, B> {
    SkMakeZipFrom(legacy_raw(a), legacy_sized(b.as_ptr(), b.len()))
}
pub fn SkMakeZipRawRight<A, B>(a: &[A], b: *const B) -> SkZip<A, B> {
    SkMakeZipFrom(legacy_sized(a.as_ptr(), a.len()), legacy_raw(b))
}

// Source: src/core/SkZip.h:183-183
// Source: src/core/SkZip.h:196-196
// Source: src/core/SkZip.h:199-199
pub fn SkMakeZip<A, B>(a: &[A], b: &[B]) -> SkZip<A, B> {
    SkMakeZipFrom(
        legacy_sized(a.as_ptr(), a.len()),
        legacy_sized(b.as_ptr(), b.len()),
    )
}
