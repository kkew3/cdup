# cdup

`cd` up to several level from current working dirctory.


## Installation

Source `cdup.sh`.


## Usage

	Usage: up                   Same as `cd ..'
	       up -n LEVEL          Same as `cd ../.. etc.' (LEVEL `..'s)
	       up NAME              Go upwards to the nearest directory named NAME
	       up /REGEX/           Go upwards to the nearest directory matching REGEX
				    (grep-style regex)
	       ... [-s DIR]         Any command pattern above plus this option will
				    go up to the underlying ancestor directory before
				    going downwards to DIR, such that there's only one
				    `cd' action
	       up [-h | --help]     Show this help and exit

One difference from "`cd ..` for N times" is that `up` function addresses the `OLDPWD` better than `cd ..` multiple times.

Several use cases:

	/home/user/directory1/directory2:$ up
	/home/user/directory1:$ 

	/home/user/directory1/directory2:$ up user
	/home/user:$ cd -
	/home/user/directory1/directory2:$ 

	/home/user/directory1/directory2:$ up '/d*1/'
	/home/user/directory1:$ cd -
	/home/user/directory1/directory2:$ 
