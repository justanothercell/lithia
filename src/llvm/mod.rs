use std::collections::HashMap;
use crate::ast::Module;

pub(crate) struct LLVMCGenEnv {
    globals: HashMap<String, Module>,
    stack: Vec<(HashMap<String, Module>, Self::opaque)>
}

impl LLVMCGenEnv{
    #[allow(non_camel_case_types)]
    pub(crate) type opaque = bool;
}