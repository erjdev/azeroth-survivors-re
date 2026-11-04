//! Optional UTC calendar dates; old achievements retain an unknown date.

pub(super) const MAX_DAY: u32 = 2_932_896; // 9999-12-31, days since 1970-01-01.

pub(super) fn today() -> Option<u32> {
    let seconds = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let day = u32::try_from(seconds / 86_400).ok()?;
    (day <= MAX_DAY).then_some(day)
}

fn leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

pub(super) fn label(mut day: u32) -> Option<String> {
    if day > MAX_DAY {
        return None;
    }
    let mut year = 1970;
    loop {
        let days = if leap(year) { 366 } else { 365 };
        if day < days {
            break;
        }
        day -= days;
        year += 1;
    }
    let months = [
        ("Jan", 31),
        ("Feb", if leap(year) { 29 } else { 28 }),
        ("Mar", 31),
        ("Apr", 30),
        ("May", 31),
        ("Jun", 30),
        ("Jul", 31),
        ("Aug", 31),
        ("Sep", 30),
        ("Oct", 31),
        ("Nov", 30),
        ("Dec", 31),
    ];
    for (month, days) in months {
        if day < days {
            return Some(format!("{} {month} {year} UTC", day + 1));
        }
        day -= days;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_boundaries_include_leap_years_and_reject_out_of_range_days() {
        for (day, expected) in [
            (0, "1 Jan 1970 UTC"),
            (365, "1 Jan 1971 UTC"),
            (11016, "29 Feb 2000 UTC"),
            (11017, "1 Mar 2000 UTC"),
            (20695, "30 Aug 2026 UTC"),
            (47541, "1 Mar 2100 UTC"),
            (MAX_DAY, "31 Dec 9999 UTC"),
        ] {
            assert_eq!(label(day).as_deref(), Some(expected));
        }
        assert!(label(MAX_DAY + 1).is_none());
        assert!(label(u32::MAX).is_none());
    }
}
