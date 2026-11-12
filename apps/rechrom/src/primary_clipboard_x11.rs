//! ICCCM SelectionRequester/SelectionOwner counterparts for the independent
//! PRIMARY and CLIPBOARD selections (ui/base/x/selection_{requester,owner}.cc).
use super::{ClipboardHostError, PrimaryClipboardSnapshot};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{xproto::*, Event},
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
    COPY_DEPTH_FROM_PARENT, CURRENT_TIME, NONE,
};
type ReadResult<T> = Result<T, ClipboardHostError>;
fn failed<T>(_: T) -> ClipboardHostError {
    ClipboardHostError::ReadFailed
}
struct Requestor {
    connection: RustConnection,
    window: Window,
    property: Atom,
    incr: Atom,
    selection: Atom,
}
impl Requestor {
    fn atom(&self, name: &[u8]) -> ReadResult<Atom> {
        Ok(self
            .connection
            .intern_atom(false, name)
            .map_err(failed)?
            .reply()
            .map_err(failed)?
            .atom)
    }
    // ICCCM selection conversion, including incremental transfers. Like
    // Chromium's SelectionRequester this owns a requestor window and never
    // substitutes an empty value for refusal, owner loss, or timeout.
    fn read(&self, target: Atom, expected: Atom) -> ReadResult<Vec<u8>> {
        let deadline = Instant::now() + Duration::from_secs(5);
        self.connection
            .delete_property(self.window, self.property)
            .map_err(failed)?;
        self.connection
            .convert_selection(
                self.window,
                self.selection,
                target,
                self.property,
                CURRENT_TIME,
            )
            .map_err(failed)?;
        self.connection.flush().map_err(failed)?;
        let mut incremental = false;
        let mut bytes = Vec::new();
        loop {
            if Instant::now() >= deadline {
                return Err(ClipboardHostError::ReadFailed);
            }
            let Some(event) = self.connection.poll_for_event().map_err(failed)? else {
                std::thread::park_timeout(Duration::from_millis(1));
                continue;
            };
            match event {
                Event::SelectionNotify(event)
                    if event.requestor == self.window
                        && event.selection == self.selection
                        && event.target == target =>
                {
                    if event.property == NONE {
                        return Err(ClipboardHostError::ReadFailed);
                    }
                    let reply = self
                        .connection
                        .get_property(true, self.window, self.property, AtomEnum::ANY, 0, u32::MAX)
                        .map_err(failed)?
                        .reply()
                        .map_err(failed)?;
                    if reply.type_ == self.incr {
                        incremental = true;
                        self.connection.flush().map_err(failed)?;
                    } else {
                        if reply.type_ != expected {
                            return Err(ClipboardHostError::ReadFailed);
                        }
                        return Ok(reply.value);
                    }
                }
                Event::PropertyNotify(event)
                    if incremental
                        && event.window == self.window
                        && event.atom == self.property
                        && event.state == Property::NEW_VALUE =>
                {
                    let reply = self
                        .connection
                        .get_property(true, self.window, self.property, AtomEnum::ANY, 0, u32::MAX)
                        .map_err(failed)?
                        .reply()
                        .map_err(failed)?;
                    if reply.type_ != expected {
                        return Err(ClipboardHostError::ReadFailed);
                    }
                    self.connection.flush().map_err(failed)?;
                    if reply.value.is_empty() {
                        return Ok(bytes);
                    }
                    bytes.extend_from_slice(&reply.value);
                }
                _ => {}
            }
        }
    }
}
fn requestor(selection: &[u8]) -> ReadResult<Requestor> {
    let (connection, screen) = x11rb::connect(None).map_err(failed)?;
    let window = connection.generate_id().map_err(failed)?;
    connection
        .create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            connection.setup().roots[screen].root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )
        .map_err(failed)?
        .check()
        .map_err(failed)?;
    let mut requestor = Requestor {
        connection,
        window,
        property: 0,
        incr: 0,
        selection: 0,
    };
    requestor.property = requestor.atom(b"RECHROM_SELECTION")?;
    requestor.incr = requestor.atom(b"INCR")?;
    requestor.selection = requestor.atom(selection)?;
    Ok(requestor)
}
fn owner(requestor: &Requestor) -> ReadResult<Window> {
    Ok(requestor
        .connection
        .get_selection_owner(requestor.selection)
        .map_err(failed)?
        .reply()
        .map_err(failed)?
        .owner)
}
fn targets(requestor: &Requestor) -> ReadResult<Vec<Atom>> {
    let targets = requestor.read(requestor.atom(b"TARGETS")?, AtomEnum::ATOM.into())?;
    Ok(targets
        .chunks_exact(4)
        .map(|bytes| u32::from_ne_bytes(bytes.try_into().unwrap()))
        .collect())
}
fn text(requestor: &Requestor, targets: &[Atom]) -> ReadResult<Option<String>> {
    for name in [
        b"UTF8_STRING".as_slice(),
        b"text/plain;charset=utf-8",
        b"text/plain",
        b"STRING",
    ] {
        let target = requestor.atom(name)?;
        if !targets.contains(&target) {
            continue;
        }
        let bytes = requestor.read(target, target)?;
        // STRING is ISO-8859-1; UTF8_STRING and the UTF-8 MIME targets are
        // decoded with replacement, matching the clipboard text boundary.
        return Ok(Some(if name == b"STRING" {
            bytes.into_iter().map(char::from).collect()
        } else {
            String::from_utf8_lossy(&bytes).into_owned()
        }));
    }
    Ok(None)
}
pub(super) fn read_primary() -> ReadResult<String> {
    let requestor = requestor(b"PRIMARY")?;
    if owner(&requestor)? == NONE {
        return Err(ClipboardHostError::Unavailable);
    }
    text(&requestor, &targets(&requestor)?)?.ok_or(ClipboardHostError::ReadFailed)
}

#[derive(Default)]
pub(super) struct Clipboard {
    writer: Mutex<Option<Writer>>,
}
impl Clipboard {
    pub(super) fn read(&self) -> ReadResult<PrimaryClipboardSnapshot> {
        let requestor = requestor(b"CLIPBOARD")?;
        let initial_owner = owner(&requestor)?;
        if initial_owner == NONE {
            return Err(ClipboardHostError::Unavailable);
        }
        let targets = targets(&requestor)?;
        let timestamp_type = requestor.atom(b"TIMESTAMP")?;
        let timestamp = if targets.contains(&timestamp_type) {
            Some(requestor.read(timestamp_type, AtomEnum::INTEGER.into())?)
        } else {
            None
        };
        let mut data = Vec::new();
        if let Some(text) = text(&requestor, &targets)? {
            data.push(("text/plain;charset=utf-8".to_owned(), text.into_bytes()));
        }
        for &name in super::linux::READ_FORMATS.iter().skip(4) {
            let target = requestor.atom(name.as_bytes())?;
            if targets.contains(&target) {
                data.push((
                    name.to_owned(),
                    if name == "chromium/x-webkit-paste" {
                        Vec::new()
                    } else {
                        requestor.read(target, target)?
                    },
                ));
            }
        }
        // A snapshot must not silently combine formats from different owners.
        if owner(&requestor)? != initial_owner {
            return Err(ClipboardHostError::ReadFailed);
        }
        if let Some(timestamp) = timestamp {
            if requestor.read(timestamp_type, AtomEnum::INTEGER.into())? != timestamp {
                return Err(ClipboardHostError::ReadFailed);
            }
        }
        super::linux::decode(data)
    }
    pub(super) fn write(&self, snapshot: &PrimaryClipboardSnapshot) -> ReadResult<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| ClipboardHostError::WriteFailed)?;
        if writer.is_none() {
            *writer = Some(Writer::new()?);
        }
        let writer = writer.as_ref().unwrap();
        let (reply, receiver) = mpsc::channel();
        writer
            .sender
            .send(Command::Write(super::linux::encode(snapshot)?, reply))
            .map_err(|_| ClipboardHostError::WriteFailed)?;
        receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| ClipboardHostError::WriteFailed)?
    }
}
enum Command {
    Write(Vec<(String, Vec<u8>)>, mpsc::Sender<ReadResult<()>>),
    Exit,
}
struct Writer {
    sender: mpsc::Sender<Command>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Writer {
    fn new() -> ReadResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("rechrom-x11-clipboard".into())
            .spawn(move || owner_loop(receiver))
            .map_err(|_| ClipboardHostError::WriteFailed)?;
        Ok(Self {
            sender,
            worker: Some(worker),
        })
    }
}
impl Drop for Writer {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Exit);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
struct Transfer {
    target: Atom,
    data: Arc<[u8]>,
    offset: usize,
    deadline: Instant,
}
struct SelectionOwner {
    requestor: Requestor,
    targets: Atom,
    timestamp: Atom,
    multiple: Atom,
    atom_pair: Atom,
    time: Timestamp,
    data: HashMap<Atom, Arc<[u8]>>,
    transfers: HashMap<(Window, Atom), Transfer>,
}
impl SelectionOwner {
    fn new() -> ReadResult<Self> {
        let requestor = requestor(b"CLIPBOARD")?;
        Ok(Self {
            targets: requestor.atom(b"TARGETS")?,
            timestamp: requestor.atom(b"TIMESTAMP")?,
            multiple: requestor.atom(b"MULTIPLE")?,
            atom_pair: requestor.atom(b"ATOM_PAIR")?,
            requestor,
            time: CURRENT_TIME,
            data: HashMap::new(),
            transfers: HashMap::new(),
        })
    }
    fn server_time(&mut self) -> ReadResult<Timestamp> {
        let requestor = &self.requestor;
        requestor
            .connection
            .change_property8(
                PropMode::REPLACE,
                requestor.window,
                requestor.property,
                AtomEnum::STRING,
                &[],
            )
            .map_err(failed)?;
        requestor.connection.flush().map_err(failed)?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Some(event) = self.requestor.connection.poll_for_event().map_err(failed)? {
                if let Event::PropertyNotify(event) = &event {
                    if event.window == self.requestor.window
                        && event.atom == self.requestor.property
                    {
                        return Ok(event.time);
                    }
                }
                self.event(event)?;
            }
            std::thread::park_timeout(Duration::from_millis(1));
        }
        Err(ClipboardHostError::WriteFailed)
    }
    fn write(&mut self, formats: Vec<(String, Vec<u8>)>) -> ReadResult<()> {
        let mut data = HashMap::new();
        for (format, bytes) in formats {
            data.insert(self.requestor.atom(format.as_bytes())?, Arc::from(bytes));
        }
        let time = self.server_time()?;
        let requestor = &self.requestor;
        requestor
            .connection
            .set_selection_owner(requestor.window, requestor.selection, time)
            .map_err(failed)?
            .check()
            .map_err(failed)?;
        if owner(requestor)? != requestor.window {
            return Err(ClipboardHostError::WriteFailed);
        }
        self.data = data;
        self.time = time;
        Ok(())
    }
    fn chunk_size(&self) -> usize {
        self.requestor
            .connection
            .maximum_request_bytes()
            .saturating_sub(24)
            .min(64 * 1024)
    }
    fn convert(&mut self, window: Window, target: Atom, property: Atom) -> ReadResult<bool> {
        if self.transfers.contains_key(&(window, property)) {
            return Ok(false);
        }
        let requestor = &self.requestor;
        if target == self.targets {
            let mut targets: Vec<Atom> = self.data.keys().copied().collect();
            targets.extend([self.targets, self.timestamp, self.multiple]);
            requestor
                .connection
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    property,
                    AtomEnum::ATOM,
                    &targets,
                )
                .map_err(failed)?
                .check()
                .map_err(failed)?;
        } else if target == self.timestamp {
            requestor
                .connection
                .change_property32(
                    PropMode::REPLACE,
                    window,
                    property,
                    AtomEnum::INTEGER,
                    &[self.time],
                )
                .map_err(failed)?
                .check()
                .map_err(failed)?;
        } else if let Some(bytes) = self.data.get(&target) {
            if bytes.len() <= self.chunk_size() {
                requestor
                    .connection
                    .change_property8(PropMode::REPLACE, window, property, target, bytes)
                    .map_err(failed)?
                    .check()
                    .map_err(failed)?;
            } else {
                let length = u32::try_from(bytes.len()).map_err(failed)?;
                requestor
                    .connection
                    .change_window_attributes(
                        window,
                        &ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE),
                    )
                    .map_err(failed)?
                    .check()
                    .map_err(failed)?;
                requestor
                    .connection
                    .change_property32(
                        PropMode::REPLACE,
                        window,
                        property,
                        requestor.incr,
                        &[length],
                    )
                    .map_err(failed)?
                    .check()
                    .map_err(failed)?;
                self.transfers.insert(
                    (window, property),
                    Transfer {
                        target,
                        data: bytes.clone(),
                        offset: 0,
                        deadline: Instant::now() + Duration::from_secs(5),
                    },
                );
            }
        } else {
            return Ok(false);
        }
        Ok(true)
    }
    fn event(&mut self, event: Event) -> ReadResult<()> {
        match event {
            Event::SelectionRequest(event) if event.selection == self.requestor.selection => {
                let property = if event.property == NONE {
                    event.target
                } else {
                    event.property
                };
                let accepted = if event.time != CURRENT_TIME && time_before(event.time, self.time) {
                    false
                } else if event.target == self.multiple && event.property != NONE {
                    let pairs = self
                        .requestor
                        .connection
                        .get_property(
                            false,
                            event.requestor,
                            property,
                            self.atom_pair,
                            0,
                            u32::MAX,
                        )
                        .map_err(failed)?
                        .reply()
                        .map_err(failed)?;
                    if pairs.type_ != self.atom_pair
                        || pairs.format != 32
                        || pairs.value.len() % 8 != 0
                    {
                        false
                    } else {
                        let mut pairs: Vec<u32> = pairs.value32().unwrap().collect();
                        for pair in pairs.chunks_exact_mut(2) {
                            if pair[0] == self.multiple
                                || pair[1] == NONE
                                || !self.convert(event.requestor, pair[0], pair[1])?
                            {
                                pair[1] = NONE;
                            }
                        }
                        self.requestor
                            .connection
                            .change_property32(
                                PropMode::REPLACE,
                                event.requestor,
                                property,
                                self.atom_pair,
                                &pairs,
                            )
                            .map_err(failed)?
                            .check()
                            .map_err(failed)?;
                        true
                    }
                } else {
                    self.convert(event.requestor, event.target, property)?
                };
                self.requestor
                    .connection
                    .send_event(
                        false,
                        event.requestor,
                        EventMask::NO_EVENT,
                        SelectionNotifyEvent {
                            response_type: SELECTION_NOTIFY_EVENT,
                            sequence: 0,
                            time: event.time,
                            requestor: event.requestor,
                            selection: event.selection,
                            target: event.target,
                            property: if accepted { property } else { NONE },
                        },
                    )
                    .map_err(failed)?
                    .check()
                    .map_err(failed)?;
            }
            Event::PropertyNotify(event) if event.state == Property::DELETE => {
                let chunk_size = self.chunk_size();
                let key = (event.window, event.atom);
                if let Some(transfer) = self.transfers.get_mut(&key) {
                    let end = (transfer.offset + chunk_size).min(transfer.data.len());
                    self.requestor
                        .connection
                        .change_property8(
                            PropMode::REPLACE,
                            event.window,
                            event.atom,
                            transfer.target,
                            &transfer.data[transfer.offset..end],
                        )
                        .map_err(failed)?
                        .check()
                        .map_err(failed)?;
                    let finished = transfer.offset == transfer.data.len();
                    transfer.offset = end;
                    transfer.deadline = Instant::now() + Duration::from_secs(5);
                    if finished {
                        self.finish_transfer(key);
                    }
                }
            }
            Event::SelectionClear(event)
                if event.selection == self.requestor.selection
                    && !time_before(event.time, self.time) =>
            {
                self.data.clear();
            }
            _ => {}
        }
        self.requestor.connection.flush().map_err(failed)
    }
    fn finish_transfer(&mut self, key: (Window, Atom)) {
        self.transfers.remove(&key);
        if !self.transfers.keys().any(|(window, _)| *window == key.0) {
            let _ = self.requestor.connection.change_window_attributes(
                key.0,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::NO_EVENT),
            );
        }
    }
}
fn time_before(first: Timestamp, second: Timestamp) -> bool {
    (first.wrapping_sub(second) as i32) < 0
}
fn owner_loop(commands: mpsc::Receiver<Command>) {
    let Ok(mut owner) = SelectionOwner::new() else {
        return;
    };
    loop {
        while let Ok(command) = commands.try_recv() {
            match command {
                Command::Exit => return,
                Command::Write(data, reply) => {
                    let result = owner
                        .write(data)
                        .map_err(|_| ClipboardHostError::WriteFailed);
                    let _ = reply.send(result);
                }
            }
        }
        // Bad or vanished requestor windows must not terminate selection
        // ownership for all other native clients.
        loop {
            match owner.requestor.connection.poll_for_event() {
                Ok(Some(event)) => {
                    let _ = owner.event(event);
                }
                Ok(None) => break,
                Err(_) => return,
            }
        }
        let expired = owner
            .transfers
            .iter()
            .filter_map(|(key, transfer)| (transfer.deadline <= Instant::now()).then_some(*key))
            .collect::<Vec<_>>();
        for key in expired {
            owner.finish_transfer(key);
        }
        std::thread::park_timeout(Duration::from_millis(1));
    }
}
