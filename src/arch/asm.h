#if defined(APPLE)
#define GLOBAL(name) .globl _ ## name; _ ## name
#else
#define GLOBAL(name) .globl name; name
#endif
