use crate::{BinBuilder, JmpType};
use crate::variable::{Type, Value};

#[allow(dead_code)]
pub(crate) fn example() -> Vec<u8>{
    let mut builder = BinBuilder::new();

    // create variable ids
    let u32_add = builder.gen_var_id();
    let u32_to_string = builder.gen_var_id();
    let u32_eq = builder.gen_var_id();
    let string_join = builder.gen_var_id();
    let println = builder.gen_var_id();
    let a = builder.gen_var_id();
    let b = builder.gen_var_id();
    let res = builder.gen_var_id();

    // load extern functions
    builder.load_extern("u32::add", &u32_add, Type::Fn(vec![Type::U32, Type::U32], vec![Type::U32]));
    builder.load_extern("u32::to_string", &u32_to_string, Type::Fn(vec![Type::U32], vec![Type::String]));
    builder.load_extern("u32::eq", &u32_eq, Type::Fn(vec![Type::U32, Type::U32], vec![Type::Bool]));
    builder.load_extern("string::join", &string_join, Type::Fn(vec![Type::String, Type::String], vec![Type::String]));
    builder.load_extern("println", &println, Type::Fn(vec![Type::String], vec![]));

    // a = 42
    builder.set_var(&a, |builder| {
        builder.push_value(Value::U32(42))
    });
    // b = 69
    builder.set_var(&b, |builder| {
        builder.push_value(Value::U32(69))
    });
    // res = a + b
    builder.set_var(&res, |builder| {
        // a + b
        builder.call_function(&u32_add, |builder| {
            builder.push_variable(&a);
            builder.push_variable(&b);
        })
    });
    // println("The result is: " + res)
    builder.call_function(&println, |builder| {
        // "The result is: " + res
        builder.call_function(&string_join, |builder| {
            builder.call_function(&u32_to_string, |builder| {
                builder.push_variable(&res);
            });
            builder.push_value(Value::String("The result is: ".to_string()));
        })
    });

    let m_false = builder.gen_marker();
    let m_end = builder.gen_marker();
    builder.call_function(&u32_eq, |builder| {
        builder.push_variable(&res);
        builder.push_value(Value::U32(111))
    });
    builder.jump(JmpType::Unless, &m_false);
    builder.call_function(&println, |builder| {
        builder.push_value(Value::String("Result is equal to 111 :)".to_string()))
    });
    builder.jump(JmpType::Jmp, &m_end);
    builder.set_marker(&m_false);
    builder.call_function(&println, |builder| {
        builder.push_value(Value::String("Result is not equal to 111 :(".to_string()))
    });
    builder.set_marker(&m_end);

    builder.return_scope(|_builder| {});


    builder.build()
}