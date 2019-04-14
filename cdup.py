#!/usr/bin/env python
import sys
import os
import re
from functools import partial
import fnmatch
import logging

ERROR_ARGS = 2
ERROR_NOMATCH = 4

RULE_TYPES = 'n', 'raw', 'glob', 'ere'

logging.basicConfig(level=logging.WARNING, format='%(levelname)s:%(message)s')


def parse_args():
    try:
        ruletype = sys.argv[1]
        rulevalue = sys.argv[2]
        try:
            subsequent = sys.argv[3] or None
        except IndexError:
            subsequent = None
        return ruletype, rulevalue, subsequent
    except IndexError:
        sys.exit(ERROR_ARGS)


def search_upward(fromdir, condition):
    """
    Search upward from ``fromdir`` (inclusive) till reaching root.
    (inclusive).

    :param fromdir: from which directory to search
    :param condition: if ``condition(current_directory)`` returns ``True``,
           returns ``current_directory``
    """
    cwd = fromdir
    if condition(cwd):
        return cwd
    next_cwd = os.path.dirname(cwd)
    while not os.path.samefile(cwd, next_cwd):
        cwd = next_cwd
        if condition(cwd):
            return cwd
        next_cwd = os.path.dirname(cwd)
    sys.exit(ERROR_NOMATCH)


def upward_atmost(fromdir, n):
    cwd = fromdir
    next_cwd = os.path.dirname(cwd)
    while n > 0 and not os.path.samefile(cwd, next_cwd):
        cwd = next_cwd
        n -= 1
        next_cwd = os.path.dirname(cwd)
    return cwd


def predicate_raw(string, cwd):
    logging.debug('(raw) %s -> pattern=%r',  cwd, string)
    return string == os.path.basename(cwd)


def predicate_glob(pattern, cwd):
    name = os.path.basename(cwd)
    return fnmatch.fnmatch(name, pattern)


def predicate_ere(pattern, cwd):
    name = os.path.basename(cwd)
    found = re.search(pattern, name)
    logging.debug('(ere) %s -> pattern=%r; name=%r; found=%r',
                  cwd, pattern, name, found)
    return bool(found)


def main():
    ruletype, rulevalue, subsequent = parse_args()
    cwd = os.path.abspath(os.getcwd())
    if ruletype == 'n':
        todir = upward_atmost(cwd, int(rulevalue))
    elif ruletype in RULE_TYPES:
        c = partial(globals()['predicate_' + ruletype], rulevalue)
        todir = search_upward(cwd, c)
    else:
        sys.exit(ERROR_ARGS)
    try:
        todir = os.path.join(todir, subsequent)
    except (TypeError, AttributeError):
        # In python2.7, AttributeError is raised when subsequent is None;
        # In python3.x, TypeError is raised instead.
        pass
    sys.stdout.write(todir)
    sys.stdout.write('\n')
    sys.stdout.flush()
    sys.exit(0)


main()
