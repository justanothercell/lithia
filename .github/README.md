# LITHIA
A statically compiled rust-inspired language compiled with LLVM 

### [Info](./legacy.md) about Lithia-csnet and other legacy projects

### Structure
these should be different modules:<br>
(not final names)
- input/source provider
- tokenizer
- astler
- llvm compiler (+clang call, maybe same or different module)

all these components will be called in series by the main compiler module