pub(crate) fn submit_answer(args: crate::SubmitArgs, default_year: u32) -> anyhow::Result<()> {
    let year = if args.year == 0 {
        default_year
    } else {
        args.year
    };
    let day = args.day;
    let url = format!("https://adventofcode.com/{year}/day/{day}/answer");

    let cookie = match std::env::var("AOC_COOKIE") {
        Ok(cookie) => cookie,
        Err(_) => {
            eprintln!("AOC_COOKIE environment variable not set.\nFind the cookie value from your browser and set it as an environment variable.\nexport AOC_COOKIE=<cookie>");
            std::process::exit(1);
        }
    };
    let client = reqwest::blocking::Client::new();
    let form = [("level", args.part), ("answer", args.answer)];

    let request = client
        .post(url)
        .header("cookie", format!("session={cookie}"))
        .form(&form)
        .build()?;

    let html = client.execute(request)?.text()?;
    let re = regex::Regex::new(r"<main>(?s).*</main>").unwrap();
    let main = re.find(&html).unwrap().as_str();
    let response = html2md::parse_html(main);
    println!("{response}");

    Ok(())
}
