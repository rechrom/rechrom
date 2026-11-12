// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source commands/smart_replace.h:36; platform source selection is supplied
// through the existing embedding policy.
pub fn IsCharacterSmartReplaceExempt(c: u32, previous: bool, mac: bool) -> bool {
    if mac {
        super::smart_replace_cf::IsCharacterSmartReplaceExempt(c, previous)
    } else {
        super::smart_replace_icu::IsCharacterSmartReplaceExempt(c, previous)
    }
}
