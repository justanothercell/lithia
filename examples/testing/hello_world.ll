; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@WELCOME1 = global [16 x i8] c"Hello, worlds!1\00"
@MPTY = global [1 x i8] zeroinitializer
@INT = global i8 127
@WELCOME2 = global [16 x i8] c"Hello, worlds!2\00"
@WELCOME3 = global [16 x i8] c"Hello, worlds!3\00"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

define void @wants_int(i8 %0) {
entry:
  ret void
}

define void @main.1() {
entry:
  call void @wants_int_ptr(ptr @INT)
  %0 = alloca [16 x i8], align 1
  store [16 x i8] c"inline welcome2\00", ptr %0, align 1
  %1 = call i32 @puts(ptr %0)
  %2 = alloca [16 x i8], align 1
  store [16 x i8] c"inline welcome3\00", ptr %2, align 1
  %3 = alloca ptr, align 8
  store ptr %2, ptr %3, align 8
  %4 = call i32 @puts(ptr %3)
  %5 = call i32 @puts(ptr @MPTY)
  %6 = call i32 @puts(ptr @INT)
  %7 = call i32 @puts(ptr @WELCOME1)
  %8 = alloca [16 x i8], align 1
  store ptr @WELCOME2, ptr %8, align 8
  %9 = call i32 @puts(ptr %8)
  %10 = alloca [16 x i8], align 1
  store ptr @WELCOME3, ptr %10, align 8
  %11 = alloca ptr, align 8
  store ptr %10, ptr %11, align 8
  %12 = call i32 @puts(ptr %11)
  ret void
}

define void @wants_int_ptr(ptr %0) {
entry:
  ret void
}
