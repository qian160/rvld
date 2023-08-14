#!/bin/bash
CC=riscv64-linux-gnu-gcc
test_name=$(basename "$0" .sh)
t=out/tests/$test_name

mkdir -p "$t"

cat <<EOF | $CC -o "$t"/a.o -c -xc -
#include <stdio.h>

int main(void) {
    printf("Hello, World\n");
    return 0;
}
EOF

# gcc will finally call ld with some arguments, and we use -B. with ln -sf to redirect to our rvld
# and we need to deal with the arguments in rvld
$CC -B. -static "$t"/a.o -o "$t"/out