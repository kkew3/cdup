# cdup

`cd` up to several level from current working dirctory.

## Installation

First put `parentlevel` to your favorite location, and adjust the 5th line of `cdup.sh` appropriately. Then source the bash script.


```bash
. cdup.sh
```

## Usage

	up: same as `cd ..`
	up DIR: same as `cd ..` for N times where N is the smallest
	        nonnegative integer such that the target directory is
	        named "DIR"
	up '/DIR': same as `cd ..` for N times where N is the smallest
	           nonnegative integer such that the target directory's
	           name matches "DIR" as a python-style regular expression

One difference from "`cd ..` for N times" is that `up` function addresses the `OLDPWD` better than `cd ..` multiple times.

Several use cases:

	/home/user/directory1/directory2:$ up
	/home/user/directory1:$ 

	/home/user/directory1/directory2:$ up user
	/home/user:$ cd -
	/home/user/directory1/directory2:$ 

	/home/user/directory1/directory2:$ up '/d*1'
	/home/user/directory1:$ cd -
	/home/user/directory1/directory2:$ 

## Compatibility

The shell function is not POSIX-compatible due to a bash-specific for loop. The author don't have much time to improve it. The depended python script `parentlevel`, which returns the number of times needed to perform `cd ..`, has compatibility for Python 2/3 and all platforms.
