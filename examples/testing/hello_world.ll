; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

define void @main.1() {
entry:
  %0 = alloca [6 x i8], align 1
  store [6 x i8] c"hello\00", ptr %0, align 1
  %c = call i8 @char_at(ptr %0, i64 1)
  %1 = alloca [4 x i8], align 1
  store [4 x i8] c"%d\0A\00", ptr %1, align 1
  %2 = call i32 (ptr, ...) @printf(ptr %1, i8 %c)
  ret void
}

declare i32 @printf(ptr, ...)

define i8 @char_at(ptr %0, i64 %1) {
entry:
  %ptr = ptrtoint ptr %0 to i64
  %char = inttoptr i64 %ptr to ptr
  %2 = load i8, ptr %char, align 1
  ret i8 %2
}

declare i32 @puts(ptr)
