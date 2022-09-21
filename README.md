# Lithia 
A compiler and bytecode-vm written in Rust for lithia (still to be created).

Lithia will be inspired by Rust's amazing syntactical features (minus the lifetime and borrowing).

last-update counter of shame: `21/09/2022`<br>
(please excuse my inability to update readmes)

### What works right now
- Running bytecode on the vm
- Generating bytecode semi-manually (like assembler but with tools and in code, see [bytecode examples](src/bytecode_examples))

### Steps of compilation
- [ ] Converting the code into token/ast tree
- [ ] (Optional:) Simplify/Convert high level constructs to more primitive representation
- [ ] Type and variable checking the ast for validity
- [x] Converting ast to bytecode representation<br>
- [x] (also able to write bytecode manually, see [bytecode examples](src/bytecode_examples))
- [x] Writing bytecode to file

### Steps to run bytecode (mostly complete)
- [x] Load bytes into memmap
- [x] decoding bytes and running sequentially
- [ ] throwing error and aborting peacefully (in case of some fault), returning stack trace position instead of crashing <br>
  (using markers and a link to the actual source to generate stacktrace)

### Implementing progress
The implementation is done "backwards", aka starting at the vm and ending with the language,
to always have a runnable and testable version.
- [x] VM 
- [x] byte code builder
- [x] compiling ast to byte code
- [ ] type checking and analyzing ast for validity
- [ ] parsing code to ast

(the last two steps might be swappable)

### General goals:
I want to create a working (preferable at least semi-usable) language *without* the usage of 
any crates that specifically aid in compiler building or vm code execution. Other crates that only do 
general work, such as for example memmap, rand, chrono or cli crates are permitted.

### General todos:
- [ ] refining language and vm:
  - [ ] support for foreign types (for example "File" type) which do not have to eb hardcoded but can be included like extern functions
  - [ ] proper struct/data/custom data type support 
    - [ ] "get field" vm bytecode instruction (Word) for structs etc
    - [ ] builtin list/array type
    - [ ] generics
    - [ ] traits/interfaces (only ast side luckily, vm doesn't know of this at all)
- [ ] mapping stdlib (a bit at a time, whenever needed)
  - [ ] proper stdlib layout 
  - [x] basic operators for primitive types (+-*/ .to_string())
  - [ ] io
    - [x] println
    - [x] input
    - [ ] fileIO
    - [ ] merging all of the above to use common traits etc