use builtin;
use std::fmt;
use std::collections::HashMap;

#[deriving(Clone)]
pub enum Expr {
    Sexpr(Vec<Expr>),
    Qexpr(Box<Expr>),
    Atom(Atom)
}

type Builtin = fn(Vec<Expr>) -> Result<Expr, Error>;

pub struct Context {
    table: HashMap<&'static str, Expr>
}

impl Context {
    pub fn global() -> Context {
        let mut tbl = HashMap::new();

        tbl.insert("+",    Expr::Atom(Atom::Builtin(builtin::add)));
        tbl.insert("-",    Expr::Atom(Atom::Builtin(builtin::sub)));
        tbl.insert("*",    Expr::Atom(Atom::Builtin(builtin::mul)));
        tbl.insert("/",    Expr::Atom(Atom::Builtin(builtin::div)));
        tbl.insert("list", Expr::Atom(Atom::Builtin(builtin::list)));
        tbl.insert("tail", Expr::Atom(Atom::Builtin(builtin::tail)));
        tbl.insert("head", Expr::Atom(Atom::Builtin(builtin::head)));
        tbl.insert("eval", Expr::Atom(Atom::Builtin(builtin::eval)));

        Context { table: tbl }
    }
}

#[deriving(Clone)]
pub enum Atom {
    Int(int),
    Sym(String),
    Builtin(Builtin)
}

impl fmt::Show for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Atom::Int(ref v) => write!(f, "{}", v),
            Atom::Sym(ref v) => write!(f, "{}", v),
            Atom::Builtin(_) => write!(f, "<function>")
        }
    }
}

impl fmt::Show for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Expr::Sexpr(ref v) => {
                let cs = v
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .connect(" ");

                write!(f, "({})", cs)
            },
            Expr::Atom(ref a) => {
                write!(f, "{}", a)
            },
            Expr::Qexpr(ref e) => {
                write!(f, "{}", e)
            }
        }
    }
}

#[allow(dead_code)]
#[deriving(Show)]
pub enum Error {
    Runtime(String),
    ZeroDivision,
    NameResolution(String),
    InvalidType(String),
    Arity(String)
}

fn lookup(name: &str, ctx: &Context) -> Option<Expr> {
    match ctx.table.get(name) {
        Some(e) => Some(e.clone()),
        None => None
    }
}

fn call(name: &str, xs: &[Expr], ctx: &Context) -> Result<Expr, Error> {
    match lookup(name, ctx) {
        Some(a) => {
            match a {
                Expr::Atom(Atom::Builtin(f)) => f(xs.to_vec()),
                _ => Err(Error::InvalidType(format!("`{}` not a function", a)))
            }
        },

        None => Err(Error::NameResolution(format!("`{}` not in current context", name.to_string())))
    }
}

pub fn sexpr(l: Vec<Expr>, ctx: &Context) -> Result<Expr, Error> {
    match l.as_slice() {
        [] => Ok(Expr::Sexpr(Vec::new())),

        [Expr::Atom(Atom::Sym(ref name)), xs..] => {
            match eval_all(xs, ctx) {
                Ok(args) => call(name.as_slice(), args.as_slice(), ctx),
                Err(e) => Err(e)
            }
        },

        [ref e, _..] => Err(Error::InvalidType(format!("`{}` not a function", e)))
    }
}

pub fn atom(a: Atom, ctx: &Context) -> Result<Expr, Error> {
    match a {
        Atom::Sym(sym) => {
            match lookup(sym.as_slice(), ctx) {
                Some(v) => Ok(v),
                None => Err(Error::NameResolution(format!("`{}` not in current context", sym.to_string())))
            }
        },

        _ => Ok(Expr::Atom(a))
    }
}

fn eval_all(list: &[Expr], ctx: &Context) -> Result<Vec<Expr>, Error> {
    list.iter()
        .fold(Ok::<Vec<Expr>, Error>(Vec::new()), |m, e| {
            match m {
                Ok(mut r) => {
                    r.push(try!(eval(e.clone(), ctx)));
                    Ok(r)
                },
                Err(e) => Err(e)
            }
        })
}

pub fn eval(e: Expr, ctx: &Context) -> Result<Expr, Error> {
    match e {
        Expr::Sexpr(es) => sexpr(es, ctx),
        Expr::Atom(a) => atom(a, ctx),
        Expr::Qexpr(qe) => Ok(*qe)
    }
}
