###### [README](./readme.md)
# Legacy Projects


## Original lithia [[branch](https://github.com/DragonFIghter603/lithia/tree/archive-old-1)]

#### Reasons for Discontinuation
- The AST parser/tokenizer was not good enough:
  - Enums didn't have common fields factored out into a parent struct
  - Lacked flexibility
  - Tokenizer differentiated too quickly into operators
- The custom bytecode interpreter was not feasible, although it worked well on simple things#### Achievements

#### Key Accomplishments
- Creation of the first parser and tokenizer
- Good error handling with error display


## Lithia C#.net [[repo](https://github.com/DragonFIghter603/lithia_csnet)]

#### Reasons for Discontinuation
- LLVM seemed like a better option for compilation
  - The name didn't fit anymore
  - A fresh start to really remove all the minor inconveniences

Note: See [minimal_language]() for a small test project that uses LLVM.

#### Key Accomplishments
- A better AST parser (apart from a legacy-induced redundant nesting)