; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@WELCOME = global [0 x i8] c"Hello, worlds!\00"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

define void @main.1() {
entry:
  %0 = call i32 @puts(ptr @WELCOME)
  ret void
}
