import sys
import os
import re
import functools
import fnmatch
import logging

ERROR_ARGS = 2
ERROR_NOMATCH = 4

RULE_TYPES = 'n', 'raw', 'glob', 'regex'

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


def getcwd():
    """
    Not using ``os.getcwd()`` unless in Win32 because it resolves symlink as
    per POSIX.1-2008 (IEEE Std 1003.1-2008). End user may adapt this function
    to suit the underlying platform.
    """
    if sys.platform == 'win32':
        return os.getcwd()
    return os.environ['PWD']


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


predicate_by_ruletype = {
    'raw': predicate_raw,
    'glob': predicate_glob,
    'regex': predicate_regex,
}


def main():
    ruletype, rulevalue, subsequent = parse_args()
    cwd = os.path.abspath(getcwd())
    if ruletype == 'n':
        todir = upward_atmost(cwd, int(rulevalue))
    elif ruletype in RULE_TYPES:
        c = functools.partial(predicate_by_ruletype[ruletype], rulevalue)
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


if __name__ == '__main__':
    main()
