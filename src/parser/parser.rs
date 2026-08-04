use std::fmt::Display;

use crate::arena::{Arena, Id};
use crate::diagnostic::{Diagnostic, Span, Stage};
use crate::lexer::lexer::{KeyWordType, SimpleToken, Token, TokenKind, TokenVec};

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
pub type ExprID = Id<Expr>;
pub type StatementID = Id<Statement>;

#[derive(Copy, Clone, Debug, enum_display::EnumDisplay)]
pub enum Unary {
    Not(ExprID),
    Minus(ExprID),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocatedPrimary {
    pub inner: Primary,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primary {
    MySelf,
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Nil,
}
impl Primary {
    pub fn located(self, span: Span) -> LocatedPrimary {
        LocatedPrimary { inner: self, span }
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
            Primary::MySelf => write!(f, "{}", "Self"),
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
pub struct FunctionDefinition {
    pub name: String,
    pub arguments: Vec<String>,
    pub statement: Statement,
}
#[derive(Clone, Debug)]
pub struct ClassDefinition {
    pub name: String,
    pub fields: Vec<String>,
    pub functions: Vec<FunctionDefinition>,
    pub constructor: FunctionDefinition,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Terminal(LocatedPrimary),
    Unary(Unary),
    Binary(Binary),
    Ternary(Ternary),
    LogicalAnd(LogicalAnd),
    LogicalOr(LogicalOr),
    Assignment(ExprID, ExprID),
    FunctionCall(FunctionCall),
    Get(ExprID, String),
}

#[derive(Clone, Debug)]
pub enum Declaration {
    Statement(Statement),
    VarDecl(String, ExprID),
    FunctionDefinition(FunctionDefinition),
    ClassDefinition(ClassDefinition),
    Comment(String),
    Empty,
}

#[derive(Clone, Debug)]
pub enum Statement {
    ExprStatement(ExprID),
    PrintStatement(Vec<ExprID>),
    Block(Vec<Declaration>),
    IfStatement(Vec<(ExprID, Statement)>, Option<StatementID>),
    Whileloop(ExprID, StatementID),
    Break(Token),
    Continue(Token),
    Return(Option<ExprID>),
}

#[derive(Clone, Debug, Default)]
pub struct AST {
    pub _tokens: TokenVec,
    pub expr_arena: Arena<Expr>,
    pub statement_arena: Arena<Statement>,
    pub roots: Vec<Declaration>,
}

impl AST {
    pub fn new(tokens: TokenVec) -> Self {
        Self {
            _tokens: tokens,
            expr_arena: Arena::with_capacity(4096),
            statement_arena: Arena::with_capacity(4096),
            roots: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum ASTError {
    Eof,
    NoConstructor(String),
    TooManyArguments,
    TokenError(Token, Vec<SimpleToken>),
    ExpectedExpression(Token),
    ExpectedIdentifier(Token),
    BinaryNoLeft(Token),
    RValueAssignment(ExprID),
}

impl Diagnostic for ASTError {
    fn stage(&self) -> Stage {
        Stage::Parsing
    }
    fn span(&self) -> Option<Span> {
        match self {
            ASTError::TokenError(located_token, _)
            | ASTError::ExpectedExpression(located_token)
            | ASTError::ExpectedIdentifier(located_token)
            | ASTError::BinaryNoLeft(located_token) => Some(located_token.span),
            ASTError::Eof
            | ASTError::NoConstructor(_)
            | ASTError::TooManyArguments
            | ASTError::RValueAssignment(_) => None,
        }
    }
    fn message(&self) -> String {
        match self {
            ASTError::Eof => "Unexpected End Of File".to_string(),
            ASTError::TokenError(located_token, expected) => {
                format!(
                    "Unexpected token \"{}\", expected {:?}",
                    located_token.kind, expected
                )
            }
            ASTError::ExpectedExpression(located_token) => format!(
                "Unexpected token \"{}\", expected an expression (a number, string, identifier, `(...)`, `self`, `true`, `false`, or `nil`)",
                located_token.kind
            ),
            ASTError::ExpectedIdentifier(located_token) => format!(
                "Unexpected token \"{}\", expected an identifier",
                located_token.kind
            ),
            ASTError::BinaryNoLeft(located_token) => format!(
                "Unexpected token \"{}\", this token should have a left and right side",
                located_token.kind
            ),
            ASTError::RValueAssignment(_) => {
                "Unexpected token, assignment should have a lvalue".to_string()
            }
            ASTError::TooManyArguments => {
                "Too many arguments for function call, maximum is 255".to_string()
            }
            ASTError::NoConstructor(name) => format!("Class {} has no constructor", name),
        }
    }
}

pub struct ASTBuilder<'a> {
    current_index: usize,
    tokens: TokenVec,
    ast: &'a mut AST,
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
impl<'a> ASTBuilder<'a> {
    fn emit(&mut self, expr: Expr) -> ExprID {
        self.ast.expr_arena.alloc(expr)
    }
    fn emit_statment(&mut self, statement: Statement) -> StatementID {
        self.ast.statement_arena.alloc(statement)
    }
    fn emit_primary(&mut self, primary: Primary, offset: i64) -> ExprID {
        // `advance()` increments `current_index` before returning the token
        // it consumed, so "the token this primary came from" is one behind.
        let span = self.tokens.tokens[(self.current_index as i64 - 1 + offset) as usize].span;
        self.ast.expr_arena.alloc(Expr::Terminal(LocatedPrimary {
            inner: primary,
            span,
        }))
    }

    fn current(&self, offset: i64) -> Token {
        self.tokens.tokens[(self.current_index as i64 + offset) as usize].clone()
    }
    fn is_last(&self) -> bool {
        self.current_index == self.tokens.tokens.len()
    }
    fn _peek_next(&self) -> Option<TokenKind> {
        if self.is_last() {
            None
        } else {
            Some(
                self.tokens.tokens[(self.current_index as i64 + 1) as usize]
                    .kind
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
        } else if matches!(
            &self.tokens.tokens[self.current_index].kind,
            TokenKind::Comment(_)
        ) {
            let text = match self.advance()? {
                TokenKind::Comment(s) => s.clone(),
                _ => unreachable!(),
            };
            return Ok(Declaration::Comment(text));
        } else if self.current(0).kind == TokenKind::Single(SimpleToken::KeyWord(KeyWordType::Var)) {
            let d = self.vardecl()?;
            self.consume(SimpleToken::SemiColon)?;
            Ok(d)
        } else if self.current(0).kind == TokenKind::Single(SimpleToken::KeyWord(KeyWordType::Fun)) {
            let d = self.function_def()?;
            Ok(d)
        } else if self.check(&TokenKind::Single(SimpleToken::KeyWord(KeyWordType::Class))) {
            self.class_definition()
        } else {
            let s = self.statement()?;
            Ok(Declaration::Statement(s))
        }
    }

    fn class_definition(&mut self) -> Result<Declaration, ASTError> {
        self.advance()?;

        let name = self.get_identifier_string()?;
        let mut fields = vec![];
        let mut functions = vec![];
        self.consume(SimpleToken::LeftBrace)?;
        let mut constructor: Option<FunctionDefinition> = None;
        loop {
            if self.my_match(&[SimpleToken::RightBrace]).is_some() {
                break;
            }
            if self.check(&TokenKind::Single(SimpleToken::KeyWord(KeyWordType::Fun))) {
                let function_def = self.function_def()?;
                let function_def = match function_def {
                    Declaration::FunctionDefinition(function_definition) => function_definition,
                    _ => unreachable!(),
                };
                if function_def.name == name {
                    constructor = Some(function_def);
                } else {
                    functions.push(function_def);
                }
            } else if self.check(&TokenKind::Single(SimpleToken::KeyWord(KeyWordType::Var))) {
                self.advance()?;
                fields.push(self.get_identifier_string()?);
                self.consume(SimpleToken::SemiColon)?;
            } else {
                return Err(ASTError::TokenError(
                    self.current(0),
                    vec![
                        SimpleToken::KeyWord(KeyWordType::Fun),
                        SimpleToken::KeyWord(KeyWordType::Var),
                    ],
                ));
            }
        }
        if constructor.is_none() {
            return Err(ASTError::NoConstructor(name));
        }
        Ok(Declaration::ClassDefinition(ClassDefinition {
            name,
            fields,
            functions,
            constructor: constructor.unwrap(),
        }))
    }

    fn function_def(&mut self) -> Result<Declaration, ASTError> {
        self.consume(SimpleToken::KeyWord(KeyWordType::Fun))?;
        let name = self.get_identifier_string()?;
        self.consume(SimpleToken::LeftParen)?;
        let mut arguments = vec![];
        while !self.my_match(&[SimpleToken::RightParen]).is_some() {
            let argument_name = self.get_identifier_string()?;
            arguments.push(argument_name);
            if self.peek() != &TokenKind::Single(SimpleToken::RightParen) {
                self.consume(SimpleToken::Comma)?;
            }
        }

        let block = self.block()?;
        Ok(Declaration::FunctionDefinition(FunctionDefinition {
            name,
            arguments,
            statement: block,
        }))
    }
    fn get_identifier_string(&mut self) -> Result<String, ASTError> {
        let primary = self.primary()?;
        let primary = &self.ast.expr_arena[primary];
        match primary {
            Expr::Terminal(located_primary) => match &located_primary.inner {
                Primary::Identifier(s) => Ok(s.clone()),
                _ => return Err(ASTError::ExpectedIdentifier(self.current(-1))),
            },
            _ => return Err(ASTError::ExpectedIdentifier(self.current(-1))),
        }
    }

    fn block(&mut self) -> Result<Statement, ASTError> {
        self.consume(SimpleToken::LeftBrace)?;
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
    }
    fn statement(&mut self) -> Result<Statement, ASTError> {
        let s = if self.check(&TokenKind::Single(SimpleToken::KeyWord(KeyWordType::Print))) {
            let r = self.print()?;
            self.consume(SimpleToken::SemiColon)?;
            Ok(r)
        } else if self.check(&TokenKind::Single(SimpleToken::LeftBrace)) {
            self.block()
        } else if self
            .my_match(&[SimpleToken::KeyWord(KeyWordType::If)])
            .is_some()
        {
            self.ifstmt()
        } else if self
            .my_match(&[SimpleToken::KeyWord(KeyWordType::While)])
            .is_some()
        {
            self.whilestmt()
        } else if self
            .my_match(&[SimpleToken::KeyWord(KeyWordType::Break)])
            .is_some()
        {
            let token = self.current(-1);
            self.consume(SimpleToken::SemiColon)?;
            Ok(Statement::Break(token))
        } else if self
            .my_match(&[SimpleToken::KeyWord(KeyWordType::Continue)])
            .is_some()
        {
            let token = self.current(-1);
            self.consume(SimpleToken::SemiColon)?;
            Ok(Statement::Continue(token))
        } else if self
            .my_match(&[SimpleToken::KeyWord(KeyWordType::Return)])
            .is_some()
        {
            let expr = if self.peek() == &TokenKind::Single(SimpleToken::SemiColon) {
                None
            } else {
                Some(self.expression()?)
            };
            let ret = Statement::Return(expr);
            self.consume(SimpleToken::SemiColon)?;
            Ok(ret)
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
            .my_match(&[SimpleToken::KeyWord(KeyWordType::Elif)])
            .is_some()
        {
            self.consume(SimpleToken::LeftParen)?;
            expr.push(self.expression()?);
            self.consume(SimpleToken::RightParen)?;

            stmt.push(self.statement()?);
        }
        if self
            .my_match(&[SimpleToken::KeyWord(KeyWordType::Else)])
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
        self.consume(SimpleToken::KeyWord(KeyWordType::Var))?;
        let ident = self.advance()?.clone();
        self.consume(SimpleToken::Equal)?;
        let right = self.expression()?;
        match ident {
            TokenKind::Identifier(s) => return Ok(Declaration::VarDecl(s, right)),
            _ => Err(ASTError::ExpectedIdentifier(self.current(-3))),
        }
    }

    fn print(&mut self) -> Result<Statement, ASTError> {
        self.consume(SimpleToken::KeyWord(KeyWordType::Print))?;
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
                    Primary::Identifier(_) => {
                        return Ok(self.emit(Expr::Assignment(left, right)));
                    }
                    _ => {
                        return Err(ASTError::RValueAssignment(left));
                    }
                },
                Expr::Get(_, _) => {
                    return Ok(self.emit(Expr::Assignment(left, right)));
                }
                _ => {
                    return Err(ASTError::RValueAssignment(left));
                }
            }
        }
        Ok(left)
    }
    fn _ternary(&mut self) -> Result<ExprID, ASTError> {
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
                _ => unreachable!(),
            }
        } else {
            self.function_call()
        }
    }

    fn function_call(&mut self) -> Result<ExprID, ASTError> {
        let mut left = self.primary()?;

        while let Some(last) = self.my_match(&[SimpleToken::LeftParen, SimpleToken::Dot]) {
            let mut arguments = vec![];
            let mut arguments_count = 0;
            loop {
                if last == SimpleToken::LeftParen {
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
                } else {
                    let right = self.get_identifier_string()?;
                    left = self.emit(Expr::Get(left, right));
                    break;
                }
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
            TokenKind::Single(SimpleToken::KeyWord(KeyWordType::MySelf)) => {
                Ok(self.emit_primary(Primary::MySelf, 0))
            }
            TokenKind::Single(token) => match token {
                SimpleToken::KeyWord(key_word_type) => match key_word_type {
                    KeyWordType::False => Ok(self.emit_primary(Primary::Boolean(false), 0)),
                    KeyWordType::True => Ok(self.emit_primary(Primary::Boolean(true), 0)),
                    KeyWordType::Nil => Ok(self.emit_primary(Primary::Nil, 0)),
                    _ => Err(ASTError::ExpectedExpression(self.current(-1))),
                },
                _ => Err(ASTError::ExpectedExpression(self.current(-1))),
            },
            TokenKind::Identifier(s) => Ok(self.emit_primary(Primary::Identifier(s), 0)),
            TokenKind::StringLitteral(s) => Ok(self.emit_primary(Primary::String(s), 0)),
            TokenKind::Number(n) => Ok(self.emit_primary(Primary::Number(n), 0)),
            TokenKind::Comment(_) => Err(ASTError::ExpectedExpression(self.current(-1))),
        }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens.tokens[self.current_index].kind
    }

    fn advance(&mut self) -> Result<&TokenKind, ASTError> {
        if self.current_index == self.tokens.tokens.len() {
            return Err(ASTError::Eof);
        }
        self.current_index += 1;
        Ok(self.previous())
    }

    fn previous(&mut self) -> &TokenKind {
        &self.tokens.tokens[self.current_index - 1].kind
    }

    fn check(&self, token: &TokenKind) -> bool {
        if self.current_index == self.tokens.tokens.len() {
            return false;
        }

        self.peek() == token
    }
    fn consume(&mut self, token: SimpleToken) -> Result<&TokenKind, ASTError> {
        if self.check(&TokenKind::Single(token)) {
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
            if self.check(&TokenKind::Single(*token)) {
                let token = self.advance().unwrap();
                return match token {
                    TokenKind::Single(simple_token) => Some(*simple_token),
                    _ => None,
                };
            }
        }
        None
    }

    pub fn parse(input: TokenVec, ast: &'a mut AST) -> ((), Vec<ASTError>) {
        let mut errors = vec![];
        let mut builder = Self {
            current_index: 0,
            tokens: input.clone(),
            ast,
        };
        let mut recovery_mode = false;
        while builder.current_index.clone() < builder.tokens.tokens.len() {
            if recovery_mode {
                let token = builder.advance();
                if token.is_err() {
                    // Only possible error here is end of file
                    errors.push(ASTError::Eof);
                    return ((), errors);
                }
                if token.unwrap() == &TokenKind::Single(SimpleToken::SemiColon) {
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
        ((), errors)
    }
}
