//! Immutable navigation/commit facts crossing the loader boundary.
//! Chromium public/web/web_navigation_type.h:36-45;
//! core/loader/frame_loader.cc:632-655,830-833;
//! web_navigation_params.h and document_loader.cc:602-605,657-662.
use crate::{FrameLoadType, HistoryItemSnapshot, RequestorOrigin};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebNavigationType {
    LinkClicked,
    FormSubmitted,
    BackForward,
    Reload,
    FormResubmittedBackForward,
    FormResubmittedReload,
    Restore,
    Other,
}
impl WebNavigationType {
    pub fn Determine(load: FrameLoadType, is_form_submission: bool, have_event: bool) -> Self {
        if is_form_submission {
            if load.IsReload() {
                return Self::FormResubmittedReload;
            }
            if load == FrameLoadType::BackForward {
                return Self::FormResubmittedBackForward;
            }
            return Self::FormSubmitted;
        }
        if have_event {
            return Self::LinkClicked;
        }
        if load.IsReload() {
            return Self::Reload;
        }
        if load == FrameLoadType::BackForward {
            return Self::BackForward;
        }
        if load == FrameLoadType::Restore {
            return Self::Restore;
        }
        Self::Other
    }
}
/// No Default: caller identifies the actual navigation producer. An absent
/// URLRequest field denotes missing navigation metadata, never an Other load.
#[derive(Clone, Debug)]
pub struct NavigationRequestMetadata {
    pub load_type: FrameLoadType,
    /// Traversal destination, or the previous item for reload propagation.
    /// Installed by DocumentLoader before the new Window/parser sees history.
    pub history_item: Option<Arc<HistoryItemSnapshot>>,
    /// Browser-owned session entries, in traversal order, on either side of
    /// the destination. Restored into the destination Window's native owners.
    pub back_entries: Arc<[Arc<HistoryItemSnapshot>]>,
    pub forward_entries: Arc<[Arc<HistoryItemSnapshot>]>,
    /// NavigationApiMethodTracker::GetSerializedState for this navigation.
    pub navigation_api_state: Option<Arc<[u8]>>,
    pub navigation_type: WebNavigationType,
    pub is_browser_initiated: bool,
    pub had_transient_user_activation: bool,
    pub requestor_origin: Option<RequestorOrigin>,
    /// Presence is WebNavigationParams::has_text_fragment_token; the value is
    /// the trusted-initiator bit attached to that exact propagated token.
    pub propagated_text_fragment_token: Option<bool>,
    pub internal_scroll_to_text_fragment: Option<String>,
}
