# Lithia 
A compiler and bytecode-vm written in Rust for lithia (still to be created).

Lithia will be inspired by Rust's amazing syntactical features (minus the lifetime and borrowing).

last-update counter of shame: `02/10/2022`<br>
(please excuse my inability to update readmes consistently)

### What works right now
- [x] Running bytecode on the vm
- [x] Generating bytecode semi-manually (like assembler but with tools and in code, see [bytecode examples](src/bytecode_examples))
- [x] converting (some) code to tokens
- [x] converting (some) tokens to ast
- [x] Converting (almost all) ast to bytecode
- [x] Running the [simple examples](src/codegen_examples/code/v1)
  - [x] while loop
  - [x] if/else
  - [x] var declaration
  - [x] extern function calls
  - [x] assigning to variables
- [x] nice compiler error messages

### Performance
Time is an estimated average over a few runs. May be off by a small amount. 
All timings are relative and should not be compared across tables!

##### HashDict solution

| HashDict implementation   | time        |
|---------------------------|-------------|
| HashMap                   | 24.5s-~26s  |
| BTreeMap                  | 23.4s-~26s  |
| HashMap+BuildNoHashHasher | 22.1s-24.4s |
| ahash::AHashMap           | 21.3s-23.1s |

##### Word perf (latest)

| word       | calls   | total          | time/call |
|------------|---------|----------------|-----------|
| PushVar    | 2000001 |   1007140800ns |     503ns |
| Call       | 2000002 |   1859070200ns |     929ns |
| JumpUnless | 1000001 |    118427700ns |     118ns |
| SetVar     | 1000001 |    362540900ns |     362ns |
| Jump       | 1000000 |    332101600ns |     332ns |
| Extern     |       5 |        45300ns |    9060ns |
| Push       | 2000003 |    694174800ns |     347ns |

##### Call perf

| implementation                    | time per call |
|-----------------------------------|---------------|
| for loop (before)                 | 6420ns        |
| split_at+reverse                  | 6405ns        |
| split_at (reversed all functions) | 5919ns        |
| removed .to_vec after call        | 5100ns +      |


##### JumpUnless + JumpIf
Removed redundant read if not jumping: ~300ns to ~100ns on release

###### => call + jumps resulted in an improvement from 21s-23s to now 17.9s-19s

##### opt-level
0 to 1 => from 20s down to sub 5

### Big stuff that's missing and I don't want to put as a sub point everywhere:
(will reopen closed stuff when I actually get to implement these)
- [ ] own functions
- [ ] generics
- [ ] inline scopes
- [ ] return values from if/for
- [ ] continue/break/...
- [ ] literal operators (+/-*)
- [ ] typedef
- [ ] struct usage
- [ ] item accessor
- [ ] export or file linking
- [x] extern Object

### Steps of compilation
- [x] Converting the code into tokens
- [ ] Converting tokens into ast tree (wip)
  - [x] if
  - [ ] for
  - [x] while
  - [x] let x = ...;
  - [x] i = i + 1;
- [ ] Simplify/Convert high level constructs to more primitive representation
  - [ ] operator to function
  - [ ] inferred types
- [ ] Type and variable checking the ast for validity
- [x] Converting ast to bytecode representation
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
- [x] parsing code to ast (wip)
- [ ] type checking and analyzing ast for validity

(the last two steps might be swappable)

### General goals:
I want to create a working (preferable at least semi-usable) language *without* the usage of 
any crates that specifically aid in compiler building or vm code execution. Other crates that only do 
general work, such as for example memmap, rand, chrono or cli crates are permitted.

### General todos:
- [ ] refining language and vm:
  - [x] support for foreign types (for example "File" type) which do not have to be hardcoded but can be included like extern functions
  - [ ] proper struct/data/custom data type support 
    - [ ] "get field" vm bytecode instruction (Word) for structs etc
    - [ ] builtin list/array type
    - [ ] generics
    - [ ] traits/interfaces (only ast side luckily, vm doesn't know of this at all)
    - [x] "extern" type wrapper to use in extern functions
- [ ] mapping stdlib (a bit at a time, whenever needed)
  - [ ] proper stdlib layout 
  - [x] basic operators for primitive types (+-*/ .to_string())
  - [ ] io
    - [x] println
    - [x] input
    - [ ] fileIO
    - [ ] merging all of the above to use common "traits" (in lithia) etc