use crate::vm::value::Value;

pub type HostInt = i64;
pub type HostFloat = f64;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct TypeInfo {
    size: usize,
    align: usize,
    name: &'static str,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypeKind {
    Bool,

    Int8,
    Int16,
    Int32,
    Int64,

    Float32,
    Float64,
}

impl TypeKind {
    pub fn of(value: &Value) -> Self {
        value.ty()
    }

    pub fn info(&self) -> TypeInfo {
        TypeInfo::of(self)
    }
}

impl TypeInfo {
    pub fn of(kind: &TypeKind) -> Self {
        match kind {
            TypeKind::Bool => Self {
                size: 1,
                align: 1,
                name: "bool",
            },
            TypeKind::Int8 => Self {
                size: 1,
                align: 1,
                name: "int8",
            },
            TypeKind::Int16 => Self {
                size: 2,
                align: 2,
                name: "int16",
            },
            TypeKind::Int32 => Self {
                size: 4,
                align: 4,
                name: "int32",
            },
            TypeKind::Int64 => Self {
                size: 8,
                align: 8,
                name: "int64",
            },
            TypeKind::Float32 => Self {
                size: 4,
                align: 4,
                name: "float32",
            },
            TypeKind::Float64 => Self {
                size: 8,
                align: 8,
                name: "float64",
            },
        }
    }
}
