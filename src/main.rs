mod fetch;
mod submit;

use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use scraper::{Html, Selector};
use serde::Deserialize;
use tinytemplate::TinyTemplate;

/// Generate the Rust file for that day's Advent of Code challenge
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Day (1-25) of the advent calendar
    #[arg(short, long)]
    day: u32,
    /// Challenge year
    #[arg(short, long, default_value_t = 0)]
    year: u32,
    /// Config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, value_enum, default_value_t=Language::Rust)]
    language: Language,
}

#[derive(serde::Serialize, Debug, Clone)]
struct Context {
    n: String,
    day: u32,
    year: u32,
    problem_statement: String,
    language: Language,
}

#[derive(Debug, Clone, clap::ValueEnum, serde::Serialize)]
pub(crate) enum Language {
    JS,
    Python,
    Rust,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let n = format!("{}", args.day);
    // Set year to current year if not provided
    let year = if args.year == 0 {
        let now = jiff::Zoned::now();
        now.year() as u32
    } else {
        args.year
    };

    let context = Context {
        n,
        day: args.day,
        year,
        problem_statement: "".to_owned(),
        language: args.language,
    };

    fetch::fetch_and_write(&context);

    Ok(())
}
