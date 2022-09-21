use crate::BinBuilder;
use crate::variable::{Type, Value};

pub(crate) fn example() -> Vec<u8>{
    let mut builder = BinBuilder::new();

    // create variable ids
    let string_join = builder.gen_var_id();
    let println = builder.gen_var_id();
    let input = builder.gen_var_id();

    builder.load_extern("string::join", &string_join, Type::Fn(vec![Type::String, Type::String], vec![Type::String]));
    builder.load_extern("println", &println, Type::Fn(vec![Type::String], vec![]));
    builder.load_extern("input", &input, Type::Fn(vec![Type::String], vec![Type::String]));

    // println("Hello " + input("Enter your name: ") + "!")
    builder.call_function(&println, |builder| {
        builder.call_function(&string_join, |builder| {
            builder.push_value(Value::String("!".to_string()));
            builder.call_function(&string_join, |builder| {
                builder.call_function(&input, |builder| {
                    builder.push_value(Value::String("Enter your name: ".to_string()));
                });
                builder.push_value(Value::String("Hello ".to_string()));
            })
        })
    });

    builder.return_scope(|_builder| {});

    builder.build()
}