use std::{
    collections::HashMap,
    io::{self, Write},
};

use expr::Expr;

use crate::{
    infer::{Engine, Type},
    parser::Parser,
};

mod expr;
mod infer;
mod lexer;
mod parser;

fn main1() {
    let test = Expr::App(
        Box::new(Expr::Abs(
            "x".to_string(),
            Box::new(Expr::Var("x".to_string())),
        )),
        Box::new(Expr::Var("1".to_string())),
    );

    let int = Type::Con {
        name: "Int".to_string(),
        args: vec![],
    };
    let boolean = Type::Con {
        name: "Bool".to_string(),
        args: vec![],
    };

    let env = [
        ("1", int.clone()),
        ("2", int.clone()),
        ("3", int.clone()),
        ("true", boolean.clone()),
        ("false", boolean.clone()),
    ]
    .into_iter()
    .map(|(name, ty)| (name.to_string(), ty))
    .collect::<HashMap<_, _>>();

    let mut engine = Engine::new(env);
    let ty = engine.infer(&test);
    println!("{test}");
    println!("\nUnsolved type:\n{ty}");
    engine.print_constraints();
    engine.solve_constraints();
    engine.print_subst();
    println!("\nSolved type:\n{}", engine.substitute(ty));
}

fn main() {
    repl()
}

fn repl() {
    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        match Parser::new(&input).parse() {
            Ok(ast) => println!("{ast}"),
            Err(err) => eprintln!("{err}"),
        }
    }
}
