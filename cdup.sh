#!/bin/bash
# also exposed to /bin/zsh

__up_help() {
	cat << EOF
usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one \`cd' action in total;
                            (quoted) recursive globbing is supported in DIR
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory will be
                            printed regardless of its existence

UPWARD_RULE

    Can be one of:

        <Nothing>           Same as \`cd ..'
        -NUM_LEVELS         Same as \`cd ..' NUM_LEVELS time but there will be
                            only one \`cd' action in total. If NUM_LEVELS
                            contains non-digit characters, or if NUM_LEVELS
                            is empty, the entire \`-NUM_LEVELS' will be
                            interpreted as \`NAME' (see below). If NUM_LEVELS
                            is \`0', nothing will be done
        [-r] NAME           Go upwards to the nearest directory named NAME.
                            The optional \`-r' disambiguates cases when NAME
                            starts with \`-'
        -g PATTERN          Go upwards to the nearest directory matching the
                            python-style globbing pattern PATTERN. Be sure to
                            add quote around PATTERN to avoid unnecessary
                            shell expansion
        -E REGEX            Go upwards to the nearest directory matching the
                            python REGEX
        -G                  Go upwards to the nearest Git root directory

The order of OPTIONS and UPWARD_RULE does not matter, and can be interleaved.
The optional \`--' marks the beginning of UPWARD_RULE. Short options cannot
be merged together. Option with argument can be merged together. No
UPWARD_RULE other than \`-NUM_LEVELS' is allowed to reach the root
directory (\`/').

Error code

    0                       Successs
    1                       cd error (\`No such file or directory'). This
                            error is most often triggered by \`-s' option as
                            unable to target directory upward will lead to
                            return code 4
    2                       Cmd argument error
    4                       Cannot find the target directory
EOF
}

up() {
	# compatible with bash then zsh;
	# reference: https://stackoverflow.com/a/54755784/7881370
	local up_basedir="$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")"
	local up_py_backend=
	local up_pythonbin=
	local up_rs_backend="$up_basedir/rs_backend"
	if [ ! -f "$up_rs_backend" ]; then
		echo "rs_backend not built yet; downgrading to Python backend" >&2
		up_rs_backend=
		up_py_backend="$up_basedir/cdup.py"
		up_pythonbin=python3
	fi

	local listonly=
	local subdir=
	local rule_value=
	local rule_type=
	local todir

	# cmd parsing related
	local rule_begin=
	# set to 1 if one of the OPTIONS or `--' has been parsed in current loop
	local option_parsed

	# parse arguments
	while [ -n "$1" ]; do
		option_parsed=
		if [ -z "$rule_begin" ]; then
			case "$1" in
				-h|--help)
					__up_help
					return 0
					;;
				-s*)
					if [ -n "${1:2}" ]; then
						subdir="${1:2}"
					elif [ -n "$2" ]; then
						subdir="$2"
						shift
					else
						echo "up: DIR missing" >&2
						return 2
					fi
					option_parsed=1
					;;
				-l)
					listonly=1
					option_parsed=1
					;;
				--)
					rule_begin=1
					option_parsed=1
					;;
			esac
		fi
		if [ -z "$option_parsed" ]; then
			if [ -n "$rule_type" ] || [ -n "$rule_value" ]; then
				echo "up: UPWARD_RULE has already been specified" >&2
				return 2
			fi
			case "$1" in
				-r*)
					rule_type="raw"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						echo "up: NAME missing" >&2
						return 2
					fi
					;;
				-g*)
					rule_type="glob"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						echo "up: PATTERN missing" >&2
						return 2
					fi
					;;
				-E*)
					rule_type="regex"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						echo "up: REGEX missing" >&2
						return 2
					fi
					;;
				-G)
					rule_type="git"
					# just a placeholder
					rule_value=1
					;;
				-)
					rule_type="raw"
					rule_value="-"
					;;
				-*)
					case "${1:1}" in
						*[!0-9]*)
							rule_type="raw"
							rule_value="$1"
							;;
						*)
							rule_type="n"
							rule_value="${1:1}"
							;;
					esac
					;;
				*)
					rule_type="raw"
					rule_value="$1"
					;;
			esac
		fi
		shift
	done
	if [ -z "$rule_type" ] || [ -z "$rule_value" ]; then
		rule_type="n"
		rule_value=1
	fi

	local cwd="$(pwd)"
	if [ -n "$up_rs_backend" ]; then
		todir="$("$up_rs_backend" "$cwd" "$rule_type" "$rule_value" "$subdir")"
	else
		todir="$("$up_pythonbin" "$up_py_backend" "$cwd" "$rule_type" "$rule_value" "$subdir")"
	fi
	local retcode="$?"
	# retcode = 8 means SAMEDIR
	if [ "$retcode" != 0 ] && [ "$retcode" != 8 ]; then
		return "$retcode"
	fi
	if [ -n "$listonly" ]; then
		echo "$todir"
	elif [ "$retcode" != 8 ]; then
		# Mean to fail if cd fails
		# shellcheck disable=SC2164
		cd "$todir"
	fi
}
