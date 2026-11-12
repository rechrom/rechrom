//! Iteration over resident `FragmentData` with the corresponding LayoutNG
//! fragment or inline cursor when one exists.
#![allow(non_snake_case)]

use std::marker::PhantomData;

use foundation::DynamicTo;
use layoutng::internal::{layout_box::LayoutBox, layout_object::LayoutObject};

use crate::{
    fragment_data::{FragmentData, FragmentDataList},
    inline_cursor::InlineCursor,
    physical_box_fragment::PhysicalBoxFragment,
};

const K_NOT_FOUND: u32 = u32::MAX;

/// Source: `fragment_data_iterator.h:20-77`.
pub struct FragmentDataIterator<'a> {
    fragment_head_: Option<&'a FragmentDataList>,
    idx_: u32,
}

impl<'a> FragmentDataIterator<'a> {
    pub fn new(object: &'a LayoutObject) -> Self {
        Self::from_list(object.FragmentList())
    }

    fn from_list(fragment_head: &'a FragmentDataList) -> Self {
        Self {
            fragment_head_: Some(fragment_head),
            idx_: 0,
        }
    }

    pub fn empty() -> Self {
        Self {
            fragment_head_: None,
            idx_: K_NOT_FOUND,
        }
    }

    pub fn GetFragmentData(&self) -> Option<&'a FragmentData> {
        if self.IsDone() {
            return None;
        }
        Some(self.fragment_head_?.at(self.idx_))
    }

    pub fn Advance(&mut self) -> bool {
        if self.IsDone() {
            return false;
        }
        self.idx_ += 1;
        if self.idx_ >= self.fragment_head_.expect("active fragment list").size() {
            self.idx_ = K_NOT_FOUND;
            return false;
        }
        true
    }

    pub fn IsDone(&self) -> bool {
        self.idx_ == K_NOT_FOUND
    }
}

impl<'a> Iterator for FragmentDataIterator<'a> {
    type Item = &'a FragmentData;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.GetFragmentData()?;
        self.Advance();
        Some(data)
    }
}

/// Source: `fragment_data_iterator.h:79-87`.
///
/// The mutable Rust constructor takes an exclusive owner reference; the raw
/// list pointer then lets the standard `Iterator` API yield disjoint entries
/// with the source iterator's one-way index progression.
pub struct MutableFragmentDataIterator<'a> {
    fragment_head_: *mut FragmentDataList,
    idx_: u32,
    _owner: PhantomData<&'a mut FragmentDataList>,
}

impl<'a> MutableFragmentDataIterator<'a> {
    pub fn new(object: &'a mut LayoutObject) -> Self {
        Self {
            fragment_head_: object.FragmentList() as *const FragmentDataList
                as *mut FragmentDataList,
            idx_: 0,
            _owner: PhantomData,
        }
    }

    fn from_list(fragment_head: &'a mut FragmentDataList) -> Self {
        Self {
            fragment_head_: fragment_head,
            idx_: 0,
            _owner: PhantomData,
        }
    }

    pub fn GetFragmentData(&mut self) -> Option<&mut FragmentData> {
        if self.IsDone() {
            return None;
        }
        Some(unsafe { &mut *self.fragment_head_ }.at_mut(self.idx_))
    }

    pub fn Advance(&mut self) -> bool {
        if self.IsDone() {
            return false;
        }
        self.idx_ += 1;
        if self.idx_ >= unsafe { &*self.fragment_head_ }.size() {
            self.idx_ = K_NOT_FOUND;
            return false;
        }
        true
    }

    pub fn IsDone(&self) -> bool {
        self.idx_ == K_NOT_FOUND
    }
}

impl<'a> Iterator for MutableFragmentDataIterator<'a> {
    type Item = &'a mut FragmentData;

    fn next(&mut self) -> Option<Self::Item> {
        if self.IsDone() {
            return None;
        }
        let index = self.idx_;
        self.Advance();
        let data = unsafe { &mut *self.fragment_head_ }.at_mut(index) as *mut FragmentData;
        Some(unsafe { &mut *data })
    }
}

/// Source: `fragment_data_iterator.h:89-115` and
/// `fragment_data_iterator.cc:14-82`.
pub struct AccompaniedFragmentIterator<'a> {
    fragments_: FragmentDataIterator<'a>,
    cursor_: Option<InlineCursor>,
    layout_box_: *const LayoutBox,
}

impl<'a> AccompaniedFragmentIterator<'a> {
    pub fn new(object: &'a LayoutObject) -> Self {
        let mut result = Self {
            fragments_: FragmentDataIterator::new(object),
            cursor_: None,
            layout_box_: DynamicTo::<LayoutBox>(object as *const LayoutObject),
        };
        if !result.layout_box_.is_null() {
            return result;
        }
        if object.IsInLayoutNGInlineFormattingContext() {
            let mut cursor = InlineCursor::default();
            cursor.MoveToIncludingCulledInline(object);
            result.cursor_ = Some(cursor);
        }
        result
    }

    pub fn GetFragmentData(&self) -> Option<&'a FragmentData> {
        self.fragments_.GetFragmentData()
    }

    pub fn IsDone(&self) -> bool {
        self.fragments_.IsDone()
    }

    pub fn Cursor(&self) -> Option<&InlineCursor> {
        self.cursor_.as_ref()
    }

    pub fn GetPhysicalBoxFragment(&self) -> Option<&PhysicalBoxFragment> {
        if self.layout_box_.is_null() || self.IsDone() {
            return None;
        }
        let fragment =
            unsafe { &*self.layout_box_ }.GetPhysicalFragment(self.fragments_.idx_ as usize);
        unsafe { fragment.as_ref() }
    }

    pub fn Advance(&mut self) -> bool {
        if self.IsDone() {
            return false;
        }

        if let Some(cursor) = &mut self.cursor_ {
            let fragmentainer_index = cursor.ContainerFragmentIndex();
            cursor.MoveToNextForSameLayoutObject();
            if cursor.IsNotNull() && fragmentainer_index == cursor.ContainerFragmentIndex() {
                return true;
            }
        }

        #[cfg(debug_assertions)]
        let previous_idx = self.fragments_.idx_;
        self.fragments_.Advance();
        if self.IsDone() {
            #[cfg(debug_assertions)]
            if let Some(cursor) = &self.cursor_ {
                debug_assert!(!cursor.IsNotNull());
            } else if !self.layout_box_.is_null() {
                debug_assert_eq!(
                    unsafe { &*self.layout_box_ }.PhysicalFragmentCount(),
                    previous_idx as usize + 1
                );
            }
            self.layout_box_ = std::ptr::null();
            return false;
        }

        #[cfg(debug_assertions)]
        if !self.layout_box_.is_null() {
            debug_assert!(
                unsafe { &*self.layout_box_ }.PhysicalFragmentCount()
                    > self.fragments_.idx_ as usize
            );
        } else if let Some(cursor) = &self.cursor_ {
            debug_assert!(cursor.IsNotNull());
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::{LayoutUnit, PhysicalOffset};

    fn offset(left: i32, top: i32) -> PhysicalOffset {
        PhysicalOffset::new(LayoutUnit::from_signed(left), LayoutUnit::from_signed(top))
    }

    fn list_with_three_fragments() -> FragmentDataList {
        let mut list = FragmentDataList::default();
        #[cfg(debug_assertions)]
        list.SetIsFirst();
        list.front_mut().SetPaintOffset(&offset(1, 2));
        list.AppendNewFragment().SetPaintOffset(&offset(3, 4));
        list.AppendNewFragment().SetPaintOffset(&offset(5, 6));
        list
    }

    #[test]
    fn immutable_iterator_visits_the_inline_and_additional_entries() {
        let list = list_with_three_fragments();
        let offsets: Vec<_> = FragmentDataIterator::from_list(&list)
            .map(FragmentData::PaintOffset)
            .collect();
        assert_eq!(offsets.len(), 3);
        assert_eq!(offsets[0], offset(1, 2));
        assert_eq!(offsets[2], offset(5, 6));
    }

    #[test]
    fn nullable_base_iterator_starts_and_stays_done() {
        let mut iterator = FragmentDataIterator::empty();
        assert!(iterator.IsDone());
        assert!(iterator.GetFragmentData().is_none());
        assert!(!iterator.Advance());
        assert!(iterator.next().is_none());
    }

    #[test]
    fn mutable_iterator_updates_each_entry_once() {
        let mut list = list_with_three_fragments();
        for (index, fragment) in MutableFragmentDataIterator::from_list(&mut list).enumerate() {
            fragment.SetPaintOffset(&offset(index as i32, 9));
        }
        assert_eq!(list.at(0).PaintOffset(), offset(0, 9));
        assert_eq!(list.at(2).PaintOffset(), offset(2, 9));
    }
}
