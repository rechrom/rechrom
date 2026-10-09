// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! runtime_enabled_features.json5:1615-1616 stable CSSDynamicRangeLimit;
//! 1641-1643 experimental CSSGridLanesLayout. Interest delays have no gate.
#![allow(non_snake_case)]
pub(crate) fn IsExposed(id: foundation::CSSPropertyID) -> bool {
    !matches!(
        id,
        foundation::CSSPropertyID::kFlowTolerance
            | foundation::CSSPropertyID::kGridLanes
            | foundation::CSSPropertyID::kGridLanesDirection
            | foundation::CSSPropertyID::kGridLanesPack
    )
}
