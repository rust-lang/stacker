#include "psm.h"
/* NOTE: fastcall calling convention used on all platforms */

.text
.globl rust_psm_stack_direction
.p2align 4
.type rust_psm_stack_direction,@function
rust_psm_stack_direction:
/* extern "fastcall" fn() -> u8 (%al) */
.cfi_startproc
    movb $STACK_DIRECTION_DESCENDING, %al # always descending on x86_64
    retl
.rust_psm_stack_direction_end:
.size       rust_psm_stack_direction,.rust_psm_stack_direction_end-rust_psm_stack_direction
.cfi_endproc


.globl rust_psm_stack_pointer
.p2align 4
.type rust_psm_stack_pointer,@function
rust_psm_stack_pointer:
/* extern "fastcall" fn() -> *mut u8 (%rax) */
.cfi_startproc
    movl %esp, %eax
    retl
.rust_psm_stack_pointer_end:
.size       rust_psm_stack_pointer,.rust_psm_stack_pointer_end-rust_psm_stack_pointer
.cfi_endproc


.globl rust_psm_replace_stack
.p2align 4
.type rust_psm_replace_stack,@function
rust_psm_replace_stack:
/* extern "fastcall" fn(%ecx: usize, %edx: extern "fastcall" fn(usize), 4(%esp): *mut u8) */
.cfi_startproc
/* All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi */
movl 4(%esp), %esp
jmpl *%edx
.rust_psm_replace_stack_end:
.size       rust_psm_replace_stack,.rust_psm_replace_stack_end-rust_psm_replace_stack
.cfi_endproc


.globl rust_psm_on_stack
.p2align 4
.type rust_psm_on_stack,@function
rust_psm_on_stack:
/* extern "fastcall" fn(%ecx: usize, %edx: usize, 4(%esp): extern "fastcall" fn(usize, usize), 8(%esp): *mut u8) */
.cfi_startproc
/* All we gotta do is set the stack pointer to %rdx & tail-call the callback in %rsi */
pushl %ebp
movl  %esp, %ebp
movl  8(%ebp), %esp
calll *4(%ebp)
movl  %ebp, %esp
popl  %ebp
retl  $8
.rust_psm_on_stack_end:
.size       rust_psm_on_stack,.rust_psm_on_stack_end-rust_psm_on_stack
.cfi_endproc
