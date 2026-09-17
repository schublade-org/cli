use std::collections::BTreeSet;
use std::convert::Infallible;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use axum::extract::{Path as PathParam, State};
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

use crate::agents;
use crate::assets::Assets;
use crate::catalog::{A11yBootstrap, Bootstrap, BrandBootstrap, Catalog};
use crate::cli::ServeArgs;
use crate::config::AppConfig;
use crate::render::render_story;
use crate::shell;
use crate::css_tokens;
use crate::mdx;
use crate::stories;
use crate::tokens::{self, TokenSet};
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
    docs_root: Option<PathBuf>,
    docs_files: Vec<PathBuf>,
    tokens: TokenSet,
    tokens_css: Option<PathBuf>,
    logo_path: Option<PathBuf>,
    favicon_path: Option<PathBuf>,
}

pub struct LoadedWorkshop {
    pub catalog: Catalog,
    pub catalog_path: Option<PathBuf>,
    pub story_files: Vec<PathBuf>,
    pub docs_root: Option<PathBuf>,
    pub docs_files: Vec<PathBuf>,
    pub tokens: TokenSet,
    pub tokens_css: Option<PathBuf>,
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
        .route("/brand/logo", get(brand_logo))
        .route("/brand/favicon", get(brand_favicon))
        .route("/favicon.ico", get(brand_favicon))
        .route("/favicon.svg", get(brand_favicon))
        .route("/AGENTS.md", get(agents_index))
        .route("/{id}/AGENTS.md", get(agents_story))
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
    let loaded = load_workshop(
        &config,
        catalog_source.as_deref(),
        stories_root.as_deref(),
        config_path.as_deref(),
    )?;
    let logo_path = config.resolve_logo_path(config_path.as_deref())?;
    let favicon_path = config.resolve_favicon_path(config_path.as_deref())?;

    Ok(Workshop {
        config,
        catalog: loaded.catalog,
        config_path,
        catalog_path: loaded.catalog_path,
        stories_root,
        story_files: loaded.story_files,
        docs_root: loaded.docs_root,
        docs_files: loaded.docs_files,
        tokens: loaded.tokens,
        tokens_css: loaded.tokens_css,
        logo_path,
        favicon_path,
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
    if !workshop.catalog.pages.is_empty() {
        println!(
            "          {} docs page{}",
            workshop.catalog.pages.len(),
            if workshop.catalog.pages.len() == 1 {
                ""
            } else {
                "s"
            }
        );
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
    if let Some(root) = &workshop.docs_root {
        println!(
            "          docs {} ({} file{})",
            root.display(),
            workshop.docs_files.len(),
            if workshop.docs_files.len() == 1 {
                ""
            } else {
                "s"
            }
        );
    }
    if let Some(path) = &workshop.tokens_css {
        println!("          tokens {}", path.display());
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
    let extra: Vec<PathBuf> = workshop.tokens_css.iter().cloned().collect();
    for spec in watch::watch_specs(
        workshop.config_path.as_deref(),
        workshop.catalog_path.as_deref(),
        workshop.stories_root.as_deref(),
        workshop.docs_root.as_deref(),
        &extra,
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
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    let stories_root = workshop.stories_root.clone();
    let docs_root = workshop.docs_root.clone();
    event
        .paths
        .iter()
        .any(|path| watch::should_reload(path, stories_root.as_deref(), docs_root.as_deref()))
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
    config_path: Option<&Path>,
) -> Result<LoadedWorkshop, String> {
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

    let tokens_css = config.resolve_tokens_css_path(config_path)?;

    let mut tokens = TokenSet::default();
    if let Some(path) = &tokens_css {
        tokens.merge(css_tokens::load_css_file(path)?);
    }
    tokens.merge(config.tokens.from_manual());

    let docs_root = config.resolve_docs_path(config_path)?;
    let (pages, docs_files) = if let Some(root) = &docs_root {
        let discovered = mdx::discover(root, &tokens)?;
        (discovered.pages, discovered.files)
    } else {
        (Vec::new(), Vec::new())
    };
    let pages = if pages.is_empty() {
        tokens::preset_pages(&tokens)
    } else {
        pages
    };
    catalog.attach_pages(pages)?;

    Ok(LoadedWorkshop {
        catalog,
        catalog_path,
        story_files,
        docs_root,
        docs_files,
        tokens,
        tokens_css,
    })
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

async fn index(State(state): State<Arc<AppState>>) -> Response {
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    let Some(file) = Assets::get("index.html") else {
        return (StatusCode::NOT_FOUND, Html("Not found")).into_response();
    };
    let template = match String::from_utf8(file.data.into_owned()) {
        Ok(html) => html,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "index.html is not utf-8").into_response(),
    };
    let bootstrap = live_bootstrap(&workshop);
    let json = match serde_json::to_string(&bootstrap) {
        Ok(json) => json,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("serialize bootstrap: {error}"),
            )
                .into_response();
        }
    };
    let html = shell::inject_workshop_shell(
        &template,
        &json,
        &workshop.catalog.name,
        "/brand/favicon",
        workshop
            .favicon_path
            .as_deref()
            .map(shell::mime_from_path)
            .unwrap_or("image/svg+xml"),
    );
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
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
    Json(live_bootstrap(&workshop))
}

async fn agents_index(State(state): State<Arc<AppState>>) -> Response {
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    markdown_response(agents::index(&workshop.catalog))
}

async fn agents_story(State(state): State<Arc<AppState>>, PathParam(id): PathParam<String>) -> Response {
    if !agents::is_safe_story_id(&id) {
        return (StatusCode::NOT_FOUND, Html("Not found")).into_response();
    }
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    match workshop.catalog.story(&id) {
        Some(story) => markdown_response(agents::story_doc(story)),
        None => (StatusCode::NOT_FOUND, Html("Not found")).into_response(),
    }
}

fn markdown_response(body: String) -> Response {
    (
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        body,
    )
        .into_response()
}

fn live_bootstrap(workshop: &Workshop) -> Bootstrap {
    Bootstrap {
        catalog: workshop.catalog.clone(),
        tokens: workshop.tokens.clone(),
        theme: workshop.config.theme.clone(),
        a11y: A11yBootstrap::from_config(workshop.config.a11y.enabled, &workshop.config.a11y.rules),
        brand: BrandBootstrap::live(&workshop.catalog.name, workshop.logo_path.is_some()),
        static_site: false,
    }
}

async fn brand_logo(State(state): State<Arc<AppState>>) -> Response {
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    match &workshop.logo_path {
        Some(path) => disk_file(path),
        None => (StatusCode::NOT_FOUND, Html("Not found")).into_response(),
    }
}

async fn brand_favicon(State(state): State<Arc<AppState>>) -> Response {
    let workshop = state
        .workshop
        .read()
        .unwrap_or_else(|error| error.into_inner());
    match &workshop.favicon_path {
        Some(path) => disk_file(path),
        None => embedded("favicon.svg"),
    }
}

fn disk_file(path: &Path) -> Response {
    match std::fs::read(path) {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, shell::mime_from_path(path).to_string())],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, Html("Not found")).into_response(),
    }
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

    #[test]
    fn bundled_docs_and_css_tokens_load() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (config, path) = AppConfig::load(Some(&manifest.join("schublade.toml"))).unwrap();
        let loaded = load_workshop(
            &config,
            Some(&manifest.join("catalog.toml")),
            None,
            path.as_deref(),
        )
        .unwrap();
        assert_eq!(loaded.catalog.name, "Demo catalog");
        assert!(loaded.catalog.page("colors").is_some());
        assert!(loaded.catalog.page("typography").is_some());
        assert!(
            loaded.tokens.colors.iter().any(|scale| scale.id == "yellow"),
            "{:?}",
            loaded.tokens.colors.iter().map(|scale| &scale.id).collect::<Vec<_>>()
        );
        assert!(loaded.tokens.colors.iter().any(|scale| scale.id == "ink"));
        assert!(loaded
            .tokens
            .typography
            .styles
            .iter()
            .any(|style| style.id == "display-xl"));
        assert!(loaded.tokens_css.is_some());
        assert_eq!(loaded.docs_files.len(), 2);
        assert!(
            loaded
                .tokens
                .typography
                .roles
                .iter()
                .any(|row| row.token == "--text-display-xl"),
            "{:?}",
            loaded
                .tokens
                .typography
                .roles
                .iter()
                .map(|row| &row.token)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn tailwind_tokens_example_loads_theme_families() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let (config, path) =
            AppConfig::load(Some(&manifest.join("examples/tailwind-tokens/schublade.toml"))).unwrap();
        let loaded = load_workshop(
            &config,
            Some(&manifest.join("examples/tailwind-tokens/catalog.toml")),
            None,
            path.as_deref(),
        )
        .unwrap();
        assert_eq!(loaded.catalog.name, "Tailwind tokens");
        assert!(loaded.catalog.page("colors").is_some());
        assert!(loaded.catalog.page("typography").is_some());
        assert!(loaded.catalog.page("tokens").is_some());
        assert!(loaded.tokens.colors.iter().any(|scale| scale.id == "yellow"));
        assert!(loaded.tokens.colors.iter().any(|scale| scale.id == "text"));
        assert!(loaded.tokens.groups.iter().any(|group| group.id == "spacing"));
        assert!(loaded.tokens.groups.iter().any(|group| group.id == "radius"));
        assert!(loaded.tokens.groups.iter().any(|group| group.id == "shadow"));
        assert!(loaded.tokens.groups.iter().any(|group| group.id == "tab-size"));
        assert!(loaded.tokens.groups.iter().any(|group| group.id == "zoom"));
        assert!(!loaded
            .tokens
            .typography
            .families
            .iter()
            .any(|row| row.token.contains("font-size")));
        assert!(!loaded
            .tokens
            .groups
            .iter()
            .any(|group| group.rows.iter().any(|row| row.token == "--ignored")));
    }

    #[tokio::test]
    async fn agents_md_handlers_use_catalog_renderer() {
        let (_dir, args) = temp_workshop("agents");
        let state = Arc::new(AppState {
            workshop: RwLock::new(load_workshop_from_args(&args).unwrap()),
            generation: AtomicU64::new(0),
            reload: broadcast::channel(8).0,
            args,
        });

        let index = agents_index(State(state.clone())).await;
        assert_eq!(index.status(), StatusCode::OK);
        assert_eq!(
            index
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("text/markdown; charset=utf-8")
        );
        let index_body = axum::body::to_bytes(index.into_body(), usize::MAX)
            .await
            .unwrap();
        let index_text = String::from_utf8(index_body.to_vec()).unwrap();
        assert!(index_text.contains("# Reload"));
        assert!(index_text.contains("/badge/AGENTS.md"));
        assert!(!index_text.contains("#/badge"));

        let story = agents_story(State(state.clone()), PathParam("badge".into())).await;
        assert_eq!(story.status(), StatusCode::OK);
        let story_body = axum::body::to_bytes(story.into_body(), usize::MAX)
            .await
            .unwrap();
        let story_text = String::from_utf8(story_body.to_vec()).unwrap();
        assert!(story_text.contains("# Badge"));
        assert!(story_text.contains("<span class=\"badge\">Old</span>") == false);
        assert!(story_text.contains("Old") || story_text.contains("Badge"));

        let missing = agents_story(State(state), PathParam("missing".into())).await;
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}
