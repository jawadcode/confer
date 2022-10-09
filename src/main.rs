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

fn main() {
    let int = Type::Con {
        name: "Int".to_string(),
        args: vec![],
    };
    let boolean = Type::Con {
        name: "Bool".to_string(),
        args: vec![],
    };

    let env = [
        ("one", int.clone()),
        ("two", int.clone()),
        ("three", int.clone()),
        ("true", boolean.clone()),
        ("false", boolean.clone()),
    ]
    .into_iter()
    .map(|(name, ty)| (name.to_string(), ty))
    .collect::<HashMap<_, _>>();

    repl(env);
}

fn repl(env: HashMap<String, Type>) {
    loop {
        let mut input = String::new();
        print!("$ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let ast = match Parser::new(&input).parse() {
            Ok(ast) => ast,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };

        let mut engine = Engine::new(env.clone());
        let ty = engine.infer(&ast);
        println!("{ast}");
        println!("\nUnsolved type:\n{ty}");
        engine.print_constraints();
        engine.solve_constraints();
        engine.print_subst();
        println!("\nSolved type:\n{}", engine.substitute(ty));
    }
}
