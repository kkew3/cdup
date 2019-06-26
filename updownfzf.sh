#!/bin/bash

#
# up-down using fzf
#

__updown_up() {
	local curdir="$(pwd)"
	local nextdir
	while [ "$curdir" != "$HOME" ]; do
		echo "$curdir"
		nextdir="$(dirname "$curdir")"
		if [ "$nextdir" = "$curdir" ]; then
			break
		fi
		curdir="$nextdir"
	done
}

__updown_down() {
	find "$1"/ -mindepth 1 -type d -print
}

ud() {
	local updir="$(__updown_up | fzf)"
	local todir
	if [ -n "$updir" ]; then
		todir="$(__updown_down "$updir" | fzf)"
		if [ -n "$todir" ]; then
			cd "$todir"
			return 0
		fi
	fi
	return 1
}
