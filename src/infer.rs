use std::{
    collections::HashMap,
    fmt::{self, Display},
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
    /// The substitution (set of mappings) from type variables (represented by `usize`) to `Type`s
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
                self.new_scope();
                self.extend(arg, &arg_ty);
                let body_ty = self.infer(body);
                self.exit_scope();
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
            (Type::Var(x_id), y) | (y, Type::Var(x_id)) => {
                // Get x's mapping from the substitution
                let map = self.subst[*x_id].clone();
                // If the mapping is simply to itself, check that x doesn't occur in y, then update the mapping to x ↦ y,
                if map == Type::Var(*x_id) {
                    assert!(!self.occurs_in(*x_id, y));
                    self.subst[*x_id] = y.clone();
                } else {
                    // otherwise, unify the type x maps to with y
                    self.unify(&map, y);
                }
            }
            (
                Type::Con {
                    name: x_name,
                    args: x_args,
                },
                Type::Con {
                    name: y_name,
                    args: y_args,
                },
            ) => {
                // Compare the equality of two type constructors by:
                // * Unifying the type arguments
                assert_eq!(x_name, y_name);
                // * Checking the arity
                assert_eq!(x_args.len(), y_args.len());
                // * Checking the names
                for (t1, t2) in x_args.iter().zip(y_args) {
                    self.unify(t1, t2);
                }
            }
        }
    }

    /// Checks if the `Type` mapped to by `index` in the substitution occurs in `ty`
    fn occurs_in(&self, index: usize, ty: &Type) -> bool {
        match ty {
            // Check if the type appears in any of the type arguments of `ty`
            Type::Con { name: _, args } => args.iter().any(|ty| self.occurs_in(index, ty)),
            Type::Var(id) => {
                // Get the `Type` mapped to by `id`
                let map = self.subst[*id].clone();
                // If the type mapped to is just a typevar of `index`, return true
                if map == Type::Var(index) {
                    true
                } else {
                    // otherwise check if the `Type` referred to by `index` occurs in the mapped to `Type`
                    self.occurs_in(index, &map)
                }
            }
        }
    }

    /// Apply `self.subst` to `ty`, returning the mapped `Type`
    pub fn substitute(&self, ty: Type) -> Type {
        match ty {
            // If the type variable doesn't map to itself, then follow the chain of type variables
            Type::Var(x) if self.subst[x] != Type::Var(x) => self.substitute(self.subst[x].clone()),
            // Apply the substitution to all of the type arguments
            Type::Con { name, args } => Type::Con {
                name,
                args: args.into_iter().map(|ty| self.substitute(ty)).collect(),
            },
            // There is no change incurred by the substitution
            t => t,
        }
    }

    // Get a variable by `name` from `self.env` starting from the innermost scope
    fn get_var(&self, name: &str) -> Option<Type> {
        for env in self.env.iter().rev() {
            match env.get(name) {
                Some(ty) => return Some(ty.clone()),
                None => continue,
            }
        }

        None
    }

    /// Print all of the current constraints separated by new lines
    pub fn print_constraints(&self) {
        println!("\nConstraints:");
        for constraint in self.constraints.iter() {
            println!("{constraint}");
        }
    }

    /// Print the current substitution separated by new lines
    pub fn print_subst(&self) {
        println!("\nSubstitution:");
        for (index, map) in self.subst.iter().enumerate() {
            println!("t{index} ↦ {map}");
        }
    }

    /// Extend the last scope on the `self.env` stack with `(name, ty)`
    fn extend(&mut self, name: &str, ty: &Type) {
        self.env
            .last_mut()
            .unwrap()
            .insert(name.to_string(), ty.clone());
    }

    /// Enter a new scope
    fn new_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    /// Exit the innermost scope by popping it from the `self.env` stack
    fn exit_scope(&mut self) {
        self.env.pop();
    }

    /// Generate a fresh type variable and add it to the list of substitutions
    fn new_tyvar(&mut self) -> Type {
        let tyvar = Type::Var(self.subst.len());
        self.subst.push(tyvar.clone());
        tyvar
    }
}
