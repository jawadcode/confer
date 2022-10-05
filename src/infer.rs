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

/// The type inference engine
pub struct Engine {
    id: usize,
}

impl Engine {
    fn new_tyvar(&mut self) -> Type {
        let tyvar = Type::Var(self.id);
        self.id += 1;
        tyvar
    }
}
