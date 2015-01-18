use builtin;
use std::fmt;
use std::collections::HashMap;

#[deriving(Clone)]
pub enum Expr {
    Sexpr(Vec<Expr>),
    Qexpr(Box<Expr>),
    Atom(Atom)
}

type Builtin = fn(Vec<Expr>, Context) -> Result<Expr, Error>;

#[deriving(Clone)]
pub enum Func {
    Builtin(Builtin),
    Lambda(Vec<String>, Box<Expr>)
}

#[deriving(Clone)]
pub struct Context {
    table: HashMap<String, Expr>,
    parent: Option<Box<Context>>
}

impl Context {
    pub fn global() -> Context {
        let mut tbl = HashMap::new();

        tbl.insert("+".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::add))));
        tbl.insert("-".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::sub))));
        tbl.insert("*".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::mul))));
        tbl.insert("/".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::div))));
        tbl.insert("list".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::list))));
        tbl.insert("tail".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::tail))));
        tbl.insert("head".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::head))));
        tbl.insert("eval".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::eval))));
        tbl.insert("lambda".to_string(), Expr::Atom(Atom::Fun(Func::Builtin(builtin::lambda))));

        Context { table: tbl, parent: None }
    }
}

#[deriving(Clone)]
pub enum Atom {
    Int(int),
    Sym(String),
    Fun(Func)
}

impl fmt::Show for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Atom::Int(ref v) => write!(f, "{}", v),
            Atom::Sym(ref v) => write!(f, "{}", v),
            Atom::Fun(_) => write!(f, "<function>")
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
    match (ctx.table.get(name), (*ctx).parent.clone()) {
        (Some(e), _) => Some(e.clone()),
        (None, Some(p)) => lookup(name, &*p),
        _ => None
    }
}

fn call_lambda(formals: Vec<String>, body: &Expr, args: Vec<Expr>, ctx: &Context) -> Result<Expr, Error> {
    if formals.len() != args.len() {
        Err(Error::Arity(format!("expected {} args, found {}", formals.len(), args.len())))
    } else {
        let cctx = (*ctx).clone();
        let bindings = formals.iter().zip(args.iter());

        let bmap = bindings
            .fold(
                HashMap::new(),
                |mut m, (name, val)| {
                    m.insert(name.clone(), val.clone());
                    m
                });

        let fctx = Context {
            table: bmap,
            parent: Some(box cctx)
        };


        eval((*body).clone(), &fctx)
    }
}

fn call(fun: Atom, xs: &[Expr], ctx: &Context) -> Result<Expr, Error> {
    match fun {
        Atom::Sym(ref name) => {
            match lookup(name.as_slice(), ctx) {
                Some(Expr::Atom(Atom::Fun(Func::Builtin(b)))) => b(xs.to_vec(), (*ctx).clone()),
                Some(Expr::Atom(Atom::Fun(Func::Lambda(fs, b)))) => call_lambda(fs, &*b, xs.to_vec(), ctx),
                None => Err(Error::NameResolution(format!("`{}` not in current context", name.to_string()))),
                _ => Err(Error::InvalidType(format!("`{}` not a function", fun)))
            }
        },

        Atom::Fun(Func::Builtin(b)) => b(xs.to_vec(), (*ctx).clone()),

        Atom::Fun(Func::Lambda(fs, b)) => call_lambda(fs, &*b, xs.to_vec(), ctx),

        _ => Err(Error::InvalidType(format!("`{}` not a function", fun)))
    }
}

pub fn sexpr(l: Vec<Expr>, ctx: &Context) -> Result<Expr, Error> {
    let es = try!(eval_all(l.as_slice(), ctx));

    match es.as_slice() {
        [] => Ok(Expr::Sexpr(Vec::new())),

        [Expr::Atom(ref a), xs..] => call((*a).clone(), xs, ctx),

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
