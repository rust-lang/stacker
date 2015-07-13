#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_stack_pointer):
    movq %rsp, %rax
    ret

#if defined(LINUX)
GLOBAL(__stacker_stack_limit):
    movq %fs:0x70, %rax
    ret

GLOBAL(__stacker_set_stack_limit):
    movq %rdi, %fs:0x70
    ret
#elif defined(APPLE)
GLOBAL(__stacker_stack_limit):
    movq %gs:0x330, %rax
    ret

GLOBAL(__stacker_set_stack_limit):
    movq %rdi, %gs:0x330
    ret
#else
#error "unsupported arch"
#endif

GLOBAL(__stacker_switch_stacks):
    push %rbp
    mov %rsp, %rbp
    mov %rdi, %rsp      // switch to our new stack
    mov %rdx, %rdi      // move the data pointer to the first argument
    call *%rsi          // call our function pointer
    mov %rbp, %rsp      // restore the old stack pointer
    pop %rbp
    ret
