# cdup

`cd` up to several level from current working dirctory.


## Installation

1. Adapt the code to your need (optional, see below)
2. Source `cdup.sh`.

### Adapt the code to your need

At the beginning of `up` function definition in `cdup.sh`:

- `up` requires local environment variable `up_backend` to point to the
  `cdup.py` or its compiled `pyc` file (to reduce setup overhead). One may
  compile `cdup.py` one his/her machine and set `up_backend` to the compiled
  file.
- `up` requires local environment variable `up_pythonbin` to point to the
  python executable to use to run `cdup.py`. Python 2.7-3.x are supported.


## Usage

> Copied from `up -h`

```plain
Usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory will be
                            printed regardless of its existence
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one `cd' action in total

UPWARD_RULE

    Can be one of:

        <Nothing>           Same as `cd ..'
        -n [NUM_LEVELS], -NUM_LEVELS
                            Same as `cd ..' NUM_LEVELS time but there will be
                            only one `cd' action in total. In the first form,
                            NUM_LEVELS is default to 1 if not specified. In
                            the second form, if NUM_LEVELS does not start with
                            `n' (in which case it falls back to the first
                            form) and contains non-digit characters, or if
                            NUM_LEVELS is empty, the entire `-NUM_LEVELS'
                            will be interpreted as `NAME' (see below)
        [-r] NAME           Go upwards to the nearest directory named NAME.
                            The optional `-r' disambiguates conflicts with
                            the `/PATTERN/' rule below when NAME starts with
                            a slash (`/')
        /PATTERN/           Go upwards to the nearest directory matching the
                            python-style globbing pattern PATTERN. Be sure to
                            add quote around PATTERN to avoid unnecessary
                            shell expansion
        -E REGEX            Go upwards to the nearest directory matching the
                            python REGEX

The order of OPTIONS and UPWARD_RULE does not matter, and can be interleaved.
The optional `--' marks the beginning of UPWARD_RULE. Short options cannot
be merged together. Option with argument can be merged together. No
UPWARD_RULE other than `-n [NUM_LEVELS], -NUM_LEVELS' is allowed to reach
the root directory (`/').

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

	/home/user/directory1/directory2:$ up user
	/home/user:$ cd -
	/home/user/directory1/directory2:$ 

	/home/user/directory1/directory2:$ up '/d*1/'
	/home/user/directory1:$ cd -
	/home/user/directory1/directory2:$ 

# Additional script using [`fzf`](https://github.com/junegunn/fzf)

`updownfzf.sh` gives function `ud` that let the user first goes up and then goes down by selecting in `fzf` menu.


## Similar projects

I just realized that there were already a number of projects addressing this use case.
I wouldn't spend time on this should I be aware of it earlier!

- https://github.com/westonruter/misc-cli-tools/blob/master/cdup
