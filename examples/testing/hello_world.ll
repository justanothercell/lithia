; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@WELCOME_PTR2 = global ptr c"Hello, ptr!2\00"
@WELCOME1 = global [15 x i8] c"Hello, worlds!1\00"
@WELCOME3 = global [15 x i8] c"Hello, worlds!3\00"
@WELCOME_PTR3 = global ptr c"Hello, ptr!3\00"
@WELCOME_PTR1 = global ptr c"Hello, ptr!1\00"
@WELCOME2 = global [15 x i8] c"Hello, worlds!2\00"

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
  %5 = call i32 @puts(ptr @WELCOME1)
  %6 = alloca [15 x i8], align 1
  store ptr @WELCOME2, ptr %6, align 8
  %7 = call i32 @puts(ptr %6)
  %8 = alloca [15 x i8], align 1
  store ptr @WELCOME3, ptr %8, align 8
  %9 = alloca ptr, align 8
  store ptr %8, ptr %9, align 8
  %10 = call i32 @puts(ptr %9)
  %11 = call i32 @puts(ptr @WELCOME_PTR1)
  %12 = alloca ptr, align 8
  store ptr @WELCOME_PTR2, ptr %12, align 8
  %13 = call i32 @puts(ptr %12)
  %14 = alloca ptr, align 8
  store ptr @WELCOME_PTR3, ptr %14, align 8
  %15 = alloca ptr, align 8
  store ptr %14, ptr %15, align 8
  %16 = call i32 @puts(ptr %15)
  ret void
}
