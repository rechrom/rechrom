//! Chromium `third_party/blink/renderer/core/inspector/console_message_storage.h`
//! and `console_message_storage.cc`: duplicate filtering and bounded retention.

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::Rc,
};

use super::console_message::ConsoleMessage;

#[derive(Default)]
pub struct ConsoleMessageStorage {
    messages: RefCell<VecDeque<Rc<ConsoleMessage>>>,
    expired_count: Cell<u64>,
}
impl ConsoleMessageStorage {
    pub fn AddConsoleMessage(&self, message: Rc<ConsoleMessage>, discard_duplicates: bool) -> bool {
        let mut messages = self.messages.borrow_mut();
        if discard_duplicates && messages.iter().any(|old| old.message == message.message) {
            return false;
        }
        if messages.len() == 1000 {
            self.expired_count.set(self.expired_count.get() + 1);
            messages.pop_front();
        }
        messages.push_back(message);
        true
    }
    pub fn Messages(&self) -> Vec<Rc<ConsoleMessage>> {
        self.messages.borrow().iter().cloned().collect()
    }
    pub fn ExpiredCount(&self) -> u64 {
        self.expired_count.get()
    }
    pub fn Clear(&self) {
        self.messages.borrow_mut().clear();
        self.expired_count.set(0);
    }
}
