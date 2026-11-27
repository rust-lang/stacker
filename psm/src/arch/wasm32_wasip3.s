#include "psm.h"
#include "gnu_stack_note.s"

# `wasm32-wasip3` links with `--cooperative-threading`, under which the shadow stack
# pointer is not a `__stack_pointer` WebAssembly global at all (see README.md): it
# lives in a component-model task-context slot, and `rust-lld` rejects any object
# that references a classic `__stack_pointer` global outright. wasi-libc's
# `crt1-*.o` gets and sets that slot through a pair of imported functions,
# `__wasm_get_stack_pointer`/`__wasm_set_stack_pointer` (module `env`), which
# `wit-component` recognizes by name and rewrites into the real task-context
# accessors when a core module becomes a component. This file imports and calls
# those same functions in place of every `global.get`/`global.set __stack_pointer`
# `wasm32.s` has, so `on_stack`/`replace_stack` work the same way here as there.
#
# Generated the same way `wasm32.o` is (see that file's own header), from this source
# instead:
#
#    clang -E -x assembler-with-cpp psm/src/arch/wasm32_wasip3.s | llvm-mc -o psm/src/arch/wasm32_wasip3.o --arch=wasm32 -filetype=obj
#
# where `clang`/`llvm-mc` need to be a relatively recent LLVM (confirmed with the one
# shipped in a WASI SDK release, since `cpp` alone does not speak this file's GAS-style
# `#`-comments the way `-x assembler-with-cpp` does).

.globl __wasm_get_stack_pointer
.type __wasm_get_stack_pointer,@function
.functype __wasm_get_stack_pointer () -> (i32)
.import_module __wasm_get_stack_pointer, env
.import_name __wasm_get_stack_pointer, __wasm_get_stack_pointer

.globl __wasm_set_stack_pointer
.type __wasm_set_stack_pointer,@function
.functype __wasm_set_stack_pointer (i32) -> ()
.import_module __wasm_set_stack_pointer, env
.import_name __wasm_set_stack_pointer, __wasm_set_stack_pointer

.globl rust_psm_stack_direction
.type rust_psm_stack_direction,@function
rust_psm_stack_direction:
.functype rust_psm_stack_direction () -> (i32)
    i32.const STACK_DIRECTION_DESCENDING
    end_function

.globl rust_psm_stack_pointer
.type rust_psm_stack_pointer,@function
rust_psm_stack_pointer:
.functype rust_psm_stack_pointer () -> (i32)
    call __wasm_get_stack_pointer
    end_function

.globl rust_psm_on_stack
.type rust_psm_on_stack,@function
rust_psm_on_stack:
.functype rust_psm_on_stack (i32, i32, i32, i32) -> ()
    # get our new stack argument, then save the old stack
    # pointer into that local
    local.get 3
    call __wasm_get_stack_pointer
    local.set 3
    call __wasm_set_stack_pointer

    # Call our indirect function specified
    local.get 0
    local.get 1
    local.get 2
    call_indirect (i32, i32) -> ()

    # restore the stack pointer before returning
    local.get 3
    call __wasm_set_stack_pointer
    end_function

.globl rust_psm_replace_stack
.type rust_psm_replace_stack,@function
rust_psm_replace_stack:
.functype rust_psm_replace_stack (i32, i32, i32) -> ()
    local.get 2
    call __wasm_set_stack_pointer
    local.get 0
    local.get 1
    call_indirect (i32) -> ()
    unreachable
    end_function
