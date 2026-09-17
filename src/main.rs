mod agents;
mod assets;
mod build;
mod catalog;
mod cli;
mod config;
mod csf;
mod css_tokens;
mod mdx;
mod props;
mod render;
mod server;
mod shell;
mod stories;
mod tokens;
mod watch;

use clap::Parser;
use cli::{Cli, Command};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Serve(cli::ServeArgs::default()));

    match command {
        Command::Serve(args) => {
            if let Err(error) = server::run(args).await {
                eprintln!("schublade: {error}");
                std::process::exit(1);
            }
        }
        Command::Build(args) => {
            if let Err(error) = build::run(args) {
                eprintln!("schublade: {error}");
                std::process::exit(1);
            }
        }
    }
}
