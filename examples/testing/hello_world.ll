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
  %msg2 = alloca [8 x i8], align 1
  store [8 x i8] c"message\00", ptr %msg2, align 1
  %0 = alloca [8 x i8], align 1
  store [8 x i8] c"message\00", ptr %0, align 1
  %msg3 = alloca ptr, align 8
  store ptr %0, ptr %msg3, align 8
  %1 = alloca [8 x i8], align 1
  store [8 x i8] c"message\00", ptr %1, align 1
  %2 = call i32 @puts(ptr %1)
  %3 = load ptr, ptr %msg3, align 8
  %4 = call i32 @puts(ptr %3)
  %5 = alloca [8 x i8], align 1
  store ptr @FOO, ptr %5, align 8
  %6 = alloca ptr, align 8
  store ptr %5, ptr %6, align 8
  %7 = alloca ptr, align 8
  store ptr %6, ptr %7, align 8
  %8 = alloca ptr, align 8
  store ptr %7, ptr %8, align 8
  %9 = load ptr, ptr %8, align 8
  %10 = load ptr, ptr %9, align 8
  %11 = load ptr, ptr %10, align 8
  %12 = load ptr, ptr %11, align 8
  %13 = alloca ptr, align 8
  store ptr %12, ptr %13, align 8
  %14 = load ptr, ptr %13, align 8
  %15 = call i32 @puts(ptr %14)
  ret void
}
