; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@INT = global i8 -1
@WELCOME2 = global [16 x i8] c"Hello, worlds!2\00"
@MPTY = global [1 x i8] zeroinitializer
@WELCOME1 = global [16 x i8] c"Hello, worlds!1\00"
@WELCOME3 = global [16 x i8] c"Hello, worlds!3\00"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

define void @main.1() {
entry:
  %0 = alloca [16 x i8], align 1
  store [16 x i8] c"inline welcome2\00", ptr %0, align 1
  %1 = call i32 @puts(ptr %0)
  %2 = alloca [16 x i8], align 1
  store [16 x i8] c"inline welcome3\00", ptr %2, align 1
  %3 = alloca ptr, align 8
  store ptr %2, ptr %3, align 8
  %4 = call i32 @puts(ptr %3)
  %5 = call i32 @puts(ptr @MPTY)
  %6 = call i32 @puts(ptr @WELCOME1)
  %7 = alloca [16 x i8], align 1
  store ptr @WELCOME2, ptr %7, align 8
  %8 = call i32 @puts(ptr %7)
  %9 = alloca [16 x i8], align 1
  store ptr @WELCOME3, ptr %9, align 8
  %10 = alloca ptr, align 8
  store ptr %9, ptr %10, align 8
  %11 = call i32 @puts(ptr %10)
  ret void
}
