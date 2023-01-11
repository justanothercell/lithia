; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@FOO = global [8 x i8] c"message\00"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

define void @main.1() {
entry:
  %0 = alloca [12 x i8], align 1
  store [12 x i8] c"hello %d %d\00", ptr %0, align 1
  %1 = call i32 (ptr, ...) @printf(ptr %0, i32 0, i32 0)
  ret void
}

declare i32 @printf(ptr, ...)
