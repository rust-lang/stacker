# wasm32

These functions are provided as a precompiled `libstacker.a`<sup>*</sup> file because support of wasm32 
target is not so widespread.

<sup>*</sup> - `libstacker.a` actually is an object file. We use `.a` suffix deliberately because otherwise linker will not consider `libstacker.o` as a linkable library.

This should work because `libstacker.a` doesn't contain any native code and rustc is using LLVM/LLD for generating wasm32-unknown-unknown binaries. However, the format of object files is not stable and future versions of LLD can fail to link to this archive. If this is the case try to re-run `./build.sh`.
