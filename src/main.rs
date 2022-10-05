use expr::Expr;

mod expr;
mod infer;

fn main() {
    let test = Expr::Abs(
        "f".to_string(),
        Box::new(Expr::Abs(
            "g".to_string(),
            Box::new(Expr::Abs(
                "x".to_string(),
                Box::new(Expr::App(
                    Box::new(Expr::Var("g".to_string())),
                    Box::new(Expr::App(
                        Box::new(Expr::Var("f".to_string())),
                        Box::new(Expr::Var("x".to_string())),
                    )),
                )),
            )),
        )),
    );

    println!("{test}");
}
