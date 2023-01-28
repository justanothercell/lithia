; ModuleID = 'examples/testing/primitive_if.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

define void @println(ptr %0) {
entry:
  %1 = alloca [4 x i8], align 1
  store [4 x i8] c"%s\0A\00", ptr %1, align 1
  %2 = call i32 (ptr, ...) @printf(ptr %1, ptr %0)
  ret void
}

define void @main.1() {
entry:
  br i1 false, label %then, label %else

then:                                             ; preds = %entry
  %0 = alloca [3 x i8], align 1
  store [3 x i8] c"gt\00", ptr %0, align 1
  call void @println(ptr %0)
  br label %ifcont

else:                                             ; preds = %entry
  %1 = alloca [3 x i8], align 1
  store [3 x i8] c"le\00", ptr %1, align 1
  call void @println(ptr %1)
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  ret void
}

declare i32 @printf(ptr, ...)
