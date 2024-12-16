pub(crate) fn validate_year(year: u32) -> anyhow::Result<u32> {
    let current_year = current_year();
    if year == 0 {
        return Ok(current_year);
    }
    if year >= 2015 && year <= current_year {
        return Ok(year);
    }
    anyhow::bail!("You must choose a year between 2015 and {}", current_year)
}

fn current_year() -> u32 {
    let now = jiff::Zoned::now();
    now.year() as u32
}

pub(crate) fn exit_if_problem_locked(year: u32, day: u32) -> anyhow::Result<()> {
    let unlock_time = jiff::civil::date(year as i16, 12, day as i8)
        .at(5, 00, 0, 0)
        .intz("UTC")?;
    let now = jiff::Zoned::now();

    let time_until_unlock = now.duration_until(&unlock_time);
    if time_until_unlock.is_positive() {
        anyhow::bail!(
            "Problem hasn't unlocked yet. Try again in {}",
            time_until_unlock
        )
    }

    Ok(())
}
