use std::fmt;
#[deriving(Clone)]
pub enum Expr {
    Sexpr(Vec<Expr>),
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
            Atom::Sym(ref v) => write!(f, "{}", v)
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
            }
        }
    }
}

#[allow(dead_code)]
#[deriving(Show)]
pub enum Error {
    ZeroDivision,
    NameResolution(String),
    InvalidType(String)
}

fn arith(op: &str, l: Expr, r: Expr) -> Result<Expr, Error> {
    match (eval(l), eval(r)) {
        (Ok(Expr::Atom(Atom::Int(i1))), Ok(Expr::Atom(Atom::Int(i2)))) => {
            match op {
                "+" => Ok(Expr::Atom(Atom::Int(i1 + i2))),
                "-" => Ok(Expr::Atom(Atom::Int(i1 - i2))),
                "*" => Ok(Expr::Atom(Atom::Int(i1 * i2))),
                "/" => {
                    match i2 {
                        0 => Err(Error::ZeroDivision),
                        _ => Ok(Expr::Atom(Atom::Int(i1 / i2)))
                    }
                }
                _ => Err(Error::NameResolution(format!("`{}` is not an arithmetic operator", op.to_string())))
            }
        },
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
        (Ok(Expr::Atom(Atom::Int(_))), nan) => Err(Error::InvalidType(format!("`{}` is not a number", nan.unwrap()))),
        (nan, Ok(Expr::Atom(Atom::Int(_)))) => Err(Error::InvalidType(format!("`{}` is not a number", nan.unwrap()))),
        (nan1, _) => Err(Error::InvalidType(format!("`{}` is not a number", nan1.unwrap())))
    }
}

fn call(func: &str, args: &[Expr]) -> Result<Expr, Error> {
    match func {
        "+" | "-" | "*" | "/" => {
            match args {
                [] => Ok(Expr::Atom(Atom::Int(0))),

                [ref l] => {
                    arith(func, Expr::Atom(Atom::Int(0)), l.clone())
                },

                [ref x, xs..] => {
                    xs.iter().fold(Ok(x.clone()), |m, r| {
                        match m {
                            Ok(l) => arith(func, l, r.clone()),
                            Err(e) => Err(e)
                        }
                    })
                },
            }
        }
        _ => Err(Error::NameResolution(format!("`{}` not in current context", func.to_string())))
    }
}

pub fn sexpr(l: Vec<Expr>) -> Result<Expr, Error> {
    match l.as_slice() {
        [] => Ok(Expr::Sexpr(Vec::new())),

        [Expr::Atom(Atom::Sym(ref func))] => call(func.as_slice(), &[]),

        [Expr::Atom(Atom::Sym(ref func)), xs..] => {
            call(func.as_slice(), xs)
        },

        [ref e, xs..] => Err(Error::InvalidType(format!("`{}` not a function", e)))
    }
}

pub fn eval(e: Expr) -> Result<Expr, Error> {
    match e {
        Expr::Sexpr(es) => sexpr(es),
        Expr::Atom(a) => Ok(Expr::Atom(a))
    }
}
