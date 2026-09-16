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
        .replace("href=\"/\", "href=\"./\")
        .replace("src=\"/\", "src=\"./\")
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
