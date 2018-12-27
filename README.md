# cdup

`cd` up to several level from current working dirctory.


## Installation

Source `cdup.sh`.


## Usage

	up: same as `cd ..`
	up -n LEVEL: same as `cd ..` for LEVEL times
	up DIR: same as `cd ..` for N times where N is the smallest
	        nonnegative integer such that the target directory is
	        named "DIR"
	up '/DIR/': same as `cd ..` for N times where N is the smallest
	            nonnegative integer such that the target directory's
	            name matches "DIR" as a grep-style regular expression

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
