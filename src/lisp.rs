#[deriving(Show)]
pub enum Expr {
    Sexpr(Vec<Expr>),
    Atom(Atom)
}

#[deriving(Show)]
pub enum Atom {
    Int(int),
    Symbol(String)
}

