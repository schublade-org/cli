use clap::{Parser, Subcommand};

/// Schublade — a component workshop served from Rust.
#[derive(Debug, Parser)]
#[command(
    name = "schublade",
    version,
    about = "A Storybook-like component workshop. Run a local server; no JS toolchain required.",
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
}

#[derive(Debug, Clone, Parser)]
pub struct ServeArgs {
    /// Path to schublade.toml
    #[arg(short, long)]
    pub config: Option<std::path::PathBuf>,

    /// Path to catalog.toml (overrides the path in schublade.toml)
    #[arg(long)]
    pub catalog: Option<std::path::PathBuf>,

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
            host: None,
            port: None,
        }
    }
}
