use std::collections::HashMap;

use klystron_types::*;

use super::ast::*;
use super::error::*;
use super::utils::*;

#[derive(Debug, Default)]
struct Env {
    scopes: Vec<HashMap<String, TypeKind>>,
}

impl Env {
    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn define(&mut self, name: String, ty: TypeKind) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<&TypeKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Analyzer<'a> {
    env: Env,
    /// function name -> (param types, return type)
    functions: HashMap<String, (bool, Vec<TypeKind>, TypeKind)>,
    /// return type of the function we're currently inside
    current_fn_ret: Option<TypeKind>,
    /// how many nested loops we're in (for break/continue)
    loop_depth: usize,
    /// error reporter
    reporter: &'a mut ErrorReporter,
}

impl<'a> Analyzer<'a> {
    pub fn new(reporter: &'a mut ErrorReporter) -> Self {
        Self {
            env: Env::default(),
            functions: HashMap::new(),
            current_fn_ret: None,
            loop_depth: 0,
            reporter,
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Error> {
        // first pass: collect all top-level struct/fn signatures
        // so functions can call each other regardless of order
        self.collect_definitions(program)?;

        self.env.push();
        for stmt in program {
            match self.analyze_stmt(stmt) {
                Ok(..) => {}
                Err(error) => {
                    self.reporter.report(error);
                }
            };
        }
        self.env.pop();

        Ok(())
    }

    // ------------------------------------------------------------------
    // First pass: collect struct/function signatures without checking bodies
    // ------------------------------------------------------------------

    fn collect_definitions(&mut self, stmts: &[Stmt]) -> Result<(), Error> {
        for stmt in stmts {
            match &stmt.node {
                StmtKind::FunctionDef {
                    name,
                    params,
                    ret,
                    variadic,
                    ..
                } => {
                    if self.functions.contains_key(name) {
                        return Err(Error::new(
                            format!("function '{}' is already defined", name),
                            stmt.span.clone(),
                        ));
                    }
                    let param_types = params.iter().map(|(_, t)| t.clone()).collect();
                    self.functions
                        .insert(name.clone(), (*variadic, param_types, ret.clone()));
                }

                StmtKind::FunctionDecl {
                    name,
                    params,
                    ret,
                    variadic,
                    ..
                } => {
                    if self.functions.contains_key(name) {
                        return Err(Error::new(
                            format!("function '{}' is already defined", name),
                            stmt.span.clone(),
                        ));
                    }
                    let param_types = params.iter().map(|(_, t)| t.clone()).collect();
                    self.functions
                        .insert(name.clone(), (*variadic, param_types, ret.clone()));
                }

                _ => {}
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Statements
    // ------------------------------------------------------------------

    fn analyze_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
        match &stmt.node {
            StmtKind::Dummy => return Err(Error::new("dummy statement".into(), stmt.span.clone())),

            StmtKind::VarDecl { name, ty, init } => {
                let init_ty = self.analyze_expr(init)?;
                self.check_type(ty, &init_ty, init.span.clone())?;
                self.env.define(name.clone(), ty.clone());
            }

            StmtKind::Expr(expr) => {
                self.analyze_expr(expr)?;
            }

            StmtKind::Return(expr) => {
                let ret_ty = match expr {
                    Some(e) => self.analyze_expr(e)?,
                    None => TypeKind::Void,
                };
                match &self.current_fn_ret {
                    None => {
                        return Err(Error::new(
                            "'return' outside of function".into(),
                            stmt.span.clone(),
                        ));
                    }
                    Some(expected) => {
                        if ret_ty != *expected {
                            return Err(Error::new(
                                format!(
                                    "return type mismatch: expected {:?}, got {:?}",
                                    expected, ret_ty
                                ),
                                stmt.span.clone(),
                            ));
                        }
                    }
                }
            }

            StmtKind::Compound { body } => {
                self.env.push();
                for s in body {
                    if let Err(error) = self.analyze_stmt(s) {
                        self.reporter.report(error);
                    }
                }
                self.env.pop();
            }

            // already registered in first pass
            StmtKind::FunctionDecl { .. } => {}

            // already registered in first pass, just check body
            StmtKind::FunctionDef {
                params, ret, body, ..
            } => {
                let prev_ret = self.current_fn_ret.replace(ret.clone());

                self.env.push();
                for (pname, ptype) in params {
                    self.env.define(pname.clone(), ptype.clone());
                }
                for s in body {
                    if let Err(error) = self.analyze_stmt(s) {
                        self.reporter.report(error);
                    }
                }
                self.env.pop();

                self.current_fn_ret = prev_ret;
            }

            StmtKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_ty = self.analyze_expr(cond)?;
                if cond_ty != TypeKind::Bool {
                    return Err(Error::new(
                        format!("if condition must be bool, got {:?}", cond_ty),
                        cond.span.clone(),
                    ));
                }
                self.analyze_stmt(then_branch)?;
                if let Some(eb) = else_branch {
                    self.analyze_stmt(eb)?;
                }
            }

            StmtKind::While { cond, body } => {
                let cond_ty = self.analyze_expr(cond)?;
                if cond_ty != TypeKind::Bool {
                    return Err(Error::new(
                        format!("while condition must be bool, got {:?}", cond_ty),
                        cond.span.clone(),
                    ));
                }
                self.loop_depth += 1;
                self.analyze_stmt(body)?;
                self.loop_depth -= 1;
            }

            StmtKind::Break | StmtKind::Continue => {
                if self.loop_depth == 0 {
                    let kw = if matches!(stmt.node, StmtKind::Break) {
                        "break"
                    } else {
                        "continue"
                    };
                    return Err(Error::new(
                        format!("'{}' outside of loop", kw),
                        stmt.span.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    // ------------------------------------------------------------------
    // Expressions — returns the type of the expression
    // ------------------------------------------------------------------

    fn analyze_expr(&mut self, expr: &Expr) -> Result<TypeKind, Error> {
        match &expr.node {
            ExprKind::Dummy => Err(Error::new("dummy expression".into(), expr.span.clone())),

            ExprKind::Cast { ty, .. } => Ok(ty.clone()),

            ExprKind::Constant(lit) => Ok(self.literal_type(lit)),

            ExprKind::Ident(name) => match self.env.lookup(name) {
                Some(ty) => Ok(ty.clone()),
                None => Err(Error::new(
                    format!("undefined variable '{}'", name),
                    expr.span.clone(),
                )),
            },

            ExprKind::Binary { op, left, right } => {
                self.analyze_binary(*op, left, right, expr.span.clone())
            }

            ExprKind::Unary { op, expr: inner } => self.analyze_unary(*op, inner),

            ExprKind::Call { callee, args } => {
                let fn_name = match &callee.node {
                    ExprKind::Ident(name) => name.clone(),
                    _ => {
                        return Err(Error::new(
                            "only direct function calls are supported".into(),
                            callee.span.clone(),
                        ))
                    }
                };

                let (variadic, param_types, ret_ty) = match self.functions.get(&fn_name).cloned() {
                    Some(sig) => sig,
                    None => {
                        return Err(Error::new(
                            format!("undefined function '{}'", fn_name),
                            callee.span.clone(),
                        ))
                    }
                };

                if !variadic {
                    if args.len() != param_types.len() {
                        return Err(Error::new(
                            format!(
                                "'{}' expects {} argument(s), got {}",
                                fn_name,
                                param_types.len(),
                                args.len()
                            ),
                            expr.span.clone(),
                        ));
                    }
                } else if args.len() < param_types.len() {
                    return Err(Error::new(
                        format!(
                            "'{}' expects at least {} argument(s), got {}",
                            fn_name,
                            param_types.len(),
                            args.len()
                        ),
                        expr.span.clone(),
                    ));
                }

                for (arg, expected) in args.iter().zip(param_types.iter()) {
                    let arg_ty = self.analyze_expr(arg)?;
                    self.check_type(expected, &arg_ty, arg.span.clone())?;
                }

                Ok(ret_ty)
            }

            ExprKind::Assign { target, value } => {
                let target_ty = self.analyze_lvalue(target)?;
                let value_ty = self.analyze_expr(value)?;
                self.check_type(&target_ty, &value_ty, value.span.clone())?;
                Ok(target_ty)
            }
        }
    }

    // ------------------------------------------------------------------
    // LValues
    // ------------------------------------------------------------------

    fn analyze_lvalue(&mut self, lvalue: &LValue) -> Result<TypeKind, Error> {
        match &lvalue.node {
            LValueKind::Ident(name) => match self.env.lookup(name) {
                Some(ty) => Ok(ty.clone()),
                None => Err(Error::new(
                    format!("undefined variable '{}'", name),
                    lvalue.span.clone(),
                )),
            },
        }
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    // fn resolve_field(&self, ty: &Type, field: &str, span: Span) -> Result<Type, Error> {
    //     match ty {
    //         Type::Struct(name) => {
    //             let fields = self.structs.get(name).ok_or_else(|| {
    //                 Error::new(format!("undefined struct '{}'", name), span.clone())
    //             })?;
    //             fields
    //                 .iter()
    //                 .find(|(n, _)| n == field)
    //                 .map(|(_, t)| t.clone())
    //                 .ok_or_else(|| {
    //                     Error::new(format!("struct '{}' has no field '{}'", name, field), span)
    //                 })
    //         }
    //         _ => Err(Error::new(
    //             format!("cannot access field '{}' on {:?}", field, ty),
    //             span,
    //         )),
    //     }
    // }

    fn analyze_binary(
        &mut self,
        op: BinOp,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Result<TypeKind, Error> {
        let lt = self.analyze_expr(left)?;
        let rt = self.analyze_expr(right)?;

        match op {
            // arithmetic
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if !lt.is_numeric() {
                    return Err(Error::new(
                        format!("arithmetic operator requires numeric type, got {:?}", lt),
                        left.span.clone(),
                    ));
                }
                self.check_type(&lt, &rt, span)?;
                Ok(lt)
            }

            // bitwise
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::BitShl | BinOp::BitShr => {
                if !lt.is_integer() {
                    return Err(Error::new(
                        format!("bitwise operator requires integer type, got {:?}", lt),
                        left.span.clone(),
                    ));
                }
                if lt != rt {
                    return Err(Error::new(
                        format!("type mismatch: {:?} vs {:?}", lt, rt),
                        span,
                    ));
                }
                Ok(lt)
            }

            // logical
            BinOp::And | BinOp::Or => {
                if lt != TypeKind::Bool {
                    return Err(Error::new(
                        format!("logical operator requires bool, got {:?}", lt),
                        left.span.clone(),
                    ));
                }
                if rt != TypeKind::Bool {
                    return Err(Error::new(
                        format!("logical operator requires bool, got {:?}", rt),
                        right.span.clone(),
                    ));
                }
                Ok(TypeKind::Bool)
            }

            // equality
            BinOp::Eq | BinOp::NotEq => {
                if lt != rt {
                    return Err(Error::new(
                        format!("cannot compare {:?} with {:?}", lt, rt),
                        span,
                    ));
                }
                Ok(TypeKind::Bool)
            }

            // comparison
            BinOp::Less | BinOp::LessEq | BinOp::Great | BinOp::GreatEq => {
                if !lt.is_numeric() {
                    return Err(Error::new(
                        format!("comparison requires numeric type, got {:?}", lt),
                        left.span.clone(),
                    ));
                }
                self.check_type(&lt, &rt, span)?;
                Ok(TypeKind::Bool)
            }
        }
    }

    fn analyze_unary(&mut self, op: UnOp, expr: &Expr) -> Result<TypeKind, Error> {
        let ty = self.analyze_expr(expr)?;
        match op {
            UnOp::Neg => {
                if !ty.is_numeric() {
                    return Err(Error::new(
                        format!("'-' requires numeric type, got {:?}", ty),
                        expr.span.clone(),
                    ));
                }
                Ok(ty)
            }
            UnOp::BitNeg => {
                if !ty.is_integer() {
                    return Err(Error::new(
                        format!("'~' requires integer type, got {:?}", ty),
                        expr.span.clone(),
                    ));
                }
                Ok(ty)
            }
            UnOp::Not => {
                if ty != TypeKind::Bool {
                    return Err(Error::new(
                        format!("'!' requires bool, got {:?}", ty),
                        expr.span.clone(),
                    ));
                }
                Ok(TypeKind::Bool)
            }
        }
    }

    fn literal_type(&self, lit: &ConstantLiteral) -> TypeKind {
        match lit {
            ConstantLiteral::Integer(_) => TypeKind::Int32,
            ConstantLiteral::FloatPoint(_) => TypeKind::Float32,
            ConstantLiteral::Boolean(_) => TypeKind::Bool,
        }
    }

    fn check_type(&self, expected: &TypeKind, actual: &TypeKind, span: Span) -> Result<(), Error> {
        if actual.is_compatible_to(expected) {
            Ok(())
        } else {
            Err(Error::new(
                format!(
                    "incompatible type: expected {:?}, got {:?}",
                    expected, actual
                ),
                span,
            ))
        }
    }
}
