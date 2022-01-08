use glob::Pattern;
use regex::Regex;
use std::path::PathBuf;
use std::process;

const ERROR_ARGS: i32 = 2;
const ERROR_NOMATCH: i32 = 4;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("up: illegal command line arguments");
        process::exit(ERROR_ARGS);
    }

    let mut fromdir = PathBuf::from(&args[1]); // assuming absolute directory
    let rule_type = &args[2]; // assuming one of ["n", "raw", "glob", "regex"]
    let rule_value = &args[3]; // assuming parseable as int if rule_type == "n"
    let subsequent_dir;
    if args[4] == "" {
        subsequent_dir = None;
    } else {
        subsequent_dir = Some(&args[4]);
    }

    if rule_type == "n" {
        let n: usize = rule_value.parse().unwrap();
        upward_atmost(&mut fromdir, n);
    } else if rule_type == "raw" {
        let found = raw_search_upward(&mut fromdir, &rule_value);
        if !found {
            eprintln!("up: no match");
            process::exit(ERROR_NOMATCH);
        }
    } else if rule_type == "glob" {
        let pattern = match Pattern::new(rule_value) {
            Ok(pat) => pat,
            Err(_) => {
                eprintln!("up: invalid glob pattern");
                process::exit(ERROR_ARGS);
            }
        };
        match glob_search_upward(&mut fromdir, &pattern) {
            Ok(found) => {
                if !found {
                    eprintln!("up: no match");
                    process::exit(ERROR_NOMATCH);
                }
            }
            Err(msg) => {
                eprintln!("up: {}", msg);
                process::exit(ERROR_ARGS);
            }
        }
    } else if rule_type == "regex" {
        let pattern = match Regex::new(rule_value) {
            Ok(pat) => pat,
            Err(_) => {
                eprintln!("up: invalid regex pattern");
                process::exit(ERROR_ARGS);
            }
        };
        match regex_search_upward(&mut fromdir, &pattern) {
            Ok(found) => {
                if !found {
                    eprintln!("up: no match");
                    process::exit(ERROR_NOMATCH);
                }
            }
            Err(msg) => {
                eprintln!("up: {}", msg);
                process::exit(ERROR_ARGS);
            }
        }
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

fn upward_atmost(fromdir: &mut PathBuf, n: usize) {
    let mut n = n;
    while n > 0 {
        if !fromdir.pop() {
            break;
        }
        n -= 1;
    }
}

fn raw_search_upward(fromdir: &mut PathBuf, name: &str) -> bool {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => {
                if basename == name {
                    return true;
                }
            }
        }
    }

    false
}

fn glob_search_upward(
    fromdir: &mut PathBuf,
    pattern: &Pattern,
) -> Result<bool, String> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => match basename.to_str() {
                None => {
                    return Err(format!("invalid Unicode in \"{:?}\"", fromdir))
                }
                Some(basename) => {
                    if pattern.matches(basename) {
                        return Ok(true);
                    }
                }
            },
        }
    }

    Ok(false)
}

fn regex_search_upward(
    fromdir: &mut PathBuf,
    pattern: &Regex,
) -> Result<bool, String> {
    loop {
        if !fromdir.pop() {
            break;
        }
        match fromdir.file_name() {
            None => break,
            Some(basename) => match basename.to_str() {
                None => {
                    return Err(format!("invalid Unicode in \"{:?}\"", fromdir))
                }
                Some(basename) => {
                    if pattern.is_match(basename) {
                        return Ok(true);
                    }
                }
            },
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_search_upward() {
        let mut path = PathBuf::from("/hello/world");
        assert_eq!(raw_search_upward(&mut path, "world"), false);

        let mut path = PathBuf::from("/hello/world");
        assert_eq!(raw_search_upward(&mut path, "hell"), false);

        let mut path = PathBuf::from("/hello/world");
        assert_eq!(raw_search_upward(&mut path, ""), false);

        let mut path = PathBuf::from("/hello/world/again");
        assert_eq!(raw_search_upward(&mut path, "world"), true);
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        assert_eq!(raw_search_upward(&mut path, "hello"), true);
        assert_eq!(path, PathBuf::from("/hello"));

        let mut path = PathBuf::from("/你好/世界/再一次");
        assert_eq!(raw_search_upward(&mut path, "世界"), true);
        assert_eq!(path, PathBuf::from("/你好/世界"));
    }

    #[test]
    fn test_glob_search_upward() {
        let mut path = PathBuf::from("/hello/world");
        let pat = Pattern::new("").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), false);

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Pattern::new("world").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Pattern::new("wo*").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/你好/世界/再一次");
        let pat = Pattern::new("*界").unwrap();
        assert_eq!(glob_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/你好/世界"));
    }

    #[test]
    fn test_regex_search_upward() {
        // Note this test case
        let mut path = PathBuf::from("/hello/world");
        let pat = Regex::new("").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/hello"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("world").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("wo.*").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/hello/world"));

        let mut path = PathBuf::from("/hello/world/again");
        let pat = Regex::new("ll").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/hello"));

        let mut path = PathBuf::from("/你好/世界/再一次");
        let pat = Regex::new("界").unwrap();
        assert_eq!(regex_search_upward(&mut path, &pat).unwrap(), true);
        assert_eq!(path, PathBuf::from("/你好/世界"));
    }
}
