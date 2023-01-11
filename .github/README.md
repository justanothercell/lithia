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
- casts:
  - cast between num types is safe
  - cast from ptr to raw ptr is safe
  - cast from raw ptr to ptr is unsafe
  - cast from slice to array is unsafe
  - cast from ptr/raw ptr to uptr is unsafe
  - all other casts are forbidden (atm)
- after casts:
  - () in exprs
  - ops in exprs

## Resources
- https://alive2.llvm.org/ce/ llvm ir analyzer

## Topics
#### [Structure & Design](./structure_design.md)
#### [Legacy Projects](./legacy.md) including Lithia-C#.net
