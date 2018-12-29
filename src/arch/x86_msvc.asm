.386
.model flat

; WTF: PUBLIC conflicts with "SYSCALL" but "SYSCALL" is the only way to stop MASM from manging the
; symbol names?
;
; PUBLIC @rust_psm_stack_direction@0
; PUBLIC @rust_psm_stack_pointer@0
; PUBLIC @rust_psm_replace_stack@12
; PUBLIC @rust_psm_on_stack@16

_TEXT SEGMENT

; extern "fastcall" fn() -> u8 (%al)
@rust_psm_stack_direction@0 PROC SYSCALL
    mov al, 2
    ret
@rust_psm_stack_direction@0 ENDP

; extern "fastcall" fn() -> *mut u8 (%rax)
@rust_psm_stack_pointer@0 PROC SYSCALL
    lea eax, [esp + 4]
    ret
@rust_psm_stack_pointer@0 ENDP

; extern "fastcall" fn(%ecx: usize, %edx: extern "fastcall" fn(usize), 4(%esp): *mut u8)
@rust_psm_replace_stack@12 PROC SYSCALL
    mov esp, [esp + 4]
    jmp edx
@rust_psm_replace_stack@12 ENDP

; extern "fastcall" fn(%ecx: usize, %edx: usize, 4(%esp): extern "fastcall" fn(usize, usize), 8(%esp): *mut u8)
@rust_psm_on_stack@16 PROC SYSCALL FRAME
    push ebp
    mov ebp, esp
    mov esp, [ebp + 12]
    call dword ptr [ebp + 8]
    mov esp, ebp
    pop ebp
    ret 8
@rust_psm_on_stack@16 ENDP

END
