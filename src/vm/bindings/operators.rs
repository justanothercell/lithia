use std::collections::HashMap;
use std::ops::*;
use crate::variable::{Type, Value};
use crate::vm::bindings::Function;

macro_rules! operator {
    ($identifier: expr, $variant: tt, $op_fn: expr, $res: tt) => {
        ($identifier.to_string(), Function(|args| {
            if let [Value::$variant(u1), Value::$variant(u2)] = args[0..2] {
                return vec![Value::$res($op_fn(u2, u1))];
            }
            panic!("Invalid args {:?} for {}", args, $identifier)
        }, vec![Type::$variant, Type::$variant], vec![Type::$res]))
    };
}

macro_rules! operator_borrow {
    ($identifier: expr, $variant: tt, $op_fn: expr, $res: tt) => {
        ($identifier.to_string(), Function(|args| {
            if let [Value::$variant(u1), Value::$variant(u2)] = args[0..2] {
                return vec![Value::$res($op_fn(&u2, &u1))];
            }
            panic!("Invalid args {:?} for {}", args, $identifier)
        }, vec![Type::$variant, Type::$variant], vec![Type::$res]))
    };
}

macro_rules! type_ops {
    ($identifier: ty, $variant: tt) => {
        [
            operator!(concat!(stringify!($identifier), "::add"), $variant, <$identifier>::add, $variant),
            operator!(concat!(stringify!($identifier), "::sub"), $variant, <$identifier>::sub, $variant),
            operator!(concat!(stringify!($identifier), "::mul"), $variant, <$identifier>::mul, $variant),
            operator!(concat!(stringify!($identifier), "::div"), $variant, <$identifier>::div, $variant),
            operator!(concat!(stringify!($identifier), "::rem"), $variant, <$identifier>::rem, $variant),
            operator!(concat!(stringify!($identifier), "::rem_euclid"), $variant, <$identifier>::rem_euclid, $variant),

            operator_borrow!(concat!(stringify!($identifier), "::eq"), $variant, <$identifier>::eq, Bool),
            operator_borrow!(concat!(stringify!($identifier), "::ne"), $variant, <$identifier>::ne, Bool),
            operator_borrow!(concat!(stringify!($identifier), "::lt"), $variant, <$identifier>::lt, Bool),
            operator_borrow!(concat!(stringify!($identifier), "::le"), $variant, <$identifier>::le, Bool),
            operator_borrow!(concat!(stringify!($identifier), "::gt"), $variant, <$identifier>::gt, Bool),
            operator_borrow!(concat!(stringify!($identifier), "::ge"), $variant, <$identifier>::ge, Bool),

            (concat!(stringify!($identifier), "::to_string").to_string(), Function(|args| {
                if let [Value::$variant(u1)] = args[0..1] {
                    return vec![Value::String(<$identifier>::to_string(&u1))];
                }
                panic!("Invalid args {:?} for {}::to_string", args, stringify!($identifier))
            }, vec![Type::$variant], vec![Type::String]))
        ]
    };
}

pub(crate) fn bindings() -> HashMap<String, Function> {
    let mut num_ops = HashMap::new();
    num_ops.extend(HashMap::from(type_ops!(u8, U8)));
    num_ops.extend(HashMap::from(type_ops!(u16, U16)));
    num_ops.extend(HashMap::from(type_ops!(u32, U32)));
    num_ops.extend(HashMap::from(type_ops!(u64, U64)));
    num_ops.extend(HashMap::from(type_ops!(u128, U128)));

    num_ops.extend(HashMap::from(type_ops!(i8, I8)));
    num_ops.extend(HashMap::from(type_ops!(i16, I16)));
    num_ops.extend(HashMap::from(type_ops!(i32, I32)));
    num_ops.extend(HashMap::from(type_ops!(i64, I64)));
    num_ops.extend(HashMap::from(type_ops!(i128, I128)));

    num_ops.extend(HashMap::from(type_ops!(f32, F32)));
    num_ops.extend(HashMap::from(type_ops!(f64, F64)));

    num_ops.extend(HashMap::from(
        [
            operator!("bool::and", Bool, bool::bitand, Bool),
            operator!("bool::or", Bool, bool::bitor, Bool),
            operator!("bool::bitxor", Bool, bool::bitxor, Bool),
            operator_borrow!("bool::eq", Bool, bool::eq, Bool),
            operator_borrow!("bool::ne", Bool, bool::ne, Bool),
            ("bool::not".to_string(), Function(|args| {
                if let [Value::Bool(u1)] = args[0..1] {
                    return vec![Value::Bool(bool::not(u1))];
                }
                panic!("Invalid args {:?} for bool::not", args)
            }, vec![Type::Bool], vec![Type::Bool])),
            ("bool::to_string".to_string(), Function(|args| {
                if let [Value::Bool(u1)] = args[0..1] {
                    return vec![Value::String(bool::to_string(&u1))];
                }
                panic!("Invalid args {:?} for bool::not", args)
            }, vec![Type::Bool], vec![Type::String]))
        ]
    ));
    num_ops.extend(HashMap::from(
        [
            ("string::eq".to_string(), Function(|args| {
                if let [Value::String(u1), Value::String(u2)] = &args[0..2] {
                    return vec![Value::Bool(String::eq(u1, u2))];
                }
                panic!("Invalid args {:?} for string::eq", args)
            }, vec![Type::String, Type::String], vec![Type::Bool])),
            ("string::ne".to_string(), Function(|args| {
                if let [Value::String(u1), Value::String(u2)] = &args[0..2] {
                    return vec![Value::Bool(String::ne(u1, u2))];
                }
                panic!("Invalid args {:?} for string::eq", args)
            }, vec![Type::String, Type::String], vec![Type::Bool])),
            ("string::join".to_string(), Function(|args| {
                if let [Value::String(u1), Value::String(u2)] = &args[0..2] {
                    let mut res = u2.clone();
                    res.push_str(u1);
                    return vec![Value::String(res)];
                }
                panic!("Invalid args {:?} for string::join", args)
            }, vec![Type::String, Type::String], vec![Type::String]))
        ]
    ));

    num_ops
}