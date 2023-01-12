# LITHIA
A compiled, Rust-inspired language using LLVM and written in Rust.

## How to use
1. Install LLVM 15 (or a compatible version). If you change the version, 
adjust `llvm-sys` in [Cargo.toml](../Cargo.toml)
You may need to compile LLVM by hand if the current releases 
for your platform are missing tools such as `llvm-config`.exe.<br>
2. Set the `LLVM_SYS_150_PREFIX` environment variable to the root directory of LLVM, or add LLVM to your `PATH`
3.Configure input files in [main](../src/main.rs) and run the compiler

## Notes To Self For Future On Current Work (NTSFFOCW)
- casts:<br>
  - cast between num types (both float and int, and also bool) is safe (cause that's normal casts)
  - cast from ptr to raw ptr is implicit
  - cast from raw ptr to ptr is unsafe (cause information is not available)
  - cast from &array to &slice is implicit
  - cast from &slice to &array is unsafe (cause information is not available)
  - cast from ptr/raw ptr to uptr is unsafe (cause pointer operations are whacky)
  - cast from uptr to ptr/raw ptr is unsafe (cause pointer operations are whacky)
  - all other casts are forbidden (atm). later unsafe casts to anything with same size (prolly)
- after casts:
  - () in exprs
  - ops in exprs

## Resources
- https://alive2.llvm.org/ce/ llvm ir analyzer

## Topics
#### [Structure & Design](./structure_design.md)
#### [Legacy Projects](./legacy.md) including Lithia-C#.net
