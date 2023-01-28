use clap::{App, Arg};
use core::str;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("Norihito <norihito@example.com>")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("input file")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .long("lines")
                .short("l")
                .help("The number of lines in each input file")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("words")
                .long("words")
                .short("w")
                .help("The number of words in each input file")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("bytes")
                .long("bytes")
                .short("c")
                .help("The number of bytes in each input file")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("chars")
                .long("chars")
                .short("m")
                .help("The number of characters in each input file")
                .takes_value(false)
                .conflicts_with("bytes"),
        )
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    // println!("{:#?}", config);
    let file_num = config.files.len();
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in config.files {
        match open(&filename) {
            Err(e) => eprint!("{}: {}", filename, e),
            Ok(file) => {
                if let Ok(fileinfo) = count(file) {
                    println!(
                        "{}{}{}{}{}",
                        format_field(fileinfo.num_lines, config.lines),
                        format_field(fileinfo.num_words, config.words),
                        format_field(fileinfo.num_bytes, config.bytes),
                        format_field(fileinfo.num_chars, config.chars),
                        if filename == "-" {
                            "".to_string()
                        } else {
                            format!(" {}", filename)
                        }
                    );
                    total_lines += fileinfo.num_lines;
                    total_words += fileinfo.num_words;
                    total_bytes += fileinfo.num_bytes;
                    total_chars += fileinfo.num_chars;
                }
            }
        }
    }
    if file_num > 1 {
        println!(
            "{}{}{}{}{}",
            format_field(total_lines, config.lines),
            format_field(total_words, config.words),
            format_field(total_bytes, config.bytes),
            format_field(total_chars, config.chars),
            " total"
        );
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut line = String::new();

    loop {
        let line_bytes = file.read_line(&mut line)?;
        match line_bytes {
            0 => break,
            _ => {
                num_bytes += line_bytes;
                num_words += line.split_whitespace().count();
                num_lines += 1;
                num_chars += line.chars().count();
                line.clear();
            }
        }
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use crate::format_field;

    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_bytes: 48,
            num_chars: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
