; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

@FOO = global [8 x i8] c"message\00"

define void @main() {
entry:
  call void @main.1()
  ret void
}

declare i32 @puts(ptr)

define void @f([8 x i8] %0) {
entry:
  %1 = alloca [8 x i8], align 1
  store [8 x i8] %0, ptr %1, align 1
  %2 = call i32 @puts(ptr %1)
  ret void
}

define void @main.1() {
entry:
  call void @f([8 x i8] c"message\00")
  call void @f([8 x i8] c"message\00")
  %0 = load [8 x i8], ptr @FOO, align 1
  call void @f([8 x i8] %0)
  ret void
}
