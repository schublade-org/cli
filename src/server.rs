use std::collections::BTreeSet;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use axum::extract::State;
use axum::http::{header, StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::{self, Stream};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::timeout;
use tower_http::trace::TraceLayer;

use crate::assets::Assets;
use crate::catalog::{Bootstrap, Catalog};
use crate::cli::ServeArgs;
use crate::config::AppConfig;
use crate::render::render_story;
use crate::stories;
use crate::watch::{self, WatchSpec};

struct AppState {
    workshop: RwLock<Workshop>,
    generation: AtomicU64,
    reload: broadcast::Sender<u64>,
    args: ServeArgs,
}

#[derive(Debug, Clone)]
struct Workshop {
    config: AppConfig,
    catalog: Catalog,
    config_path: Option<PathBuf>,
    catalog_path: Option<PathBuf>,
    stories_root: Option<PathBuf>,
    story_files: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RenderRequest {
    story: String,
    #[serde(default)]
    values: Value,
}

pub async fn run(args: ServeArgs) -> Result<(), String> {
    let workshop = load_workshop_from_args(&args)?;
    let bind = format!("{}:{}", workshop.config.server.host, workshop.config.server.port);
    let listener = TcpListener::bind(&bind)
        .await
        .map_err(|error| format!("could not bind {bind}: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("bound address: {error}"))?;

    print_startup(&workshop, address.port());

    let (reload, _) = broadcast::channel(16);
    let state = Arc::new(AppState {
        workshop: RwLock::new(workshop),
        generation: AtomicU64::new(0),
        reload,
        args,
    });
    start_watcher(state.clone());

    let app = Router::new()
        .route("/", get(index))
        .route("/preview", get(preview_page))
        .route("/api/health", get(health))
        .route("/api/bootstrap", get(bootstrap))
        .route("/api/generation", get(generation))
        .route("/api/events", get(events))
        .route("/api/render", post(render))
        .fallback(static_asset)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|error| format!("server: {error}"))
}

fn load_workshop_from_args(args: &ServeArgs) -> Result<Workshop, String> {
    let (mut config, config_path) = AppConfig::load(args.config.as_deref())?;
    config.apply_cli(args);
    let catalog_source =
        config.resolve_catalog_path(args.catalog.as_deref(), config_path.as_deref())?;
    let stories_root =
        config.resolve_stories_path(args.stories.as_deref(), config_path.as_deref())?;
    let (catalog, catalog_path, story_files) =
        load_workshop(&config, catalog_source.as_deref(), stories_root.as_deref())?;

    Ok(Workshop {
        config,
        catalog,
        config_path,
        catalog_path,
        stories_root,
        story_files,
    })
}

fn print_startup(workshop: &Workshop, port: u16) {
    let display_host = if workshop.config.server.host == "0.0.0.0" {
        "127.0.0.1"
    } else {
        workshop.config.server.host.as_str()
    };

    println!("Schublade {}", env!("CARGO_PKG_VERSION"));
    println!("Workshop  http://{display_host}:{port}");
    print_catalog_line(workshop);
    println!(
        "Theme     {} [{}]",
        workshop.config.theme.trigger.as_key(),
        workshop.config.theme.key
    );
    print!(
        "A11y      {}",
        if workshop.config.a11y.enabled {
            "on"
        } else {
            "off"
        }
    );
    if workshop.config.a11y.enabled {
        let rules: Vec<_> = workshop
            .config
            .a11y
            .rules
            .iter()
            .map(|rule| rule.as_id())
            .collect();
        println!(" ({})", rules.join(", "));
    } else {
        println!();
    }
    if let Some(path) = &workshop.config_path {
        println!("Config    {}", path.display());
    }
    println!("Watch     config, catalog, stories — no restart");
}

fn print_catalog_line(workshop: &Workshop) {
    println!(
        "Catalog   {} ({} {})",
        workshop.catalog.name,
        workshop.catalog.stories.len(),
        if workshop.catalog.stories.len() == 1 {
            "story"
        } else {
            "stories"
        }
    );
    if let Some(path) = &workshop.catalog_path {
        println!("          catalog {}", path.display());
    } else if workshop.stories_root.is_none() {
        println!("          bundled default");
    }
    if let Some(root) = &workshop.stories_root {
        println!(
            "          stories {} ({} file{})",
            root.display(),
            workshop.story_files.len(),
            if workshop.story_files.len() == 1 {
                ""
            } else {
                "s"
            }
        );
    }
}

fn start_watcher(state: Arc<AppState>) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut watcher = match notify::recommended_watcher(move |result| {
        let _ = tx.send(result);
    }) {
        Ok(watcher) => watcher,
        Err(error) => {
            eprintln!("schublade: file watch unavailable ({error}); restart to pick up changes");
            return;
        }
    };

    let initial = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    let mut watching = BTreeSet::new();
    apply_watches(&mut watcher, &mut watching, &initial);

    tokio::spawn(async move {
        loop {
            let Some(first) = rx.recv().await else {
                break;
            };
            let mut interesting = event_is_interesting(&state, first);
            let deadline = tokio::time::Instant::now() + Duration::from_millis(120);
            while tokio::time::Instant::now() < deadline {
                match timeout(deadline.saturating_duration_since(tokio::time::Instant::now()), rx.recv())
                    .await
                {
                    Ok(Some(next)) => {
                        interesting = interesting || event_is_interesting(&state, next);
                    }
                    Ok(None) => return,
                    Err(_) => break,
                }
            }
            if !interesting {
                continue;
            }
            match reload_workshop(&state) {
                Ok(workshop) => {
                    apply_watches(&mut watcher, &mut watching, &workshop);
                    println!(
                        "Reloaded  {} ({} {})",
                        workshop.catalog.name,
                        workshop.catalog.stories.len(),
                        if workshop.catalog.stories.len() == 1 {
                            "story"
                        } else {
                            "stories"
                        }
                    );
                }
                Err(error) => eprintln!("schublade: reload failed: {error}"),
            }
        }
    });
}

fn apply_watches(
    watcher: &mut RecommendedWatcher,
    watching: &mut BTreeSet<(PathBuf, bool)>,
    workshop: &Workshop,
) {
    for spec in watch::watch_specs(
        workshop.config_path.as_deref(),
        workshop.catalog_path.as_deref(),
        workshop.stories_root.as_deref(),
    ) {
        watch_spec(watcher, watching, &spec);
    }
}

fn watch_spec(
    watcher: &mut RecommendedWatcher,
    watching: &mut BTreeSet<(PathBuf, bool)>,
    spec: &WatchSpec,
) {
    if !spec.path.exists() {
        return;
    }
    let key = (spec.path.clone(), spec.recursive);
    if !watching.insert(key) {
        return;
    }
    let mode = if spec.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    if let Err(error) = watcher.watch(&spec.path, mode) {
        watching.remove(&(spec.path.clone(), spec.recursive));
        eprintln!(
            "schublade: could not watch {}: {error}",
            spec.path.display()
        );
    }
}

fn event_is_interesting(state: &AppState, result: Result<notify::Event, notify::Error>) -> bool {
    let event = match result {
        Ok(event) => event,
        Err(error) => {
            eprintln!("schublade: watch: {error}");
            return false;
        }
    };
    if matches!(event.kind, EventKind::Access(_) | EventKind::Other) {
        return false;
    }
    let stories_root = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner())
        .stories_root
        .clone();
    event
        .paths
        .iter()
        .any(|path| watch::should_reload(path, stories_root.as_deref()))
}

fn reload_workshop(state: &AppState) -> Result<Workshop, String> {
    let next = load_workshop_from_args(&state.args)?;
    {
        let mut workshop = state
            .workshop
            .write()
            .unwrap_or_else(|error| error.into_inner());
        *workshop = next.clone();
    }
    let generation = state.generation.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = state.reload.send(generation);
    Ok(next)
}

pub fn load_workshop(
    config: &AppConfig,
    catalog_source: Option<&Path>,
    stories_root: Option<&Path>,
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

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "ok": true,
        "name": "schublade",
        "generation": state.generation.load(Ordering::Relaxed),
    }))
}

async fn generation(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "generation": state.generation.load(Ordering::Relaxed),
    }))
}

async fn events(State(state): State<Arc<AppState>>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.reload.subscribe();
    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(generation) => Some((
                Ok(Event::default()
                    .event("reload")
                    .data(generation.to_string())),
                rx,
            )),
            Err(broadcast::error::RecvError::Lagged(skipped)) => Some((
                Ok(Event::default()
                    .event("reload")
                    .data(format!("lag:{skipped}"))),
                rx,
            )),
            Err(broadcast::error::RecvError::Closed) => None,
        }
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(20)).text("ping"))
}

async fn bootstrap(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    Json(Bootstrap {
        catalog: workshop.catalog.clone(),
        theme: workshop.config.theme.clone(),
        a11y: crate::catalog::A11yBootstrap::from_config(
            workshop.config.a11y.enabled,
            &workshop.config.a11y.rules,
        ),
    })
}

async fn render(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RenderRequest>,
) -> Response {
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    let Some(story) = workshop.catalog.story(&request.story) else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::AtomicU64;
    use tokio::sync::broadcast;

    fn temp_workshop(tag: &str) -> (PathBuf, ServeArgs) {
        let dir = std::env::temp_dir().join(format!("schublade-reload-{tag}-{}", std::process::id()));
        fs::create_dir_all(dir.join("components")).unwrap();
        fs::write(
            dir.join("schublade.toml"),
            "name = \"Reload\"\ncatalog = \"./catalog.toml\"\nstories = \"./components\"\n",
        )
        .unwrap();
        fs::write(dir.join("catalog.toml"), "name = \"Reload\"\n").unwrap();
        fs::write(
            dir.join("components/badge.html"),
            r#"<span class="badge">{{label}}</span>"#,
        )
        .unwrap();
        fs::write(
            dir.join("components/badge.stories.toml"),
            "id = \"badge\"\ntitle = \"Badge\"\ncomponent = \"./badge.html\"\n\n[args]\nlabel = \"Old\"\n",
        )
        .unwrap();
        let args = ServeArgs {
            config: Some(dir.join("schublade.toml")),
            ..ServeArgs::default()
        };
        (dir, args)
    }

    #[test]
    fn reload_picks_up_story_and_catalog_edits() {
        let (dir, args) = temp_workshop("ok");
        let first = load_workshop_from_args(&args).unwrap();
        assert_eq!(first.catalog.stories.len(), 1);
        assert!(first
            .catalog
            .stories
            .iter()
            .any(|story| story.title.contains("Badge")));

        fs::write(
            dir.join("components/button.html"),
            r#"<button type="button">{{label}}</button>"#,
        )
        .unwrap();
        fs::write(
            dir.join("components/button.stories.toml"),
            "id = \"button\"\ntitle = \"Button\"\ncomponent = \"./button.html\"\n\n[args]\nlabel = \"Save\"\n",
        )
        .unwrap();
        fs::write(dir.join("schublade.toml"), "name = \"Reload plus\"\ncatalog = \"./catalog.toml\"\nstories = \"./components\"\n").unwrap();

        let second = load_workshop_from_args(&args).unwrap();
        assert_eq!(second.catalog.name, "Reload plus");
        assert_eq!(
            second.catalog.stories.len(),
            2,
            "{:?}",
            second.catalog.stories.iter().map(|s| &s.id).collect::<Vec<_>>()
        );
        assert!(second.catalog.stories.iter().any(|story| story.id == "button"));
    }

    #[test]
    fn failed_reload_keeps_previous_catalog() {
        let (dir, args) = temp_workshop("keep");
        let state = AppState {
            workshop: RwLock::new(load_workshop_from_args(&args).unwrap()),
            generation: AtomicU64::new(0),
            reload: broadcast::channel(8).0,
            args: args.clone(),
        };
        fs::write(dir.join("catalog.toml"), "name = [not-toml\n").unwrap();
        let before = state.workshop.read().unwrap().catalog.stories.len();
        let error = reload_workshop(&state).unwrap_err();
        assert!(error.contains("catalog"), "{error}");
        assert_eq!(
            state.workshop.read().unwrap().catalog.stories.len(),
            before
        );
        assert_eq!(state.generation.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn successful_reload_bumps_generation() {
        let (dir, args) = temp_workshop("gen");
        let state = AppState {
            workshop: RwLock::new(load_workshop_from_args(&args).unwrap()),
            generation: AtomicU64::new(0),
            reload: broadcast::channel(8).0,
            args,
        };
        fs::write(
            dir.join("schublade.toml"),
            "name = \"After\"\ncatalog = \"./catalog.toml\"\nstories = \"./components\"\n",
        )
        .unwrap();
        reload_workshop(&state).unwrap();
        assert_eq!(state.generation.load(Ordering::Relaxed), 1);
        assert_eq!(state.workshop.read().unwrap().catalog.name, "After");
    }

    #[tokio::test]
    async fn watcher_reloads_when_a_watched_file_changes() {
        let (dir, args) = temp_workshop("live");
        let state = Arc::new(AppState {
            workshop: RwLock::new(load_workshop_from_args(&args).unwrap()),
            generation: AtomicU64::new(0),
            reload: broadcast::channel(8).0,
            args,
        });
        start_watcher(state.clone());
        tokio::time::sleep(Duration::from_millis(150)).await;
        fs::write(
            dir.join("schublade.toml"),
            "name = \"Live\"\ncatalog = \"./catalog.toml\"\nstories = \"./components\"\n",
        )
        .unwrap();

        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        while tokio::time::Instant::now() < deadline {
            if state.generation.load(Ordering::Relaxed) >= 1 {
                assert_eq!(state.workshop.read().unwrap().catalog.name, "Live");
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("watcher did not reload after schublade.toml changed");
    }
}
