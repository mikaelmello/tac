use clap::{AppSettings, Clap};

/// lox interpreter written in Rust
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// File path for script to be run
    pub script: Option<String>,
}
