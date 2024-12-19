use crate::{
    client::{self, Request},
    time,
};

pub(crate) fn submit_answer(args: crate::SubmitArgs) -> anyhow::Result<()> {
    let year = time::validate_year(args.year)?;
    let day = args.day;

    time::exit_if_problem_locked(year, day)?;

    let part = validate_part(&args.part)?;
    let form = [("level", part), ("answer", &args.answer)];
    let request = Request::Answer(year, day, &form);
    let response = client::execute(request)?;
    let cleaned_resposne = client::clean_response(response)?;
    println!("{cleaned_resposne}");

    Ok(())
}

fn validate_part(part: &str) -> anyhow::Result<&str> {
    if part == "1" || part == "2" {
        return Ok(part);
    }
    anyhow::bail!("You must choose part 1 or part 2")
}
