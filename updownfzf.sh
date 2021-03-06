#!/bin/bash

#
# up-down using fzf
#

__updown_up() {
	local curdir="$(pwd)"
	local nextdir
	if [ "$curdir" = "$HOME" ] || [ "$curdir" = "/" ]; then
		echo "$curdir"
	else
		while [ "$curdir" != "$HOME" ]; do
			echo "$curdir"
			nextdir="$(dirname "$curdir")"
			if [ "$nextdir" = "$curdir" ]; then
				break
			fi
			curdir="$nextdir"
		done
	fi
}

__updown_down() {
	find "$1"/ -mindepth 1 ! -readable -prune -o -path '*/.*/*' -prune -o -type d -print
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
