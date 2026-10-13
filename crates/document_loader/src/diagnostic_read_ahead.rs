//! Opt-in measurement only. Holds main-response bytes while the authoritative
//! parser is paused; it never creates requests, DOM nodes, or load blockers.
use html::diagnostic_scanner::DiagnosticHTMLScanner;
use std::{collections::VecDeque, io};
use url_loader::{URLLoadEvent, URLStreamOperation};

const LIMIT: usize = 1024 * 1024;
const TURN: usize = 16 * 1024;
type Pending = Result<URLLoadEvent, (io::ErrorKind, String)>;
pub(crate) struct DiagnosticReadAhead {
    scanner: Option<DiagnosticHTMLScanner>,
    queue: VecDeque<Pending>,
    queued_bytes: usize,
    ahead_total: usize,
    terminal: bool,
    observations: usize,
    pending_scan: bool,
    dropped_observations: usize,
    pub(crate) pumped: usize,
    pub(crate) appended: usize,
}
impl DiagnosticReadAhead {
    pub(crate) fn new(encoding: &str, scripting: bool) -> io::Result<Self> {
        Ok(Self {
            scanner: Some(DiagnosticHTMLScanner::new(encoding, scripting)?),
            queue: VecDeque::new(), queued_bytes: 0, ahead_total: 0,
            terminal: false, observations: 0, pending_scan: false, dropped_observations: 0, pumped: 0, appended: 0,
        })
    }
    pub(crate) fn cancel_scan(&mut self, reason: u8) {
        self.pending_scan = false;
        if self.scanner.take().is_some() {
            browser_tracing::instant("loading", "DiagnosticScanCancelled", &[("reason", reason as f64)]);
        }
    }
    pub(crate) fn observe(&mut self, bytes: &[u8], paused: bool) {
        self.pumped += bytes.len();
        for chunk in bytes.chunks(TURN) {
            if let Some(scanner) = &mut self.scanner {
                if scanner.AppendBytes(chunk).is_err() {
                    // Scanner decode/cap errors are diagnostic only. The real
                    // decoder encounters the original bytes in original order.
                    self.cancel_scan(3);
                    break;
                }
            }
            self.scan(paused);
        }
    }
    fn scan(&mut self, paused: bool) -> bool {
        let Some(scanner) = &mut self.scanner else { return false; };
        let result = scanner.Scan(512);
        self.pending_scan = result.work_pending;
        for candidate in result.observations {
            let source = candidate.raw_src.trim();
            // This is trace identity only, not URL eligibility or resolution.
            // Avoid recording credentials, fragments, or inline URL payloads.
            let allowed_scheme = !source.contains(':') || source.starts_with("https://") || source.starts_with("http://");
            if self.observations >= 128 || source.contains('@') || !allowed_scheme {
                self.dropped_observations += 1;
                continue;
            }
            let source = source.split('#').next().unwrap_or("");
            self.observations += 1;
            browser_tracing::instant_with_text("loading", "DiagnosticImageCandidate", source, &[
                ("source_utf16_end", candidate.consumed_utf16 as f64),
                ("bytes_fed", result.bytes_fed as f64),
                ("policy_epoch", candidate.policy_epoch as f64),
                ("has_base", candidate.has_base as u8 as f64),
                ("in_picture", candidate.in_picture as u8 as f64),
                ("parser_paused", paused as u8 as f64),
                ("appended_bytes", self.appended as f64),
            ]);
        }
        browser_tracing::instant("loading", "DiagnosticScanProgress", &[
            ("pumped_bytes", self.pumped as f64),
            ("appended_bytes", self.appended as f64),
            ("queued_bytes", self.queued_bytes as f64),
            ("bytes_fed", result.bytes_fed as f64),
            ("consumed_utf16", result.consumed_utf16 as f64),
            ("parser_paused", paused as u8 as f64),
            ("work_pending", result.work_pending as u8 as f64),
            ("dropped_observations", self.dropped_observations as f64),
        ]);
        result.work_pending
    }
    pub(crate) fn read_ahead(&mut self, stream: &mut dyn URLStreamOperation) -> bool {
        let mut remaining = TURN.min(LIMIT.saturating_sub(self.ahead_total));
        self.scan(true);
        if self.terminal || self.scanner.is_none() || remaining == 0 { return self.pending_scan; }
        while remaining > 0 {
            match stream.Poll(remaining) {
                Ok(None) => break,
                Ok(Some(URLLoadEvent::Data(bytes))) => {
                    if bytes.is_empty() || bytes.len() > remaining {
                        self.queue.push_back(Err((io::ErrorKind::InvalidData, "invalid document body chunk".into())));
                        self.terminal = true;
                        break;
                    }
                    remaining -= bytes.len();
                    self.queued_bytes += bytes.len();
                    self.ahead_total += bytes.len();
                    self.observe(&bytes, true);
                    self.queue.push_back(Ok(URLLoadEvent::Data(bytes)));
                }
                Ok(Some(event)) => {
                    if matches!(event, URLLoadEvent::Finished) {
                        if let Some(scanner) = &mut self.scanner {
                            if scanner.FinishBytes().is_err() { self.cancel_scan(3); }
                        }
                        self.scan(true);
                    }
                    // EOF and errors must be replayed only when the real
                    // parser asks for more bytes; do not finish it while paused.
                    self.queue.push_back(Ok(event));
                    self.terminal = true;
                    break;
                }
                Err(error) => {
                    self.queue.push_back(Err((error.kind(), error.to_string())));
                    self.terminal = true;
                    break;
                }
            }
        }
        browser_tracing::instant("loading", "DiagnosticReadAhead", &[
            ("pumped_bytes", self.pumped as f64), ("appended_bytes", self.appended as f64),
            ("queued_bytes", self.queued_bytes as f64), ("ahead_total", self.ahead_total as f64),
            ("terminal_held", self.terminal as u8 as f64),
        ]);
        self.pending_scan || (remaining == 0 && self.ahead_total < LIMIT && self.scanner.is_some())
    }
    pub(crate) fn report(&self) {
        browser_tracing::instant("loading", "DiagnosticInputTotals", &[
            ("pumped_bytes", self.pumped as f64), ("appended_bytes", self.appended as f64),
            ("queued_bytes", self.queued_bytes as f64), ("ahead_total", self.ahead_total as f64),
            ("scan_active", self.scanner.is_some() as u8 as f64),
        ]);
    }
    pub(crate) fn poll_queued(&mut self, max: usize) -> Option<io::Result<URLLoadEvent>> {
        if let Some(Ok(URLLoadEvent::Data(bytes))) = self.queue.front_mut() {
            if bytes.len() > max {
                self.queued_bytes -= max;
                return Some(Ok(URLLoadEvent::Data(bytes.drain(..max).collect())));
            }
        }
        self.queue.pop_front().map(|event| match event {
            Ok(URLLoadEvent::Data(bytes)) => { self.queued_bytes -= bytes.len(); Ok(URLLoadEvent::Data(bytes)) }
            Ok(event) => Ok(event),
            Err((kind, text)) => Err(io::Error::new(kind, text)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct FailedStream;
    impl URLStreamOperation for FailedStream {
        fn Poll(&mut self, _: usize) -> io::Result<Option<URLLoadEvent>> {
            Err(io::Error::new(io::ErrorKind::ConnectionReset, "test reset"))
        }
    }
    #[test]
    fn ahead_error_is_held_until_authoritative_demand() {
        let mut diagnostic = DiagnosticReadAhead::new("utf-8", true).unwrap();
        diagnostic.read_ahead(&mut FailedStream);
        assert!(diagnostic.terminal);
        assert!(diagnostic.scanner.is_some());
        assert_eq!(diagnostic.poll_queued(16).unwrap().unwrap_err().kind(), io::ErrorKind::ConnectionReset);
        assert!(diagnostic.poll_queued(16).is_none());
    }
}
