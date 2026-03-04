use scraper::{Html, Selector};
use std::sync::OnceLock;
use url::Url;

static A_HREF: OnceLock<Selector> = OnceLock::new();

fn selector() -> &'static Selector {
    A_HREF.get_or_init(|| Selector::parse("a[href]").unwrap())
}

pub fn extract_links(html: &str, base: &Url, path_prefix: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    document
        .select(selector())
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| resolve(href, base))
        .filter(|url| is_crawlable(url, base, path_prefix))
        .map(normalize_url)
        .collect()
}

fn resolve(href: &str, base: &Url) -> Option<Url> {
    let trimmed = href.trim();
    if trimmed.starts_with('#')
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
        || trimmed.starts_with("javascript:")
    {
        return None;
    }
    base.join(trimmed).ok()
}

fn is_crawlable(url: &Url, base: &Url, path_prefix: &str) -> bool {
    url.host_str() == base.host_str() && url.path().starts_with(path_prefix)
}

fn normalize_url(mut url: Url) -> String {
    url.set_fragment(None);
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_same_domain_links() {
        let base = Url::parse("http://example.com").unwrap();
        let html = r#"
            <a href="/about">About</a>
            <a href="http://example.com/contact">Contact</a>
            <a href="http://external.com/page">External</a>
        "#;
        let links = extract_links(html, &base, "/");
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.contains("/about")));
        assert!(links.iter().any(|l| l.contains("/contact")));
    }

    #[test]
    fn skips_special_hrefs() {
        let base = Url::parse("http://example.com").unwrap();
        let html = r##"
            <a href="mailto:foo@bar.com">Mail</a>
            <a href="tel:+1234567">Tel</a>
            <a href="javascript:void(0)">JS</a>
            <a href="#section">Fragment</a>
            <a href="/valid">Valid</a>
        "##;
        let links = extract_links(html, &base, "/");
        assert_eq!(links.len(), 1);
        assert!(links[0].contains("/valid"));
    }

    #[test]
    fn strips_fragments_from_urls() {
        let base = Url::parse("http://example.com").unwrap();
        let html = r##"<a href="/page#section">Link</a>"##;
        let links = extract_links(html, &base, "/");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], "http://example.com/page");
    }

    #[test]
    fn respects_path_prefix() {
        let base = Url::parse("http://example.com").unwrap();
        let html = r#"
            <a href="/docs/intro">Docs</a>
            <a href="/blog/post">Blog</a>
        "#;
        let links = extract_links(html, &base, "/docs");
        assert_eq!(links.len(), 1);
        assert!(links[0].contains("/docs/"));
    }
}
