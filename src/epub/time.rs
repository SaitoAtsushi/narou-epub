use utcdatetime::{DateTime, DateTimeParseError};

const fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) & (!year.is_multiple_of(100) | year.is_multiple_of(400))
}

const DAYS_IN_MONTH: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

pub trait FromJST {
    fn from_jst_str(s: &str) -> Result<Self, DateTimeParseError>
    where
        Self: Sized;
}

impl FromJST for DateTime {
    fn from_jst_str(s: &str) -> Result<DateTime, DateTimeParseError> {
        match s.as_bytes() {
            [
                y1,
                y2,
                y3,
                y4,
                b'-',
                mo1,
                mo2,
                b'-',
                d1,
                d2,
                b' ',
                h1,
                h2,
                b':',
                mi1,
                mi2,
                b':',
                s1,
                s2,
            ] if [y1, y2, y3, y4, mo1, mo2, d1, d2, h1, h2, mi1, mi2, s1, s2]
                .iter()
                .all(|&&x| x.is_ascii_digit()) =>
            {
                let year: u16 = s[0..4].parse().map_err(|_| DateTimeParseError)?;
                let month: u8 = s[5..7].parse().map_err(|_| DateTimeParseError)?;
                let day: u8 = s[8..10].parse().map_err(|_| DateTimeParseError)?;
                let hour: u8 = s[11..13].parse().map_err(|_| DateTimeParseError)?;
                let minute: u8 = s[14..16].parse().map_err(|_| DateTimeParseError)?;
                let second: u8 = s[17..19].parse().map_err(|_| DateTimeParseError)?;
                if (1970..=9999).contains(&year)
                    && (1..=12).contains(&month)
                    && (1..=DAYS_IN_MONTH[month as usize - 1] as u8
                        + u8::from(is_leap_year(year) && month == 2))
                        .contains(&day)
                    && (0..=23).contains(&hour)
                    && (0..=59).contains(&minute)
                    && (0..=59 + u8::from(hour == 23 && minute == 59)).contains(&second)
                {
                    let (hour, bf) = if hour >= 9 {
                        (hour - 9, 0)
                    } else {
                        (hour + 15, 1)
                    };
                    let (day, bf) = if day > bf {
                        (day - bf, 0)
                    } else {
                        (
                            DAYS_IN_MONTH[((month + 10) % 12) as usize] as u8
                                + u8::from(is_leap_year(year) && (month - 1 == 2)),
                            1,
                        )
                    };
                    let (month, bf) = if month > bf {
                        (month - bf, 0u16)
                    } else {
                        (12, 1u16)
                    };
                    let year = year - bf;

                    Ok(DateTime::new(year, month, day, hour, minute, second)
                        .ok_or(DateTimeParseError))?
                } else {
                    Err(DateTimeParseError)
                }
            }
            _ => Err(DateTimeParseError),
        }
    }
}
