#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_switch_stacks):
    .cfi_startproc
    push %rbp
    .cfi_def_cfa_offset 16
    .cfi_offset rbp, -16
    mov %rsp, %rbp
    .cfi_def_cfa_register rbp
    mov %rdx, %rsp      // switch to our new stack
    call *%rsi          // call our function pointer, data argument in %rdi
    mov %rbp, %rsp      // restore the old stack pointer
    pop %rbp
    ret
    .cfi_endproc
