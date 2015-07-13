#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_stack_pointer):
    movq %rsp, %rax
    ret

GLOBAL(__stacker_stack_limit):
    movq %fs:0x70, %rax
    ret

GLOBAL(__stacker_set_stack_limit):
    movq %rdi, %fs:0x70
    ret

GLOBAL(__stacker_switch_stacks):
    push %rbp
    mov %rsp, %rbp
    mov %rsp, %rax      // temporary for the old stack pointer
    mov %rdi, %rsp      // switch to our new stack
    push %rax           // save the old stack pointer
    sub $8, %rsp        // align the stack
    mov %rdx, %rdi      // move the data pointer to the first argument
    call *%rsi          // call our function pointer
    mov 8(%rsp), %rsp   // restore the old stack pointer
    pop %rbp
    ret
