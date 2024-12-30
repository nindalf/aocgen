use jiff::{RoundMode, SpanRound, Unit, Zoned};

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

pub(crate) fn error_if_problem_locked(year: u32, day: u32) -> anyhow::Result<()> {
    error_if_time_before_problem_unlock(Zoned::now(), year, day)
}

// If it hasn't unlocked will return an error with the ceiling of the time remaining.
// If there is 0.000001 seconds remaining it will recommend "Try again in 1s"
fn error_if_time_before_problem_unlock(
    time: jiff::Zoned,
    year: u32,
    day: u32,
) -> anyhow::Result<()> {
    let unlock_time = jiff::civil::date(year as i16, 12, day as i8)
        .at(5, 00, 0, 0)
        .intz("UTC")?;

    let time_until_unlock = time.until(&unlock_time)?.round(
        SpanRound::new()
            .smallest(Unit::Second)
            .mode(RoundMode::Ceil),
    )?;
    if time_until_unlock.is_positive() {
        anyhow::bail!("Problem hasn't unlocked yet. Try again in {time_until_unlock:#}")
    }

    Ok(())
}

mod tests {
    #[test]
    fn test_exit() -> anyhow::Result<()> {
        let test_cases = [
            (
                "2024-11-30T16:27:29.999999999-08:00[America/Los_Angeles]",
                "4h 32m 31s",
            ),
            (
                "2024-11-29T16:27:29.1-05:00[America/New_York]",
                "31h 32m 31s",
            ),
            ("2024-12-01T04:59:58.999999999+00:00[UTC]", "2s"),
            ("2024-12-01T12:58:59.999+08:00[Asia/Singapore]", "1m 1s"),
            ("2024-12-01T16:59:00+13:00[Pacific/Auckland]", "1h 1m"),
        ];

        for (zdt_str, expected_unlock_time) in test_cases {
            let zdt = zdt_str.parse()?;
            match super::error_if_time_before_problem_unlock(zdt, 2024, 1) {
                Ok(_) => unreachable!("Expected error"),
                Err(err) => assert_eq!(
                    err.to_string(),
                    format!("Problem hasn't unlocked yet. Try again in {expected_unlock_time}")
                ),
            }
        }

        Ok(())
    }

    #[test]
    fn test_successful_unlock() -> anyhow::Result<()> {
        let test_cases = [
            ("2024-12-30T21:00:00.000000001-08:00[America/Los_Angeles]"),
            ("2024-12-01T00:00:00-05:00[America/New_York]"),
            ("2024-12-01T05:00:01+00:00[UTC]"),
            ("2024-12-01T13:00:10+08:00[Asia/Singapore]"),
            ("2024-12-01T18:01:00+13:00[Pacific/Auckland]"),
        ];

        for zdt_str in test_cases {
            let zdt = zdt_str.parse()?;
            super::error_if_time_before_problem_unlock(zdt, 2024, 1)?;
        }

        Ok(())
    }
}
