#!/bin/bash
# also exposed to /bin/zsh

up() {
	# compatible with bash then zsh;
	# reference: https://stackoverflow.com/a/54755784/7881370
	local up_basedir=$(dirname "${BASH_SOURCE[0]:-${(%):-%x}}")
	local up_py_backend=
	local up_pythonbin=
	local up_rs_backend="$up_basedir/rs_backend"
	if [ ! -f "$up_rs_backend" ]; then
		echo "rs_backend not built yet; downgrading to Python backend" >&2
		up_rs_backend=
		up_py_backend="$up_basedir/cdup.py"
		up_pythonbin=python3
	fi

	local cmd
	if [ -n "$up_rs_backend" ]; then
		cmd="$("$up_rs_backend" "$(pwd)" "$@")"
	else
		cmd="$("$up_pythonbin" "$up_py_backend" "$(pwd)" "$@")"
	fi
	local retcode="$?"
	if [ "$retcode" != 0 ]; then
		return "$retcode"
	fi
	eval "$cmd"
}
