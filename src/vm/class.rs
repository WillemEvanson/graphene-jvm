use cesu8_str::java::JavaStr;

use super::Bytecode;
use super::{ConstantIdx, ConstantPool};

#[derive(Debug)]
pub struct Class {
    constants: ConstantPool,
    this_class: ConstantIdx,
    super_class: Option<ConstantIdx>,
    fields: Vec<Field>,
    methods: Vec<Method>,
}

impl Class {
    pub fn constants(&self) -> &ConstantPool {
        &self.constants
    }

    pub fn name(&self) -> &JavaStr {
        let name_idx = self.constants.get(self.this_class).into_class();
        self.constants.get(name_idx).into_utf8()
    }

    pub fn super_name(&self) -> Option<&JavaStr> {
        if let Some(super_class) = self.super_class {
            let name_idx = self.constants.get(super_class).into_class();
            Some(self.constants.get(name_idx).into_utf8())
        } else {
            None
        }
    }

    pub fn get_field(&self, name: &JavaStr) -> &Field {
        self.fields
            .iter()
            .find(|field| field.name(&self.constants) == name)
            .unwrap()
    }

    pub fn get_method(&self, name: &JavaStr) -> &Method {
        self.methods
            .iter()
            .find(|method| method.name(&self.constants) == name)
            .unwrap()
    }
}

#[derive(Debug)]
pub struct Field {
    name: ConstantIdx,
    descriptor: ConstantIdx,
}

impl Field {
    pub fn name<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.name).into_utf8()
    }

    pub fn descriptor<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.descriptor).into_utf8()
    }
}

pub struct Method {
    name: ConstantIdx,
    descriptor: ConstantIdx,

    bytecode: Option<Vec<u8>>,
}

impl Method {
    pub fn name<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.name).into_utf8()
    }

    pub fn descriptor<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.descriptor).into_utf8()
    }

    pub fn bytecode(&self) -> Option<Bytecode> {
        self.bytecode.as_deref().map(Bytecode::new)
    }
}

impl std::fmt::Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("name", &self.name)
            .field("descriptor", &self.descriptor)
            .field_with("bytecode", |f| {
                if let Some(bytecode) = self.bytecode() {
                    bytecode.fmt(f)
                } else {
                    f.write_str("None")
                }
            })
            .finish()
    }
}
