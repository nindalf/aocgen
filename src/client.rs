use anyhow::Context;
use reqwest::Method;

pub(crate) enum Request<'a> {
    ProblemPage(u32, u32),
    Input(u32, u32),
    Answer(u32, u32, &'a [(&'a str, &'a str)]),
}

pub(crate) fn execute(request: Request) -> anyhow::Result<String> {
    let (method, url, form): (Method, String, &[(&str, &str)]) = match request {
        Request::ProblemPage(year, day) => (
            Method::GET,
            format!("https://adventofcode.com/{year}/day/{day}"),
            &[],
        ),
        Request::Input(year, day) => (
            Method::GET,
            format!("https://adventofcode.com/{year}/day/{day}/input"),
            &[],
        ),
        Request::Answer(year, day, form) => (
            Method::POST,
            format!("https://adventofcode.com/{year}/day/{day}/answer"),
            form,
        ),
    };

    let client = reqwest::blocking::Client::new();
    let cookie = std::env::var("AOC_COOKIE")
        .with_context(|| "AOC_COOKIE environment variable not set.\nFind the cookie value from your browser and set it as an environment variable.\nexport AOC_COOKIE=<cookie>")?;
    let reqwest = client
        .request(method, url)
        .form(form)
        .header("cookie", format!("session={cookie}"))
        .build()?;

    let resp = client.execute(reqwest)?.text()?;

    match request {
        Request::ProblemPage(_, _) | Request::Answer(_, _, _) => parse_main(resp),
        Request::Input(_, _) => Ok(resp),
    }
}

fn parse_main(html: String) -> anyhow::Result<String> {
    let re = regex::Regex::new(r"<main>(?s).*</main>")?;
    let main = re.find(&html).unwrap().as_str();
    Ok(html2md::parse_html(main))
}

pub(crate) fn clean_response(mut response: String) -> anyhow::Result<String> {
    let replacements = [
        (r"&#39;", "'"),
        (r"&gt;", ">"),
        (r"&lt;", "<"),
        (r"&quot;", "\""),
        (r"You can .*Mastodon.* this puzzle.\n", ""),
        (r" ?You can .*Mastodon.*this victory or ", "\n"),
    ];
    for (re, replacement) in replacements {
        let re = regex::Regex::new(re)?;
        response = re.replace_all(&response, replacement).into_owned();
    }
    Ok(response)
}

mod tests {
    #[test]
    fn test_clean_response() {
        let cases = [
            (
                "That's the right answer! You are one gold star closer to finding the Chief Historian.\n\nYou have completed Day 19! You can [Shareon [Bluesky](https://bsky.app/intent/compose?text=I+just+completed+%22Linen+Layout%22+%2D+Day+19+%2D+Advent+of+Code+2024+%23AdventOfCode+https%3A%2F%2Fadventofcode%2Ecom%2F2024%2Fday%2F19) [Twitter](https://twitter.com/intent/tweet?text=I+just+completed+%22Linen+Layout%22+%2D+Day+19+%2D+Advent+of+Code+2024&url=https%3A%2F%2Fadventofcode%2Ecom%2F2024%2Fday%2F19&related=ericwastl&hashtags=AdventOfCode) [Mastodon](javascript:void(0);)] this victory or [[Return to Your Advent Calendar]](/2024).",
                "That's the right answer! You are one gold star closer to finding the Chief Historian.\n\nYou have completed Day 19!\n[[Return to Your Advent Calendar]](/2024)."
            ),
            (
                "&#39; &gt; &lt; &quot;",
                "' > < \"",
            ),
            (
                "&quot;air quotes&quot;",
                "\"air quotes\"",
            ),
        ];
        for (input, output) in cases {
            assert_eq!(output, &super::clean_response(input.to_owned()).unwrap());
        }
    }
}
