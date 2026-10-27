//! Non-default native trigger-scope projection.
//!
//! Included by `boundary::native_input` at the reciprocal assembly boundary.
//! The default empty scope is handled inline by the facade; only actual scope
//! construction and its input-sized name walk enter this invalidation frame.

use super::*;

fn NativeTriggerScopeAll() -> StyleTriggerScope {
    StyleNameScope::new(StyleNameScopeType::kAll, std::ptr::null(), std::ptr::null())
}

fn NativeTriggerScopeNames(names: &[AtomicString]) -> StyleTriggerScope {
    let mut entries = HeapVector::new();
    for name in names {
        let scoped = MakeGarbageCollected(ScopedCSSName::new(name, std::ptr::null()));
        entries.push_back(Member::from_ptr(scoped));
    }
    let list = MakeGarbageCollected(ScopedCSSNameList::new(entries));
    StyleNameScope::new(StyleNameScopeType::kNames, std::ptr::null(), list)
}

#[inline(never)]
pub(super) fn ApplyNonDefaultNativeTriggerScope(
    builder: &mut ComputedStyleBuilder,
    extra: &ExtendedStyle,
) {
    if extra.trigger_scope_all && !extra.trigger_scope_names.is_empty() {
        Invalid("trigger scope cannot combine all with names");
    }
    if extra.trigger_scope_all {
        builder.SetTriggerScopeOwned(NativeTriggerScopeAll());
        return;
    }
    debug_assert!(!extra.trigger_scope_names.is_empty());
    let mut names = Vec::with_capacity(extra.trigger_scope_names.len());
    for value in &extra.trigger_scope_names {
        if value.is_empty() {
            Invalid("trigger_scope_names");
        }
        names.push(AtomicString::from_utf16(
            &value.encode_utf16().collect::<Vec<_>>(),
        ));
    }
    builder.SetTriggerScopeOwned(NativeTriggerScopeNames(&names));
}
