import sys
import os
import re
import functools
import fnmatch
import glob
import logging

ERROR_ARGS = 2
ERROR_MATCH = 4
ERROR_SAMEDIR = 8

RULE_TYPES = 'n', 'raw', 'glob', 'regex'

logging.basicConfig(level=logging.WARNING, format='%(levelname)s:%(message)s')


def parse_args():
    try:
        cwd = sys.argv[1]
        ruletype = sys.argv[2]
        rulevalue = sys.argv[3]
        try:
            subsequent = sys.argv[4] or None
        except IndexError:
            subsequent = None
        return cwd, ruletype, rulevalue, subsequent
    except IndexError:
        print('up: illegal command line arguments', file=sys.stderr)
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
    # commented out so that the parent dir of cwd is the first string to match
    #if condition(cwd):
    #    return cwd
    next_cwd = os.path.dirname(cwd)
    while not os.path.samefile(cwd, next_cwd):
        cwd = next_cwd
        if condition(cwd):
            return cwd
        next_cwd = os.path.dirname(cwd)
    print("up: no match", file=sys.stderr)
    sys.exit(ERROR_MATCH)


def upward_atmost(fromdir, n):
    cwd = fromdir
    next_cwd = os.path.dirname(cwd)
    while n > 0 and not os.path.samefile(cwd, next_cwd):
        cwd = next_cwd
        n -= 1
        next_cwd = os.path.dirname(cwd)
    return cwd


def predicate_raw(string, cwd):
    logging.debug('(raw) %s -> pattern=%r', cwd, string)
    return string == os.path.basename(cwd)


def predicate_glob(pattern, cwd):
    name = os.path.basename(cwd)
    return fnmatch.fnmatch(name, pattern)


def predicate_regex(pattern, cwd):
    name = os.path.basename(cwd)
    found = re.search(pattern, name)
    logging.debug('(regex) %s -> pattern=%r; name=%r; found=%r',
                  cwd, pattern, name, found)
    return bool(found)


def predicate_git(_1, cwd):
    return os.path.isdir(os.path.join(cwd, '.git'))


def glob_downward(pattern):
    matched_subsequents = []
    for path in glob.iglob(pattern, recursive=True):
        if os.path.isdir(path):
            matched_subsequents.append(path)
        if len(matched_subsequents) > 1:
            break
    n = len(matched_subsequents)
    if n == 0:
        print('up: no match for DIR', file=sys.stderr)
        sys.exit(ERROR_MATCH)
    if n == 1:
        return matched_subsequents[0]
    print('up: multiple matches for DIR', file=sys.stderr)
    sys.exit(ERROR_MATCH)


predicate_by_ruletype = {
    'raw': predicate_raw,
    'glob': predicate_glob,
    'regex': predicate_regex,
    'git': predicate_git,
}


def main():
    cwd, ruletype, rulevalue, subsequent = parse_args()
    if ruletype == 'n':
        todir = upward_atmost(cwd, int(rulevalue))
    elif ruletype in RULE_TYPES:
        c = functools.partial(predicate_by_ruletype[ruletype], rulevalue)
        todir = search_upward(cwd, c)
    else:
        print('up: invalid rule type', file=sys.stderr)
        sys.exit(ERROR_ARGS)
    if subsequent:
        os.chdir(todir)
        matched_subsequent = glob_downward(subsequent)
        todir = os.path.join(todir, matched_subsequent)
    if os.path.isdir(todir) and os.path.samefile(todir, cwd):
        sys.exit(ERROR_SAMEDIR)
    sys.stdout.write(todir)


if __name__ == '__main__':
    main()
