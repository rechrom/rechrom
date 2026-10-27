use super::*;

// cpp: layoutng/internal/layout_box.h:536-600
// cpp: layoutng/internal/layout_box_fragment_data.cc:157-184,468-479
pub struct PhysicalFragmentList<'a> {
    layout_results_: &'a LayoutResultList,
}

pub struct PhysicalFragmentIterator<'a> {
    list: PhysicalFragmentList<'a>,
    index: usize,
}

impl<'a> PhysicalFragmentList<'a> {
    pub fn new(layout_results: &'a LayoutResultList) -> Self {
        Self {
            layout_results_: layout_results,
        }
    }

    pub fn Size(&self) -> usize {
        self.layout_results_.len()
    }

    pub fn IsEmpty(&self) -> bool {
        self.layout_results_.is_empty()
    }

    pub fn MayHaveFragmentItems(&self) -> bool {
        !self.IsEmpty() && self.front().IsInlineFormattingContext()
    }

    pub fn HasFragmentItems(&self) -> bool {
        self.MayHaveFragmentItems() && self.SlowHasFragmentItems()
    }

    #[inline(never)]
    pub fn SlowHasFragmentItems(&self) -> bool {
        self.into_iter().any(PhysicalBoxFragment::HasItems)
    }

    #[inline(never)]
    pub fn IndexOf(&self, fragment: &PhysicalBoxFragment) -> usize {
        for (index, result) in self.layout_results_.iter().enumerate() {
            if unsafe { &*result.Get() }.GetPhysicalFragment() as *const _
                == fragment as *const PhysicalBoxFragment as *const _
            {
                return index;
            }
        }
        usize::MAX
    }

    pub fn Contains(&self, fragment: &PhysicalBoxFragment) -> bool {
        self.IndexOf(fragment) != usize::MAX
    }

    pub fn front(&self) -> &PhysicalBoxFragment {
        self.at(0)
    }

    pub fn back(&self) -> &PhysicalBoxFragment {
        self.at(self.Size() - 1)
    }

    fn at(&self, index: usize) -> &PhysicalBoxFragment {
        let physical = unsafe { &*self.layout_results_[index].Get() }.GetPhysicalFragment();
        let box_fragment = To::<PhysicalBoxFragment>(physical as *const _);
        unsafe { &*box_fragment }
    }
}

impl<'a> IntoIterator for PhysicalFragmentList<'a> {
    type Item = &'a PhysicalBoxFragment;
    type IntoIter = PhysicalFragmentIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalFragmentIterator {
            list: self,
            index: 0,
        }
    }
}

impl<'a> IntoIterator for &PhysicalFragmentList<'a> {
    type Item = &'a PhysicalBoxFragment;
    type IntoIter = PhysicalFragmentIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PhysicalFragmentIterator {
            list: PhysicalFragmentList::new(self.layout_results_),
            index: 0,
        }
    }
}

impl<'a> Iterator for PhysicalFragmentIterator<'a> {
    type Item = &'a PhysicalBoxFragment;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.list.Size() {
            return None;
        }
        let index = self.index;
        self.index += 1;
        let physical = unsafe { &*self.list.layout_results_[index].Get() }.GetPhysicalFragment();
        let box_fragment = To::<PhysicalBoxFragment>(physical as *const _);
        Some(unsafe { &*box_fragment })
    }
}
