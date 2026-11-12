use std::io::{Error, ErrorKind, Result};
use std::sync::mpsc::Sender;

use sctk::reexports::calloop::channel::Channel;
use sctk::reexports::calloop::{channel, EventLoop};
use sctk::reexports::calloop_wayland_source::WaylandSource;
use sctk::reexports::client::globals::registry_queue_init;
use sctk::reexports::client::Connection;

use crate::state::{SelectionTarget, State};

/// Spawn a clipboard worker, which dispatches its own `EventQueue` and handles
/// clipboard requests.
pub fn spawn(
    name: String,
    display: Connection,
    rx_chan: Channel<Command>,
    worker_replier: Sender<Result<String>>,
) -> Option<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name(name)
        .spawn(move || {
            worker_impl(display, rx_chan, worker_replier);
        })
        .ok()
}

/// Clipboard worker thread command.
pub enum Command {
    /// Read a coherent set of MIME flavors from a standard selection offer.
    LoadFormats(Vec<String>, Sender<Result<crate::ClipboardData>>),
    /// Publish a standard selection source and acknowledge protocol submission.
    StoreFormats(crate::ClipboardData, Sender<Result<()>>),
    /// Store data to a clipboard.
    Store(String),
    /// Store data to a primary selection.
    StorePrimary(String),
    /// Load data from a clipboard.
    Load,
    /// Load primary selection.
    LoadPrimary,
    /// Shutdown the worker.
    Exit,
}

/// Handle clipboard requests.
fn worker_impl(
    connection: Connection,
    rx_chan: Channel<Command>,
    reply_tx: Sender<Result<String>>,
) {
    let (globals, event_queue) = match registry_queue_init(&connection) {
        Ok(data) => data,
        Err(_) => return,
    };

    let mut event_loop = EventLoop::<State>::try_new().unwrap();
    let loop_handle = event_loop.handle();

    let mut state = match State::new(
        &globals,
        &event_queue.handle(),
        loop_handle.clone(),
        reply_tx,
    ) {
        Some(state) => state,
        None => return,
    };

    let clipboard_connection = connection.clone();
    loop_handle
        .insert_source(rx_chan, move |event, _, state| {
            if let channel::Event::Msg(event) = event {
                match event {
                    Command::LoadFormats(formats, reply) => {
                        if let Err(err) = state.load_formats(formats, reply.clone()) {
                            let _ = reply.send(Err(err));
                        }
                    }
                    Command::StoreFormats(data, reply) => {
                        let result = state.store_formats(data).and_then(|()| {
                            clipboard_connection
                                .flush()
                                .map_err(|error| Error::new(ErrorKind::Other, error.to_string()))
                        });
                        let _ = reply.send(result);
                    }
                    Command::StorePrimary(contents) => {
                        state.store_selection(SelectionTarget::Primary, contents);
                    }
                    Command::Store(contents) => {
                        state.store_selection(SelectionTarget::Clipboard, contents);
                    }
                    Command::Load if state.data_device_manager_state.is_some() => {
                        if let Err(err) = state.load_selection(SelectionTarget::Clipboard) {
                            let _ = state.reply_tx.send(Err(err));
                        }
                    }
                    Command::LoadPrimary if state.data_device_manager_state.is_some() => {
                        if let Err(err) = state.load_selection(SelectionTarget::Primary) {
                            let _ = state.reply_tx.send(Err(err));
                        }
                    }
                    Command::Load | Command::LoadPrimary => {
                        let _ = state.reply_tx.send(Err(Error::new(
                            ErrorKind::Other,
                            "requested selection is not supported",
                        )));
                    }
                    Command::Exit => state.exit = true,
                }
            }
        })
        .unwrap();

    WaylandSource::new(connection, event_queue)
        .insert(loop_handle)
        .unwrap();

    loop {
        if event_loop.dispatch(None, &mut state).is_err() || state.exit {
            break;
        }
    }
}
