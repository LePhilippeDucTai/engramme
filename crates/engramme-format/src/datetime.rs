/// Whether `year-month-day` exists in the Gregorian calendar.
pub(crate) fn is_calendar_date(year: u32, month: u32, day: u32) -> bool {
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

/// Value of the `n` ASCII digits at the start of `bytes`.
fn number(bytes: &[u8], n: usize) -> Option<u32> {
    let digits = bytes.get(..n)?;
    digits.iter().try_fold(0u32, |acc, &b| {
        b.is_ascii_digit().then(|| acc * 10 + u32::from(b - b'0'))
    })
}

/// Whether `s` is an ISO 8601 date-time: `YYYY-MM-DDTHH:MM`, then optionally
/// `:SS` (and a fraction `.d…` after the seconds), then optionally `Z` or an
/// offset `±HH:MM`.
pub(crate) fn is_date_time(s: &str) -> bool {
    let b = s.as_bytes();
    let field = |at: usize, n: usize| b.get(at..).and_then(|rest| number(rest, n));
    let sep = |at: usize, c: u8| b.get(at) == Some(&c);
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute)) = (
        field(0, 4),
        field(5, 2),
        field(8, 2),
        field(11, 2),
        field(14, 2),
    ) else {
        return false;
    };
    if !(sep(4, b'-') && sep(7, b'-') && sep(10, b'T') && sep(13, b':'))
        || !is_calendar_date(year, month, day)
        || hour > 23
        || minute > 59
    {
        return false;
    }
    let mut at = 16;
    if sep(at, b':') {
        match field(at + 1, 2) {
            Some(second) if second <= 59 => at += 3,
            _ => return false,
        }
        if sep(at, b'.') {
            let digits = b[at + 1..]
                .iter()
                .take_while(|c| c.is_ascii_digit())
                .count();
            if digits == 0 {
                return false;
            }
            at += 1 + digits;
        }
    }
    match b.get(at) {
        None => true,
        Some(b'Z') => at + 1 == b.len(),
        Some(b'+' | b'-') => {
            at + 6 == b.len()
                && sep(at + 3, b':')
                && field(at + 1, 2).is_some_and(|h| h <= 23)
                && field(at + 4, 2).is_some_and(|m| m <= 59)
        }
        Some(_) => false,
    }
}
