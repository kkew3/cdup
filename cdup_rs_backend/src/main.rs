use glob::Pattern;
use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

const ERROR_ARGS: i32 = 2;
const ERROR_MATCH: i32 = 4;
const ERROR_SAMEDIR: i32 = 8;

#[derive(Debug)]
enum UpError {
    NoMatch,
    InvalidUnicode,
}

struct Cli {
    fromdir: PathBuf,
    rule_type: String,
    rule_value: String,
    subsequent_pattern: Option<String>,
}

fn main() {
    let mut cli = parse_args(&mut env::args());
    let cwd = cli.fromdir.clone();

    if cli.rule_type == "n" {
        handle_n(&mut cli.fromdir, &cli.rule_value);
    } else if cli.rule_type == "raw" {
        handle_raw(&mut cli.fromdir, &cli.rule_value);
    } else if cli.rule_type == "glob" {
        handle_glob(&mut cli.fromdir, &cli.rule_value);
    } else if cli.rule_type == "regex" {
        handle_regex(&mut cli.fromdir, &cli.rule_value);
    } else if cli.rule_type == "git" {
        handle_git(&mut cli.fromdir);
    } else {
        eprintln!("up: invalid rule type");
        process::exit(ERROR_ARGS);
    }

    if let Some(d) = cli.subsequent_pattern.as_ref() {
        glob_downward(&mut cli.fromdir, d);
    }

    match cli.fromdir.to_str() {
        None => {
            eprintln!("up: invalid Unicode in {:?}", cli.fromdir);
            process::exit(ERROR_ARGS);
        }
        Some(s) => print!("{}", s),
    }

    if cli.fromdir == cwd {
        process::exit(ERROR_SAMEDIR);
    }
}

fn parse_args(args_iter: &mut env::Args) -> Cli {
    let _bin = args_iter.next();
    // assuming absolute directory
    let fromdir = PathBuf::from(args_iter.next().unwrap());
    // assuming one of ["n", "raw", "glob", "regex", "git"]
    let rule_type = args_iter.next().unwrap();
    // assuming parseable as usize if rule_type == "n"
    let rule_value = args_iter.next().unwrap();
    let subsequent_pattern = args_iter.next().unwrap();
    let subsequent_pattern = match &subsequent_pattern[..] {
        "" => None,
        s => Some(String::from(s)),
    };

    Cli {
        fromdir,
        rule_type,
        rule_value,
        subsequent_pattern,
    }
}

fn handle_n(fromdir: &mut PathBuf, rule_value: &str) {
    upward_atmost(fromdir, rule_value.parse::<usize>().unwrap());
}

fn handle_raw(fromdir: &mut PathBuf, rule_value: &str) {
    match raw_search_upward(fromdir, rule_value) {
        Ok(()) => (),
        Err(UpError::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_MATCH);
        }
        Err(UpError::InvalidUnicode) => {
            // this arm shouldn't be reached since raw_search_upward should not
            // return InvalidUnicode
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
    }
}

fn handle_glob(fromdir: &mut PathBuf, rule_value: &str) {
    let pattern = match Pattern::new(rule_value) {
        Ok(pat) => pat,
        Err(_) => {
            eprintln!("up: invalid glob pattern");
            process::exit(ERROR_ARGS);
        }
    };
    match glob_search_upward(fromdir, &pattern) {
        Ok(()) => (),
        Err(UpError::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_MATCH);
        }
        Err(UpError::InvalidUnicode) => {
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
    }
}

fn handle_regex(fromdir: &mut PathBuf, rule_value: &str) {
    let pattern = match Regex::new(rule_value) {
        Ok(pat) => pat,
        Err(_) => {
            eprintln!("up: invalid regex pattern");
            process::exit(ERROR_ARGS);
        }
    };
    match regex_search_upward(fromdir, &pattern) {
        Ok(()) => (),
        Err(UpError::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_MATCH);
        }
        Err(UpError::InvalidUnicode) => {
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
    }
}

fn handle_git(fromdir: &mut PathBuf) {
    match git_search_upward(fromdir) {
        Ok(()) => (),
        Err(UpError::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_MATCH);
        }
        Err(UpError::InvalidUnicode) => {
            // this arm shouldn't be reached since git_search_upward should not
            // return InvalidUnicode
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
    }
}

fn glob_downward(fromdir: &mut PathBuf, subsequent_pattern: &str) {
    if let Err(_) = env::set_current_dir(&fromdir) {
        eprintln!(
            "up: failed to prepare to go downward by cd'ing first to {:?}",
            fromdir
        );
        process::exit(ERROR_ARGS);
    }
    let paths = match glob::glob(subsequent_pattern) {
        Ok(paths) => paths,
        Err(_) => {
            eprintln!("up: invalid glob pattern in DIR");
            process::exit(ERROR_ARGS);
        }
    };
    let mut instantiated_subsequent = Vec::new();
    for entry in paths {
        match entry {
            Ok(path) => {
                let is_dir = match fs::metadata(&path) {
                    Ok(meta) => meta.is_dir(),
                    Err(_) => {
                        // According to the doc, Err might be caused by:
                        // "The user lacks permissions to perform metadata
                        // call on path.", or "`path` does not exist."
                        // Since `path` must exist here, the Err should
                        // be caused by the former.
                        //
                        // Include this `path`
                        // into the matched list since it could be a
                        // directory.
                        true
                    }
                };
                if is_dir {
                    instantiated_subsequent.push(path);
                }
            }
            Err(e) => {
                eprintln!(
                    "up: unreachable path occurred {:?}; skipped",
                    e.path()
                )
            }
        }
        if instantiated_subsequent.len() > 1 {
            break;
        }
    }
    match instantiated_subsequent.len() {
        0 => {
            eprintln!("up: no match for DIR");
            process::exit(ERROR_MATCH);
        }
        1 => fromdir.push(&instantiated_subsequent[0]),
        _ => {
            eprintln!("up: multiple matches for DIR");
            process::exit(ERROR_MATCH);
        }
    }
}

fn upward_atmost(fromdir: &mut PathBuf, mut n: usize) {
    while n > 0 {
        if !fromdir.pop() {
            break;
        }
        n -= 1;
    }
}

fn raw_search_upward(fromdir: &mut PathBuf, name: &str) -> Result<(), UpError> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => {
                if basename == name {
                    return Ok(());
                }
            }
        }
    }

    Err(UpError::NoMatch)
}

fn glob_search_upward(
    fromdir: &mut PathBuf,
    pattern: &Pattern,
) -> Result<(), UpError> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => match basename.to_str() {
                None => return Err(UpError::InvalidUnicode),
                Some(basename) => {
                    if pattern.matches(basename) {
                        return Ok(());
                    }
                }
            },
        }
    }

    Err(UpError::NoMatch)
}

fn regex_search_upward(
    fromdir: &mut PathBuf,
    pattern: &Regex,
) -> Result<(), UpError> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => match basename.to_str() {
                None => return Err(UpError::InvalidUnicode),
                Some(basename) => {
                    if pattern.is_match(basename) {
                        return Ok(());
                    }
                }
            },
        }
    }

    Err(UpError::NoMatch)
}

fn git_search_upward(
    fromdir: &mut PathBuf,
) -> Result<(), UpError> {
    loop {
        if !fromdir.pop() {
            break;
        }
        fromdir.push(".git");
        if fromdir.is_dir() {
            fromdir.pop();
            return Ok(());
        }
        fromdir.pop();
    }

    Err(UpError::NoMatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_search_upward() {
        let mut path = PathBuf::from("/hello/world");
        assert!(matches!(
            raw_search_upward(&mut path, "world").unwrap_err(),
            UpError::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world");
        assert!(matches!(
            raw_search_upward(&mut path, "hell").unwrap_err(),
            UpError::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world");
        assert!(matches!(
            raw_search_upward(&mut path, "").unwrap_err(),
            UpError::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world/again");
        assert_eq!(raw_search_upward(&mut path, "world").unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        assert_eq!(raw_search_upward(&mut path, "hello").unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello"));

        let mut path = PathBuf::from("/你好/世界/再一次");
        assert_eq!(raw_search_upward(&mut path, "世界").unwrap(), ());
        assert_eq!(path, PathBuf::from("/你好/世界"));
    }

    #[test]
    fn test_glob_search_upward() {
        let mut path = PathBuf::from("/hello/world");
        let pat = Pattern::new("").unwrap();
        assert!(matches!(
            glob_search_upward(&mut path, &pat).unwrap_err(),
            UpError::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Pattern::new("world").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Pattern::new("wo*").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/你好/世界/再一次");
        let pat = Pattern::new("*界").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/你好/世界"));
    }

    #[test]
    fn test_regex_search_upward() {
        // Note this test case
        let mut path = PathBuf::from("/hello/world");
        let pat = Regex::new("").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("^o").unwrap();
        assert!(matches!(
            regex_search_upward(&mut path, &pat).unwrap_err(),
            UpError::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("world").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("wo.*").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("ll").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/hello"));

        let mut path = PathBuf::from("/你好/世界/再一次");
        let pat = Regex::new("界").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), ());
        assert_eq!(path, PathBuf::from("/你好/世界"));
    }
}
