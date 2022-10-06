use std::{
    collections::HashMap,
    fmt::{self, Display, Write},
};

use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A type constructor/type function, includes:
    /// * Generic/Polymorphic type constructors such as `List[Int]`
    ///     * Includes function types such as `Int -> Int` (or rather, Fun[Int, Int])
    /// * Nullary type constructors (concrete/monomorphic types) such as `Int`
    Con { name: String, args: Vec<Type> },

    /// An unconstrained type
    Var(usize),
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Con { name, args } => write!(
                f,
                "{name}{}",
                if args.is_empty() {
                    String::new()
                } else {
                    format!(
                        "[{}]",
                        args.iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            ),
            Type::Var(var) => write!(f, "t{var}"),
        }
    }
}

/// The type inference engine
pub struct Engine {
    /// The set of mappings from type variables (represented by `usize`) and `Type`s
    subst: Vec<Type>,
    /// The set of relationships between the `Type`s
    constraints: Vec<Constraint>,
    /// A stack of scopes
    env: Vec<HashMap<String, Type>>,
}

/// The relationship between 2 `Type`s
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// An equality relationship
    Eq(Type, Type),
}

impl Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constraint::Eq(t1, t2) => write!(f, "{t1} == {t2}"),
        }
    }
}

impl Engine {
    pub fn new(env: HashMap<String, Type>) -> Self {
        Self {
            subst: Vec::new(),
            constraints: Vec::new(),
            env: vec![env],
        }
    }

    /// Recursively walk `expr`, inferring immediately apparent types
    pub fn infer(&mut self, expr: &Expr) -> Type {
        match expr {
            /*
             * var : a ∈ env
             * -------------
             * env ⊢ var : a
             *
             * Description:
             * IF,  `var` of type `a` is contained within `env`
             * THEN, given the environment `env`, `var` has a type of `a`
             */
            Expr::Var(var) => self.get_var(var).unwrap(),
            /*
             * env ⊢ fun : arg_ty -> fun_out_ty  env ⊢ arg : arg_ty
             * ----------------------------------------------------
             *             env ⊢ fun arg : fun_out_ty
             *
             * Description:
             * IF,   given the environment `env`, `fun` has the type `arg_ty -> fun_out_ty`, and `arg` has the type `arg_ty`
             * THEN, given the environment `env`, `fun(arg)` has the type `fun_out_ty`
             */
            Expr::App(fun, arg) => {
                let fun_ty = self.infer(fun);
                let arg_ty = self.infer(arg);
                let fun_out_ty = self.new_tyvar();
                self.constraints.push(Constraint::Eq(
                    fun_ty,
                    Type::Con {
                        name: "Fun".to_string(),
                        args: vec![arg_ty, fun_out_ty.clone()],
                    },
                ));
                fun_out_ty
            }
            /*
             *   env, arg : arg_ty ⊢ body : body_ty
             * --------------------------------------
             * env ⊢ λ arg . body : arg_ty -> body_ty
             *
             * Description:
             * IF,   given the environment `env` extended with `arg : arg_ty`, `body` has the type `body_ty`
             * THEN, given the environment `env`, `(arg) => body` has the type `arg_ty -> body_ty`
             */
            Expr::Abs(arg, body) => {
                let arg_ty = self.new_tyvar();
                self.push_scope();
                self.extend(arg, &arg_ty);
                let body_ty = self.infer(body);
                self.pop_scope();
                // `Fun[arg_ty, body_ty]` aka `arg_ty -> body_ty`
                Type::Con {
                    name: "Fun".to_string(),
                    args: vec![arg_ty, body_ty],
                }
            }
        }
    }

    /// Solve all of the generated constraints
    pub fn solve_constraints(&mut self) {
        let constraints = self.constraints.clone();
        for constraint in constraints {
            match constraint {
                Constraint::Eq(t1, t2) => self.unify(&t1, &t2),
            }
        }
        // All of the constraints are solved, and no longer needed
        self.constraints.clear();
    }

    /// Check that `t1 == t2`, updating the substitutions in the process
    fn unify(&mut self, t1: &Type, t2: &Type) {
        match (t1, t2) {
            // If the IDs of 2 type variables are equal then they are already unified
            (Type::Var(x), Type::Var(y)) if x == y => (),
            _ => todo!("Add other 2 cases"),
        }
    }

    fn get_var(&self, name: &str) -> Option<Type> {
        for env in self.env.iter().rev() {
            match env.get(name) {
                Some(ty) => return Some(ty.clone()),
                None => continue,
            }
        }

        None
    }

    fn extend(&mut self, name: &str, ty: &Type) {
        self.env
            .last_mut()
            .unwrap()
            .insert(name.to_string(), ty.clone());
    }

    fn push_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.env.pop();
    }

    /// Generate a fresh type variable and add it to the list of substitutions
    fn new_tyvar(&mut self) -> Type {
        let tyvar = Type::Var(self.subst.len());
        self.subst.push(tyvar.clone());
        tyvar
    }

    pub fn print_constraints(&self) {
        println!("Constraints:");
        for constraint in self.constraints.iter() {
            println!("{constraint}");
        }
    }
}
