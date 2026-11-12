//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! Shared HistoryItem identity and values. No DOM, VM, event or loader dependency.
//! Source core/loader/history_item.{h,cc}: h:77-150,159-230;
//! cc:65-71,147-178,184-269; public/web/web_frame_load_type.h:38-45.
#![allow(non_snake_case)]
mod navigation_request;
pub mod unique_name;
pub use navigation_request::{NavigationRequestMetadata, WebNavigationType};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, OnceLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};
/// The VM supplies and retains this opaque immutable serialization. History
/// never deserializes it or imports a VM/script/event owner.
pub trait SerializedScriptValue: Any {
    /// Immutable structured-serialization storage, with no realm or VM owner.
    fn StorageBytes(&self) -> Arc<[u8]>;
}
/// A restored serialization is interpreted by the destination VM only when
/// script reads it. Sharing these bytes never transfers a source-realm owner.
pub struct StoredSerializedScriptValue(Arc<[u8]>);
impl StoredSerializedScriptValue {
    pub fn new(bytes: Arc<[u8]>) -> Rc<Self> {
        Rc::new(Self(bytes))
    }
}
impl SerializedScriptValue for StoredSerializedScriptValue {
    fn StorageBytes(&self) -> Arc<[u8]> {
        self.0.clone()
    }
}
/// Opaque static navigation result. FrameLoader retains this payload without
/// depending back on the document loader, DOM, or a script runtime.
/// The assembly root routes it to the concrete document-loader transaction.
pub trait StaticDocumentCommitPayload: Any {
    fn AsAny(&self) -> &dyn Any;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameLoadType {
    Standard,
    BackForward,
    Reload,
    ReplaceCurrentItem,
    ReloadBypassingCache,
    Restore,
}
impl FrameLoadType {
    pub fn IsTraversal(self) -> bool {
        matches!(self, Self::BackForward | Self::Restore)
    }
    pub fn IsReload(self) -> bool {
        matches!(self, Self::Reload | Self::ReloadBypassingCache)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserNavigationInvolvement {
    None,
    Activation,
    BrowserUI,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferrerPolicy {
    Default,
    Never,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    Always,
}
/// Original immutable SecurityOrigin key, supplied by its owning context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestorOrigin {
    Tuple {
        scheme: String,
        host: String,
        port: u16,
    },
    Opaque(u64),
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollRestorationType {
    #[default]
    Auto,
    Manual,
}
#[derive(Clone, Debug, Default)]
pub struct ScrollAnchorData {
    pub selector: String,
    pub offset: [f32; 2],
    pub simhash: u64,
}
#[derive(Clone, Debug, Default)]
pub struct ViewState {
    pub visual_viewport_scroll_offset: [f32; 2],
    pub scroll_offset: [f32; 2],
    pub page_scale_factor: f32,
    pub scroll_anchor_data: ScrollAnchorData,
}
pub trait HistoryItemIdentitySource {
    fn NextSequenceNumber(&self) -> i64;
    fn CreateCanonicalUuidString(&self) -> String;
}
/// Source's process-wide monotonically increasing, time-seeded sequence and
/// cryptographically random canonical UUID. No Page-local duplicate counters.
pub struct NativeHistoryItemIdentitySource;
impl HistoryItemIdentitySource for NativeHistoryItemIdentitySource {
    fn NextSequenceNumber(&self) -> i64 {
        static SEQUENCE: OnceLock<AtomicI64> = OnceLock::new();
        SEQUENCE
            .get_or_init(|| {
                AtomicI64::new(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("native history epoch")
                        .as_micros()
                        .try_into()
                        .expect("history time fits i64"),
                )
            })
            .fetch_add(1, Ordering::Relaxed)
            + 1
    }
    fn CreateCanonicalUuidString(&self) -> String {
        // platform/wtf/uuid.cc:12-15; base/uuid.cc:73-111.
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("native history UUID entropy");
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let mut text = String::with_capacity(36);
        use std::fmt::Write;
        for (index, byte) in bytes.into_iter().enumerate() {
            if matches!(index, 4 | 6 | 8 | 10) {
                text.push('-');
            }
            write!(text, "{byte:02x}").expect("UUID string");
        }
        text
    }
}
/// HistoryItem's original lazy DocumentState source owner. The FormController
/// implements this interface; history neither copies controls nor imports DOM.
pub trait DocumentState {
    fn ToStateVector(&self) -> Vec<Rc<[u16]>>;
}
/// Immutable browser/renderer session-history boundary, corresponding to
/// HistoryItem::ToPageState/Create (history_item.cc:73-145,287-354). The form
/// body retains its original Send-safe immutable file/blob owners. Lazy form
/// state is materialized on the source sequence; neither DocumentState nor Rc
/// crosses to another Page. This in-process representation does not claim the
/// Chromium PageState wire format.
#[derive(Clone, Debug)]
pub struct HistoryItemSnapshot {
    /// Browser NavigationEntry identity, independent of any Frame's item
    /// sequence. A new renderer commit leaves it absent for host allocation.
    pub session_entry_id: Option<i64>,
    pub url: String,
    /// FrameNavigationEntry's committed origin, supplied by the embedding at
    /// publication. Distinct from the navigation request's initiator origin.
    pub committed_origin: Option<RequestorOrigin>,
    pub referrer: String,
    pub requestor_origin: Option<RequestorOrigin>,
    pub referrer_policy: ReferrerPolicy,
    pub target: String,
    pub document_state: Arc<[Arc<[u16]>]>,
    pub view_state: Option<ViewState>,
    pub scroll_restoration_type: ScrollRestorationType,
    pub item_sequence_number: i64,
    pub document_sequence_number: i64,
    pub navigation_api_key: String,
    pub navigation_api_id: String,
    pub state_object: Option<Arc<[u8]>>,
    pub navigation_api_state: Option<Arc<[u8]>>,
    pub form_data: Option<Arc<fileapi::encoded_form_data::EncodedFormData>>,
    pub form_content_type: Option<String>,
    /// Joint session history keeps each child's original frame/document item.
    /// Names survive renderer/frame-token replacement; DOM node IDs do not.
    pub children: Arc<[ChildHistoryItemSnapshot]>,
}

#[derive(Clone, Debug)]
pub struct ChildHistoryItemSnapshot {
    pub unique_name: String,
    pub item: Arc<HistoryItemSnapshot>,
}
impl HistoryItemSnapshot {
    /// Browser entry identity differs from the frame-item identity tree. State
    /// updates and asynchronous commit acknowledgements compare the latter.
    pub fn SameItemTree(&self, other: &Self) -> bool {
        self.item_sequence_number == other.item_sequence_number
            && self.document_sequence_number == other.document_sequence_number
            && self.children.len() == other.children.len()
            && self.children.iter().all(|child| {
                other
                    .children
                    .iter()
                    .find(|other| other.unique_name == child.unique_name)
                    .is_some_and(|other| child.item.SameItemTree(&other.item))
            })
    }
}

pub struct HistoryItemOwner {
    url: RefCell<String>,
    referrer: RefCell<String>,
    requestor_origin: RefCell<Option<RequestorOrigin>>,
    referrer_policy: Cell<ReferrerPolicy>,
    target: RefCell<String>,
    document_state: RefCell<Vec<Rc<[u16]>>>,
    document_state_owner: RefCell<Option<Rc<dyn DocumentState>>>,
    view_state: RefCell<Option<ViewState>>,
    scroll_restoration_type: Cell<ScrollRestorationType>,
    item_sequence_number: Cell<i64>,
    document_sequence_number: Cell<i64>,
    navigation_api_key: RefCell<String>,
    navigation_api_id: RefCell<String>,
    state_object: RefCell<Option<Rc<dyn SerializedScriptValue>>>,
    navigation_api_state: RefCell<Option<Rc<dyn SerializedScriptValue>>>,
    form_data: RefCell<Option<std::sync::Arc<fileapi::encoded_form_data::EncodedFormData>>>,
    form_content_type: RefCell<Option<String>>,
}
impl HistoryItemOwner {
    pub fn Snapshot(&self) -> Arc<HistoryItemSnapshot> {
        Arc::new(HistoryItemSnapshot {
            session_entry_id: None,
            url: self.URL(),
            committed_origin: None,
            referrer: self.Referrer(),
            requestor_origin: self.RequestorOrigin(),
            referrer_policy: self.ReferrerPolicy(),
            target: self.Target(),
            document_state: self
                .DocumentState()
                .iter()
                .map(|value| Arc::from(value.as_ref()))
                .collect(),
            view_state: self.ViewState(),
            scroll_restoration_type: self.ScrollRestorationType(),
            item_sequence_number: self.ItemSequenceNumber(),
            document_sequence_number: self.DocumentSequenceNumber(),
            navigation_api_key: self.NavigationApiKey(),
            navigation_api_id: self.NavigationApiId(),
            state_object: self.StateObject().map(|value| value.StorageBytes()),
            navigation_api_state: self.NavigationApiState().map(|value| value.StorageBytes()),
            form_data: self.FormData(),
            form_content_type: self.FormContentType(),
            children: Arc::from([]),
        })
    }
    /// Restore exact persisted identities instead of generating replacement
    /// sequence numbers, keys, or IDs on the destination document sequence.
    pub fn Restore(snapshot: &HistoryItemSnapshot) -> Rc<Self> {
        Rc::new(Self {
            url: RefCell::new(snapshot.url.clone()),
            referrer: RefCell::new(snapshot.referrer.clone()),
            requestor_origin: RefCell::new(snapshot.requestor_origin.clone()),
            referrer_policy: Cell::new(snapshot.referrer_policy),
            target: RefCell::new(snapshot.target.clone()),
            document_state: RefCell::new(
                snapshot
                    .document_state
                    .iter()
                    .map(|value| Rc::from(value.as_ref()))
                    .collect(),
            ),
            document_state_owner: RefCell::new(None),
            view_state: RefCell::new(snapshot.view_state.clone()),
            scroll_restoration_type: Cell::new(snapshot.scroll_restoration_type),
            item_sequence_number: Cell::new(snapshot.item_sequence_number),
            document_sequence_number: Cell::new(snapshot.document_sequence_number),
            navigation_api_key: RefCell::new(snapshot.navigation_api_key.clone()),
            navigation_api_id: RefCell::new(snapshot.navigation_api_id.clone()),
            state_object: RefCell::new(snapshot.state_object.clone().map(|bytes| {
                StoredSerializedScriptValue::new(bytes) as Rc<dyn SerializedScriptValue>
            })),
            navigation_api_state: RefCell::new(snapshot.navigation_api_state.clone().map(
                |bytes| StoredSerializedScriptValue::new(bytes) as Rc<dyn SerializedScriptValue>,
            )),
            form_data: RefCell::new(snapshot.form_data.clone()),
            form_content_type: RefCell::new(snapshot.form_content_type.clone()),
        })
    }
    pub fn Create() -> Rc<Self> {
        Self::new(&NativeHistoryItemIdentitySource)
    }
    pub fn new(source: &dyn HistoryItemIdentitySource) -> Rc<Self> {
        Rc::new(Self {
            url: RefCell::new(String::new()),
            referrer: RefCell::new(String::new()),
            requestor_origin: RefCell::new(None),
            referrer_policy: Cell::new(ReferrerPolicy::Default),
            target: RefCell::new(String::new()),
            document_state: RefCell::new(Vec::new()),
            document_state_owner: RefCell::new(None),
            view_state: RefCell::new(None),
            scroll_restoration_type: Cell::new(ScrollRestorationType::Auto),
            item_sequence_number: Cell::new(source.NextSequenceNumber()),
            document_sequence_number: Cell::new(source.NextSequenceNumber()),
            navigation_api_key: RefCell::new(source.CreateCanonicalUuidString()),
            navigation_api_id: RefCell::new(source.CreateCanonicalUuidString()),
            state_object: RefCell::new(None),
            navigation_api_state: RefCell::new(None),
            form_data: RefCell::new(None),
            form_content_type: RefCell::new(None),
        })
    }
    pub fn URL(&self) -> String {
        self.url.borrow().clone()
    }
    pub fn SetURL(&self, value: String) {
        *self.url.borrow_mut() = value;
    }
    pub fn Referrer(&self) -> String {
        self.referrer.borrow().clone()
    }
    pub fn SetReferrer(&self, value: String) {
        *self.referrer.borrow_mut() = value;
    }
    pub fn RequestorOrigin(&self) -> Option<RequestorOrigin> {
        self.requestor_origin.borrow().clone()
    }
    pub fn SetRequestorOrigin(&self, value: Option<RequestorOrigin>) {
        *self.requestor_origin.borrow_mut() = value;
    }
    pub fn ReferrerPolicy(&self) -> ReferrerPolicy {
        self.referrer_policy.get()
    }
    pub fn SetReferrerPolicy(&self, value: ReferrerPolicy) {
        self.referrer_policy.set(value);
    }
    pub fn Target(&self) -> String {
        self.target.borrow().clone()
    }
    pub fn SetTarget(&self, value: String) {
        *self.target.borrow_mut() = value;
    }
    pub fn DocumentState(&self) -> Vec<Rc<[u16]>> {
        if let Some(owner) = self.document_state_owner.borrow().clone() {
            *self.document_state.borrow_mut() = owner.ToStateVector();
        }
        self.document_state.borrow().clone()
    }
    pub fn SetDocumentState(&self, value: Vec<Rc<[u16]>>) {
        assert!(self.document_state_owner.borrow().is_none());
        *self.document_state.borrow_mut() = value;
    }
    pub fn SetDocumentStateOwner(&self, value: Rc<dyn DocumentState>) {
        *self.document_state_owner.borrow_mut() = Some(value);
    }
    pub fn ClearDocumentState(&self) {
        self.document_state_owner.borrow_mut().take();
        self.document_state.borrow_mut().clear();
    }
    pub fn ViewState(&self) -> Option<ViewState> {
        self.view_state.borrow().clone()
    }
    pub fn CopyViewStateFrom(&self, item: &Self) {
        *self.view_state.borrow_mut() = item.ViewState();
    }
    pub fn ClearViewState(&self) {
        *self.view_state.borrow_mut() = None;
    }
    pub fn SetVisualViewportScrollOffset(&self, value: [f32; 2]) {
        self.view_state
            .borrow_mut()
            .get_or_insert_with(ViewState::default)
            .visual_viewport_scroll_offset = value;
    }
    pub fn SetScrollOffset(&self, value: [f32; 2]) {
        self.view_state
            .borrow_mut()
            .get_or_insert_with(ViewState::default)
            .scroll_offset = value;
    }
    pub fn SetPageScaleFactor(&self, value: f32) {
        self.view_state
            .borrow_mut()
            .get_or_insert_with(ViewState::default)
            .page_scale_factor = value;
    }
    pub fn SetScrollAnchorData(&self, value: ScrollAnchorData) {
        self.view_state
            .borrow_mut()
            .get_or_insert_with(ViewState::default)
            .scroll_anchor_data = value;
    }
    pub fn ScrollRestorationType(&self) -> ScrollRestorationType {
        self.scroll_restoration_type.get()
    }
    pub fn SetScrollRestorationType(&self, value: ScrollRestorationType) {
        self.scroll_restoration_type.set(value);
    }
    pub fn ItemSequenceNumber(&self) -> i64 {
        self.item_sequence_number.get()
    }
    pub fn DocumentSequenceNumber(&self) -> i64 {
        self.document_sequence_number.get()
    }
    pub fn SetItemSequenceNumber(&self, value: i64) {
        self.item_sequence_number.set(value);
    }
    pub fn SetDocumentSequenceNumber(&self, value: i64) {
        self.document_sequence_number.set(value);
    }
    pub fn NavigationApiKey(&self) -> String {
        self.navigation_api_key.borrow().clone()
    }
    pub fn SetNavigationApiKey(&self, value: String) {
        *self.navigation_api_key.borrow_mut() = value;
    }
    pub fn NavigationApiId(&self) -> String {
        self.navigation_api_id.borrow().clone()
    }
    pub fn SetNavigationApiId(&self, value: String) {
        *self.navigation_api_id.borrow_mut() = value;
    }
    pub fn StateObject(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.state_object.borrow().clone()
    }
    pub fn SetStateObject(&self, value: Option<Rc<dyn SerializedScriptValue>>) {
        *self.state_object.borrow_mut() = value;
    }
    pub fn NavigationApiState(&self) -> Option<Rc<dyn SerializedScriptValue>> {
        self.navigation_api_state.borrow().clone()
    }
    pub fn SetNavigationApiState(&self, value: Option<Rc<dyn SerializedScriptValue>>) {
        *self.navigation_api_state.borrow_mut() = value;
    }
    pub fn FormData(&self) -> Option<std::sync::Arc<fileapi::encoded_form_data::EncodedFormData>> {
        self.form_data.borrow().clone()
    }
    pub fn SetFormData(
        &self,
        value: Option<std::sync::Arc<fileapi::encoded_form_data::EncodedFormData>>,
    ) {
        *self.form_data.borrow_mut() = value;
    }
    pub fn FormContentType(&self) -> Option<String> {
        self.form_content_type.borrow().clone()
    }
    pub fn SetFormContentType(&self, value: Option<String>) {
        *self.form_content_type.borrow_mut() = value;
    }
}

/// OldDocumentInfoForCommit::UnloadTimingInfo, frame_loader.cc1531-1544.
/// Time values remain in the original document clock's monotonic domain.
#[derive(Clone, Copy, Debug)]
pub struct UnloadEventTiming {
    pub can_request_from_previous_document: bool,
    pub event_start: std::time::Instant,
    pub event_end: std::time::Instant,
}

/// WebViewImpl's browser-provided session-history counters, not an entries
/// store. Source web_view_impl.h:911-918, .cc:3861-3896.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionHistoryState {
    pub offset: i32,
    pub length: i32,
}
impl Default for SessionHistoryState {
    fn default() -> Self {
        Self {
            offset: -1,
            length: 0,
        }
    }
}
impl SessionHistoryState {
    pub fn SetHistoryListFromNavigation(&mut self, offset: i32, length: i32) {
        assert!(offset >= -1 && offset < length && length <= 50);
        self.offset = offset;
        self.length = length;
    }
    pub fn IncreaseHistoryListFromNavigation(&mut self) {
        self.offset = (self.offset + 1).min(49);
        self.length = self.offset + 1;
    }
    pub fn HistoryBackListCount(&self) -> i32 {
        self.offset.max(0)
    }
    pub fn HistoryForwardListCount(&self) -> i32 {
        self.length - self.HistoryBackListCount() - 1
    }
    /// LocalFrameClientImpl.cc:824-843, before GoToEntryAtOffset is sent.
    pub fn CanNavigateBackForward(&self, delta: i32) -> bool {
        delta <= self.HistoryForwardListCount() && delta >= -self.HistoryBackListCount()
    }
}

/// Host admission failures retain unconsumed form state for a later retry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormStateRestoreError {
    NativeFileReaderUnavailable,
    NativeFileUnavailable(String),
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn restores_full_history_state_on_another_document_sequence() {
        struct FormState;
        impl DocumentState for FormState {
            fn ToStateVector(&self) -> Vec<Rc<[u16]>> {
                vec![Rc::from([0x41, 0xd800, 0x42])]
            }
        }
        let item = HistoryItemOwner::Create();
        item.SetURL("https://example.test/post#section".into());
        item.SetReferrer("https://example.test/source".into());
        item.SetRequestorOrigin(Some(RequestorOrigin::Opaque(52)));
        item.SetReferrerPolicy(ReferrerPolicy::SameOrigin);
        item.SetTarget("child".into());
        item.SetDocumentStateOwner(Rc::new(FormState));
        item.SetVisualViewportScrollOffset([3.5, 7.25]);
        item.SetScrollOffset([14.5, 90.25]);
        item.SetPageScaleFactor(1.75);
        item.SetScrollAnchorData(ScrollAnchorData {
            selector: "#anchor".into(),
            offset: [2.5, 4.75],
            simhash: 1234,
        });
        item.SetScrollRestorationType(ScrollRestorationType::Manual);
        item.SetItemSequenceNumber(4321);
        item.SetDocumentSequenceNumber(8765);
        item.SetNavigationApiKey("original-key".into());
        item.SetNavigationApiId("original-id".into());
        let state: Arc<[u8]> = Arc::from([1, 2, 3]);
        let navigation_state: Arc<[u8]> = Arc::from([4, 5, 6]);
        let body = fileapi::encoded_form_data::EncodedFormData::FromBytes(vec![7, 8, 9]);
        item.SetStateObject(Some(StoredSerializedScriptValue::new(state.clone())));
        item.SetNavigationApiState(Some(StoredSerializedScriptValue::new(
            navigation_state.clone(),
        )));
        item.SetFormData(Some(body.clone()));
        item.SetFormContentType(Some("application/x-www-form-urlencoded".into()));
        let snapshot = item.Snapshot();
        assert!(Arc::ptr_eq(snapshot.state_object.as_ref().unwrap(), &state));
        assert!(Arc::ptr_eq(
            snapshot.navigation_api_state.as_ref().unwrap(),
            &navigation_state
        ));
        assert!(Arc::ptr_eq(snapshot.form_data.as_ref().unwrap(), &body));
        item.SetURL("https://example.test/later".into());
        drop(item);
        std::thread::spawn(move || {
            let restored = HistoryItemOwner::Restore(&snapshot);
            assert_eq!(restored.URL(), snapshot.url);
            assert_eq!(restored.Referrer(), snapshot.referrer);
            assert_eq!(restored.RequestorOrigin(), snapshot.requestor_origin);
            assert_eq!(restored.ReferrerPolicy(), ReferrerPolicy::SameOrigin);
            assert_eq!(restored.Target(), "child");
            assert_eq!(restored.DocumentState()[0].as_ref(), &[0x41, 0xd800, 0x42]);
            assert_eq!(restored.ItemSequenceNumber(), 4321);
            assert_eq!(restored.DocumentSequenceNumber(), 8765);
            assert_eq!(restored.NavigationApiKey(), "original-key");
            assert_eq!(restored.NavigationApiId(), "original-id");
            assert_eq!(
                restored.ScrollRestorationType(),
                ScrollRestorationType::Manual
            );
            let view = restored.ViewState().unwrap();
            assert_eq!(view.visual_viewport_scroll_offset, [3.5, 7.25]);
            assert_eq!(view.scroll_offset, [14.5, 90.25]);
            assert_eq!(view.page_scale_factor, 1.75);
            assert_eq!(view.scroll_anchor_data.selector, "#anchor");
            assert_eq!(view.scroll_anchor_data.offset, [2.5, 4.75]);
            assert_eq!(view.scroll_anchor_data.simhash, 1234);
            assert!(Arc::ptr_eq(
                &restored.StateObject().unwrap().StorageBytes(),
                &state
            ));
            assert!(Arc::ptr_eq(
                &restored.NavigationApiState().unwrap().StorageBytes(),
                &navigation_state
            ));
            assert!(Arc::ptr_eq(restored.FormData().as_ref().unwrap(), &body));
            assert_eq!(
                restored.FormContentType(),
                Some("application/x-www-form-urlencoded".into())
            );
            fn send_sync<T: Send + Sync>() {}
            send_sync::<HistoryItemSnapshot>();
        })
        .join()
        .unwrap();
    }
}
