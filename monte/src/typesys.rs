#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,

    Int8,
    Int16,
    Int32,
    Int64,

    Float32,
    Float64,

    Bool,
}

pub fn match_type(left: &Type, right: &Type) -> bool {
    match (left, right) {
        (Type::Void, Type::Void)
        | (Type::Int8, Type::Int8)
        | (Type::Int16, Type::Int16)
        | (Type::Int32, Type::Int32)
        | (Type::Int64, Type::Int64)
        | (Type::Float32, Type::Float32)
        | (Type::Float64, Type::Float64)
        | (Type::Bool, Type::Bool) => true,

        _ => false,
    }
}
