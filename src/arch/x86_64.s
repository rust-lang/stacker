#include "psm.h"
/* NOTE: sysv64 calling convention is used on all x86_64 targets, including Windows! */

.text
.globl rust_psm_stack_direction
.p2align 4
.type rust_psm_stack_direction,@function
rust_psm_stack_direction:
/* extern "sysv64" fn() -> u8 (%al) */
.cfi_startproc
    movb $STACK_DIRECTION_DESCENDING, %al # always descending on x86_64
    retq
.rust_psm_stack_direction_end:
.size       rust_psm_stack_direction,.rust_psm_stack_direction_end-rust_psm_stack_direction
.cfi_endproc


.globl rust_psm_stack_pointer
.p2align 4
.type rust_psm_stack_pointer,@function
rust_psm_stack_pointer:
/* extern "sysv64" fn() -> *mut u8 (%rax) */
.cfi_startproc
    movq %rsp, %rax
    retq
.rust_psm_stack_pointer_end:
.size       rust_psm_stack_pointer,.rust_psm_stack_pointer_end-rust_psm_stack_pointer
.cfi_endproc


.globl rust_psm_replace_stack
.p2align 4
.type rust_psm_replace_stack,@function
rust_psm_replace_stack:
/* extern "sysv64" fn(%rdi: usize, %rsi: extern "sysv64" fn(usize), %rdx: *mut u8) */
.cfi_startproc
/* All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi */
movq %rdx, %rsp
jmpq *%rsi
.rust_psm_replace_stack_end:
.size       rust_psm_replace_stack,.rust_psm_replace_stack_end-rust_psm_replace_stack
.cfi_endproc


.globl rust_psm_on_stack
.p2align 4
.type rust_psm_on_stack,@function
rust_psm_on_stack:
/* extern "sysv64" fn(%rdi: usize, %rsi: usize, %rdx: extern "sysv64" fn(usize, usize), %rcx: *mut u8) */
.cfi_startproc
/* All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi */
pushq %rbp
movq  %rsp, %rbp
movq  %rcx, %rsp
callq *%rdx
movq  %rbp, %rsp
popq  %rbp
retq
.rust_psm_on_stack_end:
.size       rust_psm_on_stack,.rust_psm_on_stack_end-rust_psm_on_stack
.cfi_endproc
