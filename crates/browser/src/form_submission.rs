//! Encodes the immutable, typed entry list completed by Interaction. The
//! browser does not reconstruct controls or formdata from DOM/string values.
use crate::page::NavigationRequest;
use std::io;
use url_loader::URLRequest;

pub(crate) fn BuildRequest(
    document_url: &str,
    parameters: &interaction::FormSubmissionParameters,
    entries: &interaction::dispatch::core::fileapi::form_data::FormData,
) -> io::Result<Option<NavigationRequest>> {
    use interaction::default::core::html::forms::html_form_element::SubmitMethod;
    if parameters.method == SubmitMethod::Dialog {
        return Ok(None);
    }
    let base = url::Url::parse(document_url).map_err(io::Error::other)?;
    let action = &parameters.action;
    let resolved_base = parameters
        .base_href
        .as_deref()
        .and_then(|href| base.join(href).ok())
        .unwrap_or_else(|| base.clone());
    let mut url = if action.is_empty() {
        base.clone()
    } else {
        resolved_base.join(&action).map_err(io::Error::other)?
    };
    if !matches!(url.scheme(), "http" | "https" | "file") {
        return Ok(None);
    }
    let encoded = entries.EncodeFormData(false);
    let mut request = URLRequest {
        referrer: document_url.into(),
        ..Default::default()
    };
    if parameters.method == SubmitMethod::Post {
        request.method = "POST".into();
        let enctype = &parameters.enctype;
        if enctype == "text/plain" {
            request.body = entries.EncodeFormData(true);
            request
                .headers
                .push(("Content-Type".into(), "text/plain".into()));
        } else if enctype == "multipart/form-data" {
            let boundary = format!("----RechromForm{}", interaction::event::NextEventIdentity());
            let encoded = entries.EncodeMultiPartFormData(&boundary);
            request.encoded_body = Some(encoded);
            request.headers.push((
                "Content-Type".into(),
                format!("multipart/form-data; boundary={boundary}"),
            ));
        } else {
            request.body = encoded;
            request.headers.push((
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            ));
        }
    } else {
        url.set_query(Some(
            std::str::from_utf8(&encoded).expect("form URL encoding is ASCII"),
        ));
    }
    request.url = url.into();
    Ok(Some(NavigationRequest {
        request,
        target: parameters.target.clone().into(),
        replace_history: false,
        host: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use interaction::{
        default::core::html::forms::html_form_element::SubmitMethod,
        dispatch::core::fileapi::form_data::FormData, FormSubmissionParameters,
    };
    fn parameters() -> FormSubmissionParameters {
        FormSubmissionParameters {
            scheduling: None,
            form: 1,
            submitter: Some(2),
            action: "new?old=2#r".into(),
            base_href: Some("https://cdn.test/base/".into()),
            target: "_blank".into(),
            method: SubmitMethod::Get,
            enctype: "application/x-www-form-urlencoded".into(),
        }
    }
    #[test]
    fn form_get_encodes_completed_entries_in_order_and_submitter_parameters() {
        let mut entries = FormData::default();
        for (name, value) in [
            ("wd", "中文 & +"),
            ("hidden", "h"),
            ("checked", "on"),
            ("submit", "yes"),
            ("legend", "ok"),
            ("choice", "b"),
            ("lines", "first\nsecond"),
            ("outside", "associated"),
        ] {
            entries.AppendString(name.into(), value.into());
        }
        let request = BuildRequest(
            "https://page.test/index?original=1",
            &parameters(),
            &entries,
        )
        .unwrap()
        .unwrap();
        assert_eq!(request.target, crate::page::NavigationTarget::NewContext);
        assert_eq!(request.request.method, "GET");
        assert_eq!(request.request.url, "https://cdn.test/base/new?wd=%E4%B8%AD%E6%96%87+%26+%2B&hidden=h&checked=on&submit=yes&legend=ok&choice=b&lines=first%0D%0Asecond&outside=associated#r");
    }
    #[test]
    fn form_post_preserves_request_body_headers_and_action_query() {
        let mut parameters = parameters();
        parameters.method = SubmitMethod::Post;
        parameters.action = "/send?token=x".into();
        parameters.base_href = None;
        let mut entries = FormData::default();
        entries.AppendString("q".into(), "a b".into());
        let request = BuildRequest("https://page.test/", &parameters, &entries)
            .unwrap()
            .unwrap()
            .request;
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://page.test/send?token=x");
        assert_eq!(request.body, b"q=a+b");
        assert_eq!(
            request.headers,
            vec![(
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into()
            )]
        );
    }
}
