use crate::string::JavaStr;

use super::{ConstantIdx, ConstantPool};
use super::{Field, Method};

pub struct Class {
    pub(super) constants: ConstantPool,
    pub(super) this_class: ConstantIdx,
    pub(super) super_class: Option<ConstantIdx>,
    pub(super) fields: Vec<Field>,
    pub(super) methods: Vec<Method>,
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

    pub fn get_method(&self, name: &JavaStr, descriptor: &JavaStr) -> &Method {
        self.methods
            .iter()
            .find(|method| {
                method.name(&self.constants) == name
                    && method.descriptor(&self.constants) == descriptor
            })
            .unwrap()
    }
}

impl std::fmt::Debug for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Class")
            .field("constants", &self.constants)
            .field("this_class", {
                let class = self.constants.get(self.this_class).into_class();
                &self.constants.get(class).into_utf8()
            })
            .field_with("super_class", |f| {
                if let Some(super_class) = self.super_class {
                    let class = self.constants.get(super_class).into_class();
                    self.constants.get(class).into_utf8().fmt(f)
                } else {
                    f.write_str("None")
                }
            })
            .field_with("fields", |f| {
                let mut debug_list = f.debug_list();
                for field in self.fields.iter() {
                    debug_list.entry_with(|f| {
                        f.debug_struct("Field")
                            .field("name", &field.name(&self.constants))
                            .field("descriptor", &field.descriptor(&self.constants))
                            .finish()
                    });
                }
                debug_list.finish()
            })
            .field_with("methods", |f| {
                let mut debug_list = f.debug_list();
                for method in self.methods.iter() {
                    debug_list.entry_with(|f| {
                        f.debug_struct("Method")
                            .field("name", &method.name(&self.constants))
                            .field("descriptor", &method.descriptor(&self.constants))
                            .field_with("flags", |f| std::fmt::Display::fmt(&method.flags, f))
                            .field_with("code", |f| {
                                if let Some(code) = &method.code {
                                    code.fmt(f)
                                } else {
                                    f.write_str("None")
                                }
                            })
                            .finish()
                    });
                }
                debug_list.finish()
            })
            .finish()
    }
}
