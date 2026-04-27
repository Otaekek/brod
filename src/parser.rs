use crate::lexer::{SimpleToken, Token, TokenVec};

use super::lexer;
use typed_arena::Arena;
pub enum Operator {
    Equal,
    NotEqual,
    Assignment,
    Lesser,
    LesserEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Slash,
    Star,
}

pub enum Unary<'a> {
    Not(&'a Expr<'a>),
    Minus(&'a Expr<'a>),
}

pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}
pub struct Binary<'a> {
    left: &'a Expr<'a>,
    operator: Operator,
    right: &'a Expr<'a>,
}

pub enum Expr<'a> {
    Literal(Literal),
    Unary(Unary<'a>),
    Binary(Binary<'a>),
}

pub struct AST<'a> {
    pub arena: Arena<Expr<'a>>,
    pub roots: Vec<&'a Expr<'a>>,
}

impl<'a> AST<'a> {
    pub fn new() -> Self {
        Self {
            arena: Arena::with_capacity(4096),
            roots: vec![],
        }
    }

    // fn visit(input: &'a Expr<'a>) {
    //     match input {
    //         Expr::Literal(literal) => todo!(),
    //         Expr::Unary(unary) => todo!(),
    //         Expr::Binary(binary) => todo!(),
    //     };
    // }
}

pub struct ASTBuilder {
    current_index: usize,
    tokens: TokenVec,
}

// expression → equality ;
// equality → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term → factor ( ( "-" | "+" ) factor )* ;
// factor → unary ( ( "/" | "*" ) unary )* ;
// unary → ( "!" | "-" ) unary
// | primary ;
// primary → NUMBER | STRING | "true" | "false" | "nil"
// | "(" expression ")"
//
impl<'a> ASTBuilder {
    fn comparison(&mut self) -> Expr<'a> {}
    fn equality(&mut self) -> Expr<'a> {
        let comparison = self.comparison();

        while (self.my_match(&[
            Token::Single(SimpleToken::Equal),
            Token::Single(SimpleToken::BangEqual),
        ])) {}
        comparison
    }
    fn peek(&self) -> &Token {
        &self.tokens.tokens[self.current_index].token
    }

    fn advance(&mut self) -> &Token {
        self.current_index += 1;
        self.previous()
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
    fn consume(&mut self, token: &Token) {
        if self.check(token) {
            self.advance();
        }
        // error
    }
    fn my_match(&mut self, tokens: &[Token]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }
    pub fn parse(mut input: lexer::TokenVec) -> AST<'a> {
        let mut ret = AST::new();
        for token in input.tokens.iter() {
            match token.token {
                lexer::Token::Single(simple_token) => todo!(),
                lexer::Token::StringLitteral(_) => todo!(),
                lexer::Token::Identifier(_) => todo!(),
                lexer::Token::Number(_) => todo!(),
            }
        }
        ret
    }
}
