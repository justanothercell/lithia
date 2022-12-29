###### [README](./readme.md)
# Legacy Projects
## Original lithia [[branch](https://github.com/DragonFIghter603/lithia/tree/archive-old-1)]
#### Reason For Discontinuation
- ast parser/tokenizer was not good enough
  - among others, enums didn't have common fields factored out into parent struct
  - lacked in flexibility
  - tokenizer differentiated too quickly into operators
- custom bytecode interpreter turned out to be not feasible, but worked well on simple things
#### Achievements
- creation of first parser and tokenizer
- nice error handling with error display
## Lithia C#.net [[repo](https://github.com/DragonFIghter603/lithia_csnet)]
#### Reason For Discontinuation
- LLVM seemed like a better option for compilation
  - name didn't fit anymore
  - fresh start to really remove all the minor inconveniences
note: look at [minimal_language]()
#### Achievements
- better, amazing ast parser (apart from a legacy induced redundant nesting)
- better, amazing ast