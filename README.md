# trivalibs

Shared Rust libraries used for graphics programming targeting WebAssembly and desktop.

## !IMPORTANT!

For the time beeing, because of a libm bug, use following command to build the shader crates:

```bash
cargo gpu build --spirv-builder-source "https://github.com/Rust-GPU/rust-gpu" --spirv-builder-version "2aa4d4f8a8ba73103501562cfca17b8163e5a887" --multimodule
# and
watchexec -r -e rs cargo gpu build --spirv-builder-source "https://github.com/Rust-GPU/rust-gpu" --spirv-builder-version "2aa4d4f8a8ba73103501562cfca17b8163e5a887" --multimodule
```
