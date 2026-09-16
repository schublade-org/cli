use std::path::Path;

use crate::assets::Assets;
use crate::catalog::{A11yBootstrap, Bootstrap};
use crate::cli::BuildArgs;
use crate::config::AppConfig;
use crate::server;

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

    let bootstrap = Bootstrap {
        catalog: catalog.clone(),
        theme: config.theme.clone(),
        a11y: A11yBootstrap::from_config(config.a11y.enabled, &config.a11y.rules),
    };
    let bootstrap_json = serialize_static_bootstrap(&bootstrap, &catalog)?;

    write_static_site(&args.out, &bootstrap_json)?;

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
    println!("          index.html · preview.html · bootstrap.json");
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

fn write_static_site(out: &Path, bootstrap_json: &str) -> Result<(), String> {
    std::fs::create_dir_all(out)
        .map_err(|error| format!("could not create {}: {error}", out.display()))?;

    for name in Assets::iter() {
        let path = name.as_ref();
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
                html = inject_bootstrap(&html, bootstrap_json);
            }
            html.into_bytes()
        } else {
            file.data.into_owned()
        };

        std::fs::write(&dest, bytes)
            .map_err(|error| format!("could not write {}: {error}", dest.display()))?;
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

fn inject_bootstrap(html: &str, bootstrap_json: &str) -> String {
    let safe = bootstrap_json.replace('<', "\\u003c");
    let block = format!(
        "<script type=\"application/json\" id=\"schublade-bootstrap\">{safe}</script>\n    "
    );
    let mut html = if html.contains("id=\"schublade-bootstrap\"") {
        html.to_string()
    } else if let Some(index) = html.find("<script src=\"./workshop.js\"") {
        let mut next = String::with_capacity(html.len() + block.len());
        next.push_str(&html[..index]);
        next.push_str(&block);
        next.push_str(&html[index..]);
        next
    } else {
        html.replace("</body>", &format!("{block}</body>"))
    };

    if !html.contains("render.js") {
        html = html.replace(
            "<script src=\"./workshop.js\"",
            "<script src=\"./render.js\"></script>\n    <script src=\"./workshop.js\"",
        );
    }
    html
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
            r#"<link href=\"/workshop.css\"><iframe src=\"/preview\"><script src=\"/vendor/react.production.min.js\"></script>"#,
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
            r#"name = \"Static kit\"\n\n[[stories]]\nid = \"button\"\ntitle = \"Button\"\nsection = \"Components\"\ndescription = \"A button\"\ntemplate = \"<button>{{label}}</button>\"\ncode = \"<Button>{{label}}</Button>\"\n\n[[stories.controls]]\nkind = \"text\"\nid = \"label\"\nlabel = \"Label\"\ndefault = \"Save\"\n"#,
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
        assert!(index.contains("Static kit"));
        assert!(index.contains("./workshop.css"));
        assert!(index.contains("./preview.html"));
        assert!(index.contains("./render.js"));
        assert!(index.contains("./workshop.js"));
        assert!(!index.contains("src=\"/preview\""));

        let preview = std::fs::read_to_string(out.join("preview.html")).unwrap();
        assert!(preview.contains("./preview.css"));
        assert!(preview.contains("./jsx.js"));

        let bootstrap: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("bootstrap.json")).unwrap())
                .unwrap();
        assert_eq!(bootstrap["catalog"]["name"], "Static kit");
        assert_eq!(bootstrap["catalog"]["stories"].as_array().unwrap().len(), 1);

        assert!(out.join("render.js").exists());
        assert!(out.join("workshop.js").exists());
        assert!(out.join("vercel.json").exists());
        assert!(out.join("vendor/react.production.min.js").exists());

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
