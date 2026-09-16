use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::assets::Assets;
use crate::catalog::{Bootstrap, Catalog};
use crate::cli::ServeArgs;
use crate::config::AppConfig;
use crate::render::render_story;
use crate::stories;

struct AppState {
    config: AppConfig,
    catalog: Catalog,
}

#[derive(Debug, Deserialize)]
struct RenderRequest {
    story: String,
    #[serde(default)]
    values: Value,
}

pub async fn run(args: ServeArgs) -> Result<(), String> {
    let (mut config, config_path) = AppConfig::load(args.config.as_deref())?;
    config.apply_cli(&args);
    let catalog_source =
        config.resolve_catalog_path(args.catalog.as_deref(), config_path.as_deref())?;
    let stories_root =
        config.resolve_stories_path(args.stories.as_deref(), config_path.as_deref())?;
    let (catalog, catalog_path, story_files) =
        load_workshop(&config, catalog_source.as_deref(), stories_root.as_deref())?;

    let bind = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&bind)
        .await
        .map_err(|error| format!("could not bind {bind}: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("bound address: {error}"))?;

    let display_host = if config.server.host == "0.0.0.0" {
        "127.0.0.1"
    } else {
        config.server.host.as_str()
    };

    println!("Schublade {}", env!("CARGO_PKG_VERSION"));
    println!("Workshop  http://{display_host}:{}", address.port());
    println!(
        "Catalog   {} ({} {})",
        catalog.name,
        catalog.stories.len(),
        if catalog.stories.len() == 1 {
            "story"
        } else {
            "stories"
        }
    );
    if let Some(path) = &catalog_path {
        println!("          catalog {}", path.display());
    } else if stories_root.is_none() {
        println!("          bundled default");
    }
    if let Some(root) = &stories_root {
        println!(
            "          stories {} ({} file{})",
            root.display(),
            story_files.len(),
            if story_files.len() == 1 { "" } else { "s" }
        );
    }
    println!(
        "Theme     {} [{}]",
        config.theme.trigger.as_key(),
        config.theme.key
    );
    print!(
        "A11y      {}",
        if config.a11y.enabled { "on" } else { "off" }
    );
    if config.a11y.enabled {
        let rules: Vec<_> = config.a11y.rules.iter().map(|rule| rule.as_id()).collect();
        println!(" ({})", rules.join(", "));
    } else {
        println!();
    }
    if let Some(path) = config_path {
        println!("Config    {}", path.display());
    }

    let state = Arc::new(AppState { config, catalog });
    let app = Router::new()
        .route("/", get(index))
        .route("/preview", get(preview_page))
        .route("/api/health", get(health))
        .route("/api/bootstrap", get(bootstrap))
        .route("/api/render", post(render))
        .fallback(static_asset)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| format!("server: {error}"))
}

pub fn load_workshop(
    config: &AppConfig,
    catalog_source: Option<&std::path::Path>,
    stories_root: Option<&std::path::Path>,
) -> Result<(Catalog, Option<PathBuf>, Vec<PathBuf>), String> {
    let (mut catalog, catalog_path) = match catalog_source {
        Some(path) => Catalog::load(Some(path))?,
        None if stories_root.is_some() => (
            Catalog::empty(config.name.clone().unwrap_or_else(|| "Catalog".into())),
            None,
        ),
        None => Catalog::load(None)?,
    };

    if let Some(name) = &config.name {
        catalog.name = name.clone();
    }

    let mut story_files = Vec::new();
    if let Some(root) = stories_root {
        let discovered = stories::discover(root)?;
        story_files = discovered.files;
        catalog.merge_stories(discovered.stories);
    }

    Ok((catalog, catalog_path, story_files))
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index() -> Response {
    embedded("index.html")
}

async fn preview_page() -> Response {
    embedded("preview.html")
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true, "name": "schublade" }))
}

async fn bootstrap(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(Bootstrap {
        catalog: state.catalog.clone(),
        theme: state.config.theme.clone(),
        a11y: crate::catalog::A11yBootstrap::from_config(
            state.config.a11y.enabled,
            &state.config.a11y.rules,
        ),
    })
}

async fn render(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RenderRequest>,
) -> Response {
    let Some(story) = state.catalog.story(&request.story) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "unknown story" })),
        )
            .into_response();
    };

    match render_story(story, &request.values) {
        Ok(rendered) => {
            let react = rendered.react.map(|react| {
                serde_json::json!({
                    "source": react.source,
                    "exportName": react.export_name,
                    "props": react.props,
                })
            });
            Json(serde_json::json!({
                "html": rendered.html,
                "code": rendered.code,
                "react": react,
                "title": story.title,
                "description": story.description,
            }))
            .into_response()
        }
        Err(error) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": error })),
        )
            .into_response(),
    }
}

async fn static_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return embedded("index.html");
    }
    embedded(path)
}

fn embedded(path: &str) -> Response {
    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.essence_str().to_string())],
                file.data,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, Html("Not found")).into_response(),
    }
}
