//! Native DateTimeEditElement custom-style dispatch, element.cc:5104-5117 and
//! date_time_edit_element.cc:545-576. Only the live UA node/class aliases are
//! retained here; sizing and font algorithms remain with their source owners.
use super::{DOMNodeType, PersistentDocument};
use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle;
use std::rc::Rc;
#[derive(Clone, Copy, Debug)]
pub enum DateTimeCustomStyleError {
    FontPending,
    FontUnavailable,
    OwnerRetired,
    InvalidNativeTree,
}
pub trait DateTimeCustomStyleTextMetrics {
    fn TextWidth(
        &self,
        text: &[u16],
        original: &ComputedStyle,
    ) -> Result<f32, DateTimeCustomStyleError>;
}
/// A call-scoped child cursor. textContent remains lazy: source does not read
/// field text when it dispatches the native field's MaximumWidth virtual.
pub struct DateTimeStyleChild<'a> {
    document: &'a PersistentDocument,
    node: usize,
}
impl DateTimeStyleChild<'_> {
    pub fn NodeId(&self) -> u64 {
        self.document.Node(self.node).Id()
    }
    pub fn TextContent(&self) -> Rc<[u16]> {
        self.document.DescendantTextUTF16(self.node)
    }
}
pub trait DateTimeEditCustomStyle {
    fn CustomStyleForLayoutObject<'a>(
        &self,
        original: &ComputedStyle,
        children: &mut dyn Iterator<Item = DateTimeStyleChild<'a>>,
        metrics: &dyn DateTimeCustomStyleTextMetrics,
        device_pixel_ratio: f32,
    ) -> Result<Persistent<ComputedStyle>, DateTimeCustomStyleError>;
}
pub(super) struct DateTimeCustomStyleEntry {
    wrapper_node: u64,
    callback: Rc<dyn DateTimeEditCustomStyle>,
}
impl PersistentDocument {
    pub fn RegisterDateTimeEditCustomStyle(
        &mut self,
        node: u64,
        wrapper_node: u64,
        callback: Rc<dyn DateTimeEditCustomStyle>,
    ) {
        let index = self.FindNodeById(node).expect("native DateTimeEdit node");
        if self
            .date_time_custom_styles
            .get(&node)
            .is_some_and(|entry| entry.wrapper_node == wrapper_node)
        {
            return;
        }
        self.date_time_custom_styles.insert(
            node,
            DateTimeCustomStyleEntry {
                wrapper_node,
                callback,
            },
        );
        self.InvalidateNodeStyle(index);
    }
    pub fn HasDateTimeEditCustomStyle(&self, node: usize) -> bool {
        self.date_time_custom_styles
            .contains_key(&self.Node(node).Id())
    }
    pub fn DateTimeEditCustomStyle(
        &self,
        node: usize,
        original: &ComputedStyle,
        metrics: &dyn DateTimeCustomStyleTextMetrics,
        device_pixel_ratio: f32,
    ) -> Result<Option<Persistent<ComputedStyle>>, DateTimeCustomStyleError> {
        let Some(entry) = self.date_time_custom_styles.get(&self.Node(node).Id()) else {
            return Ok(None);
        };
        let wrapper = self
            .FindNodeById(entry.wrapper_node)
            .ok_or(DateTimeCustomStyleError::InvalidNativeTree)?;
        let mut children = self
            .Node(wrapper)
            .Children()
            .iter()
            .copied()
            .filter(|&node| self.Node(node).Type() == DOMNodeType::kElement)
            .map(|node| DateTimeStyleChild {
                document: self,
                node,
            });
        entry
            .callback
            .CustomStyleForLayoutObject(original, &mut children, metrics, device_pixel_ratio)
            .map(Some)
    }
}
