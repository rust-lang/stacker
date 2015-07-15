_text SEGMENT

__stacker_black_box PROC
    RET
__stacker_black_box ENDP

__stacker_stack_pointer PROC
    MOV RAX, RSP
    RET
__stacker_stack_pointer ENDP

__stacker_morestack_stack_limit PROC
    MOV RAX, 0
    RET
__stacker_morestack_stack_limit ENDP

__stacker_set_morestack_stack_limit PROC
    RET
__stacker_set_morestack_stack_limit ENDP

__stacker_switch_stacks PROC
    PUSH RBP
    MOV RBP, RSP
    MOV RSP, RDI            ; switch to our new stack
    MOV RDI, RDX            ; move the data pointer to the first argument
    CALL RSI                ; call our function pointer
    MOV RSP, RBP            ; restore the old stack pointer
    POP RBP
    RET
__stacker_switch_stacks ENDP

END
