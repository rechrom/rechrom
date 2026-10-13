//! net::ReferrerPolicyFromHeader and URLRequestJob::ComputeReferrerForPolicy.
//! Pure request policy; document ownership and policy installation stay upstream.
use url::Url;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    #[default]
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

#[allow(non_snake_case)]
impl ReferrerPolicy {
    /// Modern element/meta policy keywords. Unknown values inherit the policy.
    pub fn FromString(value: &str) -> Option<Self> {
        match value
            .trim_matches(|c: char| c.is_ascii_whitespace())
            .to_ascii_lowercase()
            .as_str()
        {
            "no-referrer" => Some(Self::NoReferrer),
            "no-referrer-when-downgrade" => Some(Self::NoReferrerWhenDowngrade),
            "origin" => Some(Self::Origin),
            "origin-when-cross-origin" => Some(Self::OriginWhenCrossOrigin),
            "same-origin" => Some(Self::SameOrigin),
            "strict-origin" => Some(Self::StrictOrigin),
            "strict-origin-when-cross-origin" => Some(Self::StrictOriginWhenCrossOrigin),
            "unsafe-url" => Some(Self::UnsafeUrl),
            _ => None,
        }
    }

    /// A response header uses the last recognized comma-delimited policy.
    pub fn FromHeader(value: &str) -> Option<Self> {
        value.split(',').rev().find_map(Self::FromString)
    }

    pub fn GenerateReferrer(self, source: &str, destination: &str) -> String {
        if self == Self::NoReferrer {
            return String::new();
        }
        let (Ok(mut source), Ok(destination)) = (Url::parse(source), Url::parse(destination))
        else {
            return String::new();
        };
        // GURL::GetAsReferrer permits HTTP(S), and strips credentials/fragment.
        if !matches!(source.scheme(), "http" | "https") {
            return String::new();
        }
        let same_origin = source.origin() == destination.origin();
        let downgrade =
            source.scheme() == "https" && !matches!(destination.scheme(), "https" | "wss");
        if (downgrade
            && matches!(
                self,
                Self::NoReferrerWhenDowngrade
                    | Self::StrictOrigin
                    | Self::StrictOriginWhenCrossOrigin
            ))
            || (self == Self::SameOrigin && !same_origin)
        {
            return String::new();
        }
        let _ = source.set_username("");
        let _ = source.set_password(None);
        source.set_fragment(None);
        let origin_only = source.as_str().len() > 4096
            || matches!(self, Self::Origin | Self::StrictOrigin)
            || (!same_origin
                && matches!(
                    self,
                    Self::OriginWhenCrossOrigin | Self::StrictOriginWhenCrossOrigin
                ));
        if origin_only {
            format!("{}/", source.origin().ascii_serialization())
        } else {
            source.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognized_header_fallback_and_element_keywords() {
        assert_eq!(
            ReferrerPolicy::FromHeader("origin, unknown, SAME-ORIGIN, unknown"),
            Some(ReferrerPolicy::SameOrigin)
        );
        assert_eq!(ReferrerPolicy::FromString("origin, no-referrer"), None);
        assert_eq!(ReferrerPolicy::FromHeader("never, default"), None);
    }

    #[test]
    fn policy_matrix_strips_credentials_fragments_and_long_sources() {
        use ReferrerPolicy::*;
        let source = "https://user:secret@a.test:443/private?q=1#fragment";
        for (policy, cross, downgrade) in [
            (NoReferrer, "", ""),
            (NoReferrerWhenDowngrade, "https://a.test/private?q=1", ""),
            (Origin, "https://a.test/", "https://a.test/"),
            (OriginWhenCrossOrigin, "https://a.test/", "https://a.test/"),
            (SameOrigin, "", ""),
            (StrictOrigin, "https://a.test/", ""),
            (StrictOriginWhenCrossOrigin, "https://a.test/", ""),
            (
                UnsafeUrl,
                "https://a.test/private?q=1",
                "https://a.test/private?q=1",
            ),
        ] {
            assert_eq!(policy.GenerateReferrer(source, "https://b.test/"), cross);
            assert_eq!(policy.GenerateReferrer(source, "http://b.test/"), downgrade);
            let same = policy.GenerateReferrer(source, "https://a.test:443/image");
            assert_eq!(
                same,
                match policy {
                    NoReferrer => "",
                    Origin | StrictOrigin => "https://a.test/",
                    _ => "https://a.test/private?q=1",
                }
            );
        }
        assert_eq!(
            UnsafeUrl.GenerateReferrer(
                &format!("https://a.test/{}", "x".repeat(4096)),
                "https://a.test/"
            ),
            "https://a.test/"
        );
        assert_eq!(
            UnsafeUrl.GenerateReferrer("file:///private/path", "https://a.test/"),
            ""
        );
    }
}
