use clap::{App, Arg};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use regex::{Regex, RegexBuilder};
use std::fs::metadata;
use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .about("Rust fortune")
        .author("Norihito <norihito@example.com>")
        .arg(
            Arg::with_name("sources")
                .value_name("SOURCE")
                .help("source files")
                .multiple(true)
                .required(true),
        )
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .help("matched text")
                .short("m")
                .long("pattern"),
        )
        .arg(
            Arg::with_name("seed")
                .value_name("SEED")
                .help("Random Seed")
                .short("s")
                .long("seed"),
        )
        .arg(
            Arg::with_name("insensitive")
                .value_name("INSENSITIVE")
                .help("Case-insensitive")
                .short("i")
                .long("insensitive")
                .takes_value(false),
        )
        .get_matches();

    let pattern = matches
        .value_of("pattern")
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid --pattern {}", val))
        })
        .transpose()?;

    let seed = matches.value_of("seed").map(parse_u64).transpose()?;

    Ok(Config {
        sources: matches.values_of_lossy("sources").unwrap(),
        pattern,
        seed,
    })
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;

    if let Some(pattern) = config.pattern {
        let mut prev_source = None;
        for fortune in fortunes.iter().filter(|f| pattern.is_match(&f.text)) {
            if prev_source.as_ref().map_or(true, |s| s != &fortune.source) {
                eprintln!("({})\n%", fortune.source);
                prev_source = Some(fortune.source.clone());
            }
            println!("{}\n%", fortune.text);
        }
    } else {
        println!(
            "{}",
            pick_fortune(&fortunes, config.seed).unwrap_or_else(|| "No fortunes found".to_string())
        )
    }
    Ok(())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut result = vec![];
    for path in paths {
        match metadata(path) {
            Err(e) => return Err(format!("{}: {}", path, e).into()),
            Ok(metadata) => {
                if metadata.is_file() {
                    result.push(PathBuf::from(path));
                } else if metadata.is_dir() {
                    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                        if entry.metadata().unwrap().is_file() {
                            result.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }
    }
    result.sort();
    result.dedup();
    Ok(result)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut result = vec![];
    let mut buf = vec![];

    for path in paths {
        match fs::File::open(path) {
            Err(e) => eprintln!("{}: {}", path.to_string_lossy().into_owned(), e),
            Ok(file) => {
                let reader = BufReader::new(&file);
                for line in reader.lines().filter_map(|e| e.ok()) {
                    if line == "%" && !buf.is_empty() {
                        result.push(Fortune {
                            source: path.file_name().unwrap().to_str().unwrap().to_string(),
                            text: buf.join("\n"),
                        });
                        buf.clear();
                    } else {
                        buf.push(line);
                    }
                }
            }
        }
    }
    Ok(result)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    if let Some(val) = seed {
        let mut rng = StdRng::seed_from_u64(val);
        fortunes.choose(&mut rng).map(|f| f.text.to_string())
    } else {
        let mut rng = rand::thread_rng();
        fortunes.choose(&mut rng).map(|f| f.text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{find_files, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // Parses all the fortunes without a filter
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Filters for matching text
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
