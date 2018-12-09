#include "psm.h"

.text
.globl rust_psm_stack_direction
.p2align 2
.type rust_psm_stack_direction,%function
.code 32
rust_psm_stack_direction:
/* extern "C" fn() -> u8 */
.fnstart
.cfi_startproc
    mov r0, #STACK_DIRECTION_DESCENDING
    bx lr
.rust_psm_stack_direction_end:
.size       rust_psm_stack_direction,.rust_psm_stack_direction_end-rust_psm_stack_direction
.cfi_endproc
.cantunwind
.fnend

.globl rust_psm_stack_pointer
.p2align 2
.type rust_psm_stack_pointer,%function
.code 32
rust_psm_stack_pointer:
/* extern "C" fn() -> *mut u8 */
.fnstart
.cfi_startproc
    mov r0, sp
    bx  lr
.rust_psm_stack_pointer_end:
.size       rust_psm_stack_pointer,.rust_psm_stack_pointer_end-rust_psm_stack_pointer
.cfi_endproc
.cantunwind
.fnend


.globl rust_psm_replace_stack
.p2align 2
.type rust_psm_replace_stack,%function
.code 32
rust_psm_replace_stack:
/* extern "C" fn(r0: usize, r1: extern "C" fn(usize), r2: *mut u8) */
.fnstart
.cfi_startproc
/* All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi */
    mov sp, r2
    bx r1
.rust_psm_replace_stack_end:
.size       rust_psm_replace_stack,.rust_psm_replace_stack_end-rust_psm_replace_stack
.cfi_endproc
.cantunwind
.fnend


.globl rust_psm_on_stack
.p2align 2
.type rust_psm_on_stack,%function
.code 32
rust_psm_on_stack:
/* extern "C" fn(r0: usize, r1: usize, r2: extern "C" fn(usize, usize), r3: *mut u8) */
.fnstart
.cfi_startproc
    push {r4, lr}
    mov r4, sp
    mov sp, r3
    blx r2
    mov sp, r4
    pop {r4, pc}
.rust_psm_on_stack_end:
.size       rust_psm_on_stack,.rust_psm_on_stack_end-rust_psm_on_stack
.cfi_endproc
.cantunwind
.fnend
