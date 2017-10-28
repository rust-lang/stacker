#include <windows.h>

void __stacker_black_box() {}

PVOID __stacker_get_current_fiber() {
    return GetCurrentFiber();
}

static size_t calc_stack_limit(size_t stack_low, size_t stack_guarantee) {
    return stack_low +
           max(stack_guarantee, sizeof(void *) == 4 ? 0x1000 : 0x2000) + // The guaranteed pages on a stack overflow 
           0x1000; // The guard page
}

#if defined(_M_X64)
size_t __stacker_get_stack_limit() {
    return calc_stack_limit(__readgsqword(0x1478), // The base address of the stack. Referenced in GetCurrentThreadStackLimits
                            __readgsqword(0x1748)); // The guaranteed pages on a stack overflow. Referenced in SetThreadStackGuarantee
}
#endif

#ifdef _M_IX86
size_t __stacker_get_stack_limit() {
    return calc_stack_limit(__readfsdword(0xE0C), // The base address of the stack. Referenced in GetCurrentThreadStackLimits
                            __readfsdword(0xF78)); // The guaranteed pages on a stack overflow. Referenced in SetThreadStackGuarantee
}
#endif