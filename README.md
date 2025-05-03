# trivalibs

Shared Rust libraries used for graphics programming targeting WebAssembly and desktop.

## !IMPORTANT!

For the time beeing, because of a libm bug, use following command to build the shader crates:

```bash
cargo gpu build --spirv-builder-source "https://github.com/Rust-GPU/rust-gpu" --spirv-builder-version "1e4e468ccf7965d90a9748c7513f72e852fb5041" --multimodule
# and
watchexec -r -e rs cargo gpu build --spirv-builder-source "https://github.com/Rust-GPU/rust-gpu" --spirv-builder-version "1e4e468ccf7965d90a9748c7513f72e852fb5041" --multimodule
```
