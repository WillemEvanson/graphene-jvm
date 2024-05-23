use cesu8_str::java::JavaStr;

use crate::{ConstantIdx, ConstantPool};

#[derive(Debug)]
pub struct Class {
    constants: ConstantPool,
    this_class: ConstantIdx,
    super_class: Option<ConstantIdx>,
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
}
