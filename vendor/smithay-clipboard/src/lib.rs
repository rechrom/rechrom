//! Smithay Clipboard
//!
//! Provides access to the Wayland clipboard for gui applications. The user
//! should have surface around.

#![deny(clippy::all, clippy::if_not_else, clippy::enum_glob_use)]
use std::ffi::c_void;
use std::io::Result;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use sctk::reexports::calloop::channel::{self, Sender};
use sctk::reexports::client::backend::Backend;
use sctk::reexports::client::Connection;

mod mime;
mod state;
mod worker;

/// Immutable MIME flavors belonging to one standard-clipboard offer/source.
/// PRIMARY retains its existing, separate text-only API.
pub type ClipboardData = Vec<(String, Vec<u8>)>;

/// Access to a Wayland clipboard.
pub struct Clipboard {
    request_sender: Sender<worker::Command>,
    request_receiver: Receiver<Result<String>>,
    clipboard_thread: Option<std::thread::JoinHandle<()>>,
}

impl Clipboard {
    /// Read supported flavors from the same Wayland selection offer. Each
    /// invocation owns its reply channel so a timed-out transfer cannot satisfy
    /// a later clipboard read.
    pub fn load_formats(&self, formats: &[&str]) -> Result<ClipboardData> {
        let (reply, receive) = mpsc::channel();
        self.request_sender
            .send(worker::Command::LoadFormats(
                formats.iter().map(|format| (*format).to_owned()).collect(),
                reply,
            ))
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "clipboard is dead")
            })?;
        receive.recv_timeout(Duration::from_secs(5)).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "clipboard read timed out")
        })?
    }

    /// Publish all flavors on one native source, reporting missing focus,
    /// missing data-device support and a failed protocol flush to the caller.
    pub fn store_formats(&self, data: ClipboardData) -> Result<()> {
        let (reply, receive) = mpsc::channel();
        self.request_sender
            .send(worker::Command::StoreFormats(data, reply))
            .map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::BrokenPipe, "clipboard is dead")
            })?;
        receive.recv_timeout(Duration::from_secs(5)).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::TimedOut, "clipboard write timed out")
        })?
    }
    /// Creates new clipboard which will be running on its own thread with its
    /// own event queue to handle clipboard requests.
    ///
    /// # Safety
    ///
    /// `display` must be a valid `*mut wl_display` pointer, and it must remain
    /// valid for as long as `Clipboard` object is alive.
    pub unsafe fn new(display: *mut c_void) -> Self {
        let backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let connection = Connection::from_backend(backend);

        // Create channel to send data to clipboard thread.
        let (request_sender, rx_chan) = channel::channel();
        // Create channel to get data from the clipboard thread.
        let (clipboard_reply_sender, request_receiver) = mpsc::channel();

        let name = String::from("smithay-clipboard");
        let clipboard_thread = worker::spawn(name, connection, rx_chan, clipboard_reply_sender);

        Self {
            request_receiver,
            request_sender,
            clipboard_thread,
        }
    }

    /// Load clipboard data.
    ///
    /// Loads content from a clipboard on a last observed seat.
    pub fn load(&self) -> Result<String> {
        let _ = self.request_sender.send(worker::Command::Load);

        if let Ok(reply) = self.request_receiver.recv() {
            reply
        } else {
            // The clipboard thread is dead, however we shouldn't crash downstream, so
            // propogating an error.
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "clipboard is dead.",
            ))
        }
    }

    /// Store to a clipboard.
    ///
    /// Stores to a clipboard on a last observed seat.
    pub fn store<T: Into<String>>(&self, text: T) {
        let request = worker::Command::Store(text.into());
        let _ = self.request_sender.send(request);
    }

    /// Load primary clipboard data.
    ///
    /// Loads content from a  primary clipboard on a last observed seat.
    pub fn load_primary(&self) -> Result<String> {
        let _ = self.request_sender.send(worker::Command::LoadPrimary);

        if let Ok(reply) = self.request_receiver.recv() {
            reply
        } else {
            // The clipboard thread is dead, however we shouldn't crash downstream, so
            // propogating an error.
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "clipboard is dead.",
            ))
        }
    }

    /// Store to a primary clipboard.
    ///
    /// Stores to a primary clipboard on a last observed seat.
    pub fn store_primary<T: Into<String>>(&self, text: T) {
        let request = worker::Command::StorePrimary(text.into());
        let _ = self.request_sender.send(request);
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        // Shutdown smithay-clipboard.
        let _ = self.request_sender.send(worker::Command::Exit);
        if let Some(clipboard_thread) = self.clipboard_thread.take() {
            let _ = clipboard_thread.join();
        }
    }
}
