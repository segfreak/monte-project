use super::utils::*;

use crate::typesys::Type;

pub type Program = Vec<Stmt>;
pub type Expr = Spanned<ExprKind>;
pub type Stmt = Spanned<StmtKind>;
pub type LValue = Spanned<LValueKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantLiteral {
    Integer(i64),
    FloatPoint(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    /// l + r
    Add,
    /// l - r
    Sub,
    /// l * r
    Mul,
    /// l / r
    Div,
    /// l % r
    Mod,

    /// l & r
    BitAnd,
    /// l | r
    BitOr,
    /// l ^ r
    BitXor,
    /// l << r
    BitShl,
    /// l >> r
    BitShr,

    /// l && r
    And,
    // l || r
    Or,
    /// l == r
    Eq,
    /// l != r
    NotEq,

    /// l < r
    Less,
    /// l <= r
    LessEq,
    /// l > r
    Great,
    /// l >= r
    GreatEq,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    /// -v
    Neg,
    /// ~v
    BitNeg,
    /// !v
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LValueKind {
    Ident(String),
    // MemberAccess {
    //     target: Box<LValue>,
    //     field: String,
    // },
    IndexAccess {
        target: Box<LValue>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// Dummy node, placed on errors while parsing
    Dummy,

    Constant(ConstantLiteral),
    Ident(String),

    /// {left} {op} {right}
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// {op} {expr}
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },

    /// {expr} as {ty}
    Cast {
        expr: Box<Expr>,
        ty: Type,
    },

    /// {target}[{index}]
    IndexAccess {
        target: Box<Expr>,
        index: Box<Expr>,
    },

    /// [e1, e2, e3]
    ArrayLiteral {
        elements: Vec<Expr>,
    },

    // /// {name} { {fields...} }
    // StructLiteral {
    //     struct_name: String,
    //     fields: Vec<(Spanned<String>, Expr)>,
    // },
    /// {expr}.{field_name}
    // MemberAccess {
    //     target: Box<Expr>,
    //     field: String,
    // },

    /// {callee}({args...})
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },

    /// {target} = {value}
    Assign {
        target: LValue,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Return(Option<Expr>),
    Expr(Expr),

    /// let {name}: {Type} = {Expr}
    VarDecl {
        name: String,
        ty: Type,
        init: Expr,
    },

    /// { body... }
    Compound {
        body: Vec<Stmt>,
    },

    /// struct {name} { {fields...} }
    StructDef {
        name: String,
        fields: Vec<(String, Type)>,
    },

    /// fn {name}({params...}) ?{: ret}
    FunctionDecl {
        /// function name
        name: String,
        /// function params
        params: Vec<(String, Type)>,
        /// function is variadic
        variadic: bool,
        /// return type
        ret: Type,
    },

    /// fn {name}({params...}) ?{: ret} {body...}
    FunctionDef {
        /// function name
        name: String,
        /// function params
        params: Vec<(String, Type)>,
        /// function is variadic
        variadic: bool,
        /// return type
        ret: Type,
        /// function body
        body: Vec<Stmt>,
    },

    /// if ({cond}) {then} else {else}
    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    /// break
    Break,
    /// continue
    Continue,
    /// while ({cond}) {body}
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
}
