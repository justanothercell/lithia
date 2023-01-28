; ModuleID = 'examples/testing/fibonacci_recursive.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

define void @main.1() {
entry:
  call void @run_fib(i32 1)
  call void @run_fib(i32 2)
  call void @run_fib(i32 3)
  call void @run_fib(i32 4)
  call void @run_fib(i32 5)
  call void @run_fib(i32 6)
  call void @run_fib(i32 7)
  call void @run_fib(i32 8)
  call void @run_fib(i32 9)
  call void @run_fib(i32 10)
  call void @run_fib(i32 11)
  call void @run_fib(i32 12)
  ret void
}

declare i32 @printf(ptr, ...)

define void @run_fib(i32 %0) {
entry:
  %1 = alloca [14 x i8], align 1
  store [14 x i8] c"fib(%d) = %d\0A\00", ptr %1, align 1
  %2 = call i32 @fibonacci(i32 %0)
  %3 = call i32 (ptr, ...) @printf(ptr %1, i32 %0, i32 %2)
  ret void
}

define i32 @fibonacci(i32 %0) {
entry:
  %1 = icmp sle i32 %0, 2
  %2 = alloca i32, align 4
  br i1 %1, label %then, label %else

then:                                             ; preds = %entry
  store i32 1, ptr %2, align 4
  br label %ifcont

else:                                             ; preds = %entry
  %3 = sub i32 %0, 1
  %4 = call i32 @fibonacci(i32 %3)
  %5 = sub i32 %0, 2
  %6 = call i32 @fibonacci(i32 %5)
  %7 = add i32 %4, %6
  store i32 %7, ptr %2, align 4
  br label %ifcont

ifcont:                                           ; preds = %else, %then
  %8 = load i32, ptr %2, align 4
  ret i32 %8
}
