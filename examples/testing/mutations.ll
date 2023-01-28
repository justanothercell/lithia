; ModuleID = 'examples/testing/mutations.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

define void @main.1() {
entry:
  %0 = alloca i32, align 4
  store i32 0, ptr %0, align 4
  %1 = alloca [8 x i8], align 1
  store [8 x i8] c"x = %d\0A\00", ptr %1, align 1
  %2 = load i32, ptr %0, align 4
  %3 = call i32 (ptr, ...) @printf(ptr %1, i32 %2)
  store i32 5, ptr %0, align 4
  %4 = alloca [8 x i8], align 1
  store [8 x i8] c"x = %d\0A\00", ptr %4, align 1
  %5 = load i32, ptr %0, align 4
  %6 = call i32 (ptr, ...) @printf(ptr %4, i32 %5)
  %7 = load i32, ptr %0, align 4
  %8 = add i32 %7, 2
  store i32 %8, ptr %0, align 4
  %9 = alloca [8 x i8], align 1
  store [8 x i8] c"x = %d\0A\00", ptr %9, align 1
  %10 = load i32, ptr %0, align 4
  %11 = call i32 (ptr, ...) @printf(ptr %9, i32 %10)
  %12 = load i32, ptr %0, align 4
  %13 = add i32 %12, 3
  store i32 %13, ptr %0, align 4
  %14 = alloca [8 x i8], align 1
  store [8 x i8] c"x = %d\0A\00", ptr %14, align 1
  %15 = load i32, ptr %0, align 4
  %16 = call i32 (ptr, ...) @printf(ptr %14, i32 %15)
  ret void
}

declare i32 @printf(ptr, ...)
