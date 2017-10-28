#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_switch_stacks):
    .cfi_startproc
    push %ebp
    .cfi_def_cfa_offset 8
    .cfi_offset ebp, -8
    mov %esp, %ebp
    .cfi_def_cfa_register ebp
    mov 16(%ebp), %esp  // switch to our new stack
    mov 12(%ebp), %eax  // load function we're going to call
    push 8(%ebp)        // push argument to first function
    call *%eax          // call our function pointer
    mov %ebp, %esp      // restore the old stack pointer
    pop %ebp
    ret
    .cfi_endproc
