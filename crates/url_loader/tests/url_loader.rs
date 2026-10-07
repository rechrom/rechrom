use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use url_loader::{
    DefaultURLLoader, DefaultURLLoaderOptions, RequestDestination, URLLoadOperation, URLLoader,
    URLRequest,
};

fn wait(operation: &mut dyn URLLoadOperation) -> io::Result<url_loader::URLResponse> {
    for _ in 0..500 {
        if let Some(response) = operation.Poll()? {
            return Ok(response);
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "URL load test timed out",
    ))
}

#[test]
fn default_loader_preserves_scheme_bytes_and_response_metadata() {
    let mut loader = DefaultURLLoader::new(DefaultURLLoaderOptions::default()).unwrap();

    let request = URLRequest {
        url: "data:text/plain;charset=WINDOWS-1252;base64,SGVsbG8=".to_owned(),
        ..URLRequest::default()
    };
    let mut data = loader.Load(&request).unwrap();
    let response = wait(&mut *data).unwrap();
    assert_eq!(response.body, b"Hello");
    assert_eq!(response.mime_type, "text/plain");
    assert_eq!(response.text_encoding, "windows-1252");
    assert!(data.Poll().unwrap().is_none());

    loader
        .RegisterMemoryURL(
            "memory:logo".to_owned(),
            "image/png".to_owned(),
            vec![0, 255, 1],
            "utf-8".to_owned(),
        )
        .unwrap();
    assert!(loader
        .RegisterMemoryURL(
            "memory:logo".to_owned(),
            "image/png".to_owned(),
            vec![],
            "utf-8".to_owned(),
        )
        .is_err());
    let mut memory = loader
        .Load(&URLRequest {
            url: "memory:logo".to_owned(),
            destination: RequestDestination::kImage,
            ..URLRequest::default()
        })
        .unwrap();
    assert_eq!(wait(&mut *memory).unwrap().body, [0, 255, 1]);

    let path = std::env::temp_dir().join(format!("layoutng url {}.bin", std::process::id()));
    std::fs::write(&path, [0, 255, 1]).unwrap();
    let file_url = format!("file://{}", path.to_string_lossy().replace(' ', "%20"));
    let mut file = loader
        .Load(&URLRequest {
            url: file_url.clone(),
            destination: RequestDestination::kFont,
            ..URLRequest::default()
        })
        .unwrap();
    let response = wait(&mut *file).unwrap();
    assert_eq!(response.body, [0, 255, 1]);
    assert_eq!(response.final_url, file_url);
    assert_eq!(response.mime_type, "application/octet-stream");
    std::fs::remove_file(path).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut input = [0_u8; 2048];
            let size = stream.read(&mut input).unwrap();
            let request = String::from_utf8_lossy(&input[..size]);
            let response = if request.starts_with("GET /start ") {
                "HTTP/1.1 302 Found\r\nLocation: /style.css\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            } else {
                assert!(request.starts_with("GET /style.css "));
                "HTTP/1.1 200 OK\r\nContent-Type: text/css; charset=Windows-1252\r\nContent-Length: 4\r\nConnection: close\r\n\r\nbody"
            };
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    let mut http = loader
        .Load(&URLRequest {
            url: format!("http://127.0.0.1:{port}/start"),
            destination: RequestDestination::kStyleSheet,
            ..URLRequest::default()
        })
        .unwrap();
    let response = wait(&mut *http).unwrap();
    server.join().unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(
        response.final_url,
        format!("http://127.0.0.1:{port}/style.css")
    );
    assert_eq!(response.mime_type, "text/css");
    assert_eq!(response.text_encoding, "windows-1252");
    assert_eq!(response.body, b"body");
}

#[test]
fn streaming_sources_deliver_headers_bounded_chunks_and_one_eof() {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    use url_loader::{DefaultURLLoader, URLLoadEvent, URLLoader, URLRequest};
    let bytes = b"<p>hello</p>";
    let path = std::env::temp_dir().join(format!(
        "url-stream-{}-{}.html",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, bytes).unwrap();
    let mut loader = DefaultURLLoader::new(Default::default()).unwrap();
    loader
        .RegisterMemoryURL(
            "memory:stream".into(),
            "text/html".into(),
            bytes.to_vec(),
            "utf-8".into(),
        )
        .unwrap();
    for url in [
        "memory:stream".to_owned(),
        "data:text/html,%3Cp%3Ehello%3C%2Fp%3E".into(),
        format!("file://{}", path.display()),
    ] {
        let mut stream = loader
            .LoadStream(&URLRequest {
                url,
                ..Default::default()
            })
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut header = false;
        let mut body = vec![];
        loop {
            match stream.Poll(3).unwrap() {
                Some(URLLoadEvent::Response(head)) => {
                    assert!(!header);
                    header = true;
                    assert_eq!(head.mime_type, "text/html");
                }
                Some(URLLoadEvent::Data(data)) => {
                    assert!(header);
                    assert!(data.len() <= 3);
                    body.extend(data);
                }
                Some(URLLoadEvent::Finished) => {
                    assert!(header);
                    break;
                }
                None => {
                    assert!(Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        assert_eq!(body, bytes);
        assert!(stream.Poll(3).unwrap().is_none());
    }
    let mut limited = DefaultURLLoader::new(url_loader::DefaultURLLoaderOptions {
        max_response_bytes: 3,
        ..Default::default()
    })
    .unwrap();
    let mut stream = limited
        .LoadStream(&URLRequest {
            url: format!("file://{}", path.display()),
            ..Default::default()
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut header = false;
    loop {
        match stream.Poll(3) {
            Ok(Some(URLLoadEvent::Response(_))) => header = true,
            Err(error) => {
                assert!(header);
                assert!(error.to_string().contains("size limit"));
                break;
            }
            Ok(None) => {
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(1));
            }
            other => panic!("unexpected over-limit delivery: {other:?}"),
        }
    }
    std::fs::remove_file(path).unwrap();
}
