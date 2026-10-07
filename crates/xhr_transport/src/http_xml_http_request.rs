use crate::xml_http_request_transport::*;
use http_transport::{
    HTTPRequestData, HTTPRequestOperation, HTTPRequestTransport, HTTPTransportOptions,
    StartHTTPRequest,
};
use std::io;

struct Operation {
    request: Box<dyn HTTPRequestOperation>,
}

// cpp: xhr_transport/http_xml_http_request.cc:17-24
// cpp: xhr_transport/http_xml_http_request.cc:44-51
impl XMLHttpRequestOperation for Operation {
    fn Poll(&mut self) -> io::Result<Option<XMLHttpResponseData>> {
        Ok(self.request.Poll()?.map(|response| XMLHttpResponseData {
            status: response.status,
            final_url: response.final_url,
            mime_type: response.mime_type,
            body: response.body,
        }))
    }
}

fn HTTPRequest(request: &XMLHttpRequestData) -> HTTPRequestData {
    HTTPRequestData {
        method: request.method.clone(),
        url: request.url.clone(),
        referrer: request.referrer.clone(),
        body: request.body.clone(),
        headers: request.headers.clone(),
    }
}

// cpp: xhr_transport/http_xml_http_request.cc:11-31
// cpp: xhr_transport/http_xml_http_request.h:8-9
pub fn StartHTTPXMLHttpRequest(
    request: &XMLHttpRequestData,
    defaults: &HTTPTransportOptions,
) -> io::Result<Box<dyn XMLHttpRequestOperation>> {
    Ok(Box::new(Operation {
        request: StartHTTPRequest(&HTTPRequest(request), defaults)?,
    }))
}

struct Transport {
    transport: HTTPRequestTransport,
}
// cpp: xhr_transport/http_xml_http_request.cc:39-59
impl XMLHttpRequestTransport for Transport {
    fn Start(
        &mut self,
        request: &XMLHttpRequestData,
    ) -> io::Result<Box<dyn XMLHttpRequestOperation>> {
        Ok(Box::new(Operation {
            request: self.transport.Start(&HTTPRequest(request))?,
        }))
    }
}

// cpp: xhr_transport/http_xml_http_request.cc:33-38,63-64
// cpp: xhr_transport/http_xml_http_request.h:10-11
pub fn CreateHTTPXMLHttpRequestTransport(
    defaults: &HTTPTransportOptions,
) -> io::Result<Box<dyn XMLHttpRequestTransport>> {
    Ok(Box::new(Transport {
        transport: HTTPRequestTransport::new(defaults.clone())?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    #[test]
    fn http_adapter_preserves_request_and_response_bytes() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/endpoint", server.local_addr().unwrap());
        let served = std::thread::spawn(move || {
            let (mut socket, _) = server.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut input = Vec::new();
            let mut buffer = [0; 4096];
            loop {
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0);
                input.extend_from_slice(&buffer[..count]);
                if let Some(end) = input.windows(4).position(|w| w == b"\r\n\r\n") {
                    if input.len() >= end + 4 + 3 {
                        break;
                    }
                }
            }
            let headers = String::from_utf8_lossy(&input).to_ascii_lowercase();
            assert!(headers.starts_with("post /endpoint http/"));
            assert!(headers.contains("x-probe: adapter"));
            assert!(headers.contains("referer: https://example.test/"));
            assert!(input.ends_with(&[0, 255, b'x']));
            socket.write_all(b"HTTP/1.1 201 Created\r\nContent-Type: application/octet-stream\r\nContent-Length: 3\r\nConnection: close\r\n\r\n\0\xffx").unwrap();
        });
        let options = HTTPTransportOptions {
            timeout_ms: 3000,
            max_retries: 0,
            ..Default::default()
        };
        let mut transport = CreateHTTPXMLHttpRequestTransport(&options).unwrap();
        let mut operation = transport
            .Start(&XMLHttpRequestData {
                method: "POST".into(),
                url: url.clone(),
                referrer: "https://example.test/".into(),
                body: vec![0, 255, b'x'],
                headers: vec![("X-Probe".into(), "adapter".into())],
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let response = loop {
            if let Some(response) = operation.Poll().unwrap() {
                break response;
            }
            assert!(Instant::now() < deadline, "response remained pending");
            std::thread::sleep(Duration::from_millis(1));
        };
        served.join().unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.final_url, url);
        assert_eq!(response.mime_type, "application/octet-stream");
        assert_eq!(response.body, vec![0, 255, b'x']);
    }
}
