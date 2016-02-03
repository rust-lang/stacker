#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_stack_pointer):
    mov %esp, %eax
    ret

GLOBAL(__stacker_switch_stacks):
    push %ebp
    mov %esp, %ebp
    mov 8(%ebp), %esp   // switch to our new stack
    mov 12(%ebp), %eax  // load function we're going to call
    push 16(%ebp)       // push argument to first function
    call *%eax          // call our function pointer
    mov %ebp, %esp      // restore the old stack pointer
    pop %ebp
    ret
