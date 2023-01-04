# LITHIA
A compiled rust-inspired language using LLVM written in rust

# Currently working on
- compiling main function with puts("hello, worlds"), only implementing the
necessary components

## How to use
- Install LLVM 15 (other versions are probably file aswell,
  you will just need to adjust [cargo.toml](cargo.toml) to use the matching
  version of `llvm-sys`.<br>
  You may need to compile LLVM by hand, as the current releases for windows lack some needed
  tools such as `llvm-config.exe`
- set environment variable `LLVM_SYS_150_PREFIX` (replace the 150 with your version) to the llvm root directory
  or make sure llvm is on PATH (the compiler will complain and will tell you which variable exactly
  needs to be set)
- configure input files etc. (most likely in [main](../src/main.rs))) and run


## Topics
#### [Structure & Design](./structure_design.md)
#### [Info](./legacy.md) about Lithia-C#.net and other legacy projects
