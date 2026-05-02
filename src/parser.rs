use crate::lexer::{Token, TokenVec};

use super::lexer;
#[derive(Copy, Clone, Debug)]
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
pub type ExprID = usize;
#[derive(Clone, Debug)]
pub enum Unary {
    Not(ExprID),
    Minus(ExprID),
}

#[derive(Clone, Debug)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Nil,
}
#[derive(Clone, Debug)]
pub struct Binary {
    pub left: ExprID,
    pub operator: Operator,
    pub right: ExprID,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Literal),
    Unary(Unary),
    Binary(Binary),
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

    fn traverse_post<T: ASTVisitor>(&self, input: ExprID, visitor: &mut T) {
        match &self.arena[input] {
            Expr::Literal(literal) => visitor.visit_literal(&literal),
            Expr::Unary(unary) => visitor.visit_unary(&self.arena, &unary),
            Expr::Binary(binary) => {
                self.traverse_post(binary.right, visitor);
                self.traverse_post(binary.left, visitor);
                visitor.visit_binary(&self.arena, &binary);
            }
        };
    }
    fn traverse_pre<T: ASTVisitor>(&self, input: ExprID, visitor: &mut T) {
        match &self.arena[input] {
            Expr::Literal(literal) => visitor.visit_literal(&literal),
            Expr::Unary(unary) => visitor.visit_unary(&self.arena, &unary),
            Expr::Binary(binary) => {
                visitor.visit_binary(&self.arena, &binary);
                self.traverse_post(binary.right, visitor);
                self.traverse_post(binary.left, visitor);
            }
        };
    }
}
pub trait ASTVisitor {
    fn visit_binary(&mut self, arena: &[Expr], binary: &Binary);
    fn visit_literal(&mut self, literal: &Literal);
    fn visit_unary(&mut self, arena: &[Expr], unary: &Unary);
}
pub struct ASTBuilder {
    current_index: usize,
    tokens: TokenVec,
    ast: AST,
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
impl ASTBuilder {
    // fn expression(&mut self) -> Expr<'a> {
    //     self.comparison()
    // }
    // fn comparison(&mut self) -> Expr<'a> {
    //     let term = self.term();
    //     term
    // }

    fn emit(&mut self, expr: Expr) -> ExprID {
        self.ast.arena.push(expr);
        self.ast.arena.len() - 1
    }
    // fn equality(&mut self) -> Expr<'a> {
    //     let comparison = self.comparison();
    //     //     while (self.my_match(&[
    //         Token::Single(SimpleToken::Equal),
    //         Token::Single(SimpleToken::BangEqual),
    //     ])) {
    //
    // let operator = previous;
    //         let right = self.comparison();
    //         let expr = Expr::Binary(Binary { left: comparison, operator: (), right: right };
    //     }
    //     comparison
    // }
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
    pub fn parse(input: TokenVec) -> AST {
        let mut ret = Self {
            current_index: 0,
            tokens: input,
            ast: AST::new(),
        };
        ret.ast
    }
}
