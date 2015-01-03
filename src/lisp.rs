use builtin;
use std::fmt;

#[deriving(Clone)]
pub enum Expr {
    Sexpr(Vec<Expr>),
    Qexpr(Box<Expr>),
    Atom(Atom)
}

#[deriving(Clone)]
pub enum Atom {
    Int(int),
    Sym(String)
}

impl fmt::Show for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Atom::Int(ref v) => write!(f, "{}", v),
            Atom::Sym(ref v) => write!(f, "{}", v),
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

fn call(func: &str, args: &[Expr]) -> Result<Expr, Error> {
    let e = try!(eval_all(args));
    let eargs = e.as_slice();

    match func {
        "+" => builtin::add(eargs),
        "-" => builtin::sub(eargs),
        "*" => builtin::mul(eargs),
        "/" => builtin::div(eargs),
        "list" => builtin::list(eargs),
        "tail" => builtin::tail(eargs),
        "head" => builtin::head(eargs),
        "eval" => builtin::eval(eargs),
        _ => Err(Error::NameResolution(format!("`{}` not in current context", func.to_string())))
    }
}

pub fn sexpr(l: Vec<Expr>) -> Result<Expr, Error> {
    match l.as_slice() {
        [] => Ok(Expr::Sexpr(Vec::new())),

        [Expr::Atom(Atom::Sym(ref func)), xs..] => {
            call(func.as_slice(), xs)
        },

        [ref e, _..] => Err(Error::InvalidType(format!("`{}` not a function", e)))
    }
}

fn eval_all(list: &[Expr]) -> Result<Vec<Expr>, Error> {
    list.iter()
        .fold(Ok::<Vec<Expr>, Error>(Vec::new()), |m, e| {
            match m {
                Ok(mut r) => {
                    r.push(try!(eval(e.clone())));
                    Ok(r)
                },
                Err(e) => Err(e)
            }
        })
}

pub fn eval(e: Expr) -> Result<Expr, Error> {
    match e {
        Expr::Sexpr(es) => sexpr(es),
        Expr::Atom(a) => Ok(Expr::Atom(a)),
        Expr::Qexpr(qe) => Ok(*qe)
    }
}
