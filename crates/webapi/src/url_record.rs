//! WHATWG URL records shared by location, URL and history host bindings.
use url::Url;

pub(crate) const URL: u8 = 0;
pub(crate) const SCHEME: u8 = 1;
pub(crate) const USER: u8 = 2;
pub(crate) const PASSWORD: u8 = 3;
pub(crate) const HOST: u8 = 5;
pub(crate) const PORT: u8 = 6;
pub(crate) const PATH: u8 = 7;
pub(crate) const QUERY: u8 = 8;
pub(crate) const FRAGMENT: u8 = 9;

pub(crate) struct URLRecord(Option<Url>);

impl URLRecord {
    pub(crate) fn new() -> Self {
        Self(None)
    }

    pub(crate) fn set(&mut self, value: &str) -> bool {
        // Retain the source adapter's NUL-terminated input behavior.
        let value = value.split('\0').next().unwrap_or_default();
        match Url::options().base_url(self.0.as_ref()).parse(value) {
            Ok(url) => {
                self.0 = Some(url);
                true
            }
            Err(_) => false,
        }
    }

    pub(crate) fn origin(&self) -> String {
        self.0
            .as_ref()
            .map_or_else(|| "null".into(), |url| url.origin().ascii_serialization())
    }

    pub(crate) fn part(&self, part: u8) -> String {
        let Some(url) = &self.0 else {
            return String::new();
        };
        match part {
            URL => url.as_str().into(),
            SCHEME => url.scheme().into(),
            USER => url.username().into(),
            PASSWORD => url.password().unwrap_or_default().into(),
            HOST => url.host_str().unwrap_or_default().into(),
            PORT => url.port().map_or_else(String::new, |port| port.to_string()),
            PATH => url.path().into(),
            QUERY => url.query().unwrap_or_default().into(),
            FRAGMENT => url.fragment().unwrap_or_default().into(),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_url_and_normalizes_default_port() {
        let mut record = URLRecord::new();
        assert!(record.set("https://example.com:443/a/b"));
        assert!(record.set("../c?q=天气#x"));
        assert_eq!(
            record.part(URL),
            "https://example.com/c?q=%E5%A4%A9%E6%B0%94#x"
        );
        assert_eq!(record.part(PORT), "");
        assert_eq!(record.origin(), "https://example.com");
    }

    #[test]
    fn handles_ipv6_credentials_and_opaque_origins() {
        let mut record = URLRecord::new();
        assert!(record.set("https://user:pass@[::1]:8443/path"));
        assert_eq!(record.part(HOST), "[::1]");
        assert_eq!(record.part(USER), "user");
        assert_eq!(record.part(PASSWORD), "pass");
        assert_eq!(record.origin(), "https://[::1]:8443");
        assert!(record.set("about:blank"));
        assert_eq!(record.origin(), "null");
        assert!(!record.set("relative"));
        assert!(!record.set("http://[invalid"));
    }
}
