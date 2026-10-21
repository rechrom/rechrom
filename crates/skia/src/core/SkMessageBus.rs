// Copyright 2013 Google Inc. BSD-style license.
use crate::include::private::SkMutex::{
    SkAutoMutexExclusive, SkAutoMutexExclusiveStorage, SkMutex,
};
use core::{cell::UnsafeCell, marker::PhantomPinned, pin::Pin, ptr::NonNull};

// Native SkShouldPostMessageToBus must have its real per-message specialization.
// No Send/Sync/'static payload bound or default routing is fabricated.
pub trait RoutedMessage<ID> {
    fn should_post(&self, uniqueID: ID) -> bool;
}

// Source: src/core/SkMessageBus.h:28-38
/// Rust's explicit projection of the native false-domain type condition.
/// Safety: a non-sk_sp type must not implement Clone or Copy. IS_SK_SP is true
/// only for a real native sk_sp specialization, and its checks inspect that same
/// intrusive pointer/count. Neither this contract nor internal locking proves
/// that arbitrary borrowed payload graphs may cross threads.
pub unsafe trait MoveMessage<ID>: RoutedMessage<ID> {
    const IS_SK_SP: bool;
    fn assert_unique_if_sk_sp(&self);
    fn is_empty_if_sk_sp(&self) -> bool;
}
// Source: src/core/SkMessageBus.h:31-32,135-136,144-146
unsafe impl<T, ID> MoveMessage<ID> for crate::include::core::SkRefCnt::sk_sp<T>
where
    T: crate::include::core::SkRefCnt::SkRefCntObject + ?Sized,
    crate::include::core::SkRefCnt::sk_sp<T>: RoutedMessage<ID>,
{
    const IS_SK_SP: bool = true;
    fn assert_unique_if_sk_sp(&self) {
        crate::SkASSERT!(@source b"src/core/SkMessageBus.h\0".as_ptr(),136; @condition b"m->unique()\0".as_ptr(); self.ref_cnt_base().unique());
    }
    fn is_empty_if_sk_sp(&self) -> bool {
        self.get().is_none()
    }
}
trait MessageBusAdmissible<M, ID, const COPY: bool> {}
impl<M, ID> MessageBusAdmissible<M, ID, true> for () {}
impl<M: MoveMessage<ID>, ID> MessageBusAdmissible<M, ID, false> for () {}

// Source: src/core/SkMessageBus.h:44-69
// Native reverse field destruction: messages mutex before messages array.
// bus is only a nonowning registration identity/construction-state token. It
// never retains the bus or inbox and is not a second lifetime owner.
pub struct Inbox<M: RoutedMessage<ID>, ID: Copy, const COPY: bool = true> {
    fMessagesMutex: SkMutex,
    fMessages: UnsafeCell<Vec<M>>,
    fUniqueID: ID,
    bus: Option<NonNull<SkMessageBus<M, ID, COPY>>>,
    _pin: PhantomPinned,
}
// Source: src/core/SkMessageBus.h:28-29,67-69
// Native reverse field destruction: mutex before pointer array. All array
// entries are raw nonowning identities; no Box, Arc or Weak belongs to an entry.
pub struct SkMessageBus<M: RoutedMessage<ID>, ID: Copy, const COPY: bool = true> {
    fInboxesMutex: SkMutex,
    fInboxes: UnsafeCell<Vec<NonNull<Inbox<M, ID, COPY>>>>,
    _pin: PhantomPinned,
}
// Source: src/core/SkMessageBus.h:49-49
impl<M: RoutedMessage<ID>, ID: Copy, const COPY: bool> Inbox<M, ID, COPY> {
    pub fn uniqueID(&self) -> ID {
        self.fUniqueID
    }
    /// Rust preconstruction storage. The real native constructor body must run
    /// via construct() at the final pinned address before any poll/Post use.
    pub fn uninit(uniqueID: ID) -> Self {
        Self {
            fMessagesMutex: SkMutex::new(),
            fMessages: UnsafeCell::new(Vec::new()),
            fUniqueID: uniqueID,
            bus: None,
            _pin: PhantomPinned,
        }
    }
    // Source: src/core/SkMessageBus.h:83-90
    /// Safety: this is the final pinned allocation and has not been registered.
    /// The native message-graph/thread leases hold. Its singleton bus outlives
    /// the registration, and no access races construction or final destruction.
    pub unsafe fn construct(mut self: Pin<&mut Self>)
    where
        M: MessageBusSingleton<ID, COPY>,
        (): MessageBusAdmissible<M, ID, COPY>,
    {
        let this = unsafe { NonNull::from(self.as_mut().get_unchecked_mut()) };
        let bus = unsafe { M::Get() };
        let mut storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { Pin::new_unchecked(&(*bus.as_ptr()).fInboxesMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(storage.as_mut(), mutex) };
        unsafe {
            (*(*bus.as_ptr()).fInboxes.get()).push(this);
        }
        unsafe {
            (*this.as_ptr()).bus = Some(bus);
        }
    }
    // Source: src/core/SkMessageBus.h:106-110
    /// Safety: actual registered inbox and OS mutex remain alive through the
    /// final signal, and the caller holds this bus's inbox mutex as native Post
    /// does. Payload move/destruction and callbacks honor their original leases.
    unsafe fn receive(this: NonNull<Self>, message: M) {
        let mut storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { Pin::new_unchecked(&(*this.as_ptr()).fMessagesMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(storage.as_mut(), mutex) };
        unsafe {
            (*(*this.as_ptr()).fMessages.get()).push(message);
        }
    }
    // Source: src/core/SkMessageBus.h:112-119
    /// Safety: this is the constructed pinned inbox, alive until the final OS
    /// signal returns; output is exclusive and disjoint from its actual queue.
    /// No concurrent destructor or invalid payload graph is permitted.
    pub unsafe fn poll(this: NonNull<Self>, messages: &mut Vec<M>) {
        messages.clear();
        let mut storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { Pin::new_unchecked(&(*this.as_ptr()).fMessagesMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(storage.as_mut(), mutex) };
        unsafe {
            core::mem::swap(&mut *(*this.as_ptr()).fMessages.get(), messages);
        }
    }
}
// Source: src/core/SkMessageBus.h:92-104
impl<M: RoutedMessage<ID>, ID: Copy, const COPY: bool> Drop for Inbox<M, ID, COPY> {
    fn drop(&mut self) {
        let Some(bus) = self.bus else {
            return;
        }; // no native object constructed yet
        let this = NonNull::from(&mut *self);
        let mut storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { Pin::new_unchecked(&(*bus.as_ptr()).fInboxesMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(storage.as_mut(), mutex) };
        let inboxes = unsafe { &mut *(*bus.as_ptr()).fInboxes.get() };
        for i in 0..inboxes.len() {
            if inboxes[i] == this {
                inboxes.swap_remove(i);
                break;
            }
        }
        // Unsubscribe releases its guard before the actual fields are destroyed.
        // The owner must already have ended all concurrent raw poll operations.
    }
}
// Source: src/core/SkMessageBus.h:123-124
impl<M: RoutedMessage<ID>, ID: Copy, const COPY: bool> SkMessageBus<M, ID, COPY> {
    fn new() -> Self {
        Self {
            fInboxesMutex: SkMutex::new(),
            fInboxes: UnsafeCell::new(Vec::new()),
            _pin: PhantomPinned,
        }
    }
}
// Source: src/core/SkMessageBus.h:126-133,141-148
impl<M: RoutedMessage<ID> + Clone + MessageBusSingleton<ID, true>, ID: Copy>
    SkMessageBus<M, ID, true>
{
    /// Safety: uphold native cross-thread payload/callback leases; no routing
    /// reentry into this bus. Every registered inbox is stable until unsubscribe.
    pub unsafe fn Post(message: M) {
        let bus = unsafe { M::Get() };
        let mut storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { Pin::new_unchecked(&(*bus.as_ptr()).fInboxesMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(storage.as_mut(), mutex) };
        let inboxes = unsafe { &*(*bus.as_ptr()).fInboxes.get() };
        for i in 0..inboxes.len() {
            let inbox = inboxes[i];
            if message.should_post(unsafe { (*inbox.as_ptr()).fUniqueID }) {
                unsafe {
                    Inbox::receive(inbox, message.clone());
                }
            }
        }
    }
}
// Source: src/core/SkMessageBus.h:126-148
impl<M: MoveMessage<ID> + MessageBusSingleton<ID, false>, ID: Copy> SkMessageBus<M, ID, false> {
    /// Safety: the same native thread/lifetime and no-reentry contract as true
    /// Post applies. A move-only payload has exactly its native recipient owner.
    pub unsafe fn Post(message: M) {
        // Parameter ownership predates the guard, so an undelivered message is
        // destroyed after unlocking, in the original native parameter order.
        let mut message = Some(message);
        let bus = unsafe { M::Get() };
        let mut storage = core::pin::pin!(SkAutoMutexExclusiveStorage::new());
        let mutex = unsafe { Pin::new_unchecked(&(*bus.as_ptr()).fInboxesMutex) };
        let _guard = unsafe { SkAutoMutexExclusive::init(storage.as_mut(), mutex) };
        let inboxes = unsafe { &*(*bus.as_ptr()).fInboxes.get() };
        for i in 0..inboxes.len() {
            let inbox = inboxes[i];
            if message
                .as_ref()
                .unwrap()
                .should_post(unsafe { (*inbox.as_ptr()).fUniqueID })
            {
                if M::IS_SK_SP {
                    message.as_ref().unwrap().assert_unique_if_sk_sp();
                }
                unsafe {
                    Inbox::receive(inbox, message.take().unwrap());
                }
                break;
            }
        }
        if M::IS_SK_SP {
            crate::SkASSERT!(@source b"src/core/SkMessageBus.h\0".as_ptr(),146; @condition b"!m\0".as_ptr(); message.as_ref().is_none_or(M::is_empty_if_sk_sp));
        }
    }
}

// Source: src/core/SkMessageBus.h:65-65,73-79
/// Actual DECLARE_SKMESSAGEBUS_MESSAGE specialization, rather than a registry
/// that owns buses via Arc. Safety: Get returns the same fully constructed,
/// pinned, immortal native bus for this exact message/ID/boolean specialization.
pub unsafe trait MessageBusSingleton<ID: Copy, const COPY: bool>:
    RoutedMessage<ID> + Sized
{
    unsafe fn Get() -> NonNull<SkMessageBus<Self, ID, COPY>>;
}
// Only this initialization cell is Sync: it publishes an opaque native pointer
// using SkOnce. The pointee/message graph is deliberately not Send or Sync.
#[doc(hidden)]
pub struct MessageBusSingletonState<M: RoutedMessage<ID>, ID: Copy, const COPY: bool> {
    once: crate::include::private::SkOnce::SkOnce,
    pointer: UnsafeCell<*mut SkMessageBus<M, ID, COPY>>,
}
unsafe impl<M: RoutedMessage<ID>, ID: Copy, const COPY: bool> Sync
    for MessageBusSingletonState<M, ID, COPY>
{
}
impl<M: RoutedMessage<ID>, ID: Copy, const COPY: bool> MessageBusSingletonState<M, ID, COPY> {
    pub const fn new() -> Self {
        Self {
            once: crate::include::private::SkOnce::SkOnce::new(),
            pointer: UnsafeCell::new(core::ptr::null_mut()),
        }
    }
    /// Safety: the cell is the real stable per-specialization immortal static.
    pub unsafe fn get(this: *const Self) -> NonNull<SkMessageBus<M, ID, COPY>>
    where
        (): MessageBusAdmissible<M, ID, COPY>,
    {
        unsafe {
            crate::include::private::SkOnce::SkOnce::call_once(
                core::ptr::addr_of!((*this).once),
                || {
                    (*(*this).pointer.get()) = Box::into_raw(Box::new(SkMessageBus::new()));
                },
            );
            NonNull::new_unchecked(*(*this).pointer.get())
        }
    }
}
// Native macro must be instantiated at its real producer TU exactly once.
macro_rules! DECLARE_SKMESSAGEBUS_MESSAGE {
    ($message:ty,$id:ty,$copy:expr) => {
        unsafe impl $crate::src::core::SkMessageBus::MessageBusSingleton<$id, $copy> for $message {
            unsafe fn Get(
            ) -> core::ptr::NonNull<$crate::src::core::SkMessageBus::SkMessageBus<Self, $id, $copy>>
            {
                static STATE: $crate::src::core::SkMessageBus::MessageBusSingletonState<
                    $message,
                    $id,
                    $copy,
                > = $crate::src::core::SkMessageBus::MessageBusSingletonState::new();
                unsafe {
                    $crate::src::core::SkMessageBus::MessageBusSingletonState::get(
                        core::ptr::addr_of!(STATE),
                    )
                }
            }
        }
    };
}
pub(crate) use DECLARE_SKMESSAGEBUS_MESSAGE;
