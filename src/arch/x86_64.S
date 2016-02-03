#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_stack_pointer):
    movq %rsp, %rax
    ret

#if defined(WINDOWS)
#define ARG1 %rcx
#define ARG2 %rdx
#define ARG3 %r8
#else
#define ARG1 %rdi
#define ARG2 %rsi
#define ARG3 %rdx
#endif

GLOBAL(__stacker_switch_stacks):
    push %rbp
    mov %rsp, %rbp
    mov ARG1, %rsp      // switch to our new stack
    mov ARG3, ARG1      // move the data pointer to the first argument
    call *ARG2          // call our function pointer
    mov %rbp, %rsp      // restore the old stack pointer
    pop %rbp
    ret
