use glob::Pattern;
use regex::Regex;
use std::path::PathBuf;
use std::process;

pub const ERROR_ARGS: i32 = 2;
pub const ERROR_NOMATCH: i32 = 4;

#[derive(Debug)]
enum ErrorType {
    NoMatch,
    InvalidUnicode,
}

pub enum RuleType {
    N,
    Raw,
    Glob,
    Regex,
}

pub struct Cli<'a> {
    pub fromdir: PathBuf,
    pub rule_type: RuleType,
    pub rule_value: &'a str,
    pub subsequent_dir: Option<&'a str>,
    pub list: bool,
}

pub fn parse_args(args: &Vec<String>) -> Cli {
    let fromdir = PathBuf::from(args[1].clone());
    let mut subsequent_dir: Option<&str> = None;
    let mut list = false;
    let mut rule_type: Option<RuleType> = None;
    let mut rule_value: Option<&str> = None;

    let mut rule_begin = false;
    // set to true if one of the OPTIONS or `--' has been parsed in current
    // loop
    let mut option_parsed: bool;
    let n = args.len();
    let mut cur: usize = 2;

    while cur < n {
        option_parsed = false;
        if !rule_begin {
            if args[cur] == "-h" || args[cur] == "--help" {
                eprintln!(
                    "\
usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one `cd' action in total
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory will be
                            printed regardless of its existence

UPWARD_RULE

    Can be one of:

        <Nothing>           Same as `cd ..'
        -NUM_LEVELS         Same as `cd ..' NUM_LEVELS time but there will be
                            only one `cd' action in total. If NUM_LEVELS
                            contains non-digit characters, or if NUM_LEVELS
                            is empty, the entire `-NUM_LEVELS' will be
                            interpreted as `NAME' (see below). If NUM_LEVELS
                            is `0', nothing will be done
        [-r] NAME           Go upwards to the nearest directory named NAME.
                            The optional `-r' disambiguates cases when NAME
                            starts with `-'
        -g PATTERN          Go upwards to the nearest directory matching the
                            python-style globbing pattern PATTERN. Be sure to
                            add quote around PATTERN to avoid unnecessary
                            shell expansion
        -E REGEX            Go upwards to the nearest directory matching the
                            python REGEX

The order of OPTIONS and UPWARD_RULE does not matter, and can be interleaved.
The optional `--' marks the beginning of UPWARD_RULE. Short options cannot
be merged together. Option with argument can be merged together. No
UPWARD_RULE other than `-NUM_LEVELS' is allowed to reach the root
directory (`/').

Error code

    0                       Successs
    1                       cd error (`No such file or directory'). This
                            error is most often triggered by `-s' option as
                            unable to target directory upward will lead to
                            return code 4
    2                       Cmd argument error
    4                       Cannot find the target directory upward"
                );
                process::exit(0);
            }
            if args[cur].starts_with("-s") {
                if args[cur].len() > 2 {
                    subsequent_dir = Some(&args[cur][2..]);
                } else if cur + 1 < n {
                    subsequent_dir = Some(&args[cur + 1]);
                    cur += 1;
                } else {
                    eprintln!("DIR missing");
                    process::exit(ERROR_ARGS);
                }
                option_parsed = true;
            } else if args[cur] == "-l" {
                list = true;
                option_parsed = true;
            } else if args[cur] == "--" {
                rule_begin = true;
                option_parsed = true;
            }
        }
        if !option_parsed {
            if let Some(_) = rule_type {
                eprintln!("UPWARD_RULE has already been specified");
                process::exit(ERROR_ARGS);
            }
            if args[cur].starts_with("-r") {
                rule_type = Some(RuleType::Raw);
                if args[cur].len() > 2 {
                    rule_value = Some(&args[cur][2..]);
                } else if cur + 1 < n {
                    rule_value = Some(&args[cur + 1]);
                    cur += 1;
                } else {
                    eprintln!("NAME missing");
                    process::exit(ERROR_ARGS);
                }
            } else if args[cur].starts_with("-g") {
                rule_type = Some(RuleType::Glob);
                if args[cur].len() > 2 {
                    rule_value = Some(&args[cur][2..]);
                } else if cur + 1 < n {
                    rule_value = Some(&args[cur + 1]);
                    cur += 1;
                } else {
                    eprintln!("PATTERN missing");
                    process::exit(ERROR_ARGS);
                }
            } else if args[cur].starts_with("-E") {
                rule_type = Some(RuleType::Regex);
                if args[cur].len() > 2 {
                    rule_value = Some(&args[cur][2..]);
                } else if cur + 1 < n {
                    rule_value = Some(&args[cur + 1]);
                    cur += 1;
                } else {
                    eprintln!("REGEX missing");
                    process::exit(ERROR_ARGS);
                }
            } else if args[cur].starts_with("-") {
                if args[cur].len() > 1
                    && args[cur][1..].chars().all(|c| c.is_ascii_digit())
                {
                    rule_type = Some(RuleType::N);
                    rule_value = Some(&args[cur][1..]);
                } else {
                    rule_type = Some(RuleType::Raw);
                    rule_value = Some(&args[cur]);
                }
            } else {
                rule_type = Some(RuleType::Raw);
                rule_value = Some(&args[cur]);
            }
        }
        cur += 1;
    }

    if let None = rule_type {
        rule_type = Some(RuleType::N);
        rule_value = Some("1");
    }

    Cli {
        fromdir,
        rule_type: rule_type.unwrap(),
        rule_value: rule_value.unwrap(),
        subsequent_dir,
        list,
    }
}

pub fn handle_n(fromdir: &mut PathBuf, rule_value: &str) {
    upward_atmost(fromdir, rule_value.parse::<usize>().unwrap());
}

pub fn handle_raw(fromdir: &mut PathBuf, rule_value: &str) {
    match raw_search_upward(fromdir, rule_value) {
        Ok(()) => (),
        Err(ErrorType::NoMatch) => {
            eprintln!("up: no match");
            process::exit(ERROR_NOMATCH);
        }
        Err(ErrorType::InvalidUnicode) => {
            // this arm shouldn't be reached since raw_search_upward should not
            // return InvalidUnicode
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
    }
}

pub fn handle_glob(fromdir: &mut PathBuf, rule_value: &str) {
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
            eprintln!("up: invalid Unicode in {:?}", fromdir);
            process::exit(ERROR_ARGS);
        }
    }
}

pub fn handle_regex(fromdir: &mut PathBuf, rule_value: &str) {
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
            eprintln!("up: invalid Unicode in {:?}", fromdir);
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
    fn test_parse_args() {
        let args = vec![String::from("bin"), String::from("/hello/world")];
        let cli = parse_args(&args);
        assert_eq!(cli.fromdir, PathBuf::from("/hello/world"));
        assert!(matches!(cli.subsequent_dir, None));
        assert_eq!(cli.list, false);
        assert!(matches!(cli.rule_type, RuleType::N));
        assert_eq!(cli.rule_value, "1");

        let args = vec![
            String::from("bin"),
            String::from("/hello/world"),
            String::from("-gP*s"),
        ];
        let cli = parse_args(&args);
        assert_eq!(cli.fromdir, PathBuf::from("/hello/world"));
        assert!(matches!(cli.subsequent_dir, None));
        assert_eq!(cli.list, false);
        assert!(matches!(cli.rule_type, RuleType::Glob));
        assert_eq!(cli.rule_value, "P*s");

        let args = vec![
            String::from("bin"),
            String::from("/hello/world"),
            String::from("-E"),
            String::from("P"),
            String::from("-sagain"),
            String::from("-l"),
        ];
        let cli = parse_args(&args);
        assert_eq!(cli.fromdir, PathBuf::from("/hello/world"));
        assert!(matches!(cli.subsequent_dir, Some("again")));
        assert_eq!(cli.list, true);
        assert!(matches!(cli.rule_type, RuleType::Regex));
        assert_eq!(cli.rule_value, "P");

        let args = vec![
            String::from("bin"),
            String::from("/hello/world"),
            String::from("--xxx"),
        ];
        let cli = parse_args(&args);
        assert_eq!(cli.fromdir, PathBuf::from("/hello/world"));
        assert!(matches!(cli.subsequent_dir, None));
        assert_eq!(cli.list, false);
        assert!(matches!(cli.rule_type, RuleType::Raw));
        assert_eq!(cli.rule_value, "--xxx");

        let args = vec![
            String::from("bin"),
            String::from("/hello/world"),
            String::from("--"),
            String::from("-s"),
        ];
        let cli = parse_args(&args);
        assert_eq!(cli.fromdir, PathBuf::from("/hello/world"));
        assert!(matches!(cli.subsequent_dir, None));
        assert_eq!(cli.list, false);
        assert!(matches!(cli.rule_type, RuleType::Raw));
        assert_eq!(cli.rule_value, "-s");

        let args = vec![
            String::from("bin"),
            String::from("/hello/world"),
            String::from("-r-sagain"),
            String::from("-s"),
            String::from("from"),
        ];
        let cli = parse_args(&args);
        assert_eq!(cli.fromdir, PathBuf::from("/hello/world"));
        assert!(matches!(cli.subsequent_dir, Some("from")));
        assert_eq!(cli.list, false);
        assert!(matches!(cli.rule_type, RuleType::Raw));
        assert_eq!(cli.rule_value, "-sagain");
    }

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