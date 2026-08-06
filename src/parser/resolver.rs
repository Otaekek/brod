use std::collections::{HashMap, HashSet};

use crate::diagnostic::Span;
use crate::parser::parser::{
    AST, ASTError, Declaration, DeclarationID, Expr, ExprID, Primary, Statement, StatementID,
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
        name: &str,
        span: Span,
    ) -> Result<Option<(usize, usize)>, ASTError> {
        let mut scope = self.map.len();
        for vars in self.map.iter().rev() {
            if let Some(index) = vars.get(name) {
                return Ok(Some((*index, scope - 1)));
            }
            scope -= 1;
        }
        if self.globals.contains(name) {
            return Ok(None);
        }
        Err(ASTError::UndeclaredVariable(name.to_string(), span))
    }

    fn visit_expression(&mut self, ast: &mut AST, expr: ExprID) -> Result<(), ASTError> {
        // Global scope: Early return
        match ast.expr_arena[expr].clone() {
            Expr::Terminal(located_primary) => match located_primary.inner {
                Primary::Identifier(name) => {
                    let resolved = self.resolve_variable(&name, located_primary.span)?;
                    if let Some((index, scope)) = resolved {
                        ast.expr_arena[expr] =
                            Expr::Terminal(crate::parser::parser::LocatedPrimary {
                                inner: Primary::Local((index, scope)),
                                span: located_primary.span,
                            })
                    }
                }
                _ => (),
            },
            Expr::Unary(unary) => match unary {
                crate::parser::parser::Unary::Not(id) => self.visit_expression(ast, id)?,
                crate::parser::parser::Unary::Minus(id) => self.visit_expression(ast, id)?,
            },
            Expr::Binary(binary) => {
                self.visit_expression(ast, binary.left)?;
                self.visit_expression(ast, binary.right)?;
            }
            Expr::Ternary(_) => unreachable!(),
            Expr::LogicalAnd(logical_and) => {
                self.visit_expression(ast, logical_and.left)?;
                self.visit_expression(ast, logical_and.right)?;
            }
            Expr::LogicalOr(logical_or) => {
                self.visit_expression(ast, logical_or.left)?;
                self.visit_expression(ast, logical_or.right)?;
            }
            Expr::Assignment(id, id1) => {
                self.visit_expression(ast, id)?;
                self.visit_expression(ast, id1)?;
            }
            Expr::FunctionCall(function_call) => {
                match &ast.expr_arena[function_call.func] {
                    Expr::Terminal(located_primary) => match &located_primary.inner {
                        Primary::Identifier(s) => {
                            if let Some(index) = ast.functions.get(s) {
                                ast.expr_arena[function_call.func] = Expr::Terminal(
                                    Primary::FunctionId(*index).located(located_primary.span),
                                )
                            } else {
                                return Err(ASTError::Eof);
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                }
                for x in function_call.arguments {
                    self.visit_expression(ast, x)?;
                }
            }
            Expr::Get(id, _name) => {
                self.visit_expression(ast, id)?;
            }
        }
        Ok(())
    }
    fn visit_statement(&mut self, ast: &mut AST, statement: StatementID) -> Result<(), ASTError> {
        match &ast.statement_arena[statement] {
            Statement::ExprStatement(id) => {
                let id = *id;
                self.visit_expression(ast, id)?;
            }
            Statement::PrintStatement(ids) => {
                for id in ids.clone() {
                    self.visit_expression(ast, id)?;
                }
            }
            Statement::Block(block) => {
                let build_function_local = self.building_function;
                self.building_function = false;
                if !build_function_local {
                    self.map.push(HashMap::new());
                }
                let scope = self.map.len() - 1;
                let block = block.clone();
                for decl in block.declarations {
                    self.visit_declaration(ast, decl)?;
                }
                if !build_function_local {
                    self.map.pop();
                }
                if let Statement::Block(b) = &mut ast.statement_arena[statement] {
                    b.scope = scope;
                }
            }
            Statement::IfStatement(items, id) => {
                let id = id.clone();
                for (expr_id, statement_id) in items.clone() {
                    self.visit_expression(ast, expr_id)?;
                    self.visit_statement(ast, statement_id)?;
                }
                if let Some(id) = id {
                    self.visit_statement(ast, id)?;
                }
            }
            Statement::Whileloop(id, id1) => {
                let id = *id;
                let id1 = *id1;
                self.visit_expression(ast, id)?;
                self.visit_statement(ast, id1)?;
            }
            Statement::Break(_) => (),
            Statement::Continue(_) => (),
            Statement::Return(_, id) => {
                if let Some(id) = id {
                    let id = *id;
                    self.visit_expression(ast, id)?;
                }
            }
        };
        Ok(())
    }
    fn visit_declaration(
        &mut self,
        ast: &mut AST,
        declaration: DeclarationID,
    ) -> Result<(), ASTError> {
        match &ast.declaration_arena[declaration] {
            Declaration::Statement(statement) => {
                let statement = *statement;
                self.visit_statement(ast, statement)?;
            }
            Declaration::VarDecl(name, id) => {
                let id = *id;
                let name = name.clone();
                // Global scope, used mostly for the REPL: nothing to resolve.
                if self.map.is_empty() {
                    self.globals.insert(name);
                } else {
                    let offset = self.map.last().unwrap().len();
                    self.map.last_mut().unwrap().insert(name, offset);
                }
                self.visit_expression(ast, id)?;
            }
            Declaration::FunctionDefinition(function_definition) => {
                let arguments = function_definition.arguments.clone();
                let statement = function_definition.statement;
                let mut new_env = HashMap::new();
                for (i, arg) in arguments.iter().enumerate() {
                    if let Primary::String(s) = arg {
                        new_env.insert(s.to_string(), i);
                    } else {
                        unreachable!()
                    }
                }
                self.map.push(new_env);
                self.building_function = true;
                self.visit_statement(ast, statement)?;
                self.map.pop();
            }
            Declaration::ClassDefinition(_class_definition) => (),
            Declaration::Comment(_) => (),
            Declaration::Empty => (),
        };
        Ok(())
    }

    pub fn resolve(&mut self, ast: &mut AST) -> Result<(), ASTError> {
        for declaration in ast.roots.clone() {
            self.visit_declaration(ast, declaration)?;
        }
        Ok(())
    }
}
