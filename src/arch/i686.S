#include <asm.h>

.text

GLOBAL(__stacker_black_box):
    ret

GLOBAL(__stacker_stack_pointer):
    mov %esp, %eax
    ret

#if defined(LINUX)
GLOBAL(__stacker_stack_limit):
    mov %gs:0x30, %eax
    ret

GLOBAL(__stacker_set_stack_limit):
    mov 4(%esp), %eax
    mov %eax, %gs:0x30
    ret
#elif defined(APPLE)
GLOBAL(__stacker_stack_limit):
    mov %gs:0x1b0, %eax
    ret

GLOBAL(__stacker_set_stack_limit):
    mov 4(%esp), %eax
    mov %eax, %gs:0x1b0
    ret
#else
#error "unsupported arch"
#endif

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
