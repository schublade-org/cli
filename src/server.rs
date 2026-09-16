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
