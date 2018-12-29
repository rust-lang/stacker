PUBLIC rust_psm_stack_direction
PUBLIC rust_psm_stack_pointer
PUBLIC rust_psm_replace_stack
PUBLIC rust_psm_on_stack

_TEXT SEGMENT

; extern "sysv64" fn() -> u8 (%al)
rust_psm_stack_direction PROC
    mov al, 2
    ret
rust_psm_stack_direction ENDP

; extern "sysv64" fn() -> *mut u8 (%rax)
rust_psm_stack_pointer PROC
    lea rax, [rsp + 8]
    ret
rust_psm_stack_pointer ENDP

; extern "sysv64" fn(%rdi: usize, %rsi: extern "sysv64" fn(usize), %rdx: *mut u8)
rust_psm_replace_stack PROC
    lea rsp, [rdx - 8]
    jmp rsi
rust_psm_replace_stack ENDP

; extern "sysv64" fn(%rdi: usize, %rsi: usize, %rdx: extern "sysv64" fn(usize, usize), %rcx: *mut u8)
rust_psm_on_stack PROC FRAME
    push rbp
    .pushreg rbp
    mov rbp, rsp
    .setframe rbp, 0
    .endprolog
    mov rsp, rcx
    call rdx
    mov rsp, rbp
    pop rbp
    ret
rust_psm_on_stack ENDP

_TEXT ENDS

END
