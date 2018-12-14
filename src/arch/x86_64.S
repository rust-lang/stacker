#include <asm.h>

.text

GLOBAL(__stacker_stack_pointer):
    movq %rsp, %rax
    ret

GLOBAL(__stacker_switch_stacks):
    push %rbp
    mov %rsp, %rbp
    mov %rdx, %rsp      // switch to our new stack
    call *%rsi          // call our function pointer, data argument in %rdi
    mov %rbp, %rsp      // restore the old stack pointer
    pop %rbp
    ret
