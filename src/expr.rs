use std::fmt::{self, Display, Write};

#[derive(Debug, Clone, PartialEq)]
/// An expression node
pub enum Expr {
    // A variable
    Var(String),

    // A function application, i.e. a function call
    App(Box<Self>, Box<Self>),

    // A function abstraction, i.e. an anonymous function
    Abs(String, Box<Self>),
    // // A let-binding
    // Let(String, Box<Self>, Box<Self>),

    // // A literal
    // Lit(Lit),
}

/*
#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    String(String),
    Number(f64),
    Bool(bool),
}
 */

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_sexpr(f, 0)
    }
}

const TAB: &'static str = "|   ";

impl Expr {
    fn fmt_sexpr(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        f.write_str(&TAB.repeat(depth))?;

        match self {
            Expr::Var(var) => f.write_str(var),
            Expr::App(fun, arg) => {
                f.write_str("(app\n")?;
                fun.as_ref().fmt_sexpr(f, depth + 1)?;
                f.write_char('\n')?;
                arg.as_ref().fmt_sexpr(f, depth + 1)?;
                f.write_char(')')
            }
            Expr::Abs(arg, body) => {
                f.write_str("(fun\n")?;
                f.write_str(&TAB.repeat(depth + 1))?;
                f.write_str(arg)?;
                f.write_char('\n')?;
                body.as_ref().fmt_sexpr(f, depth + 1)?;
                f.write_char(')')
            }
        }
    }
}
