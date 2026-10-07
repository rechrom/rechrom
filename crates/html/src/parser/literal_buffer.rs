#![allow(non_snake_case)]

use foundation::{AtomicString, BlinkString};
use std::ops::Index;

// cpp: html/parser/literal_buffer.h:41-79
// Rust's const generic INLINE corresponds to BUFFER_INLINE_CAPACITY in the
// normal build. The ANNOTATE_CONTIGUOUS_CONTAINER configuration uses INLINE=0.
pub struct LiteralBufferBase<T: Copy + Default, const INLINE: usize> {
    inline_storage: [T; INLINE],
    inline_len: usize,
    heap_storage: Option<Vec<T>>,
    // C++ end_of_storage_ records the requested element capacity even when
    // PartitionAllocator reserves a larger quantized allocation underneath.
    heap_capacity: usize,
}

impl<T: Copy + Default, const INLINE: usize> Default for LiteralBufferBase<T, INLINE> {
    fn default() -> Self {
        Self {
            inline_storage: std::array::from_fn(|_| T::default()),
            inline_len: 0,
            heap_storage: None,
            heap_capacity: 0,
        }
    }
}

impl<T: Copy + Default, const INLINE: usize> Clone for LiteralBufferBase<T, INLINE> {
    fn clone(&self) -> Self {
        let mut copy = Self::default();
        copy.Copy(self);
        copy
    }
}

impl<T: Copy + Default, const INLINE: usize> LiteralBufferBase<T, INLINE> {
    pub fn data(&self) -> *const T {
        self.as_slice().as_ptr()
    }
    pub fn size(&self) -> u32 {
        u32::try_from(self.as_slice().len()).expect("LiteralBufferBase size overflow")
    }
    pub fn begin(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }
    pub fn end(&self) -> std::slice::Iter<'_, T> {
        self.as_slice()[self.as_slice().len()..].iter()
    }
    pub fn IsEmpty(&self) -> bool {
        self.as_slice().is_empty()
    }
    pub fn as_slice(&self) -> &[T] {
        match &self.heap_storage {
            Some(heap) => heap.as_slice(),
            None => &self.inline_storage[..self.inline_len],
        }
    }

    // cpp: html/parser/literal_buffer.h:82-84
    pub fn ClearImpl(&mut self) {
        match &mut self.heap_storage {
            Some(heap) => heap.clear(),
            None => self.inline_len = 0,
        }
    }

    // cpp: html/parser/literal_buffer.h:86-96
    pub fn AddCharImpl(&mut self, value: T) {
        if self.as_slice().len() == self.capacity() {
            self.Grow(0);
        }
        match &mut self.heap_storage {
            Some(heap) => heap.push(value),
            None => {
                self.inline_storage[self.inline_len] = value;
                self.inline_len += 1;
            }
        }
    }

    // cpp: html/parser/literal_buffer.h:98-112
    pub fn AppendLiteralImpl<U: Copy + Default + Into<T>, const OTHER: usize>(
        &mut self,
        value: &LiteralBufferBase<U, OTHER>,
    ) {
        let count = value.as_slice().len();
        let new_size = self
            .as_slice()
            .len()
            .checked_add(count)
            .expect("LiteralBufferBase size overflow");
        let _: u32 = new_size
            .try_into()
            .expect("LiteralBufferBase size overflow");
        if self.capacity() < new_size {
            self.Grow(new_size);
        }
        for character in value.as_slice() {
            self.AddCharImpl((*character).into());
        }
    }

    // cpp: html/parser/literal_buffer.h:114-134
    pub fn Copy<const OTHER: usize>(&mut self, other: &LiteralBufferBase<T, OTHER>) {
        let other_size = other.as_slice().len();
        if self.capacity() < other_size {
            let mut new_heap = Vec::with_capacity(other_size);
            new_heap.extend_from_slice(other.as_slice());
            self.heap_storage = Some(new_heap);
            self.heap_capacity = other_size;
            self.inline_len = 0;
            return;
        }
        self.ClearImpl();
        match &mut self.heap_storage {
            Some(heap) => heap.extend_from_slice(other.as_slice()),
            None => {
                self.inline_storage[..other_size].copy_from_slice(other.as_slice());
                self.inline_len = other_size;
            }
        }
    }

    // cpp: html/parser/literal_buffer.h:136-161
    pub fn Move(&mut self, other: &mut Self) {
        debug_assert!(!std::ptr::eq(self, other));
        if !other.is_stored_inline() {
            self.heap_storage = other.heap_storage.take();
            self.heap_capacity = std::mem::take(&mut other.heap_capacity);
            self.inline_len = 0;
            other.inline_len = 0;
        } else {
            debug_assert!(self.capacity() >= other.as_slice().len());
            self.Copy(other);
        }
    }

    // cpp: html/parser/literal_buffer.h:164-174
    pub fn capacity(&self) -> usize {
        if self.heap_storage.is_some() {
            self.heap_capacity
        } else {
            INLINE
        }
    }
    pub fn is_stored_inline(&self) -> bool {
        self.heap_storage.is_none()
    }

    // cpp: html/parser/literal_buffer.h:176-184
    fn RoundUpToPowerOfTwo(value: usize) -> usize {
        debug_assert!(value <= 1usize << (usize::BITS - 1));
        value
            .checked_next_power_of_two()
            .expect("LiteralBufferBase capacity overflow")
    }

    // cpp: html/parser/literal_buffer.h:186-210
    fn Grow(&mut self, min_capacity: usize) {
        let in_use = self.as_slice().len();
        let doubled = self
            .capacity()
            .checked_mul(2)
            .expect("LiteralBufferBase capacity overflow");
        let new_capacity = Self::RoundUpToPowerOfTwo(min_capacity.max(doubled));
        let mut new_heap = Vec::with_capacity(new_capacity);
        new_heap.extend_from_slice(self.as_slice());
        debug_assert!(in_use <= new_heap.capacity());
        self.heap_storage = Some(new_heap);
        self.heap_capacity = new_capacity;
        self.inline_len = 0;
    }
}

impl<T: Copy + Default, const INLINE: usize> Index<usize> for LiteralBufferBase<T, INLINE> {
    type Output = T;
    // cpp: html/parser/literal_buffer.h:73-77
    fn index(&self, index: usize) -> &Self::Output {
        assert!(self.as_slice().len() > index);
        &self.as_slice()[index]
    }
}

// cpp: html/parser/literal_buffer.h:229-257
pub struct LCharLiteralBuffer<const INLINE: usize> {
    base: LiteralBufferBase<u8, INLINE>,
}

impl<const INLINE: usize> Default for LCharLiteralBuffer<INLINE> {
    fn default() -> Self {
        Self {
            base: LiteralBufferBase::default(),
        }
    }
}
impl<const INLINE: usize> Clone for LCharLiteralBuffer<INLINE> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
        }
    }
}
impl<const INLINE: usize> LCharLiteralBuffer<INLINE> {
    pub fn data(&self) -> *const u8 {
        self.base.data()
    }
    pub fn size(&self) -> u32 {
        self.base.size()
    }
    pub fn as_slice(&self) -> &[u8] {
        self.base.as_slice()
    }
    pub fn IsEmpty(&self) -> bool {
        self.base.IsEmpty()
    }
    pub fn clear(&mut self) {
        self.base.ClearImpl();
    }
    pub fn AddChar(&mut self, value: u8) {
        self.base.AddCharImpl(value);
    }
    pub fn Copy<const OTHER: usize>(&mut self, other: &LCharLiteralBuffer<OTHER>) {
        if self.data() != other.data() {
            self.base.Copy(&other.base);
        }
    }
    pub fn Move(&mut self, other: &mut Self) {
        if !std::ptr::eq(self, other) {
            self.base.Move(&mut other.base);
        }
    }
    pub fn AsString(&self) -> BlinkString {
        BlinkString::from_latin1(self.as_slice())
    }
}

// cpp: html/parser/literal_buffer.h:260-338
pub struct UCharLiteralBuffer<const INLINE: usize> {
    base: LiteralBufferBase<u16, INLINE>,
    bitwise_or_all_chars_: u16,
}

impl<const INLINE: usize> Default for UCharLiteralBuffer<INLINE> {
    fn default() -> Self {
        Self {
            base: LiteralBufferBase::default(),
            bitwise_or_all_chars_: 0,
        }
    }
}
impl<const INLINE: usize> Clone for UCharLiteralBuffer<INLINE> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            bitwise_or_all_chars_: self.bitwise_or_all_chars_,
        }
    }
}
impl<const INLINE: usize> UCharLiteralBuffer<INLINE> {
    pub fn data(&self) -> *const u16 {
        self.base.data()
    }
    pub fn size(&self) -> u32 {
        self.base.size()
    }
    pub fn as_slice(&self) -> &[u16] {
        self.base.as_slice()
    }
    pub fn IsEmpty(&self) -> bool {
        self.base.IsEmpty()
    }
    pub fn Copy<const OTHER: usize>(&mut self, other: &UCharLiteralBuffer<OTHER>) {
        if self.data() == other.data() {
            return;
        }
        self.base.Copy(&other.base);
        self.bitwise_or_all_chars_ = other.bitwise_or_all_chars_;
    }
    pub fn Move(&mut self, other: &mut Self) {
        if std::ptr::eq(self, other) {
            return;
        }
        let other_bits = other.bitwise_or_all_chars_;
        self.base.Move(&mut other.base);
        self.bitwise_or_all_chars_ = other_bits;
    }
    pub fn clear(&mut self) {
        self.base.ClearImpl();
        self.bitwise_or_all_chars_ = 0;
    }
    pub fn AddChar(&mut self, value: u16) {
        self.base.AddCharImpl(value);
        self.bitwise_or_all_chars_ |= value;
    }
    pub fn AppendLiteral<const OTHER: usize>(&mut self, value: &LCharLiteralBuffer<OTHER>) {
        self.base.AppendLiteralImpl(&value.base);
    }
    pub fn AsString(&self) -> BlinkString {
        if self.Is8Bit() {
            BlinkString::Make8BitFrom16BitSource(self.as_slice())
        } else {
            BlinkString::from_utf16(self.as_slice())
        }
    }
    pub fn AsAtomicString(&self) -> AtomicString {
        AtomicString::from_utf16(self.as_slice())
    }
    pub fn Is8Bit(&self) -> bool {
        (self.bitwise_or_all_chars_ & !0xff) == 0
    }
}
