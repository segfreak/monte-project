#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,

    IntLit,
    Int8,
    Int16,
    Int32,
    Int64,

    Float32,
    Float64,

    Bool,

    Function { params: Vec<Type>, ret: Box<Type> },

    String,
    Array(Box<Type>, usize), // [T; N]
}

pub fn match_type(left: &Type, right: &Type) -> bool {
    match (left, right) {
        (Type::Void, Type::Void)
        | (Type::Int8, Type::Int8)
        | (Type::Int16, Type::Int16)
        | (Type::Int32, Type::Int32)
        | (Type::Int64, Type::Int64)
        | (Type::IntLit, Type::IntLit)
        | (Type::IntLit, Type::Int8)
        | (Type::IntLit, Type::Int16)
        | (Type::IntLit, Type::Int32)
        | (Type::IntLit, Type::Int64)
        | (Type::Int8, Type::IntLit)
        | (Type::Int16, Type::IntLit)
        | (Type::Int32, Type::IntLit)
        | (Type::Int64, Type::IntLit)
        | (Type::Float32, Type::Float32)
        | (Type::Float64, Type::Float64)
        | (Type::Bool, Type::Bool)
        | (Type::String, Type::String) => true,

        (Type::Array(t1, n1), Type::Array(t2, n2)) => n1 == n2 && match_type(t1, t2),

        (
            Type::Function {
                params: p1,
                ret: r1,
            },
            Type::Function {
                params: p2,
                ret: r2,
            },
        ) => {
            p1.len() == p2.len()
                && p1.iter().zip(p2).all(|(a, b)| match_type(a, b))
                && match_type(r1, r2)
        }

        _ => false,
    }
}
