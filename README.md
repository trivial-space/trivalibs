# trivalibs

Shared Rust libraries used for graphics programming targeting WebAssembly and desktop.

## shader crates in trivalibs-painter

Shaders are compiled from Rust to rspirv shaders using [cargo-gpu](https://github.com/Rust-GPU/cargo-gpu).

To compile a shader crate, run `cargo gpu build` in the crate directory.

To watch compile, install `watchexec` and run `watchexec -r -e rs cargo gpu build` in the shader crate directory.
