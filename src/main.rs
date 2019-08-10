//! a tool for transforming and categorizing git log output
//! expects input in the format of the output of
//! `git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merges`
use recap::Recap;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{stdin, BufRead, BufReader},
    path::Path as StdPath,
};

#[derive(Clone, Deserialize, Recap)]
#[recap(regex = r#"(?x)
    "(?P<sha>\S+)"
    ,
    "(?P<author>\S+)"
    ,
    "(?P<timestamp>.+)"
  "#)]
struct Header {
    sha: String,
    author: String,
    timestamp: String,
}

/// text-only path changes
/// binary file changes represent line
/// changes with `-` which is of no use
/// to us
#[derive(Deserialize, Recap)]
#[recap(regex = r#"(?x)
    (?P<additions>\d+)
    \s+
    (?P<deletions>\d+)
    \s+
    (?P<path>\S+)
  "#)]
struct Path {
    additions: usize,
    deletions: usize,
    path: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Category {
    Test,
    Default,
}

#[derive(Debug, Serialize)]
struct Change {
    repo: String,
    sha: String,
    author: String,
    timestamp: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ext: Option<String>,
    category: Category,
    additions: usize,
    deletions: usize,
}

impl Change {
    fn categorize(path: &str) -> Category {
        if path.contains("test") {
            Category::Test
        } else {
            Category::Default
        }
    }
}

impl Into<Change> for (String, Header, Path) {
    fn into(self: (String, Header, Path)) -> Change {
        let (
            repo,
            Header {
                sha,
                author,
                timestamp,
            },
            Path {
                additions,
                deletions,
                path,
            },
        ) = self;
        let category = Change::categorize(&path);
        let ext = StdPath::new(&path)
            .extension()
            .and_then(OsStr::to_str)
            .map(|s| s.into());
        Change {
            repo,
            sha,
            author,
            timestamp,
            path,
            category,
            ext,
            additions,
            deletions,
        }
    }
}

enum State {
    Reset,
    Next(Header),
    Emit(Header, Path),
}

trait Emitter {
    fn emit(
        &mut self,
        line: Change,
    ) -> Result<(), Box<dyn Error>>;
}

struct Stdout;

impl Emitter for Stdout {
    fn emit(
        &mut self,
        line: Change,
    ) -> Result<(), Box<dyn Error>> {
        println!("{}", serde_json::to_string(&line)?);
        Ok(())
    }
}

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "git-linecat",
    about = "A tool for transforming and categorizing git log data"
)]
struct Options {
    #[structopt(short = "r", long = "repository", help = "Repository name")]
    repository: String,
    #[structopt(
        short = "l",
        long = "logs",
        help = "Path to git log output. use `-` to read from stdin",
        default_value = "-"
    )]
    logs: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let Options { repository, logs } = Options::from_args();
    match &logs[..] {
        "-" => run(
            repository,
            &mut stdin().lock().lines().filter_map(Result::ok),
            &mut Stdout,
        ),
        _ => run(
            repository,
            &mut BufReader::new(&File::open(logs)?)
                .lines()
                .filter_map(Result::ok),
            &mut Stdout,
        ),
    }
}

fn run<L, E>(
    repository: String,
    lines: &mut L,
    emitter: &mut E,
) -> Result<(), Box<dyn Error>>
where
    L: Iterator<Item = String>,
    E: Emitter,
{
    lines
        .try_fold(State::Reset, |state, line| {
            Ok(match state {
                State::Reset => State::Next(line.parse()?),
                State::Next(header) => {
                    if line.is_empty() {
                        State::Reset
                    } else if line.starts_with('-') {
                        // binary file
                        State::Next(header)
                    } else {
                        // we expect a path, but some commits may be empty (no path) so we must be flexible
                        match line.parse::<Path>() {
                            Ok(path) => State::Emit(header, path),
                            _ => State::Next(line.parse()?),
                        }
                    }
                }
                State::Emit(header, diff) => {
                    emitter.emit((repository.clone(), header.clone(), diff).into())?;
                    if line.is_empty() {
                        State::Reset
                    } else {
                        State::Next(header)
                    }
                }
            })
        })
        .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header_line_parses() -> Result<(), Box<dyn Error>> {
        let _: Header = r#""61708727af02089cef4a72c6a532ddf332111b14","luna@moon.com","2019-08-08 18:03:38 -0400""#.parse()?;
        Ok(())
    }

    #[test]
    fn path_line_parses() -> Result<(), Box<dyn Error>> {
        let _: Path = r#"6       3       foo/bar/baz.rs"#.parse()?;
        Ok(())
    }

    #[test]
    fn paths_with_test_are_categorized() {
        assert_eq!(Change::categorize("foo/test/bar.txt"), Category::Test)
    }

    #[test]
    fn paths_without_test_are_categorized() {
        assert_eq!(Change::categorize("foo/bar/baz.txt"), Category::Default)
    }

    #[test]
    fn parses_lines() {
        #[derive(Default)]
        struct Counter {
            n: usize,
        }
        impl Emitter for Counter {
            fn emit(
                &mut self,
                _: Change,
            ) -> Result<(), Box<dyn Error>> {
                self.n += 1;
                Ok(())
            }
        }
        let mut counter = Counter::default();
        drop(run(
            "test".into(),
            &mut include_str!("../tests/data/git.log")
                .lines()
                .map(|l| l.to_string()),
            &mut counter,
        ));
        assert_eq!(1, counter.n);
    }
}
