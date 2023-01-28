use crate::Column::*;
use clap::{App, Arg};
use std::cmp::Ordering::*;
use std::fs::File;
use std::{
    error::Error,
    io::{self, BufRead, BufReader},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    insensitive: bool,
    delimiter: String,
}

enum Column<'a> {
    Col1(&'a str),
    Col2(&'a str),
    Col3(&'a str),
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("commr")
        .about("Rust comm")
        .version("0.1.0")
        .author("Norihito <norihito@example.com>")
        .arg(
            Arg::with_name("file1")
                .value_name("FILE1")
                .help("input file1")
                .required(true),
        )
        .arg(
            Arg::with_name("file2")
                .value_name("FILE2")
                .help("input file2")
                .required(true),
        )
        .arg(
            Arg::with_name("show_col1")
                .value_name("show_col1")
                .short("1")
                .help("Suppress printing of column 1")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("show_col2")
                .value_name("show_col2")
                .short("2")
                .help("Suppress printing of column 2")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("show_col3")
                .value_name("show_col3")
                .short("3")
                .help("Suppress printing of column 3")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("insensitive")
                .value_name("insensitive")
                .short("i")
                .long("insensitive")
                .help("insensitive regex")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("delimiter")
                .value_name("DELIM")
                .short("d")
                .long("output-delimiter")
                .help("use DELIM instead of TAB for delimiter")
                .default_value("\t"),
        )
        .get_matches();

    Ok(Config {
        file1: matches.value_of("file1").unwrap().to_string(),
        file2: matches.value_of("file2").unwrap().to_string(),
        show_col1: !matches.is_present("show_col1"),
        show_col2: !matches.is_present("show_col2"),
        show_col3: !matches.is_present("show_col3"),
        insensitive: matches.is_present("insensitive"),
        delimiter: matches.value_of("delimiter").unwrap().to_string(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let file1 = &config.file1;
    let file2 = &config.file2;

    let print = |col: Column| {
        let mut columns = vec![];
        match col {
            Col1(val) => {
                if config.show_col1 {
                    columns.push(val);
                }
            }
            Col2(val) => {
                if config.show_col2 {
                    if config.show_col1 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
            Col3(val) => {
                if config.show_col3 {
                    if config.show_col1 {
                        columns.push("");
                    }
                    if config.show_col2 {
                        columns.push("");
                    }
                    columns.push(val);
                }
            }
        };

        if !columns.is_empty() {
            println!("{}", columns.join(&config.delimiter));
        }
    };

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let mut lines1 = open(file1)?.lines().filter_map(Result::ok);
    let mut lines2 = open(file2)?.lines().filter_map(Result::ok);

    let mut line1 = lines1.next();
    let mut line2 = lines2.next();

    while line1.is_some() || line2.is_some() {
        match (&line1, &line2) {
            (Some(val1), Some(val2)) => match val1.cmp(val2) {
                Equal => {
                    print(Col3(val1));
                    line1 = lines1.next();
                    line2 = lines2.next();
                }
                Less => {
                    print(Col1(val1));
                    line1 = lines1.next();
                }
                Greater => {
                    print(Col2(val2));
                    line2 = lines2.next();
                } // let _val1 = val1.parse::<char>().unwrap() as u8;
                  // let _val2 = val2.parse::<char>().unwrap() as u8;
                  // if _val1 == _val2 {
                  //     println!("{}", val1);
                  //     line1 = lines1.next();
                  //     line2 = lines2.next();
                  // } else if _val1 < _val2 {
                  //     println!("{}", val1);
                  //     line1 = lines1.next();
                  // } else {
                  //     println!("{}", val2);
                  //     line2 = lines2.next();
                  // }
            },
            (Some(val1), None) => {
                print(Col1(val1));
                line1 = lines1.next();
            }
            (None, Some(val2)) => {
                print(Col2(val2));
                line2 = lines2.next();
            }
            _ => (),
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
