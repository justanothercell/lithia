###### [Booklet](README.md)
# Type Casting and Conversion

## Syntax
```rust
let x: T1 = ...;
let y: T2 = x as T2;
```

## Cast Table
- `A` automatic
- `UA` unsafe automatic
- `C` cast
- `UC` unsafe cast
- `-` disallowed

Notes: 
- The table assumes a conversion from row (y) to column (x) <br>
ex.: `table[2][1]` means "from Int to Float"
- Casting at `A` results in compiler error
- `bool` counts as `Int` for this table
- All casts assume same size. Use RawPtr to get around that
- Ref and deref is not looked at in this table, as those have a different functionality than just "interpretation as different type"
- A == B is always `A`
- The diagonals assume "T of Foo" and "T of Bar" where Sizeof(Foo) == Sizeof(Bar) and Foo != Bar.
The `...` means to look at the rules of the "inner" type
- `SameSize` overrules `-`

|          | Int | Float | Ptr | RawPtr | Array | Slice | SameSize |
|----------|-----|-------|-----|--------|-------|-------|----------|
| Int      | C   | C     | UC  | UC     | -     | -     | UC       |
| Float    | C   | C     | -   | -      | -     | -     | UC       |
| Ptr      | UC  | -     | ... | A      | -     | -     | -        |
| RawPtr   | UC  | -     | UC  | -      | -     | -     | -        |
| Array    | -   | -     | -   | -      | ...   | A     | UC       |
| Slice    | -   | -     | -   | -      | UC    | ...   | UC       |
| SameSize | UC  | UC    | -   | -      | UC    | UC    | UC       |

