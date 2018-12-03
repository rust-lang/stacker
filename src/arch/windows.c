#include <windows.h>

void __stacker_black_box() {}

PVOID __stacker_get_current_fiber() {
    return GetCurrentFiber();
}
