use std::path::Path;

use crate::assets::Assets;
use crate::catalog::{A11yBootstrap, Bootstrap, BrandBootstrap};
use crate::cli::BuildArgs;
use crate::config::AppConfig;
use crate::server;
use crate::shell;

const VERCEL_JSON: &str = r#"{
  "cleanUrls": false,
  "trailingSlash": false
}
"#;

pub fn run(args: BuildArgs) -> Result<(), String> {
    let (mut config, config_path) = AppConfig::load(args.config.as_deref())?;
    config.apply_workshop_cli(args.name.as_deref(), args.stories.as_deref());
    let catalog_source =
        config.resolve_catalog_path(args.catalog.as_deref(), config_path.as_deref())?;
    let stories_root =
        config.resolve_stories_path(args.stories.as_deref(), config_path.as_deref())?;
    let (catalog, catalog_path, story_files) =
        server::load_workshop(&config, catalog_source.as_deref(), stories_root.as_deref())?;

    let logo_path = config.resolve_logo_path(config_path.as_deref())?;
    let favicon_path = config.resolve_favicon_path(config_path.as_deref())?;
    let logo_file = logo_path
        .as_deref()
        .map(|path| shell::brand_public_filename(path, "logo"));
    let favicon_file = favicon_path
        .as_deref()
        .map(|path| shell::brand_public_filename(path, "favicon"))
        .unwrap_or_else(|| "favicon.svg".to_string());

    let bootstrap = Bootstrap {
        catalog: catalog.clone(),
        theme: config.theme.clone(),
        a11y: A11yBootstrap::from_config(config.a11y.enabled, &config.a11y.rules),
        brand: BrandBootstrap::static_site(&catalog.name, logo_file.clone(), favicon_file.clone()),
        static_site: true,
    };
    let bootstrap_json = serialize_static_bootstrap(&bootstrap, &catalog)?;

    write_static_site(
        &args.out,
        &bootstrap_json,
        &catalog.name,
        logo_path.as_deref(),
        logo_file.as_deref(),
        favicon_path.as_deref(),
        &favicon_file,
    )?;

    println!("Schublade {}", env!("CARGO_PKG_VERSION"));
    println!(
        "Wrote     {} ({} {})",
        args.out.display(),
        catalog.stories.len(),
        if catalog.stories.len() == 1 {
            "story"
        } else {
            "stories"
        }
    );
    if let Some(path) = &catalog_path {
        println!("          catalog {}", path.display());
    }
    if let Some(root) = &stories_root {
        println!(
            "          stories {} ({} file{})",
            root.display(),
            story_files.len(),
            if story_files.len() == 1 { "" } else { "s" }
        );
    }
    if let Some(path) = config_path {
        println!("          config {}", path.display());
    }
    println!("          index.html · preview.html · bootstrap.json · favicon");
    println!(
        "Deploy    any static host — point it at {}",
        args.out.display()
    );

    Ok(())
}

fn serialize_static_bootstrap(
    bootstrap: &Bootstrap,
    catalog: &crate::catalog::Catalog,
) -> Result<String, String> {
    let mut value = serde_json::to_value(bootstrap)
        .map_err(|error| format!("serialize bootstrap: {error}"))?;
    if let Some(stories) = value
        .get_mut("catalog")
        .and_then(|catalog| catalog.get_mut("stories"))
        .and_then(|stories| stories.as_array_mut())
    {
        for (story, encoded) in catalog.stories.iter().zip(stories.iter_mut()) {
            if let Some(source) = &story.component_source {
                encoded["component_source"] = serde_json::Value::String(source.clone());
            }
        }
    }
    serde_json::to_string_pretty(&value).map_err(|error| format!("serialize bootstrap: {error}"))
}

fn write_static_site(
    out: &Path,
    bootstrap_json: &str,
    catalog_name: &str,
    logo_path: Option<&Path>,
    logo_file: Option<&str>,
    favicon_path: Option<&Path>,
    favicon_file: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(out)
        .map_err(|error| format!("could not create {}: {error}", out.display()))?;

    let favicon_href = format!("./{favicon_file}");
    let favicon_mime = favicon_path
        .map(shell::mime_from_path)
        .unwrap_or("image/svg+xml");

    for name in Assets::iter() {
        let path = name.as_ref();
        if path.starts_with("src/") || path.ends_with(".jsx") {
            continue;
        }
        let file = Assets::get(path).ok_or_else(|| format!("missing embedded asset {path}"))?;
        let dest = out.join(path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                format!("could not create {}: {error}", parent.display())
            })?;
        }

        let bytes = if path.ends_with(".html") {
            let mut html = String::from_utf8(file.data.into_owned())
                .map_err(|error| format!("{path} is not utf-8: {error}"))?;
            html = rewrite_asset_urls(&html);
            if path == "index.html" {
                html = shell::inject_workshop_shell(
                    &html,
                    bootstrap_json,
                    catalog_name,
                    &favicon_href,
                    favicon_mime,
                );
                if !html.contains("render.js") {
                    html = html.replace(
                        "<script type=\"module\" src=\"./chrome/main.js\"",
                        "<script src=\"./render.js\"></script>\n    <script type=\"module\" src=\"./chrome/main.js\"",
                    );
                }
            }
            html.into_bytes()
        } else {
            file.data.into_owned()
        };

        std::fs::write(&dest, bytes)
            .map_err(|error| format!("could not write {}: {error}", dest.display()))?;
    }

    if let (Some(source), Some(name)) = (logo_path, logo_file) {
        copy_brand_file(source, &out.join(name))?;
    }
    if let Some(source) = favicon_path {
        copy_brand_file(source, &out.join(favicon_file))?;
    }

    std::fs::write(out.join("bootstrap.json"), bootstrap_json).map_err(|error| {
        format!(
            "could not write {}: {error}",
            out.join("bootstrap.json").display()
        )
    })?;
    std::fs::write(out.join("vercel.json"), VERCEL_JSON).map_err(|error| {
        format!(
            "could not write {}: {error}",
            out.join("vercel.json").display()
        )
    })?;

    Ok(())
}

pub(crate) fn rewrite_asset_urls(html: &str) -> String {
    html.replace("src=\"/preview\"", "src=\"./preview.html\"")
        .replace("href=\"/", "href=\"./")
        .replace("src=\"/", "src=\"./")
}

fn copy_brand_file(source: &Path, dest: &Path) -> Result<(), String> {
    std::fs::copy(source, dest).map_err(|error| {
        format!(
            "could not copy {} to {}: {error}",
            source.display(),
            dest.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::cli::BuildArgs;

    fn unique_root(stamp: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "schublade-build-{}-{}-{}",
            std::process::id(),
            stamp,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ))
    }

    #[test]
    fn rewrite_makes_assets_relative() {
        let html = rewrite_asset_urls(
            r#"<link href="/workshop.css"><iframe src="/preview"><script src="/vendor/react.production.min.js"></script>"#,
        );
        assert!(html.contains("./workshop.css"));
        assert!(html.contains("./preview.html"));
        assert!(html.contains("./vendor/react.production.min.js"));
        assert!(!html.contains("src=\"/preview\""));
    }

    #[test]
    fn writes_portable_static_site() {
        let root = unique_root("catalog");
        std::fs::create_dir_all(&root).unwrap();
        let catalog = root.join("catalog.toml");
        std::fs::write(
            &catalog,
            r#"name = "Static kit"

[[stories]]
id = "button"
title = "Button"
section = "Components"
description = "A button"
template = "<button>{{label}}</button>"
code = "<Button>{{label}}</Button>"

[[stories.controls]]
kind = "text"
id = "label"
label = "Label"
default = "Save"
"#,
        )
        .unwrap();

        let out = root.join("dist");
        run(BuildArgs {
            config: None,
            catalog: Some(catalog),
            stories: None,
            name: Some("Static kit".into()),
            out: out.clone(),
        })
        .unwrap();

        let index = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(index.contains("id=\"schublade-bootstrap\""));
        assert!(index.contains("<title>Static kit · Schublade</title>"), "{index}");
        assert!(index.contains("Static kit"));
        assert!(index.contains("./workshop.css"));
        assert!(index.contains("./render.js"));
        assert!(index.contains("./chrome/main.js"));
        assert!(index.contains("type=\"importmap\""));
        assert!(index.contains("@base-ui/react@1.8.0"));
        assert!(index.contains("./favicon.svg"));
        assert!(!index.contains("src=\"/preview\""));
        assert!(!index.contains("id=\"catalog-name\">Schublade<"));
        assert!(out.join("favicon.svg").exists());
        assert!(out.join("preview.html").exists());
        assert!(out.join("chrome/main.js").exists());
        assert!(out.join("chrome/App.js").exists());
        let chrome = std::fs::read_to_string(out.join("chrome/App.js")).unwrap();
        assert!(
            chrome.contains("./preview.html"),
            "static chrome must point the iframe at ./preview.html"
        );

        let preview = std::fs::read_to_string(out.join("preview.html")).unwrap();
        assert!(preview.contains("./preview.css"));
        assert!(preview.contains("./jsx.js"));

        let bootstrap: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("bootstrap.json")).unwrap())
                .unwrap();
        assert_eq!(bootstrap["catalog"]["name"], "Static kit");
        assert_eq!(bootstrap["brand"]["name"], "Static kit");
        assert_eq!(bootstrap["brand"]["favicon"], "./favicon.svg");
        assert_eq!(bootstrap["static"], true);
        assert_eq!(bootstrap["catalog"]["stories"].as_array().unwrap().len(), 1);

        assert!(out.join("render.js").exists());
        assert!(out.join("chrome/main.js").exists());
        assert!(out.join("vercel.json").exists());
        assert!(out.join("vendor/react.production.min.js").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn copies_configured_logo_and_favicon() {
        let root = unique_root("brand");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("catalog.toml"),
            "name = \"Branded kit\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("logo.svg"),
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect width="16" height="16" fill="#111"/></svg>"##,
        )
        .unwrap();
        std::fs::write(
            root.join("mark.ico"),
            b"fake-ico",
        )
        .unwrap();
        std::fs::write(
            root.join("schublade.toml"),
            r#"
catalog = "./catalog.toml"
logo = "./logo.svg"
favicon = "./mark.ico"
"#,
        )
        .unwrap();

        let out = root.join("dist");
        run(BuildArgs {
            config: Some(root.join("schublade.toml")),
            catalog: None,
            stories: None,
            name: None,
            out: out.clone(),
        })
        .unwrap();

        let index = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(index.contains("<title>Branded kit · Schublade</title>"), "{index}");
        assert!(index.contains("./favicon.ico"), "{index}");
        assert!(index.contains("image/x-icon"), "{index}");
        assert_eq!(std::fs::read(out.join("logo.svg")).unwrap(), std::fs::read(root.join("logo.svg")).unwrap());
        assert_eq!(std::fs::read(out.join("favicon.ico")).unwrap(), b"fake-ico");

        let bootstrap: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("bootstrap.json")).unwrap())
                .unwrap();
        assert_eq!(bootstrap["brand"]["logo"], "./logo.svg");
        assert_eq!(bootstrap["brand"]["favicon"], "./favicon.ico");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn story_files_example_keeps_react_source() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out = unique_root("story-files").join("dist");
        run(BuildArgs {
            config: Some(manifest.join("examples/story-files/schublade.toml")),
            catalog: None,
            stories: None,
            name: None,
            out: out.clone(),
        })
        .unwrap();

        let bootstrap: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("bootstrap.json")).unwrap())
                .unwrap();
        let stories = bootstrap["catalog"]["stories"].as_array().unwrap();
        assert!(stories.len() >= 4, "{stories:?}");
        let button = stories
            .iter()
            .find(|story| story["id"] == "button")
            .expect("button story");
        assert_eq!(button["generator"], "react");
        assert!(
            button["component_source"]
                .as_str()
                .unwrap()
                .contains("function Button"),
            "{}",
            button["component_source"]
        );

        let _ = std::fs::remove_dir_all(out.parent().unwrap());
    }
}
