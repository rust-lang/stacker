#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret
GLOBAL(__stacker_stack_pointer):
    movq %rsp, %rax
    ret

GLOBAL(__stacker_switch_stacks):
    // CFI instructions tells the unwinder how to unwind this function
    // This enables unwinding through our extended stacks and also
    // backtrackes
    .cfi_startproc
    push %rbp
    .cfi_def_cfa_offset 16 // restore rsp by adding 16
    .cfi_offset rbp, -16   // restore rbp from the stack
    mov %rsp, %rbp
    .cfi_def_cfa_register rbp // restore rsp from rbp
    mov %rdx, %rsp      // switch to our new stack
    call *%rsi          // call our function pointer, data argument in %rdi
    mov %rbp, %rsp      // restore the old stack pointer
    pop %rbp
    ret
    .cfi_endproc
