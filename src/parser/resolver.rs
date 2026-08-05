use std::collections::HashMap;

use crate::parser::parser::{
    AST, Declaration, DeclarationID, Expr, ExprID, Statement, StatementID,
};

pub struct Resolver {
    map: Vec<HashMap<String, usize>>,
    scope: usize,
    next_id: usize,
}

impl Resolver {
    pub fn init() -> Self {
        Self {
            map: vec![],
            scope: 0,
            next_id: 0,
        }
    }

    fn resolve_variable(&mut self, ast: &mut AST, expr: ExprID, name: &str) -> usize {
        let mut scope = self.scope;
        // println!("{}", scope);
        while (scope > 0) {
            let vars = &self.map[self.scope - 1];

            if let Some(index) = vars.get(name) {
                return *index;
            }
            scope -= 1;
        }
        unreachable!("Undefined variable {}", name);
    }

    fn visit_expression(&mut self, ast: &mut AST, expr: ExprID) {
        // Global scope: Early return
        if self.scope == 0 {
            return;
        }
        match ast.expr_arena[expr].clone() {
            Expr::Terminal(located_primary) => match located_primary.inner {
                crate::parser::parser::Primary::Identifier(name) => {
                    let index = self.resolve_variable(ast, expr, &name);
                    ast.expr_arena[expr] = Expr::Terminal(crate::parser::parser::LocatedPrimary {
                        inner: crate::parser::parser::Primary::Local(index),
                        span: located_primary.span,
                    })
                }
                _ => (),
            },
            Expr::Unary(unary) => match unary {
                crate::parser::parser::Unary::Not(id) => self.visit_expression(ast, id),
                crate::parser::parser::Unary::Minus(id) => self.visit_expression(ast, id),
            },
            Expr::Binary(binary) => {
                self.visit_expression(ast, binary.left);
                self.visit_expression(ast, binary.right);
            }
            Expr::Ternary(ternary) => unreachable!(),
            Expr::LogicalAnd(logical_and) => {
                self.visit_expression(ast, logical_and.left);
                self.visit_expression(ast, logical_and.right);
            }
            Expr::LogicalOr(logical_or) => {
                self.visit_expression(ast, logical_or.left);
                self.visit_expression(ast, logical_or.right);
            }
            Expr::Assignment(id, id1) => {
                self.visit_expression(ast, id);
                self.visit_expression(ast, id1);
            }
            Expr::FunctionCall(function_call) => {
                for x in function_call.arguments {
                    self.visit_expression(ast, x);
                }
            }
            Expr::Get(id, name) => {
                self.visit_expression(ast, id);
            }
        }
    }
    fn visit_statement(&mut self, ast: &mut AST, statement: StatementID) {
        match &ast.statement_arena[statement] {
            Statement::ExprStatement(id) => {
                self.visit_expression(ast, *id);
            }
            Statement::PrintStatement(ids) => {
                for id in ids.clone() {
                    self.visit_expression(ast, id);
                }
            }
            Statement::Block(declarations) => {
                let declarations = declarations.clone();
                self.scope += 1;

                while self.map.len() <= self.scope {
                    self.map.push(HashMap::new());
                }

                for decl in declarations.declarations {
                    self.visit_declaration(ast, decl);
                }
                self.next_id -= self.map[self.scope - 1].len();
                self.scope -= 1;
            }
            Statement::IfStatement(items, id) => {
                let id = id.clone();
                for (expr_id, statement_id) in items.clone() {
                    self.visit_expression(ast, expr_id);
                    self.visit_statement(ast, statement_id);
                }
                if let Some(id) = id {
                    self.visit_statement(ast, id);
                }
            }
            Statement::Whileloop(id, id1) => {
                let id1 = *id1;
                self.visit_statement(ast, id1);
            }
            Statement::Break(token) => (),
            Statement::Continue(token) => (),
            Statement::Return(token, id) => {
                if id.is_some() {
                    // id.map(|x| self.visit_expression(ast, x));
                }
            }
        };
    }
    fn visit_declaration(&mut self, ast: &mut AST, declaration: DeclarationID) {
        match &ast.declaration_arena[declaration] {
            Declaration::Statement(statement) => {
                let statement = *statement;
                self.visit_statement(ast, statement)
            }
            Declaration::VarDecl(name, id) => {
                // Global scope user mostly for REPL
                // Nothing to do
                if self.scope == 0 {
                } else {
                    self.map[self.scope - 1].insert(name.clone(), self.next_id);
                    self.next_id = self.next_id + 1;
                }
            }
            Declaration::FunctionDefinition(function_definition) => {
                self.visit_statement(ast, function_definition.statement);
            }
            Declaration::ClassDefinition(class_definition) => (),
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
