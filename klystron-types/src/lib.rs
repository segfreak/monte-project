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
    Void,

    Bool,

    Int8,
    Int16,
    Int32,
    Int64,

    Float32,
    Float64,
}

impl TypeKind {
    pub fn info(&self) -> TypeInfo {
        TypeInfo::of(self)
    }
}

impl TypeInfo {
    pub fn of(kind: &TypeKind) -> Self {
        match kind {
            TypeKind::Void => Self {
                size: 0,
                align: 0,
                name: "void",
            },
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

impl TypeKind {
    pub fn is_compatible_to(&self, right: &Self) -> bool {
        if matches!(self, TypeKind::Void) || matches!(right, TypeKind::Void) {
            return false;
        }

        if self == right {
            return true;
        }

        match (self, right) {
            // integer widening
            (a, b) if a.is_integer() && b.is_integer() => a.info().size <= b.info().size,

            // float widening
            (a, b) if a.is_float() && b.is_float() => a.info().size <= b.info().size,

            // int -> float allowed
            (a, b) if a.is_integer() && b.is_float() => true,

            _ => false,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            TypeKind::Int8 | TypeKind::Int16 | TypeKind::Int32 | TypeKind::Int64
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, TypeKind::Float32 | TypeKind::Float64)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSig {
    pub params: Vec<TypeKind>,
    pub returns: Option<TypeKind>,
}

impl FunctionSig {
    pub fn new(params: &[TypeKind], returns: Option<TypeKind>) -> Self {
        Self {
            params: params.into(),
            returns,
        }
    }
}
