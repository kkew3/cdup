#!/bin/bash

__up_help() {
	cat << EOF
Usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory won't be
                            printed and will return 1 if it does not exist
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one \`cd' action in total

UPWARD_RULE

    Can be one of:

        <Nothing>           Same as \`cd ..'
        -n [NUM_LEVELS], -NUM_LEVELS
                            Same as \`cd ..' NUM_LEVELS time but there will be
                            only one \`cd' action in total. In the first form,
                            NUM_LEVELS is default to 1 if not specified
        [-r] NAME           Go upwards to the nearest directory named NAME.
                            The optional \`-r' disambiguates conflicts with
                            the \`/PATTERN/' rule below when NAME starts with
                            a slash (\`/')
        /PATTERN/           Go upwards to the nearest directory matching the
                            bash-style globbing pattern PATTERN. Be sure to
                            add quote around PATTERN to avoid unnecessary
                            globbing at current working directory by shell
        -e REGEX            Go upwards to the nearest directory matching the
                            grep basic regex REGEX
        -E REGEX            Go upwards to the nearest directory matching the
                            grep extended regex REGEX

The order of OPTIONS and UPWARD_RULE does not matter, and can be interleaved.
The optional \`--' marks the beginning of UPWARD_RULE. Short options cannot
be merged together. Option with argument can be merged together. No
UPWARD_RULE other than \`-n [NUM_LEVELS], -NUM_LEVELS' is allowed to reach
the root directory (\`/').

Error code

    0                       Successs
    1                       cd error (\`No such file or directory'). This
                            error is most often triggered by \`-s' option as
                            unable to target directory upward will lead to
                            return code 4
    2                       Cmd argument error
    4                       Cannot find the target directory upward
EOF
}

up() {
	local listonly=false
	local subdir=
	local rule_value=
	local rule_type=
	local todir

	# cmd parsing related
	local rule_begin=false
	local option_parsed

	# parse arguments
	while [ -n "$1" ]; do
		option_parsed=true
		if ! $rule_begin; then
			case "$1" in
				-h|--help)
					__up_help
					return 0
					;;
				-l)
					listonly=true
					;;
				-s*)
					if [ -n "${1:2}" ]; then
						subdir="${1:2}"
					elif [ -n "$2" ]; then
						subdir="$2"
						shift
					else
						echo "DIR not specified" >&2
						return 2
					fi
					;;
				--)
					rule_begin=true
					;;
				*)
					option_parsed=false
					;;
			esac
		fi
		if ! $option_parsed; then
			if [ -n "$rule_type" ] || [ -n "$rule_value" ]; then
				echo "UPWARD_RULE has already been specified" >&2
				return 2
			fi
			case "$1" in
				-n*)
					rule_type="n"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						rule_value=1
					fi
					case "$rule_value" in
						*[!0-9]*)
							echo "NUM_LEVELS should be int but got $rule_value" >&2
							return 2
							;;
					esac
					;;
				-r*)
					rule_type="raw"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						echo "NAME missing" >&2
						return 2
					fi
					;;
				/*/)
					rule_type="glob"
					if [ -z "${1:1:-1}" ]; then
						echo "PATTERN missing" >&2
						return 2
					else
						rule_value="${1:1:-1}"
					fi
					;;
				-e*)
					rule_type="bre"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						echo "REGEX missing" >&2
						return 2
					fi
					;;
				-E*)
					rule_type="ere"
					if [ -n "${1:2}" ]; then
						rule_value="${1:2}"
					elif [ -n "$2" ]; then
						rule_value="$2"
						shift
					else
						echo "REGEX missing" >&2
						return 2
					fi
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

	# do cd
	todir="$(pwd)"
	case "$rule_type" in
		n)
			while [ "$rule_value" -gt 0 ]; do
				todir="$(dirname "$todir")"
				if [ "$todir" = "/" ]; then
					break
				fi
				rule_value="$(( rule_value - 1 ))"
			done
			;;
		raw)
			todir="$(dirname "$todir")" 
			while [ "$todir" != "/" ]; do
				if [ "$(basename "$todir")" = "$rule_value" ]; then
					break
				fi
				todir="$(dirname "$todir")"
			done
			if [ "$todir" = "/" ]; then
				return 4
			fi
			;;
		glob)
			todir="$(dirname "$todir")"
			while [ "$todir" != "/" ]; do
				if find "$(dirname "$todir")"/ -mindepth 1 -maxdepth 1 -type d -name "$rule_value" \
						| sed 's://*:/:g' \
						| grep -qxF "$todir"; then
					break
				fi
				todir="$(dirname "$todir")"
			done
			if [ "$todir" = "/" ]; then
				return 4
			fi
			;;
		bre)
			todir="$(dirname "$todir")"
			while [ "$todir" != "/" ]; do
				if grep -q "$rule_value" <<< "$(basename "$todir")"; then
					break
				fi
				todir="$(dirname "$todir")"
			done
			if [ "$todir" = "/" ]; then
				return 4
			fi
			;;
		ere)
			todir="$(dirname "$todir")"
			while [ "$todir" != "/" ]; do
				if grep -qE "$rule_value" <<< "$(basename "$todir")"; then
					break
				fi
				todir="$(dirname "$todir")"
			done
			if [ "$todir" = "/" ]; then
				return 4
			fi
			;;
	esac
	if [ -n "$subdir" ]; then
		todir="$(sed 's://*:/:g' <<< "${todir}/${subdir}")"
	fi
	if $listonly; then
		( cd "$todir" && pwd; )
	else
		# Mean to fail if cd fails
		# shellcheck disable=SC2164
		cd "$todir"
	fi
}
