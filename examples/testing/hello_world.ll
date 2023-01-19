; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

define void @main.1() {
entry:
  %string = alloca [6 x i8], align 1
  store [6 x i8] c"hello\00", ptr %string, align 1
  %c = call i8 @compact_char_at(ptr %string, i64 1)
  %0 = alloca [42 x i8], align 1
  store [42 x i8] c"char at index %d of \22%s\22: chr(%d) = '%c'\0A\00", ptr %0, align 1
  %1 = zext i8 %c to i32
  %2 = call i32 (ptr, ...) @printf(ptr %0, i64 1, ptr %string, i32 %1, i8 %c)
  ret void
}

define i8 @compact_char_at(ptr %0, i64 %1) {
entry:
  %2 = ptrtoint ptr %0 to i64
  %3 = add i64 %2, %1
  %4 = inttoptr i64 %3 to ptr
  %5 = load i8, ptr %4, align 1
  ret i8 %5
}

define i8 @char_at(ptr %0, i64 %1) {
entry:
  %start_ptr = ptrtoint ptr %0 to i64
  %idx_ptr = add i64 %start_ptr, %1
  %char = inttoptr i64 %idx_ptr to ptr
  %2 = load i8, ptr %char, align 1
  ret i8 %2
}

declare i32 @printf(ptr, ...)

declare i32 @puts(ptr)
