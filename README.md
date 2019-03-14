# cdup

`cd` up to several level from current working dirctory.


## Installation

Source `cdup.sh`.


## Usage

> Copied from `cd -h'

```plain
Usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory won't be
                            printed and will return 1 if it does not exist
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one `cd' action in total

UPWARD_RULE

    Can be one of:

        <Nothing>           Same as `cd ..'
        -n [NUM_LEVELS], -NUM_LEVELS
                            Same as `cd ..' NUM_LEVELS time but there will be
                            only one `cd' action in total. In the first form,
                            NUM_LEVELS is default to 1 if not specified
        [-r] NAME           Go upwards to the nearest directory named NAME.
                            The optional `-r' disambiguates conflicts with
                            the `/PATTERN/' rule below when NAME starts with
                            a slash (`/')
        /PATTERN/           Go upwards to the nearest directory matching the
                            bash-style globbing pattern PATTERN. Be sure to
                            add quote around PATTERN to avoid unnecessary
                            globbing at current working directory by shell
        -e REGEX            Go upwards to the nearest directory matching the
                            grep basic regex REGEX
        -E REGEX            Go upwards to the nearest directory matching the
                            grep extended regex REGEX

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
