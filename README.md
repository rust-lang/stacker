# stacker

[![Build Status](https://travis-ci.com/alexcrichton/stacker.svg?branch=master)](https://travis-ci.com/alexcrichton/stacker)
[![Build status](https://ci.appveyor.com/api/projects/status/1yca9gp2bhe9h2by?svg=true)](https://ci.appveyor.com/project/alexcrichton/stacker)

[Documentation](https://docs.rs/stacker)

A stack-growth library for Rust. Enables annotating fixed points in programs
where the stack may want to grow larger. Spills over to the heap if the stack
has hit its limit.

This library is intended on helping implement recursive algorithms.

```toml
# Cargo.toml
[dependencies]
stacker = "0.1"
```

## Platform Support

This library currently is verified to work on the following platforms:

* 32/64 bit Linux
* 32/64 bit OSX
* 32/64 bit MinGW Windows
* 32/64 bit MSVC Windows
* wasm32-unknown-unknown (*)

On all other platforms this library is a noop. It should compile and run, but it
won't actually grow the stack and code will continue to hit the guard pages
typically in place.

(*) wasm32-unknown-unknown support isn't first class because the library only helps with
growing the shadow stack (i.e. stack implemented in terms of linear memory). Implementation
defined stacks (such as stacks for values, locals and call stacks) still can overflow.
Moreover, wasm doesn't provide a way to put guard pages so memory corruption
is possible. Use reasonable values for red zone size!

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
