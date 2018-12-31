#!/bin/bash

__up_help ()
{
    cat  <<EOF
Usage: up                   Same as \`cd ..'
       up -n LEVEL          Same as \`cd ../.. etc.' (LEVEL \`..'s)
       up NAME              Go upwards to the nearest directory named NAME
       up /REGEX/           Go upwards to the nearest directory matching REGEX
                            (grep-style regex)
       ... [-s DIR]         Any command pattern above plus this option will
                            go up to the underlying ancestor directory before
                            going downwards to DIR, such that there's only one
                            \`cd' action
       up [-h | --help]     Show this help and exit
EOF
}


up ()
{
    local uplevel=;
    local todirname=;
    local todirpattern=;
    local xpat=;
    local todir=;
    local subdir=;
    while [ -n "$1" ]; do
        if [ "${1::2}" = "-n" ]; then
            uplevel="${1:2}";
            case "$uplevel" in
                '' | *[!0-9]*)
                    uplevel=
                ;;
                *)
                    :
                ;;
            esac;
            if [ -z "$uplevel" ]; then
                shift;
                if [ -z "$1" ]; then
                    __up_help;
                    return 1;
                else
                    uplevel="$1";
                fi;
            fi;
        else
            if [ "$1" = "-h" -o "$1" = "--help" ]; then
                __up_help;
                return 0;
            else
                if [ "$1" = "-E" ]; then
                    xpat="-E";
                else
                    if [ "${1::2}" = "-s" ]; then
                        if [ -z "${1:2}" ]; then
                            shift;
                            if [ -z "$1" ]; then
                                __up_help;
                                return 1;
                            else
                                subdir="$1";
                            fi;
                        else
                            subdir="${1:2}";
                        fi;
                    else
                        case "$1" in
                            /*/)
                                todirpattern="$1"
                            ;;
                            *)
                                todirname="$1"
                            ;;
                        esac;
                    fi;
                fi;
            fi;
        fi;
        shift;
    done;
    if [ -z "$uplevel" -a -z "$todirname" -a -z "$todirpattern" ]; then
        uplevel=1;
    fi;
    if [ -n "$uplevel" ]; then
        if [ "$uplevel" -gt 0 ]; then
            todir="$(pwd)";
            while [ "$uplevel" -gt 0 ]; do
                todir="$(dirname "$todir")";
                uplevel="$(( "$uplevel" - 1 ))";
            done;
            if [ -n "$subdir" ]; then
                cd "$todir/$subdir";
            else
                cd "$todir";
            fi;
        else
            return 0;
        fi;
    else
        if [ -n "$todirname" ]; then
            todir="$(pwd)";
            while [ "$todir" != "/" ]; do
                if [ "$(basename "$todir")" = "$todirname" ]; then
                    break;
                fi;
                todir="$(dirname "$todir")";
            done;
            if [ "$todir" = "/" ]; then
                echo Nothing matches NAME \"$todirname\" >> /dev/stderr;
                return 4;
            else
                if [ -n "$subdir" ]; then
                    cd "$todir/$subdir";
                else
                    cd "$todir";
                fi;
            fi;
        else
            if [ -n "$todirpattern" ]; then
                todirpattern="$(echo "$todirpattern" | cut -c2- | rev | cut -c2- | rev)";
                todir="$(pwd)";
                while [ "$todir" != "/" ]; do
                    if echo $(basename "$todir") | grep -q $xpat "$todirpattern"; then
                        break;
                    fi;
                    todir="$(dirname "$todir")";
                done;
                if [ "$todir" = "/" ]; then
                    echo Nothing matches REGEX \'$todirpattern\' >> /dev/stderr;
                    return 4;
                else
                    if [ -n "$subdir" ]; then
                        cd "$todir/$subdir";
                    else
                        cd "$todir";
                    fi;
                fi;
            fi;
        fi;
    fi
}
