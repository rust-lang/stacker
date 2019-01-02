#!/bin/sh

# See the README.md for more info.

set -eux

clang -c --target=wasm32 black_box.s stack_pointer.s switch_stacks.s
lld -flavor wasm -r black_box.o switch_stacks.o stack_pointer.o -o libstacker.a
