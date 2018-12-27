#!/bin/bash

__up_help() 
{ 
    cat  <<EOF
Usage: up                   Same as \`cd ..'
       up -n LEVEL          Same as \`cd ../.. etc.' (LEVEL \`..'s)
       up NAME              Go upwards to the nearest directory named NAME
       up /REGEX/           Go upwards to the nearest directory matching REGEX
                            (grep-style regex)
       up [-h | --help]     Show this help and exit
EOF
}

up() 
{ 
    local uplevel=;
    local todirname=;
    local todirpattern=;
    local xpat=;
    local todir=;
    while [ -n "$1" ]; do
        if [ "$1" = "-n" ]; then
            shift;
            if [ -z "$1" ]; then
                __up_help;
                return 1;
            else
                uplevel="$1";
            fi;
        else
            if [ "$1" = "-h" -o "$1" = "--help" ]; then
                __up_help;
                return 0;
            else
                if [ "$1" = "-E" ]; then
                    xpat="-E";
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
        shift;
    done;
    if [ -z "$uplevel" -a -z "$todirname" -a -z "$todirpattern" ]; then
        uplevel=1;
    fi;
    if [ -n "$uplevel" ]; then
        if echo "$uplevel" | grep -q '^+\?[0-9]\+$'; then
            if [ "$uplevel" -gt 0 ]; then
                todir="$(pwd)";
                while [ "$uplevel" -gt 0 ]; do
                    todir="$(dirname "$todir")";
                    uplevel="$(( "$uplevel" - 1 ))";
                done;
                cd "$todir";
            else
                return 0;
            fi;
        else
            echo LEVEL should be nonnegative integer >> /dev/stderr;
            return 1;
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
                cd "$todir";
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
                    cd "$todir";
                fi;
            fi;
        fi;
    fi
}
