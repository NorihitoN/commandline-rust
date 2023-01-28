use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("Norihito norihtito@exmaple.com")
        .about("Rust uniq")
        .arg(
            Arg::with_name("input_file")
                .value_name("IN_FILE")
                .default_value("-")
                .help("input file to compare adjacent lines"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .help("output file to writes a copy of uniqu input"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .takes_value(false)
                .help("precede each output line with the count of the numer of times"),
        )
        .get_matches();
    Ok(Config {
        in_file: matches.value_of_lossy("input_file").unwrap().to_string(),
        out_file: matches.value_of("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut out_file: Box<dyn Write> = match config.out_file {
        Some(out_name) => Box::new(File::create(out_name)?),
        _ => Box::new(io::stdout()),
    };
    let mut line_cur = String::new();
    let mut line_prev = String::new();
    let mut count: u64 = 0;
    let mut print = |count: u64, text: &str| -> MyResult<()> {
        if config.count {
            write!(out_file, "{:>4} {}", count, text)?;
        } else {
            write!(out_file, "{}", text)?;
        }
        Ok(())
    };
    loop {
        let byte = file.read_line(&mut line_cur)?;
        if count == 0 || line_prev.trim_end() == line_cur.trim_end() {
            count += 1;
        } else {
            print(count, &line_prev)?;
            count = 1;
        }
        if count <= 1 {
            line_prev = line_cur.clone();
        }
        line_cur.clear();
        if byte == 0 {
            break;
        }
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
