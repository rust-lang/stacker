_text SEGMENT

__stacker_stack_pointer PROC
    MOV RAX, RSP
    RET
__stacker_stack_pointer ENDP
