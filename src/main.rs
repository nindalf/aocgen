use anyhow::Result;
use clap::Parser;
use tinytemplate::TinyTemplate;

static TEMPLATE: &'static str = include_str!("dayn.rs.tmpl");

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

    let rs_path = std::path::Path::new("src").join(format!("day{n}.rs"));
    if !rs_path.exists() {
        let rendered = rendered_template(&n)?;
        std::fs::write(rs_path, rendered)?;
    }

    let inputs = std::path::Path::new("inputs");
    if !inputs.exists() {
        std::fs::create_dir(inputs)?;
    }

    let dummy_test_input = inputs.join(format!("day{n}-test.txt"));
    if !dummy_test_input.exists() {
        std::fs::File::create(dummy_test_input)?;
    }

    let real_test_input = inputs.join(format!("day{n}.txt"));
    if !real_test_input.exists() {
        let test_input = fetch_test_input(args.year, args.day)?;
        std::fs::write(real_test_input, test_input)?;
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
    let url = format!("https://adventofcode.com/{year}/day/{day}/input");
    let client = reqwest::blocking::Client::new();
    let cookie = std::env::var("AOC_COOKIE")?;
    let request = client
        .get(url)
        .header("cookie", format!("session={cookie}"))
        .build()?;
    Ok(client.execute(request)?.text()?)
}
