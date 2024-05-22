use std::num::NonZeroU16;

use cesu8_str::java::{JavaStr, JavaString};

/// The runtime constant pool for the JVM. This represents the various strings,
/// names, and other constants referred to within the class.
pub struct ConstantPool {
    entries: Vec<Slot>,
}

impl ConstantPool {
    /// Creates a new constant pool with the specificed `capacity`. This
    /// capacity limit is only taken as a lower bound currently, though this may
    /// be subject to change.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    /// Adds an entry to the constant pool. This method should only be used
    /// during construction of the constant pool.
    pub fn add(&mut self, entry: Entry) {
        match entry {
            Entry::Long(_) | Entry::Double(_) => {
                self.entries.push(Slot::Entry(entry));
                self.entries.push(Slot::Marker);
            }
            _ => self.entries.push(Slot::Entry(entry)),
        }
    }

    /// Gets an entry from the constant pool. This method deals with the 1-based
    /// indexing and the irregular sizes of the JVM's `long` and `double`
    /// constants.
    pub fn get(&self, index: ConstantIdx) -> &Entry {
        let raw_index = index.0.get() - 1;
        match self.entries.get(raw_index as usize) {
            Some(Slot::Entry(entry)) => entry,
            _ => panic!("invalid constant pool index: {:?}", index.0),
        }
    }
}

/// An index into the constant pool.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantIdx(NonZeroU16);

enum Slot {
    Entry(Entry),
    Marker,
}

/// An entry within the constant pool of a class.
pub enum Entry {
    /// A constant string value. This is used by a number of other entries
    /// within the constant pool. This uses a modified UTF-8 encoding
    /// represented by a `JavaString`.
    Utf8(JavaString),
    /// An `i32`.
    Integer(i32),
    /// A `f32`.
    Float(f32),
    /// An `i64`.
    Long(i64),
    /// A `f64`.
    Double(f64),
    /// A reference to a class or interface.
    Class(ConstantIdx),
    /// A constant Java String.
    String(ConstantIdx),
    /// A reference to a field.
    FieldRef(ConstantIdx, ConstantIdx),
    /// A reference to a class's method.
    MethodRef(ConstantIdx, ConstantIdx),
    /// A reference to an interface's method.
    InterfaceMethodRef(ConstantIdx, ConstantIdx),
    /// A reference to the name and type of a field or method, without reference
    /// to which class it belongs to.
    NameType(ConstantIdx, ConstantIdx),
    /// A method handle.
    MethodHandle(ReferenceKind, ConstantIdx),
    /// A method type.
    MethodType(ConstantIdx),
    /// Specifies the bootstrap method, dynamic invocation name, the argument
    /// and return types of the call, and optionally, a sequence of additional
    /// constants called *static arguments* to the bootstrap method.
    InvokeDynamic(u16, ConstantIdx),
}

impl Entry {
    pub fn into_utf8(&self) -> &JavaStr {
        match self {
            Self::Utf8(string) => string.as_str(),
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_integer(&self) -> i32 {
        match self {
            Self::Integer(value) => *value,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_float(&self) -> f32 {
        match self {
            Self::Float(value) => *value,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_long(&self) -> i64 {
        match self {
            Self::Long(value) => *value,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_double(&self) -> f64 {
        match self {
            Self::Double(value) => *value,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_class(&self) -> ConstantIdx {
        match self {
            Self::Class(index) => *index,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_string(&self) -> ConstantIdx {
        match self {
            Self::String(index) => *index,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_ref(&self) -> (ConstantIdx, ConstantIdx) {
        match self {
            Self::FieldRef(class, name_type)
            | Self::MethodRef(class, name_type)
            | Self::InterfaceMethodRef(class, name_type) => (*class, *name_type),
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_name_type(&self) -> (ConstantIdx, ConstantIdx) {
        match self {
            Self::NameType(name, descriptor) => (*name, *descriptor),
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_method_handle(&self) -> (ReferenceKind, ConstantIdx) {
        match self {
            Self::MethodHandle(kind, reference) => (*kind, *reference),
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_method_type(&self) -> ConstantIdx {
        match self {
            Self::MethodType(descriptor) => *descriptor,
            entry => panic!("invalid entry: {entry:?}"),
        }
    }

    pub fn into_invoke_dynamic(&self) -> (u16, ConstantIdx) {
        match self {
            Self::InvokeDynamic(bootstrap_method, name_type) => (*bootstrap_method, *name_type),
            entry => panic!("invalid entry: {entry:?}"),
        }
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Utf8(utf8) => write!(f, "Utf8({utf8:?})"),
            Self::Integer(value) => write!(f, "Integer({value})"),
            Self::Float(value) => write!(f, "Float({value})"),
            Self::Long(value) => write!(f, "Long({value})"),
            Self::Double(value) => write!(f, "Double({value})"),
            Self::Class(name_idx) => write!(f, "Class({})", name_idx.0),
            Self::String(utf8_idx) => write!(f, "String({})", utf8_idx.0),
            Self::FieldRef(class_idx, name_type_idx) => {
                write!(f, "FieldRef({}, {})", class_idx.0, name_type_idx.0)
            }
            Self::MethodRef(class_idx, name_type_idx) => {
                write!(f, "MethodRef({}, {})", class_idx.0, name_type_idx.0)
            }
            Self::InterfaceMethodRef(class_idx, name_type_idx) => write!(
                f,
                "InterfaceMethodRef({}, {})",
                class_idx.0, name_type_idx.0
            ),
            Self::NameType(name_idx, descriptor_idx) => {
                write!(f, "NameType({}, {})", name_idx.0, descriptor_idx.0)
            }
            Self::MethodHandle(kind, reference) => {
                write!(f, "MethodHandle({kind:?}, {})", reference.0)
            }
            Self::MethodType(descriptor_idx) => write!(f, "MethodType({})", descriptor_idx.0),
            Self::InvokeDynamic(bootstrap_method, name_type_idx) => write!(
                f,
                "InvokeDynamic({}, {})",
                bootstrap_method, name_type_idx.0
            ),
        }
    }
}

/// Denotes the *kind* of a method handle, characterizing its bytecode behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    InvokeVirtual,
    InvokeStatic,
    InvokeSpecial,
    NewInvokeSpecial,
    InvokeInterface,
}
