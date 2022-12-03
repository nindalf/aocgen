use anyhow::Result;
use clap::Parser;
use scraper::{Html, Selector};
use tinytemplate::TinyTemplate;

static TEMPLATE: &str = include_str!("dayn.rs.tmpl");

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
}

#[derive(serde::Serialize)]
struct Context {
    n: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let n = format!("{:01$}", args.day, 2);

    let src = std::path::Path::new("src");
    let rs_path = src.join(format!("day{n}.rs"));
    if !rs_path.exists() {
        let rendered = rendered_template(&n)?;
        std::fs::write(rs_path, rendered)?;
    }

    let inputs = std::path::Path::new("inputs");
    if !inputs.exists() {
        std::fs::create_dir(inputs)?;
    }

    let test_input_file = inputs.join(format!("day{n}-test.txt"));
    if !test_input_file.exists() {
        let test_input = fetch_test_input(args.year, args.day)?;
        std::fs::write(test_input_file, test_input)?;
    }

    let real_input_file = inputs.join(format!("day{n}.txt"));
    if !real_input_file.exists() {
        let real_input = fetch_real_input(args.year, args.day)?;
        std::fs::write(real_input_file, real_input)?;
    }

    Ok(())
}

fn rendered_template(n: &str) -> Result<String> {
    let mut tt = TinyTemplate::new();
    tt.add_template("rs_file", TEMPLATE)?;
    let context = Context { n: n.to_string() };
    Ok(tt.render("rs_file", &context)?)
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
    let fragment = Html::parse_fragment(&html);
    let code_selector = Selector::parse("code").unwrap();
    let mut code_fragments: Vec<String> = fragment.select(&code_selector)
        .filter_map(|element| element.first_child())
        .filter_map(|child| child.value().as_text())
        .map(|text| text.to_string())
        .collect();
    code_fragments.sort_by(|a, b| a.len().cmp(&b.len()));
    code_fragments.pop().ok_or_else(|| anyhow::anyhow!("No code blocks found"))
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
