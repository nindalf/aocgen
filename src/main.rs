use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use chrono::Datelike;
use clap::Parser;
use scraper::{Html, Selector};
use serde::Deserialize;
use tinytemplate::TinyTemplate;

#[derive(Deserialize, Debug)]
struct LangConfig {
    exec_file_paths: Vec<String>,
    input_file_paths: Vec<String>,
    test_input_file_paths: Vec<String>,
    readme_file_paths: Vec<String>,
}

#[derive(Debug)]
struct MaterialisedConfig {
    template: String,
    exec_file_paths: Vec<PathBuf>,
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
    #[arg(short, long, default_value_t = 0)]
    year: u32,
    /// Config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(value_enum, default_value_t=Language::Rust)]
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
enum Language {
    JS,
    Python,
    Rust,
}

static JS_TEMPLATE: &str = include_str!("../templates/js.tmpl");
static PYTHON_TEMPLATE: &str = include_str!("../templates/python.tmpl");
static RUST_TEMPLATE: &str = include_str!("../templates/rust.tmpl");

static README_TEMPLATE: &str = include_str!("../templates/readme.tmpl");

static JS_CONFIG: &str = include_str!("../configs/js_config.json");
static PYTHON_CONFIG: &str = include_str!("../configs/python_config.json");
static RUST_CONFIG: &str = include_str!("../configs/rust_config.json");

fn main() -> Result<()> {
    let args = Args::parse();
    let n = format!("{}", args.day);
    // Set year to current year if not provided
    let year = if args.year == 0 {
        let now = chrono::Utc::now();
        now.year_ce().1
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

    let config = get_config(&context, args.config)?;

    write_to_files(&config.template, &config.exec_file_paths)?;

    let test_input = fetch_test_input(&context)?;
    write_to_files(&test_input, &config.test_input_file_paths)?;

    let real_input = fetch_real_input(&context)?;
    write_to_files(&real_input, &config.input_file_paths)?;

    let readme = fetch_readme(&context)?;
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

fn get_config(context: &Context, path: Option<PathBuf>) -> Result<MaterialisedConfig> {
    let config: LangConfig = if let Some(path) = path {
        let config_file = File::open(path)?;
        let reader = BufReader::new(config_file);
        serde_json::from_reader(reader)?
    } else {
        let config_str = match context.language {
            Language::JS => JS_CONFIG,
            Language::Python => PYTHON_CONFIG,
            Language::Rust => RUST_CONFIG,
        };
        serde_json::from_str(config_str)?
    };

    let mut tt = TinyTemplate::new();

    let exec_file_template = match context.language {
        Language::JS => JS_TEMPLATE,
        Language::Python => PYTHON_TEMPLATE,
        Language::Rust => RUST_TEMPLATE,
    };
    tt.add_template("exec_file", exec_file_template).unwrap();
    let template = tt.render("exec_file", &context).unwrap();

    let exec_file_paths = materialise_paths(config.exec_file_paths, context);
    let test_input_file_paths = materialise_paths(config.test_input_file_paths, context);
    let input_file_paths = materialise_paths(config.input_file_paths, context);
    let readme_file_paths = materialise_paths(config.readme_file_paths, context);

    Ok(MaterialisedConfig {
        template,
        exec_file_paths,
        input_file_paths,
        test_input_file_paths,
        readme_file_paths,
    })
}

fn fetch_readme(context: &Context) -> Result<String> {
    let html = fetch_problem_page(context)?;
    let re = regex::Regex::new(r"<main>(?s).*</main>").unwrap();
    let main = re.find(&html).unwrap().as_str();
    let problem_statement = html2md::parse_html(main);

    let mut tt = TinyTemplate::new();
    tt.add_template("readme", README_TEMPLATE)?;
    let mut context = context.clone();
    context.problem_statement = problem_statement;

    let mut readme = tt.render("readme", &context).unwrap();
    let replacements = [(r"&#39;", "'"), (r"&gt;", ">"), (r"&lt;", "<")];
    for (re, replacement) in replacements {
        let re = regex::Regex::new(re).unwrap();
        readme = re.replace_all(&readme, replacement).into_owned();
    }
    Ok(readme)
}

fn fetch_test_input(context: &Context) -> Result<String> {
    let html = fetch_problem_page(context)?;
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

fn fetch_problem_page(context: &Context) -> Result<String> {
    let year = context.year;
    let day = context.day;
    let url = format!("https://adventofcode.com/{year}/day/{day}");
    let client = reqwest::blocking::Client::new();
    let cookie = match std::env::var("AOC_COOKIE") {
        Ok(cookie) => cookie,
        Err(_) => {
            eprintln!("AOC_COOKIE environment variable not set.\nFind the cookie value from your browser and set it as an environment variable.\nexport AOC_COOKIE=<cookie>");
            std::process::exit(1);
        }
    };
    let request = client
        .get(url)
        .header("cookie", format!("session={cookie}"))
        .build()?;
    Ok(client.execute(request)?.text()?)
}

fn fetch_real_input(context: &Context) -> Result<String> {
    let year = context.year;
    let day = context.day;
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
