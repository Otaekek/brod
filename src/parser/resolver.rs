use std::collections::{HashMap, HashSet};

use crate::parser::parser::{
    AST, Declaration, DeclarationID, Expr, ExprID, Primary, Statement, StatementID,
};

pub struct Resolver {
    map: Vec<HashMap<String, usize>>,
    globals: HashSet<String>,
    building_function: bool,
}

impl Resolver {
    pub fn init() -> Self {
        Self {
            map: vec![],
            globals: HashSet::new(),
            building_function: false,
        }
    }

    fn resolve_variable(
        &mut self,
        _ast: &mut AST,
        _expr: ExprID,
        name: &str,
    ) -> Option<(usize, usize)> {
        let mut scope = self.map.len();
        for vars in self.map.iter().rev() {
            if let Some(index) = vars.get(name) {
                return Some((*index, scope));
            }
            scope -= 1;
        }
        if let Some(_) = self.globals.get(name) {
            return None;
        }
        unreachable!("Undefined variable {}", name);
    }

    fn visit_expression(&mut self, ast: &mut AST, expr: ExprID) {
        // Global scope: Early return
        if self.map.is_empty() {
            return;
        }

        match ast.expr_arena[expr].clone() {
            Expr::Terminal(located_primary) => match located_primary.inner {
                crate::parser::parser::Primary::Identifier(name) => {
                    let resolved = self.resolve_variable(ast, expr, &name);
                    if let Some((index, scope)) = resolved {
                        ast.expr_arena[expr] =
                            Expr::Terminal(crate::parser::parser::LocatedPrimary {
                                inner: crate::parser::parser::Primary::Local((index, scope - 1)),
                                span: located_primary.span,
                            })
                    }
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
            Expr::Ternary(_) => unreachable!(),
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
            Expr::Get(id, _name) => {
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
            Statement::Block(block) => {
                let build_function_local = self.building_function;
                self.building_function = false;
                if !build_function_local {
                    self.map.push(HashMap::new());
                }
                let block = block.clone();
                for decl in block.declarations {
                    self.visit_declaration(ast, decl);
                }
                if !build_function_local {
                    self.map.pop();
                }
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
                self.visit_expression(ast, id.clone());
                self.visit_statement(ast, id1.clone());
            }
            Statement::Break(_) => (),
            Statement::Continue(_) => (),
            Statement::Return(_, id) => {
                if let Some(id) = id {
                    self.visit_expression(ast, *id);
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
                if self.map.is_empty() {
                    self.globals.insert(name.to_string());
                } else {
                    // if self.map.last().unwrap().get(name).is_some() {
                    //     println!("Shadowing !");
                    // }
                    let offset = self.map.last().unwrap().len();
                    self.map.last_mut().unwrap().insert(name.clone(), offset);
                }
                self.visit_expression(ast, *id);
            }
            Declaration::FunctionDefinition(function_definition) => {
                let mut new_env = HashMap::new();
                let mut i = 0;
                for arg in &function_definition.arguments {
                    if let Primary::String(s) = arg {
                        new_env.insert(s.to_string(), i);
                    } else {
                        unreachable!()
                    }
                    i = i + 1;
                }
                self.map.push(new_env);
                self.building_function = true;
                self.visit_statement(ast, function_definition.statement);
                self.map.pop();
            }
            Declaration::ClassDefinition(_class_definition) => (),
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
