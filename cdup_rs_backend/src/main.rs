use glob::Pattern;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use std::process;

const ERROR_ARGS: i32 = 2;
const ERROR_NOMATCH: i32 = 4;

#[derive(Debug)]
enum ErrorType {
    NoMatch,
    InvalidUnicode,
}

fn main() {
    let mut args_iter = env::args();
    let _ = args_iter.next();
    // assuming absolute directory
    let mut fromdir = PathBuf::from(args_iter.next().unwrap());
    // assuming one of ["n", "raw", "glob", "regex"]
    let rule_type = args_iter.next().unwrap();
    // assuming parseable as usize if rule_type == "n"
    let rule_value = args_iter.next().unwrap();
    let subsequent_dir = args_iter.next().unwrap();
    let subsequent_dir = match subsequent_dir.as_str() {
        "" => None,
        s => Some(s),
    };

    if rule_type == "n" {
        handle_n(&mut fromdir, &rule_value);
    } else if rule_type == "raw" {
        handle_raw(&mut fromdir, &rule_value);
    } else if rule_type == "glob" {
        handle_glob(&mut fromdir, &rule_value);
    } else if rule_type == "regex" {
        handle_regex(&mut fromdir, &rule_value);
    } else {
        eprintln!("up: invalid rule type");
        process::exit(ERROR_ARGS);
    }

    if let Some(d) = subsequent_dir {
        fromdir.push(d);
    }
    match fromdir.to_str() {
        None => {
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
        Some(s) => print!("{}", s),
    }
}

fn handle_n(fromdir: &mut PathBuf, rule_value: &str) {
    upward_atmost(fromdir, rule_value.parse::<usize>().unwrap());
}

fn handle_raw(fromdir: &mut PathBuf, rule_value: &str) {
    match raw_search_upward(fromdir, rule_value) {
        Ok(()) => (),
        Err(ErrorType::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_NOMATCH);
        }
        Err(ErrorType::InvalidUnicode) => {
            // this arm shouldn't be reached since raw_search_upward should not
            // return InvalidUnicode
            eprintln!("up: invalid Unicode in \"{:?}\"", fromdir);
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
        Err(ErrorType::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_NOMATCH);
        }
        Err(ErrorType::InvalidUnicode) => {
            eprintln!("up: invalid Unicode in \"{:?}\"", fromdir);
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
        Err(ErrorType::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_NOMATCH);
        }
        Err(ErrorType::InvalidUnicode) => {
            eprintln!("up: invalid Unicode in \"{:?}\"", fromdir);
            process::exit(ERROR_ARGS);
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

fn raw_search_upward(
    fromdir: &mut PathBuf,
    name: &str,
) -> Result<(), ErrorType> {
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

    Err(ErrorType::NoMatch)
}

fn glob_search_upward(
    fromdir: &mut PathBuf,
    pattern: &Pattern,
) -> Result<(), ErrorType> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => match basename.to_str() {
                None => return Err(ErrorType::InvalidUnicode),
                Some(basename) => {
                    if pattern.matches(basename) {
                        return Ok(());
                    }
                }
            },
        }
    }

    Err(ErrorType::NoMatch)
}

fn regex_search_upward(
    fromdir: &mut PathBuf,
    pattern: &Regex,
) -> Result<(), ErrorType> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => match basename.to_str() {
                None => return Err(ErrorType::InvalidUnicode),
                Some(basename) => {
                    if pattern.is_match(basename) {
                        return Ok(());
                    }
                }
            },
        }
    }

    Err(ErrorType::NoMatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_search_upward() {
        let mut path = PathBuf::from("/hello/world");
        assert!(matches!(
            raw_search_upward(&mut path, "world").unwrap_err(),
            ErrorType::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world");
        assert!(matches!(
            raw_search_upward(&mut path, "hell").unwrap_err(),
            ErrorType::NoMatch
        ));

        let mut path = PathBuf::from("/hello/world");
        assert!(matches!(
            raw_search_upward(&mut path, "").unwrap_err(),
            ErrorType::NoMatch
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
            ErrorType::NoMatch
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
            ErrorType::NoMatch
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
