use crate::{BinBuilder, JmpType};
use crate::variable::Value::String;
use crate::variable::{Type, Value};

pub(crate) fn example() -> Vec<u8>{
    let mut builder = BinBuilder::new();

    // create variable ids
    let i32_add = builder.gen_var_id();
    let i32_to_string = builder.gen_var_id();
    let i32_lt = builder.gen_var_id();
    let string_join = builder.gen_var_id();
    let println = builder.gen_var_id();

    let i = builder.gen_var_id();

    // create markers
    let loop_start = builder.gen_marker();
    let loop_end = builder.gen_marker();

    // load extern functions
    builder.load_extern("i32::add", &i32_add, Type::Fn(vec![Type::I32, Type::I32], vec![Type::I32]));
    builder.load_extern("i32::to_string", &i32_to_string, Type::Fn(vec![Type::I32], vec![Type::String]));
    builder.load_extern("i32::lt", &i32_lt, Type::Fn(vec![Type::I32, Type::I32], vec![Type::Bool]));
    builder.load_extern("string::join", &string_join, Type::Fn(vec![Type::String, Type::String], vec![Type::String]));
    builder.load_extern("println", &println, Type::Fn(vec![Type::String], vec![]));

    // === for loop ===
    // i = 0
    builder.set_var(&i, |builder| {
        builder.push_value(Value::I32(0))
    });

    // actual loop
    builder.set_marker(&loop_start);
    builder.set_var(&i, |builder| {
        // condition: i < 10
        builder.call_function(&i32_lt, |builder| {
            builder.push_value(Value::I32(10));
            builder.push_variable(&i);
        });
        builder.jump(JmpType::Unless, &loop_end);

        // body:
        // println("Counting: " + i)
        builder.call_function(&println, |builder| {
            // "Counting: " + i
            builder.call_function(&string_join, |builder| {
                builder.call_function(&i32_to_string, |builder| {
                    builder.push_variable(&i)
                });
                builder.push_value(String("Counting: ".to_string()));
            });
        });

        // i += 1
        builder.call_function(&i32_add, |builder| {
            builder.push_variable(&i);
            builder.push_value(Value::I32(1))
        });
    });
    // return to top
    builder.jump(JmpType::Jmp, &loop_start);
    builder.set_marker(&loop_end);

    // println("Finished")
    builder.call_function(&println, |builder| {
        builder.push_value(String("Finished!".to_string()));
    });

    builder.return_scope(|_builder| {});

    builder.build()
}