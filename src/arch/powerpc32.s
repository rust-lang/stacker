#include "psm.h"

/* NOTE: required stack alignment is 16 bytes */
/* FIXME: should test this */

.text
.globl rust_psm_stack_direction
.p2align 2
.type rust_psm_stack_direction,@function
rust_psm_stack_direction:
/* extern "C" fn() -> u8 */
.cfi_startproc
    li 3, STACK_DIRECTION_DESCENDING
    blr
.rust_psm_stack_direction_end:
.size       rust_psm_stack_direction,.rust_psm_stack_direction_end-rust_psm_stack_direction
.cfi_endproc


.globl rust_psm_stack_pointer
.p2align 2
.type rust_psm_stack_pointer,@function
rust_psm_stack_pointer:
/* extern "C" fn() -> *mut u8 */
.cfi_startproc
    mr 3, 1
    blr
.rust_psm_stack_pointer_end:
.size       rust_psm_stack_pointer,.rust_psm_stack_pointer_end-rust_psm_stack_pointer
.cfi_endproc


.globl rust_psm_replace_stack
.p2align 2
.type rust_psm_replace_stack,@function
rust_psm_replace_stack:
/* extern "C" fn(3: usize, 4: extern "C" fn(usize), 5: *mut u8) */
.cfi_startproc
    add 5, 5, 16
    mr 1, 5
    mtctr 4
    bctr
.rust_psm_replace_stack_end:
.size       rust_psm_replace_stack,.rust_psm_replace_stack_end-rust_psm_replace_stack
.cfi_endproc


.globl rust_psm_on_stack
.p2align 2
.type rust_psm_on_stack,@function
rust_psm_on_stack:
/* extern "C" fn(3: usize, 4: usize, 5: extern "C" fn(usize, usize), 6: *mut u8) */
.cfi_startproc
/* This is somewhat different from x86/ARM. While the stack in PowerPC also grows downward, its
 * operation is fundamentally different. Most notably, on x86/ARM, the callee never looks at the
 * caller’s stack area, whereas on PowerPC it is a given. Therefore, unlike on those architectures,
 * we must set-up our replacement stack in certain ways to get something caller would expect from
 * us.
 */
    /* store ctr from 5th register */
    mtctr 5

    /* save the link register, and "push" stack */
    mflr 0
    stw 0, 4(1)
    stwu 1, -16(1)
    /* Save the non-volatile 14-th register on stack */
    stw 14, 8(1)
    /* do the stack pointer swapping */
    mr 14, 1
    mr 1, 6
    /* call address stored in ctr with linking back */
    bctrl
    /* Restore old stack pointer */
    mr 1, 14
    /* Load the non-volatile 14th register from stack */
    lwz 14, 8(1)
    /* Load link register back from stack and return */
    lwz 0, 4(1)
    mtlr 0
    blr
.rust_psm_on_stack_end:
.size       rust_psm_on_stack,.rust_psm_on_stack_end-rust_psm_on_stack
.cfi_endproc
