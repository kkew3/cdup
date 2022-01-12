# cdup

`cd` up several levels from current working dirctory.


## Installation

1. `git clone` the repository to `directory_of_your_choice`:

2. Build the [Rust](https://www.rust-lang.org) backend

If not already installed `cargo`, [install it](https://doc.rust-lang.org/cargo/getting-started/installation.html).
And then,

```sh
cd "directory_of_your_choice/cdup_rs_backend"
cargo build --release
cd ..
ln -s directory_of_your_choice/cdup_rs_backend/target/release/cdup_rs_backend rs_backend
```

3. Source the absolute path of `cdup.sh`.

If using `bash`, everything is fine. If using `zsh`, remember to add `setopt SH_WORD_SPLIT` to `~/.zshenv` so that `-x COMMAND` option works (see below).


## Usage

> Copied from `up -h`

```plain
usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory will be
                            printed regardless of its existence
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one `cd' action in total
    -x COMMAND              Run COMMAND on the target directory; `-x cd' is
                            the default behavior; other examples include
                            `-x open' (on macOS) and `-x "ls -l"'

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
    4                       Cannot find the target directory upward
```

One difference from "`cd ..` for N times" is that `up` function addresses the `OLDPWD` better than `cd ..` multiple times.

Several use cases:

	/home/user/directory1/directory2:$ up
	/home/user/directory1:$ 

	/home/user/directory1/directory2:$ up -3
	/home:$ cd -
	/home/user/directory1/directory2:$

	/home/user/directory1/directory2:$ up user
	/home/user:$ cd -
	/home/user/directory1/directory2:$ 

	/home/user/directory1/directory2:$ up -g'd*1'
	/home/user/directory1:$ cd -
	/home/user/directory1/directory2:$ 

	/home/user/directory1/directory2:$ up -Ey1
	/home/user/directory1:$


## Profiling

- `cd ../../..`: 0.014s
- `up -3`: 0.026s

Using `up [-r] NAME`, `up -g PATTERN`, or `up -E REGEX` shouldn't induce more time than 0.005s compared with `up -NUM_LEVELS`.


## Similar projects

- https://github.com/westonruter/misc-cli-tools/blob/master/cdup
