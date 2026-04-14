use super::utils::*;

use typesys::*;

pub type Program = Vec<Stmt>;
pub type Expr = Spanned<ExprKind>;
pub type Stmt = Spanned<StmtKind>;
pub type LValue = Spanned<LValueKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantLiteral {
    Integer(i64),
    FloatPoint(f64),
    Boolean(bool),
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
        ty: TypeKind,
    },

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
    Dummy,

    Return(Option<Expr>),
    Expr(Expr),

    /// let {name}: {Type} = {Expr}
    VarDecl {
        name: String,
        ty: TypeKind,
        init: Expr,
    },

    /// { body... }
    Compound {
        body: Vec<Stmt>,
    },

    /// fn {name}({params...}) ?{: ret}
    FunctionDecl {
        /// function name
        name: String,
        /// function params
        params: Vec<(String, TypeKind)>,
        /// function is variadic
        variadic: bool,
        /// return type
        ret: TypeKind,
    },

    /// fn {name}({params...}) ?{: ret} {body...}
    FunctionDef {
        /// function name
        name: String,
        /// function params
        params: Vec<(String, TypeKind)>,
        /// function is variadic
        variadic: bool,
        /// return type
        ret: TypeKind,
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
