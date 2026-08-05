use std::collections::HashMap;

use crate::parser::parser::{AST, Declaration, Expr, Statement};

#[derive(Clone, Debug)]
pub struct Resolved {
    name: String,
    index: usize,
}

pub struct Resolver<'a> {
    resolveds: Vec<Resolved>,
    map: HashMap<&'a str, Resolved>,
    scope: usize,
    next_id: usize,
}

impl<'a> Resolver<'a> {
    pub fn init() -> Self {
        Self {
            resolveds: vec![],
            map: HashMap::new(),
            scope: 0,
            next_id: 0,
        }
    }

    fn visit_expression(&mut self, ast: &mut AST, expr: Expr) {
        match expr {
            Expr::Terminal(located_primary) => todo!(),
            Expr::Unary(unary) => todo!(),
            Expr::Binary(binary) => todo!(),
            Expr::Ternary(ternary) => todo!(),
            Expr::LogicalAnd(logical_and) => todo!(),
            Expr::LogicalOr(logical_or) => todo!(),
            Expr::Assignment(id, id1) => todo!(),
            Expr::FunctionCall(function_call) => todo!(),
            Expr::Get(id, _) => todo!(),
        }
    }
    fn visit_statement(&mut self, ast: &mut AST, statement: Statement) {
        match statement {
            Statement::ExprStatement(id) => todo!(),
            Statement::PrintStatement(ids) => todo!(),
            Statement::Block(declarations) => {
                self.scope += 1;
                for decl in declarations {
                    self.visit_declaration(ast, decl.clone());
                }
                self.scope -= 1;
            }
            Statement::IfStatement(items, id) => todo!(),
            Statement::Whileloop(id, id1) => {
                self.visit_statement(ast, ast.statement_arena[id1].clone());
            }
            Statement::Break(token) => (),
            Statement::Continue(token) => (),
            Statement::Return(token, id) => {
                if id.is_some() {
                    // id.map(|x| self.visit_expression(&self.ast.expr_arena[*x]));
                }
            }
        };
    }
    fn visit_declaration(&mut self, ast: &mut AST, declaration: Declaration) {
        match declaration {
            Declaration::Statement(statement) => self.visit_statement(ast, statement.clone()),
            Declaration::VarDecl(name, id) => {
                // Global scope user mostly for REPL
                if self.scope == 0 {}
            }
            Declaration::FunctionDefinition(function_definition) => todo!(),
            Declaration::ClassDefinition(class_definition) => todo!(),
            Declaration::Comment(_) => (),
            Declaration::Empty => (),
        };
    }

    pub fn resolve(&mut self, ast: &mut AST) {
        for declaration in ast.roots.clone() {
            self.visit_declaration(ast, declaration);
        }
    }
}
