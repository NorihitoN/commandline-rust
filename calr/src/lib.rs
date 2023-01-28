use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::{App, Arg};
use std::{error::Error, str::FromStr};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "Febrary",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "Octorber",
    "November",
    "December",
];
const LINE_WIDTH: usize = 22;

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .about("rust cal")
        .author("Norihito <norihito@example.com>")
        .arg(
            Arg::with_name("month")
                .value_name("MONTH")
                .help("Month name or number (1-12)")
                .short("m")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("year")
                .value_name("YEAR")
                .help("Year (1~9999)"),
        )
        .arg(
            Arg::with_name("year_flag")
                .help("Show whole current year")
                .short("y")
                .long("year")
                .takes_value(false)
                .conflicts_with_all(&["month", "year"]),
        )
        .get_matches();

    let today = Local::today();
    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches
        .value_of("year")
        .map_or(Ok(today.year()), parse_year)?;
    if matches.is_present("year_flag") {
        year = today.year();
        month = None;
    }

    Ok(Config {
        month,
        year,
        today: today.naive_local(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            let lines = format_month(config.year, 1, true, config.today);
            println!("{}", lines.join("\n"));
        }
    }
    Ok(())
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse::<T>()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    let parsed_year = parse_int(year)?;
    if parsed_year >= 1 && parsed_year <= 9999 {
        Ok(parsed_year)
    } else {
        Err(format!("year \"{}\" not in the range 1 through 9999", parsed_year).into())
    }
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        Ok(val) => {
            if (1..=12).contains(&val) {
                Ok(val)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", val).into())
            }
        }
        _ => {
            let lower = month.to_lowercase();
            let matched: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(&lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();
            if matched.len() == 1 {
                Ok(matched[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days: Vec<String> = (1..=first.weekday().num_days_from_sunday())
        .into_iter()
        .map(|_| "  ".to_string())
        .collect();

    let is_today = |day: u32| year == today.year() && month == today.month() && day == today.day();

    let last = last_day_in_month(year, month);
    days.extend((first.day()..=last.day()).into_iter().map(|num| {
        let fmt = format!("{:>2}", num);
        if is_today(num) {
            Style::new().reverse().paint(fmt).to_string()
        } else {
            fmt
        }
    }));
    let mut lines = vec![];
    let month_name = MONTH_NAMES[month as usize - 1];

    lines.push(format!(
        "{:^20}  ",
        if print_year {
            format!("{} {}", month_name, year)
        } else {
            month_name.to_string()
        }
    ));

    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());

    for week in days.chunks(7) {
        lines.push(format!(
            "{:width$}  ",
            week.join(" "),
            width = LINE_WIDTH - 2
        ));
    }
    lines
    // "   February 2020      ",
    // "Su Mo Tu We Th Fr Sa  ",
    // "                   1  ",
    // " 2  3  4  5  6  7  8  ",
    // " 9 10 11 12 13 14 15  ",
    // "16 17 18 19 20 21 22  ",
    // "23 24 25 26 27 28 29  ",
    // "                      ",
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let (y, m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd(y, m, 1).pred_opt().unwrap()
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_int() {
        // Parse positive int as usize
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // Parse negative int as i32
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // Fail on a string
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd(0, 1, 1);
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd(2021, 4, 7);
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }
    #[test]
    fn test_last_day_in_month() {
        assert_eq!(last_day_in_month(2020, 1), NaiveDate::from_ymd(2020, 1, 31));
        assert_eq!(last_day_in_month(2020, 2), NaiveDate::from_ymd(2020, 2, 29));
        assert_eq!(last_day_in_month(2020, 4), NaiveDate::from_ymd(2020, 4, 30));
    }
}
