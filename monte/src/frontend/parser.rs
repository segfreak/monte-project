use super::ast::*;
use super::error::*;
use super::utils::*;

use klystron_types::*;

type Token = Spanned<TokenKind>;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// Dummy node, placed on errors while lexing
    Dummy,

    // literals...
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Ident(String),

    // keywords
    Let,
    Fn,
    Return,
    If,
    Else,
    As,
    While,
    Break,
    Continue,

    // operators
    /// ~v
    BitNeg,
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

    /// !v
    Not,
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

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,

    // symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Ellipsis,
    Arrow, // ->

    EOF,
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
    reporter: &'a mut ErrorReporter,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &Source<'a>, reporter: &'a mut ErrorReporter) -> Self {
        let cursor = Cursor::new(src.input);

        Self { cursor, reporter }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.cursor.chars.next();
        if c.is_some() {
            self.cursor.pos += 1;
        }
        c
    }

    fn peek(&mut self) -> Option<&char> {
        self.cursor.chars.peek()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.bump();
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start = self.cursor.pos;
        let c = match self.bump() {
            Some(c) => c,
            None => {
                return Spanned::new(TokenKind::EOF, self.cursor.pos..self.cursor.pos);
            }
        };

        let kind = match c {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some(&'>') {
                    self.bump();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '!' => {
                if self.peek() == Some(&'=') {
                    self.bump();
                    TokenKind::NotEq
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                if self.peek() == Some(&'=') {
                    self.bump();
                    TokenKind::LessEq
                } else if self.peek() == Some(&'<') {
                    self.bump();
                    TokenKind::BitShl
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.peek() == Some(&'=') {
                    self.bump();
                    TokenKind::GreatEq
                } else if self.peek() == Some(&'>') {
                    self.bump();
                    TokenKind::BitShr
                } else {
                    TokenKind::Great
                }
            }
            '=' => {
                if self.peek() == Some(&'=') {
                    self.bump();
                    TokenKind::Eq
                } else {
                    TokenKind::Assign
                }
            }
            '&' => {
                if self.peek() == Some(&'&') {
                    self.bump();
                    TokenKind::And
                } else {
                    TokenKind::BitAnd
                }
            }
            '|' => {
                if self.peek() == Some(&'|') {
                    self.bump();
                    TokenKind::Or
                } else {
                    TokenKind::BitOr
                }
            }
            '~' => TokenKind::BitNeg,
            '^' => TokenKind::BitXor,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '0'..='9' => {
                let mut s = c.to_string();
                while let Some(&d) = self.peek() {
                    if d.is_ascii_digit() {
                        s.push(self.bump().unwrap());
                    } else {
                        break;
                    }
                }
                TokenKind::Integer(s.parse().unwrap())
            }
            '"' => {
                let mut s = String::new();
                while let Some(&ch) = self.peek() {
                    if ch == '"' {
                        self.bump();
                        break;
                    }
                    s.push(self.bump().unwrap());
                }
                TokenKind::String(s)
            }
            '.' => {
                if self.peek() == Some(&'.') {
                    self.bump();

                    if self.peek() == Some(&'.') {
                        self.bump();
                        TokenKind::Ellipsis
                    } else {
                        self.reporter.report(Error::new(
                            "unexpected '..', expected '...'".into(),
                            self.cursor.pos..self.cursor.pos,
                        ));
                        return Token::new(TokenKind::Dummy, self.cursor.pos..self.cursor.pos);
                    }
                } else {
                    self.reporter.report(Error::new(
                        "unexpected '.', expected '...'".into(),
                        self.cursor.pos..self.cursor.pos,
                    ));
                    return Token::new(TokenKind::Dummy, self.cursor.pos..self.cursor.pos);
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut s = c.to_string();
                while let Some(&ch) = self.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        s.push(self.bump().unwrap());
                    } else {
                        break;
                    }
                }
                match s.as_str() {
                    "true" => TokenKind::Boolean(true),
                    "false" => TokenKind::Boolean(false),
                    "let" => TokenKind::Let,
                    "fn" => TokenKind::Fn,
                    "return" => TokenKind::Return,
                    "if" => TokenKind::If,
                    "as" => TokenKind::As,
                    "else" => TokenKind::Else,
                    "while" => TokenKind::While,
                    "break" => TokenKind::Break,
                    "continue" => TokenKind::Continue,
                    _ => TokenKind::Ident(s),
                }
            }
            c => {
                self.reporter.report(Error::new(
                    format!("unexpected char: {}", c),
                    self.cursor.pos..self.cursor.pos,
                ));
                TokenKind::Dummy
            }
        };

        Spanned::new(kind, start..self.cursor.pos)
    }

    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok.node == TokenKind::EOF {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }
}

// =====================
// Pratt Parser
// =====================
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn peek(&self) -> &TokenKind {
        if self.pos >= self.tokens.len() {
            &TokenKind::EOF
        } else {
            &self.tokens[self.pos].node
        }
    }

    fn peek_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span.clone()
        } else {
            0..0
        }
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span.clone()
        } else {
            0..0
        }
    }

    fn next(&mut self) -> TokenKind {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].node.clone();
            self.pos += 1;
            tok
        } else {
            TokenKind::EOF
        }
    }

    fn eat(&mut self, kind: &TokenKind) -> Result<(), Error> {
        let span = self.peek_span();
        let t = self.next();

        if &t == kind {
            Ok(())
        } else {
            Err(Error::new(
                format!("expected {:?}, got {:?}", kind, t),
                span,
            ))
        }
    }

    fn infix_binding_power(tok: &TokenKind) -> Option<(u8, u8, BinOp)> {
        use TokenKind::*;

        let res = match tok {
            Or => (2, 3, BinOp::Or),
            And => (4, 5, BinOp::And),

            BitOr => (6, 7, BinOp::BitOr),
            BitXor => (8, 9, BinOp::BitXor),
            BitAnd => (10, 11, BinOp::BitAnd),

            Eq => (12, 13, BinOp::Eq),
            NotEq => (12, 13, BinOp::NotEq),

            Less => (14, 15, BinOp::Less),
            Great => (14, 15, BinOp::Great),
            LessEq => (14, 15, BinOp::LessEq),
            GreatEq => (14, 15, BinOp::GreatEq),

            BitShl => (16, 17, BinOp::BitShl),
            BitShr => (16, 17, BinOp::BitShr),

            Plus => (18, 19, BinOp::Add),
            Minus => (18, 19, BinOp::Sub),

            Star => (20, 21, BinOp::Mul),
            Slash => (20, 21, BinOp::Div),
            Percent => (20, 21, BinOp::Mod),

            _ => return None,
        };

        Some(res)
    }

    fn parse_type(&mut self) -> Result<TypeKind, Error> {
        match self.next() {
            TokenKind::Ident(s) => Ok(match s.as_str() {
                "void" => TypeKind::Void,
                "int8" => TypeKind::Int8,
                "int16" => TypeKind::Int16,
                "int32" => TypeKind::Int32,
                "int64" => TypeKind::Int64,
                "float32" => TypeKind::Float32,
                "float64" => TypeKind::Float64,
                "bool" => TypeKind::Bool,
                _ => {
                    return Err(Error::new("undefined type".into(), self.prev_span()));
                }
            }),

            t => Err(Error::new(
                format!("expected type, got {:?}", t),
                self.prev_span(),
            )),
        }
    }

    pub fn parse_break(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.next();
        self.eat(&TokenKind::Semicolon)?;
        Ok(Spanned::new(StmtKind::Break, start..self.prev_span().end))
    }

    pub fn parse_continue(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.next();
        self.eat(&TokenKind::Semicolon)?;
        Ok(Spanned::new(
            StmtKind::Continue,
            start..self.prev_span().end,
        ))
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, Error> {
        let start = self.peek_span().start;
        let mut lhs = self.parse_prefix()?;

        loop {
            if let TokenKind::LParen = self.peek() {
                self.next();
                let mut args = vec![];
                while self.peek() != &TokenKind::RParen {
                    args.push(self.parse_expr(0)?);
                    if self.peek() == &TokenKind::Comma {
                        self.next();
                    } else {
                        break;
                    }
                }
                self.eat(&TokenKind::RParen)?;
                let end = self.prev_span().end;
                lhs = Spanned::new(
                    ExprKind::Call {
                        callee: Box::new(lhs),
                        args,
                    },
                    start..end,
                );
                continue;
            }

            if self.peek() == &TokenKind::As {
                self.next();
                let ty = self.parse_type()?;
                let end = self.peek_span().end;

                lhs = Spanned::new(
                    ExprKind::Cast {
                        expr: Box::new(lhs),
                        ty,
                    },
                    start..end,
                );

                continue;
            }

            if let TokenKind::Assign = self.peek() {
                if min_bp > 1 {
                    break;
                }
                self.next();
                let rhs = self.parse_expr(1)?;
                let end = rhs.span.end;
                let target = match &lhs.node {
                    ExprKind::Ident(name) => LValue::new(LValueKind::Ident(name.clone()), lhs.span),
                    _ => {
                        return Err(Error::new(
                            "expected assignment target".into(),
                            self.prev_span(),
                        ));
                    }
                };
                lhs = Spanned::new(
                    ExprKind::Assign {
                        target,
                        value: Box::new(rhs),
                    },
                    start..end,
                );
                continue;
            }

            let (l_bp, r_bp, op) = match Self::infix_binding_power(self.peek()) {
                Some(v) => v,
                None => break,
            };

            if l_bp < min_bp {
                break;
            }

            self.next();
            let rhs = self.parse_expr(r_bp)?;
            let end = rhs.span.end;
            lhs = Spanned::new(
                ExprKind::Binary {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                },
                start..end,
            );
        }

        Ok(lhs)
    }

    pub fn parse_stmt_expr(&mut self, min_bp: u8) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        let expr = self.parse_expr(min_bp)?;
        self.eat(&TokenKind::Semicolon)?;
        let end = self.prev_span().end;
        Ok(Spanned::new(StmtKind::Expr(expr), start..end))
    }

    pub fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            TokenKind::Let => match self.parse_var_decl() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::Return => match self.parse_return() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::While => match self.parse_while() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::If => match self.parse_if() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::Fn => match self.parse_function() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::Break => match self.parse_break() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::Continue => match self.parse_continue() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            TokenKind::LBrace => match self.parse_block() {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
            _ => match self.parse_stmt_expr(0) {
                Ok(r) => r,
                Err(e) => {
                    self.errors.push(e);
                    Stmt::new(StmtKind::Dummy, self.prev_span())
                }
            },
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut stmts = Program::new();

        while self.peek() != &TokenKind::EOF {
            stmts.push(self.parse_stmt());
        }

        stmts
    }

    fn parse_block(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.eat(&TokenKind::LBrace)?;

        let mut body = Vec::new();
        while self.peek() != &TokenKind::RBrace {
            body.push(self.parse_stmt());
        }

        self.eat(&TokenKind::RBrace)?;
        let end = self.prev_span().end;
        Ok(Spanned::new(StmtKind::Compound { body }, start..end))
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.eat(&TokenKind::Let)?;

        let name = match self.next() {
            TokenKind::Ident(s) => s,
            _ => {
                return Err(Error::new("expected identifier".into(), self.prev_span()));
            }
        };

        self.eat(&TokenKind::Colon)?;
        let ty = self.parse_type()?;

        self.eat(&TokenKind::Assign)?;
        let init = self.parse_expr(0)?;

        self.eat(&TokenKind::Semicolon)?;
        let end = self.prev_span().end;
        Ok(Spanned::new(
            StmtKind::VarDecl { name, ty, init },
            start..end,
        ))
    }

    fn parse_return(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.eat(&TokenKind::Return)?;

        let (expr, end) = if self.peek() == &TokenKind::Semicolon {
            self.next();
            (None, self.prev_span().end)
        } else {
            let e = self.parse_expr(0)?;
            self.eat(&TokenKind::Semicolon)?;
            let end = self.prev_span().end;
            (Some(e), end)
        };

        Ok(Spanned::new(StmtKind::Return(expr), start..end))
    }

    fn parse_while(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.eat(&TokenKind::While)?;
        self.eat(&TokenKind::LParen)?;

        let cond = self.parse_expr(0)?;
        self.eat(&TokenKind::RParen)?;

        let body = Box::new(self.parse_stmt());
        let end = body.span.end;
        Ok(Spanned::new(StmtKind::While { cond, body }, start..end))
    }

    fn parse_if(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.eat(&TokenKind::If)?;
        self.eat(&TokenKind::LParen)?;

        let cond = self.parse_expr(0)?;
        self.eat(&TokenKind::RParen)?;

        let then_branch = Box::new(self.parse_stmt());

        let (else_branch, end) = if self.peek() == &TokenKind::Else {
            self.next();
            let eb = Box::new(self.parse_stmt());
            let end = eb.span.end;
            (Some(eb), end)
        } else {
            let end = then_branch.span.end;
            (None, end)
        };

        Ok(Spanned::new(
            StmtKind::If {
                cond,
                then_branch,
                else_branch,
            },
            start..end,
        ))
    }

    fn parse_function(&mut self) -> Result<Stmt, Error> {
        let start = self.peek_span().start;
        self.eat(&TokenKind::Fn)?;

        let name = match self.next() {
            TokenKind::Ident(s) => s,
            _ => {
                return Err(Error::new(
                    "expected function name".into(),
                    self.prev_span(),
                ));
            }
        };

        self.eat(&TokenKind::LParen)?;

        let mut params = Vec::new();
        let mut variadic = false;

        if self.peek() != &TokenKind::RParen {
            loop {
                if let TokenKind::Ellipsis = self.peek() {
                    self.next();

                    if self.peek() != &TokenKind::RParen {
                        return Err(Error::new(
                            "variadic must be last parameter".into(),
                            self.peek_span(),
                        ));
                    }

                    variadic = true;
                    break;
                }

                let pname = match self.next() {
                    TokenKind::Ident(s) => s,
                    _ => {
                        return Err(Error::new("expected param name".into(), self.prev_span()));
                    }
                };

                self.eat(&TokenKind::Colon)?;
                let ptype = self.parse_type()?;

                params.push((pname, ptype));

                if self.peek() == &TokenKind::Comma {
                    self.next();
                } else {
                    break;
                }
            }
        }

        self.eat(&TokenKind::RParen)?;

        let ret = if self.peek() == &TokenKind::Arrow {
            self.next();
            self.parse_type()?
        } else {
            TypeKind::Void
        };

        if self.peek() == &TokenKind::Semicolon {
            self.next();
            let end = self.prev_span().end;

            return Ok(Spanned::new(
                StmtKind::FunctionDecl {
                    name,
                    params,
                    variadic,
                    ret,
                },
                start..end,
            ));
        } else {
            let block = self.parse_block()?;
            let end = block.span.end;
            let body = match block.node {
                StmtKind::Compound { body } => body,
                _ => unreachable!(),
            };

            Ok(Spanned::new(
                StmtKind::FunctionDef {
                    name,
                    params,
                    variadic,
                    ret,
                    body,
                },
                start..end,
            ))
        }
    }

    fn is_type_ahead(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(_))
    }

    fn parse_prefix(&mut self) -> Result<Expr, Error> {
        let start = self.peek_span().start;

        let kind = match self.next() {
            TokenKind::Integer(n) => ExprKind::Constant(ConstantLiteral::Integer(n)),
            TokenKind::Boolean(b) => ExprKind::Constant(ConstantLiteral::Boolean(b)),
            TokenKind::Ident(name) => ExprKind::Ident(name),
            TokenKind::Minus => ExprKind::Unary {
                op: UnOp::Neg,
                expr: Box::new(self.parse_expr(30)?),
            },
            TokenKind::Not => ExprKind::Unary {
                op: UnOp::Not,
                expr: Box::new(self.parse_expr(30)?),
            },
            TokenKind::BitNeg => ExprKind::Unary {
                op: UnOp::BitNeg,
                expr: Box::new(self.parse_expr(30)?),
            },
            TokenKind::LParen => {
                if self.is_type_ahead() {
                    let ty = self.parse_type()?;
                    self.eat(&TokenKind::RParen)?;
                    let expr = self.parse_expr(30)?;
                    return Ok(Spanned::new(
                        ExprKind::Cast {
                            expr: Box::new(expr),
                            ty,
                        },
                        start..self.prev_span().end,
                    ));
                }

                let expr = self.parse_expr(0)?;
                self.eat(&TokenKind::RParen)?;
                return Ok(expr);
            }
            t => {
                return Err(Error::new(
                    format!("unexpected token {:?}", t),
                    self.prev_span(),
                ));
            }
        };

        let end = self.prev_span().end;
        Ok(Spanned::new(kind, start..end))
    }

    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.parse_expr(0)
    }
}
