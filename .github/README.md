# LITHIA
A compiled, Rust-inspired language using LLVM and written in Rust.

## How to use
1. Install LLVM 15 (or a compatible version). If you change the version, 
adjust `llvm-sys` in [Cargo.toml](../Cargo.toml)
You may need to compile LLVM by hand if the current releases 
for your platform are missing tools such as `llvm-config`.exe.<br>
2. Set the `LLVM_SYS_150_PREFIX` environment variable to the root directory of LLVM, or add LLVM to your `PATH`
3.Configure input files in [main](../src/main.rs) and run the compiler


## Topics
#### [Structure & Design](./structure_design.md)
#### [Legacy Projects](./legacy.md) including Lithia-C#.net
