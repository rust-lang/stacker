.586
.MODEL FLAT, C
.CODE

__stacker_stack_pointer PROC
    MOV EAX, ESP
    RET
__stacker_stack_pointer ENDP
