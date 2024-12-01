use std::{fs::File, io::BufReader, path::PathBuf, borrow::Cow};

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
    bench_file_paths: Vec<String>,
    readme_file_paths: Vec<String>,
}

#[derive(Debug)]
struct MaterialisedConfig {
    template: String,
    exec_file_paths: Vec<PathBuf>,
    bench_template: String,
    bench_file_paths: Vec<PathBuf>,
    input_file_paths: Vec<PathBuf>,
    test_input_file_paths: Vec<PathBuf>,
    readme_file_paths: Vec<PathBuf>,
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
    day: u32,
    year: u32,
    problem_statement: String,
}

static JS_TEMPLATE: &str = include_str!("../templates/js.tmpl");
static PYTHON_TEMPLATE: &str = include_str!("../templates/python.tmpl");
static RUST_TEMPLATE: &str = include_str!("../templates/rust.tmpl");

static RUST_BENCH_TEMPLATE: &str = include_str!("../templates/rust_bench.tmpl");

static README_TEMPLATE: &str = include_str!("../templates/readme.tmpl");

fn main() -> Result<()> {
    let args = Args::parse();
    let n = format!("{:01$}", args.day, 2);
    let context = Context {
        n,
        day: args.day,
        year: args.year,
        problem_statement: "".to_owned(),
    };

    let config = get_config(&context, args.config)?;

    write_to_files(&config.template, &config.exec_file_paths)?;

    write_to_files(&config.bench_template, &config.bench_file_paths)?;

    let test_input = fetch_test_input(args.year, args.day)?;
    write_to_files(&test_input, &config.test_input_file_paths)?;

    let real_input = fetch_real_input(args.year, args.day)?;
    write_to_files(&real_input, &config.input_file_paths)?;

    let readme = fetch_readme(args.year, args.day, "".to_owned())?;
    write_to_files(&readme, &config.readme_file_paths)?;

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

fn get_config(context: &Context, path: PathBuf) -> Result<MaterialisedConfig> {
    let config_file = File::open(path)?;
    let reader = BufReader::new(config_file);
    let config: LangConfig = serde_json::from_reader(reader)?;

    let mut tt = TinyTemplate::new();

    let exec_file_template = match config.template_name.as_str() {
        "js" => JS_TEMPLATE,
        "python" => PYTHON_TEMPLATE,
        "rust" => RUST_TEMPLATE,
        _ => panic!("Unsupported template"),
    };
    tt.add_template("exec_file", exec_file_template).unwrap();
    let template = tt.render("exec_file", &context).unwrap();

    let bench_file_template = match config.template_name.as_str() {
        "rust" => RUST_BENCH_TEMPLATE,
        _ => panic!("Unsupported template"),
    };
    tt.add_template("bench_file", bench_file_template).unwrap();
    let bench_template = tt.render("bench_file", &context).unwrap();

    let bench_file_paths= materialise_paths(config.bench_file_paths, context);
    let exec_file_paths = materialise_paths(config.exec_file_paths, context);
    let test_input_file_paths = materialise_paths(config.test_input_file_paths, context);
    let input_file_paths = materialise_paths(config.input_file_paths, context);
    let readme_file_paths = materialise_paths(config.readme_file_paths, context);

    Ok(MaterialisedConfig {
        template,
        exec_file_paths,
        bench_template,
        bench_file_paths,
        input_file_paths,
        test_input_file_paths,
        readme_file_paths,
    })
}

fn fetch_readme(year: u32, day: u32, n: String) -> Result<String> {
    let html = fetch_problem_page(year, day)?;
    let re = regex::Regex::new(r"<main>(?s).*</main>").unwrap();
    let main = re.find(&html).unwrap().as_str();
    let problem_statement = html2md::parse_html(&main);

    let mut tt = TinyTemplate::new();
    tt.add_template("readme", README_TEMPLATE)?;
    let context = Context {
        problem_statement,
        year,
        day,
        n,
    };

    let mut readme = tt.render("readme", &context).unwrap();
    let replacements = [
        (r"&#39;", "'"),
        (r"&gt;", ">"),
        (r"&lt;", "<"),
    ];
    for (re, replacement) in replacements {
        let re = regex::Regex::new(re).unwrap();
        readme = re.replace_all(&readme, replacement).into_owned();
    }
    Ok(readme)
}

fn fetch_test_input(year: u32, day: u32) -> Result<String> {
    let html = fetch_problem_page(year, day)?;
    let fragment = Html::parse_fragment(&html);
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

fn fetch_problem_page(year: u32, day: u32) -> Result<String> {
    let url = format!("https://adventofcode.com/{year}/day/{day}");
    let client = reqwest::blocking::Client::new();
    let cookie = std::env::var("AOC_COOKIE")?;
    let request = client
        .get(url)
        .header("cookie", format!("session={cookie}"))
        .build()?;
    Ok(client.execute(request)?.text()?)
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

fn materialise_paths(input: Vec<String>, context: &Context) -> Vec<PathBuf> {
    let mut result = vec![];
    for path in input {
        let mut tt = TinyTemplate::new();
        tt.add_template("temp", &path).unwrap();
        result.push(PathBuf::from(tt.render("temp", &context).unwrap()));
    }
    result
}
