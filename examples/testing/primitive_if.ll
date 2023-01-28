; ModuleID = 'examples/testing/primitive_if.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @printf(ptr, ...)

define void @main.1() {
entry:
  %0 = alloca [7 x i8], align 1
  store [7 x i8] c"r: %d\0A\00", ptr %0, align 1
  %1 = alloca i32, align 4
  br i1 false, label %then, label %else

then:                                             ; preds = %entry
  %2 = alloca [5 x i8], align 1
  store [5 x i8] c"gt!\0A\00", ptr %2, align 1
  %3 = call i32 (ptr, ...) @printf(ptr %2)
  store i32 42, ptr %1, align 4
  br label %ifcont

else:                                             ; preds = %entry
  %4 = alloca [5 x i8], align 1
  store [5 x i8] c"le!\0A\00", ptr %4, align 1
  %5 = call i32 (ptr, ...) @printf(ptr %4)
  store i32 69, ptr %1, align 4
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  %6 = load i32, ptr %1, align 4
  %7 = call i32 (ptr, ...) @printf(ptr %0, i32 %6)
  ret void
}
