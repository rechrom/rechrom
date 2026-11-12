//! Document::Token owns its lazy UnguessableToken (document.h:1005-1010,2686).
//! The parser transport can retain a PersistentDocument allocation across its
//! initial response commit. That explicit commit replaces the logical Document
//! token; URL/history writes do not. Observers cancel old-document continuations
//! before the token changes and may not read or mutate the DOM.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

pub fn CreateNavigationToken() -> [u64; 2] {
    loop {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).expect("OS entropy for a native navigation token");
        let token = [
            u64::from_le_bytes(bytes[..8].try_into().unwrap()),
            u64::from_le_bytes(bytes[8..].try_into().unwrap()),
        ];
        if token != [0, 0] {
            return token;
        }
    }
}
pub trait DocumentTokenObserver {
    fn WillCommitNewDocument(&self);
}
#[derive(Default)]
pub struct DocumentTokenOwner {
    token: Cell<Option<[u64; 2]>>,
    observers: RefCell<Vec<Weak<dyn DocumentTokenObserver>>>,
}
impl DocumentTokenOwner {
    pub fn Token(&self) -> [u64; 2] {
        if let Some(token) = self.token.get() {
            return token;
        }
        let token = CreateNavigationToken();
        self.token.set(Some(token));
        token
    }
    pub fn Register(&self, observer: &Rc<dyn DocumentTokenObserver>) {
        let mut observers = self.observers.borrow_mut();
        observers.retain(|entry| entry.strong_count() != 0);
        if !observers.iter().any(|entry| {
            entry
                .upgrade()
                .is_some_and(|old| Rc::ptr_eq(&old, observer))
        }) {
            observers.push(Rc::downgrade(observer));
        }
    }
    pub(crate) fn CommitNewDocument(&self) {
        let observers = self.observers.borrow().clone();
        for observer in observers {
            if let Some(observer) = observer.upgrade() {
                observer.WillCommitNewDocument();
            }
        }
        self.token.set(None);
    }
}

impl DocumentTokenOwner {
    /// DocumentInit::WithToken: a new Document may deliberately use the same
    /// token as the old one (JavaScript/XSLT), while keeping a separate lifetime
    /// and observer list. This does not notify or mutate the previous owner.
    pub(crate) fn WithToken(token: [u64; 2]) -> Rc<Self> {
        assert_ne!(token, [0, 0]);
        Rc::new(Self {
            token: Cell::new(Some(token)),
            observers: RefCell::new(Vec::new()),
        })
    }
}
