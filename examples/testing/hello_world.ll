; ModuleID = 'examples/testing/hello_world.bc'
source_filename = "main"

define void @main() {
entry:
  call void @main.1()
  ret void
}

define void @main.1() {
entry:
  ret void
}
