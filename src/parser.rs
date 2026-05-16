use std::fmt::Display;

use crate::lexer::{LocatedToken, SimpleToken, Token, TokenVec};

use super::lexer;
#[derive(Copy, Clone, Debug, PartialEq, enum_display::EnumDisplay)]
pub enum Operator {
    Equal,
    NotEqual,
    Lesser,
    LesserEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Slash,
    Star,
}
pub type ExprID = usize;
pub type StatementID = usize;
pub type TokenID = usize;

#[derive(Copy, Clone, Debug, enum_display::EnumDisplay)]
pub enum Unary {
    Not(ExprID),
    Minus(ExprID),
}
#[derive(Clone, Debug, PartialEq)]
pub struct LocatedPrimary {
    pub inner: Primary,
    // Inclusive range
    pub token_start: TokenID,
    pub token_end: TokenID,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primary {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Callee(String, usize),
    Nil,
}
impl Primary {
    pub fn located(self, token_start: usize, token_end: usize) -> LocatedPrimary {
        LocatedPrimary {
            inner: self,
            token_start,
            token_end,
        }
    }
}
impl Display for Primary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primary::Number(v) => write!(f, "Number: {}", v),
            Primary::String(v) => write!(f, "String: {}", v),
            Primary::Boolean(v) => write!(f, "Bool: {}", v),
            Primary::Nil => write!(f, "{}", "Nil"),
            Primary::Identifier(s) => write!(f, "Identifier({})", s),
            Primary::Callee(s, _) => write!(f, "Function Call({})", s),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Binary {
    pub left: ExprID,
    pub operator: Operator,
    pub right: ExprID,
}
#[derive(Clone, Debug)]
pub struct Ternary {
    pub left: ExprID,
    pub middle: ExprID,
    pub right: ExprID,
}
#[derive(Clone, Debug)]
pub struct LogicalAnd {
    pub left: ExprID,
    pub right: ExprID,
}
#[derive(Clone, Debug)]
pub struct LogicalOr {
    pub left: ExprID,
    pub right: ExprID,
}
#[derive(Clone, Debug)]
pub struct FunctionCall {
    pub func: ExprID,
    pub arguments: Vec<ExprID>,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Terminal(LocatedPrimary),
    Unary(Unary),
    Binary(Binary),
    Ternary(Ternary),
    LogicalAnd(LogicalAnd),
    LogicalOr(LogicalOr),
    Assignment(String, ExprID),
    FunctionCall(FunctionCall),
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Statement(Statement),
    VarDecl(String, ExprID),
    Empty,
}
#[derive(Clone, Debug)]
pub enum Statement {
    ExprStatement(ExprID),
    PrintStatement(Vec<ExprID>),
    Block(Vec<Declaration>),
    IfStatement(Vec<(ExprID, Statement)>, Option<StatementID>),
    Whileloop(ExprID, StatementID),
    Break(LocatedToken),
    Continue(LocatedToken),
}

#[derive(Clone, Debug)]
pub struct AST {
    pub _tokens: TokenVec,
    pub expr_arena: Vec<Expr>,
    pub statement_arena: Vec<Statement>,
    pub roots: Vec<Declaration>,
}

impl AST {
    pub fn new(tokens: TokenVec) -> Self {
        Self {
            _tokens: tokens,
            expr_arena: Vec::with_capacity(4096),
            statement_arena: Vec::with_capacity(4096),
            roots: vec![],
        }
    }

    // pub fn traverse_lrn<T: ASTVisitor>(&self, input: ExprID, visitor: &mut T) {
    //     match &self.arena[input] {
    //         Expr::Terminal(literal) => visitor.visit_terminal(&literal),
    //         Expr::Unary(unary) => visitor.visit_unary(&self.arena, &unary),
    //         Expr::Binary(binary) => {
    //             self.traverse_lrn(binary.left, visitor);
    //             self.traverse_lrn(binary.right, visitor);
    //             visitor.visit_binary(&self.arena, &binary);
    //         }
    //         Expr::Statement(expr_id) => self.traverse_lrn(*expr_id, visitor),
    //         Expr::Ternary(ternary) => {
    //             self.traverse_lrn(ternary.left, visitor);
    //             self.traverse_lrn(ternary.middle, visitor);
    //             self.traverse_lrn(ternary.right, visitor);
    //             visitor.visit_ternary(&self.arena, &ternary);
    //         }
    //     };
    // }
}
pub trait _ASTVisitor {
    fn visit_ternary(&mut self, arena: &[Expr], ternary: &Ternary);
    fn visit_binary(&mut self, arena: &[Expr], binary: &Binary);
    fn visit_terminal(&mut self, literal: &Primary);
    fn visit_unary(&mut self, arena: &[Expr], unary: &Unary);
}

#[derive(Debug, Clone)]
pub enum ASTError {
    Eof,
    TooManyArguments,
    TokenError(LocatedToken, Vec<SimpleToken>),
    BinaryNoLeft(LocatedToken),
    RValueAssignment(ExprID),
}

struct ErrorDisplay<'a> {
    error: &'a ASTError,
    source: &'a str,
}

impl<'a> std::fmt::Display for ErrorDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.format_error(self.source, f)
    }
}

impl ASTError {
    fn format_error(&self, source: &str, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTError::Eof => write!(f, "{}", "Unexpected End Of File"),
            ASTError::TokenError(located_token, expected) => write!(
                f,
                "Unexpected token \"{}\" at {}:{}:{}, expected {:#?}",
                located_token.token, source, located_token.line, located_token.row, expected
            ),
            ASTError::BinaryNoLeft(located_token) => write!(
                f,
                "Unexpected token \"{}\" at {}:{}:{}, This Token should have a left and right side",
                located_token.token, source, located_token.line, located_token.row
            ),
            ASTError::RValueAssignment(_) => {
                write!(f, "Unexpected token assignment should have a lvalue",)
            }
            ASTError::TooManyArguments => write!(
                f,
                "{}",
                "Too many argument for function call, maximum is 255"
            ),
        }
    }

    pub fn get_formated_error(&self, source: &str) -> String {
        format!(
            "{}",
            ErrorDisplay {
                error: self,
                source
            }
        )
    }
}

pub struct ASTBuilder {
    current_index: usize,
    tokens: TokenVec,
    ast: AST,
}

/// Statement → expression | print | assignment ;
/// print → print (expression*)
/// assignment → var IDENT = expression
/// expression → ternary
/// ternary → equality ? expression : expression
/// equality → comparison ( ( "!=" | "==" ) comparison )*
/// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )*
/// term → factor ( ( "-" | "+" ) factor )*
/// factor → unary ( ( "/" | "*" ) unary )*
/// unary → ( "!" | "-" ) unary
/// | primary
/// primary → NUMBER | STRING | "true" | "false" | "nil"
/// | "(" expression ")"
///
/// Error Productions :
/// binary error production
impl ASTBuilder {
    fn emit(&mut self, expr: Expr) -> ExprID {
        self.ast.expr_arena.push(expr);
        self.ast.expr_arena.len() - 1
    }
    fn emit_statment(&mut self, statement: Statement) -> StatementID {
        self.ast.statement_arena.push(statement);
        self.ast.statement_arena.len() - 1
    }
    fn emit_primary(&mut self, primary: Primary, offset: i64) -> ExprID {
        self.ast.expr_arena.push(Expr::Terminal(LocatedPrimary {
            inner: primary,
            token_start: self.current_index + offset as usize,
            token_end: self.current_index + offset as usize,
        }));
        self.ast.expr_arena.len() - 1
    }

    fn current(&self, offset: i64) -> LocatedToken {
        self.tokens.tokens[(self.current_index as i64 + offset) as usize].clone()
    }
    fn is_last(&self) -> bool {
        self.current_index == self.tokens.tokens.len()
    }
    fn _peek_next(&self) -> Option<Token> {
        if self.is_last() {
            None
        } else {
            Some(
                self.tokens.tokens[(self.current_index as i64 + 1) as usize]
                    .token
                    .clone(),
            )
        }
    }
    fn error_token(&self, token_offset: i64, expected: Vec<SimpleToken>) -> ASTError {
        let token = self.tokens.tokens[(self.current_index as i64 + token_offset) as usize].clone();
        ASTError::TokenError(token, expected)
    }
    fn declaration(&mut self) -> Result<Declaration, ASTError> {
        while let Some(_) = self.my_match(&[SimpleToken::SemiColon]) {}
        if self.is_last() {
            return Ok(Declaration::Empty);
        } else if self.current(0).token
            == Token::Single(SimpleToken::KeyWord(lexer::KeyWordType::Var))
        {
            let d = self.vardecl()?;
            self.consume(SimpleToken::SemiColon)?;
            Ok(d)
        } else {
            let s = self.statement()?;
            Ok(Declaration::Statement(s))
        }
    }
    fn statement(&mut self) -> Result<Statement, ASTError> {
        let s = if self.check(&Token::Single(SimpleToken::KeyWord(
            lexer::KeyWordType::Print,
        ))) {
            let r = self.print()?;
            self.consume(SimpleToken::SemiColon)?;
            Ok(r)
        } else if self.my_match(&[SimpleToken::LeftBrace]).is_some() {
            let mut declarations = vec![];
            loop {
                if self.my_match(&[SimpleToken::RightBrace]).is_some() {
                    return Ok(Statement::Block(declarations));
                }
                let declaration = self.declaration()?;
                declarations.push(declaration);
                if self.is_last() {
                    return Err(ASTError::Eof);
                }
            }
        } else if self
            .my_match(&[SimpleToken::KeyWord(lexer::KeyWordType::If)])
            .is_some()
        {
            self.ifstmt()
        } else if self
            .my_match(&[SimpleToken::KeyWord(lexer::KeyWordType::While)])
            .is_some()
        {
            self.whilestmt()
        } else if self
            .my_match(&[SimpleToken::KeyWord(lexer::KeyWordType::Break)])
            .is_some()
        {
            self.consume(SimpleToken::SemiColon)?;
            Ok(Statement::Break(self.current(-1)))
        } else if self
            .my_match(&[SimpleToken::KeyWord(lexer::KeyWordType::Continue)])
            .is_some()
        {
            self.consume(SimpleToken::SemiColon)?;
            Ok(Statement::Continue(self.current(-1)))
        } else {
            let expression = self.expression()?;
            self.consume(SimpleToken::SemiColon)?;
            Ok(Statement::ExprStatement(expression))
        };

        s
    }

    fn whilestmt(&mut self) -> Result<Statement, ASTError> {
        self.consume(SimpleToken::LeftParen)?;
        let expr = self.expression()?;
        self.consume(SimpleToken::RightParen)?;
        let stmt = self.statement()?;
        Ok(Statement::Whileloop(expr, self.emit_statment(stmt)))
    }

    fn ifstmt(&mut self) -> Result<Statement, ASTError> {
        self.consume(SimpleToken::LeftParen)?;
        let mut expr = vec![self.expression()?];
        self.consume(SimpleToken::RightParen)?;
        let mut else_stmt = None;
        let mut stmt = vec![self.statement()?];
        while self
            .my_match(&[SimpleToken::KeyWord(lexer::KeyWordType::Elif)])
            .is_some()
        {
            self.consume(SimpleToken::LeftParen)?;
            expr.push(self.expression()?);
            self.consume(SimpleToken::RightParen)?;

            stmt.push(self.statement()?);
        }
        if self
            .my_match(&[SimpleToken::KeyWord(lexer::KeyWordType::Else)])
            .is_some()
        {
            let stmt = self.statement()?;
            else_stmt = Some(self.emit_statment(stmt));
        }
        Ok(Statement::IfStatement(
            expr.into_iter().zip(stmt.into_iter()).collect(),
            else_stmt,
        ))
    }

    fn vardecl(&mut self) -> Result<Declaration, ASTError> {
        self.consume(SimpleToken::KeyWord(lexer::KeyWordType::Var))?;
        let ident = self.advance()?.clone();
        self.consume(SimpleToken::Equal)?;
        let right = self.expression()?;
        match ident {
            Token::Identifier(s) => return Ok(Declaration::VarDecl(s, right)),
            // TODO: Expected identifier
            _ => Err(self.error_token(-3, vec![])),
        }
    }

    fn print(&mut self) -> Result<Statement, ASTError> {
        self.consume(SimpleToken::KeyWord(lexer::KeyWordType::Print))?;
        let mut to_print = vec![];
        self.consume(SimpleToken::LeftParen)?;
        loop {
            to_print.push(self.expression()?);
            if !self.my_match(&[SimpleToken::Comma]).is_some() {
                break;
            }
        }
        self.consume(SimpleToken::RightParen)?;
        Ok(Statement::PrintStatement(to_print))
    }
    fn expression(&mut self) -> Result<ExprID, ASTError> {
        self.binary_error_production()
    }

    fn binary_error_production(&mut self) -> Result<ExprID, ASTError> {
        use SimpleToken::*;
        if let Some(_) = self.my_match(&[
            Plus,
            Equal,
            BangEqual,
            Less,
            LessEqual,
            Greater,
            GreaterEqual,
            Star,
            Slash,
            Equal,
        ]) {
            return Err(ASTError::BinaryNoLeft(self.current(-1)));
        }
        self.assignment()
    }

    fn assignment(&mut self) -> Result<ExprID, ASTError> {
        let left = self.logical_and()?;
        if self.my_match(&[SimpleToken::Equal]).is_some() {
            let right = self.expression()?;
            match &self.ast.expr_arena[left] {
                Expr::Terminal(located_primary) => match &located_primary.inner {
                    Primary::Identifier(s) => {
                        return Ok(self.emit(Expr::Assignment(s.clone(), right)))
                    }
                    _ => return Err(ASTError::RValueAssignment(left)),
                },
                // No chaining assignment for now
                // Expr::Assignment(s, r) => todo!(),
                _ => return Err(ASTError::RValueAssignment(left)),
            }
        }
        Ok(left)
    }
    fn ternary(&mut self) -> Result<ExprID, ASTError> {
        let left = self.logical_and()?;

        if let Some(_) = self.my_match(&[SimpleToken::Question]) {
            let middle = self.expression()?;
            self.consume(SimpleToken::Colon)?;
            let right = self.expression()?;
            return Ok(self.emit(Expr::Ternary(Ternary {
                left,
                middle,
                right,
            })));
        }
        Ok(left)
    }

    fn logical_and(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.logical_or()?;

        while self.my_match(&[SimpleToken::And]).is_some() {
            let right = self.logical_and()?;

            left = self.emit(Expr::LogicalAnd(LogicalAnd { left, right }));
        }
        Ok(left)
    }

    fn logical_or(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.equality()?;

        while self.my_match(&[SimpleToken::Or]).is_some() {
            let right = self.logical_or()?;

            left = self.emit(Expr::LogicalOr(LogicalOr { left, right }));
        }
        Ok(left)
    }

    fn equality(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.comparison()?;
        while let Some(simple_token) =
            self.my_match(&[SimpleToken::EqualEqual, SimpleToken::BangEqual])
        {
            let operator = match simple_token {
                SimpleToken::BangEqual => Operator::NotEqual,
                SimpleToken::EqualEqual => Operator::Equal,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            left = self.emit(Expr::Binary(Binary {
                left,
                operator,
                right,
            }));
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.term()?;
        while let Some(simple_token) = self.my_match(&[
            SimpleToken::Greater,
            SimpleToken::GreaterEqual,
            SimpleToken::Less,
            SimpleToken::LessEqual,
        ]) {
            let operator = match simple_token {
                SimpleToken::GreaterEqual => Operator::GreaterEqual,
                SimpleToken::Greater => Operator::Greater,
                SimpleToken::LessEqual => Operator::LesserEqual,
                SimpleToken::Less => Operator::Lesser,
                _ => unreachable!(),
            };
            let right = self.term()?;
            left = self.emit(Expr::Binary(Binary {
                left,
                operator,
                right,
            }));
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.factor()?;
        while let Some(simple_token) = self.my_match(&[SimpleToken::Plus, SimpleToken::Minus]) {
            let operator = match simple_token {
                SimpleToken::Minus => Operator::Minus,
                SimpleToken::Plus => Operator::Plus,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            left = self.emit(Expr::Binary(Binary {
                left,
                operator,
                right,
            }));
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.unary()?;
        while let Some(simple_token) = self.my_match(&[SimpleToken::Star, SimpleToken::Slash]) {
            let operator = match simple_token {
                SimpleToken::Slash => Operator::Slash,
                SimpleToken::Star => Operator::Star,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            left = self.emit(Expr::Binary(Binary {
                left,
                operator,
                right,
            }));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<ExprID, ASTError> {
        if let Some(p) = self.my_match(&[SimpleToken::Bang, SimpleToken::Minus]) {
            let expr = self.unary()?;
            match p {
                SimpleToken::Bang => Ok(self.emit(Expr::Unary(Unary::Not(expr)))),
                SimpleToken::Minus => Ok(self.emit(Expr::Unary(Unary::Minus(expr)))),
                _ => return Err(self.error_token(-1, vec![SimpleToken::Bang, SimpleToken::Minus])),
            }
        } else {
            self.function_call()
        }
    }

    fn function_call(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.primary()?;

        while self.my_match(&[SimpleToken::LeftParen]).is_some() {
            let mut arguments = vec![];
            let mut arguments_count = 0;
            loop {
                if self.my_match(&[SimpleToken::RightParen]).is_some() {
                    left = self.emit(Expr::FunctionCall(FunctionCall {
                        func: left,
                        arguments: vec![],
                    }));
                    arguments.clear();
                    arguments_count += 1;
                    if arguments_count > 255 {
                        return Err(ASTError::TooManyArguments);
                    }
                    break;
                }
                arguments.push(self.expression()?);

                if self.my_match(&[SimpleToken::RightParen]).is_some() {
                    left = self.emit(Expr::FunctionCall(FunctionCall {
                        func: left,
                        arguments: arguments.clone(),
                    }));
                    arguments_count += 1;
                    if arguments_count > 255 {
                        return Err(ASTError::TooManyArguments);
                    }
                    arguments.clear();
                    break;
                }
                self.consume(SimpleToken::Comma)?;
            }
        }
        Ok(left)
    }
    fn primary(&mut self) -> Result<ExprID, ASTError> {
        if let Some(_) = self.my_match(&[SimpleToken::LeftParen]) {
            let expression = self.expression()?;
            self.consume(SimpleToken::RightParen)?;
            return Ok(expression);
        }
        match self.advance()?.clone() {
            Token::Single(token) => match token {
                lexer::SimpleToken::KeyWord(key_word_type) => match key_word_type {
                    lexer::KeyWordType::False => Ok(self.emit_primary(Primary::Boolean(false), 0)),
                    lexer::KeyWordType::True => Ok(self.emit_primary(Primary::Boolean(true), 0)),
                    lexer::KeyWordType::Nil => Ok(self.emit_primary(Primary::Nil, 0)),
                    _ => Err(self.error_token(
                        -1,
                        vec![
                            SimpleToken::KeyWord(lexer::KeyWordType::True),
                            SimpleToken::KeyWord(lexer::KeyWordType::False),
                            SimpleToken::KeyWord(lexer::KeyWordType::Nil),
                        ],
                    )),
                },
                _ => Err(self.error_token(
                    -1,
                    vec![
                        SimpleToken::KeyWord(lexer::KeyWordType::True),
                        SimpleToken::KeyWord(lexer::KeyWordType::False),
                        SimpleToken::KeyWord(lexer::KeyWordType::Nil),
                    ],
                )),
            },
            Token::Identifier(s) => Ok(self.emit_primary(Primary::Identifier(s), 0)),
            Token::StringLitteral(s) => Ok(self.emit_primary(Primary::String(s), 0)),
            Token::Number(n) => Ok(self.emit_primary(Primary::Number(n), 0)),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens.tokens[self.current_index].token
    }

    fn advance(&mut self) -> Result<&Token, ASTError> {
        if self.current_index == self.tokens.tokens.len() {
            return Err(ASTError::Eof);
        }
        self.current_index += 1;
        Ok(self.previous())
    }

    fn previous(&mut self) -> &Token {
        &self.tokens.tokens[self.current_index - 1].token
    }

    fn check(&self, token: &Token) -> bool {
        if self.current_index == self.tokens.tokens.len() {
            return false;
        }

        self.peek() == token
    }
    fn consume(&mut self, token: SimpleToken) -> Result<&Token, ASTError> {
        if self.check(&Token::Single(token)) {
            self.advance()
        } else {
            if self.current_index == self.tokens.tokens.len() {
                return Err(ASTError::Eof);
            }
            Err(self.error_token(0, vec![token]))
        }
    }

    fn my_match(&mut self, tokens: &[SimpleToken]) -> Option<SimpleToken> {
        assert!(!tokens.is_empty());
        for token in tokens {
            if self.check(&Token::Single(*token)) {
                let token = self.advance().unwrap();
                return match token {
                    Token::Single(simple_token) => Some(*simple_token),
                    _ => None,
                };
            }
        }
        None
    }

    pub fn parse(input: TokenVec) -> (AST, Vec<ASTError>) {
        let mut errors = vec![];
        let mut builder = Self {
            current_index: 0,
            tokens: input.clone(),
            ast: AST::new(input.clone()),
        };
        let mut recovery_mode = false;
        while builder.current_index.clone() < builder.tokens.tokens.len() {
            if recovery_mode {
                let token = builder.advance();
                if token.is_err() {
                    // Only possible error here is end of file
                    errors.push(ASTError::Eof);
                    return (builder.ast, errors);
                }
                if token.unwrap() == &Token::Single(SimpleToken::SemiColon) {
                    recovery_mode = false;
                }
            } else {
                let declaration = builder.declaration();
                match declaration {
                    Ok(declaration) => builder.ast.roots.push(declaration),
                    Err(err) => {
                        errors.push(err);
                        recovery_mode = true;
                    }
                }
            }
        }
        (builder.ast, errors)
    }
}
