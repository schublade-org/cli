//! Workshop HTML shell: catalog name, favicon, and bootstrap JSON
//! are written into `index.html` before first paint so the brand does
//! not flash "Schublade" and then swap to the catalog name.

pub fn inject_workshop_shell(
    html: &str,
    bootstrap_json: &str,
    catalog_name: &str,
    favicon_href: &str,
    favicon_mime: &str,
) -> String {
    let html = set_document_title(html, catalog_name);
    let html = set_favicon(&html, favicon_href, favicon_mime);
    inject_bootstrap(&html, bootstrap_json)
}

pub fn set_document_title(html: &str, catalog_name: &str) -> String {
    let title = escape_text(&format!("{catalog_name} · Schublade"));
    replace_tag(html, "<title>", "</title>", &format!("<title>{title}</title>"))
}

pub fn set_favicon(html: &str, href: &str, mime: &str) -> String {
    let tag = format!(
        r#"<link rel="icon" href="{}" type="{}" id="schublade-favicon" />"#,
        escape_attr(href),
        escape_attr(mime)
    );
    if let Some(start) = html.find(r#"id="schublade-favicon""#) {
        if let Some(open) = html[..start].rfind("<link") {
            if let Some(rel_end) = html[start..].find("/>") {
                let end = start + rel_end + 2;
                let mut next = String::with_capacity(html.len() + tag.len());
                next.push_str(&html[..open]);
                next.push_str(&tag);
                next.push_str(&html[end..]);
                return next;
            }
        }
    }
    if let Some(index) = html.find("</head>") {
        let mut next = String::with_capacity(html.len() + tag.len() + 8);
        next.push_str(&html[..index]);
        next.push_str("    ");
        next.push_str(&tag);
        next.push('\n');
        next.push_str(&html[index..]);
        next
    } else {
        format!("{tag}\n{html}")
    }
}

pub fn inject_bootstrap(html: &str, bootstrap_json: &str) -> String {
    let safe = bootstrap_json.replace('<', "\\u003c");
    let block = format!(
        r#"<script type="application/json" id="schublade-bootstrap">{safe}</script>"#
    );

    if let Some(start) = html.find(r#"id="schublade-bootstrap""#) {
        if let Some(open) = html[..start].rfind("<script") {
            if let Some(rel_end) = html[start..].find("</script>") {
                let end = start + rel_end + "</script>".len();
                let mut next = String::with_capacity(html.len() + block.len());
                next.push_str(&html[..open]);
                next.push_str(&block);
                next.push_str(&html[end..]);
                return next;
            }
        }
    }

    for marker in [
        "<script src=\"./workshop.js\"",
        "<script src=\"/workshop.js\"",
    ] {
        if let Some(index) = html.find(marker) {
            let mut next = String::with_capacity(html.len() + block.len() + 8);
            next.push_str(&html[..index]);
            next.push_str(&block);
            next.push('\n');
            next.push_str("    ");
            next.push_str(&html[index..]);
            return next;
        }
    }

    html.replace("</body>", &format!("    {block}\n  </body>"))
}

fn replace_tag(html: &str, open: &str, close: &str, replacement: &str) -> String {
    if let Some(start) = html.find(open) {
        if let Some(rel_end) = html[start..].find(close) {
            let end = start + rel_end + close.len();
            let mut next = String::with_capacity(html.len() + replacement.len());
            next.push_str(&html[..start]);
            next.push_str(replacement);
            next.push_str(&html[end..]);
            return next;
        }
    }
    html.to_string()
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}

pub fn brand_public_filename(path: &std::path::Path, stem: &str) -> String {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{stem}.{ext}"),
        None => stem.to_string(),
    }
}

pub fn mime_from_path(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn injects_name_favicon_and_bootstrap() {
        let html = r#"<!DOCTYPE html><html><head>
    <title>Schublade</title>
    <link rel="icon" href="/favicon.svg" type="image/svg+xml" id="schublade-favicon" />
  </head><body>
    <div id="root"></div>
    <script src="/workshop.js"></script>
  </body></html>"#;
        let out = inject_workshop_shell(
            html,
            r#"{"catalog":{"name":"Story Files"}}"#,
            "Story Files",
            "/brand/favicon",
            "image/svg+xml",
        );
        assert!(out.contains("<title>Story Files · Schublade</title>"), "{out}");
        assert!(out.contains(r#"href="/brand/favicon""#), "{out}");
        assert!(out.contains(r#"id="schublade-bootstrap""#), "{out}");
        assert!(out.contains("Story Files"), "{out}");
        assert!(!out.contains("<title>Schublade</title>"), "{out}");
    }

    #[test]
    fn brand_filenames_keep_extension() {
        assert_eq!(
            brand_public_filename(Path::new("marks/logo.svg"), "logo"),
            "logo.svg"
        );
        assert_eq!(
            brand_public_filename(Path::new("favicon.ico"), "favicon"),
            "favicon.ico"
        );
        assert_eq!(mime_from_path(Path::new("a.svg")), "image/svg+xml");
        assert_eq!(mime_from_path(Path::new("a.ICO")), "image/x-icon");
    }
}
