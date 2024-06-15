use crate::string::JavaStr;

use super::{ConstantIdx, ConstantPool, FieldType};

pub struct Field {
    pub(super) name: ConstantIdx,
    pub(super) descriptor: ConstantIdx,
    pub(super) parsed_descriptor: FieldType,
}

impl Field {
    pub fn name<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.name).into_utf8()
    }

    pub fn descriptor<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.descriptor).into_utf8()
    }

    pub fn parsed_descriptor(&self) -> &FieldType {
        &self.parsed_descriptor
    }
}

impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("descriptor", &self.parsed_descriptor)
            .finish()
    }
}
