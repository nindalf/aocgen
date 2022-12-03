use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use scraper::{Html, Selector};
use serde::Deserialize;
use tinytemplate::TinyTemplate;

#[derive(Deserialize, Debug)]
struct LangConfig {
    template_name: String,
    exec_file_paths: Vec<String>,
    input_file_paths: Vec<String>,
    test_input_file_paths: Vec<String>,
}

#[derive(Debug)]
struct MaterialisedConfig {
    template: String,
    exec_file_paths: Vec<PathBuf>,
    input_file_paths: Vec<PathBuf>,
    test_input_file_paths: Vec<PathBuf>,
}

/// Generate the Rust file for that day's Advent of Code challenge
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Day (1-25) of the advent calendar
    #[arg(short, long)]
    day: u32,
    /// Challenge year
    #[arg(short, long, default_value_t = 2022)]
    year: u32,
    /// Config file
    #[arg(short, long, value_name = "FILE")]
    config: PathBuf,
}

#[derive(serde::Serialize, Debug)]
struct Context {
    n: String,
}

static RUST_TEMPLATE: &str = include_str!("../templates/rust.tmpl");
static JS_TEMPLATE: &str = include_str!("../templates/js.tmpl");

fn main() -> Result<()> {
    let args = Args::parse();
    let n = format!("{:01$}", args.day, 2);

    let config = get_config(&n, args.config)?;

    write_to_files(&config.template, &config.exec_file_paths)?;

    let test_input = fetch_test_input(args.year, args.day)?;
    write_to_files(&test_input, &config.test_input_file_paths)?;

    let real_input = fetch_real_input(args.year, args.day)?;
    write_to_files(&real_input, &config.input_file_paths)?;

    Ok(())
}

fn write_to_files(content: &str, paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        if path.exists() {
            // file already exists, do nothing
            continue;
        }
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        std::fs::write(path, content)?;
    }
    Ok(())
}

fn get_config(n: &str, path: PathBuf) -> Result<MaterialisedConfig> {
    let config_file = File::open(path)?;
    let reader = BufReader::new(config_file);
    let config: LangConfig = serde_json::from_reader(reader)?;

    let context = Context { n: n.to_string() };
    let mut tt = TinyTemplate::new();

    let exec_file_template = match config.template_name.as_str() {
        "rust" => RUST_TEMPLATE,
        "js" => JS_TEMPLATE,
        _ => panic!("Unsupported template"),
    };
    tt.add_template("exec_file", exec_file_template).unwrap();
    let template = tt.render("exec_file", &context).unwrap();

    let exec_file_paths: Vec<_> = config
        .exec_file_paths
        .iter()
        .map(|s| {
            tt.add_template("temp", s).unwrap();
            PathBuf::from(tt.render("temp", &context).unwrap())
        })
        .collect();
    let test_input_file_paths: Vec<_> = config
        .test_input_file_paths
        .iter()
        .map(|s| {
            tt.add_template("temp", s).unwrap();
            PathBuf::from(tt.render("temp", &context).unwrap())
        })
        .collect();
    let input_file_paths: Vec<_> = config
        .input_file_paths
        .iter()
        .map(|s| {
            tt.add_template("temp", s).unwrap();
            PathBuf::from(tt.render("temp", &context).unwrap())
        })
        .collect();
    Ok(MaterialisedConfig {
        template,
        exec_file_paths,
        input_file_paths,
        test_input_file_paths,
    })
}

fn fetch_test_input(year: u32, day: u32) -> Result<String> {
    let url = format!("https://adventofcode.com/{year}/day/{day}");
    let client = reqwest::blocking::Client::new();
    let cookie = std::env::var("AOC_COOKIE")?;
    let request = client
        .get(url)
        .header("cookie", format!("session={cookie}"))
        .build()?;
    let html = client.execute(request)?.text()?;
    largest_code_block(&html)
}

fn largest_code_block(html: &str) -> Result<String> {
    let fragment = Html::parse_fragment(html);
    let code_selector = Selector::parse("code").unwrap();
    let mut code_fragments: Vec<String> = fragment
        .select(&code_selector)
        .filter_map(|element| element.first_child())
        .filter_map(|child| child.value().as_text())
        .map(|text| text.to_string())
        .collect();
    code_fragments.sort_by_key(|a| a.len());
    code_fragments
        .pop()
        .ok_or_else(|| anyhow::anyhow!("No code blocks found"))
}

fn fetch_real_input(year: u32, day: u32) -> Result<String> {
    let url = format!("https://adventofcode.com/{year}/day/{day}/input");
    let client = reqwest::blocking::Client::new();
    let cookie = std::env::var("AOC_COOKIE")?;
    let request = client
        .get(url)
        .header("cookie", format!("session={cookie}"))
        .build()?;
    Ok(client.execute(request)?.text()?)
}
