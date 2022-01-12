use cdup_rs_backend as up;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cli = up::parse_args(&args);
    let cwd = cli.fromdir.clone();

    match cli.rule_type {
        up::RuleType::N => up::handle_n(&mut cli.fromdir, cli.rule_value),
        up::RuleType::Raw => up::handle_raw(&mut cli.fromdir, cli.rule_value),
        up::RuleType::Glob => up::handle_glob(&mut cli.fromdir, cli.rule_value),
        up::RuleType::Regex => {
            up::handle_regex(&mut cli.fromdir, cli.rule_value)
        }
    }

    if let Some(d) = cli.subsequent_dir {
        cli.fromdir.push(d);
    }

    match cli.fromdir.to_str() {
        None => {
            eprintln!("up: invalid Unicode in {:?}", cli.fromdir);
            process::exit(up::ERROR_ARGS);
        }
        Some(s) => {
            if cli.list {
                println!("echo \"{}\"", s);
            } else if cli.fromdir != cwd {
                println!("cd \"{}\"", s);
            }
        }
    }
}
