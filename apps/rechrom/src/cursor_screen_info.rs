//! Host-owned ScreenInfos::system_cursor_size and its native updates.
//!
//! Chromium ui/base/ui_base_features.cc:151-165 enables the size producer on
//! Windows only. Other platforms retain ScreenInfos' empty gfx::Size; this is
//! a source value, independent of the platform cursor bitmap or CSS viewport.
use crate::page_runtime::Command;
use std::sync::mpsc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SystemCursorSize {
    pub width_dip: u32,
    pub height_dip: u32,
}

pub(crate) fn current() -> SystemCursorSize {
    #[cfg(target_os = "windows")]
    {
        windows::current()
    }
    #[cfg(not(target_os = "windows"))]
    {
        SystemCursorSize {
            width_dip: 0,
            height_dip: 0,
        }
    }
}

/// A Page keeps only its subscription, while one native host observer watches
/// the registry. Notifications enqueue typed values; they never enter a Page
/// or block another Page's document executor.
pub(crate) struct Subscription {
    #[cfg(target_os = "windows")]
    id: u64,
}
pub(crate) fn subscribe(sender: mpsc::Sender<Command>, initial: SystemCursorSize) -> Subscription {
    #[cfg(target_os = "windows")]
    {
        windows::subscribe(sender, initial)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (sender, initial);
        Subscription {}
    }
}
#[cfg(target_os = "windows")]
impl Drop for Subscription {
    fn drop(&mut self) {
        windows::unsubscribe(self.id);
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::{
        ffi::c_void,
        ptr,
        sync::{Mutex, OnceLock},
    };
    type HKey = *mut c_void;
    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            key: HKey,
            name: *const u16,
            options: u32,
            access: u32,
            result: *mut HKey,
        ) -> i32;
        fn RegQueryValueExW(
            key: HKey,
            name: *const u16,
            reserved: *mut u32,
            kind: *mut u32,
            data: *mut u8,
            size: *mut u32,
        ) -> i32;
        fn RegNotifyChangeKeyValue(
            key: HKey,
            subtree: i32,
            filter: u32,
            event: *mut c_void,
            asynchronous: i32,
        ) -> i32;
        fn RegCloseKey(key: HKey) -> i32;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(
            attributes: *mut c_void,
            manual_reset: i32,
            initial_state: i32,
            name: *const u16,
        ) -> *mut c_void;
        fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    struct Event(*mut c_void);
    impl Drop for Event {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    struct Key(HKey);
    impl Drop for Key {
        fn drop(&mut self) {
            unsafe {
                RegCloseKey(self.0);
            }
        }
    }
    impl Key {
        fn open() -> Option<Self> {
            let name: Vec<u16> = "Control Panel\\Cursors\0".encode_utf16().collect();
            let mut key = ptr::null_mut();
            // HKEY_CURRENT_USER; KEY_READ | KEY_NOTIFY, as Chromium's owner.
            let result = unsafe {
                RegOpenKeyExW(
                    0x80000001u32 as i32 as isize as HKey,
                    name.as_ptr(),
                    0,
                    0x20019 | 0x10,
                    &mut key,
                )
            };
            (result == 0).then(|| Self(key))
        }
        // Source desktop_native_cursor_manager_win.cc:85-97. A failed read
        // leaves the last committed size unchanged. CursorBaseSize is DIP.
        fn size(&self) -> Option<SystemCursorSize> {
            let name: Vec<u16> = "CursorBaseSize\0".encode_utf16().collect();
            let mut value = 0u32;
            let mut size = std::mem::size_of::<u32>() as u32;
            let mut kind = 0u32;
            let result = unsafe {
                RegQueryValueExW(
                    self.0,
                    name.as_ptr(),
                    ptr::null_mut(),
                    &mut kind,
                    (&mut value as *mut u32).cast(),
                    &mut size,
                )
            };
            if result != 0 || kind != 4 || size != 4 {
                return None;
            }
            let value = i32::try_from(value).expect("native CursorBaseSize fits gfx::Size");
            Some(SystemCursorSize {
                width_dip: value as u32,
                height_dip: value as u32,
            })
        }
    }
    struct Service {
        current: SystemCursorSize,
        next_id: u64,
        subscribers: Vec<(u64, mpsc::Sender<Command>)>,
    }
    fn service() -> &'static Mutex<Service> {
        static SERVICE: OnceLock<Mutex<Service>> = OnceLock::new();
        SERVICE.get_or_init(|| {
            // Source .cc:27,120-129: the enabled Windows producer starts at
            // kDefaultCursorSize, then replaces it with the real registry value.
            let initial = Key::open()
                .and_then(|key| key.size())
                .unwrap_or(SystemCursorSize {
                    width_dip: 32,
                    height_dip: 32,
                });
            std::thread::Builder::new()
                .name("system-cursor-size".into())
                .spawn(observe)
                .expect("native cursor-size observer thread");
            Mutex::new(Service {
                current: initial,
                next_id: 0,
                subscribers: Vec::new(),
            })
        })
    }
    pub(super) fn current() -> SystemCursorSize {
        service().lock().unwrap().current
    }
    pub(super) fn subscribe(
        sender: mpsc::Sender<Command>,
        initial: SystemCursorSize,
    ) -> Subscription {
        let mut state = service().lock().unwrap();
        if state.current != initial {
            let _ = sender.send(Command::SystemCursorSize(state.current));
        }
        state.next_id += 1;
        let id = state.next_id;
        state.subscribers.push((id, sender));
        Subscription { id }
    }
    pub(super) fn unsubscribe(id: u64) {
        service()
            .lock()
            .unwrap()
            .subscribers
            .retain(|(current, _)| *current != id);
    }
    fn publish(size: SystemCursorSize) {
        let mut state = service().lock().unwrap();
        if state.current == size {
            return;
        }
        state.current = size;
        state
            .subscribers
            .retain(|(_, sender)| sender.send(Command::SystemCursorSize(size)).is_ok());
    }
    fn observe() {
        let event = unsafe { CreateEventW(ptr::null_mut(), 0, 0, ptr::null()) };
        if event.is_null() {
            return;
        }
        let event = Event(event);
        // Drop the key before its event: RegCloseKey cancels its outstanding
        // native notification before CloseHandle releases the signal target.
        let Some(key) = Key::open() else {
            return;
        };
        let arm = || unsafe { RegNotifyChangeKeyValue(key.0, 0, 4, event.0, 1) } == 0;
        if !arm() {
            return;
        }
        // Source .cc:120-129 registers the watch before its initial read, so a
        // settings change racing with publication cannot leave stale metrics.
        if let Some(size) = key.size() {
            publish(size);
        }
        // Source .cc:99-114 rearms the one-shot registry watch after each
        // callback. Only this host worker waits; Pages receive queued values.
        while unsafe { WaitForSingleObject(event.0, u32::MAX) } == 0 {
            if let Some(size) = key.size() {
                publish(size);
            }
            if !arm() {
                break;
            }
        }
    }
}
