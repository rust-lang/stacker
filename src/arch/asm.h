#if defined(APPLE) || defined(WINDOWS)
#define GLOBAL(name) .globl _ ## name; _ ## name
#else
#define GLOBAL(name) .globl name; name
#endif
