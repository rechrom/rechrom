//! SVG text descendant index maintenance at the assembly boundary.

use super::{
    GCedHeapHashSet, LayoutBlock, LayoutBox, LayoutView, MakeGarbageCollected, Member,
    SVGTextDescendantsMap, WeakMember,
};

impl LayoutView {
    // cpp: layoutng/internal/layout_view.h:249-256
    pub fn SetContainsNonScalingStroke(&mut self) {
        self.CheckIsNotDestroyed();
        self.contains_non_scaling_stroke_ = true;
    }

    pub fn ContainsNonScalingStroke(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.contains_non_scaling_stroke_
    }

    // cpp: layoutng/internal/layout_view.h:323-328
    pub fn SvgTextDescendantsMap(&mut self) -> &mut SVGTextDescendantsMap {
        self.CheckIsNotDestroyed();
        &mut self.svg_text_descendants_
    }
}

impl LayoutBlock {
    // cpp: layoutng/internal/layout_block.h:142-142
    // cpp: layoutng/internal/layout_view_data.cc:163-173
    #[inline(never)]
    pub fn AddSvgTextDescendant(&mut self, svg_text: &mut LayoutBox) {
        self.CheckIsNotDestroyed();
        assert!(svg_text.IsSVGText());
        let view = unsafe { &mut *self.View() };
        let key = WeakMember::from_ptr(self as *mut LayoutBlock);
        let map = view.SvgTextDescendantsMap();
        let is_new_entry = map.insert(key.clone(), Member::default());
        if is_new_entry {
            *map.get_mut(&key).unwrap() = Member::from_ptr(MakeGarbageCollected(
                GCedHeapHashSet::<Member<LayoutBox>>::default(),
            ));
        }
        let descendants = unsafe { &mut *map.get(&key).unwrap().Get() };
        descendants.insert(Member::from_ptr(svg_text as *mut LayoutBox));
        self.SetHasSVGTextDescendants(true);
    }

    // cpp: layoutng/internal/layout_block.h:143-143
    // cpp: layoutng/internal/layout_view_data.cc:175-188
    #[inline(never)]
    pub fn RemoveSvgTextDescendant(&mut self, svg_text: &mut LayoutBox) {
        self.CheckIsNotDestroyed();
        assert!(svg_text.IsSVGText());
        let view = unsafe { &mut *self.View() };
        let map = view.SvgTextDescendantsMap();
        let key = WeakMember::from_ptr(self as *mut LayoutBlock);
        let Some(descendants_member) = map.get(&key) else {
            return;
        };
        let descendants = unsafe { &mut *descendants_member.Get() };
        descendants.erase(&Member::from_ptr(svg_text as *mut LayoutBox));
        if descendants.empty() {
            map.remove(&key);
            self.SetHasSVGTextDescendants(false);
        }
    }
}
