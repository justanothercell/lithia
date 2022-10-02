use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) struct Ident(pub(crate) String);

#[derive(Debug, Clone)]
pub(crate) struct Arg(pub(crate) Ident, pub(crate) Type);

#[derive(Clone)]
pub(crate) struct VarObject {
    obj: Box<dyn VMObject>
}


impl VarObject {
    pub(crate) fn new<T: VMObject>(obj: T) -> Self{
        VarObject { obj: Box::from(obj) }
    }

    pub(crate) fn to<T: VMObject>(self) -> T {
        Self::vmo_to(self.obj)
    }

    pub(crate) fn vmo_to<T: VMObject>(vmo: Box<dyn VMObject>) -> T {
        let res = (vmo as Box<dyn Any>).downcast();
        let box b = res.unwrap();
        b
    }
}


pub(crate) trait VMObject: Any {
    fn box_clone(&self) -> Box<dyn VMObject>;
    fn debug(&self) -> String;
}

impl Clone for Box<dyn VMObject>{
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl Debug for VarObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "VarObject({})", self.obj.debug())
    }
}

#[repr(u8)]
#[derive(Debug, Clone)]
pub(crate) enum Type {
    Empty,

    I8,
    I16,
    I32,
    I64,
    I128,

    U8,
    U16,
    U32,
    U64,
    U128,

    F32,
    F64,

    Bool,
    String,

    Tuple(Vec<Type>),
    Struct(HashMap<Ident, Type>),
    Enum(HashMap<Ident, Type>),
    Fn(Vec<Type>, Vec<Type>),
    Object
}

impl Type {
    pub(crate) fn variant(&self) -> u8{
        match &self {
            Type::Empty => 0u8,
            Type::I8 => 1u8,
            Type::I16 => 2u8,
            Type::I32 => 3u8,
            Type::I64 => 4u8,
            Type::I128 => 5u8,
            Type::U8 => 6u8,
            Type::U16 => 7u8,
            Type::U32 => 8u8,
            Type::U64 => 9u8,
            Type::U128 => 10u8,
            Type::F32 => 11u8,
            Type::F64 => 12u8,
            Type::Bool => 13u8,
            Type::String => 14u8,
            Type::Tuple(_) => 15u8,
            Type::Struct(_) => 16u8,
            Type::Enum(_) => 17u8,
            Type::Fn(_, _) => 18u8,
            Type::Object => 19u8,
        }
    }

    pub(crate) fn u8(self) -> Vec<u8> {
        let mut bin_t = vec![self.variant()];
        match self {
            Type::Tuple(types) => {
                bin_t.append(&mut Type::uint_u8(types.len() as u128, 4));
                for t in types {
                    bin_t.append(&mut t.u8())
                }
            },
            Type::Struct(members) => {
                bin_t.append(&mut Type::uint_u8(members.len() as u128, 4));
                for (k, v) in members {
                    bin_t.append(&mut Type::str_u8(&k.0));
                    bin_t.append(&mut v.u8());
                }
            },
            Type::Enum(variants) => {
                bin_t.append(&mut Type::uint_u8(variants.len() as u128, 4));
                for (k, v) in variants {
                    bin_t.append(&mut Type::str_u8(&k.0));
                    bin_t.append(&mut v.u8());
                }
            }
            Type::Fn(args, ret) => {
                bin_t.append(&mut Type::uint_u8(args.len() as u128, 4));
                for a in args {
                    bin_t.append(&mut a.u8())
                }
                bin_t.append(&mut Type::uint_u8(ret.len() as u128, 4));
                for r in ret {
                    bin_t.append(&mut r.u8())
                }
            }
            _ => ()
        }

        bin_t
    }

    pub(crate) fn from_u8(s: &[u8]) -> (Type, usize) {
        match s[0] {
            0 => (Type::Empty, 1),
            1 => (Type::I8, 1),
            2 => (Type::I16, 1),
            3 => (Type::I32, 1),
            4 => (Type::I64, 1),
            5 => (Type::I128, 1),
            6 => (Type::U8, 1),
            7 => (Type::U16, 1),
            8 => (Type::U32, 1),
            9 => (Type::U64, 1),
            10 => (Type::U128, 1),
            11 => (Type::F32, 1),
            12 => (Type::F64, 1),
            13 => (Type::Bool, 1),
            14 => (Type::String, 1),
            15 => {
                let mut o = 5;
                let mut t = vec![];
                for _ in 0..Type::u8_uint(&s[1..], 4).0 as usize {
                    let (r, i) = Type::from_u8(&s[o..]);
                    o += i;
                    t.push(r);
                }
                (Type::Tuple(t), o)
            },
            16 => {
                let mut o = 5;
                let mut t = HashMap::new();
                for _ in 0..Type::u8_uint(&s[1..], 4).0 as usize {
                    let (k, i) = Type::u8_str(&s[o..]);
                    o += i;
                    let (v, i) = Type::from_u8(&s[o..]);
                    o += i;
                    t.insert(Ident(k), v);
                }
                (Type::Struct(t), o)
            },
            17 => {
                let mut o = 5;
                let mut t = HashMap::new();
                for _ in 0..Type::u8_uint(&s[1..], 4).0 as usize {
                    let (k, i) = Type::u8_str(&s[o..]);
                    o += i;
                    let (v, i) = Type::from_u8(&s[o..]);
                    o += i;
                    t.insert(Ident(k), v);
                }
                (Type::Enum(t), o)
            },
            18 => {
                let mut o = 1 + 4;
                let mut t = vec![];
                for _ in 0..Type::u8_uint(&s[1..], 4).0 as usize {
                    let (tt, i) = Type::from_u8(&s[o..]);
                    o += i;
                    t.push(tt);
                }
                o += 4;
                let mut r = vec![];
                for _ in 0..Type::u8_uint(&s[(o - 4)..], 4).0 as usize {
                    let (rr, i) = Type::from_u8(&s[o..]);
                    o += i;
                    r.push(rr);
                }
                (Type::Fn(t, r), o)
            },
            19 => (Type::Object, 1),
            invalid => unreachable!("This match case is an invalid type: {}", invalid)
        }
    }

    pub(crate) fn uint_u8(uint: u128, size: u8) -> Vec<u8>{
        uint.to_le_bytes()[0..size as usize].to_vec()
    }

    pub(crate) fn u8_uint(s: &[u8], size: u8) -> (u128, usize){
        let mut arr = [0u8; 16];
        let mut m = s[0..size as usize].to_vec();
        m.extend_from_slice(&arr[size as usize..16]);
        arr.copy_from_slice(&m);
        (u128::from_le_bytes(arr), size as usize)
    }

    pub(crate) fn int_u8(uint: i128, size: u8) -> Vec<u8>{
        uint.to_le_bytes()[0..size as usize].to_vec()
    }

    pub(crate) fn u8_int(s: &[u8], size: u8) -> (i128, usize){
        let mut arr = [0u8; 16];
        let mut m = s[0..size as usize].to_vec();
        m.extend_from_slice(&arr[size as usize..16]);
        arr.copy_from_slice(&m);
        (i128::from_le_bytes(arr), size as usize)
    }

    pub(crate) fn float_u8(uint: f64, size: u8) -> Vec<u8>{
        uint.to_le_bytes()[0..size as usize].to_vec()
    }

    pub(crate) fn u8_float(s: &[u8], size: u8) -> (f64, usize){
        let mut arr = [0u8; 8];
        let mut m = s[0..size as usize].to_vec();
        m.extend_from_slice(&arr[size as usize..8]);
        arr.copy_from_slice(&m);
        (f64::from_le_bytes(arr), size as usize)
    }

    pub(crate) fn str_u8(string: &str) -> Vec<u8>{
        let mut s: Vec<u8> = Type::uint_u8(string.len() as u128, 4);
        s.append(&mut string.as_bytes().to_vec());
        s
    }

    pub(crate) fn u8_str(s: &[u8]) -> (String, usize){
        if s.len() > 0 {
            let len = Type::u8_uint(s, 4).0 as usize;
            (String::from_utf8(s[4..(len + 4)].to_vec()).expect("Failed to create String"), len + 4)
        }
        else {
            (String::new(), 0)
        }
    }
}

#[derive(Clone)]
pub(crate) enum Value {
    Empty,

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    F32(f32),
    F64(f64),

    Bool(bool),
    String(String),

    Tuple(Vec<Value>),
    Struct(HashMap<Ident, Value>),
    Enum(HashMap<Ident, Value>),
    Fn(fn(Vec<Value>) -> Vec<Value>, Vec<Type>, Vec<Type>),
    Object(VarObject)
}

impl Value {
    pub(crate) fn get_type(&self) -> Type{
        match self {
            Value::Empty => Type::Empty,

            Value::I8(_) => Type::I8,
            Value::I16(_) => Type::I16,
            Value::I32(_) => Type::I32,
            Value::I64(_) => Type::I64,
            Value::I128(_) => Type::I128,

            Value::U8(_) => Type::U8,
            Value::U16(_) => Type::U16,
            Value::U32(_) => Type::U32,
            Value::U64(_) => Type::U64,
            Value::U128(_) => Type::U128,

            Value::F32(_) => Type::F32,
            Value::F64(_) => Type::F64,

            Value::Bool(_) => Type::Bool,
            Value::String(_) => Type::String,

            Value::Tuple(vals) => Type::Tuple(vals.iter().map(|v| v.get_type()).collect()),
            Value::Struct(fields) => Type::Struct(fields.iter().map(|(k, v)| (k.clone(), v.get_type())).collect()),
            Value::Enum(variants) => Type::Enum(variants.iter().map(|(k, v)| (k.clone(), v.get_type())).collect()),
            Value::Fn(_fn, args, ret) => Type::Fn(args.clone(), ret.clone()),
            Value::Object(_) => Type::Object
        }
    }

    pub(crate) fn u8(self) -> Vec<u8>{
        let mut s = vec![];
        let t = self.get_type();
        s.push(t.variant());
        s.append(&mut match self {
            Value::Empty => vec![],
            Value::I8(i8) => Type::int_u8(i8 as i128, 1),
            Value::I16(i16) => Type::int_u8(i16 as i128, 2),
            Value::I32(i32) => Type::int_u8(i32 as i128, 4),
            Value::I64(i64) => Type::int_u8(i64 as i128, 8),
            Value::I128(i128) => Type::int_u8(i128, 16),
            Value::U8(u8) => Type::uint_u8(u8 as u128, 1),
            Value::U16(u16) => Type::uint_u8(u16 as u128, 2),
            Value::U32(u32) => Type::uint_u8(u32 as u128, 4),
            Value::U64(u64) => Type::uint_u8(u64 as u128, 8),
            Value::U128(u128) => Type::uint_u8(u128, 16),
            Value::F32(f32) => Type::float_u8(f32 as f64, 4),
            Value::F64(f64) => Type::float_u8(f64, 8),
            Value::Bool(bool) => if bool { vec![1] } else { vec![0] },
            Value::String(string) => Type::str_u8(&*string),
            Value::Tuple(_) => unimplemented!(),
            Value::Struct(_) => unimplemented!(),
            Value::Enum(_) => unimplemented!(),
            Value::Fn(_, _, _) => panic!("Unable to express function as a literal value and therefore cannot parse from Value::Fn to &[u8]"),
            Value::Object(_) => panic!("Unable to express type as a literal value and therefore cannot parse from Value::Fn to &[u8]"),
        });
        s
    }

    pub(crate) fn from_u8(s: &[u8]) -> (Value, usize){
        match s[0] {
            //Empty
            0 => (Value::Empty, 1),
            //I8(i8)
            1 => (Value::I8(Type::u8_int(&s[1..], 1).0 as i8), 1+1),
            //I16(i16)
            2 => (Value::I16(Type::u8_int(&s[1..], 2).0 as i16), 2+1),
            //I32(i32)
            3 => (Value::I32(Type::u8_int(&s[1..], 4).0 as i32), 4+1),
            //I64(i64)
            4 => (Value::I64(Type::u8_int(&s[1..], 8).0 as i64), 8+1),
            //I128(i128)
            5 => (Value::I128(Type::u8_int(&s[1..], 16).0), 16+1),
            //U8(u8)
            6 => (Value::U8(Type::u8_uint(&s[1..], 1).0 as u8), 1+1),
            //U16(u16)
            7 => (Value::U16(Type::u8_uint(&s[1..], 2).0 as u16), 2+1),
            //U32(u32)
            8 => (Value::U32(Type::u8_uint(&s[1..], 41).0 as u32), 4+1),
            //U64(u64)
            9 => (Value::U64(Type::u8_uint(&s[1..], 8).0 as u64), 8+1),
            //U128(u128)
            10 => (Value::U128(Type::u8_uint(&s[1..], 16).0), 16+1),
            //F32(f32)
            11 => (Value::F32(Type::u8_float(&s[1..], 8).0 as f32), 4+1),
            //F64(f64)
            12 => (Value::F64(Type::u8_float(&s[1..], 16).0), 4+1),
            //Bool(bool)
            13 => (Value::Bool(if s[1] == 0 { false} else { true }), 1+1),
            //String(String)
            14 => {
                let (str, i) = Type::u8_str(&s[1..]);
                (Value::String(str), i + 1)
            }
            //Tuple(Vec<Value>)
            15 => unimplemented!(),
            //Struct(HashMap<Ident, Value>)
            16 => unimplemented!(),
            //Enum(HashMap<Ident, Value>)
            17 => unimplemented!(),
            //Fn(fn(&[Value]) -> Value, Vec<Type>, Type)
            18 => panic!("Unable to express function as a literal value and therefore cannot parse from &[u8] to Value::Fn"),
            invalid => unreachable!("This variable type does not exist: {}", invalid)
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Value::I8(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I8", &__self_0)
            }
            Value::I16(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I16", &__self_0)
            }
            Value::I32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I32", &__self_0)
            }
            Value::I64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I64", &__self_0)
            }
            Value::I128(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "I128", &__self_0)
            }
            Value::U8(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U8", &__self_0)
            }
            Value::U16(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U16", &__self_0)
            }
            Value::U32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U32", &__self_0)
            }
            Value::U64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U64", &__self_0)
            }
            Value::U128(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "U128", &__self_0)
            }
            Value::F32(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "F32", &__self_0)
            }
            Value::F64(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "F64", &__self_0)
            }
            Value::Bool(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Bool", &__self_0)
            }
            Value::String(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "String", &__self_0)
            }
            Value::Empty => ::core::fmt::Formatter::write_str(f, "Empty"),
            Value::Tuple(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Tuple", &__self_0)
            }
            Value::Struct(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Struct", &__self_0)
            }
            Value::Enum(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Enum", &__self_0)
            }
            Value::Fn(__self_0, __self_1, __self_2) => {
                ::core::fmt::Formatter::debug_tuple_field2_finish(
                    f, "Fn", &__self_1, &__self_2,
                )
            }
            Value::Object(__self_0) => {
                ::core::fmt::Formatter::debug_struct(
                    f, &*format!("{:?}", &__self_0),
                ).finish()
            }
        }
    }
}
