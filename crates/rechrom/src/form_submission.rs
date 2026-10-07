//! HTML form entry construction and navigation request generation. Interaction
//! dispatches activation/submit; the browser serializes the live DOM afterwards.
use crate::page::NavigationRequest;
use dom::Document;
use std::io;
use url_loader::URLRequest;

fn attr(document: &Document, index: usize, name: &str) -> String {
    document
        .Node(index)
        .FindAttribute(name)
        .map(|a| a.value.clone())
        .unwrap_or_default()
}
fn has(document: &Document, index: usize, name: &str) -> bool {
    document.Node(index).FindAttribute(name).is_some()
}
fn within(document: &Document, index: usize, ancestor: usize) -> bool {
    let mut next = Some(index);
    while let Some(i) = next {
        if i == ancestor {
            return true;
        }
        next = document.Node(i).Parent();
    }
    false
}
fn owner(document: &Document, index: usize) -> Option<usize> {
    if has(document, index, "form") {
        let id = attr(document, index, "form");
        return (0..document.NodeCount())
            .find(|&i| has(document, i, "id") && attr(document, i, "id") == id)
            .filter(|&i| document.Node(i).IsHTMLElement("form"));
    }
    let mut next = document.Node(index).Parent();
    while let Some(i) = next {
        if document.Node(i).IsHTMLElement("form") {
            return Some(i);
        }
        next = document.Node(i).Parent();
    }
    None
}
fn disabled(document: &Document, index: usize) -> bool {
    if has(document, index, "disabled") {
        return true;
    }
    let mut next = document.Node(index).Parent();
    while let Some(i) = next {
        let n = document.Node(i);
        if n.IsHTMLElement("fieldset") && has(document, i, "disabled") {
            let legend = n
                .Children()
                .iter()
                .copied()
                .find(|&child| document.Node(child).IsHTMLElement("legend"));
            if !legend.is_some_and(|legend| within(document, index, legend)) {
                return true;
            }
        }
        next = n.Parent();
    }
    false
}
fn text(document: &Document, index: usize) -> String {
    let n = document.Node(index);
    let mut value = if n.Type() == dom::persistent_document::DOMNodeType::kText {
        n.Data().to_owned()
    } else {
        String::new()
    };
    for &child in n.Children() {
        value.push_str(&text(document, child));
    }
    value
}
fn newline(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', "\r\n")
}
fn entries(document: &Document, form: usize, submitter: Option<usize>) -> Vec<(String, String)> {
    let mut result = Vec::new();
    // DOM arena allocation order can differ after insertBefore/reparenting.
    fn walk(document: &Document, index: usize, ids: &mut Vec<usize>) {
        ids.push(index);
        for &child in document.Node(index).Children() {
            walk(document, child, ids);
        }
    }
    let mut ids = Vec::new();
    walk(document, document.Root(), &mut ids);
    for i in ids {
        let node = document.Node(i);
        if !["input", "button", "textarea", "select"]
            .iter()
            .any(|tag| node.IsHTMLElement(tag))
            || owner(document, i) != Some(form)
            || disabled(document, i)
        {
            continue;
        }
        let mut ancestor = node.Parent();
        let mut in_datalist = false;
        while let Some(parent) = ancestor {
            if document.Node(parent).IsHTMLElement("datalist") {
                in_datalist = true;
                break;
            }
            ancestor = document.Node(parent).Parent();
        }
        if in_datalist {
            continue;
        }
        let name = attr(document, i, "name");
        let ty = attr(document, i, "type").to_ascii_lowercase();
        if node.IsHTMLElement("input") && ty == "image" {
            if submitter == Some(i) {
                let prefix = if name.is_empty() {
                    String::new()
                } else {
                    format!("{name}.")
                };
                result.push((format!("{prefix}x"), "0".into()));
                result.push((format!("{prefix}y"), "0".into()));
            }
            continue;
        }
        if name.is_empty() {
            continue;
        }
        if node.IsHTMLElement("button") {
            if submitter != Some(i) || matches!(ty.as_str(), "button" | "reset") {
                continue;
            }
        } else if node.IsHTMLElement("input") {
            if matches!(ty.as_str(), "button" | "reset") {
                continue;
            }
            if ty == "submit" && submitter != Some(i) {
                continue;
            }
            if matches!(ty.as_str(), "checkbox" | "radio") && !document.ControlChecked(i) {
                continue;
            }
            if ty == "file" {
                result.push((name, String::new()));
                continue;
            }
        }
        if node.IsHTMLElement("select") {
            let options: Vec<usize> = (0..document.NodeCount())
                .filter(|&option| {
                    document.Node(option).IsHTMLElement("option") && within(document, option, i)
                })
                .collect();
            let multiple = has(document, i, "multiple");
            let current = document.ControlValue(i);
            let selected: Vec<usize> = if multiple {
                options
                    .iter()
                    .copied()
                    .filter(|&option| has(document, option, "selected"))
                    .collect()
            } else {
                options
                    .iter()
                    .copied()
                    .find(|&option| {
                        let value = if has(document, option, "value") {
                            attr(document, option, "value")
                        } else {
                            text(document, option)
                        };
                        value == current
                    })
                    .or_else(|| {
                        options
                            .iter()
                            .copied()
                            .find(|&option| has(document, option, "selected"))
                    })
                    .or_else(|| options.first().copied())
                    .into_iter()
                    .collect()
            };
            for option in selected {
                if disabled(document, option)
                    || document.Node(option).Parent().is_some_and(|p| {
                        document.Node(p).IsHTMLElement("optgroup") && has(document, p, "disabled")
                    })
                {
                    continue;
                }
                result.push((
                    name.clone(),
                    if has(document, option, "value") {
                        attr(document, option, "value")
                    } else {
                        text(document, option)
                    },
                ));
            }
            continue;
        }
        let value = if node.IsHTMLElement("button") {
            attr(document, i, "value")
        } else if matches!(ty.as_str(), "checkbox" | "radio") && !has(document, i, "value") {
            "on".into()
        } else if ty == "hidden" && name == "_charset_" {
            "UTF-8".into()
        } else {
            document.ControlValue(i)
        };
        result.push((newline(&name), newline(&value)));
    }
    result
}

pub(crate) fn BuildRequest(
    document: &Document,
    document_url: &str,
    form_id: u64,
    submitter_id: Option<u64>,
) -> io::Result<Option<NavigationRequest>> {
    let Some(form) = document
        .FindNodeById(form_id)
        .filter(|&i| document.Node(i).IsHTMLElement("form"))
    else {
        return Ok(None);
    };
    if !within(document, form, document.Root()) {
        return Ok(None);
    }
    let submitter = submitter_id
        .and_then(|id| document.FindNodeById(id))
        .filter(|&i| owner(document, i) == Some(form));
    let override_attr = |name: &str, button_name: &str| {
        submitter
            .filter(|&i| has(document, i, button_name))
            .map(|i| attr(document, i, button_name))
            .unwrap_or_else(|| attr(document, form, name))
    };
    let method = override_attr("method", "formmethod").to_ascii_lowercase();
    if method == "dialog" {
        return Ok(None);
    }
    let base = url::Url::parse(document_url).map_err(io::Error::other)?;
    let base_element = (0..document.NodeCount()).find(|&i| document.Node(i).IsHTMLElement("base"));
    let action = override_attr("action", "formaction");
    let resolved_base = base_element
        .filter(|&i| has(document, i, "href"))
        .and_then(|i| base.join(&attr(document, i, "href")).ok())
        .unwrap_or_else(|| base.clone());
    let mut url = if action.is_empty() {
        base.clone()
    } else {
        resolved_base.join(&action).map_err(io::Error::other)?
    };
    if !matches!(url.scheme(), "http" | "https" | "file") {
        return Ok(None);
    }
    let mut target = override_attr("target", "formtarget");
    if target.is_empty() {
        target = base_element
            .map(|i| attr(document, i, "target"))
            .unwrap_or_default();
    }
    let entries = entries(document, form, submitter);
    let encoded: String = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(
            entries
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        )
        .finish();
    let mut request = URLRequest {
        referrer: document_url.into(),
        ..Default::default()
    };
    if method == "post" {
        request.method = "POST".into();
        let enctype = override_attr("enctype", "formenctype").to_ascii_lowercase();
        if enctype == "text/plain" {
            request.body = entries
                .iter()
                .map(|(key, value)| format!("{key}={value}\r\n"))
                .collect::<String>()
                .into_bytes();
            request
                .headers
                .push(("Content-Type".into(), "text/plain".into()));
        } else if enctype == "multipart/form-data" {
            // File upload storage is not yet exposed by the DOM, so no synthetic
            // file data is invented. Text entries preserve multipart framing.
            let boundary = format!("----RechromForm{}", interaction::event::NextEventIdentity());
            let mut body = String::new();
            for (key, value) in entries {
                body.push_str(&format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{}\"\r\n\r\n{value}\r\n", key.replace('\r', "%0D").replace('\n', "%0A").replace('"', "%22")));
            }
            body.push_str(&format!("--{boundary}--\r\n"));
            request.body = body.into_bytes();
            request.headers.push((
                "Content-Type".into(),
                format!("multipart/form-data; boundary={boundary}"),
            ));
        } else {
            request.body = encoded.into_bytes();
            request.headers.push((
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            ));
        }
    } else {
        url.set_query(Some(&encoded));
    }
    request.url = url.into();
    Ok(Some(NavigationRequest {
        request,
        target,
        replace_history: false,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(document: &Document, name: &str) -> u64 {
        (0..document.NodeCount())
            .find_map(|i| (attr(document, i, "id") == name).then_some(document.Node(i).Id()))
            .unwrap()
    }
    #[test]
    fn form_get_serializes_live_successful_controls_and_submitter_overrides() {
        let mut owner = html::html_parser::ParseHTML(
            r#"<base href='https://cdn.test/base/' target='_blank'><form id=f action='find?old=1#results'><input id=q name=wd value=old><input name=hidden type=hidden value=h><input type=checkbox name=checked checked><input type=checkbox name=unchecked><input name=disabled disabled value=x><input value=unnamed><input type=submit name=other value=no><button id=go name=submit value=yes formaction='new?old=2#r'>Go</button><fieldset disabled><legend><input name=legend value=ok></legend><input name=blocked value=no></fieldset><select name=choice><option value=a>A</option><option selected value=b>B</option></select><textarea name=lines>first
second</textarea></form><input form=f name=outside value=associated>"#,
        );
        let document = owner.GetDocumentMut();
        let q = document.FindNodeById(id(document, "q")).unwrap();
        document.SetControlValue(q, "中文 & +".into());
        let request = BuildRequest(
            document,
            "https://page.test/index?original=1",
            id(document, "f"),
            Some(id(document, "go")),
        )
        .unwrap()
        .unwrap();
        assert_eq!(request.target, "_blank");
        assert_eq!(request.request.method, "GET");
        assert_eq!(request.request.url,"https://cdn.test/base/new?wd=%E4%B8%AD%E6%96%87+%26+%2B&hidden=h&checked=on&submit=yes&legend=ok&choice=b&lines=first%0D%0Asecond&outside=associated#r");
    }
    #[test]
    fn form_post_preserves_request_body_headers_and_action_query() {
        let owner = html::html_parser::ParseHTML(
            "<form id=f action='/send?token=x' method=post><input name=q value='a b'></form>",
        );
        let document = owner.GetDocument();
        let request = BuildRequest(document, "https://page.test/", id(document, "f"), None)
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
