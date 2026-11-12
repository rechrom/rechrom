// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/dom/focusgroup_flags.h:27-108; focusgroup_flags.cc:46-99,197-604,643-647.
//! Shared scalar focusgroup values. DOM owns parsing admission and retention.
#![allow(non_snake_case, non_upper_case_globals)]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusgroupBehavior {
    #[default]
    kNoBehavior,
    kToolbar,
    kTablist,
    kRadiogroup,
    kListbox,
    kMenu,
    kMenubar,
    kFeed,
    kGrid,
    kOptOut,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusgroupFlags(pub u16);
impl FocusgroupFlags {
    pub const kNone: Self = Self(0);
    pub const kInline: Self = Self(1);
    pub const kBlock: Self = Self(2);
    pub const kWrapInline: Self = Self(4);
    pub const kWrapBlock: Self = Self(8);
    pub const kRowFlow: Self = Self(16);
    pub const kColFlow: Self = Self(32);
    pub const kNoMemory: Self = Self(64);
    pub const kItemControls: Self = Self(128);
    pub fn Intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}
impl std::ops::BitOr for FocusgroupFlags {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitAnd for FocusgroupFlags {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl std::ops::BitOrAssign for FocusgroupFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusgroupData {
    pub behavior: FocusgroupBehavior,
    pub flags: FocusgroupFlags,
}
impl FocusgroupData {
    pub fn IsActualFocusgroup(self) -> bool {
        !matches!(
            self.behavior,
            FocusgroupBehavior::kNoBehavior | FocusgroupBehavior::kOptOut
        )
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusgroupFeatures {
    pub enabled: bool,
    pub v2: bool,
}

/// Pure value portion of ParseFocusgroup. Console diagnostics/use counters
/// remain effects of the DOM owner; this function makes no host callbacks.
pub fn ParseFocusgroup(input: &str, v2: bool) -> FocusgroupData {
    use FocusgroupBehavior as B;
    use FocusgroupFlags as F;
    let input = input.to_ascii_lowercase();
    let tokens: Vec<_> = input
        .split(|c| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c'))
        .filter(|token| !token.is_empty())
        .collect();
    if tokens.contains(&"none") {
        return FocusgroupData {
            behavior: B::kOptOut,
            flags: F::kNone,
        };
    }
    let behavior = tokens
        .iter()
        .find_map(|&token| {
            Some(match token {
                "toolbar" => B::kToolbar,
                "tablist" => B::kTablist,
                "radiogroup" => B::kRadiogroup,
                "listbox" => B::kListbox,
                "menu" => B::kMenu,
                "menubar" => B::kMenubar,
                "feed" if v2 => B::kFeed,
                "grid" => B::kGrid,
                _ => return None,
            })
        })
        .unwrap_or_default();
    if behavior == B::kNoBehavior || (behavior == B::kGrid && !v2) {
        return FocusgroupData::default();
    }
    let has = |token| tokens.contains(&token);
    let mut wrap = has("wrap");
    let mut nowrap = has("nowrap");
    if wrap && nowrap {
        wrap = false;
        nowrap = false;
    }
    let mut flags = if has("nomemory") {
        F::kNoMemory
    } else {
        F::kNone
    };
    if v2 && !has("noitemcontrols") && (has("itemcontrols") || behavior == B::kFeed) {
        flags |= F::kItemControls;
    }
    if behavior == B::kGrid {
        if wrap {
            flags |= F::kWrapInline | F::kWrapBlock;
        } else {
            if has("row-wrap") {
                flags |= F::kWrapInline;
            }
            if has("col-wrap") {
                flags |= F::kWrapBlock;
            }
        }
        if has("flow") {
            if flags.Intersects(F::kWrapInline | F::kWrapBlock) {
                return FocusgroupData::default();
            }
            flags |= F::kRowFlow | F::kColFlow;
        } else {
            if has("row-flow") {
                if flags.Intersects(F::kWrapInline) {
                    return FocusgroupData::default();
                }
                flags |= F::kRowFlow;
            }
            if has("col-flow") {
                if flags.Intersects(F::kWrapBlock) {
                    return FocusgroupData::default();
                }
                flags |= F::kColFlow;
            }
        }
    } else {
        let defaults = match behavior {
            B::kToolbar | B::kTablist | B::kMenubar => F::kInline,
            B::kListbox | B::kMenu | B::kFeed => F::kBlock,
            _ => F::kInline | F::kBlock,
        };
        if has("inline") || has("block") {
            if has("inline") {
                flags |= F::kInline;
            }
            if has("block") {
                flags |= F::kBlock;
            }
        } else {
            flags |= defaults;
        }
        let default_wrap = matches!(
            behavior,
            B::kTablist | B::kRadiogroup | B::kMenu | B::kMenubar
        );
        if !nowrap && (wrap || default_wrap) {
            if flags.Intersects(F::kInline) {
                flags |= F::kWrapInline;
            }
            if flags.Intersects(F::kBlock) {
                flags |= F::kWrapBlock;
            }
        }
    }
    FocusgroupData { behavior, flags }
}

/// Dynamic-class branch selected by the canonical DOM allocation owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeArrowKeyClass {
    Node,
    Element,
    HTMLElement,
    Input,
    Select,
    TextArea,
    Frame,
    Media,
}
