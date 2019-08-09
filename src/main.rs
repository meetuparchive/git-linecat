//! a tool for transforming and categorizing git log output
//! expects input in the format of the output of
//! `git log --pretty=format:'"%H","%ae","%ai"' --numstat --no-merges`
use recap::Recap;
use serde::{Deserialize, Serialize};
use std::{
    convert::identity,
    error::Error,
    io::{stdin, BufRead},
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
struct Line {
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

impl Line {
    fn categorize(path: &str) -> Category {
        if path.contains("test") {
            Category::Test
        } else {
            Category::Default
        }
    }
}

impl Into<Line> for (String, Header, Path) {
    fn into(self: (String, Header, Path)) -> Line {
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
        let category = Line::categorize(&path);
        let ext = StdPath::new(&path)
            .extension()
            .map(std::ffi::OsStr::to_str)
            .and_then(identity)
            .map(|s| s.into());
        Line {
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
    fn emit(line: Line) -> Result<(), Box<dyn Error>>;
}

struct Stdout;

impl Emitter for Stdout {
    fn emit(line: Line) -> Result<(), Box<dyn Error>> {
        println!("{}", serde_json::to_string(&line)?);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    stdin()
        .lock()
        .lines()
        .filter_map(Result::ok)
        .try_fold(State::Reset, |state, line| {
            Ok(match state {
                State::Reset => State::Next(line.parse()?),
                State::Next(header) => {
                    if line.is_empty() {
                        State::Reset
                    } else {
                        State::Emit(header, line.parse()?)
                    }
                }
                State::Emit(header, diff) => {
                    let next = if line.is_empty() {
                        State::Reset
                    } else {
                        State::Next(header.clone())
                    };
                    Stdout::emit(("".to_string(), header, diff).into())?;
                    next
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
        assert_eq!(Line::categorize("foo/test/bar.txt"), Category::Test)
    }

    #[test]
    fn paths_without_test_are_categorized() {
        assert_eq!(Line::categorize("foo/bar/baz.txt"), Category::Default)
    }
}
