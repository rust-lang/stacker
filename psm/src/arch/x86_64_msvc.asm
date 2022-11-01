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

; extern "sysv64" fn(%rdi: usize, %rsi: extern "sysv64" fn(usize), %rdx: *mut u8, %rcx: *mut u8)
rust_psm_replace_stack PROC
    mov qword ptr gs:[00h], -1
    mov qword ptr gs:[10h], rcx
    mov qword ptr gs:[08h], rdx
    mov qword ptr gs:[1478h], rcx
    mov qword ptr gs:[1748h], 0

    lea rsp, [rdx - 8]
    jmp rsi
    ud2
rust_psm_replace_stack ENDP

; extern "sysv64" fn(%rdi: usize, %rsi: usize,
;                    %rdx: extern "sysv64" fn(usize, usize), %rcx: *mut u8, %r8: *mut u8)
;
; NB: on Windows for SEH to work at all, the pointers in TIB, thread information block, need to be
; fixed up. Otherwise, it seems that exception mechanism on Windows will not bother looking for
; exception handlers at *all* if they happen to fall outside the are specified in TIB.
;
; This necessitates an API difference from the usual 4-argument signature used elsewhere.
;
; FIXME: this needs a catch-all exception handler that aborts in case anything unwinds into here.
rust_psm_on_stack PROC FRAME
    push qword ptr rbp
    .pushreg rbp
    push qword ptr gs:[1748h] ; GuaranteedStackBytes
    .allocstack 8
    push qword ptr gs:[1478h] ; DeallocationStack
    .allocstack 8
    push qword ptr gs:[10h]   ; StackLimit
    .allocstack 8
    push qword ptr gs:[08h]   ; StackBase
    .allocstack 8
    push qword ptr gs:[00h]   ; ExceptionList
    .allocstack 8
    mov rbp, rsp
    .setframe rbp, 0
    .endprolog

    mov rsp, rcx

    mov qword ptr gs:[00h], -1
    mov qword ptr gs:[08h], rcx
    mov qword ptr gs:[10h], r8
    ; TODO: these need a more proper handling
    mov qword ptr gs:[1478h], rcx
    mov qword ptr gs:[1748h], 0

    call rdx

    mov rsp, rbp
    pop qword ptr gs:[00h]
    pop qword ptr gs:[08h]
    pop qword ptr gs:[10h]
    pop qword ptr gs:[1478h]
    pop qword ptr gs:[1748h]
    pop qword ptr rbp
    ret
rust_psm_on_stack ENDP

_TEXT ENDS

END
