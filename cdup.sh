#!/bin/bash

up() {
	# the executable path of `parentlevel` python script
	local parentlevel=./parentlevel

	local parent="$1"
	local todir=""
	local errno=0
	if [ -z "$parent" ]; then
		todir=".."
	else
		local levelup=""
		if [ "${parent:0:1}" = "/" ]; then
			if ! "$parentlevel" -e "${parent#?}" > /dev/null; then errno=1; fi
			levelup=$("$parentlevel" -e "${parent#?}")
		else
			if ! "$parentlevel" "${parent}" > /dev/null; then errno=1; fi
			levelup=$("$parentlevel" "${parent}")
		fi
		if [ "$errno" = 0 ]; then
			if [ "$levelup" -gt 0 ]; then
				todir="."
				for ((n=0;n<$levelup;n++)); do
					todir="$todir/.."
				done
			fi
		fi
	fi
	if [ "$errno" = 0 ]; then
		if [ ! -z "$todir" ]; then
			cd "$todir"
		fi
	else
		return "$errno"
	fi
}
