use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use crate::variable::{Type, Value};

pub(crate) fn bindings() -> HashMap<String, Value> {
    HashMap::from([
        ("print".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for print, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                print!("{}", u1);
                stdout().flush().expect("Error flushing stdout");
                return vec![];
            }
            panic!("Invalid args {:?} for print", args)
        }, vec![Type::String], vec![])),
        ("println".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for println, expected 1", args) }
            if let [Value::String(u1)] = &args[0..1] {
                println!("{}", u1);
                return vec![];
            }
            panic!("Invalid args {:?} for println", args)
        }, vec![Type::String], vec![])),
        ("input".to_string(), Value::Fn(|args| {
            if args.len() != 1 { panic!("Invalid number of args {:?} for println, expected 0", args) }
            if let [Value::String(u1)] = &args[0..1] {
                let mut s=String::new();
                print!("{}", u1);
                stdout().flush().expect("Error flushing stdout");
                stdin().read_line(&mut s).expect("Could not read stdin");
                if let Some('\n')=s.chars().next_back() {
                    s.pop();
                }
                if let Some('\r')=s.chars().next_back() {
                    s.pop();
                }
                return vec![Value::String(s)];
            }
            panic!("Invalid args {:?} for println", args)
        }, vec![Type::String], vec![Type::String]))
    ])
}