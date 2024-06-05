use crate::string::JavaStr;

use super::Bytecode;
use super::{ConstantIdx, ConstantPool};

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

    pub fn get_method(&self, name: &JavaStr) -> &Method {
        self.methods
            .iter()
            .find(|method| method.name(&self.constants) == name)
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
                            .field("name", &self.constants.get(field.name).into_utf8())
                            .field(
                                "descriptor",
                                &self.constants.get(field.descriptor).into_utf8(),
                            )
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
                            .field("name", &self.constants.get(method.name).into_utf8())
                            .field(
                                "descriptor",
                                &self.constants.get(method.descriptor).into_utf8(),
                            )
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

#[derive(Debug)]
pub struct Field {
    pub(super) name: ConstantIdx,
    pub(super) descriptor: ConstantIdx,
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
    pub(super) name: ConstantIdx,
    pub(super) descriptor: ConstantIdx,
    pub(super) flags: MethodFlags,
    pub(super) code: Option<Code>,
}

impl Method {
    pub fn name<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.name).into_utf8()
    }

    pub fn descriptor<'a>(&self, constant_pool: &'a ConstantPool) -> &'a JavaStr {
        constant_pool.get(self.descriptor).into_utf8()
    }

    pub fn flags(&self) -> MethodFlags {
        self.flags
    }

    pub fn bytecode(&self) -> Option<&Code> {
        self.code.as_ref()
    }
}

impl std::fmt::Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("name", &self.name)
            .field("descriptor", &self.descriptor)
            .field("flags", &self.flags)
            .field_with("code", |f| {
                if let Some(code) = &self.code {
                    code.fmt(f)
                } else {
                    f.write_str("None")
                }
            })
            .finish()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MethodFlags(u16);

impl MethodFlags {
    /// Declared public; may be accessed from outside its
    /// package.
    pub const PUBLIC: Self = Self(0x0001);
    /// Declared private; usable only within the defining
    /// class.
    pub const PRIVATE: Self = Self(0x0002);
    /// Declared protected; may be accessed within
    /// subclasses.
    pub const PROTECTED: Self = Self(0x0004);
    /// Declared static
    pub const STATIC: Self = Self(0x0008);
    /// Declared final; never directly assigned to after
    /// object construction.
    pub const FINAL: Self = Self(0x0010);
    /// Declared synchronized; invocation is wrapped by a
    /// monitor use.
    pub const SYNCHRONIZED: Self = Self(0x0020);
    /// A bridge method, generated by the compiler.
    pub const BRIDGE: Self = Self(0x0040);
    /// Declared with a variable number of arguments
    pub const VARARGS: Self = Self(0x0080);
    /// Declared native; implemented in a language other
    /// than Java.
    pub const NATIVE: Self = Self(0x0100);
    /// Declared abstract; no implementation is provided.
    pub const ABSTRACT: Self = Self(0x0400);
    /// Declared strictfp; floating-point mode is FP-strict.
    pub const STRICT: Self = Self(0x0800);
    /// Declared synthetic; not present in the source code.
    pub const SYNTHETIC: Self = Self(0x1000);

    /// Used for constructing a value during parsing. We do
    /// check bits here because unknown bits are to be
    /// ignored according to the specification.
    pub(super) const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    const FLAGS: [(MethodFlags, &'static str); 12] = [
        (MethodFlags::PUBLIC, "PUBLIC"),
        (MethodFlags::PRIVATE, "PRIVATE"),
        (MethodFlags::PROTECTED, "PROTECTED"),
        (MethodFlags::STATIC, "STATIC"),
        (MethodFlags::FINAL, "FINAL"),
        (MethodFlags::SYNCHRONIZED, "SYNCHRONIZED"),
        (MethodFlags::BRIDGE, "BRIDGE"),
        (MethodFlags::VARARGS, "VARARGS"),
        (MethodFlags::NATIVE, "NATIVE"),
        (MethodFlags::ABSTRACT, "ABSTRACT"),
        (MethodFlags::STRICT, "STRICT"),
        (MethodFlags::SYNTHETIC, "SYNTHETIC"),
    ];
}

impl std::ops::BitAnd<MethodFlags> for MethodFlags {
    type Output = MethodFlags;

    fn bitand(self, rhs: MethodFlags) -> Self::Output {
        MethodFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitAnd<MethodFlags> for &MethodFlags {
    type Output = MethodFlags;

    fn bitand(self, rhs: MethodFlags) -> Self::Output {
        MethodFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitOr<MethodFlags> for MethodFlags {
    type Output = MethodFlags;

    fn bitor(self, rhs: MethodFlags) -> Self::Output {
        MethodFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOr<MethodFlags> for &MethodFlags {
    type Output = MethodFlags;

    fn bitor(self, rhs: MethodFlags) -> Self::Output {
        MethodFlags(self.0 | rhs.0)
    }
}

impl std::fmt::Debug for MethodFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MethodFlags({})", self)
    }
}

impl std::fmt::Display for MethodFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for (flag, name) in Self::FLAGS {
            if self & flag == flag {
                if !first {
                    write!(f, " | {name}")?;
                } else {
                    write!(f, "{name}")?;
                    first = false;
                }
            }
        }
        Ok(())
    }
}

pub struct Code {
    pub max_stack: u16,
    pub max_locals: u16,
    pub bytecode: Vec<u8>,
}

impl Code {
    pub fn max_stack(&self) -> u16 {
        self.max_stack
    }

    pub fn max_locals(&self) -> u16 {
        self.max_locals
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode::new(self.bytecode.as_slice())
    }
}

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code")
            .field("max_stack", &self.max_stack)
            .field("max_locals", &self.max_locals)
            .field("bytecode", &self.bytecode())
            .finish()
    }
}
