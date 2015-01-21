use builtin;

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone)]
pub enum Expr {
    Sexpr(Vec<Expr>),
    Qexpr(Box<Expr>),
    Atom(Atom)
}

type Builtin = fn(Vec<Expr>, Rc<RefCell<Context>>) -> Result<Expr, Error>;

#[derive(Clone)]
pub enum Func {
    Builtin(Builtin),
    Lambda(Vec<String>, Box<Expr>)
}

#[derive(Clone)]
pub struct Context {
    pub table: HashMap<String, Expr>,
    pub parent: Option<Rc<RefCell<Context>>>
}

impl Context {
    pub fn global() -> Context {
        let mut tbl = HashMap::new();

        tbl.insert("+".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::add))));
        tbl.insert("-".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::sub))));
        tbl.insert("*".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::mul))));
        tbl.insert("/".to_string(),      Expr::Atom(Atom::Fun(Func::Builtin(builtin::div))));
        tbl.insert("def".to_string(),    Expr::Atom(Atom::Fun(Func::Builtin(builtin::def))));
        tbl.insert("list".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::list))));
        tbl.insert("tail".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::tail))));
        tbl.insert("head".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::head))));
        tbl.insert("eval".to_string(),   Expr::Atom(Atom::Fun(Func::Builtin(builtin::eval))));
        tbl.insert("lambda".to_string(), Expr::Atom(Atom::Fun(Func::Builtin(builtin::lambda))));

        Context { table: tbl, parent: None }
    }
}

#[derive(Clone)]
pub enum Atom {
    Int(i64),
    Sym(String),
    Fun(Func)
}

impl fmt::String for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Atom::Int(ref v) => write!(f, "{}", v),
            Atom::Sym(ref v) => write!(f, "{}", v),
            Atom::Fun(_) => write!(f, "<function>")
        }
    }
}

impl fmt::String for Expr {
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
#[derive(Show)]
pub enum Error {
    Runtime(String),
    ZeroDivision,
    NameResolution(String),
    InvalidType(String),
    Arity(String)
}

fn lookup(name: &str, ctx: Rc<RefCell<Context>>) -> Option<Expr> {
    match (ctx.borrow().table.get(name), ctx.borrow().parent.clone()) {
        (Some(e), _) => Some((*e).clone()),
        (None, Some(p)) => lookup(name, p),
        _ => None
    }
}

fn bind(formals: Vec<String>, args: Vec<Expr>, parent: Rc<RefCell<Context>>) -> Result<Context, Error> {
    if formals.len() == args.len() {
        let bindings = formals
            .iter()
            .zip(args.iter())
            .fold(
                HashMap::new(),
                |mut m, (name, val)| {
                    m.insert(name.clone(), val.clone());
                    m
                });

        Ok(Context { table: bindings, parent: Some(parent) })
    } else {
        Err(Error::Arity(format!("expected {} args, but received {}", formals.len(), args.len())))
    }
}

fn call(fun: Atom, xs: &[Expr], ctx: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match fun {
        Atom::Fun(Func::Builtin(b)) => b(xs.to_vec(), ctx.clone()),

        Atom::Fun(Func::Lambda(fs, b)) => {
            let binding = try!(bind(fs, xs.to_vec(), ctx.clone()));

            eval((*b).clone(), Rc::new(RefCell::new(binding)))
        },

        _ => Err(Error::InvalidType(format!("`{}` not a function", fun)))
    }
}

pub fn sexpr(l: Vec<Expr>, ctx: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    let es = try!(eval_all(l.as_slice(), ctx.clone()));

    match es.as_slice() {
        [] => Ok(Expr::Sexpr(Vec::new())),

        [Expr::Atom(ref a), xs..] => call((*a).clone(), xs, ctx.clone()),

        [ref e, _..] => Err(Error::InvalidType(format!("`{}` not a function", e)))
    }
}

pub fn atom(a: Atom, ctx: Rc<RefCell<Context>>) -> Result<Expr, Error> {
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

fn eval_all(list: &[Expr], ctx: Rc<RefCell<Context>>) -> Result<Vec<Expr>, Error> {
    list.iter()
        .fold(Ok::<Vec<Expr>, Error>(Vec::new()), |m, e| {
            match m {
                Ok(mut r) => {
                    r.push(try!(eval(e.clone(), ctx.clone())));
                    Ok(r)
                },
                Err(e) => Err(e)
            }
        })
}

pub fn eval(e: Expr, ctx: Rc<RefCell<Context>>) -> Result<Expr, Error> {
    match e {
        Expr::Sexpr(es) => sexpr(es, ctx),
        Expr::Atom(a) => atom(a, ctx),
        Expr::Qexpr(qe) => Ok(*qe)
    }
}
