use clap::{Parser, Subcommand};

/// Schublade — a component workshop served from Rust.
#[derive(Debug, Parser)]
#[command(
    name = "schublade",
    version,
    about = "A Storybook-like component workshop. Serve it locally or build static HTML.",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the workshop server
    Serve(ServeArgs),
    /// Write a static HTML workshop that can be deployed anywhere
    Build(BuildArgs),
}

#[derive(Debug, Clone, Parser)]
pub struct ServeArgs {
    /// Path to schublade.toml
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,

    /// Path to catalog.toml (overrides the path in schublade.toml)
    #[arg(long)]
    pub catalog: Option<std::path::PathBuf>,

    /// Directory of `*.stories.js(x)` / `*.stories.toml` files (overrides `stories` in schublade.toml)
    #[arg(long)]
    pub stories: Option<std::path::PathBuf>,

    /// Catalog name when no catalog.toml is present
    #[arg(long)]
    pub name: Option<String>,

    /// Bind address
    #[arg(long)]
    pub host: Option<String>,

    /// Bind port
    #[arg(short, long)]
    pub port: Option<u16>,
}

impl Default for ServeArgs {
    fn default() -> Self {
        Self {
            config: None,
            catalog: None,
            stories: None,
            name: None,
            host: None,
            port: None,
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub struct BuildArgs {
    /// Path to schublade.toml
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,

    /// Path to catalog.toml (overrides the path in schublade.toml)
    #[arg(long)]
    pub catalog: Option<std::path::PathBuf>,

    /// Directory of story files (overrides `stories` in schublade.toml)
    #[arg(long)]
    pub stories: Option<std::path::PathBuf>,

    /// Catalog name when no catalog.toml is present
    #[arg(long)]
    pub name: Option<String>,

    /// Output directory
    #[arg(short, long, default_value = "dist")]
    pub out: std::path::PathBuf,
}

impl Default for BuildArgs {
    fn default() -> Self {
        Self {
            config: None,
            catalog: None,
            stories: None,
            name: None,
            out: std::path::PathBuf::from("dist"),
        }
    }
}
