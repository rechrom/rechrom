use browser_tracing::{Context, RecordedSpan, Snapshot, StartOptions};
use http_transport::{
    HTTPRequestData, HTTPRequestTransport, HTTPStreamEvent, HTTPTransportOptions,
};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn wait_for_trace(recording: u64, ready: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let trace = browser_tracing::snapshot(Some(recording)).unwrap();
        if ready(&trace) {
            return trace;
        }
        assert!(Instant::now() < deadline, "HTTP trace event timed out");
        thread::sleep(Duration::from_millis(1));
    }
}

fn has(trace: &Snapshot, name: &str) -> bool {
    trace.instants.iter().any(|event| event.name == name)
}

fn request_events(trace: &Snapshot, id: f64) -> Vec<&RecordedSpan> {
    trace
        .instants
        .iter()
        .filter(|event| event.fields.get("request_id") == Some(&id))
        .collect()
}

// One test owns the process-global recorder. Transport events must retain the
// caller's target on Tokio IO/codec threads and finish without a consumer Poll.
#[test]
fn trace_separates_network_decode_admission_and_delivery() {
    let plain = b"a compressed response whose completion must precede its consumer poll";
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Default::default());
    encoder.write_all(plain).unwrap();
    let encoded = encoder.finish().unwrap();
    let encoded_len = encoded.len();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (release, wait) = mpsc::channel();
    let server = thread::spawn(move || {
        for index in 0..2 {
            let (mut socket, _) = listener.accept().unwrap();
            let mut input = BufReader::new(socket.try_clone().unwrap());
            loop {
                let mut line = String::new();
                assert_ne!(input.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
            }
            write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                encoded.len()
            )
            .unwrap();
            if index == 0 {
                wait.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            socket.write_all(&encoded).unwrap();
        }
    });

    let mut transport = HTTPRequestTransport::new(HTTPTransportOptions {
        max_parallel_requests: 1,
        max_retries: 0,
        ..Default::default()
    })
    .unwrap();
    let recording = browser_tracing::start(StartOptions {
        max_events: 1024,
        target_id: 731,
    })
    .unwrap();
    let _scope = browser_tracing::scope(Context {
        target_id: 731,
        source_id: 19,
        ..Default::default()
    });
    let request = |path: &str| HTTPRequestData {
        method: "GET".into(),
        url: format!("http://trace-user:trace-password@{address}/{path}#fragment"),
        ..Default::default()
    };
    let mut buffered = transport.Start(&request("buffered")).unwrap();
    wait_for_trace(recording, |trace| has(trace, "HTTPResponseHeaders"));
    let mut streamed = transport.StartStream(&request("streamed")).unwrap();
    // This request cannot obtain the only admission slot before it is dropped.
    drop(transport.Start(&request("cancelled")).unwrap());
    let queued = wait_for_trace(recording, |trace| has(trace, "HTTPRequestCancelled"));
    let queued_event = |path: &str| {
        queued
            .instants
            .iter()
            .find(|event| {
                event.name == "HTTPRequestQueued"
                    && event.text.as_deref() == Some(format!("http://{address}/{path}").as_str())
            })
            .unwrap()
            .fields["request_id"]
    };
    let buffered_id = queued_event("buffered");
    let streamed_id = queued_event("streamed");
    let cancelled_id = queued_event("cancelled");
    assert!(!request_events(&queued, streamed_id)
        .iter()
        .any(|event| event.name == "HTTPRequestAdmitted"));
    release.send(()).unwrap();

    let completed = wait_for_trace(recording, |trace| {
        has(trace, "HTTPResponseReady") && has(trace, "HTTPStreamComplete")
    });
    for id in [buffered_id, streamed_id] {
        let events = request_events(&completed, id);
        let event = |name: &str| *events.iter().find(|event| event.name == name).unwrap();
        assert!(event("HTTPRequestQueued").start_ns <= event("HTTPRequestAdmitted").start_ns);
        assert!(event("HTTPRequestAdmitted").start_ns <= event("HTTPRequestStart").start_ns);
        assert!(event("HTTPRequestStart").start_ns <= event("HTTPResponseHeaders").start_ns);
        assert!(event("HTTPResponseHeaders").start_ns <= event("HTTPNetworkBodyComplete").start_ns);
        assert!(event("HTTPNetworkBodyComplete").start_ns <= event("HTTPBodyComplete").start_ns);
        assert_eq!(
            event("HTTPBodyComplete").fields["encoded_bytes"],
            encoded_len as f64
        );
        assert_eq!(
            event("HTTPBodyComplete").fields["decoded_bytes"],
            plain.len() as f64
        );
        assert_eq!(event("HTTPBodyComplete").fields["network_eof"], 1.0);
        assert_eq!(
            events.iter().filter(|event| event.text.is_some()).count(),
            1
        );
        assert!(events
            .iter()
            .all(|event| { event.context.target_id == 731 && event.context.source_id == 19 }));
    }
    assert!(!has(&completed, "HTTPResponseDelivered"));
    assert!(!has(&completed, "HTTPStreamDelivered"));
    assert_eq!(buffered.Poll().unwrap().unwrap().body, plain);
    let mut body = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match streamed.Poll(7).unwrap() {
            Some(HTTPStreamEvent::Data(data)) => body.extend(data),
            Some(HTTPStreamEvent::Finished) => break,
            _ => {
                assert!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
    assert_eq!(body, plain);
    let mut invalid = transport
        .Start(&HTTPRequestData {
            method: "GET".into(),
            url: "file:///not-an-http-resource".into(),
            ..Default::default()
        })
        .unwrap();
    let failed = wait_for_trace(recording, |trace| has(trace, "HTTPRequestFailed"));
    let failure = failed
        .instants
        .iter()
        .find(|event| event.name == "HTTPRequestFailed")
        .unwrap();
    assert_eq!(failure.fields["will_retry"], 0.0);
    let deadline = Instant::now() + Duration::from_secs(5);
    while invalid.Poll().is_ok() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    browser_tracing::stop().unwrap();
    let final_trace = browser_tracing::snapshot(Some(recording)).unwrap();
    assert!(has(&final_trace, "HTTPResponseDelivered"));
    assert!(has(&final_trace, "HTTPStreamDelivered"));
    assert!(!request_events(&final_trace, cancelled_id)
        .iter()
        .any(|event| { matches!(event.name.as_str(), "HTTPRequestStart" | "HTTPBodyComplete") }));
    assert!(final_trace
        .instants
        .iter()
        .all(|event| event.fields.len() <= 8));
    server.join().unwrap();
}
