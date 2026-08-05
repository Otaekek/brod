use std::{fs::read, path::PathBuf, process::exit};

use brod::{
    diagnostic::render,
    lexer::lexer,
    parser::parser::{
        AST, ASTBuilder, ClassDefinition, Declaration, DeclarationID, Expr, FunctionCall,
        FunctionDefinition, LocatedPrimary, Operator, Primary, Statement, Unary,
    },
};
use clap::Parser;

fn tabs(indent: usize) {
    for _ in 0..indent {
        print!("    ");
    }
}
fn display_primary(_ast: &AST, primary: &LocatedPrimary, _indent: usize) {
    match &primary.inner {
        Primary::Number(s) => print!("{}", s),
        Primary::String(s) => print!("\"{}\"", s),
        Primary::Boolean(s) => print!("{}", s),
        Primary::Identifier(s) => print!("{}", s),
        Primary::Nil => print!("nil"),
        Primary::MySelf => todo!(),
    };
}
fn display_expression(ast: &AST, expr: &Expr, indent: usize) {
    match expr {
        Expr::Terminal(located_primary) => display_primary(ast, located_primary, indent),
        Expr::Unary(unary) => match unary {
            Unary::Not(s) => {
                print!("!");
                let s = &ast.expr_arena[*s];
                display_expression(ast, s, indent);
            }
            Unary::Minus(s) => {
                print!("-");
                let s = &ast.expr_arena[*s];
                display_expression(ast, s, indent);
            }
        },
        Expr::Binary(binary) => {
            display_expression(ast, &ast.expr_arena[binary.left], indent);
            match binary.operator {
                Operator::Equal => print!(" == "),
                Operator::NotEqual => print!(" != "),
                Operator::Lesser => print!(" < "),
                Operator::LesserEqual => print!(" <= "),
                Operator::Greater => print!(" > "),
                Operator::GreaterEqual => print!(" >= "),
                Operator::Plus => print!(" + "),
                Operator::Minus => print!(" - "),
                Operator::Slash => print!(" / "),
                Operator::Star => print!(" = "),
            }
            display_expression(ast, &ast.expr_arena[binary.right], indent);
        }
        Expr::Ternary(_) => unreachable!(),
        Expr::LogicalAnd(logical_and) => {
            display_expression(ast, &ast.expr_arena[logical_and.left], indent);
            print!(" && ");
            display_expression(ast, &ast.expr_arena[logical_and.right], indent);
        }
        Expr::LogicalOr(logical_or) => {
            display_expression(ast, &ast.expr_arena[logical_or.left], indent);
            print!(" || ");
            display_expression(ast, &ast.expr_arena[logical_or.right], indent);
        }
        Expr::Assignment(left, right) => {
            print!("{} = ", left);
            display_expression(ast, &ast.expr_arena[*right], indent);
        }
        Expr::FunctionCall(function_call) => display_function_call(ast, &function_call, indent),
        Expr::Get(_expr, _name) => todo!(),
    }
}

fn display_function_call(ast: &AST, input: &FunctionCall, indent: usize) {
    display_expression(ast, &ast.expr_arena[input.func], indent);
    print!("(");
    for x in &input.arguments {
        display_expression(ast, &ast.expr_arena[*x], indent);
    }
    print!(")");
}

fn display_statement(ast: &AST, statement: &Statement, indent: usize) {
    // tabs(indent);
    match statement {
        Statement::ExprStatement(expr) => {
            display_expression(ast, &ast.expr_arena[*expr], indent);
            println!(";");
        }
        Statement::PrintStatement(items) => {
            print!("print(");
            for x in items {
                display_expression(ast, &ast.expr_arena[*x], indent);
            }
            println!(");");
        }
        Statement::Block(declarations) => {
            println!("{{");
            for x in declarations {
                display_decl(ast, *x, indent + 1);
            }
            tabs(indent);
            println!("}}");
        }
        Statement::IfStatement(items, block) => {
            print!("if (");
            let f = items.first().unwrap();
            display_expression(ast, &ast.expr_arena[f.0], indent);
            println!(")");
            display_statement(ast, &ast.statement_arena[f.1], indent);
            for f in &items[1..] {
                print!("else if (");
                display_expression(ast, &ast.expr_arena[f.0], indent);
                println!(")");
                display_statement(ast, &ast.statement_arena[f.1], indent);
            }
            if let Some(block) = block {
                display_statement(ast, &ast.statement_arena[*block], indent);
            }
        }
        Statement::Whileloop(cond, statement) => {
            print!("while (");
            display_expression(ast, &ast.expr_arena[*cond], indent);
            print!(") ");
            display_statement(ast, &ast.statement_arena[*statement], indent);
        }
        Statement::Break(_) => println!("break;"),
        Statement::Continue(_) => println!("continue;"),
        Statement::Return(_, expr_id) => {
            print!("return");
            if let Some(expr_id) = expr_id {
                display_expression(ast, &ast.expr_arena[*expr_id], indent);
            }
            println!(";");
        }
    };
}
fn display_function_definition(ast: &AST, function_definition: &FunctionDefinition, indent: usize) {
    print!("fn {} (", function_definition.name);

    for i in 0..function_definition.arguments.len() {
        let is_last = i == function_definition.arguments.len() - 1;
        if is_last {
            print!("{})", function_definition.arguments[i]);
        } else {
            print!("{},", function_definition.arguments[i]);
        }
    }
    print!(" ");
    display_statement(ast, &ast.statement_arena[function_definition.statement], indent);
}

fn display_class_definition(ast: &AST, class_definition: &ClassDefinition, indent: usize) {
    println!("class {} {{", class_definition.name);

    for v in &class_definition.fields {
        tabs(indent);
        println!("var {};", v);
    }
    for f in &class_definition.functions {
        tabs(indent);
        display_function_definition(ast, f, indent + 1);
    }
    tabs(indent);
    println!("}}");
}

fn display_decl(ast: &AST, declaration: DeclarationID, indent: usize) {
    tabs(indent);
    match &ast.declaration_arena[declaration] {
        Declaration::Statement(statement) => {
            display_statement(ast, &ast.statement_arena[*statement], indent)
        }
        Declaration::VarDecl(s, expr_id) => {
            print!("var {} = ", s);
            display_expression(ast, &ast.expr_arena[*expr_id], indent);
            println!(";");
        }
        Declaration::FunctionDefinition(function_definition) => {
            display_function_definition(ast, function_definition, indent);
        }
        Declaration::Comment(text) => println!("//{}", text),
        Declaration::Empty => (),
        Declaration::ClassDefinition(class_definition) => {
            display_class_definition(ast, class_definition, indent)
        }
    }
}
pub fn display_ast(ast: &AST) {
    for declaration in &ast.roots {
        display_decl(ast, *declaration, 0);
        println!("");
    }
}

#[derive(Clone, Debug, Parser)]
struct CliArgs {
    source_path: Option<PathBuf>,
}
pub fn main() {
    let args = CliArgs::parse();
    if let Some(path) = args.source_path {
        if path.extension().and_then(|ext| ext.to_str()) != Some("brod") {
            eprintln!("Error: Only .brod files are accepted");
            exit(1);
        }
        let buf = read(&path).unwrap();
        let source = String::from_utf8(buf).expect("Only utf-8 encoding is accepted");
        let mut ast = AST::default();
        let source_name = path.display().to_string();
        let tokens = lexer::lex(source.to_owned());
        if let Err(err) = tokens {
            eprintln!("{}", render(&err, &source, &source_name));
        } else if let Ok(tokens) = tokens {
            let res = ASTBuilder::parse(tokens, &mut ast);

            for err in &res.1 {
                eprintln!("{}", render(err, &source, &source_name));
            }
            if res.1.is_empty() {
                display_ast(&ast);
            }
        }
    }
}
