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

#if defined(LINUX)
GLOBAL(__stacker_morestack_stack_limit):
    movq %fs:0x70, %rax
    ret

GLOBAL(__stacker_set_morestack_stack_limit):
    movq %rdi, %fs:0x70
    ret
#elif defined(APPLE)
GLOBAL(__stacker_morestack_stack_limit):
    movq %gs:0x330, %rax
    ret

GLOBAL(__stacker_set_morestack_stack_limit):
    movq %rdi, %gs:0x330
    ret
#elif defined(WINDOWS)
GLOBAL(__stacker_morestack_stack_limit):
    movq $0, %rax
    ret

GLOBAL(__stacker_set_morestack_stack_limit):
    ret
#else
#error "unsupported arch"
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
