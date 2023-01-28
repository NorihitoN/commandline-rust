mod owner;

use chrono::{DateTime, Local};
use clap::{App, Arg};
use users::{get_user_by_uid, get_group_by_gid};
use std::{error::Error, path::PathBuf, fs::{metadata, read_dir}, os::unix::prelude::MetadataExt};
use tabular::{Row, Table};
use owner::Owner;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config{
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("lsr")
    .about("Rust ls")
    .version("0.1.0")
    .author("Norihito <norihito@example.com>")
    .arg(
        Arg::with_name("paths")
        .value_name("PATHS")
        .help("search paths")
        .default_value(".")
        .multiple(true)
    )
    .arg(
        Arg::with_name("long")
        .help("show long information")
        .short("l")
        .long("long")
        .takes_value(false)
    )
    .arg(
        Arg::with_name("show_hidden")
        .help("show hidden files")
        .short("a")
        .long("all")
        .takes_value(false)
    )
    .get_matches();

    Ok(Config{
        paths: matches.values_of_lossy("paths").unwrap(),
        long: matches.is_present("long"),
        show_hidden: matches.is_present("show_hidden"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    //          1   2   3   4   5   6   7   8
    let fmt = "{:<}{:<} {:>} {:<} {:<} {:>} {:<} {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let meta = path.metadata()?;
        let uid = meta.uid();
        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| uid.to_string());

        let gid = meta.gid();
        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| gid.to_string());
        
        let file_type = if path.is_dir() { "d" } else {"-" };
        let perms = format_mode(meta.mode());
        let modified: DateTime<Local> = DateTime::from(meta.modified()?);

        table.add_row(
            Row::new()
            .with_cell(file_type)
            .with_cell(perms) // 1 "d" or "-"
            .with_cell(meta.nlink()) // 1 "d" or "-"
            .with_cell(user) // 1 "d" or "-"
            .with_cell(group) // 1 "d" or "-"
            .with_cell(meta.len()) // 1 "d" or "-"
            .with_cell(modified.format("%b %d %y %H:%M"))
            .with_cell(path.display()), // 1 "d" or "-"
        );
    }
    Ok(format!("{}", table))
}

// assert_eq!(format_mode(0o755), "rwxr-xr-x");
/// Given a file mode in octal format like 0o752,
/// return a string like "rwxr-x--x"
fn format_mode(mode: u32) -> String {
    format!(
        "{}{}{}",
        mk_triple(mode, Owner::User),
        mk_triple(mode, Owner::Group),
        mk_triple(mode, Owner::Other),
    )
}

pub fn mk_triple(mode: u32, owner: Owner) -> String {
    let [read, write, execute] = owner.masks();
    format!{
        "{}{}{}",
        if mode & read == 0 { "-" } else { "r" },
        if mode & write == 0 { "-" } else { "w" },
        if mode & execute == 0 { "-" } else { "x" },

    }
}

fn find_files(
    paths: &[String],
    show_hidden: bool
) -> MyResult<Vec<PathBuf>> {
    let mut res = vec![];
    for path in paths {
        match metadata(path) {
            Err(e) => eprintln!("{}: {}", path, e),
            Ok(meta) => {
                if meta.is_file() {
                    res.push(PathBuf::from(path));
                } else if meta.is_dir() {
                    for entry in read_dir(path)? {
                        let entry = entry?;
                        let is_hidden = entry.file_name().to_str().map(|s| s.starts_with('.')).unwrap_or(false);
                        if !is_hidden || show_hidden {
                            res.push(entry.path());
                        }
                    }
                }
            }
        }
    }
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::{find_files, format_mode, mk_triple, Owner, format_output};
    use std::path::PathBuf;
    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(parts.len() > 0 && parts.len() <= 10);

        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> =
            out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> =
            out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            &empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(&dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}