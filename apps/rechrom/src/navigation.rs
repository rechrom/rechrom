use browser::page::Page;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disposition {
    CurrentTab,
    NewTab,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigationRequest {
    pub url: String,
    pub disposition: Disposition,
}

/// Resolve an anchor's default navigation after dispatch and preventDefault.
/// This host policy creates tabs; the DOM and interaction crates never do.
pub fn link_request(page: &Page, target: u64) -> Option<NavigationRequest> {
    let owner = page.Document();
    let document = owner.GetDocument();
    let mut index = document.FindNodeById(target);
    while let Some(i) = index {
        let node = document.Node(i);
        if node.IsHTMLElement("a") {
            let href = &node.FindAttribute("href")?.value;
            if href.starts_with('#') {
                return None;
            }
            let base_node =
                (0..document.NodeCount()).find(|&i| document.Node(i).IsHTMLElement("base"));
            let base_href = base_node.and_then(|i| document.Node(i).FindAttribute("href"));
            let base = base_href
                .and_then(|href| url::Url::parse(page.URL()).ok()?.join(&href.value).ok())
                .or_else(|| url::Url::parse(page.URL()).ok())?;
            let resolved = base.join(href).ok()?;
            if !matches!(resolved.scheme(), "http" | "https" | "file") {
                return None;
            }
            let target = node
                .FindAttribute("target")
                .or_else(|| base_node.and_then(|i| document.Node(i).FindAttribute("target")));
            let disposition =
                if target.is_some_and(|target| target.value.eq_ignore_ascii_case("_blank")) {
                    Disposition::NewTab
                } else {
                    Disposition::CurrentTab
                };
            return Some(NavigationRequest {
                url: resolved.into(),
                disposition,
            });
        }
        index = node.Parent();
    }
    None
}
