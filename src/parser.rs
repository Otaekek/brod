use std::fmt::Display;

use crate::lexer::{LocatedToken, SimpleToken, Token, TokenVec};

use super::lexer;
#[derive(Copy, Clone, Debug)]
pub enum Operator {
    Equal,
    NotEqual,
    // Assignment,
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
#[derive(Clone, Debug)]
pub enum Unary {
    Not(ExprID),
    Minus(ExprID),
}

#[derive(Clone, Debug)]
pub enum Terminal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
    SemiColon,
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
pub enum Expr {
    Terminal(Terminal),
    Unary(Unary),
    Binary(Binary),
    Ternary(Ternary),
    Statement(ExprID),
}

#[derive(Clone, Debug)]
pub struct AST {
    pub arena: Vec<Expr>,
    pub roots: Vec<ExprID>,
}

impl AST {
    pub fn new() -> Self {
        Self {
            arena: Vec::with_capacity(4096),
            roots: vec![],
        }
    }

    pub fn traverse_lrn<T: ASTVisitor>(&self, input: ExprID, visitor: &mut T) {
        match &self.arena[input] {
            Expr::Terminal(literal) => visitor.visit_literal(&literal),
            Expr::Unary(unary) => visitor.visit_unary(&self.arena, &unary),
            Expr::Binary(binary) => {
                self.traverse_lrn(binary.left, visitor);
                self.traverse_lrn(binary.right, visitor);
                visitor.visit_binary(&self.arena, &binary);
            }
            Expr::Statement(expr_id) => self.traverse_lrn(*expr_id, visitor),
            Expr::Ternary(ternary) => {
                self.traverse_lrn(ternary.left, visitor);
                self.traverse_lrn(ternary.middle, visitor);
                self.traverse_lrn(ternary.right, visitor);
                visitor.visit_ternary(&self.arena, &ternary);
            }
        };
    }
}
pub trait ASTVisitor {
    fn visit_ternary(&mut self, arena: &[Expr], ternary: &Ternary);
    fn visit_binary(&mut self, arena: &[Expr], binary: &Binary);
    fn visit_literal(&mut self, literal: &Terminal);
    fn visit_unary(&mut self, arena: &[Expr], unary: &Unary);
}

#[derive(Debug)]
pub enum ASTError {
    Eof,
    TokenError(LocatedToken),
    BinaryNoLeft(LocatedToken),
}

impl Display for ASTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ASTError::Eof => write!(f, "{}", "Unexpected End Of File"),
            ASTError::TokenError(located_token) => write!(
                f,
                "Unexpected token \"{}\" at {}:{}",
                located_token.token, located_token.line, located_token.row
            ),
            ASTError::BinaryNoLeft(located_token) => write!(
                f,
                "Unexpected token \"{}\" at {}:{}, This Token should have a left and right side",
                located_token.token, located_token.line, located_token.row
            ),
        }
    }
}
pub struct ASTBuilder {
    current_index: usize,
    tokens: TokenVec,
    ast: AST,
}

/// Statement → expression ;
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
        self.ast.arena.push(expr);
        self.ast.arena.len() - 1
    }
    fn emit_terminal(&mut self, terminal: Terminal) -> ExprID {
        self.ast.arena.push(Expr::Terminal(terminal));
        self.ast.arena.len() - 1
    }

    fn current(&self, offset: i64) -> LocatedToken {
        self.tokens.tokens[(self.current_index as i64 + offset) as usize].clone()
    }
    fn error_token(&self, token_offset: i64) -> ASTError {
        let token = self.tokens.tokens[(self.current_index as i64 + token_offset) as usize].clone();
        ASTError::TokenError(token)
    }
    fn statement(&mut self) -> Result<ExprID, ASTError> {
        let expression = self.expression()?;
        if let Some(_) = self.my_match(&[SimpleToken::SemiColon]) {
            return Ok(self.emit(Expr::Statement(expression)));
        }

        Ok(expression)
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
        self.ternary()
    }

    fn ternary(&mut self) -> Result<ExprID, ASTError> {
        let left = self.equality()?;

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
                _ => return Err(self.error_token(-1)),
            }
        } else {
            self.primary()
        }
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
                    lexer::KeyWordType::False => Ok(self.emit_terminal(Terminal::False)),
                    lexer::KeyWordType::True => Ok(self.emit_terminal(Terminal::True)),
                    lexer::KeyWordType::Nil => Ok(self.emit_terminal(Terminal::Nil)),
                    _ => Err(self.error_token(-1)),
                },
                _ => Err(self.error_token(-1)),
            },
            Token::StringLitteral(s) => Ok(self.emit_terminal(Terminal::String(s))),
            Token::Identifier(_) => Err(self.error_token(-1)),
            Token::Number(n) => Ok(self.emit_terminal(Terminal::Number(n))),
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
    // fn previous_simple(&mut self) -> Result<SimpleToken, ASTError> {
    //     let token = &self.tokens.tokens[self.current_index - 1].token;
    //     match token {
    //         Token::Single(simple_token) => Ok(*simple_token),
    //         Token::StringLitteral(_) => Err(self.error_token(-1)),
    //         Token::Identifier(_) => Err(self.error_token(-1)),
    //         Token::Number(_) => Err(self.error_token(-1)),
    //     }
    // }

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
            Err(self.error_token(0))
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
            tokens: input,
            ast: AST::new(),
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
                let expression = builder.statement();
                match expression {
                    Ok(expression) => builder.ast.roots.push(expression),
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
