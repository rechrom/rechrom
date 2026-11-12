Upstream: hyper-util 0.1.21, unchanged crates.io release (Cargo.lock).
Original crate SHA-256: ddc03d96684f9226b8a787cdb71488417b53ab5ea8fdb1dac946cb9431cc8bff.

Local patch: `ConnectionCheck` / `CheckConnection` request extension, awaited
in legacy Client::try_send_request after connection_for and capture metadata,
before HTTP/1 target normalization and either protocol send path. Every request,
including pool reuse, runs the same asynchronous check with original Connected.
No DNS, permission, URL, or browser policy is implemented in this library.

`CaptureSendStart` retains the monotonic timestamp immediately before the
protocol send operation, after connection checkout and async admission. HTTP
resource timing consumes this original send timestamp; no enqueue or Page poll
time is substituted. No browser timing policy is implemented in this library.
