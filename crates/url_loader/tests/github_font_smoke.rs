use std::thread;
use std::time::{Duration, Instant};
use url_loader::{
    DefaultURLLoader, DefaultURLLoaderOptions, RequestDestination, URLLoader, URLRequest,
};

#[test]
fn github_font_body_completes() {
    let mut options = DefaultURLLoaderOptions::default();
    options.trace_requests = true;
    let mut loader = DefaultURLLoader::new(options).unwrap();
    let mut operation = loader
        .Load(&URLRequest {
            url: "https://github.githubassets.com/assets/MonaSansVF-wdth-wght-opsz-7a868ff47c93bd97.woff2".into(),
            destination: RequestDestination::kFont,
            ..URLRequest::default()
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(response) = operation.Poll().unwrap() {
            assert_eq!(response.status_code, 200);
            assert_eq!(response.body.len(), 219_388);
            break;
        }
        assert!(Instant::now() < deadline, "font body did not complete");
        thread::sleep(Duration::from_millis(10));
    }
}
