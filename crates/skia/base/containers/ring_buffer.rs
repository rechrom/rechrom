// Source: base/containers/ring_buffer.h: RingBuffer<T,kSize> original inline methods.
// T default construction is actual C++ std::array construction, never blanket zero initialization.
// Native primitive uninitialized/default and nontrivial assignment/destruction domains are not widened into this Copy-valued specialization.
pub struct RingBuffer<T: Copy, const N: usize> {
    current_index_: usize,
    buffer_: [T; N],
}
impl<T: Copy + Default, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            buffer_: core::array::from_fn(|_| T::default()),
            current_index_: 0,
        }
    }
}
impl<T: Copy, const N: usize> RingBuffer<T, N> {
    pub const fn BufferSize(&self) -> usize {
        N
    }
    pub fn CurrentIndex(&self) -> usize {
        self.current_index_
    }
    pub fn IsFilledIndex(&self, n: usize) -> bool {
        self.IsFilledIndexByBufferIndex(self.BufferIndex(n))
    }
    pub unsafe fn ReadBuffer(&self, n: usize) -> &T {
        let index = self.BufferIndex(n);
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/containers/ring_buffer.h\0".as_ptr().cast(),49); @condition b"IsFilledIndexByBufferIndex(buffer_index)\0".as_ptr();self.IsFilledIndexByBufferIndex(index));
        self.buffer_.get_unchecked(index)
    }
    pub unsafe fn MutableReadBuffer(&mut self, n: usize) -> *mut T {
        let index = self.BufferIndex(n);
        crate::CHECK!(@location crate::base::location::Location::CurrentWithoutFunctionName(b"base/containers/ring_buffer.h\0".as_ptr().cast(),55); @condition b"IsFilledIndexByBufferIndex(buffer_index)\0".as_ptr();self.IsFilledIndexByBufferIndex(index));
        self.buffer_.as_mut_ptr().add(index)
    }
    pub fn SaveToBuffer(&mut self, value: T) {
        let index = self.BufferIndex(0);
        self.buffer_[index] = value;
        self.current_index_ = self.current_index_.wrapping_add(1);
    }
    pub fn Clear(&mut self) {
        self.current_index_ = 0;
    }
    pub fn Begin(&self) -> Iterator<'_, T, N> {
        Iterator::new(
            self,
            if self.current_index_ < N {
                N - self.current_index_
            } else {
                0
            },
        )
    }
    pub fn End(&self) -> Iterator<'_, T, N> {
        Iterator::new(self, N.wrapping_sub(1))
    }
    fn BufferIndex(&self, n: usize) -> usize {
        self.current_index_.wrapping_add(n) % N
    }
    fn IsFilledIndexByBufferIndex(&self, index: usize) -> bool {
        index < self.current_index_
    }
}
pub struct Iterator<'a, T: Copy, const N: usize> {
    buffer_: &'a RingBuffer<T, N>,
    index_: usize,
    out_of_range_: bool,
}
impl<'a, T: Copy, const N: usize> Iterator<'a, T, N> {
    fn new(buffer: &'a RingBuffer<T, N>, index: usize) -> Self {
        Self {
            buffer_: buffer,
            index_: index,
            out_of_range_: false,
        }
    }
    pub fn index(&self) -> usize {
        self.index_
    }
    pub unsafe fn get(&self) -> &'a T {
        self.buffer_.ReadBuffer(self.index_)
    }
    pub fn increment(&mut self) -> &mut Self {
        self.index_ = self.index_.wrapping_add(1);
        if self.index_ == N {
            self.out_of_range_ = true;
        }
        self
    }
    pub fn decrement(&mut self) -> &mut Self {
        if self.index_ == 0 {
            self.out_of_range_ = true;
        }
        self.index_ = self.index_.wrapping_sub(1);
        self
    }
    pub fn is_valid(&self) -> bool {
        !self.out_of_range_ && self.buffer_.IsFilledIndex(self.index_)
    }
}
