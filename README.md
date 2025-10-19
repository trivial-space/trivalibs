# trivalibs

Shared Rust libraries used for graphics programming targeting WebAssembly and desktop.

## !IMPORTANT!

For the time beeing, because of a libm bug, use following command to build the shader crates:

```bash
cargo gpu build
# and
watchexec -r -e rs cargo gpu build
```
