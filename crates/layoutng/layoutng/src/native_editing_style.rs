//! Scalar reads of the current resident LayoutObject/ComputedStyle owner.
//! Unlike fragment geometry, GetLayoutObject/style queries remain valid after
//! style/attachment has dirtied layout and before a new fragment is produced.
use crate::layout_engine::LayoutEngine;
use foundation::{EUserDrag, EUserModify, EUserSelect};
pub struct EditingNodeStyleFacts {
    pub selectable: bool,
    pub visible_to_hit_testing: bool,
    pub used_value_user_select: EUserSelect,
    pub user_drag: EUserDrag,
}
impl LayoutEngine {
    pub fn HasLayoutObjectForEditing(&self, id: u64) -> bool {
        self.GetLayoutTree()
            .and_then(|tree| tree.OutputNode(id))
            .is_some()
    }
    pub fn EditingNodeStyleFacts(&self, id: u64) -> Option<EditingNodeStyleFacts> {
        let _scope = self.NativeScrollPassScope();
        let object = self.GetLayoutTree()?.OutputNode(id)?;
        let style = object.StyleRef();
        Some(EditingNodeStyleFacts {
            selectable: object.IsSelectable(),
            visible_to_hit_testing: object.VisibleToHitTesting(),
            used_value_user_select: if style.UsedUserModify() != EUserModify::kReadOnly {
                EUserSelect::kText
            } else {
                style.UsedUserSelect()
            },
            user_drag: style.UserDrag(),
        })
    }
}
