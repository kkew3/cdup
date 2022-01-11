import sys
import os
import re
import functools
import fnmatch
import logging
import collections

ERROR_ARGS = 2
ERROR_NOMATCH = 4

RULE_TYPES = ['n', 'raw', 'glob', 'regex']

DESCRIPTION = """usage: up [OPTIONS...] [[--] UPWARD_RULE]

OPTIONS

    -h, --help              Show this help and return 0
    -s DIR                  Going downwards to DIR after going upwards, such
                            that there's only one `cd' action in total
    -l                      Print the absolute target directory rather than
                            actually cd to it; the target directory will be
                            printed regardless of its existence

UPWARD_RULE

    Can be one of:

        <Nothing>           Same as `cd ..'
        -NUM_LEVELS         Same as `cd ..' NUM_LEVELS time but there will be
                            only one `cd' action in total. If NUM_LEVELS
                            contains non-digit characters, or if NUM_LEVELS
                            is empty, the entire `-NUM_LEVELS' will be
                            interpreted as `NAME' (see below). If NUM_LEVELS
                            is `0', nothing will be done
        [-r] NAME           Go upwards to the nearest directory named NAME.
                            The optional `-r' disambiguates cases when NAME
                            starts with `-'
        -g PATTERN          Go upwards to the nearest directory matching the
                            python-style globbing pattern PATTERN. Be sure to
                            add quote around PATTERN to avoid unnecessary
                            shell expansion
        -E REGEX            Go upwards to the nearest directory matching the
                            python REGEX

The order of OPTIONS and UPWARD_RULE does not matter, and can be interleaved.
The optional `--' marks the beginning of UPWARD_RULE. Short options cannot
be merged together. Option with argument can be merged together. No
UPWARD_RULE other than `-NUM_LEVELS' is allowed to reach the root
directory (`/').

Error code

    0                       Successs
    1                       cd error (`No such file or directory'). This
                            error is most often triggered by `-s' option as
                            unable to target directory upward will lead to
                            return code 4
    2                       Cmd argument error
    4                       Cannot find the target directory upward"""

logging.basicConfig(level=logging.WARNING, format='%(levelname)s:%(message)s')

Cli = collections.namedtuple('Cli', 'cwd subsequent list rule_type rule_value')


# pylint: disable=too-many-statements, too-many-branches
def parse_args(argv):
    """
    >>> parse_args_full(['bin', '/hello/world'])
    Cli(cwd='/hello/world', subsequent=None, list=False, rule_type='n', rule_value='1')
    >>> parse_args_full(['bin', '/hello/world', '-gP*s'])
    Cli(cwd='/hello/world', subsequent=None, list=False, rule_type='glob', rule_value='P*s')
    >>> parse_args_full(['bin', '/hello/world', '-E', 'P', '-sagain', '-l'])
    Cli(cwd='/hello/world', subsequent='again', list=True, rule_type='regex', rule_value='P')
    >>> parse_args_full(['bin', '/hello/world', '--xxx'])
    Cli(cwd='/hello/world', subsequent=None, list=False, rule_type='raw', rule_value='--xxx')
    >>> parse_args_full(['bin', '/hello/world', '--', '-s'])
    Cli(cwd='/hello/world', subsequent=None, list=False, rule_type='raw', rule_value='-s')
    >>> parse_args_full(['bin', '/hello/world', '-r-sagain', '-s', 'from'])
    Cli(cwd='/hello/world', subsequent='from', list=False, rule_type='raw', rule_value='-sagain')
    """
    cwd = None
    subsequent = None
    list_ = False
    rule_type = None
    rule_value = None

    rule_begin = False

    n = len(argv)
    # the first argument must be cwd
    cwd = argv[1]
    cur = 2

    while cur < n:
        # set to True if one of the OPTIONS or `--' has been parsed in current
        # loop
        option_parsed = False
        if not rule_begin:
            if argv[cur] in ['-h', '--help']:
                print(DESCRIPTION)
                sys.exit(0)
            if argv[cur].startswith('-s'):
                if argv[cur][2:]:
                    subsequent = argv[cur][2:]
                elif cur + 1 < n:
                    subsequent = argv[cur + 1]
                    cur += 1
                else:
                    print('DIR missing', file=sys.stderr)
                    sys.exit(ERROR_ARGS)
                option_parsed = True
            elif argv[cur] == '-l':
                list_ = True
                option_parsed = True
            elif argv[cur] == '--':
                rule_begin = True
                option_parsed = True
        if not option_parsed:
            if rule_type:
                print(
                    'UPWARD_RULE has already been specified', file=sys.stderr)
                sys.exit(ERROR_ARGS)
            if argv[cur].startswith('-r'):
                rule_type = 'raw'
                if argv[cur][2:]:
                    rule_value = argv[cur][2:]
                elif cur + 1 < n:
                    rule_value = argv[cur + 1]
                    cur += 1
                else:
                    print('NAME missing', file=sys.stderr)
                    sys.exit(ERROR_ARGS)
            elif argv[cur].startswith('-g'):
                rule_type = 'glob'
                if argv[cur][2:]:
                    rule_value = argv[cur][2:]
                elif cur + 1 < n:
                    rule_value = argv[cur + 1]
                    cur += 1
                else:
                    print('PATTERN missing', file=sys.stderr)
                    sys.exit(ERROR_ARGS)
            elif argv[cur].startswith('-E'):
                rule_type = 'regex'
                if argv[cur][2:]:
                    rule_value = argv[cur][2:]
                elif cur + 1 < n:
                    rule_value = argv[cur + 1]
                    cur += 1
                else:
                    print('REGEX missing', file=sys.stderr)
                    sys.exit(ERROR_ARGS)
            elif argv[cur].startswith('-'):
                if argv[cur][1:].isnumeric():
                    # argv[cur][1:] is nonempty and contains only digits
                    rule_type = 'n'
                    rule_value = argv[cur][1:]
                else:
                    rule_type = 'raw'
                    rule_value = argv[cur]
            else:
                rule_type = 'raw'
                rule_value = argv[cur]
        cur += 1

    if not rule_type:
        rule_type = 'n'
        rule_value = '1'

    return Cli(cwd, subsequent, list_, rule_type, rule_value)


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
    logging.debug('(regex) %s -> pattern=%r; name=%r; found=%r', cwd, pattern,
                  name, found)
    return bool(found)


predicate_by_ruletype = {
    'raw': predicate_raw,
    'glob': predicate_glob,
    'regex': predicate_regex,
}


def main():
    cli = parse_args(sys.argv)
    if cli.rule_type == 'n':
        todir = upward_atmost(cli.cwd, int(cli.rule_value))
    elif cli.rule_type in RULE_TYPES:
        c = functools.partial(predicate_by_ruletype[cli.rule_type],
                              cli.rule_value)
        todir = search_upward(cli.cwd, c)
    else:
        print('up: invalid rule type', file=sys.stderr)
        sys.exit(ERROR_ARGS)
    if cli.subsequent:
        todir = os.path.join(todir, cli.subsequent)
    if cli.list:
        print('echo "{}"'.format(todir))
    elif not os.path.isdir(todir) or not os.path.samefile(cli.cwd, todir):
        print('cd "{}"'.format(todir))


if __name__ == '__main__':
    main()
