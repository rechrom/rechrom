//! Non-default timeline-trigger metadata construction.
//!
//! Included by `layout_node_metadata` at the reciprocal assembly boundary.
//! Empty trigger lists remain an inline no-op; name validation, UTF-16
//! conversion, atom allocation, and list population own this invalidation
//! sampling frame.

use super::*;

#[inline(never)]
pub(super) fn PopulateTimelineTriggerNames(
    element: &mut Element,
    input: &NativeNodeConstructionData,
) -> Result<(), ElementMetadataError> {
    let style = input
        .style
        .extended
        .as_ref()
        .expect("nonempty trigger list");
    debug_assert!(!style.timeline_trigger_names.is_empty());
    for value in &style.timeline_trigger_names {
        let Some(name) = value else {
            element.timeline_trigger_names_.push(Member::default());
            continue;
        };
        if name.is_empty() {
            return Err(ElementMetadataError::InvalidInput(
                "Timeline trigger requires a nonempty UTF-8 name",
            ));
        }
        let units: Vec<u16> = name.encode_utf16().collect();
        let atom = AtomicString::from_utf16(&units);
        let scoped_name = MakeGarbageCollected(ScopedCSSName::new(&atom, std::ptr::null()));
        element
            .timeline_trigger_names_
            .push(Member::from_ptr(scoped_name));
    }
    Ok(())
}
