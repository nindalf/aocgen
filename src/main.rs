mod client;
mod fetch;
mod submit;
mod time;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Generate the Rust file for that day's Advent of Code challenge

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch puzzle input and description
    Fetch(FetchArgs),
    /// Submit puzzle solution
    Submit(SubmitArgs),
}

#[derive(Parser, Debug)]
pub(crate) struct FetchArgs {
    /// Day (1-25) of the advent calendar
    #[arg(short, long, value_parser=clap::value_parser!(u32).range(1..=25))]
    day: u32,

    /// Challenge year
    #[arg(short, long, default_value_t = 0)]
    year: u32,

    #[arg(short, long, value_enum, default_value_t = Language::Rust)]
    language: Language,

    /// Config file
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub(crate) struct SubmitArgs {
    /// Day (1-25) of the advent calendar
    #[arg(short, long, value_parser=clap::value_parser!(u32).range(1..=25))]
    day: u32,

    /// Challenge year
    #[arg(short, long, default_value_t = 0)]
    year: u32,

    /// Solution part number (1 or 2)
    #[arg(short, long)]
    part: String,

    /// Solution answer
    #[arg(short, long)]
    answer: String,
}

#[derive(Debug, Clone, clap::ValueEnum, serde::Serialize)]
pub(crate) enum Language {
    JS,
    Python,
    Rust,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Fetch(fetch_args) => fetch::fetch_and_write(fetch_args),
        Commands::Submit(submit_args) => submit::submit_answer(submit_args),
    }?;

    Ok(())
}
