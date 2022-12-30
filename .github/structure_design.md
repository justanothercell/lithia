###### [README](./readme.md)

# Structure & Design

## Structure
these should be different modules:<br>
(not final names)
- lib (general traits and utility functions etc)
- input/source provider
- tokenizer
- astler
- llvm compiler (+clang call, maybe same or different module)

all these components will be called in series by the main compiler module

### Flow (in [compiler.rs](../src/compiler.rs))
(each step returns a Result, which is omitted in this diagram)
- `source::Source::from_file(file)` -> `Source`
- `tokens::tokenize(Source)` -> `Tokens`
- `ast::parse(Tokens)` -> `Ast` (probably `Module`)
- `llvm::build_llvmir(Ast)` -> LLVMModuleRef
- `llvm::build_exe(LLVMModuleRef)` -> fp: String

## Design
### Module directories: `foo/mod.rs` vs `foo.rs`
I will try - other than the previous times - a `mod.rs` approach since it seems
to make the root directory far less cluttered.
Regardless of whether this will change in the future, it will be handled uniformly
and not mixed between the two approaches
### `pub` vs `pub(crate)`
Everything will be pub(crate) or private initially