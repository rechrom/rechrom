// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Compatibility exports; implementations live with their Chromium source owner.
pub use super::composite_edit_command::{
    CanRebalance, RebalanceWhitespaceAtWithContext as RebalanceWhitespaceAt,
    RebalanceWhitespaceOnTextSubstring, WhitespaceRebalanceContext,
};
pub use crate::core::editing::editing_utilities::{
    IsCollapsibleWhitespace, IsWhitespace, StringWithRebalancedWhitespace,
};
