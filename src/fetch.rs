use std::{fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;
use tinytemplate::TinyTemplate;

use crate::{client, time, Language};

static JS_TEMPLATE: &str = include_str!("../templates/js.tmpl");
static PYTHON_TEMPLATE: &str = include_str!("../templates/python.tmpl");
static RUST_TEMPLATE: &str = include_str!("../templates/rust.tmpl");

static README_TEMPLATE: &str = include_str!("../templates/readme.tmpl");

static JS_CONFIG: &str = include_str!("../configs/js_config.json");
static PYTHON_CONFIG: &str = include_str!("../configs/python_config.json");
static RUST_CONFIG: &str = include_str!("../configs/rust_config.json");

#[derive(serde::Serialize, Debug, Clone)]
struct FetchContext {
    n: String,
    day: u32,
    year: u32,
    problem_statement: String,
    language: Language,
}

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

pub(crate) fn fetch_and_write(args: crate::FetchArgs) -> Result<()> {
    // Set year to current year if not provided
    let year = time::validate_year(args.year)?;
    time::exit_if_problem_locked(year, args.day)?;

    let context = FetchContext {
        n: args.day.to_string(),
        day: args.day,
        year,
        problem_statement: "".to_owned(),
        language: args.language,
    };

    let config = get_config(&context, args.config)?;

    let real_input = fetch_real_input(&context)?;
    write_to_files(&real_input, &config.input_file_paths)?;

    let readme = fetch_readme(&context)?;
    write_to_files(&readme, &config.readme_file_paths)?;

    let test_input = guess_test_input(&readme)?;
    write_to_files(test_input, &config.test_input_file_paths)?;

    write_to_files(&config.template, &config.exec_file_paths)?;

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

fn get_config(context: &FetchContext, path: Option<PathBuf>) -> Result<MaterialisedConfig> {
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

fn fetch_real_input(context: &FetchContext) -> Result<String> {
    let real_input_request = client::Request::Input(context.year, context.day);
    client::execute(real_input_request)
}

fn fetch_readme(context: &FetchContext) -> Result<String> {
    let request = crate::client::Request::ProblemPage(context.year, context.day);
    let problem_statement = crate::client::execute(request)?;

    let mut tt = TinyTemplate::new();
    tt.add_template("readme", README_TEMPLATE)?;
    let mut context = context.clone();
    context.problem_statement = problem_statement;

    let rendered = tt.render("readme", &context)?;
    let cleaned = client::clean_response(rendered)?;

    Ok(cleaned)
}

fn guess_test_input(readme: &str) -> Result<&str> {
    let re = regex::Regex::new(r"```\n([\s\S]*?)\n```")?;
    let mut code_blocks: Vec<&str> = re
        .captures_iter(readme)
        .filter_map(|s| s.get(1))
        .map(|s| s.as_str())
        .collect();

    code_blocks.sort_by_key(|a| a.len());

    if code_blocks.is_empty() {
        return Ok("");
    }

    Ok(code_blocks[code_blocks.len() - 1])
}

fn materialise_paths(input: Vec<String>, context: &FetchContext) -> Vec<PathBuf> {
    let mut result = vec![];
    for path in input {
        let mut tt = TinyTemplate::new();
        tt.add_template("temp", &path).unwrap();
        result.push(PathBuf::from(tt.render("temp", &context).unwrap()));
    }
    result
}
