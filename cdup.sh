#!/bin/bash

up() {
	# the executable path of `parentlevel` python script
	local parentlevel=./parentlevel

	local parent="$1"
	local todir=""
	if [ -z "$parent" ]; then
		todir=".."
	else
		local levelup=""
		if [ "${parent:0:1}" = "/" ]; then
			levelup=$("$parentlevel" -e "${parent#?}")
		else
			levelup=$("$parentlevel" "${parent}")
		fi
		if [ "$levelup" -gt 0 ]; then
			todir="."
			for ((n=0;n<$levelup;n++)); do
				todir="$todir/.."
			done
		fi
	fi
	if [ ! -z "$todir" ]; then
		cd "$todir"
	fi
}
