use crate::http_request::{HTTPRequestData, HTTPResponseData, StartHTTPRequest};
use crate::http_transport_options::HTTPTransportOptions;
use std::{io, thread, time::Duration};

pub type Response = HTTPResponseData;

// The static renderer's synchronous adapter waits for the package's
// nonblocking operation. It does not alter transport admission or retries.
pub fn GetResponse(url: &str) -> io::Result<Response> {
    let request = HTTPRequestData {
        method: "GET".to_owned(),
        url: url.to_owned(),
        ..HTTPRequestData::default()
    };
    let mut operation = StartHTTPRequest(&request, &HTTPTransportOptions::default())?;
    loop {
        if let Some(response) = operation.Poll()? {
            return Ok(response);
        }
        thread::sleep(Duration::from_millis(10));
    }
}

pub fn Get(url: &str) -> io::Result<Vec<u8>> {
    Ok(GetResponse(url)?.body)
}
