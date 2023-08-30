#!/bin/bash
CC=riscv64-linux-gnu-gcc
test_name=$(basename "$0" .sh)
t=out/tests/$test_name

set -e

mkdir -p "$t"

cat <<EOF | $CC -o "$t"/a.o -c -xc -
#include <stdio.h>

int main(void) {
    printf("Hello, World\n");
    return 0;
}
EOF
# -Bprefix
# This option specifies where to find the executables, libraries, include files, and data files of the compiler itself.
# The compiler driver program runs one or more of the subprograms cpp, cc1, as and ld. It tries prefix as a prefix for each program it tries to run, both with and without ‘machine/version/’ for the corresponding target machine and compiler version.
# For each subprogram to be run, the compiler driver first tries the -B prefix, if any. If that name is not found, or if -B is not specified, the driver tries two standard prefixes, /usr/lib/gcc/ and /usr/local/lib/gcc/. If neither of those results in a file name that is found, the unmodified program name is searched for using the directories specified in your PATH environment variable.
# The compiler checks to see if the path provided by -B refers to a directory, and if necessary it adds a directory separator character at the end of the path.
# -B prefixes that effectively specify directory names also apply to libraries in the linker, because the compiler translates these options into -L options for the linker. They also apply to include files in the preprocessor, because the compiler translates these options into -isystem options for the preprocessor. In this case, the compiler appends ‘include’ to the prefix.
# The runtime support file libgcc.a can also be searched for using the -B prefix, if needed. If it is not found there, the two standard prefixes above are tried, and that is all. The file is left out of the link if it is not found by those means.
# Another way to specify a prefix much like the -B prefix is to use the environment variable GCC_EXEC_PREFIX.
$CC -B. -static "$t"/a.o -o "$t"/out
file "$t"/out
stat "$t"/out
file "$t"/out | grep -q "ELF"
