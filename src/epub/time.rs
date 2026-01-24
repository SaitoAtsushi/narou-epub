/// 入力を日本時間として解釈して UTC として保持する
use std::fmt::{Display, Write};
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum Error {
    UnexpectedChar,
    EarlyTermination,
    UnexpectedRange,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Time {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl FromStr for Time {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 19 && s.as_bytes().iter().all(u8::is_ascii_graphic) {
            return Err(Error::EarlyTermination);
        };
        if s.as_bytes()[4] != b'-'
            || s.as_bytes()[7] != b'-'
            || s.as_bytes()[10] != b' '
            || s.as_bytes()[13] != b':'
            || s.as_bytes()[16] != b':'
        {
            return Err(Error::UnexpectedChar);
        }
        let year: u16 = s[0..=3].parse().map_err(|_| Error::UnexpectedChar)?;
        let month = s[5..=6].parse().map_err(|_| Error::UnexpectedChar)?;
        let day = s[8..=9].parse().map_err(|_| Error::UnexpectedChar)?;
        let hour = s[11..=12].parse().map_err(|_| Error::UnexpectedChar)?;
        let minute = s[14..=15].parse().map_err(|_| Error::UnexpectedChar)?;
        let second = s[17..=18].parse().map_err(|_| Error::UnexpectedChar)?;

        Time::new(year, month, day, hour, minute, second)
    }
}

#[cfg(test)]
const fn year_to_days(year: u16) -> u32 {
    let y = year as u32;
    (y - 1) * 365 + (y - 1) / 4 - (y - 1) / 100 + (y - 1) / 400
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100))
}

const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Unix Epoch からの日数を計算する
#[cfg(test)]
fn date_to_days(year: u16, month: u8, day: u8) -> u32 {
    let days = year_to_days(year) - year_to_days(1970)
        + DAYS_IN_MONTH.iter().take(month as usize - 1).sum::<u32>()
        + u32::from((month > 2) && is_leap_year(year))
        + day as u32
        - 1;
    days
}

impl Time {
    #[cfg(test)]
    fn to_unix_time(&self) -> u64 {
        date_to_days(self.year, self.month, self.day) as u64 * 24 * 60 * 60
            + self.hour as u64 * 60 * 60
            + self.minute as u64 * 60
            + self.second as u64
    }

    fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Result<Self, Error> {
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

            Ok(Time {
                year,
                month,
                day,
                hour,
                minute,
                second,
            })
        } else {
            Err(Error::UnexpectedRange)
        }
    }
}

trait WriteFixWidth {
    fn write_fix_width<const WIDTH: usize>(&mut self, n: u32) -> std::fmt::Result;
}

impl WriteFixWidth for std::fmt::Formatter<'_> {
    fn write_fix_width<const WIDTH: usize>(&mut self, mut n: u32) -> std::fmt::Result {
        let mut a = [0_u8; WIDTH];
        for i in (0..WIDTH).rev() {
            a[i] = (n % 10) as u8 + b'0';
            n /= 10;
        }
        for ch in a {
            self.write_char(ch as char)?;
        }
        Ok(())
    }
}

impl Display for Time {
    /// ISO 8601の形式で表示する
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fix_width::<4>(self.year as _)?;
        f.write_char('-')?;
        f.write_fix_width::<2>(self.month as _)?;
        f.write_char('-')?;
        f.write_fix_width::<2>(self.day as _)?;
        f.write_char('T')?;
        f.write_fix_width::<2>(self.hour as _)?;
        f.write_char(':')?;
        f.write_fix_width::<2>(self.minute as _)?;
        f.write_char(':')?;
        f.write_fix_width::<2>(self.second as _)?;
        f.write_char('Z')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JST: u64 = 9 * 60 * 60; // 日本のタイムゾーンは +09:00

    #[test]
    fn cases_of_success() {
        // うるう年の二月二十九日は許容される
        assert_eq!(
            Time::from_str("2016-02-29 12:34:56").unwrap(),
            Time::new(2016, 2, 29, 12, 34, 56).unwrap()
        );

        // タイムゾーン補正とUNIX時間の算出
        const TEST_CASES: [(&str, &str, u64); 15] = [
            (
                "1999-01-01 00:00:00",
                "1998-12-31T15:00:00Z",
                915148800 - JST,
            ),
            (
                "2000-03-01 00:00:00",
                "2000-02-29T15:00:00Z",
                951868800 - JST,
            ),
            (
                "2000-02-28 23:59:59",
                "2000-02-28T14:59:59Z",
                951782399 - JST,
            ),
            (
                "9999-12-31 23:59:59",
                "9999-12-31T14:59:59Z",
                253402300799 - JST,
            ),
            (
                "2038-01-19 03:14:07",
                "2038-01-18T18:14:07Z",
                2147483647 - JST,
            ),
            (
                "2038-01-19 03:14:08",
                "2038-01-18T18:14:08Z",
                2147483648 - JST,
            ),
            (
                "1998-12-31 23:59:59",
                "1998-12-31T14:59:59Z",
                915148799 - JST,
            ),
            (
                "2000-02-29 00:00:00",
                "2000-02-28T15:00:00Z",
                951782400 - JST,
            ),
            (
                "1972-02-28 23:59:59",
                "1972-02-28T14:59:59Z",
                68169599 - JST,
            ),
            (
                "1972-02-29 00:00:00",
                "1972-02-28T15:00:00Z",
                68169600 - JST,
            ),
            (
                "1972-02-29 23:59:59",
                "1972-02-29T14:59:59Z",
                68255999 - JST,
            ),
            (
                "1972-03-01 00:00:00",
                "1972-02-29T15:00:00Z",
                68256000 - JST,
            ),
            (
                "1971-02-28 23:59:59",
                "1971-02-28T14:59:59Z",
                36633599 - JST,
            ),
            (
                "1971-03-01 00:00:00",
                "1971-02-28T15:00:00Z",
                36633600 - JST,
            ),
            (
                "1998-12-31 23:59:60",
                "1998-12-31T14:59:60Z",
                915148800 - JST,
            ),
        ];
        for (s, utc, n) in TEST_CASES {
            let time = Time::from_str(s).unwrap();
            assert_eq!(time.to_string(), utc);
            assert_eq!(time.to_unix_time(), n);
        }
    }

    #[test]
    fn cases_of_out_range() {
        const TEST_CASES: [&'static str; 7] = [
            "2017-02-29 12:34:56", // うるう年以外は二月二十九日はない
            "1969-02-28 12:34:56", // 1970年未満の時間は扱えないことにする
            "2016-00-28 12:34:56", // 月がゼロということはない
            "2016-01-00 12:34:56", // 日がゼロということはない
            "2016-02-28 24:34:56", // 時が大きすぎる
            "2016-02-28 12:60:56", // 分が大きすぎる
            "2016-02-28 12:34:60", // 秒が大きすぎる
        ];
        for case in TEST_CASES {
            assert_eq!(Time::from_str(case), Err(Error::UnexpectedRange));
        }
    }
}
