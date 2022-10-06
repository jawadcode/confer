use std::collections::HashMap;

use expr::Expr;

use crate::infer::{Engine, Type};

mod expr;
mod infer;

fn main() {
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
    println!("Type:\n{ty}");
    engine.print_constraints();
}
