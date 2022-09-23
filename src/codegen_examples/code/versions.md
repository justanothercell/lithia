# Syntax Version Index (SVI)

The syntax is derived from Rust's syntax, changes applied sequentially 
from top to bottom across versions. A new version is made when incompatible 
changes are made ot the syntax.

### v1 (*.li)
- rust style variables (but no mut kwd)
- mandatory explicit type declaration
- while loop and if
- only explicit operations, ie. i32::add instead of just "+"