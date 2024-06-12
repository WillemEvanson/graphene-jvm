use std::fmt::Write;

use crate::string::JavaString;

#[derive(Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub(super) parameters: Vec<FieldType>,
    pub(super) ret: Option<FieldType>,
}

impl MethodDescriptor {
    #[inline]
    #[must_use]
    pub fn args(&self) -> &[FieldType] {
        &self.parameters
    }

    #[inline]
    #[must_use]
    pub fn result(&self) -> Option<&FieldType> {
        self.ret.as_ref()
    }
}

impl std::fmt::Debug for MethodDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        std::fmt::Display::fmt(self, f)?;
        f.write_char('"')
    }
}

impl std::fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        for parameter in &self.parameters {
            std::fmt::Display::fmt(parameter, f)?;
        }
        f.write_char(')')?;

        if let Some(ret) = &self.ret {
            std::fmt::Display::fmt(ret, f)
        } else {
            f.write_char('V')
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum FieldType {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Char,
    Bool,
    Class(JavaString),
    Array(Box<FieldType>),
}

impl std::fmt::Debug for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        std::fmt::Display::fmt(self, f)?;
        f.write_char('"')
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte => f.write_char('B'),
            Self::Short => f.write_char('S'),
            Self::Int => f.write_char('I'),
            Self::Long => f.write_char('J'),
            Self::Float => f.write_char('F'),
            Self::Double => f.write_char('D'),
            Self::Char => f.write_char('C'),
            Self::Bool => f.write_char('Z'),
            Self::Class(string) => write!(f, "L{string};"),
            Self::Array(kind) => write!(f, "[{kind:?}"),
        }
    }
}
