; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@INT = global i8 127
@WELCOME = global [0 x i8] c"Hello, worlds!\00"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

define void @main.1() {
entry:
  call void @wants_int_ptr(ptr @INT)
  %0 = alloca [15 x i8], align 1
  store [15 x i8] c"inline welcome\00", ptr %0, align 1
  %1 = call i32 @puts(ptr %0)
  %2 = alloca [19 x i8], align 1
  store [19 x i8] c"inline welcome two\00", ptr %2, align 1
  %3 = alloca ptr, align 8
  store ptr %2, ptr %3, align 8
  %4 = call i32 @puts(ptr %3)
  %5 = call i32 @puts(ptr @WELCOME)
  %6 = alloca [0 x i8], align 1
  store ptr @WELCOME, ptr %6, align 8
  %7 = call i32 @puts(ptr %6)
  %8 = alloca [7 x i8], align 1
  store [7 x i8] c"ref!!!\00", ptr %8, align 1
  %9 = alloca ptr, align 8
  store ptr %8, ptr %9, align 8
  call void @puts_ref(ptr %9)
  call void @puts_hi()
  %10 = alloca [2 x i8], align 1
  store [2 x i8] c"a\00", ptr %10, align 1
  %11 = alloca [2 x i8], align 1
  store [2 x i8] c"b\00", ptr %11, align 1
  call void @puts_ab(ptr %10, ptr %11)
  ret void
}

define void @wants_int_ptr(ptr %0) {
entry:
  ret void
}

define void @puts_ref(ptr %0) {
entry:
  %1 = load ptr, ptr %0, align 8
  %2 = call i32 @puts(ptr %1)
  ret void
}

define void @puts_hi() {
entry:
  %0 = alloca [3 x i8], align 1
  store [3 x i8] c"hi\00", ptr %0, align 1
  %1 = call i32 @puts(ptr %0)
  ret void
}

define void @puts_ab(ptr %0, ptr %1) {
entry:
  %2 = call i32 @puts(ptr %0)
  %3 = call i32 @puts(ptr %1)
  ret void
}
